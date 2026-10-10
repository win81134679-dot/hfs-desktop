//! Win32 小工具：單一執行個體、顯示/隱藏視窗、開網址、檔案總管

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HWND};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, IsIconic, SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE, SW_SHOW, SW_SHOWNORMAL,
};

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// 回傳 false 代表已經有另一個在跑
pub fn single_instance(name: &str) -> bool {
    unsafe {
        let handle = CreateMutexW(null(), 0, wide(name).as_ptr());
        // handle 故意不關，程式結束時系統會收回
        !handle.is_null() && GetLastError() != ERROR_ALREADY_EXISTS
    }
}

pub fn activate_existing(title: &str) {
    let hwnd = unsafe { FindWindowW(null(), wide(title).as_ptr()) };
    if !hwnd.is_null() {
        show(hwnd as isize);
    }
}

pub fn show(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    let h = hwnd as HWND;
    unsafe {
        ShowWindow(h, SW_SHOW);
        if IsIconic(h) != 0 {
            ShowWindow(h, SW_RESTORE);
        }
        SetForegroundWindow(h);
    }
}

pub fn hide(hwnd: isize) {
    if hwnd != 0 {
        unsafe { ShowWindow(hwnd as HWND, SW_HIDE) };
    }
}

/// 用預設程式開啟網址或路徑
pub fn open(target: &str) {
    unsafe {
        ShellExecuteW(
            null_mut(),
            wide("open").as_ptr(),
            wide(target).as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        );
    }
}

/// 資料夾直接開；檔案則開所在資料夾並選取它
pub fn reveal(path: &str, is_folder: bool) {
    if is_folder {
        open(path);
    } else {
        let _ = Command::new("explorer.exe").raw_arg(format!("/select,\"{path}\"")).spawn();
    }
}
