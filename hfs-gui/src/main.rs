#![windows_subsystem = "windows"]

mod api;
mod embedded;
mod icon;
mod prefs;
mod server;
mod theme;
mod ui;
mod updater;
mod win;

use eframe::egui;
use std::path::PathBuf;

pub const TITLE: &str = "HFS ~ HTTP 檔案伺服器";

fn main() -> eframe::Result<()> {
    // 已經開著（可能縮在系統匣）就把舊視窗叫出來，不開第二個
    if !win::single_instance("Local\\hfs-gui-single-instance") {
        win::activate_existing(TITLE);
        return Ok(());
    }
    let hfs_dir = find_hfs_dir();
    prefs::init(&hfs_dir);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(TITLE)
        .with_inner_size([1120.0, 720.0])
        .with_min_inner_size([860.0, 520.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(icon::WINDOW_PNG) {
        viewport = viewport.with_icon(icon);
    }
    // 開機自動啟動帶 --tray，或使用者選了「打開時縮到系統匣」：視窗先不顯示，伺服器照常啟動
    if std::env::args().any(|a| a == "--tray") || prefs::load().start_in_tray {
        viewport = viewport.with_visible(false);
    }
    let options = eframe::NativeOptions {
        viewport: viewport
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(ui::App::new(cc, hfs_dir)))),
    )
}

/// 找 hfs.exe：先找自己旁邊、hfs\ 子資料夾、開發時的 ..\..\..\hfs；
/// 都沒有就用 %LOCALAPPDATA%\HFS Desktop（單一 exe 模式，第一次啟動會自動下載 HFS）
fn find_hfs_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    let candidates = [
        exe_dir.clone(),
        exe_dir.join("hfs"),
        exe_dir.join("..").join("hfs"),
        exe_dir.join("..").join("..").join("..").join("hfs"),
        embedded::data_dir(),
    ];
    candidates
        .iter()
        .find(|d| d.join("hfs.exe").is_file())
        .map(|d| d.canonicalize().map(strip_verbatim).unwrap_or_else(|_| d.clone()))
        .unwrap_or_else(|| {
            let dir = embedded::data_dir();
            let _ = std::fs::create_dir_all(&dir);
            dir
        })
}

/// canonicalize 會加上 \\?\ 前綴，hfs.exe 和檔案總管都不喜歡
fn strip_verbatim(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) => PathBuf::from(rest),
        None => p,
    }
}
