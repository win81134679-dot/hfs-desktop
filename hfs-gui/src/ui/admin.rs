//! 帳號、設定分頁，以及「屬性與權限」「帳號」對話框

use super::{dialog_buttons, dialog_title, dialog_window, Action, App};
use crate::server::{Shared, Status};
use crate::theme::{self, Pill};
use eframe::egui::{self, RichText, Vec2};
use serde_json::{json, Map, Value};

// ---------- 權限 ----------

/// HFS 的 Who 描述：true / false / "*" / "admin" / [帳號…] / "can_xxx"
#[derive(Clone, PartialEq)]
pub enum Who {
    Inherit,
    Anyone,
    LoggedIn,
    Admin,
    NoOne,
    Accounts(Vec<String>),
    Other(Value),
}

impl Who {
    fn from(v: Option<&Value>) -> Self {
        match v {
            None | Some(Value::Null) => Who::Inherit,
            Some(Value::Bool(true)) => Who::Anyone,
            Some(Value::Bool(false)) => Who::NoOne,
            Some(Value::String(s)) if s == "*" => Who::LoggedIn,
            Some(Value::String(s)) if s == "admin" => Who::Admin,
            Some(Value::Array(a)) => Who::Accounts(a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()),
            Some(other) => Who::Other(other.clone()),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Who::Inherit => Value::Null,
            Who::Anyone => json!(true),
            Who::NoOne => json!(false),
            Who::LoggedIn => json!("*"),
            Who::Admin => json!("admin"),
            Who::Accounts(list) => json!(list),
            Who::Other(v) => v.clone(),
        }
    }

    fn label(&self) -> String {
        match self {
            Who::Inherit => "繼承上層".into(),
            Who::Anyone => "任何人".into(),
            Who::LoggedIn => "已登入的帳號".into(),
            Who::Admin => "只有管理員".into(),
            Who::NoOne => "禁止".into(),
            Who::Accounts(l) if l.is_empty() => "指定帳號（尚未選擇）".into(),
            Who::Accounts(l) => format!("指定帳號：{}", l.join("、")),
            Who::Other(v) => match v.as_str() {
                Some(s) if s.starts_with("can_") => format!("同「{}」", perm_title(s)),
                _ => "自訂".into(),
            },
        }
    }
}

const PERMS: [(&str, &str, &str); 6] = [
    ("can_read", "下載", "誰可以下載這個項目"),
    ("can_see", "顯示在清單裡", "看不到的人在清單中找不到它"),
    ("can_list", "瀏覽資料夾內容", "誰可以打開資料夾看裡面有什麼"),
    ("can_archive", "打包成 ZIP 下載", "多選或整個資料夾下載"),
    ("can_upload", "上傳", "只有對應實際資料夾的項目能上傳"),
    ("can_delete", "刪除與重新命名", "會真的刪除電腦上的檔案"),
];

fn perm_title(key: &str) -> &'static str {
    PERMS.iter().find(|p| p.0 == key).map_or("其他權限", |p| p.1)
}

/// 根目錄沒有上層可繼承時的 HFS 預設值
fn default_label(key: &str) -> String {
    match key {
        "can_read" => "任何人".into(),
        "can_upload" | "can_delete" => "只有管理員".into(),
        _ => "同「下載」".into(),
    }
}

