//! Thème sombre « pro » (mêmes couleurs que la version HTML).
use eframe::egui::{self, Color32, CornerRadius, Stroke};

/// palette (sombre ou claire)
#[allow(non_snake_case)]
pub mod C {
    use eframe::egui::Color32;
    use std::sync::atomic::{AtomicBool, Ordering};
    pub static LIGHT: AtomicBool = AtomicBool::new(false);
    fn l() -> bool {
        LIGHT.load(Ordering::Relaxed)
    }
    macro_rules! col {
        ($n:ident, $d:expr, $l:expr) => {
            pub fn $n() -> Color32 {
                if l() { $l } else { $d }
            }
        };
    }
    const fn rgb(v: u32) -> Color32 {
        Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }
    col!(BG0, rgb(0x0d0f13), rgb(0xeef0f4));
    col!(BG1, rgb(0x13161c), rgb(0xffffff));
    col!(BG2, rgb(0x191d25), rgb(0xf5f6f8));
    col!(BG3, rgb(0x21262f), rgb(0xeaecf0));
    col!(BG4, rgb(0x2a303b), rgb(0xdfe3e9));
    col!(LINE, rgb(0x232832), rgb(0xe2e5eb));
    col!(LINE2, rgb(0x2f3542), rgb(0xd2d7df));
    col!(LINE3, rgb(0x3a4150), rgb(0xb9c0cb));
    col!(TEXT, rgb(0xe8eaf0), rgb(0x161a21));
    col!(TEXT2, rgb(0xd3d7df), rgb(0x262b34));
    col!(MUTED, rgb(0x9aa2b2), rgb(0x5d6472));
    col!(DIM, rgb(0x626a7b), rgb(0x8a909d));
    col!(ACCENT, rgb(0xff8a3d), rgb(0xe8701f));
    col!(ACCENT_H, rgb(0xff9e5e), rgb(0xc85a10));
    col!(ACCENT_SOFT, Color32::from_rgba_premultiplied(33, 18, 8, 33), Color32::from_rgba_premultiplied(36, 20, 6, 36));
    col!(DANGER, rgb(0xf0616d), rgb(0xc22f3b));
    col!(OK, rgb(0x3ddc97), rgb(0x0f9a60));
    col!(COND, rgb(0x5aa9ff), rgb(0x1f5fc4));
    col!(ACT, rgb(0x3ddc97), rgb(0x0f7a50));
    col!(SYS, rgb(0xb48cff), rgb(0x6d43c9));
    col!(CODE, rgb(0xffd2b0), rgb(0xa8480a));
    col!(PURPLE, rgb(0xc8adff), rgb(0x6d43c9));
    col!(GREEN_T, rgb(0x7ef0b8), rgb(0x0f7a50));
    col!(BLUE_T, rgb(0x8cc2ff), rgb(0x1f5fc4));
    col!(CANVAS, rgb(0x0b0d11), rgb(0x0b0d11));
}

pub fn set_light(ctx: &egui::Context, light: bool) {
    C::LIGHT.store(light, std::sync::atomic::Ordering::Relaxed);
    apply_style(ctx);
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
    apply_style(ctx);
}

fn apply_style(ctx: &egui::Context) {
    let light = C::LIGHT.load(std::sync::atomic::Ordering::Relaxed);
    let mut st = (*ctx.style()).clone();
    st.spacing.item_spacing = egui::vec2(7.0, 5.0);
    st.spacing.button_padding = egui::vec2(10.0, 4.0);
    st.spacing.interact_size.y = 24.0;
    let v = &mut st.visuals;
    *v = if light { egui::Visuals::light() } else { egui::Visuals::dark() };
    v.panel_fill = C::BG1();
    v.window_fill = C::BG2();
    v.extreme_bg_color = C::BG0();
    v.faint_bg_color = C::BG2();
    v.window_stroke = Stroke::new(1.0_f32, C::LINE2());
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(8);
    v.selection.bg_fill = if light { Color32::from_rgb(0xff, 0xd9, 0xbd) } else { Color32::from_rgb(0x5a, 0x34, 0x1a) };
    v.selection.stroke = Stroke::new(1.0_f32, C::ACCENT());
    v.hyperlink_color = C::ACCENT();
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(6);
    }
    v.widgets.noninteractive.bg_fill = C::BG2();
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, C::LINE());
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, C::TEXT2());
    v.widgets.inactive.bg_fill = C::BG3();
    v.widgets.inactive.weak_bg_fill = C::BG3();
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, C::LINE2());
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, C::TEXT2());
    v.widgets.hovered.bg_fill = C::BG4();
    v.widgets.hovered.weak_bg_fill = C::BG4();
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, C::LINE3());
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, C::TEXT());
    v.widgets.active.bg_fill = C::BG4();
    v.widgets.active.weak_bg_fill = C::BG4();
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, C::ACCENT());
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, C::TEXT());
    v.widgets.open.bg_fill = C::BG3();
    ctx.set_style(st);
}

pub fn btn(t: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(t.to_string()).color(C::TEXT2())).fill(C::BG3()).stroke(Stroke::new(1.0_f32, C::LINE2())).corner_radius(7)
}
pub fn btn_accent(t: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(t.to_string()).color(Color32::from_rgb(0x1a, 0x0f, 0x07)).strong()).fill(Color32::from_rgb(0xff, 0x8a, 0x3d)).corner_radius(7)
}
pub fn btn_small(t: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(t.to_string()).size(12.0).color(C::TEXT2())).fill(C::BG3()).stroke(Stroke::new(1.0_f32, C::LINE2())).corner_radius(6)
}
pub fn tab(on: bool, t: &str) -> egui::Button<'static> {
    let b = egui::Button::new(egui::RichText::new(t.to_string()).color(if on { C::TEXT() } else { C::MUTED() })).corner_radius(7);
    if on {
        b.fill(C::BG3()).stroke(Stroke::new(1.0_f32, C::ACCENT()))
    } else {
        b.fill(Color32::TRANSPARENT).stroke(Stroke::NONE)
    }
}
pub fn section(ui: &mut egui::Ui, t: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(t.to_uppercase()).size(11.0).color(C::DIM()).strong());
    ui.add_space(2.0);
}
pub fn color32(s: &str) -> Color32 {
    let c = pf_runtime::gfx::parse_color(s);
    Color32::from_rgba_unmultiplied((c.0 * 255.0) as u8, (c.1 * 255.0) as u8, (c.2 * 255.0) as u8, (c.3 * 255.0) as u8)
}

/// taille par défaut et taille maximale d'une fenêtre, adaptées à l'écran
pub fn fit(ctx: &egui::Context, w: f32, h: f32) -> (egui::Vec2, egui::Vec2) {
    let sr = ctx.screen_rect();
    let mx = egui::vec2((sr.width() - 24.0).max(320.0), (sr.height() - 70.0).max(240.0));
    (egui::vec2(w.min(mx.x), h.min(mx.y)), mx)
}
