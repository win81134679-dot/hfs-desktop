//! 主視窗（Google 雲端硬碟風格）：上方 Logo／網址／狀態，左邊分享項目，右邊動態記錄與目前連線卡片。

mod admin;
mod update;

use crate::server::{self, LogKind, LogLine, Req, Server, Shared, Status};
use crate::theme::{self, Pill, Row};
use crate::{icon, prefs, win};
use eframe::egui::{self, Align2, Color32, CornerRadius, Margin, Rect, RichText, Sense, Vec2};
use egui_extras::{Column, TableBuilder};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// 畫面上按下去的動作，畫完、放開鎖之後才執行
enum Action {
    Start,
    Stop,
    AddFiles(Option<String>),
    AddFolder(Option<String>),
    AddPaths(Option<String>, Vec<PathBuf>),
    AddVirtual(Option<String>),
    Remove(String),
    SetUpload(String, bool),
    OpenUrl(String),
    Copy(String),
    Reveal(String, bool),
    Disconnect(String, u16),
    DisconnectAll,
    BlockIp(String),
    Unblock(String),
    EditPort,
    ClearLog,
    ExportLog,
    Props(String),
    /// 分享項目樹拖曳：(來源 uri, 目的資料夾 uri)
    MoveVfs(String, String),
    /// 切換分頁時重新讀帳號和設定（可能被網頁管理介面改過）
    Refresh,
    CheckUpdate,
    OpenUpdate,
    Unskip,
    SaveConfig(Value, &'static str),
    SetAutostart(bool),
    SavePrefs,
    Exit,
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Accounts,
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
enum LogFilter {
    All,
    Download,
    Upload,
    Error,
}

pub struct App {
    ctx: egui::Context,
    server: Server,
    hwnd: isize,
    tray: Option<TrayIcon>,
    tray_tip: String,
    /// 系統匣選單按了「停止伺服器並結束」
    quit_req: Arc<AtomicBool>,
    /// 伺服器停好了，可以真的關窗
    exit_now: Arc<AtomicBool>,
    allow_close: bool,
    close_dialog: bool,
    exiting: bool,
    /// (uri, 是否資料夾)
    selected: Option<(String, bool)>,
    collapsed: HashSet<String>,
    url_idx: usize,
    port_dialog: Option<String>,
    vdir_dialog: Option<(Option<String>, String)>,
    // 開著的下拉選單，記住按鈕位置
    new_menu: Option<Rect>,
    more_menu: Option<Rect>,
    url_menu: Option<Rect>,
    qr_menu: Option<Rect>,
    tab: Tab,
    log_filter: LogFilter,
    log_search: String,
    prefs: prefs::Prefs,
    autostart: bool,
    props: Option<admin::PropsForm>,
    account_form: Option<admin::AccountForm>,
    confirm_delete: Option<String>,
    settings: Option<admin::SettingsForm>,
    block_input: String,
    /// 目前埠號（每幀更新），組本機網址用
    port: u16,
    /// 分享項目樹正在拖曳的 (uri, 顯示名稱)
    tree_drag: Option<(String, String)>,
    /// 這一幀游標下可以放下的資料夾
    tree_drop: Option<String>,
    // ---- 更新 ----
    update_dialog: bool,
    /// 這次開程式已經自動提醒過
    update_prompted: bool,
    /// 正在等手動檢查的結果
    manual_check: bool,
    last_manual_check: Option<std::time::Instant>,
    /// 測試用：HFS_GUI_FAKE_VERSION 假裝目前是舊版
    fake_version: Option<String>,
    /// 測試用：HFS_GUI_TEST_UPGRADE=1 提醒出現就自動按「立即升級」
    test_upgrade: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, hfs_dir: PathBuf) -> Self {
        let ctx = cc.egui_ctx.clone();
        theme::apply(&ctx);
        let hwnd = cc
            .window_handle()
            .ok()
            .and_then(|h| match h.as_raw() {
                RawWindowHandle::Win32(w) => Some(w.hwnd.get()),
                _ => None,
            })
            .unwrap_or(0);
        let server = Server::new(ctx.clone(), hfs_dir);
        let quit_req = Arc::new(AtomicBool::new(false));
        let tray = match make_tray(&ctx, hwnd, quit_req.clone()) {
            Ok(t) => Some(t),
            Err(e) => {
                server.log(LogKind::Error, format!("建立系統匣圖示失敗：{e}"));
                None
            }
        };
        server.log(LogKind::Info, format!("HFS 位置：{}", server.hfs_dir.display()));
        server.start();
        // 測試用：HFS_GUI_TAB=accounts|settings 直接開到該分頁
        let tab = match std::env::var("HFS_GUI_TAB").as_deref() {
            Ok("accounts") => Tab::Accounts,
            Ok("settings") => Tab::Settings,
            _ => Tab::Overview,
        };
        Self {
            ctx,
            server,
            hwnd,
            tray,
            tray_tip: String::new(),
            quit_req,
            exit_now: Arc::new(AtomicBool::new(false)),
            allow_close: false,
            close_dialog: false,
            exiting: false,
            selected: None,
            collapsed: HashSet::new(),
            url_idx: 0,
            port_dialog: None,
            vdir_dialog: None,
            new_menu: None,
            more_menu: None,
            url_menu: None,
            qr_menu: None,
            tab,
            log_filter: LogFilter::All,
            log_search: String::new(),
            prefs: prefs::load(),
            autostart: prefs::autostart_enabled(),
            props: None,
            account_form: None,
            confirm_delete: None,
            settings: None,
            block_input: String::new(),
            port: server::DEFAULT_PORT,
            tree_drag: None,
            tree_drop: None,
            update_dialog: false,
            update_prompted: false,
            manual_check: false,
            last_manual_check: None,
            fake_version: std::env::var("HFS_GUI_FAKE_VERSION").ok().filter(|v| !v.is_empty()),
            test_upgrade: std::env::var("HFS_GUI_TEST_UPGRADE").as_deref() == Ok("1"),
        }
    }

    /// 新增檔案時的目標：選到資料夾就放進去，選到檔案就放在它旁邊，沒選就放根目錄
    fn target_parent(&self) -> Option<String> {
        match &self.selected {
            Some((uri, true)) if uri != "/" => Some(uri.clone()),
            Some((uri, false)) => parent_of(uri),
            _ => None,
        }
    }

    fn begin_exit(&mut self) {
        if self.exiting {
            return;
        }
        self.exiting = true;
        self.close_dialog = false;
        let status = self.server.shared.lock().unwrap().status.clone();
        if status.is_active() || status == Status::Stopping {
            let flag = self.exit_now.clone();
            let ctx = self.ctx.clone();
            self.server.stop(Some(Box::new(move || {
                flag.store(true, Ordering::SeqCst);
                ctx.request_repaint();
            })));
        } else {
            self.exit_now.store(true, Ordering::SeqCst);
        }
    }

    fn run(&mut self, action: Action) {
        let running = self.server.shared.lock().unwrap().status == Status::Running;
        match action {
            Action::Start => self.server.start(),
            Action::Stop => self.server.stop(None),
            Action::AddFiles(parent) => {
                if let Some(files) = rfd::FileDialog::new().set_title("選擇要分享的檔案").pick_files() {
                    self.server.add_paths(parent, files);
                }
            }
            Action::AddFolder(parent) => {
                if let Some(dir) = rfd::FileDialog::new().set_title("選擇要分享的資料夾").pick_folder() {
                    self.server.add_paths(parent, vec![dir]);
                }
            }
            Action::AddPaths(parent, paths) => {
                if running {
                    self.server.add_paths(parent, paths);
                } else {
                    self.server.log(LogKind::Error, "請先啟動伺服器再加入檔案");
                }
            }
            Action::AddVirtual(parent) => self.vdir_dialog = Some((parent, String::new())),
            Action::Remove(uri) => {
                if self.selected.as_ref().is_some_and(|(u, _)| u.starts_with(&uri)) {
                    self.selected = None;
                }
                self.server.remove(uri);
            }
            Action::SetUpload(uri, on) => self.server.set_upload(uri, on),
            Action::OpenUrl(url) => win::open(&url),
            Action::Copy(text) => {
                self.ctx.copy_text(text.clone());
                self.server.log(LogKind::Info, format!("已複製：{text}"));
            }
            Action::Reveal(path, is_folder) => win::reveal(&path, is_folder),
            Action::Disconnect(ip, port) => self.server.disconnect(ip, port),
            Action::DisconnectAll => self.server.disconnect_all(),
            Action::BlockIp(ip) => self.server.block_ip(ip),
            Action::Unblock(ip) => self.server.unblock_ip(ip),
            Action::EditPort => self.port_dialog = Some(self.server.port().to_string()),
            Action::ClearLog => self.server.shared.lock().unwrap().log.clear(),
            Action::ExportLog => {
                let text = {
                    let s = self.server.shared.lock().unwrap();
                    s.log.iter().map(|l| format!("{}  {}", l.time, l.text)).collect::<Vec<_>>().join("\r\n")
                };
                let name = format!("HFS 記錄 {}.txt", chrono::Local::now().format("%Y-%m-%d %H%M"));
                if let Some(path) = rfd::FileDialog::new().set_file_name(&name).add_filter("文字檔", &["txt"]).save_file() {
                    match std::fs::write(&path, text) {
                        Ok(()) => self.server.log(LogKind::Info, format!("記錄已匯出：{}", path.display())),
                        Err(e) => self.server.log(LogKind::Error, format!("匯出失敗：{e}")),
                    }
                }
            }
            Action::Props(uri) => {
                let node = self.server.shared.lock().unwrap().vfs.as_ref().and_then(|r| admin::find_node(r, &uri));
                if let Some(node) = node {
                    self.props = Some(admin::PropsForm::from_node(&uri, &node));
                }
            }
            Action::MoveVfs(from, parent) => {
                if self.selected.as_ref().is_some_and(|(u, _)| u.starts_with(&from)) {
                    self.selected = None;
                }
                self.server.move_vfs(from, parent);
            }
            Action::CheckUpdate => self.check_now(),
            Action::OpenUpdate => self.update_dialog = true,
            Action::Unskip => {
                self.prefs.skipped_version = None;
                let _ = prefs::save(&self.prefs);
            }
            Action::Refresh => {
                if running {
                    let server = self.server.clone();
                    std::thread::spawn(move || {
                        server.refresh_accounts();
                        server.refresh_config();
                    });
                }
            }
            Action::SaveConfig(values, msg) => self.server.save_config(values, msg),
            Action::SetAutostart(on) => match prefs::set_autostart(on) {
                Ok(()) => {
                    self.autostart = on;
                    self.server.log(LogKind::Info, if on { "已設定開機自動啟動" } else { "已取消開機自動啟動" });
                }
                Err(e) => self.server.log(LogKind::Error, format!("設定開機啟動失敗：{e}")),
            },
            Action::SavePrefs => {
                if let Err(e) = prefs::save(&self.prefs) {
                    self.server.log(LogKind::Error, format!("儲存介面偏好失敗：{e}"));
                }
            }
            Action::Exit => self.begin_exit(),
        }
    }

    fn base_url(&self, s: &Shared) -> String {
        s.urls.get(self.url_idx).cloned().unwrap_or_else(|| format!("http://localhost:{}", s.port))
    }

    // ---------- 上方列 ----------

    fn top_bar(&mut self, ctx: &egui::Context, s: &Shared, actions: &mut Vec<Action>) {
        egui::TopBottomPanel::top("top")
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: 18, right: 16, top: 10, bottom: 8 }))
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.set_height(48.0);
                    theme::logo(ui, 38.0);
                    ui.add_space(2.0);
                    ui.label(RichText::new("HFS 檔案分享").font(theme::regular(21.0)).color(theme::TEXT2));
                    ui.add_space(28.0);
                    self.url_pill(ui, s, actions);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let more = theme::icon_button_sized(ui, "more", "更多", 40.0, theme::TEXT2);
                        if more.clicked() {
                            self.more_menu = if self.more_menu.is_some() { None } else { Some(more.rect) };
                        }
                        ui.add_space(4.0);
                        match &s.status {
                            Status::Running => {
                                if theme::pill(ui, Pill::Tonal, Some("power"), "停止伺服器", true).clicked() {
                                    actions.push(Action::Stop);
                                }
                            }
                            Status::Starting | Status::Stopping => {
                                theme::pill(ui, Pill::Tonal, Some("power"), "請稍候…", false);
                            }
                            Status::Stopped | Status::Failed(_) => {
                                if theme::pill(ui, Pill::Filled, Some("play"), "啟動伺服器", true).clicked() {
                                    actions.push(Action::Start);
                                }
                            }
                        }
                        let port = theme::pill(ui, Pill::Outlined, Some("tune"), &format!("埠 {}", s.port), true);
                        if port.on_hover_text("變更埠號").clicked() {
                            actions.push(Action::EditPort);
                        }
                        ui.add_space(4.0);
                        if let Some(rel) = self.pending_update(s) {
                            let resp = theme::pill(ui, Pill::Tonal, Some("cloudUp"), &format!("新版 {}", rel.tag), true);
                            if resp.on_hover_text("HFS 有新版本，按一下看看").clicked() {
                                actions.push(Action::OpenUpdate);
                            }
                        }
                        let (text, fg, bg) = match &s.status {
                            Status::Running if s.attached => ("運行中（接管）".to_string(), theme::GREEN, theme::GREEN_BG),
                            Status::Running => ("運行中".to_string(), theme::GREEN, theme::GREEN_BG),
                            Status::Starting => ("啟動中".to_string(), theme::AMBER, theme::AMBER_BG),
                            Status::Stopping => ("停止中".to_string(), theme::AMBER, theme::AMBER_BG),
                            Status::Stopped => ("已停止".to_string(), theme::TEXT2, theme::CARD),
                            Status::Failed(_) => ("發生錯誤".to_string(), theme::RED, theme::RED_BG),
                        };
                        let chip = theme::chip(ui, &text, fg, bg, true);
                        if let Status::Failed(msg) = &s.status {
                            chip.on_hover_text(msg);
                        }
                    });
                });
            });
    }

    fn url_pill(&mut self, ui: &mut egui::Ui, s: &Shared, actions: &mut Vec<Action>) {
        let width = (ui.available_width() - 460.0).clamp(300.0, 600.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 46.0), Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::same(23), theme::SEARCH);
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink2(egui::vec2(10.0, 7.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let ui = &mut child;
        ui.add_space(4.0);
        ui.add(theme::icon("lan", theme::TEXT2).fit_to_exact_size(Vec2::splat(20.0)));
        ui.add_space(4.0);
        if s.urls.is_empty() {
            ui.label(RichText::new("伺服器未運行").color(theme::TEXT3));
            return;
        }
        if let Some(i) = self.prefs.preferred_url.as_ref().and_then(|p| s.urls.iter().position(|u| u == p)) {
            self.url_idx = i;
        }
        if self.url_idx >= s.urls.len() {
            self.url_idx = 0;
        }
        let url = format!("{}/", s.urls[self.url_idx]);
        let text_w = ui.available_width() - 112.0;
        let (r, resp) = ui.allocate_exact_size(egui::vec2(text_w, 32.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r, CornerRadius::same(16), theme::HOVER);
        }
        let many = s.urls.len() > 1;
        let galley = theme::elide(ui, &url, theme::regular(15.0), theme::TEXT, text_w - if many { 40.0 } else { 16.0 });
        ui.painter().galley(egui::pos2(r.left() + 8.0, r.center().y - galley.size().y / 2.0), galley, theme::TEXT);
        if many {
            theme::paint_icon(ui, "expandMore", Rect::from_center_size(egui::pos2(r.right() - 16.0, r.center().y), Vec2::splat(20.0)), theme::TEXT2);
        }
        let resp = resp.on_hover_text(if many { "按一下選擇其他網址" } else { "分享網址" });
        if resp.clicked() && many {
            self.url_menu = if self.url_menu.is_some() { None } else { Some(r) };
        }
        if theme::icon_button(ui, "copy", "複製網址").clicked() {
            actions.push(Action::Copy(url.clone()));
        }
        // 自己開用 localhost：HFS 視本機為管理者，才能刪除、重新命名
        if theme::icon_button(ui, "open", "用瀏覽器開啟（本機管理者身分）").clicked() {
            actions.push(Action::OpenUrl(format!("http://localhost:{}/", s.port)));
        }
        let qr = theme::icon_button(ui, "qr", "顯示 QR Code，給手機掃描");
        if qr.clicked() {
            self.qr_menu = if self.qr_menu.is_some() { None } else { Some(qr.rect) };
        }
    }

    // ---------- 左邊：分享項目 ----------

    fn vfs_panel(&mut self, ctx: &egui::Context, s: &Shared, actions: &mut Vec<Action>) {
        let running = s.status == Status::Running;
        let base = self.base_url(s);
        egui::SidePanel::left("vfs")
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: 12, right: 10, top: 6, bottom: 10 }))
            .show_separator_line(false)
            .resizable(true)
            .default_width(310.0)
            .min_width(230.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    let fab = theme::fab(ui, "新增", running);
                    if fab.clicked() {
                        self.new_menu = if self.new_menu.is_some() { None } else { Some(fab.rect) };
                    }
                    if !running {
                        fab.on_hover_text("伺服器啟動後才能新增");
                    }
                });
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.label(RichText::new("分享項目").font(theme::bold(12.5)).color(theme::TEXT3));
                });
                ui.add_space(2.0);
                egui::TopBottomPanel::bottom("vfs_hint")
                    .frame(egui::Frame::new().inner_margin(Margin { left: 14, right: 8, top: 10, bottom: 2 }))
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.add(theme::icon("info", theme::TEXT3).fit_to_exact_size(Vec2::splat(16.0)));
                            ui.add(egui::Label::new(RichText::new("把檔案或資料夾拖進視窗就能分享，右鍵可設定上傳權限").small().color(theme::TEXT3)).wrap());
                        });
                    });
                self.tree_drop = None;
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    match &s.vfs {
                        Some(root) => self.draw_node(ui, root, "/".to_string(), 0, &base, running, actions),
                        None => {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.add_space(14.0);
                                ui.label(RichText::new(if running { "讀取中…" } else { "伺服器未運行" }).color(theme::TEXT3));
                            });
                        }
                    }
                });
                self.tree_drag_feedback(ui.ctx(), actions);
            });
    }

    /// 拖曳中：游標旁顯示項目名稱；放開時若在資料夾上就搬過去
    fn tree_drag_feedback(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        let Some((from, label)) = self.tree_drag.clone() else { return };
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("tree_drag")));
            let text = match &self.tree_drop {
                Some(to) => format!("{label}  →  {}", if to == "/" { "/".to_string() } else { server::percent_decode(to.trim_end_matches('/')).rsplit('/').next().unwrap_or("").to_string() }),
                None => label.clone(),
            };
            let galley = painter.layout_no_wrap(text, theme::bold(13.0), theme::ON_SELECTED);
            let rect = Rect::from_min_size(pos + egui::vec2(14.0, 10.0), galley.size() + egui::vec2(24.0, 14.0));
            painter.rect_filled(rect, CornerRadius::same(10), theme::SELECTED);
            painter.galley(rect.min + egui::vec2(12.0, 7.0), galley, theme::ON_SELECTED);
        }
        ctx.set_cursor_icon(if self.tree_drop.is_some() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::NoDrop });
        if ctx.input(|i| i.pointer.any_released()) {
            if let Some(to) = self.tree_drop.take() {
                actions.push(Action::MoveVfs(from, to));
            }
            self.tree_drag = None;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_node(&mut self, ui: &mut egui::Ui, node: &Value, uri: String, depth: usize, base: &str, running: bool, actions: &mut Vec<Action>) {
        let children = node.get("children").and_then(Value::as_array);
        let is_root = uri == "/";
        let is_folder = is_root || node["type"].as_str() == Some("folder") || children.is_some();
        let name = node["name"].as_str().unwrap_or_default();
        let source = node["source"].as_str().map(str::to_owned);
        let explicit = node.get("can_upload").filter(|v| !v.is_null());
        let upload = explicit.or_else(|| node.get("inherited").and_then(|i| i.get("can_upload"))) == Some(&Value::Bool(true));
        let has_children = children.is_some_and(|c| !c.is_empty());
        let collapsed = self.collapsed.contains(&uri);

        let label = if is_root {
            match &source {
                Some(src) => format!("/  {}", src.trim_end_matches('\\').rsplit('\\').next().unwrap_or("")),
                None => "/".to_string(),
            }
        } else {
            name.to_string()
        };
        let (icon_name, icon_color) = if is_root { ("home", theme::TEXT2) } else { theme::file_icon(name, is_folder) };
        let url = format!("{base}{uri}");
        let local_url = format!("http://localhost:{}{uri}", self.port);
        let chevron_x = 12.0 + depth as f32 * 18.0;
        let row = Row {
            selected: self.selected.as_ref().is_some_and(|(u, _)| *u == uri),
            indent: depth as f32 * 18.0 + 20.0,
            height: 34.0,
            italic: is_folder && source.is_none() && !is_root,
            trailing: (upload && is_folder && source.is_some()).then_some(("可上傳", theme::GREEN, theme::GREEN_BG)),
            ..Row::new(icon_name, icon_color, &label)
        };
        let resp = row.show(ui);
        let chevron = Rect::from_center_size(egui::pos2(resp.rect.left() + chevron_x + 8.0, resp.rect.center().y), Vec2::splat(18.0));
        if has_children {
            theme::paint_icon(ui, if collapsed { "chevronRight" } else { "expandMore" }, chevron, theme::TEXT3);
        }
        // 拖曳搬移：開始拖非根目錄的項目；拖到別的資料夾上就標示為放下目標
        if resp.drag_started() && !is_root && running {
            self.tree_drag = Some((uri.clone(), label.clone()));
        }
        if let Some((from, _)) = &self.tree_drag {
            let over = ui.input(|i| i.pointer.hover_pos()).is_some_and(|p| resp.rect.contains(p));
            let current_parent = parent_of(from).unwrap_or_else(|| "/".into());
            if over && is_folder && uri != *from && !uri.starts_with(from.as_str()) && uri != current_parent {
                ui.painter().rect_stroke(resp.rect, CornerRadius::same(17), egui::Stroke::new(2.0_f32, theme::PRIMARY), egui::StrokeKind::Inside);
                self.tree_drop = Some(uri.clone());
            }
        }
        let on_chevron = has_children && ui.input(|i| i.pointer.interact_pos()).is_some_and(|p| chevron.expand(4.0).contains(p));
        if resp.clicked() && on_chevron {
            if collapsed {
                self.collapsed.remove(&uri);
            } else {
                self.collapsed.insert(uri.clone());
            }
        } else if resp.clicked() || resp.secondary_clicked() {
            self.selected = Some((uri.clone(), is_folder));
        }
        if resp.double_clicked() && !on_chevron {
            if has_children {
                if collapsed {
                    self.collapsed.remove(&uri);
                } else {
                    self.collapsed.insert(uri.clone());
                }
            } else {
                actions.push(Action::OpenUrl(local_url.clone()));
            }
        }
        let hover = match &source {
            Some(src) => format!("{src}\n{url}"),
            None if is_folder => format!("虛擬資料夾\n{url}"),
            None => url.clone(),
        };
        let resp = resp.on_hover_text(hover);
        resp.context_menu(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            if theme::menu_item(ui, "open", "用瀏覽器開啟", true).clicked() {
                actions.push(Action::OpenUrl(local_url.clone()));
                ui.close();
            }
            if theme::menu_item(ui, "link", "複製網址", true).clicked() {
                actions.push(Action::Copy(url.clone()));
                ui.close();
            }
            if theme::menu_item(ui, "tune", "屬性與權限…", running).clicked() {
                actions.push(Action::Props(uri.clone()));
                ui.close();
            }
            if let Some(src) = &source {
                if theme::menu_item(ui, "folder", "在檔案總管中顯示", true).clicked() {
                    actions.push(Action::Reveal(src.clone(), is_folder));
                    ui.close();
                }
            }
            if is_folder {
                let here = if is_root { None } else { Some(uri.clone()) };
                ui.separator();
                if theme::menu_item(ui, "upload", "新增檔案到這裡…", running).clicked() {
                    actions.push(Action::AddFiles(here.clone()));
                    ui.close();
                }
                if theme::menu_item(ui, "folderUpload", "新增資料夾到這裡…", running).clicked() {
                    actions.push(Action::AddFolder(here.clone()));
                    ui.close();
                }
                if theme::menu_item(ui, "newFolder", "新增虛擬資料夾…", running).clicked() {
                    actions.push(Action::AddVirtual(here));
                    ui.close();
                }
                if source.is_some() {
                    ui.separator();
                    let (ic, text) = if upload { ("check", "允許任何人上傳（已開啟）") } else { ("cloudUp", "允許任何人上傳") };
                    let color = if upload { theme::GREEN } else { theme::TEXT2 };
                    if theme::menu_item_colored(ui, ic, text, running, color, theme::TEXT).clicked() {
                        actions.push(Action::SetUpload(uri.clone(), !upload));
                        ui.close();
                    }
                }
            }
            if !is_root {
                ui.separator();
                if theme::menu_item_colored(ui, "delete", "從分享中移除", running, theme::RED, theme::RED).clicked() {
                    actions.push(Action::Remove(uri.clone()));
                    ui.close();
                }
            }
        });

        if is_folder && !collapsed {
            for child in children.into_iter().flatten() {
                let child_name = child["name"].as_str().unwrap_or_default();
                let child_folder = child["type"].as_str() == Some("folder") || child.get("children").is_some_and(Value::is_array);
                let child_uri = format!("{uri}{}{}", server::encode_uri_component(child_name), if child_folder { "/" } else { "" });
                self.draw_node(ui, child, child_uri, depth + 1, base, running, actions);
            }
        }
    }

    // ---------- 右邊：記錄與連線 ----------

    fn main_area(&mut self, ctx: &egui::Context, s: &Shared, rows: &[&Map<String, Value>], actions: &mut Vec<Action>) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: 4, right: 16, top: 2, bottom: 6 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (tab, icon, label) in [
                        (Tab::Overview, "dashboard", "總覽".to_string()),
                        (Tab::Accounts, "people", format!("帳號（{}）", s.accounts.len())),
                        (Tab::Settings, "settings", "設定".to_string()),
                    ] {
                        if theme::tab(ui, icon, &label, self.tab == tab).clicked() {
                            self.tab = tab;
                            actions.push(Action::Refresh);
                        }
                    }
                });
                ui.add_space(8.0);
                match self.tab {
                    Tab::Overview => self.overview(ui, s, rows, actions),
                    Tab::Accounts => self.accounts_page(ui, s, actions),
                    Tab::Settings => self.settings_page(ui, s, actions),
                }
            });
    }

    fn overview(&mut self, ui: &mut egui::Ui, s: &Shared, rows: &[&Map<String, Value>], actions: &mut Vec<Action>) {
        let avail = ui.available_height();
        let conn_h = if rows.is_empty() { 96.0 } else { (avail * 0.4).clamp(150.0, 320.0) };
        let log_h = (avail - conn_h - 12.0 - 64.0).max(120.0);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(log_h);
            self.log_card(ui, s, actions);
        });
        ui.add_space(12.0);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(conn_h);
            self.conn_card(ui, rows, actions);
        });
    }

    fn log_card(&mut self, ui: &mut egui::Ui, s: &Shared, actions: &mut Vec<Action>) {
        theme::card_header(ui, "history", "動態記錄", |ui| {
            if theme::icon_button(ui, "delete", "清除記錄").clicked() {
                actions.push(Action::ClearLog);
            }
            if theme::icon_button(ui, "save", "匯出記錄為文字檔").clicked() {
                actions.push(Action::ExportLog);
            }
            ui.add(
                egui::TextEdit::singleline(&mut self.log_search)
                    .hint_text("搜尋 IP 或檔名")
                    .desired_width(170.0)
                    .margin(egui::vec2(12.0, 5.0)),
            );
            ui.add_space(6.0);
            for (f, label) in [(LogFilter::Error, "錯誤"), (LogFilter::Upload, "上傳"), (LogFilter::Download, "下載"), (LogFilter::All, "全部")] {
                if theme::pill(ui, if self.log_filter == f { Pill::Tonal } else { Pill::Text }, None, label, true).clicked() {
                    self.log_filter = f;
                }
            }
        });
        ui.add_space(4.0);
        let q = self.log_search.trim().to_lowercase();
        let lines: Vec<&LogLine> = s
            .log
            .iter()
            .filter(|l| match self.log_filter {
                LogFilter::All => true,
                LogFilter::Error => l.kind == LogKind::Error,
                LogFilter::Download => l.req.as_ref().is_some_and(|r| r.status < 400 && r.method == "GET" && !r.path.ends_with('/')),
                LogFilter::Upload => l.req.as_ref().is_some_and(|r| r.method == "PUT" || r.method == "POST"),
            })
            .filter(|l| q.is_empty() || l.text.to_lowercase().contains(&q))
            .collect();
        if lines.is_empty() {
            ui.centered_and_justified(|ui| {
                let msg = if s.log.is_empty() { "還沒有任何動態" } else { "沒有符合條件的記錄" };
                ui.label(RichText::new(msg).color(theme::TEXT3));
            });
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show_rows(ui, 30.0, lines.len(), |ui, range| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for line in &lines[range] {
                    log_row(ui, line);
                }
            });
    }

    fn conn_card(&self, ui: &mut egui::Ui, rows: &[&Map<String, Value>], actions: &mut Vec<Action>) {
        theme::card_header(ui, "people", &format!("目前連線（{}）", rows.len()), |ui| {
            if !rows.is_empty() && theme::pill(ui, Pill::Text, Some("close"), "全部中斷", true).clicked() {
                actions.push(Action::DisconnectAll);
            }
        });
        if rows.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new("目前沒有人連線").color(theme::TEXT3));
            return;
        }
        ui.add_space(2.0);
        TableBuilder::new(ui)
            .striped(false)
            .resizable(false)
            .auto_shrink([false, false])
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(160.0))
            .column(Column::remainder().at_least(80.0).clip(true))
            .column(Column::exact(80.0))
            .column(Column::exact(160.0))
            .column(Column::exact(120.0))
            .column(Column::exact(72.0))
            .header(26.0, |mut h| {
                for t in ["IP 位址", "檔案", "狀態", "傳輸", "進度", ""] {
                    h.col(|ui| {
                        ui.label(RichText::new(t).font(theme::bold(12.5)).color(theme::TEXT2));
                    });
                }
            })
            .body(|mut body| {
                for c in rows {
                    let ip = c.get("ip").and_then(Value::as_str).unwrap_or("").to_string();
                    let port = c.get("port").and_then(Value::as_u64).unwrap_or(0) as u16;
                    let op = c.get("op").and_then(Value::as_str);
                    let path = c.get("path").and_then(Value::as_str).unwrap_or("");
                    let uploading = op == Some("upload");
                    body.row(36.0, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Label::new(RichText::new(format!("{ip}:{port}")).color(theme::TEXT2)).truncate());
                        });
                        row.col(|ui| {
                            let archive = c.get("archive").is_some_and(|a| !a.is_null());
                            let name = if archive { format!("打包下載 {path}") } else { path.to_string() };
                            let (ic, color) = theme::file_icon(path, path.ends_with('/') || archive);
                            ui.add(theme::icon(if archive { "archive" } else { ic }, color).fit_to_exact_size(Vec2::splat(18.0)));
                            ui.add(egui::Label::new(RichText::new(name).color(theme::TEXT)).truncate());
                        });
                        row.col(|ui| {
                            let (t, fg, bg) = match op {
                                Some("download") => ("下載中", theme::GREEN, theme::GREEN_BG),
                                Some("upload") => ("上傳中", theme::PURPLE, theme::PURPLE_BG),
                                Some("browsing") => ("瀏覽", theme::PRIMARY, theme::BLUE_BG),
                                _ => ("閒置", theme::TEXT2, theme::CARD),
                            };
                            theme::chip(ui, t, fg, bg, false);
                        });
                        row.col(|ui| {
                            let kb = c.get(if uploading { "inSpeedKb" } else { "outSpeedKb" }).and_then(Value::as_f64).unwrap_or(0.0);
                            let n = c.get(if uploading { "got" } else { "sent" }).and_then(Value::as_f64).unwrap_or(0.0);
                            if kb > 0.0 {
                                ui.label(RichText::new(format!("{}/s", server::fmt_bytes(kb * 1024.0))).color(theme::TEXT));
                            }
                            ui.label(RichText::new(server::fmt_bytes(n)).small().color(theme::TEXT3));
                        });
                        row.col(|ui| {
                            if let Some(p) = c.get("opProgress").and_then(Value::as_f64) {
                                progress(ui, p as f32);
                            }
                        });
                        row.col(|ui| {
                            if theme::icon_button(ui, "close", "中斷這個連線").clicked() {
                                actions.push(Action::Disconnect(ip.clone(), port));
                            }
                            if !server::is_loopback(&ip) && theme::icon_button_sized(ui, "block", "封鎖這個 IP", 32.0, theme::RED).clicked() {
                                actions.push(Action::BlockIp(ip.clone()));
                            }
                        });
                    });
                }
            });
    }

    fn status_bar(&self, ctx: &egui::Context, s: &Shared, visible_conns: usize) {
        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: 22, right: 22, top: 4, bottom: 8 }))
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let st = &s.stats;
                    let stat = |ui: &mut egui::Ui, icon: &'static str, color: Color32, text: String| {
                        ui.add(theme::icon(icon, color).fit_to_exact_size(Vec2::splat(16.0)));
                        ui.label(RichText::new(text).small().color(theme::TEXT2));
                        ui.add_space(10.0);
                    };
                    theme::sparkline(ui, &s.history, egui::vec2(120.0, 20.0)).on_hover_text("最近兩分鐘的流量（綠：送出、藍：接收）");
                    ui.add_space(6.0);
                    stat(ui, "up", theme::GREEN, format!("送出 {}/s", server::fmt_bytes(st.out_kb * 1024.0)));
                    stat(ui, "down", theme::PRIMARY, format!("接收 {}/s", server::fmt_bytes(st.in_kb * 1024.0)));
                    ui.label(
                        RichText::new(format!("總送出 {}  ·  總接收 {}", server::fmt_bytes(st.sent), server::fmt_bytes(st.got)))
                            .small()
                            .color(theme::TEXT3),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{visible_conns} 個連線")).small().color(theme::TEXT2));
                        ui.add(theme::icon("people", theme::TEXT3).fit_to_exact_size(Vec2::splat(16.0)));
                    });
                });
            });
    }

    // ---------- 下拉選單與對話框 ----------

    fn menus(&mut self, ctx: &egui::Context, s: &Shared, actions: &mut Vec<Action>) {
        let running = s.status == Status::Running;
        if let Some(anchor) = self.new_menu {
            let parent = self.target_parent();
            let target = parent.as_deref().map(server::percent_decode).unwrap_or_else(|| "/".into());
            let close = theme::dropdown(ctx, "new_menu", anchor, 250.0, |ui| {
                ui.label(RichText::new(format!("加入到 {target}")).small().color(theme::TEXT3));
                ui.add_space(4.0);
                let mut done = false;
                if theme::menu_item(ui, "upload", "新增檔案…", running).clicked() {
                    actions.push(Action::AddFiles(parent.clone()));
                    done = true;
                }
                if theme::menu_item(ui, "folderUpload", "新增資料夾…", running).clicked() {
                    actions.push(Action::AddFolder(parent.clone()));
                    done = true;
                }
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                if theme::menu_item(ui, "newFolder", "新增虛擬資料夾…", running).clicked() {
                    actions.push(Action::AddVirtual(parent.clone()));
                    done = true;
                }
                done
            });
            if close {
                self.new_menu = None;
            }
        }
        if let Some(anchor) = self.more_menu {
            let root_source = s.vfs.as_ref().and_then(|r| r["source"].as_str()).map(str::to_owned);
            let anchor = Rect::from_min_max(egui::pos2(anchor.right() - 260.0, anchor.top()), anchor.max);
            let close = theme::dropdown(ctx, "more_menu", anchor, 260.0, |ui| {
                let mut done = false;
                if theme::menu_item(ui, "folder", "開啟分享資料夾", root_source.is_some()).clicked() {
                    actions.push(Action::Reveal(root_source.clone().unwrap_or_default(), true));
                    done = true;
                }
                if theme::menu_item(ui, "web", "開啟分享網頁", running).clicked() {
                    actions.push(Action::OpenUrl(format!("http://localhost:{}/", s.port)));
                    done = true;
                }
                if theme::menu_item(ui, "tune", "網頁管理介面（進階設定）", running).clicked() {
                    actions.push(Action::OpenUrl(format!("http://localhost:{}/~/admin/", s.port)));
                    done = true;
                }
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                if theme::menu_item_colored(ui, "exit", "停止伺服器並結束", true, theme::RED, theme::RED).clicked() {
                    actions.push(Action::Exit);
                    done = true;
                }
                done
            });
            if close {
                self.more_menu = None;
            }
        }
        if let Some(anchor) = self.url_menu {
            let mut pick = None;
            let close = theme::dropdown(ctx, "url_menu", anchor, anchor.width(), |ui| {
                for (i, u) in s.urls.iter().enumerate() {
                    let ic = if i == self.url_idx { "check" } else { "lan" };
                    let color = if i == self.url_idx { theme::PRIMARY } else { theme::TEXT3 };
                    if theme::menu_item_colored(ui, ic, u, true, color, theme::TEXT).clicked() {
                        pick = Some(i);
                    }
                }
                pick.is_some()
            });
            if let Some(i) = pick {
                self.url_idx = i;
                self.prefs.preferred_url = Some(s.urls[i].clone());
                actions.push(Action::SavePrefs);
            }
            if close {
                self.url_menu = None;
            }
        }
        if let Some(anchor) = self.qr_menu {
            let url = format!("{}/", self.base_url(s));
            let anchor = Rect::from_min_max(egui::pos2(anchor.right() - 280.0, anchor.top()), anchor.max);
            let close = theme::dropdown(ctx, "qr_menu", anchor, 280.0, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if s.urls.is_empty() {
                        ui.label(RichText::new("伺服器未運行").color(theme::TEXT3));
                    } else {
                        theme::qr(ui, &url, 240.0);
                        ui.add_space(8.0);
                        ui.label(RichText::new("用手機相機掃描即可開啟").font(theme::bold(14.0)).color(theme::TEXT));
                        ui.add_space(2.0);
                        ui.label(RichText::new(&url).small().color(theme::TEXT3));
                        ui.label(RichText::new("手機要連在同一個 Wi-Fi").small().color(theme::TEXT3));
                    }
                    ui.add_space(10.0);
                });
                false
            });
            if close {
                self.qr_menu = None;
            }
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        self.props_dialog(ctx, escape);
        self.account_dialog(ctx, escape);
        self.confirm_dialog(ctx, escape);
        self.update_dialogs(ctx, escape);

        if let Some(buf) = &mut self.port_dialog {
            let mut done = false;
            let mut chosen = None;
            dialog_window("port").show(ctx, |ui| {
                dialog_title(ui, "變更埠號");
                ui.label(RichText::new("其他裝置要用新的埠號連線，例如 8080、8000。").color(theme::TEXT2));
                ui.add_space(12.0);
                text_field(ui, buf);
                let parsed = buf.trim().parse::<u16>().ok().filter(|p| *p > 0);
                if parsed.is_none() {
                    ui.label(RichText::new("請輸入 1–65535 的數字").small().color(theme::RED));
                }
                dialog_buttons(ui, |ui| {
                    if theme::pill(ui, Pill::Filled, None, "確定", parsed.is_some()).clicked() || (enter && parsed.is_some()) {
                        chosen = parsed;
                    }
                    if theme::pill(ui, Pill::Text, None, "取消", true).clicked() || escape {
                        done = true;
                    }
                });
            });
            if let Some(p) = chosen {
                self.server.set_port(p);
                done = true;
            }
            if done {
                self.port_dialog = None;
            }
        }

        if let Some((parent, name)) = &mut self.vdir_dialog {
            let mut done = false;
            let mut ok = false;
            dialog_window("vdir").show(ctx, |ui| {
                dialog_title(ui, "新增虛擬資料夾");
                ui.label(RichText::new("虛擬資料夾不對應實際的資料夾，用來整理分享項目。").color(theme::TEXT2));
                ui.add_space(12.0);
                text_field(ui, name);
                let valid = !name.trim().is_empty() && !name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']);
                dialog_buttons(ui, |ui| {
                    if theme::pill(ui, Pill::Filled, None, "建立", valid).clicked() || (enter && valid) {
                        ok = true;
                    }
                    if theme::pill(ui, Pill::Text, None, "取消", true).clicked() || escape {
                        done = true;
                    }
                });
            });
            if ok {
                self.server.add_virtual(parent.clone(), name.trim().to_string());
                done = true;
            }
            if done {
                self.vdir_dialog = None;
            }
        }

        if self.close_dialog {
            let mut choice = 0;
            dialog_window("close").show(ctx, |ui| {
                dialog_title(ui, "要讓伺服器繼續運行嗎？");
                ui.label(RichText::new("繼續運行的話，視窗會縮到右下角的系統匣，其他人仍然可以下載檔案。").color(theme::TEXT2));
                ui.add_space(4.0);
                dialog_buttons(ui, |ui| {
                    if theme::pill(ui, Pill::Filled, Some("check"), "繼續在背景運行", true).clicked() {
                        choice = 1;
                    }
                    if theme::pill(ui, Pill::Outlined, Some("power"), "停止並結束", true).clicked() {
                        choice = 2;
                    }
                    if theme::pill(ui, Pill::Text, None, "取消", true).clicked() || escape {
                        choice = 3;
                    }
                });
            });
            match choice {
                1 => {
                    self.close_dialog = false;
                    if self.tray.is_some() {
                        win::hide(self.hwnd);
                    } else {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                }
                2 => self.begin_exit(),
                3 => self.close_dialog = false,
                _ => {}
            }
        }

        if self.exiting {
            dialog_window("exiting").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("正在停止伺服器…").font(theme::regular(16.0)));
                });
            });
        }
    }

    fn drop_overlay(&self, ctx: &egui::Context) {
        if ctx.input(|i| i.raw.hovered_files.is_empty()) {
            return;
        }
        let target = match self.target_parent() {
            Some(uri) => server::percent_decode(&uri),
            None => "/".to_string(),
        };
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("drop")));
        let rect = ctx.screen_rect();
        painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgba_unmultiplied(11, 87, 208, 30));
        painter.rect_stroke(rect.shrink(6.0), CornerRadius::same(16), egui::Stroke::new(2.0_f32, theme::PRIMARY), egui::StrokeKind::Inside);
        let text = format!("放開滑鼠，加入分享到 {target}");
        let font = theme::bold(16.0);
        let galley = painter.layout_no_wrap(text, font, Color32::WHITE);
        let pill = Rect::from_center_size(egui::pos2(rect.center().x, rect.bottom() - 70.0), galley.size() + egui::vec2(48.0, 24.0));
        painter.rect_filled(pill, CornerRadius::same(24), theme::PRIMARY);
        painter.galley(pill.center() - galley.size() / 2.0, galley, Color32::WHITE);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.quit_req.swap(false, Ordering::SeqCst) {
            self.begin_exit();
        }
        if self.exit_now.load(Ordering::SeqCst) {
            self.allow_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close {
            let active = self.server.shared.lock().unwrap().status.is_active();
            if active || self.exiting {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if !self.exiting {
                    self.close_dialog = true;
                }
            }
        }

        let mut actions = Vec::new();
        self.update_tick(&mut actions);
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
        if !dropped.is_empty() {
            actions.push(Action::AddPaths(self.target_parent(), dropped));
        }

        let shared = self.server.shared.clone();
        {
            let s = shared.lock().unwrap();
            self.port = s.port;
            // 隱藏 GUI 自己和網頁介面連 API 的連線
            let rows: Vec<&Map<String, Value>> = s
                .conns
                .iter()
                .filter(|c| {
                    let ip = c.get("ip").and_then(Value::as_str).unwrap_or("");
                    let path = c.get("path").and_then(Value::as_str).unwrap_or("");
                    !(server::is_loopback(ip) && path.starts_with("/~/"))
                })
                .collect();
            self.top_bar(ctx, &s, &mut actions);
            self.status_bar(ctx, &s, rows.len());
            self.vfs_panel(ctx, &s, &mut actions);
            self.main_area(ctx, &s, &rows, &mut actions);
            self.menus(ctx, &s, &mut actions);

            let state = match &s.status {
                Status::Running => format!("運行中 :{}，{} 個連線", s.port, rows.len()),
                Status::Starting => "啟動中".into(),
                Status::Stopping => "停止中".into(),
                Status::Stopped => "已停止".into(),
                Status::Failed(_) => "發生錯誤".into(),
            };
            let tip = format!("HFS – {state}");
            if tip != self.tray_tip {
                if let Some(t) = &self.tray {
                    let _ = t.set_tooltip(Some(&tip));
                }
                self.tray_tip = tip;
            }
        }
        self.dialogs(ctx);
        self.drop_overlay(ctx);
        for a in actions {
            self.run(a);
        }
        ctx.request_repaint_after(Duration::from_secs(1));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 保險：不論怎麼結束，都不留下我們開的 hfs.exe
        self.server.kill_child();
    }
}

