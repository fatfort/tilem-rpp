use crate::emu::Emulator;
use crate::keymap;
use crate::rmpp_hal::types::{InputEvent, MultitouchEvent};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

/// Track which scancode is currently held by each touch tracking id
/// so Release on the same id can release the correct key.
pub struct TouchRouter {
    key_rects: Vec<(i32, i32, i32, i32, i32)>,
    active: HashMap<i32, i32>, // tracking_id -> scancode
}

impl TouchRouter {
    pub fn new(key_rects: Vec<(i32, i32, i32, i32, i32)>) -> Self {
        Self {
            key_rects,
            active: HashMap::new(),
        }
    }

    pub fn pump(&mut self, rx: &Receiver<InputEvent>, emu: &mut Emulator) {
        loop {
            match rx.try_recv() {
                Ok(InputEvent::MultitouchEvent { event }) => self.on_event(event, emu),
                Ok(_) => {}
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            }
        }
    }

    fn on_event(&mut self, ev: MultitouchEvent, emu: &mut Emulator) {
        match ev {
            MultitouchEvent::Press { finger } => {
                if let Some(sc) = keymap::hit(
                    &self.key_rects,
                    finger.pos.x as i32,
                    finger.pos.y as i32,
                ) {
                    emu.press_key(sc);
                    self.active.insert(finger.tracking_id, sc);
                }
            }
            MultitouchEvent::Release { finger } => {
                if let Some(sc) = self.active.remove(&finger.tracking_id) {
                    emu.release_key(sc);
                }
            }
            MultitouchEvent::Move { .. } => {
                // Ignore drags; the calc treats press/release atomically.
            }
        }
    }
}
