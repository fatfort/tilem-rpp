use crate::emu::Emulator;
use crate::keymap::{self, Key};
use crate::render;
use crate::rmpp_hal::display::QtfbDisplay;
use crate::rmpp_hal::types::{InputEvent, MultitouchEvent, mxcfb_rect};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

/// Minimum real-time the emulator must see a key "held" before we release it.
/// Fast taps would otherwise land both Press and Release in the same pump()
/// call with zero z80 cycles between them, so the OS never polls the matrix
/// while the bit is set.  50 ms is comfortably above the keypad scan period.
const MIN_HOLD: Duration = Duration::from_millis(50);

struct Held {
    scancode: i32,
    rect: (i32, i32, i32, i32, i32),
    press_at: Instant,
}

pub struct TouchRouter {
    key_rects: Vec<(i32, i32, i32, i32, i32)>,
    active: HashMap<i32, Held>,
    /// Releases that were deferred because the tap was shorter than MIN_HOLD.
    /// Stored as (due_at, scancode, rect).
    pending_releases: Vec<(Instant, i32, (i32, i32, i32, i32, i32))>,
}

impl TouchRouter {
    pub fn new(key_rects: Vec<(i32, i32, i32, i32, i32)>) -> Self {
        Self {
            key_rects,
            active: HashMap::new(),
            pending_releases: Vec::new(),
        }
    }

    pub fn pump(
        &mut self,
        rx: &Receiver<InputEvent>,
        emu: &mut Emulator,
        display: &mut QtfbDisplay,
        keymap: &[Vec<Key>],
    ) -> Vec<mxcfb_rect> {
        let mut dirty: Vec<mxcfb_rect> = Vec::new();

        // Flush any deferred releases first so the next press is free.
        self.flush_pending_releases(emu, display, keymap, &mut dirty);

        loop {
            match rx.try_recv() {
                Ok(InputEvent::MultitouchEvent { event }) => {
                    self.on_event(event, emu, display, &mut dirty);
                }
                Ok(_) => {}
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            }
        }
        dirty
    }

    fn on_event(
        &mut self,
        ev: MultitouchEvent,
        emu: &mut Emulator,
        display: &mut QtfbDisplay,
        dirty: &mut Vec<mxcfb_rect>,
    ) {
        match ev {
            MultitouchEvent::Press { finger } => {
                let px = finger.pos.x as i32;
                let py = finger.pos.y as i32;
                let hit = self
                    .key_rects
                    .iter()
                    .copied()
                    .find(|&(_, x, y, w, h)| px >= x && px < x + w && py >= y && py < y + h);
                match hit {
                    Some(rect) => {
                        let (sc, _, _, _, _) = rect;
                        log::debug!(
                            "touch press ({},{}) -> scancode 0x{:02x} id={}",
                            px,
                            py,
                            sc,
                            finger.tracking_id
                        );
                        emu.press_key(sc);
                        dirty.push(render::draw_key_pressed(display, rect));
                        self.active.insert(
                            finger.tracking_id,
                            Held {
                                scancode: sc,
                                rect,
                                press_at: Instant::now(),
                            },
                        );
                    }
                    None => {
                        log::debug!("touch press ({},{}) -> miss id={}", px, py, finger.tracking_id);
                    }
                }
            }
            MultitouchEvent::Release { finger } => {
                if let Some(h) = self.active.remove(&finger.tracking_id) {
                    let now = Instant::now();
                    let elapsed = now.duration_since(h.press_at);
                    if elapsed < MIN_HOLD {
                        let due = h.press_at + MIN_HOLD;
                        log::debug!(
                            "touch release id={} scancode 0x{:02x} deferred +{}ms",
                            finger.tracking_id,
                            h.scancode,
                            (due - now).as_millis()
                        );
                        self.pending_releases.push((due, h.scancode, h.rect));
                    } else {
                        log::debug!(
                            "touch release id={} scancode 0x{:02x} immediate",
                            finger.tracking_id,
                            h.scancode
                        );
                        emu.release_key(h.scancode);
                        // Immediate redraw of the key in idle state is deferred
                        // to flush_pending_releases so that the chrome-repaint
                        // code path is exercised on every release.
                        self.pending_releases.push((now, h.scancode, h.rect));
                    }
                }
            }
            MultitouchEvent::Move { .. } => {
                // Ignore drags; the calc treats press/release atomically.
                let _ = keymap::hit; // keep import alive
            }
        }
    }

    fn flush_pending_releases(
        &mut self,
        emu: &mut Emulator,
        display: &mut QtfbDisplay,
        keymap: &[Vec<Key>],
        dirty: &mut Vec<mxcfb_rect>,
    ) {
        let now = Instant::now();
        let mut i = 0;
        while i < self.pending_releases.len() {
            if now >= self.pending_releases[i].0 {
                let (_, sc, rect) = self.pending_releases.swap_remove(i);
                log::debug!("deferred release fired: scancode 0x{:02x}", sc);
                emu.release_key(sc);
                dirty.push(render::draw_key_idle(display, keymap, rect));
            } else {
                i += 1;
            }
        }
    }
}
