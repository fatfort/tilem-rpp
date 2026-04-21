use crate::keymap::Key;
use crate::rmpp_hal::display::QtfbDisplay;
use crate::rmpp_hal::types::{color, mxcfb_rect};
use cgmath::{Point2, Vector2};
use std::sync::OnceLock;

pub const LCD_NATIVE_W: u32 = 96;
pub const LCD_NATIVE_H: u32 = 64;

/// All pixel-space dimensions, computed once at startup from the actual
/// display size reported by the HAL.  The reference units are the Move's
/// 954×1696; ferrari scales up ~1.698× by width ratio.
pub struct Layout {
    pub display_w: u32,
    pub display_h: u32,
    pub lcd_scale: u32,
    pub lcd_w: u32,
    pub lcd_h: u32,
    pub header_h: u32,
    pub lcd_x: u32,
    pub lcd_y: u32,
    pub keypad_x: u32,
    pub keypad_y: u32,
    pub keypad_w: u32,
    pub keypad_h: u32,
    pub header_font: f32,
    pub ui_scale: f32,
}

impl Layout {
    fn new(display_w: u32, display_h: u32) -> Self {
        let ui_scale = display_w as f32 / 954.0;

        // LCD: scale 96×64 native monochrome up to a readable block.
        // Move: 8× → 768×512.  Ferrari: 14× → 1344×896.
        let lcd_scale = ((8.0 * ui_scale).round() as u32).max(4);
        let lcd_w = LCD_NATIVE_W * lcd_scale;
        let lcd_h = LCD_NATIVE_H * lcd_scale;

        let header_h = ((80.0 * ui_scale).round() as u32).max(40);
        let lcd_x = display_w.saturating_sub(lcd_w) / 2;
        let lcd_y = header_h + (16.0 * ui_scale).round() as u32;
        let keypad_x = (10.0 * ui_scale).round() as u32;
        let keypad_y = lcd_y + lcd_h + (24.0 * ui_scale).round() as u32;
        let keypad_w = display_w.saturating_sub(2 * keypad_x);
        let keypad_h = display_h
            .saturating_sub(keypad_y)
            .saturating_sub((16.0 * ui_scale).round() as u32);
        let header_font = 40.0 * ui_scale;

        Self {
            display_w,
            display_h,
            lcd_scale,
            lcd_w,
            lcd_h,
            header_h,
            lcd_x,
            lcd_y,
            keypad_x,
            keypad_y,
            keypad_w,
            keypad_h,
            header_font,
            ui_scale,
        }
    }
}

static LAYOUT: OnceLock<Layout> = OnceLock::new();

pub fn init(display_w: u32, display_h: u32) -> &'static Layout {
    LAYOUT.get_or_init(|| Layout::new(display_w, display_h))
}

pub fn layout() -> &'static Layout {
    LAYOUT
        .get()
        .expect("render::init must be called before layout()")
}

pub fn draw_chrome(display: &mut QtfbDisplay, keymap: &[Vec<Key>]) {
    let l = layout();
    display.clear();

    // Header band.
    display.fill_rect(
        Point2 { x: 0, y: 0 },
        Vector2 {
            x: l.display_w,
            y: l.header_h,
        },
        color::HEADER_BG,
    );
    display.draw_text(
        Point2 {
            x: 24.0 * l.ui_scale,
            y: 18.0 * l.ui_scale,
        },
        "TilEm — TI-84 Plus",
        l.header_font,
        color::WHITE,
        false,
    );

    // LCD frame (black border).
    display.draw_rect(
        Point2 {
            x: l.lcd_x as i32 - 4,
            y: l.lcd_y as i32 - 4,
        },
        Vector2 {
            x: l.lcd_w + 8,
            y: l.lcd_h + 8,
        },
        2,
        color::BLACK,
    );

    draw_keypad_static(display, keymap);
}

pub fn draw_keypad_static(display: &mut QtfbDisplay, keymap: &[Vec<Key>]) {
    let l = layout();
    let rects = crate::keymap::layout(
        keymap,
        l.keypad_x as i32,
        l.keypad_y as i32,
        l.keypad_w as i32,
        l.keypad_h as i32,
    );

    // Paint keypad background white so leftover artifacts get cleared.
    display.fill_rect(
        Point2 {
            x: l.keypad_x as i32,
            y: l.keypad_y as i32,
        },
        Vector2 {
            x: l.keypad_w,
            y: l.keypad_h,
        },
        color::WHITE,
    );

    for &(scancode, x, y, w, h) in &rects {
        // Border.
        display.draw_rect(
            Point2 { x, y },
            Vector2 {
                x: w as u32,
                y: h as u32,
            },
            2,
            color::BLACK,
        );

        // Find this key's labels in the keymap.
        let key = find_key(keymap, scancode);
        if let Some(k) = key {
            draw_key_labels(display, x, y, w, h, k);
        }
    }
}

fn find_key(keymap: &[Vec<Key>], scancode: i32) -> Option<&Key> {
    for row in keymap {
        for k in row {
            if k.scancode == scancode {
                return Some(k);
            }
        }
    }
    None
}

