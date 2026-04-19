#![allow(dead_code)]

use crate::emu::ffi;

/// A single on-screen key.  `width` is in "column units"; most keys are 1,
/// some rows have half-width filler (scancode=0) around the arrow cluster.
#[derive(Clone)]
pub struct Key {
    pub scancode: i32,
    pub front: &'static str,
    pub shift: &'static str,
    pub alpha: &'static str,
    pub width: f32,
}

const fn k(scancode: u32, front: &'static str, shift: &'static str, alpha: &'static str) -> Key {
    Key {
        scancode: scancode as i32,
        front,
        shift,
        alpha,
        width: 1.0,
    }
}

const fn kw(
    scancode: u32,
    front: &'static str,
    shift: &'static str,
    alpha: &'static str,
    width: f32,
) -> Key {
    Key {
        scancode: scancode as i32,
        front,
        shift,
        alpha,
        width,
    }
}

/// Cribbed from rm2-stuff-ref/apps/tilem/main.cpp — 11 rows, 5 wide.
pub fn keymap() -> Vec<Vec<Key>> {
    use ffi::*;
    vec![
        vec![
            k(TILEM_KEY_YEQU, "Y=", "STATPLOT", "F1"),
            k(TILEM_KEY_WINDOW, "WINDOW", "TBLST", "F2"),
            k(TILEM_KEY_ZOOM, "ZOOM", "FORMAT", "F3"),
            k(TILEM_KEY_TRACE, "TRACE", "CALC", "F4"),
            k(TILEM_KEY_GRAPH, "GRAPH", "TABLE", "F5"),
        ],
        vec![
            kw(0, "", "", "", 3.5),
            kw(TILEM_KEY_UP, "^", "", "", 1.0),
        ],
        vec![
            k(TILEM_KEY_2ND, "2ND", "", ""),
            k(TILEM_KEY_MODE, "MODE", "QUIT", ""),
            k(TILEM_KEY_DEL, "DEL", "INS", ""),
            k(TILEM_KEY_LEFT, "<", "", ""),
            k(TILEM_KEY_RIGHT, ">", "", ""),
        ],
        vec![
            k(TILEM_KEY_ALPHA, "ALPHA", "A-LOCK", ""),
            k(TILEM_KEY_GRAPHVAR, "X,T,θ,n", "LINK", ""),
            k(TILEM_KEY_STAT, "STAT", "LIST", ""),
            kw(0, "", "", "", 0.5),
            kw(TILEM_KEY_DOWN, "v", "", "", 1.0),
        ],
        vec![
            k(TILEM_KEY_MATH, "MATH", "TEST", "A"),
            k(TILEM_KEY_MATRIX, "APPS", "ANGLE", "B"),
            k(TILEM_KEY_PRGM, "PRGM", "DRAW", "C"),
            k(TILEM_KEY_VARS, "VARS", "DISTR", ""),
            k(TILEM_KEY_CLEAR, "CLEAR", "", ""),
        ],
        vec![
            k(TILEM_KEY_RECIP, "x^-1", "MATRIX", "D"),
            k(TILEM_KEY_SIN, "SIN", "SIN-1", "E"),
            k(TILEM_KEY_COS, "COS", "COS-1", "F"),
            k(TILEM_KEY_TAN, "TAN", "TAN-1", "G"),
            k(TILEM_KEY_POWER, "^", "π", "H"),
        ],
        vec![
            k(TILEM_KEY_SQUARE, "x²", "√", "I"),
            k(TILEM_KEY_COMMA, ",", "EE", "J"),
            k(TILEM_KEY_LPAREN, "(", "{", "K"),
            k(TILEM_KEY_RPAREN, ")", "}", "L"),
            k(TILEM_KEY_DIV, "/", "e", "M"),
        ],
        vec![
            k(TILEM_KEY_LOG, "LOG", "10^x", "N"),
            k(TILEM_KEY_7, "7", "u", "O"),
            k(TILEM_KEY_8, "8", "v", "P"),
            k(TILEM_KEY_9, "9", "w", "Q"),
            k(TILEM_KEY_MUL, "*", "[", "R"),
        ],
        vec![
            k(TILEM_KEY_LN, "LN", "e^x", "S"),
            k(TILEM_KEY_4, "4", "L4", "T"),
            k(TILEM_KEY_5, "5", "L5", "U"),
            k(TILEM_KEY_6, "6", "L6", "V"),
            k(TILEM_KEY_SUB, "-", "]", "W"),
        ],
        vec![
            k(TILEM_KEY_STORE, "STO>", "RCL", "X"),
            k(TILEM_KEY_1, "1", "L1", "Y"),
            k(TILEM_KEY_2, "2", "L2", "Z"),
            k(TILEM_KEY_3, "3", "L3", "θ"),
            k(TILEM_KEY_ADD, "+", "MEM", "\""),
        ],
        vec![
            k(TILEM_KEY_ON, "ON", "OFF", ""),
            k(TILEM_KEY_0, "0", "CATALOG", "_"),
            k(TILEM_KEY_DECPNT, ".", "i", ":"),
            k(TILEM_KEY_CHS, "(-)", "ANS", "?"),
            k(TILEM_KEY_ENTER, "ENTER", "ENTRY", "SOLVE"),
        ],
    ]
}

/// Compute per-key rects for a keypad region of the given pixel bounds.
pub fn layout(
    keymap: &[Vec<Key>],
    origin_x: i32,
    origin_y: i32,
    area_w: i32,
    area_h: i32,
) -> Vec<(i32, i32, i32, i32, i32)> {
    let num_rows = keymap.len() as i32;
    let max_cols = keymap
        .iter()
        .map(|r| r.iter().map(|k| k.width).sum::<f32>().ceil() as i32)
        .max()
        .unwrap_or(5);
    let key_w = area_w / max_cols;
    let key_h = area_h / num_rows;

    let mut out = Vec::new();
    let mut y = origin_y;
    for row in keymap {
        let mut x = origin_x;
        for key in row {
            let w = (key_w as f32 * key.width) as i32;
            if key.scancode != 0 {
                out.push((key.scancode, x, y, w, key_h));
            }
            x += w;
        }
        y += key_h;
    }
    out
}

pub fn hit(rects: &[(i32, i32, i32, i32, i32)], px: i32, py: i32) -> Option<i32> {
    for &(sc, x, y, w, h) in rects {
        if px >= x && px < x + w && py >= y && py < y + h {
            return Some(sc);
        }
    }
    None
}
