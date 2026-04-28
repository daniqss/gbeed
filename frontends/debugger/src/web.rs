use std::{fs, path::Path};

use crate::EmulatorApp;

// Emscripten bindings to be able to:
// 1. set up the main loop in a way that works with the browser's event loop
// 2. use the browser's APIs injecting Javascript code from Rust
#[allow(improper_ctypes)]
unsafe extern "C" {
    pub fn emscripten_set_main_loop_arg(
        func: unsafe extern "C" fn(*mut EmulatorApp),
        arg: *mut EmulatorApp,
        fps: std::os::raw::c_int,
        simulate_infinite_loop: std::os::raw::c_int,
    );
    pub fn emscripten_run_script(script: *const std::os::raw::c_char);
    pub fn emscripten_run_script_int(script: *const std::os::raw::c_char) -> std::os::raw::c_int;
}

pub unsafe extern "C" fn wasm_main_loop(app: *mut EmulatorApp) {
    let app = unsafe { &mut *app };
    if let Err(e) = app.update() {
        eprintln!("Error during update: {e}");
    }
    app.draw();
}

/// We need a static reference to be able to call save_game from JavaScript
pub static mut APP_PTR: *mut EmulatorApp = std::ptr::null_mut();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn save_game_wasm() {
    unsafe {
        if !APP_PTR.is_null() {
            let _ = (*APP_PTR).save_game();
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn load_rom_from_js(path_ptr: *const std::ffi::c_char) {
    let path = unsafe { std::ffi::CStr::from_ptr(path_ptr) }
        .to_str()
        .unwrap_or("");
    if let Some(app) = unsafe { APP_PTR.as_mut() } {
        match app.load_rom(path) {
            Ok(state) => app.state = state,
            Err(e) => eprintln!("Failed to load ROM from JS: {e}"),
        }
    }
}

pub fn hide_open_rom_button() {
    let script = "document.getElementById('open-rom-btn').style.display = 'none';";
    if let Ok(script) = std::ffi::CString::new(script) {
        unsafe {
            emscripten_run_script(script.as_ptr());
        }
    }
}

pub mod local_storage {
    use super::*;

    pub fn store_save(path: &Path, data: &[u8]) {
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("unknown");

        // use of MEMFS to bridge the save data to JavaScript
        let tmp_path = "/tmp_save.sav";
        if let Err(e) = fs::write(tmp_path, data) {
            eprintln!("Failed to write bridge file: {:?}", e);
            return;
        }

        let script = format!(
            "try {{ \
        const bytes = FS.readFile('{}'); \
        const binary = Array.from(bytes, b => String.fromCharCode(b)).join(''); \
        const encoded = btoa(binary); \
        localStorage.setItem('{}', encoded); \
        console.log('Game saved to localStorage: {}'); \
    }} catch (e) {{ \
        console.error('Error saving to localStorage:', e); \
    }}",
            tmp_path, filename, filename
        );

        match std::ffi::CString::new(script) {
            Ok(script) => unsafe {
                emscripten_run_script(script.as_ptr());
            },
            _ => eprintln!("Failed to create script string for saving to localStorage"),
        };
        let _ = fs::remove_file(tmp_path);
    }

    pub fn load_save(path: &Path) -> Option<Vec<u8>> {
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("unknown");
        let tmp_path = "/tmp_load.sav";

        let script = format!(
            "try {{ \
        const data = localStorage.getItem('{}'); \
        if (data) {{ \
            const binary = atob(data); \
            const bytes = Uint8Array.from(binary, c => c.charCodeAt(0)); \
            FS.writeFile('{}', bytes); \
        }} \
    }} catch (e) {{ \
        console.error('Error loading from localStorage:', e); \
    }}",
            filename, tmp_path
        );

        match std::ffi::CString::new(script) {
            Ok(script) => unsafe {
                emscripten_run_script(script.as_ptr());
            },
            _ => eprintln!("Failed to create script string for loading from localStorage"),
        };

        let result = fs::read(tmp_path).ok();
        if result.is_some() {
            println!("Loaded save from localStorage with key: {}", filename);
            let _ = fs::remove_file(tmp_path);
        }
        result
    }
}