fn draw_key_labels(display: &mut QtfbDisplay, x: i32, y: i32, w: i32, h: i32, key: &Key) {
    let front_size = (h as f32 * 0.38).min(w as f32 * 0.34);
    let upper_size = (h as f32 * 0.22).min(w as f32 * 0.22);

    // Estimate front-text width with a dry run and center it.
    let front_rect = display.draw_text(
        Point2 { x: 0.0, y: 0.0 },
        key.front,
        front_size,
        color::BLACK,
        true,
    );
    let front_x = x + (w - front_rect.width as i32) / 2;
    let front_y = y + (h as f32 * 0.42) as i32;
    display.draw_text(
        Point2 {
            x: front_x as f32,
            y: front_y as f32,
        },
        key.front,
        front_size,
        color::BLACK,
        false,
    );

    if !key.shift.is_empty() {
        let r = display.draw_text(
            Point2 { x: 0.0, y: 0.0 },
            key.shift,
            upper_size,
            color::MEDIUM_GRAY,
            true,
        );
        let sx = x + 4;
        let sy = y + 4;
        if r.width > 0 {
            display.draw_text(
                Point2 {
                    x: sx as f32,
                    y: sy as f32,
                },
                key.shift,
                upper_size,
                color::MEDIUM_GRAY,
                false,
            );
        }
    }
    if !key.alpha.is_empty() {
        let r = display.draw_text(
            Point2 { x: 0.0, y: 0.0 },
            key.alpha,
            upper_size,
            color::DARK_GRAY,
            true,
        );
        let ax = x + w - r.width as i32 - 4;
        let ay = y + 4;
        display.draw_text(
            Point2 {
                x: ax as f32,
                y: ay as f32,
            },
            key.alpha,
            upper_size,
            color::DARK_GRAY,
            false,
        );
    }
}

/// Render the 96x64 LCD buffer into the on-screen LCD rectangle.
/// `frame` is W*H bytes (0 = off).  Binary render (ON=black, OFF=white)
/// looks sharpest on e-ink and sidesteps the contrast=0 black-out that the
/// rM2 port had — a fresh ROM boots at contrast 0 and the user would
/// otherwise see a solid black square.
pub fn draw_lcd(
    display: &mut QtfbDisplay,
    frame: &[u8],
    fw: u32,
    fh: u32,
    contrast: u8,
) -> mxcfb_rect {
    let l = layout();
    let sx = l.lcd_x as i32;
    let sy = l.lcd_y as i32;

    // contrast==0 means the LCD is off (calc halted / not yet turned on).  The
    // frame buffer returned by tilem is malloc'd (not calloc'd) and tilem does
    // not fill it in the off-state, so rendering it shows uninitialized noise.
    // Paint white — an asleep calculator has a blank screen.
    if contrast == 0 || frame.is_empty() || fw == 0 || fh == 0 {
        display.fill_rect(
            Point2 { x: sx, y: sy },
            Vector2 {
                x: l.lcd_w,
                y: l.lcd_h,
            },
            color::WHITE,
        );
        return lcd_rect();
    }

    let scale = l.lcd_scale as i32;
    for py in 0..fh as i32 {
        for px in 0..fw as i32 {
            let v = frame[(py as usize) * (fw as usize) + px as usize];
            let c = if v != 0 { color::BLACK } else { color::WHITE };
            display.fill_rect(
                Point2 {
                    x: sx + px * scale,
                    y: sy + py * scale,
                },
                Vector2 {
                    x: scale as u32,
                    y: scale as u32,
                },
                c,
            );
        }
    }
    lcd_rect()
}

/// Fill a single key rect with a "pressed" look and return its bounds for
/// partial refresh.  Called while the user's finger is on the key.
pub fn draw_key_pressed(display: &mut QtfbDisplay, rect: (i32, i32, i32, i32, i32)) -> mxcfb_rect {
    let (_, x, y, w, h) = rect;
    display.fill_rect(
        Point2 { x, y },
        Vector2 {
            x: w as u32,
            y: h as u32,
        },
        color::MEDIUM_GRAY,
    );
    display.draw_rect(
        Point2 { x, y },
        Vector2 {
            x: w as u32,
            y: h as u32,
        },
        3,
        color::BLACK,
    );
    mxcfb_rect {
        top: y.max(0) as u32,
        left: x.max(0) as u32,
        width: w as u32,
        height: h as u32,
    }
}

/// Repaint a single key in its idle state.  Uses the keymap to find its
/// labels.  Returns the rect for partial refresh.
pub fn draw_key_idle(
    display: &mut QtfbDisplay,
    keymap: &[Vec<crate::keymap::Key>],
    rect: (i32, i32, i32, i32, i32),
) -> mxcfb_rect {
    let (sc, x, y, w, h) = rect;
    display.fill_rect(
        Point2 { x, y },
        Vector2 {
            x: w as u32,
            y: h as u32,
        },
        color::WHITE,
    );
    display.draw_rect(
        Point2 { x, y },
        Vector2 {
            x: w as u32,
            y: h as u32,
        },
        2,
        color::BLACK,
    );
    if let Some(k) = find_key(keymap, sc) {
        draw_key_labels(display, x, y, w, h, k);
    }
    mxcfb_rect {
        top: y.max(0) as u32,
        left: x.max(0) as u32,
        width: w as u32,
        height: h as u32,
    }
}

pub fn lcd_rect() -> mxcfb_rect {
    let l = layout();
    mxcfb_rect {
        top: l.lcd_y,
        left: l.lcd_x,
        width: l.lcd_w,
        height: l.lcd_h,
    }
}