/// 記錄的一列：時間｜IP｜動作標籤｜路徑｜大小
fn log_row(ui: &mut egui::Ui, line: &LogLine) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 30.0), Sense::hover());
    let p = ui.painter();
    if resp.hovered() {
        p.rect_filled(rect, CornerRadius::same(8), theme::CARD);
    }
    let y = rect.center().y;
    let mut x = rect.left() + 8.0;
    p.text(egui::pos2(x, y), Align2::LEFT_CENTER, &line.time, egui::FontId::monospace(12.5), theme::TEXT3);
    x += 74.0;
    let right = rect.right() - 8.0;
    match &line.req {
        Some(r) => {
            let ip = theme::elide(ui, &r.ip, theme::regular(13.5), theme::TEXT2, 128.0);
            p.galley(egui::pos2(x, y - ip.size().y / 2.0), ip, theme::TEXT2);
            x += 136.0;
            let (label, fg, bg) = req_badge(r);
            let badge = Rect::from_min_size(egui::pos2(x, y - 11.0), egui::vec2(52.0, 22.0));
            p.rect_filled(badge, CornerRadius::same(11), bg);
            p.text(badge.center(), Align2::CENTER_CENTER, &label, theme::bold(12.0), fg);
            x += 64.0;
            let mut tail = String::new();
            if let Some(size) = r.size {
                tail = server::fmt_bytes(size);
            }
            if r.parts > 1 {
                tail = format!("{} 段 · {tail}", r.parts);
            }
            if let Some(user) = &r.user {
                tail = if tail.is_empty() { user.clone() } else { format!("{user} · {tail}") };
            }
            let tail_w = if tail.is_empty() { 0.0 } else { theme::text_width(ui, &tail, theme::regular(13.0)) + 12.0 };
            let (ic, color) = theme::file_icon(&r.path, r.path.ends_with('/'));
            theme::paint_icon(ui, ic, Rect::from_min_size(egui::pos2(x, y - 9.0), Vec2::splat(18.0)), color);
            x += 26.0;
            let path = theme::elide(ui, &r.path, theme::regular(14.0), theme::TEXT, right - x - tail_w);
            p.galley(egui::pos2(x, y - path.size().y / 2.0), path, theme::TEXT);
            if !tail.is_empty() {
                p.text(egui::pos2(right, y), Align2::RIGHT_CENTER, tail, theme::regular(13.0), theme::TEXT3);
            }
        }
        None => {
            let (ic, color, text_color) = match line.kind {
                LogKind::Info => ("info", theme::PRIMARY, theme::TEXT),
                LogKind::Error => ("error", theme::RED, theme::RED),
                _ => ("terminal", theme::TEXT3, theme::TEXT2),
            };
            theme::paint_icon(ui, ic, Rect::from_min_size(egui::pos2(x, y - 8.0), Vec2::splat(16.0)), color);
            x += 26.0;
            let text = theme::elide(ui, &line.text, theme::regular(13.5), text_color, right - x);
            p.galley(egui::pos2(x, y - text.size().y / 2.0), text, text_color);
        }
    }
    if line.req.is_some() || line.text.chars().count() > 60 {
        resp.on_hover_text(&line.text);
    }
}

