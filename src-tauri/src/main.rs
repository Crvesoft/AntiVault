// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod single_instance {
    extern "system" {
        fn CreateMutexW(lpMutexAttributes: *mut std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> isize;
        fn GetLastError() -> u32;
        fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
        fn SetForegroundWindow(hWnd: isize) -> i32;
        fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
        fn IsIconic(hWnd: isize) -> i32;
    }

    const ERROR_ALREADY_EXISTS: u32 = 183;
    static mut MUTEX_HANDLE: isize = 0;

    pub fn check_and_activate_existing() -> bool {
        let mutex_name: Vec<u16> = "Local\\AntiVault_App_SingleInstance_Mutex\0".encode_utf16().collect();
        unsafe {
            let handle = CreateMutexW(std::ptr::null_mut(), 0, mutex_name.as_ptr());
            if handle != 0 {
                MUTEX_HANDLE = handle;
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    // Try to find and bring the existing AntiVault window to the front
                    let window_title: Vec<u16> = "AntiVault\0".encode_utf16().collect();
                    let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());
                    if hwnd != 0 {
                        if IsIconic(hwnd) != 0 {
                            ShowWindow(hwnd, 9); // SW_RESTORE
                        } else {
                            ShowWindow(hwnd, 5); // SW_SHOW
                        }
                        SetForegroundWindow(hwnd);
                    }
                    return true;
                }
            }
        }
        false
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    {
        if single_instance::check_and_activate_existing() {
            std::process::exit(0);
        }
    }

    antivault_lib::run()
}
