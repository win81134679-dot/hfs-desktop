//! Google 雲端硬碟（Material 3）風格：配色、字型、SVG 圖示和自繪元件。

use eframe::egui::{
    self, text::LayoutJob, Align2, Color32, CornerRadius, FontData, FontFamily, FontId, Margin, Rect, Response,
    Sense, Shadow, Stroke, StrokeKind, TextStyle, Ui, Vec2,
};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub const BG: Color32 = Color32::from_rgb(0xf8, 0xfa, 0xfd);
pub const SURFACE: Color32 = Color32::WHITE;
pub const CARD: Color32 = Color32::from_rgb(0xf0, 0xf4, 0xf9);
pub const SEARCH: Color32 = Color32::from_rgb(0xe9, 0xee, 0xf6);
pub const TEXT: Color32 = Color32::from_rgb(0x1f, 0x1f, 0x1f);
pub const TEXT2: Color32 = Color32::from_rgb(0x44, 0x47, 0x46);
pub const TEXT3: Color32 = Color32::from_rgb(0x74, 0x77, 0x75);
pub const PRIMARY: Color32 = Color32::from_rgb(0x0b, 0x57, 0xd0);
pub const SELECTED: Color32 = Color32::from_rgb(0xc2, 0xe7, 0xff);
pub const ON_SELECTED: Color32 = Color32::from_rgb(0x00, 0x1d, 0x35);
pub const DIVIDER: Color32 = Color32::from_rgb(0xe1, 0xe3, 0xe1);
pub const OUTLINE: Color32 = Color32::from_rgb(0xc4, 0xc7, 0xc5);
pub const HOVER: Color32 = Color32::from_rgba_premultiplied(10, 10, 10, 14);
pub const PRESS: Color32 = Color32::from_rgba_premultiplied(10, 10, 10, 26);
pub const GREEN: Color32 = Color32::from_rgb(0x13, 0x73, 0x33);
pub const GREEN_BG: Color32 = Color32::from_rgb(0xe6, 0xf4, 0xea);
pub const RED: Color32 = Color32::from_rgb(0xc5, 0x22, 0x1f);
pub const RED_BG: Color32 = Color32::from_rgb(0xfc, 0xe8, 0xe6);
pub const AMBER: Color32 = Color32::from_rgb(0xb0, 0x60, 0x00);
pub const AMBER_BG: Color32 = Color32::from_rgb(0xfe, 0xf7, 0xe0);
pub const BLUE_BG: Color32 = Color32::from_rgb(0xe8, 0xf0, 0xfe);
pub const PURPLE: Color32 = Color32::from_rgb(0x84, 0x30, 0xce);
pub const PURPLE_BG: Color32 = Color32::from_rgb(0xf3, 0xe8, 0xfd);

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

// ---------- 字型與樣式 ----------