pub struct PropsForm {
    uri: String,
    is_root: bool,
    is_folder: bool,
    source: Option<String>,
    orig_name: String,
    name: String,
    /// (權限鍵, 目前設定, 繼承來的值的說明)
    perms: Vec<(&'static str, Who, String)>,
}

impl PropsForm {
    pub fn from_node(uri: &str, node: &Value) -> Self {
        let is_root = uri == "/";
        let is_folder = is_root || node["type"].as_str() == Some("folder") || node.get("children").is_some_and(Value::is_array);
        let source = node["source"].as_str().map(str::to_owned);
        let name = node["name"].as_str().unwrap_or_default().to_string();
        let inherited = node.get("inherited");
        let perms = PERMS
            .iter()
            .filter(|(k, ..)| match *k {
                "can_list" => is_folder,
                "can_upload" => is_folder && source.is_some(),
                "can_delete" => source.is_some(),
                _ => true,
            })
            .map(|(k, ..)| {
                let inh = inherited.and_then(|i| i.get(*k)).filter(|v| !v.is_null());
                let inh_label = inh.map(|v| Who::from(Some(v)).label()).unwrap_or_else(|| default_label(k));
                (*k, Who::from(node.get(*k)), inh_label)
            })
            .collect();
        Self { uri: uri.to_string(), is_root, is_folder, source, orig_name: name.clone(), name, perms }
    }
}

/// 依網址在 get_vfs 的樹裡找節點（網址編碼方式和左邊樹一致）
pub fn find_node(root: &Value, uri: &str) -> Option<Value> {
    fn walk(node: &Value, cur: &str, target: &str) -> Option<Value> {
        if cur == target {
            return Some(node.clone());
        }
        for child in node.get("children").and_then(Value::as_array).into_iter().flatten() {
            let name = child["name"].as_str().unwrap_or_default();
            let folder = child["type"].as_str() == Some("folder") || child.get("children").is_some_and(Value::is_array);
            let child_uri = format!("{cur}{}{}", crate::server::encode_uri_component(name), if folder { "/" } else { "" });
            if target.starts_with(&child_uri) {
                if let Some(found) = walk(child, &child_uri, target) {
                    return Some(found);
                }
            }
        }
        None
    }
    walk(root, "/", uri)
}

// ---------- 帳號 ----------

pub struct AccountForm {
    is_new: bool,
    username: String,
    password: String,
    password2: String,
    admin: bool,
    disabled: bool,
    notes: String,
    has_password: bool,
}

impl AccountForm {
    fn new() -> Self {
        Self { is_new: true, username: String::new(), password: String::new(), password2: String::new(), admin: false, disabled: false, notes: String::new(), has_password: false }
    }

    fn from_account(a: &Value) -> Self {
        Self {
            is_new: false,
            username: a["username"].as_str().unwrap_or_default().to_string(),
            password: String::new(),
            password2: String::new(),
            admin: a["admin"].as_bool().unwrap_or(false),
            disabled: a["disabled"].as_bool().unwrap_or(false),
            notes: a["notes"].as_str().unwrap_or_default().to_string(),
            has_password: a["hasPassword"].as_bool().unwrap_or(false),
        }
    }
}

// ---------- 設定 ----------

pub struct SettingsForm {
    dirty: bool,
    title: String,
    max_kbps: String,
    max_kbps_per_ip: String,
    max_downloads_per_ip: String,
    min_available_mb: String,
    dont_overwrite: bool,
    show_hidden: bool,
}

impl SettingsForm {
    fn from_config(c: &Value) -> Self {
        // HFS 用 0 或 Infinity（JSON 裡是 null）表示不限制，都顯示成空白
        let num = |k: &str| match &c[k] {
            Value::Number(n) if n.as_f64() == Some(0.0) && k != "min_available_mb" => String::new(),
            Value::Number(n) => n.as_f64().map(|v| if v.fract() == 0.0 { format!("{v:.0}") } else { v.to_string() }).unwrap_or_default(),
            Value::String(s) => s.clone(),
            _ => String::new(),
        };
        Self {
            dirty: false,
            title: c["title"].as_str().unwrap_or_default().to_string(),
            max_kbps: num("max_kbps"),
            max_kbps_per_ip: num("max_kbps_per_ip"),
            max_downloads_per_ip: num("max_downloads_per_ip"),
            min_available_mb: num("min_available_mb"),
            dont_overwrite: c["dont_overwrite_uploading"].as_bool().unwrap_or(false),
            show_hidden: c["show_hidden_files"].as_bool().unwrap_or(false),
        }
    }