fn req_badge(r: &Req) -> (String, Color32, Color32) {
    if r.status >= 400 {
        return (r.status.to_string(), theme::RED, theme::RED_BG);
    }
    let (t, fg, bg) = match r.method.as_str() {
        "PUT" | "POST" => ("上傳", theme::PURPLE, theme::PURPLE_BG),
        "DELETE" => ("刪除", theme::RED, theme::RED_BG),
        _ if r.path.ends_with('/') => ("瀏覽", theme::PRIMARY, theme::BLUE_BG),
        _ if r.parts > 1 => ("多通道", theme::GREEN, theme::GREEN_BG),
        _ => ("下載", theme::GREEN, theme::GREEN_BG),
    };
    (t.to_string(), fg, bg)
}

fn progress(ui: &mut egui::Ui, p: f32) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 20.0), Sense::hover());
    let bar = Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 3.0), egui::vec2(w - 44.0, 6.0));
    let painter = ui.painter();
    painter.rect_filled(bar, CornerRadius::same(3), theme::SEARCH);
    let mut done = bar;
    done.set_width(bar.width() * p.clamp(0.0, 1.0));
    painter.rect_filled(done, CornerRadius::same(3), theme::PRIMARY);
    painter.text(egui::pos2(rect.right(), rect.center().y), Align2::RIGHT_CENTER, format!("{:.0}%", p * 100.0), theme::regular(12.0), theme::TEXT2);
}