pub fn apply(ctx: &egui::Context) {
    setup_fonts(ctx);
    ctx.set_theme(egui::Theme::Light);
    ctx.style_mut_of(egui::Theme::Light, |s| {
        use TextStyle::*;
        s.text_styles = [
            (Small, regular(12.0)),
            (Body, regular(14.5)),
            (Button, regular(14.5)),
            (Heading, bold(20.0)),
            (Monospace, FontId::monospace(13.0)),
        ]
        .into();
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(14.0, 6.0);
        s.spacing.interact_size.y = 30.0;
        s.spacing.menu_margin = Margin::same(6);
        s.spacing.window_margin = Margin::same(24);
        s.interaction.selectable_labels = false;

        let v = &mut s.visuals;
        *v = egui::Visuals::light();
        v.panel_fill = BG;
        v.window_fill = SURFACE;
        v.extreme_bg_color = SEARCH;
        v.faint_bg_color = CARD;
        v.code_bg_color = CARD;
        v.override_text_color = None;
        v.hyperlink_color = PRIMARY;
        v.selection.bg_fill = SELECTED;
        v.selection.stroke = Stroke::new(1.0_f32, ON_SELECTED);
        v.window_corner_radius = CornerRadius::same(24);
        v.menu_corner_radius = CornerRadius::same(10);
        v.window_stroke = Stroke::NONE;
        v.window_shadow = Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(45) };
        v.popup_shadow = Shadow { offset: [0, 3], blur: 12, spread: 0, color: Color32::from_black_alpha(40) };
        let w = &mut v.widgets;
        for ws in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            // 太圓的話勾選框會變成圓形，看起來像單選鈕
            ws.corner_radius = CornerRadius::same(4);
            ws.expansion = 0.0;
        }
        w.noninteractive.bg_stroke = Stroke::new(1.0_f32, DIVIDER);
        w.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
        w.inactive.weak_bg_fill = SURFACE;
        w.inactive.bg_fill = SEARCH;
        w.inactive.bg_stroke = Stroke::new(1.0_f32, OUTLINE);
        w.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT2);
        w.hovered.weak_bg_fill = Color32::from_rgb(0xf1, 0xf3, 0xf6);
        w.hovered.bg_fill = Color32::from_rgb(0xdf, 0xe5, 0xee);
        w.hovered.bg_stroke = Stroke::new(1.0_f32, OUTLINE);
        w.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
        w.active.weak_bg_fill = Color32::from_rgb(0xe3, 0xe7, 0xec);
        w.active.bg_fill = Color32::from_rgb(0xd3, 0xdb, 0xe6);
        w.active.bg_stroke = Stroke::new(1.0_f32, OUTLINE);
        w.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
        w.open.weak_bg_fill = SEARCH;
    });
    egui_extras::install_image_loaders(ctx);
}

/// 英數用 Segoe UI，中文用微軟正黑體 UI；另外準備一組粗體給標題
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut add = |name: &str, files: &[(&str, u32)]| -> bool {
        for (file, index) in files {
            if let Ok(bytes) = std::fs::read(format!(r"C:\Windows\Fonts\{file}")) {
                let mut data = FontData::from_owned(bytes);
                data.index = *index;
                fonts.font_data.insert(name.into(), Arc::new(data));
                return true;
            }
        }
        false
    };
    let latin = add("latin", &[("segoeui.ttf", 0)]);
    let latin_bold = add("latin-bold", &[("seguisb.ttf", 0), ("segoeuib.ttf", 0)]);
    let cjk = add("cjk", &[("msjh.ttc", 1), ("msjh.ttc", 0), ("msyh.ttc", 0)]);
    let cjk_bold = add("cjk-bold", &[("msjhbd.ttc", 1), ("msjhbd.ttc", 0)]);

    let defaults = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut regular: Vec<String> = Vec::new();
    if latin {
        regular.push("latin".into());
    }
    if cjk {
        regular.push("cjk".into());
    }
    regular.extend(defaults.iter().cloned());
    let mut heavy: Vec<String> = Vec::new();
    if latin_bold {
        heavy.push("latin-bold".into());
    }
    if cjk_bold {
        heavy.push("cjk-bold".into());
    }
    heavy.extend(regular.iter().cloned());
    fonts.families.insert(FontFamily::Proportional, regular);
    fonts.families.insert(FontFamily::Name("bold".into()), heavy);
    if cjk {
        fonts.families.entry(FontFamily::Monospace).or_default().push("cjk".into());
    }
    ctx.set_fonts(fonts);
}

// ---------- 圖示 ----------