    /// 轉成 set_config 的值；空白 = null（恢復預設）
    fn to_values(&self) -> Result<Value, String> {
        fn num(label: &str, s: &str, integer: bool) -> Result<Value, String> {
            let t = s.trim();
            if t.is_empty() {
                return Ok(Value::Null);
            }
            let v: f64 = t.parse().map_err(|_| format!("「{label}」要填數字"))?;
            if v < 0.0 || (integer && v.fract() != 0.0) {
                return Err(format!("「{label}」要填{}", if integer { "正整數" } else { "正數" }));
            }
            Ok(if integer { json!(v as u64) } else { json!(v) })
        }
        Ok(json!({
            "title": if self.title.trim().is_empty() { Value::Null } else { json!(self.title.trim()) },
            "max_kbps": num("總速度上限", &self.max_kbps, false)?,
            "max_kbps_per_ip": num("每個 IP 速度上限", &self.max_kbps_per_ip, false)?,
            "max_downloads_per_ip": num("每個 IP 同時下載數", &self.max_downloads_per_ip, true)?,
            "min_available_mb": num("保留磁碟空間", &self.min_available_mb, false)?,
            "dont_overwrite_uploading": self.dont_overwrite,
            "show_hidden_files": self.show_hidden,
        }))
    }
}

fn field(ui: &mut egui::Ui, buf: &mut String, hint: &str, password: bool, width: f32) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(buf)
            .hint_text(hint)
            .password(password)
            .font(theme::regular(15.0))
            .margin(egui::vec2(12.0, 8.0))
            .desired_width(width),
    )
}

fn label_col(ui: &mut egui::Ui, title: &str, hint: &str, width: f32) {
    ui.vertical(|ui| {
        ui.set_width(width);
        ui.label(RichText::new(title).font(theme::regular(14.5)).color(theme::TEXT));
        if !hint.is_empty() {
            ui.add(egui::Label::new(RichText::new(hint).small().color(theme::TEXT3)).wrap());
        }
    });
}

impl App {
    // ---------- 帳號分頁 ----------

