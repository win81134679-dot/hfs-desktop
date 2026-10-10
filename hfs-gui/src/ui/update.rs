//! HFS 更新：節流檢查、提醒（立即升級／稍後提醒／略過這個版本）、升級進度

use super::{dialog_buttons, dialog_title, dialog_window, Action, App};
use crate::prefs;
use crate::server::{LogKind, Shared, Status, UpdatePhase};
use crate::theme::{self, Pill};
use crate::updater::{self, Release};
use eframe::egui::{self, RichText};
use std::time::{Duration, Instant};

const DAY: u64 = 24 * 3600;
/// 查詢失敗時的退避
const RETRY_AFTER_FAIL: u64 = 6 * 3600;
/// 手動「檢查更新」的最短間隔，防連點
const MANUAL_MIN_GAP: Duration = Duration::from_secs(60);

fn fmt_time(secs: u64) -> String {
    chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "—".into())
}

fn fmt_date(iso: &str) -> String {
    iso.get(..10).unwrap_or(iso).to_string()
}

impl App {
    fn latest_release(&self) -> Option<Release> {
        self.prefs.latest.as_ref().and_then(Release::from_json)
    }

    /// 目前版本（測試時可用 HFS_GUI_FAKE_VERSION 假裝是舊版）
    fn current_version(&self, s: &Shared) -> Option<String> {
        self.fake_version.clone().or_else(|| s.hfs_version.clone())
    }

    /// 有比目前新、而且沒被略過的版本
    pub(super) fn pending_update(&self, s: &Shared) -> Option<Release> {
        let rel = self.latest_release()?;
        let cur = self.current_version(s)?;
        (updater::is_newer(&rel, &cur) && self.prefs.skipped_version.as_deref() != Some(rel.tag.as_str())).then_some(rel)
    }

    /// 每幀呼叫：收檢查結果、到期才自動查、決定要不要跳提醒
    pub(super) fn update_tick(&mut self, actions: &mut Vec<Action>) {
        let now = prefs::now();
        let (running, version, result, checking, idle) = {
            let mut s = self.server.shared.lock().unwrap();
            (s.status == Status::Running, self.current_version(&s), s.update.result.take(), s.update.checking, s.update.phase == UpdatePhase::Idle)
        };
        if let Some(result) = result {
            let manual = std::mem::take(&mut self.manual_check);
            match result {
                Ok(rel) => {
                    self.prefs.latest = Some(rel.to_json());
                    self.prefs.last_checked = now;
                    self.prefs.next_check = now + DAY;
                    actions.push(Action::SavePrefs);
                    let newer = version.as_deref().is_some_and(|v| updater::is_newer(&rel, v));
                    if manual {
                        if newer {
                            self.update_dialog = true;
                        } else {
                            self.server.log(LogKind::Info, format!("HFS 已經是最新版（{}）", rel.tag));
                        }
                    }
                }
                Err(e) => {
                    self.prefs.next_check = now + RETRY_AFTER_FAIL;
                    actions.push(Action::SavePrefs);
                    let kind = if manual { LogKind::Error } else { LogKind::Console };
                    self.server.log(kind, format!("檢查更新失敗：{e}"));
                }
            }
        }
        // 自動檢查：一天最多一次，時間落地在 prefs，重開程式也不會多查
        if self.prefs.auto_update && running && version.is_some() && !checking && now >= self.prefs.next_check {
            self.prefs.next_check = now + DAY;
            actions.push(Action::SavePrefs);
            self.server.check_update();
        }
        // 跳提醒：每次開程式最多一次，而且要過了「稍後提醒」的時間
        if !self.update_prompted && idle && now >= self.prefs.remind_after {
            let s = self.server.shared.lock().unwrap();
            let pending = self.pending_update(&s).is_some();
            drop(s);
            if pending {
                self.update_prompted = true;
                self.update_dialog = true;
            }
        }
    }

    pub(super) fn check_now(&mut self) {
        if self.last_manual_check.is_some_and(|t| t.elapsed() < MANUAL_MIN_GAP) {
            self.server.log(LogKind::Info, "剛剛才檢查過，請稍等一分鐘再試");
            return;
        }
        self.last_manual_check = Some(Instant::now());
        self.manual_check = true;
        self.prefs.next_check = prefs::now() + DAY;
        let _ = prefs::save(&self.prefs);
        self.server.check_update();
    }