const ICONS: &[(&str, &str)] = &[
    ("folder", "M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"),
    ("file", "M6 2c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6H6zm7 7V3.5L18.5 9H13z"),
    ("image", "M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"),
    ("movie", "M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z"),
    ("audio", "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"),
    ("doc", "M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"),
    ("sheet", "M20 2H4c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zM8 20H4v-4h4v4zm0-6H4v-4h4v4zm0-6H4V4h4v4zm6 12h-4v-4h4v4zm0-6h-4v-4h4v4zm0-6h-4V4h4v4zm6 12h-4v-4h4v4zm0-6h-4v-4h4v4zm0-6h-4V4h4v4z"),
    ("slide", "M10 8v8l5-4-5-4zm9-5H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14z"),
    ("archive", "M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM12 17.5L6.5 12H10v-2h4v2h3.5L12 17.5zM5.12 5l.81-1h12l.94 1H5.12z"),
    ("home", "M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"),
    ("add", "M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"),
    ("upload", "M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z"),
    ("newFolder", "M20 6h-8l-2-2H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2zm-1 8h-3v3h-2v-3h-3v-2h3V9h2v3h3v2z"),
    ("folderUpload", "M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10zM8 13.01l1.41 1.41L11 12.84V17h2v-4.16l1.59 1.59L16 13.01 12.01 9 8 13.01z"),
    ("link", "M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"),
    ("copy", "M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"),
    ("open", "M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"),
    ("more", "M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"),
    ("power", "M13 3h-2v10h2V3zm4.83 2.17l-1.42 1.42C17.99 7.86 19 9.81 19 12c0 3.87-3.13 7-7 7s-7-3.13-7-7c0-2.19 1.01-4.14 2.58-5.42L6.17 5.17C4.23 6.82 3 9.26 3 12c0 4.97 4.03 9 9 9s9-4.03 9-9c0-2.74-1.23-5.18-3.17-6.83z"),
    ("play", "M8 5v14l11-7z"),
    ("lan", "M13 22h8v-7h-3v-4h-5V9h3V2H8v7h3v2H6v4H3v7h8v-7H8v-2h8v2h-3z"),
    ("tune", "M3 17v2h6v-2H3zM3 5v2h10V5H3zm10 16v-2h8v-2h-8v-2h-2v6h2zM7 9v2H3v2h4v2h2V9H7zm14 4v-2H11v2h10zm-6-4h2V7h4V5h-4V3h-2v6z"),
    ("history", "M13 3c-4.97 0-9 4.03-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42C8.27 19.99 10.51 21 13 21c4.97 0 9-4.03 9-9s-4.03-9-9-9zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z"),
    ("people", "M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"),
    ("up", "M4 12l1.41 1.41L11 7.83V20h2V7.83l5.58 5.59L20 12l-8-8-8 8z"),
    ("down", "M20 12l-1.41-1.41L13 16.17V4h-2v12.17l-5.58-5.59L4 12l8 8 8-8z"),
    ("cloudUp", "M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"),
    ("close", "M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"),
    ("delete", "M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"),
    ("check", "M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"),
    ("chevronRight", "M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"),
    ("expandMore", "M16.59 8.59L12 13.17 7.41 8.59 6 10l6 6 6-6z"),
    ("info", "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"),
    ("error", "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"),
    ("terminal", "M20 4H4c-1.11 0-2 .9-2 2v12c0 1.1.89 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.89-2-2-2zm0 14H4V8h16v10zm-2-1h-6v-2h6v2zM7.5 17l-1.41-1.41L8.67 13l-2.59-2.59L7.5 9l4 4-4 4z"),
    ("web", "M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm-5 14H4v-4h11v4zm0-5H4V9h11v4zm5 5h-4V9h4v9z"),
    ("exit", "M10.09 15.59L11.5 17l5-5-5-5-1.41 1.41L12.67 11H3v2h9.67l-2.58 2.59zM19 3H5c-1.11 0-2 .9-2 2v4h2V5h14v14H5v-4H3v4c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2z"),
    ("qr", "M3 11h8V3H3v8zm2-6h4v4H5V5zM3 21h8v-8H3v8zm2-6h4v4H5v-4zM13 3v8h8V3h-8zm6 6h-4V5h4v4zM13 13h2v2h-2zM15 15h2v2h-2zM13 17h2v2h-2zM17 17h2v2h-2zM19 19h2v2h-2zM15 19h2v2h-2zM17 13h2v2h-2zM19 15h2v2h-2z"),
    ("block", "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zM4 12c0-4.42 3.58-8 8-8 1.85 0 3.55.63 4.9 1.69L5.69 16.9C4.63 15.55 4 13.85 4 12zm8 8c-1.85 0-3.55-.63-4.9-1.69L18.31 7.1C19.37 8.45 20 10.15 20 12c0 4.42-3.58 8-8 8z"),
    ("person", "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"),
    ("personAdd", "M15 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm-9-2V7H4v3H1v2h3v3h2v-3h3v-2H6zm9 4c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"),
    ("edit", "M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34a.9959.9959 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"),
    ("search", "M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"),
    ("save", "M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z"),
    ("dashboard", "M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z"),
    ("settings", "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.488.488 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"),
    ("lock", "M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"),
];