    pub(super) fn accounts_page(&mut self, ui: &mut egui::Ui, s: &Shared, actions: &mut Vec<Action>) {
        let running = s.status == Status::Running;
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height((ui.available_height() - 8.0).max(200.0));
            theme::card_header(ui, "people", "帳號", |ui| {
                if theme::pill(ui, Pill::Filled, Some("personAdd"), "新增帳號", running).clicked() {
                    self.account_form = Some(AccountForm::new());
                }
            });
            ui.add(
                egui::Label::new(
                    RichText::new("訪客可以在分享網頁右上角登入。到左邊分享項目按右鍵 →「屬性與權限」，就能指定哪些帳號可以下載、上傳或刪除。")
                        .small()
                        .color(theme::TEXT3),
                )
                .wrap(),
            );
            ui.add_space(10.0);
            if !running {
                ui.label(RichText::new("伺服器未運行").color(theme::TEXT3));
                return;
            }
            if s.accounts.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.add(theme::icon("person", theme::OUTLINE).fit_to_exact_size(Vec2::splat(56.0)));
                    ui.label(RichText::new("還沒有任何帳號").font(theme::regular(17.0)).color(theme::TEXT2));
                    ui.label(RichText::new("沒有帳號時，所有訪客都是匿名的").small().color(theme::TEXT3));
                });
                return;
            }
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                for acc in &s.accounts {
                    let name = acc["username"].as_str().unwrap_or_default();
                    let w = ui.available_width();
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 60.0), egui::Sense::hover());
                    if resp.hovered() {
                        ui.painter().rect_filled(rect, egui::CornerRadius::same(12), theme::CARD);
                    }
                    let mut row = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(egui::vec2(12.0, 8.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
                    let ui = &mut row;
                    theme::avatar(ui, name, 40.0);
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(name).font(theme::bold(15.0)).color(theme::TEXT));
                            if acc["admin"].as_bool() == Some(true) {
                                theme::chip(ui, "管理員", theme::PRIMARY, theme::BLUE_BG, false);
                            }
                            if acc["disabled"].as_bool() == Some(true) {
                                theme::chip(ui, "已停用", theme::TEXT2, theme::CARD, false);
                            }
                            if acc["hasPassword"].as_bool() == Some(false) && acc["isGroup"].as_bool() != Some(true) {
                                theme::chip(ui, "沒有密碼", theme::AMBER, theme::AMBER_BG, false);
                            }
                        });
                        let notes = acc["notes"].as_str().unwrap_or_default();
                        if !notes.is_empty() {
                            ui.label(RichText::new(notes).small().color(theme::TEXT3));
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::icon_button_sized(ui, "delete", "刪除帳號", 36.0, theme::RED).clicked() {
                            self.confirm_delete = Some(name.to_string());
                        }
                        if theme::icon_button_sized(ui, "edit", "編輯", 36.0, theme::TEXT2).clicked() {
                            self.account_form = Some(AccountForm::from_account(acc));
                        }
                    });
                }
            });
        });
        let _ = actions;
    }

    pub(super) fn account_dialog(&mut self, ctx: &egui::Context, escape: bool) {
        let Some(f) = &mut self.account_form else { return };
        let mut close = escape;
        let mut save = None;
        dialog_window("account").min_width(440.0).show(ctx, |ui| {
            dialog_title(ui, if f.is_new { "新增帳號" } else { "編輯帳號" });
            ui.spacing_mut().item_spacing.y = 10.0;
            if f.is_new {
                field(ui, &mut f.username, "帳號名稱", false, f32::INFINITY);
            } else {
                ui.horizontal(|ui| {
                    theme::avatar(ui, &f.username, 32.0);
                    ui.label(RichText::new(&f.username).font(theme::bold(16.0)));
                });
            }
            let hint = if f.is_new { "密碼" } else if f.has_password { "新密碼（留空代表不變更）" } else { "設定密碼" };
            field(ui, &mut f.password, hint, true, f32::INFINITY);
            field(ui, &mut f.password2, "再輸入一次密碼", true, f32::INFINITY);
            field(ui, &mut f.notes, "備註（選填）", false, f32::INFINITY);
            ui.checkbox(&mut f.admin, "管理員（可以登入 HFS 網頁管理介面）");
            ui.checkbox(&mut f.disabled, "暫時停用這個帳號");
            let error = if f.is_new && f.username.trim().is_empty() {
                Some("請輸入帳號名稱")
            } else if f.is_new && f.password.is_empty() {
                Some("請設定密碼")
            } else if f.password != f.password2 {
                Some("兩次輸入的密碼不一樣")
            } else {
                None
            };
            if let Some(e) = error {
                if !(f.is_new && f.username.is_empty() && f.password.is_empty()) {
                    ui.label(RichText::new(e).small().color(theme::RED));
                }
            }
            dialog_buttons(ui, |ui| {
                if theme::pill(ui, Pill::Filled, None, if f.is_new { "建立" } else { "儲存" }, error.is_none()).clicked() {
                    let mut changes = json!({ "admin": f.admin, "disabled": f.disabled, "notes": f.notes.trim() });
                    if !f.password.is_empty() {
                        changes["password"] = json!(f.password);
                    }
                    save = Some((f.username.trim().to_string(), changes, f.is_new));
                }
                if theme::pill(ui, Pill::Text, None, "取消", true).clicked() {
                    close = true;
                }
            });
        });
        if let Some((username, changes, is_new)) = save {
            self.server.save_account(username, changes, is_new);
            close = true;
        }
        if close {
            self.account_form = None;
        }
    }

    pub(super) fn confirm_dialog(&mut self, ctx: &egui::Context, escape: bool) {
        let Some(name) = self.confirm_delete.clone() else { return };
        let mut close = escape;
        dialog_window("confirm").show(ctx, |ui| {
            dialog_title(ui, &format!("要刪除帳號「{name}」嗎？"));
            ui.label(RichText::new("這個帳號將無法再登入。分享項目權限裡指定的這個帳號也會失效。").color(theme::TEXT2));
            dialog_buttons(ui, |ui| {
                if theme::pill(ui, Pill::Filled, Some("delete"), "刪除", true).clicked() {
                    self.server.delete_account(name.clone());
                    close = true;
                }
                if theme::pill(ui, Pill::Text, None, "取消", true).clicked() {
                    close = true;
                }
            });
        });
        if close {
            self.confirm_delete = None;
        }
    }

    // ---------- 屬性與權限 ----------

    pub(super) fn props_dialog(&mut self, ctx: &egui::Context, escape: bool) {
        if self.props.is_none() {
            return;
        }
        let usernames = self.server.shared.lock().unwrap().usernames();
        let f = self.props.as_mut().unwrap();
        let mut close = escape;
        let mut save = false;
        dialog_window("props").min_width(560.0).show(ctx, |ui| {
            dialog_title(ui, "屬性與權限");
            ui.horizontal(|ui| {
                let (ic, color) = if f.is_root { ("home", theme::TEXT2) } else { theme::file_icon(&f.orig_name, f.is_folder) };
                ui.add(theme::icon(ic, color).fit_to_exact_size(Vec2::splat(22.0)));
                ui.label(RichText::new(if f.is_root { "/（根目錄）" } else { &f.orig_name }).font(theme::bold(15.0)));
            });
            ui.label(
                RichText::new(match &f.source {
                    Some(s) => s.clone(),
                    None if f.is_folder => "虛擬資料夾（不對應電腦上的資料夾）".into(),
                    None => String::new(),
                })
                .small()
                .color(theme::TEXT3),
            );
            ui.add_space(10.0);
            if !f.is_root {
                ui.horizontal(|ui| {
                    label_col(ui, "名稱", "訪客看到的名稱，不會改電腦上的檔名", 180.0);
                    field(ui, &mut f.name, "名稱", false, 300.0);
                });
                ui.add_space(8.0);
            }
            theme::section(ui, "誰可以…");
            for (key, who, inherited) in f.perms.iter_mut() {
                let (_, title, hint) = PERMS.iter().find(|p| p.0 == *key).copied().unwrap_or((key, key, ""));
                ui.horizontal(|ui| {
                    label_col(ui, title, hint, 180.0);
                    let selected = if *who == Who::Inherit { format!("繼承（{inherited}）") } else { who.label() };
                    egui::ComboBox::from_id_salt(("perm", *key)).width(300.0).selected_text(selected).show_ui(ui, |ui| {
                        ui.selectable_value(who, Who::Inherit, format!("繼承上層（目前：{inherited}）"));
                        ui.selectable_value(who, Who::Anyone, "任何人");
                        ui.selectable_value(who, Who::LoggedIn, "已登入的帳號");
                        let is_accounts = matches!(who, Who::Accounts(_));
                        if ui.selectable_label(is_accounts, "指定帳號…").clicked() && !is_accounts {
                            *who = Who::Accounts(Vec::new());
                        }
                        ui.selectable_value(who, Who::Admin, "只有管理員");
                        ui.selectable_value(who, Who::NoOne, "禁止");
                    });
                });
                if let Who::Accounts(list) = who {
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(192.0);
                        if usernames.is_empty() {
                            ui.label(RichText::new("還沒有帳號，請先到「帳號」分頁新增").small().color(theme::AMBER));
                        }
                        for u in &usernames {
                            let mut on = list.contains(u);
                            if ui.checkbox(&mut on, u).changed() {
                                if on {
                                    list.push(u.clone());
                                } else {
                                    list.retain(|x| x != u);
                                }
                            }
                        }
                    });
                }
                ui.add_space(6.0);
            }
            let name_ok = f.is_root || (!f.name.trim().is_empty() && !f.name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']));
            dialog_buttons(ui, |ui| {
                if theme::pill(ui, Pill::Filled, None, "儲存", name_ok).clicked() {
                    save = true;
                }
                if theme::pill(ui, Pill::Text, None, "取消", true).clicked() {
                    close = true;
                }
            });
        });
        if save {
            let mut props = Map::new();
            for (key, who, _) in &f.perms {
                props.insert(key.to_string(), who.to_value());
            }
            let name = f.name.trim().to_string();
            if !f.is_root && name != f.orig_name {
                props.insert("name".into(), json!(name));
                self.selected = None;
            }
            let label = if f.is_root { "/".to_string() } else { name };
            self.server.set_vfs_props(f.uri.clone(), Value::Object(props), format!("已更新「{label}」的屬性與權限"));
            close = true;
        }
        if close {
            self.props = None;
        }
    }

    // ---------- 設定分頁 ----------

    pub(super) fn settings_page(&mut self, ui: &mut egui::Ui, s: &Shared, actions: &mut Vec<Action>) {
        let running = s.status == Status::Running;
        if self.settings.as_ref().is_none_or(|f| !f.dirty) {
            if let Some(c) = &s.config {
                self.settings = Some(SettingsForm::from_config(c));
            }
        }
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::card_header(ui, "settings", "伺服器設定", |_| {});
                ui.add_space(6.0);
                let Some(f) = self.settings.as_mut().filter(|_| running) else {
                    ui.label(RichText::new("伺服器啟動後才能修改").color(theme::TEXT3));
                    return;
                };
                let lw = 240.0;
                let row = |ui: &mut egui::Ui, title: &str, hint: &str, add: &mut dyn FnMut(&mut egui::Ui) -> bool| {
                    let mut changed = false;
                    ui.horizontal(|ui| {
                        label_col(ui, title, hint, lw);
                        changed = add(ui);
                    });
                    ui.add_space(8.0);
                    changed
                };
                let mut dirty = false;
                theme::section(ui, "分享網頁");
                dirty |= row(ui, "網站標題", "顯示在瀏覽器分頁上", &mut |ui| field(ui, &mut f.title, "File server", false, 280.0).changed());
                theme::section(ui, "速度與連線");
                row(ui, "埠號", "改了之後，網址會跟著變", &mut |ui| {
                    ui.label(RichText::new(s.port.to_string()).font(theme::bold(15.0)));
                    if theme::pill(ui, Pill::Outlined, None, "變更…", true).clicked() {
                        actions.push(Action::EditPort);
                    }
                    false
                });
                let unit = |ui: &mut egui::Ui, t: &str| {
                    ui.label(RichText::new(t).color(theme::TEXT2));
                };
                dirty |= row(ui, "總速度上限", "所有訪客加起來；空白代表不限制", &mut |ui| {
                    let c = field(ui, &mut f.max_kbps, "不限制", false, 140.0).changed();
                    unit(ui, "KB/s");
                    c
                });
                dirty |= row(ui, "每個 IP 速度上限", "避免一個人佔滿頻寬", &mut |ui| {
                    let c = field(ui, &mut f.max_kbps_per_ip, "不限制", false, 140.0).changed();
                    unit(ui, "KB/s");
                    c
                });
                dirty |= row(ui, "每個 IP 同時下載數", "空白代表不限制", &mut |ui| {
                    let c = field(ui, &mut f.max_downloads_per_ip, "不限制", false, 140.0).changed();
                    unit(ui, "個");
                    c
                });
                theme::section(ui, "上傳");
                dirty |= row(ui, "同名檔案", "訪客上傳的檔名已經存在時", &mut |ui| ui.checkbox(&mut f.dont_overwrite, "自動在檔名後面加編號，不覆蓋").changed());
                dirty |= row(ui, "保留磁碟空間", "剩餘空間低於這個值就拒絕上傳", &mut |ui| {
                    let c = field(ui, &mut f.min_available_mb, "100", false, 140.0).changed();
                    unit(ui, "MB");
                    c
                });
                theme::section(ui, "其他");
                dirty |= row(ui, "隱藏檔案", "Windows 標成隱藏的檔案和資料夾", &mut |ui| ui.checkbox(&mut f.show_hidden, "也顯示在分享網頁上").changed());
                if dirty {
                    f.dirty = true;
                }
                let values = f.to_values();
                if let Err(e) = &values {
                    ui.label(RichText::new(e).color(theme::RED));
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(lw + 8.0);
                    if theme::pill(ui, Pill::Filled, Some("check"), "儲存設定", f.dirty && values.is_ok()).clicked() {
                        if let Ok(v) = values {
                            actions.push(Action::SaveConfig(v, "設定已儲存"));
                            f.dirty = false;
                        }
                    }
                    if theme::pill(ui, Pill::Text, None, "還原", f.dirty).clicked() {
                        f.dirty = false;
                    }
                });
            });
            ui.add_space(12.0);

            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::card_header(ui, "block", "封鎖的 IP", |_| {});
                ui.add(egui::Label::new(RichText::new("被封鎖的位址連不上這台伺服器。也可以在「目前連線」直接按封鎖。").small().color(theme::TEXT3)).wrap());
                ui.add_space(8.0);
                let rules = s.config.as_ref().and_then(|c| c["block"].as_array().cloned()).unwrap_or_default();
                if rules.is_empty() {
                    ui.label(RichText::new("目前沒有封鎖任何 IP").color(theme::TEXT3));
                }
                for r in &rules {
                    let ip = r["ip"].as_str().unwrap_or_default().to_string();
                    ui.horizontal(|ui| {
                        ui.add(theme::icon("block", theme::RED).fit_to_exact_size(Vec2::splat(18.0)));
                        ui.label(RichText::new(&ip).font(theme::bold(14.5)));
                        if let Some(c) = r["comment"].as_str() {
                            ui.label(RichText::new(c).small().color(theme::TEXT3));
                        }
                        if let Some(e) = r["expire"].as_str() {
                            ui.label(RichText::new(format!("到期：{e}")).small().color(theme::TEXT3));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if theme::pill(ui, Pill::Text, None, "解除封鎖", running).clicked() {
                                actions.push(Action::Unblock(ip.clone()));
                            }
                        });
                    });
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    field(ui, &mut self.block_input, "輸入 IP，例如 192.168.1.50", false, 260.0);
                    let ip = self.block_input.trim().to_string();
                    let valid = ip.parse::<std::net::IpAddr>().is_ok();
                    if theme::pill(ui, Pill::Outlined, Some("block"), "封鎖", running && valid).clicked() {
                        actions.push(Action::BlockIp(ip));
                        self.block_input.clear();
                    }
                });
            });
            ui.add_space(12.0);

            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::card_header(ui, "web", "桌面程式", |_| {});
                ui.add_space(6.0);
                let mut auto = self.autostart;
                if ui.checkbox(&mut auto, "開機時自動啟動（啟動後直接縮在系統匣）").changed() {
                    actions.push(Action::SetAutostart(auto));
                }
                if ui.checkbox(&mut self.prefs.start_in_tray, "打開程式時直接縮到系統匣").changed() {
                    actions.push(Action::SavePrefs);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("HFS 設定檔：{}", self.server.hfs_dir.join("config.yaml").display())).small().color(theme::TEXT3));
                    if theme::pill(ui, Pill::Text, Some("folder"), "開啟資料夾", true).clicked() {
                        actions.push(Action::Reveal(self.server.hfs_dir.display().to_string(), true));
                    }
                });
            });
            ui.add_space(12.0);
            self.update_card(ui, s, actions);
            ui.add_space(8.0);
        });
    }
}