    pub(super) fn update_dialogs(&mut self, ctx: &egui::Context, escape: bool) {
        let (phase, first) = {
            let s = self.server.shared.lock().unwrap();
            (s.update.phase.clone(), s.update.first_install)
        };

        // 升級進行中／完成／失敗
        if phase != UpdatePhase::Idle {
            let mut close = false;
            // 每個階段用不同的視窗 id，避免沿用上一個階段的尺寸
            let id = match phase {
                UpdatePhase::Downloading(..) => "upgrade-download",
                UpdatePhase::Installing | UpdatePhase::Restarting => "upgrade-install",
                _ => "upgrade-result",
            };
            dialog_window(id).min_width(440.0).show(ctx, |ui| match &phase {
                UpdatePhase::Downloading(done, total) => {
                    dialog_title(ui, if first { "第一次使用：正在下載 HFS" } else { "正在下載新版 HFS（多通道）" });
                    let p = if *total > 0 { *done as f32 / *total as f32 } else { 0.0 };
                    ui.add(egui::ProgressBar::new(p).fill(theme::PRIMARY).desired_height(8.0));
                    if *total > 0 {
                        ui.label(RichText::new(format!("{} / {}", crate::server::fmt_bytes(*done as f64), crate::server::fmt_bytes(*total as f64))).small().color(theme::TEXT2));
                    } else {
                        ui.label(RichText::new("正在查詢最新版本…").small().color(theme::TEXT2));
                    }
                    ui.label(
                        RichText::new(if first { "從 GitHub 下載官方 HFS，完成後自動套用雲端硬碟風格與預設設定" } else { "下載期間伺服器照常運作" })
                            .small()
                            .color(theme::TEXT3),
                    );
                }
                UpdatePhase::Installing | UpdatePhase::Restarting => {
                    dialog_title(ui, if first { "正在安裝 HFS" } else { "正在安裝新版 HFS" });
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(if phase == UpdatePhase::Installing { "停止伺服器、替換程式檔…" } else { "重新啟動伺服器…" });
                    });
                    ui.label(RichText::new("伺服器會短暫中斷幾秒鐘").small().color(theme::TEXT3));
                }
                UpdatePhase::Done(msg) => {
                    dialog_title(ui, "升級完成");
                    ui.label(RichText::new(msg).color(theme::GREEN));
                    ui.label(RichText::new("設定、帳號和分享清單都保留著。舊版備份為 hfs.exe.old。").small().color(theme::TEXT3));
                    dialog_buttons(ui, |ui| {
                        if theme::pill(ui, Pill::Filled, None, "確定", true).clicked() {
                            close = true;
                        }
                    });
                }
                UpdatePhase::Failed(msg) => {
                    dialog_title(ui, if first { "HFS 下載失敗" } else { "升級沒有完成" });
                    ui.label(RichText::new(msg).color(theme::RED));
                    ui.label(
                        RichText::new(if first { "請確認網路連線，再按右上角「啟動伺服器」重試。" } else { "伺服器維持原本的版本，可以稍後再試。" })
                            .small()
                            .color(theme::TEXT3),
                    );
                    dialog_buttons(ui, |ui| {
                        if theme::pill(ui, Pill::Filled, None, "確定", true).clicked() {
                            close = true;
                        }
                    });
                }
                UpdatePhase::Idle => {}
            });
            if close || (escape && matches!(phase, UpdatePhase::Done(_) | UpdatePhase::Failed(_))) {
                self.server.clear_update_phase();
            }
            ctx.request_repaint_after(Duration::from_millis(200));
            return;
        }

        if !self.update_dialog {
            return;
        }
        let (rel, cur) = {
            let s = self.server.shared.lock().unwrap();
            (self.pending_update(&s), self.current_version(&s))
        };
        let Some(rel) = rel else {
            self.update_dialog = false;
            return;
        };
        let mut choice = 0;
        dialog_window("update").min_width(480.0).show(ctx, |ui| {
            dialog_title(ui, "HFS 有新版本");
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("目前 {}", cur.as_deref().unwrap_or("?"))).color(theme::TEXT2));
                ui.label(RichText::new("→").color(theme::TEXT3));
                ui.label(RichText::new(&rel.tag).font(theme::bold(16.0)).color(theme::PRIMARY));
                if !rel.published.is_empty() {
                    ui.label(RichText::new(format!("（{} 發布）", fmt_date(&rel.published))).small().color(theme::TEXT3));
                }
            });
            let notes = updater::short_notes(&rel.notes, 14);
            if !notes.is_empty() {
                ui.add_space(8.0);
                ui.label(RichText::new("更新內容").font(theme::bold(14.0)));
                egui::Frame::new().fill(theme::CARD).corner_radius(egui::CornerRadius::same(10)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(notes).small().color(theme::TEXT2));
                    });
                });
            }
            ui.add_space(6.0);
            ui.label(RichText::new(format!("升級會下載約 {}，伺服器中斷幾秒後自動重新啟動；設定和分享清單不受影響。", crate::server::fmt_bytes(rel.zip_size as f64))).small().color(theme::TEXT3));
            dialog_buttons(ui, |ui| {
                if theme::pill(ui, Pill::Filled, Some("upload"), "立即升級", true).clicked() || self.test_upgrade {
                    choice = 1;
                }
                if theme::pill(ui, Pill::Outlined, None, "稍後提醒", true).clicked() || escape {
                    choice = 2;
                }
                if theme::pill(ui, Pill::Text, None, "略過這個版本", true).clicked() {
                    choice = 3;
                }
            });
        });
        match choice {
            1 => {
                self.update_dialog = false;
                self.test_upgrade = false;
                self.server.upgrade(rel);
            }
            2 => {
                self.update_dialog = false;
                self.prefs.remind_after = prefs::now() + DAY;
                let _ = prefs::save(&self.prefs);
                self.server.log(LogKind::Info, format!("好的，明天再提醒升級到 {}", rel.tag));
            }
            3 => {
                self.update_dialog = false;
                self.prefs.skipped_version = Some(rel.tag.clone());
                let _ = prefs::save(&self.prefs);
                self.server.log(LogKind::Info, format!("已略過 {}，有更新的版本時再提醒", rel.tag));
            }
            _ => {}
        }
    }

    /// 設定分頁裡的「更新」卡片
    pub(super) fn update_card(&mut self, ui: &mut egui::Ui, s: &Shared, actions: &mut Vec<Action>) {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            theme::card_header(ui, "cloudUp", "更新", |_| {});
            ui.add_space(6.0);
            let cur = self.current_version(s).unwrap_or_else(|| "（伺服器未運行）".into());
            let latest = self.latest_release();
            let pending = self.pending_update(s);
            egui::Grid::new("update_grid").num_columns(2).spacing([24.0, 8.0]).show(ui, |ui| {
                ui.label(RichText::new("目前版本").color(theme::TEXT2));
                ui.label(RichText::new(&cur).font(theme::bold(14.5)));
                ui.end_row();
                ui.label(RichText::new("最新版本").color(theme::TEXT2));
                match &latest {
                    Some(r) => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&r.tag).font(theme::bold(14.5)));
                            if pending.is_some() {
                                theme::chip(ui, "可以升級", theme::PRIMARY, theme::BLUE_BG, false);
                            } else if self.prefs.skipped_version.as_deref() == Some(r.tag.as_str()) {
                                theme::chip(ui, "已略過", theme::TEXT2, theme::CARD, false);
                            } else {
                                theme::chip(ui, "已是最新", theme::GREEN, theme::GREEN_BG, false);
                            }
                        });
                    }
                    None => {
                        ui.label(RichText::new("還沒檢查過").color(theme::TEXT3));
                    }
                }
                ui.end_row();
                ui.label(RichText::new("上次檢查").color(theme::TEXT2));
                ui.label(if self.prefs.last_checked > 0 { fmt_time(self.prefs.last_checked) } else { "—".into() });
                ui.end_row();
            });
            ui.add_space(8.0);
            if ui.checkbox(&mut self.prefs.auto_update, "自動檢查更新（每天最多一次，有新版會提醒）").changed() {
                actions.push(Action::SavePrefs);
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let checking = s.update.checking;
                if theme::pill(ui, Pill::Outlined, Some("history"), if checking { "檢查中…" } else { "檢查更新" }, !checking && s.status == Status::Running).clicked() {
                    actions.push(Action::CheckUpdate);
                }
                if let Some(r) = &pending {
                    if theme::pill(ui, Pill::Filled, Some("upload"), &format!("升級到 {}", r.tag), s.update.phase == UpdatePhase::Idle).clicked() {
                        actions.push(Action::OpenUpdate);
                    }
                }
                if let Some(skipped) = &self.prefs.skipped_version {
                    if theme::pill(ui, Pill::Text, None, &format!("取消略過 {skipped}"), true).clicked() {
                        actions.push(Action::Unskip);
                    }
                }
            });
        });
    }
}