fn dialog_window(id: &str) -> egui::Window<'static> {
    egui::Window::new(id.to_string())
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .min_width(380.0)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
}

fn dialog_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).font(theme::regular(22.0)).color(theme::TEXT));
    ui.add_space(8.0);
}

fn dialog_buttons(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(16.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
}

fn text_field(ui: &mut egui::Ui, buf: &mut String) {
    let edit = egui::TextEdit::singleline(buf)
        .font(theme::regular(16.0))
        .margin(egui::vec2(14.0, 10.0))
        .desired_width(f32::INFINITY);
    ui.add(edit).request_focus();
}

fn parent_of(uri: &str) -> Option<String> {
    let trimmed = uri.trim_end_matches('/');
    let idx = trimmed.rfind('/')?;
    let parent = &trimmed[..=idx];
    (parent != "/").then(|| parent.to_string())
}

fn make_tray(ctx: &egui::Context, hwnd: isize, quit_req: Arc<AtomicBool>) -> Result<TrayIcon, String> {
    let data = eframe::icon_data::from_png_bytes(icon::TRAY_PNG).map_err(|e| e.to_string())?;
    let icon = tray_icon::Icon::from_rgba(data.rgba, data.width, data.height).map_err(|e| e.to_string())?;
    let show = MenuItem::new("顯示視窗", true, None);
    let quit = MenuItem::new("停止伺服器並結束", true, None);
    let menu = Menu::new();
    menu.append(&show).map_err(|e| e.to_string())?;
    menu.append(&quit).map_err(|e| e.to_string())?;
    let (show_id, quit_id) = (show.id().clone(), quit.id().clone());
    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("HFS")
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()
        .map_err(|e| e.to_string())?;

    // 視窗隱藏時 egui 不會跑 update，所以直接用 Win32 把視窗叫出來
    let c = ctx.clone();
    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        let open = matches!(
            e,
            TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }
                | TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }
        );
        if open {
            win::show(hwnd);
            c.request_repaint();
        }
    }));
    let c = ctx.clone();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        if e.id == quit_id {
            quit_req.store(true, Ordering::SeqCst);
        }
        if e.id == show_id || e.id == quit_id {
            win::show(hwnd);
            c.request_repaint();
        }
    }));
    Ok(tray)
}

#[cfg(test)]
mod tests {
    use super::parent_of;

    #[test]
    fn parents() {
        assert_eq!(parent_of("/a.txt"), None);
        assert_eq!(parent_of("/dir/"), None);
        assert_eq!(parent_of("/dir/a.txt"), Some("/dir/".into()));
        assert_eq!(parent_of("/dir/sub/"), Some("/dir/".into()));
    }
}