fn svg_bytes(name: &str) -> &'static [u8] {
    static MAP: OnceLock<HashMap<&'static str, &'static [u8]>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        ICONS
            .iter()
            .map(|(n, d)| {
                let svg = format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="48" height="48"><path fill="#ffffff" d="{d}"/></svg>"##);
                (*n, &*Box::leak(svg.into_bytes().into_boxed_slice()))
            })
            .collect()
    });
    map.get(name).copied().unwrap_or_else(|| map["file"])
}

pub fn icon(name: &'static str, color: Color32) -> egui::Image<'static> {
    egui::Image::from_bytes(format!("bytes://icon/{name}.svg"), svg_bytes(name)).tint(color)
}

pub fn paint_icon(ui: &Ui, name: &'static str, rect: Rect, color: Color32) {
    icon(name, color).paint_at(ui, rect);
}

/// 依副檔名給 (圖示, 顏色)，配色跟分享網頁一致
pub fn file_icon(name: &str, is_folder: bool) -> (&'static str, Color32) {
    if is_folder {
        return ("folder", TEXT2);
    }
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    let red = Color32::from_rgb(0xd9, 0x30, 0x25);
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" | "heic" | "avif" | "tif" | "tiff" | "ico" => ("image", red),
        "mp4" | "mkv" | "webm" | "mov" | "avi" | "wmv" | "m4v" | "flv" | "ts" | "mpg" | "mpeg" => ("movie", red),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => ("audio", Color32::from_rgb(0xa1, 0x42, 0xf4)),
        "pdf" => ("doc", Color32::from_rgb(0xea, 0x43, 0x35)),
        "doc" | "docx" | "odt" | "rtf" => ("doc", Color32::from_rgb(0x42, 0x85, 0xf4)),
        "xls" | "xlsx" | "ods" | "csv" => ("sheet", Color32::from_rgb(0x18, 0x80, 0x38)),
        "ppt" | "pptx" | "odp" | "key" => ("slide", Color32::from_rgb(0xf9, 0xab, 0x00)),
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => ("archive", Color32::from_rgb(0x5f, 0x63, 0x68)),
        _ => ("file", Color32::from_rgb(0x5f, 0x63, 0x68)),
    }
}

// ---------- 自繪元件 ----------

/// 單行文字，太長就截斷加「…」
pub fn elide(ui: &Ui, text: &str, font: FontId, color: Color32, max_width: f32) -> Arc<egui::Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap.max_width = max_width.max(10.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    ui.fonts(|f| f.layout_job(job))
}

pub fn text_width(ui: &Ui, text: &str, font: FontId) -> f32 {
    ui.fonts(|f| f.layout_no_wrap(text.to_owned(), font, TEXT).size().x)
}

/// 圓形圖示按鈕（32×32）
pub fn icon_button(ui: &mut Ui, name: &'static str, tip: &str) -> Response {
    icon_button_sized(ui, name, tip, 32.0, TEXT2)
}

pub fn icon_button_sized(ui: &mut Ui, name: &'static str, tip: &str, size: f32, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if resp.hovered() {
        let fill = if resp.is_pointer_button_down_on() { PRESS } else { HOVER };
        ui.painter().circle_filled(rect.center(), size / 2.0, fill);
    }
    let s = (size * 0.6).round();
    paint_icon(ui, name, Rect::from_center_size(rect.center(), Vec2::splat(s)), color);
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Pill {
    Filled,
    Tonal,
    Outlined,
    Text,
}

