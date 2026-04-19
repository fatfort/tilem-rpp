#![allow(non_camel_case_types, non_snake_case)]

mod emu;
mod input;
mod keymap;
mod render;
mod rmpp_hal;

use crate::emu::Emulator;
use crate::input::TouchRouter;
use crate::rmpp_hal::input::start_input_threads;
use crate::rmpp_hal::types::InputEvent;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

extern "C" fn on_sig(_: libc::c_int) {
    SHUTDOWN.store(true, Ordering::SeqCst);
}

fn setup_signals() {
    unsafe {
        libc::signal(libc::SIGTERM, on_sig as libc::sighandler_t);
        libc::signal(libc::SIGINT, on_sig as libc::sighandler_t);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "INFO") };
    }
    env_logger::init();
    log::info!("TilEm starting on Move");
    setup_signals();

    // ROM + save paths (user must scp a ROM into place; see manifest description).
    let rom_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/root/ti84plus.rom"));
    let save_path = {
        let mut p = rom_path.clone();
        let fname = p
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "ti84plus.rom".to_string());
        p.set_file_name(format!("{}.sav", fname));
        p
    };
    log::info!("ROM: {}", rom_path.display());
    log::info!("Save: {}", save_path.display());

    let mut display = rmpp_hal::display::QtfbDisplay::new();
    let (tx, rx) = std::sync::mpsc::channel::<InputEvent>();
    start_input_threads(tx, display.socket_fd());

    // Load emulator.  If the ROM is missing, draw a diagnostic screen and spin.
    let mut emu = match Emulator::new_ti84p(&rom_path, Some(save_path.as_path())) {
        Ok(e) => e,
        Err(msg) => {
            log::error!("Failed to init emulator: {}", msg);
            draw_error(&mut display, &rom_path, &msg);
            display.full_refresh();
            while !SHUTDOWN.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(250));
            }
            return;
        }
    };

    let km = keymap::keymap();
    let rects = keymap::layout(
        &km,
        render::KEYPAD_X as i32,
        render::KEYPAD_Y as i32,
        render::KEYPAD_W as i32,
        render::KEYPAD_H as i32,
    );
    let mut router = TouchRouter::new(rects);

    render::draw_chrome(&mut display, &km);
    display.full_refresh();
    display.set_refresh_mode(true);

    let mut lcd_buf: Vec<u8> = Vec::with_capacity(96 * 64);
    let mut last_lcd: Vec<u8> = Vec::new();
    let mut last_contrast: u8 = 0xFF;

    let tick = Duration::from_millis(10); // ~100 ticks/sec
    let mut last_tick = Instant::now();
    let mut frame_counter: u32 = 0;

    while !SHUTDOWN.load(Ordering::SeqCst) {
        let now = Instant::now();

        // Input first, so keys affect the current tick.
        let key_dirty = router.pump(&rx, &mut emu, &mut display, &km);
        for r in &key_dirty {
            display.partial_refresh(r);
        }

        // Advance emulator by elapsed wall-clock time (microseconds).
        let mut dt = now.duration_since(last_tick);
        if dt > Duration::from_secs(1) {
            dt = tick;
        }
        let us = dt.as_micros() as i32;
        if us > 0 {
            emu.run_microseconds(us);
        }
        last_tick = now;

        // Redraw LCD every few ticks (~20 fps) if changed.
        frame_counter = frame_counter.wrapping_add(1);
        if frame_counter % 5 == 0 {
            let (fw, fh, contrast) = emu.get_lcd_frame(&mut lcd_buf);
            let changed = contrast != last_contrast || lcd_buf != last_lcd;
            if changed {
                let region = render::draw_lcd(&mut display, &lcd_buf, fw, fh, contrast);
                display.partial_refresh(&region);
                last_lcd.clear();
                last_lcd.extend_from_slice(&lcd_buf);
                last_contrast = contrast;
            }
        }

        let elapsed = now.elapsed();
        if elapsed < tick {
            std::thread::sleep(tick - elapsed);
        }
    }

    log::info!("Shutdown — saving state to {}", save_path.display());
    if let Err(e) = emu.save_state(&save_path) {
        log::warn!("save_state failed: {}", e);
    }
}

fn draw_error(display: &mut rmpp_hal::display::QtfbDisplay, rom_path: &std::path::Path, msg: &str) {
    use crate::rmpp_hal::types::color;
    use cgmath::Point2;
    display.clear();
    display.draw_text(
        Point2 { x: 40.0, y: 80.0 },
        "TilEm — ROM load failed",
        56.0,
        color::BLACK,
        false,
    );
    display.draw_text(
        Point2 { x: 40.0, y: 180.0 },
        &format!("Expected ROM at: {}", rom_path.display()),
        32.0,
        color::DARK_GRAY,
        false,
    );
    display.draw_text(
        Point2 { x: 40.0, y: 240.0 },
        msg,
        28.0,
        color::DARK_GRAY,
        false,
    );
    display.draw_text(
        Point2 { x: 40.0, y: 320.0 },
        "Copy a TI-84+ ROM to /home/root/ti84plus.rom and restart.",
        28.0,
        color::DARK_GRAY,
        false,
    );
}
