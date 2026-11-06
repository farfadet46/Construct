//! Thème sombre « pro » (mêmes couleurs que la version HTML).
use eframe::egui::{self, Color32, CornerRadius, Stroke};

#[allow(non_snake_case)]
pub mod C {
    use eframe::egui::Color32;
    pub const BG0: Color32 = Color32::from_rgb(0x0d, 0x0f, 0x13);
    pub const BG1: Color32 = Color32::from_rgb(0x13, 0x16, 0x1c);
    pub const BG2: Color32 = Color32::from_rgb(0x19, 0x1d, 0x25);
    pub const BG3: Color32 = Color32::from_rgb(0x21, 0x26, 0x2f);
    pub const BG4: Color32 = Color32::from_rgb(0x2a, 0x30, 0x3b);
    pub const LINE: Color32 = Color32::from_rgb(0x23, 0x28, 0x32);
    pub const LINE2: Color32 = Color32::from_rgb(0x2f, 0x35, 0x42);
    pub const LINE3: Color32 = Color32::from_rgb(0x3a, 0x41, 0x50);
    pub const TEXT: Color32 = Color32::from_rgb(0xe8, 0xea, 0xf0);
    pub const TEXT2: Color32 = Color32::from_rgb(0xd3, 0xd7, 0xdf);
    pub const MUTED: Color32 = Color32::from_rgb(0x9a, 0xa2, 0xb2);
    pub const DIM: Color32 = Color32::from_rgb(0x62, 0x6a, 0x7b);
    pub const ACCENT: Color32 = Color32::from_rgb(0xff, 0x8a, 0x3d);
    pub const ACCENT_H: Color32 = Color32::from_rgb(0xff, 0x9e, 0x5e);
    pub const ACCENT_SOFT: Color32 = Color32::from_rgba_premultiplied(33, 18, 8, 33);
    pub const DANGER: Color32 = Color32::from_rgb(0xf0, 0x61, 0x6d);
    pub const OK: Color32 = Color32::from_rgb(0x3d, 0xdc, 0x97);
    pub const COND: Color32 = Color32::from_rgb(0x5a, 0xa9, 0xff);
    pub const ACT: Color32 = Color32::from_rgb(0x3d, 0xdc, 0x97);
    pub const SYS: Color32 = Color32::from_rgb(0xb4, 0x8c, 0xff);
    pub const CODE: Color32 = Color32::from_rgb(0xff, 0xd2, 0xb0);
    pub const PURPLE: Color32 = Color32::from_rgb(0xc8, 0xad, 0xff);
    pub const GREEN_T: Color32 = Color32::from_rgb(0x7e, 0xf0, 0xb8);
    pub const BLUE_T: Color32 = Color32::from_rgb(0x8c, 0xc2, 0xff);
    pub const CANVAS: Color32 = Color32::from_rgb(0x0b, 0x0d, 0x11);
    pub const _UNUSED: Color32 = Color32::TRANSPARENT;
}

pub fn install(ctx: &egui::Context) {
    // polices : Poppins (titres arrondis) + DejaVu Sans, comme le jeu
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("ui".into(), std::sync::Arc::new(egui::FontData::from_static(pf_runtime::gfx::font_bytes("sans", false))));
    fonts.font_data.insert("ui_b".into(), std::sync::Arc::new(egui::FontData::from_static(pf_runtime::gfx::font_bytes("round", true))));
    fonts.font_data.insert("mono".into(), std::sync::Arc::new(egui::FontData::from_static(pf_runtime::gfx::font_bytes("mono", false))));
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "ui".into());
    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "mono".into());
    fonts.families.insert(egui::FontFamily::Name("bold".into()), vec!["ui_b".into(), "ui".into()]);
    ctx.set_fonts(fonts);
    let mut st = (*ctx.style()).clone();
    st.spacing.item_spacing = egui::vec2(7.0, 5.0);
    st.spacing.button_padding = egui::vec2(10.0, 4.0);
    st.spacing.interact_size.y = 24.0;
    let v = &mut st.visuals;
    *v = egui::Visuals::dark();
    v.panel_fill = C::BG1;
    v.window_fill = C::BG2;
    v.extreme_bg_color = C::BG0;
    v.faint_bg_color = C::BG2;
    v.window_stroke = Stroke::new(1.0, C::LINE2);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(8);
    v.selection.bg_fill = Color32::from_rgb(0x5a, 0x34, 0x1a);
    v.selection.stroke = Stroke::new(1.0, C::ACCENT);
    v.hyperlink_color = C::ACCENT;
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(6);
    }
    v.widgets.noninteractive.bg_fill = C::BG2;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, C::LINE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, C::TEXT2);
    v.widgets.inactive.bg_fill = C::BG3;
    v.widgets.inactive.weak_bg_fill = C::BG3;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, C::LINE2);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, C::TEXT2);
    v.widgets.hovered.bg_fill = C::BG4;
    v.widgets.hovered.weak_bg_fill = C::BG4;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, C::LINE3);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, C::TEXT);
    v.widgets.active.bg_fill = C::BG4;
    v.widgets.active.weak_bg_fill = C::BG4;
    v.widgets.active.bg_stroke = Stroke::new(1.0, C::ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, C::TEXT);
    v.widgets.open.bg_fill = C::BG3;
    ctx.set_style(st);
}

pub fn btn(t: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(t.to_string()).color(C::TEXT2)).fill(C::BG3).stroke(Stroke::new(1.0, C::LINE2)).corner_radius(7)
}
pub fn btn_accent(t: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(t.to_string()).color(Color32::from_rgb(0x1a, 0x0f, 0x07)).strong()).fill(C::ACCENT).corner_radius(7)
}
pub fn btn_small(t: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(t.to_string()).size(12.0).color(C::TEXT2)).fill(C::BG3).stroke(Stroke::new(1.0, C::LINE2)).corner_radius(6)
}
pub fn tab(on: bool, t: &str) -> egui::Button<'static> {
    let b = egui::Button::new(egui::RichText::new(t.to_string()).color(if on { C::TEXT } else { C::MUTED })).corner_radius(7);
    if on {
        b.fill(C::BG3).stroke(Stroke::new(1.0, C::ACCENT))
    } else {
        b.fill(Color32::TRANSPARENT).stroke(Stroke::NONE)
    }
}
pub fn section(ui: &mut egui::Ui, t: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(t.to_uppercase()).size(11.0).color(C::DIM).strong());
    ui.add_space(2.0);
}
pub fn color32(s: &str) -> Color32 {
    let c = pf_runtime::gfx::parse_color(s);
    Color32::from_rgba_unmultiplied((c.0 * 255.0) as u8, (c.1 * 255.0) as u8, (c.2 * 255.0) as u8, (c.3 * 255.0) as u8)
}