/// 膠囊按鈕（高 36）
pub fn pill(ui: &mut Ui, kind: Pill, icon_name: Option<&'static str>, text: &str, enabled: bool) -> Response {
    let font = bold(14.0);
    let tw = text_width(ui, text, font.clone());
    let pad = if icon_name.is_some() { 16.0 } else { 20.0 };
    let w = pad + icon_name.map_or(0.0, |_| 26.0) + tw + 20.0;
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 36.0), sense);
    let (fill, fg, stroke) = match kind {
        Pill::Filled => (PRIMARY, Color32::WHITE, Stroke::NONE),
        Pill::Tonal => (SELECTED, ON_SELECTED, Stroke::NONE),
        Pill::Outlined => (Color32::TRANSPARENT, PRIMARY, Stroke::new(1.0_f32, OUTLINE)),
        Pill::Text => (Color32::TRANSPARENT, PRIMARY, Stroke::NONE),
    };
    let (fill, fg) = if enabled { (fill, fg) } else { (if fill == Color32::TRANSPARENT { fill } else { Color32::from_rgb(0xe3, 0xe3, 0xe3) }, TEXT3) };
    let p = ui.painter();
    let cr = CornerRadius::same(18);
    p.rect_filled(rect, cr, fill);
    if stroke != Stroke::NONE {
        p.rect_stroke(rect, cr, stroke, StrokeKind::Inside);
    }
    if enabled && resp.hovered() {
        p.rect_filled(rect, cr, if resp.is_pointer_button_down_on() { PRESS } else { HOVER });
    }
    let mut x = rect.left() + pad;
    if let Some(name) = icon_name {
        paint_icon(ui, name, Rect::from_min_size(egui::pos2(x, rect.center().y - 9.0), Vec2::splat(18.0)), fg);
        x += 26.0;
    }
    p.text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, text, font, fg);
    if enabled {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// 小圓角標籤，例如「● 運行中」
pub fn chip(ui: &mut Ui, text: &str, fg: Color32, bg: Color32, dot: bool) -> Response {
    let font = bold(13.0);
    let tw = text_width(ui, text, font.clone());
    let w = tw + 24.0 + if dot { 14.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 28.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(14), bg);
    let mut x = rect.left() + 12.0;
    if dot {
        p.circle_filled(egui::pos2(x + 4.0, rect.center().y), 4.0, fg);
        x += 14.0;
    }
    p.text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, text, font, fg);
    resp
}

/// 側邊列／選單的一列：圖示 + 文字，滿版寬度、圓角底色
pub struct Row<'a> {
    pub icon: &'static str,
    pub icon_color: Color32,
    pub text: &'a str,
    pub selected: bool,
    pub indent: f32,
    pub height: f32,
    pub italic: bool,
    pub trailing: Option<(&'a str, Color32, Color32)>,
}

impl<'a> Row<'a> {
    pub fn new(icon: &'static str, icon_color: Color32, text: &'a str) -> Self {
        Self { icon, icon_color, text, selected: false, indent: 0.0, height: 32.0, italic: false, trailing: None }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, self.height), Sense::click_and_drag());
        let p = ui.painter();
        let cr = CornerRadius::same((self.height / 2.0) as u8);
        if self.selected {
            p.rect_filled(rect, cr, SELECTED);
        } else if resp.hovered() {
            p.rect_filled(rect, cr, HOVER);
        }
        let fg = if self.selected { ON_SELECTED } else { TEXT2 };
        let x = rect.left() + 12.0 + self.indent;
        paint_icon(ui, self.icon, Rect::from_min_size(egui::pos2(x, rect.center().y - 10.0), Vec2::splat(20.0)), if self.selected { ON_SELECTED } else { self.icon_color });
        let text_x = x + 32.0;
        let mut right = rect.right() - 12.0;
        if let Some((t, tfg, tbg)) = self.trailing {
            let font = regular(11.5);
            let tw = text_width(ui, t, font.clone());
            let r = Rect::from_min_max(egui::pos2(right - tw - 14.0, rect.center().y - 10.0), egui::pos2(right, rect.center().y + 10.0));
            p.rect_filled(r, CornerRadius::same(10), tbg);
            p.text(r.center(), Align2::CENTER_CENTER, t, font, tfg);
            right = r.left() - 8.0;
        }
        let font = if self.selected { bold(14.0) } else { regular(14.0) };
        let mut job = LayoutJob::simple_singleline(self.text.to_owned(), font, fg);
        if self.italic {
            if let Some(s) = job.sections.first_mut() {
                s.format.italics = true;
            }
        }
        job.wrap.max_width = (right - text_x).max(10.0);
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
        let galley = ui.fonts(|f| f.layout_job(job));
        p.galley(egui::pos2(text_x, rect.center().y - galley.size().y / 2.0), galley, fg);
        resp
    }
}

