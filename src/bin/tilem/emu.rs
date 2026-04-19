#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;

pub mod ffi {
    #![allow(
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        dead_code
    )]
    include!(concat!(env!("OUT_DIR"), "/tilem_bindings.rs"));
}

pub const TILEM_CALC_TI84P: c_char = b'4' as c_char;

pub struct Emulator {
    calc: *mut ffi::TilemCalc,
    lcd_buf: *mut ffi::TilemLCDBuffer,
}

unsafe impl Send for Emulator {}

impl Emulator {
    pub fn new_ti84p(rom_path: &Path, save_path: Option<&Path>) -> Result<Self, String> {
        unsafe {
            let calc = ffi::tilem_calc_new(TILEM_CALC_TI84P);
            if calc.is_null() {
                return Err("tilem_calc_new returned NULL".into());
            }

            let rom_c = CString::new(rom_path.as_os_str().to_str().unwrap_or("")).unwrap();
            let mode_r = CString::new("r").unwrap();
            let rom_fp = libc::fopen(rom_c.as_ptr(), mode_r.as_ptr());
            if rom_fp.is_null() {
                ffi::tilem_calc_free(calc);
                return Err(format!("failed to open ROM {}", rom_path.display()));
            }

            let sav_fp = if let Some(p) = save_path {
                let sav_c = CString::new(p.as_os_str().to_str().unwrap_or("")).unwrap();
                let fp = libc::fopen(sav_c.as_ptr(), mode_r.as_ptr());
                if fp.is_null() {
                    log::info!("no save file at {}", p.display());
                }
                fp
            } else {
                ptr::null_mut()
            };

            let rc = ffi::tilem_calc_load_state(calc, rom_fp as *mut _, sav_fp as *mut _);
            libc::fclose(rom_fp);
            if !sav_fp.is_null() {
                libc::fclose(sav_fp);
            }
            if rc != 0 {
                ffi::tilem_calc_free(calc);
                return Err(format!(
                    "tilem_calc_load_state failed (rc={}) for {}",
                    rc,
                    rom_path.display()
                ));
            }

            let lcd_buf = ffi::tilem_lcd_buffer_new();
            if lcd_buf.is_null() {
                ffi::tilem_calc_free(calc);
                return Err("tilem_lcd_buffer_new returned NULL".into());
            }

            Ok(Emulator { calc, lcd_buf })
        }
    }

    pub fn run_microseconds(&mut self, microseconds: c_int) {
        unsafe {
            ffi::tilem_z80_run_time(self.calc, microseconds, ptr::null_mut());
        }
    }

    pub fn press_key(&mut self, scancode: c_int) {
        unsafe { ffi::tilem_keypad_press_key(self.calc, scancode) }
    }

    pub fn release_key(&mut self, scancode: c_int) {
        unsafe { ffi::tilem_keypad_release_key(self.calc, scancode) }
    }

    /// Copy the 96x64 monochrome LCD frame into `dst`. Returns (width, height, contrast).
    /// Buffer is arranged row-major, 1 byte per pixel (0 = off, non-zero = on).
    pub fn get_lcd_frame(&mut self, dst: &mut Vec<u8>) -> (u32, u32, u8) {
        unsafe {
            ffi::tilem_lcd_get_frame(self.calc, self.lcd_buf);
            let buf = &*self.lcd_buf;
            let w = buf.width as u32;
            let h = buf.height as u32;
            let stride = buf.rowstride as usize;
            let contrast = buf.contrast;

            dst.clear();
            if buf.data.is_null() || w == 0 || h == 0 {
                return (w, h, contrast);
            }
            dst.reserve((w * h) as usize);
            for y in 0..(h as usize) {
                let row = buf.data.add(y * stride);
                for x in 0..(w as usize) {
                    dst.push(*row.add(x));
                }
            }
            (w, h, contrast)
        }
    }

    pub fn save_state(&mut self, save_path: &Path) -> Result<(), String> {
        unsafe {
            let sav_c = CString::new(save_path.as_os_str().to_str().unwrap_or("")).unwrap();
            let mode_w = CString::new("w").unwrap();
            let fp = libc::fopen(sav_c.as_ptr(), mode_w.as_ptr());
            if fp.is_null() {
                return Err(format!("failed to open {} for write", save_path.display()));
            }
            let rc = ffi::tilem_calc_save_state(self.calc, ptr::null_mut(), fp as *mut _);
            libc::fclose(fp);
            if rc != 0 {
                return Err(format!("tilem_calc_save_state rc={}", rc));
            }
            Ok(())
        }
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        unsafe {
            if !self.lcd_buf.is_null() {
                ffi::tilem_lcd_buffer_free(self.lcd_buf);
            }
            if !self.calc.is_null() {
                ffi::tilem_calc_free(self.calc);
            }
        }
    }
}
