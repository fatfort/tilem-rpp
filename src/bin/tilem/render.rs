use crate::keymap::Key;
use crate::rmpp_hal::display::{QtfbDisplay, RMPP_HEIGHT, RMPP_WIDTH};
use crate::rmpp_hal::types::{color, mxcfb_rect};
use cgmath::{Point2, Vector2};

pub const DISPLAY_W: u32 = RMPP_WIDTH;
pub const DISPLAY_H: u32 = RMPP_HEIGHT;

// LCD region (TI-84+: 96x64 monochrome; scale up ~8x to 768x512).
pub const LCD_SCALE: u32 = 8;
pub const LCD_NATIVE_W: u32 = 96;
pub const LCD_NATIVE_H: u32 = 64;
pub const LCD_W: u32 = LCD_NATIVE_W * LCD_SCALE;
pub const LCD_H: u32 = LCD_NATIVE_H * LCD_SCALE;

pub const HEADER_H: u32 = 80;
pub const LCD_X: u32 = (DISPLAY_W - LCD_W) / 2;
pub const LCD_Y: u32 = HEADER_H + 16;

pub const KEYPAD_X: u32 = 10;
pub const KEYPAD_Y: u32 = LCD_Y + LCD_H + 24;
pub const KEYPAD_W: u32 = DISPLAY_W - 2 * KEYPAD_X;
pub const KEYPAD_H: u32 = DISPLAY_H - KEYPAD_Y - 16;

pub fn draw_chrome(display: &mut QtfbDisplay, keymap: &[Vec<Key>]) {
    display.clear();

    // Header band.
    display.fill_rect(
        Point2 { x: 0, y: 0 },
        Vector2 {
            x: DISPLAY_W,
            y: HEADER_H,
        },
        color::HEADER_BG,
    );
    display.draw_text(
        Point2 { x: 24.0, y: 18.0 },
        "TilEm — TI-84 Plus",
        40.0,
        color::WHITE,
        false,
    );

    // LCD frame (black border).
    display.draw_rect(
        Point2 {
            x: LCD_X as i32 - 4,
            y: LCD_Y as i32 - 4,
        },
        Vector2 {
            x: LCD_W + 8,
            y: LCD_H + 8,
        },
        2,
        color::BLACK,
    );

    draw_keypad_static(display, keymap);
}

pub fn draw_keypad_static(display: &mut QtfbDisplay, keymap: &[Vec<Key>]) {
    let rects = crate::keymap::layout(
        keymap,
        KEYPAD_X as i32,
        KEYPAD_Y as i32,
        KEYPAD_W as i32,
        KEYPAD_H as i32,
    );

    // Paint keypad background white so leftover artifacts get cleared.
    display.fill_rect(
        Point2 {
            x: KEYPAD_X as i32,
            y: KEYPAD_Y as i32,
        },
        Vector2 {
            x: KEYPAD_W,
            y: KEYPAD_H,
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
pub fn draw_lcd(display: &mut QtfbDisplay, frame: &[u8], fw: u32, fh: u32, contrast: u8) -> mxcfb_rect {
    let sx = LCD_X as i32;
    let sy = LCD_Y as i32;

    // contrast==0 means the LCD is off (calc halted / not yet turned on).  The
    // frame buffer returned by tilem is malloc'd (not calloc'd) and tilem does
    // not fill it in the off-state, so rendering it shows uninitialized noise.
    // Paint white — an asleep calculator has a blank screen.
    if contrast == 0 || frame.is_empty() || fw == 0 || fh == 0 {
        display.fill_rect(
            Point2 { x: sx, y: sy },
            Vector2 { x: LCD_W, y: LCD_H },
            color::WHITE,
        );
        return lcd_rect();
    }

    let scale = LCD_SCALE as i32;
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
    mxcfb_rect {
        top: LCD_Y,
        left: LCD_X,
        width: LCD_W,
        height: LCD_H,
    }
}