/// 選單項目
pub fn menu_item(ui: &mut Ui, icon_name: &'static str, text: &str, enabled: bool) -> Response {
    menu_item_colored(ui, icon_name, text, enabled, TEXT2, TEXT)
}

pub fn menu_item_colored(ui: &mut Ui, icon_name: &'static str, text: &str, enabled: bool, icon_color: Color32, text_color: Color32) -> Response {
    let w = ui.available_width().max(220.0);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 36.0), sense);
    let p = ui.painter();
    if enabled && resp.hovered() {
        p.rect_filled(rect, CornerRadius::same(6), HOVER);
    }
    let (ic, tc) = if enabled { (icon_color, text_color) } else { (OUTLINE, TEXT3) };
    paint_icon(ui, icon_name, Rect::from_min_size(egui::pos2(rect.left() + 12.0, rect.center().y - 10.0), Vec2::splat(20.0)), ic);
    p.text(egui::pos2(rect.left() + 46.0, rect.center().y), Align2::LEFT_CENTER, text, regular(14.0), tc);
    resp
}

/// 白色圓角卡片
pub fn card() -> egui::Frame {
    egui::Frame::new().fill(SURFACE).corner_radius(CornerRadius::same(16)).inner_margin(Margin::symmetric(20, 16))
}

/// 卡片標題列：圖示 + 粗體標題 + 右側內容
pub fn card_header(ui: &mut Ui, icon_name: &'static str, title: &str, right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add(icon(icon_name, TEXT2).fit_to_exact_size(Vec2::splat(20.0)));
        ui.label(egui::RichText::new(title).font(bold(16.0)).color(TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
    });
}

/// 大大的「＋ 新增」按鈕（白底、陰影、圓角 16）
pub fn fab(ui: &mut Ui, text: &str, enabled: bool) -> Response {
    let font = bold(14.5);
    let w = 16.0 + 24.0 + 12.0 + text_width(ui, text, font.clone()) + 24.0;
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 56.0), sense);
    let p = ui.painter();
    let cr = CornerRadius::same(16);
    let lifted = enabled && resp.hovered();
    let shadow = Shadow { offset: [0, if lifted { 3 } else { 1 }], blur: if lifted { 10 } else { 4 }, spread: 0, color: Color32::from_black_alpha(if lifted { 50 } else { 40 }) };
    p.add(shadow.as_shape(rect, cr));
    p.rect_filled(rect, cr, if lifted { CARD } else { SURFACE });
    let fg = if enabled { TEXT } else { TEXT3 };
    paint_icon(ui, "add", Rect::from_min_size(egui::pos2(rect.left() + 16.0, rect.center().y - 12.0), Vec2::splat(24.0)), if enabled { TEXT } else { OUTLINE });
    p.text(egui::pos2(rect.left() + 52.0, rect.center().y), Align2::LEFT_CENTER, text, font, fg);
    if enabled {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// 在 anchor 下方顯示的下拉選單；回傳 true 表示該關掉了
pub fn dropdown(ctx: &egui::Context, id: &str, anchor: Rect, min_width: f32, add: impl FnOnce(&mut Ui) -> bool) -> bool {
    let area = egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor.left_bottom() + egui::vec2(0.0, 4.0))
        .constrain(true)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).inner_margin(Margin::same(6)).show(ui, |ui| {
                ui.set_min_width(min_width);
                ui.set_max_width(min_width);
                ui.spacing_mut().item_spacing.y = 0.0;
                add(ui)
            })
            .inner
        });
    let clicked_outside = ctx.input(|i| i.pointer.any_pressed())
        && !area.response.contains_pointer()
        && !ctx.input(|i| i.pointer.interact_pos().is_some_and(|p| anchor.contains(p)));
    area.inner || clicked_outside || ctx.input(|i| i.key_pressed(egui::Key::Escape))
}

