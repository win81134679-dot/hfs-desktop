//! 介面偏好（存在 exe 旁的 hfs-gui.json）與開機自動啟動（HKCU Run 機碼）

use serde_json::{json, Value};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use windows_sys::Win32::System::Registry::{
    RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "HFS 檔案分享";

#[derive(Clone, PartialEq)]
pub struct Prefs {
    /// 一打開就縮到系統匣（伺服器照樣啟動）
    pub start_in_tray: bool,
    /// 上次選的分享網址
    pub preferred_url: Option<String>,
    // ---- 更新（節流狀態要落地，重開程式也不會多查） ----
    pub auto_update: bool,
    /// 下次可以自動查 GitHub 的時間（unix 秒）
    pub next_check: u64,
    /// 上次成功查到的時間（顯示用）
    pub last_checked: u64,
    /// 上次查到的最新版（Release::to_json）
    pub latest: Option<Value>,
    /// 「稍後提醒」：這個時間之前不再跳提醒
    pub remind_after: u64,
    /// 「略過這個版本」
    pub skipped_version: Option<String>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            start_in_tray: false,
            preferred_url: None,
            auto_update: true,
            next_check: 0,
            last_checked: 0,
            latest: None,
            remind_after: 0,
            skipped_version: None,
        }
    }
}

pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

static FILE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// 偏好檔放在 HFS 資料夾裡（exe 放桌面時才不會在桌面多出檔案）
pub fn init(hfs_dir: &std::path::Path) {
    let _ = FILE.set(hfs_dir.join("hfs-gui.json"));
}

fn file() -> PathBuf {
    FILE.get().cloned().unwrap_or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("hfs-gui.json")))
            .unwrap_or_else(|| PathBuf::from("hfs-gui.json"))
    })
}

pub fn load() -> Prefs {
    let v: Value = std::fs::read_to_string(file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    Prefs {
        start_in_tray: v["start_in_tray"].as_bool().unwrap_or(false),
        preferred_url: v["preferred_url"].as_str().map(str::to_owned),
        auto_update: v["auto_update"].as_bool().unwrap_or(true),
        next_check: v["next_check"].as_u64().unwrap_or(0),
        last_checked: v["last_checked"].as_u64().unwrap_or(0),
        latest: v.get("latest").filter(|x| x.is_object()).cloned(),
        remind_after: v["remind_after"].as_u64().unwrap_or(0),
        skipped_version: v["skipped_version"].as_str().map(str::to_owned),
    }
}

pub fn save(p: &Prefs) -> std::io::Result<()> {
    let v = json!({
        "start_in_tray": p.start_in_tray,
        "preferred_url": p.preferred_url,
        "auto_update": p.auto_update,
        "next_check": p.next_check,
        "last_checked": p.last_checked,
        "latest": p.latest,
        "remind_after": p.remind_after,
        "skipped_version": p.skipped_version,
    });
    std::fs::write(file(), serde_json::to_string_pretty(&v).unwrap_or_default())
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn autostart_enabled() -> bool {
    let mut size = 0u32;
    let rc = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), wide(RUN_VALUE).as_ptr(), RRF_RT_REG_SZ, null_mut(), null_mut(), &mut size)
    };
    rc == 0
}

/// 開機自動啟動時帶 --tray，直接縮在系統匣
pub fn set_autostart(on: bool) -> Result<(), String> {
    let rc = unsafe {
        if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let data = wide(&format!("\"{}\" --tray", exe.display()));
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                wide(RUN_KEY).as_ptr(),
                wide(RUN_VALUE).as_ptr(),
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), wide(RUN_VALUE).as_ptr())
        }
    };
    // 關閉時本來就不存在（2 = ERROR_FILE_NOT_FOUND）也算成功
    if rc == 0 || (!on && rc == 2) {
        Ok(())
    } else {
        Err(format!("登錄檔錯誤 {rc}"))
    }
}