/// 分頁標籤（選中時淡藍底）
pub fn tab(ui: &mut Ui, icon_name: &'static str, text: &str, selected: bool) -> Response {
    let font = bold(14.0);
    let w = 14.0 + 20.0 + 10.0 + text_width(ui, text, font.clone()) + 18.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 36.0), Sense::click());
    let p = ui.painter();
    let cr = CornerRadius::same(18);
    if selected {
        p.rect_filled(rect, cr, SELECTED);
    } else if resp.hovered() {
        p.rect_filled(rect, cr, HOVER);
    }
    let fg = if selected { ON_SELECTED } else { TEXT2 };
    paint_icon(ui, icon_name, Rect::from_min_size(egui::pos2(rect.left() + 14.0, rect.center().y - 10.0), Vec2::splat(20.0)), fg);
    p.text(egui::pos2(rect.left() + 44.0, rect.center().y), Align2::LEFT_CENTER, text, font, fg);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// 畫 QR Code（白底、黑點、四周留白）
pub fn qr(ui: &mut Ui, text: &str, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(12), Color32::WHITE);
    let Ok(code) = qrcode::QrCode::new(text.as_bytes()) else {
        p.text(rect.center(), Align2::CENTER_CENTER, "網址太長", regular(14.0), TEXT2);
        return;
    };
    let n = code.width();
    let colors = code.to_colors();
    let quiet = 2.0;
    let cell = (size / (n as f32 + quiet * 2.0)).floor();
    let origin = rect.center() - Vec2::splat(cell * n as f32 / 2.0);
    for y in 0..n {
        for x in 0..n {
            if colors[y * n + x] == qrcode::Color::Dark {
                let min = origin + egui::vec2(x as f32 * cell, y as f32 * cell);
                p.rect_filled(Rect::from_min_size(min, Vec2::splat(cell)), CornerRadius::ZERO, TEXT);
            }
        }
    }
}

/// 送出／接收速度的迷你折線圖
pub fn sparkline(ui: &mut Ui, data: &std::collections::VecDeque<(f32, f32)>, size: Vec2) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(6), CARD);
    if data.len() < 2 {
        return resp;
    }
    let max = data.iter().map(|(o, i)| o.max(*i)).fold(1.0_f32, f32::max);
    let n = 120.0;
    let pts = |pick: fn(&(f32, f32)) -> f32, color: Color32| {
        let start = n - data.len() as f32;
        let points: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(k, d)| {
                let x = rect.left() + 3.0 + (start + k as f32) / (n - 1.0) * (rect.width() - 6.0);
                let y = rect.bottom() - 3.0 - pick(d) / max * (rect.height() - 6.0);
                egui::pos2(x, y)
            })
            .collect();
        egui::Shape::line(points, Stroke::new(1.5_f32, color))
    };
    p.add(pts(|d| d.1, PRIMARY));
    p.add(pts(|d| d.0, Color32::from_rgb(0x1e, 0x8e, 0x3e)));
    resp
}

/// 圓形頭像（取名字第一個字）
pub fn avatar(ui: &mut Ui, name: &str, size: f32) {
    const COLORS: [Color32; 6] = [
        Color32::from_rgb(0x1a, 0x73, 0xe8),
        Color32::from_rgb(0xd9, 0x30, 0x25),
        Color32::from_rgb(0x18, 0x80, 0x38),
        Color32::from_rgb(0xe3, 0x74, 0x00),
        Color32::from_rgb(0x84, 0x30, 0xce),
        Color32::from_rgb(0x00, 0x7b, 0x83),
    ];
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let idx = name.bytes().fold(0usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize)) % COLORS.len();
    ui.painter().circle_filled(rect.center(), size / 2.0, COLORS[idx]);
    let initial: String = name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, initial, bold(size * 0.45), Color32::WHITE);
}

/// 區段標題
pub fn section(ui: &mut Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(title).font(bold(15.0)).color(TEXT));
    ui.add_space(8.0);
}

/// 程式 Logo（assets\logo.svg）
pub fn logo(ui: &mut Ui, size: f32) -> Response {
    ui.add(egui::Image::from_bytes("bytes://logo.svg", crate::icon::LOGO_SVG).fit_to_exact_size(Vec2::splat(size)))
}
