//! Vue du niveau : aperçu fidèle (même moteur de dessin que le jeu), sélection, déplacement,
//! redimensionnement, glisser-déposer depuis le panneau Objets, zoom et défilement.

use crate::theme::{self, C};
use crate::{App, Sel};
use eframe::egui;
use pf_core::{json, Json, JsonExt};
use pf_runtime::coll::draw_box;
use pf_runtime::draw::{draw_inst, DrawCtx};
use pf_runtime::gfx::{parse_color, Gfx, Rgba};
use pf_runtime::model::{BBox, Inst};
use std::collections::HashSet;

pub struct ViewState {
    pub cam: (f64, f64),
    pub zoom: f64,
    pub tex: Option<egui::TextureHandle>,
    pub dirty: bool,
    pub fit_pending: bool,
    pub key: (u64, i64, i64, i64, u32, u32, String),
    pub drag: Option<Drag>,
    pub grid: bool,
    pub snap: bool,
    pub grid_size: f64,
    pub active_layer: Option<String>,
    pub hidden: HashSet<String>,
    pub locked: HashSet<String>,
    pub rect: egui::Rect,
    pub g: Option<Gfx>,
    pub fit_frames: u32,
    pub tool: crate::tools::Tool,
    pub tile: i64,
    pub fill: bool,
    pub tdrag: Option<usize>,
    pub painting: bool,
}
impl Default for ViewState {
    fn default() -> Self {
        ViewState { cam: (0.0, 0.0), zoom: 1.0, tex: None, dirty: true, fit_pending: true, key: Default::default(), drag: None, grid: true, snap: true, grid_size: 16.0, active_layer: None, hidden: HashSet::new(), locked: HashSet::new(), rect: egui::Rect::NOTHING, g: None, fit_frames: 0, tool: crate::tools::Tool::Select, tile: 0, fill: false, tdrag: None, painting: false }
    }
}

pub enum Drag {
    Move { id: String, sx: f64, sy: f64, mx: f64, my: f64 },
    Resize { id: String },
    Pan { cx: f64, cy: f64, mx: f32, my: f32 },
}

impl ViewState {
    pub fn to_world(&self, p: egui::Pos2) -> (f64, f64) {
        (self.cam.0 + (p.x - self.rect.min.x) as f64 / self.zoom, self.cam.1 + (p.y - self.rect.min.y) as f64 / self.zoom)
    }
    pub fn to_screen(&self, x: f64, y: f64) -> egui::Pos2 {
        egui::pos2(self.rect.min.x + ((x - self.cam.0) * self.zoom) as f32, self.rect.min.y + ((y - self.cam.1) * self.zoom) as f32)
    }
    pub fn center_world(&self) -> (f64, f64) {
        self.to_world(self.rect.center())
    }
}

fn layer_of(l: &Json, i: &Json) -> String {
    let lid = i.s("layer");
    if l.arr("layers").iter().any(|y| y.s("id") == lid) {
        lid.to_string()
    } else {
        l.arr("layers").first().map(|y| y.s("id").to_string()).unwrap_or_default()
    }
}

/// boîte d'affichage d'une instance du niveau (coordonnées du monde)
pub fn inst_box(app: &App, i: &Json) -> Option<BBox> {
    let d = app.defs.get(i.s("obj"))?;
    let pi = Inst::preview(0, d, i);
    Some(draw_box(d, &pi))
}

fn hit(app: &App, x: f64, y: f64) -> Option<String> {
    let l = app.layout()?;
    for i in l.arr("instances").iter().rev() {
        let ly = layer_of(l, i);
        if app.view.hidden.contains(&ly) || app.view.locked.contains(&ly) {
            continue;
        }
        if let Some(b) = inst_box(app, i) {
            let (l0, r0) = (b.l.min(b.r), b.l.max(b.r));
            if x >= l0 && x <= r0 && y >= b.t && y <= b.b {
                return Some(i.s("id").to_string());
            }
        }
    }
    None
}

fn render(app: &mut App, pw: u32, ph: u32, ppp: f32) {
    let l = match app.layout() {
        Some(l) => l.clone(),
        None => return,
    };
    let g = app.view.g.get_or_insert_with(|| Gfx::new(pw, ph));
    g.resize(pw, ph);
    g.clear(Rgba(0x0b as f32 / 255.0, 0x0d as f32 / 255.0, 0x11 as f32 / 255.0, 1.0));
    let z = (app.view.zoom * ppp as f64) as f32;
    let (cx, cy) = app.view.cam;
    g.set_transform(z, 0.0, 0.0, z, (-cx * z as f64) as f32, (-cy * z as f64) as f32);
    let (lw, lh) = (l.f("w", 854.0) as f32, l.f("h", 480.0) as f32);
    let bg = if l.s("bg").is_empty() { "#000000" } else { l.s("bg") };
    g.fill_rect(0.0, 0.0, lw, lh, parse_color(bg));
    let pixel = app.proj.get("settings").map(|s| s.get("pixelArt") != Some(&Json::Bool(false))).unwrap_or(true);
    g.set_smooth(!pixel);
    let defs = app.defs.clone();
    let lk = |id: &str| -> Option<(f64, std::rc::Rc<tiny_skia::Pixmap>)> {
        let d = defs.get(id)?;
        if d.kind != pf_runtime::model::Kind::Sprite {
            return None;
        }
        let k = *d.anims.first()?.frames.first()?;
        use pf_runtime::draw::Imgs;
        Some((d.j.f_or("defW", 0.0), app.imgs.img(k)?))
    };
    let cxs = DrawCtx { imgs: &app.imgs, editor: true, rd_lk: &lk, mm: None, now_ms: 0.0 };
    for y in l.arr("layers") {
        let lid = y.s("id");
        if app.view.hidden.contains(lid) {
            continue;
        }
        let la = y.f("opacity", 100.0) as f32 / 100.0;
        for i in l.arr("instances") {
            if layer_of(&l, i) != lid {
                continue;
            }
            let d = match defs.get(i.s("obj")) {
                Some(d) => d,
                None => continue,
            };
            let pi = Inst::preview(0, d, i);
            g.save();
            let mut a = la * if i.get("visible") == Some(&Json::Bool(false)) { 0.35 } else { 1.0 };
            if pf_runtime::rt::jtruthy(i.get("off")) {
                a *= 0.3;
            }
            g.mul_alpha(a);
            draw_inst(g, d, &pi, &cxs);
            g.restore();
        }
    }
    if l.arr("layers").is_empty() {
        for i in l.arr("instances") {
            if let Some(d) = defs.get(i.s("obj")) {
                let pi = Inst::preview(0, d, i);
                draw_inst(g, d, &pi, &cxs);
            }
        }
    }
    // grille
    if app.view.grid && app.view.zoom * app.view.grid_size >= 6.0 {
        let gs = app.view.grid_size as f32;
        let c = Rgba(1.0, 1.0, 1.0, 0.06);
        let w1 = 1.0 / z;
        let mut x = 0.0;
        while x <= lw {
            g.fill_rect(x, 0.0, w1, lh, c);
            x += gs;
        }
        let mut y = 0.0;
        while y <= lh {
            g.fill_rect(0.0, y, lw, w1, c);
            y += gs;
        }
    }
}

pub fn ui(app: &mut App, ui: &mut egui::Ui, now: f64) {
    let _ = now;
    // ---- barre d'outils
    egui::Frame::new().fill(C::BG1).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(format!("{} %", (app.view.zoom * 100.0).round())).color(C::MUTED).monospace());
            if ui.add(theme::btn_small("−")).clicked() {
                zoom_at(app, app.view.rect.center(), 1.0 / 1.25);
            }
            if ui.add(theme::btn_small("+")).clicked() {
                zoom_at(app, app.view.rect.center(), 1.25);
            }
            if ui.add(theme::btn_small("Ajuster")).on_hover_text("Voir tout le niveau").clicked() {
                app.view.fit_pending = true;
            }
            if ui.add(theme::btn_small("100 %")).clicked() {
                let c = app.view.center_world();
                app.view.zoom = 1.0;
                app.view.cam = (c.0 - app.view.rect.width() as f64 / 2.0, c.1 - app.view.rect.height() as f64 / 2.0);
            }
            ui.separator();
            crate::tools::toolbar(app, ui);
            ui.separator();
            if ui.toggle_value(&mut app.view.grid, "Grille").changed() {
                app.view.dirty = true;
            }
            ui.toggle_value(&mut app.view.snap, "Aimanter");
            ui.add(egui::DragValue::new(&mut app.view.grid_size).range(2.0..=256.0).suffix(" px")).on_hover_text("Taille de la grille");
            ui.separator();
            // calques
            if let Some(l) = app.layout().cloned() {
                ui.label(egui::RichText::new("Calque :").color(C::MUTED));
                let layers = l.arr("layers").to_vec();
                let cur = app.view.active_layer.clone().filter(|a| layers.iter().any(|y| y.s("id") == a));
                let cur_name = cur.as_ref().and_then(|c| layers.iter().find(|y| y.s("id") == c)).map(|y| y.s("name").to_string()).unwrap_or_else(|| "(automatique)".into());
                egui::ComboBox::from_id_salt("layer").selected_text(cur_name).show_ui(ui, |ui| {
                    if ui.selectable_label(cur.is_none(), "(automatique)").clicked() {
                        app.view.active_layer = None;
                    }
                    for y in &layers {
                        if ui.selectable_label(cur.as_deref() == Some(y.s("id")), y.s("name")).clicked() {
                            app.view.active_layer = Some(y.s("id").to_string());
                        }
                    }
                });
                for y in &layers {
                    let id = y.s("id").to_string();
                    let vis = !app.view.hidden.contains(&id);
                    let lock = app.view.locked.contains(&id);
                    let t = format!("{}{} {}", if vis { "👁" } else { "◌" }, if lock { "🔒" } else { "" }, y.s("name"));
                    let r = ui.add(egui::Button::new(egui::RichText::new(t).size(12.0).color(if vis { C::TEXT2 } else { C::DIM })).fill(C::BG2)).on_hover_text("Clic : afficher / masquer · Clic droit : verrouiller");
                    if r.clicked() {
                        if vis {
                            app.view.hidden.insert(id.clone());
                        } else {
                            app.view.hidden.remove(&id);
                        }
                        app.view.dirty = true;
                    }
                    if r.secondary_clicked() {
                        if lock {
                            app.view.locked.remove(&id);
                        } else {
                            app.view.locked.insert(id.clone());
                        }
                    }
                }
            }
        });
    });

    let (resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = resp.rect;
    app.view.rect = rect;
    let ppp = ui.ctx().pixels_per_point();
    if app.view.fit_pending && rect.width() > 10.0 {
        if let Some(l) = app.layout() {
            let (lw, lh) = (l.f("w", 854.0), l.f("h", 480.0));
            let z = ((rect.width() as f64 - 40.0) / lw).min((rect.height() as f64 - 40.0) / lh).clamp(0.05, 4.0);
            app.view.zoom = z;
            app.view.cam = (lw / 2.0 - rect.width() as f64 / 2.0 / z, lh / 2.0 - rect.height() as f64 / 2.0 / z);
        }
        app.view.fit_frames += 1;
        if app.view.fit_frames > 3 {
            app.view.fit_pending = false;
            app.view.fit_frames = 0;
        }
    }

    // ---- interactions
    let mouse = resp.hover_pos();
    let scroll = ui.ctx().input(|i| i.raw_scroll_delta.y);
    if let Some(m) = mouse {
        if scroll != 0.0 {
            zoom_at(app, m, if scroll > 0.0 { 1.15 } else { 1.0 / 1.15 });
        }
    }
    let consumed = crate::tools::interact(app, &resp, ui.ctx());
    if !consumed {
    let pressed_mid = ui.ctx().input(|i| i.pointer.middle_down() || i.pointer.secondary_down() || (i.key_down(egui::Key::Space) && i.pointer.primary_down()));
    if resp.drag_started() {
        let p = resp.interact_pointer_pos().unwrap_or(rect.center());
        if pressed_mid {
            app.view.drag = Some(Drag::Pan { cx: app.view.cam.0, cy: app.view.cam.1, mx: p.x, my: p.y });
        } else {
            let (wx, wy) = app.view.to_world(p);
            // poignée de redimensionnement de la sélection ?
            let mut on_handle = false;
            if let Sel::Inst(id) = app.sel.clone() {
                if let Some(b) = app.find_inst(&id).and_then(|i| inst_box(app, i)) {
                    let hp = app.view.to_screen(b.l.max(b.r), b.b);
                    if (hp - p).length() < 9.0 {
                        on_handle = true;
                        app.snapshot();
                        app.view.drag = Some(Drag::Resize { id: id.clone() });
                    }
                }
            }
            if !on_handle {
                match hit(app, wx, wy) {
                    Some(id) => {
                        let i = app.find_inst(&id).cloned().unwrap();
                        app.snapshot();
                        app.sel = Sel::Inst(id.clone());
                        app.view.drag = Some(Drag::Move { id, sx: i.f("x", 0.0), sy: i.f("y", 0.0), mx: wx, my: wy });
                    }
                    None => {
                        app.view.drag = Some(Drag::Pan { cx: app.view.cam.0, cy: app.view.cam.1, mx: p.x, my: p.y });
                    }
                }
            }
        }
    }
    if resp.dragged() {
        if let Some(p) = resp.interact_pointer_pos() {
            match &app.view.drag {
                Some(Drag::Pan { cx, cy, mx, my }) => {
                    app.view.cam = (cx - (p.x - mx) as f64 / app.view.zoom, cy - (p.y - my) as f64 / app.view.zoom);
                }
                Some(Drag::Move { id, sx, sy, mx, my }) => {
                    let (wx, wy) = app.view.to_world(p);
                    let (mut nx, mut ny) = (sx + wx - mx, sy + wy - my);
                    if app.view.snap && !ui.ctx().input(|i| i.modifiers.alt) {
                        let gs = app.view.grid_size;
                        nx = (nx / gs).round() * gs;
                        ny = (ny / gs).round() * gs;
                    }
                    let id = id.clone();
                    if let Some(i) = app.inst_mut(&id) {
                        i["x"] = json!(nx);
                        i["y"] = json!(ny);
                    }
                    app.changed();
                }
                Some(Drag::Resize { id }) => {
                    let id = id.clone();
                    let (wx, wy) = app.view.to_world(p);
                    if let Some(i) = app.find_inst(&id).cloned() {
                        if let Some(d) = app.defs.get(i.s("obj")).cloned() {
                            let (x, y) = (i.f("x", 0.0), i.f("y", 0.0));
                            let (mut w, mut h);
                            if d.kind == pf_runtime::model::Kind::Sprite {
                                let fx = (d.w - d.ox) / d.w;
                                let fy = (d.h - d.oy) / d.h;
                                w = if fx > 0.0 { (wx - x) / fx } else { i.f("w", 32.0) };
                                h = if fy > 0.0 { (wy - y) / fy } else { i.f("h", 32.0) };
                            } else {
                                w = wx - x;
                                h = wy - y;
                            }
                            if app.view.snap && !ui.ctx().input(|i| i.modifiers.alt) {
                                let gs = app.view.grid_size;
                                w = (w / gs).round() * gs;
                                h = (h / gs).round() * gs;
                            }
                            if ui.ctx().input(|i| i.modifiers.shift) && i.f("h", 1.0) != 0.0 {
                                h = w * i.f("h", 1.0) / i.f("w", 1.0).max(0.001);
                            }
                            if let Some(ii) = app.inst_mut(&id) {
                                ii["w"] = json!(w.max(1.0).round());
                                ii["h"] = json!(h.max(1.0).round());
                            }
                            app.changed();
                        }
                    }
                }
                None => {}
            }
        }
    }
    if resp.drag_stopped() {
        app.view.drag = None;
    }
    if resp.clicked() {
        if let Some(p) = resp.interact_pointer_pos() {
            let (wx, wy) = app.view.to_world(p);
            app.sel = match hit(app, wx, wy) {
                Some(id) => Sel::Inst(id),
                None => Sel::Layout(app.cur_layout.clone()),
            };
        }
    }
    if resp.double_clicked() {
        if let Sel::Inst(id) = &app.sel {
            if let Some(o) = app.find_inst(id).map(|i| i.s("obj").to_string()) {
                app.sel = Sel::Obj(o);
            }
        }
    }
    // glisser-déposer d'un objet depuis le panneau
    if let Some(oid) = resp.dnd_release_payload::<String>() {
        if let Some(p) = ui.ctx().input(|i| i.pointer.interact_pos()) {
            let (mut wx, mut wy) = app.view.to_world(p);
            if app.view.snap {
                let gs = app.view.grid_size;
                wx = (wx / gs).round() * gs;
                wy = (wy / gs).round() * gs;
            }
            let (ox, oy) = app.obj(&oid).map(|o| if o.s("kind") == "sprite" { (0.0, 0.0) } else { (o.f_or("defW", 32.0) / 2.0, o.f_or("defH", 32.0) / 2.0) }).unwrap_or((0.0, 0.0));
            crate::tree::add_instance(app, &oid, wx - ox, wy - oy);
        }
    }
    // flèches : déplacer la sélection
    if resp.hovered() || resp.has_focus() {
        if let Sel::Inst(id) = app.sel.clone() {
            let (dx, dy, big) = ui.ctx().input(|i| {
                let mut dx = 0.0;
                let mut dy = 0.0;
                if i.key_pressed(egui::Key::ArrowLeft) {
                    dx -= 1.0;
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    dx += 1.0;
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    dy -= 1.0;
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    dy += 1.0;
                }
                (dx, dy, i.modifiers.shift)
            });
            if (dx != 0.0 || dy != 0.0) && !ui.ctx().wants_keyboard_input() {
                let k = if big { app.view.grid_size } else { 1.0 };
                app.snapshot();
                if let Some(i) = app.inst_mut(&id) {
                    i["x"] = json!(i.f("x", 0.0) + dx * k);
                    i["y"] = json!(i.f("y", 0.0) + dy * k);
                }
                app.changed();
            }
        }
    }

    }
    // ---- rendu (seulement si quelque chose a changé)
    let (pw, ph) = ((rect.width() * ppp).round().max(1.0) as u32, (rect.height() * ppp).round().max(1.0) as u32);
    let key = (app.ver, (app.view.cam.0 * 100.0) as i64, (app.view.cam.1 * 100.0) as i64, (app.view.zoom * 1000.0) as i64, pw, ph, format!("{}{}{:?}", app.cur_layout, app.view.grid, app.view.hidden));
    if key != app.view.key || app.view.dirty || app.view.tex.is_none() {
        app.view.key = key;
        app.view.dirty = false;
        render(app, pw, ph, ppp);
        if let Some(g) = &app.view.g {
            let rgba = pf_runtime::img::to_rgba(&g.pm);
            let im = egui::ColorImage::from_rgba_unmultiplied([pw as usize, ph as usize], &rgba);
            match &mut app.view.tex {
                Some(t) => t.set(im, egui::TextureOptions::NEAREST),
                None => app.view.tex = Some(ui.ctx().load_texture("layout", im, egui::TextureOptions::NEAREST)),
            }
        }
    }
    if let Some(t) = &app.view.tex {
        painter.image(t.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    }
    // cadre de l'écran de jeu (vue de départ) et limites du niveau
    if let Some(l) = app.layout() {
        let (gw, gh) = app.proj.get("settings").map(|s| (s.f("width", 854.0), s.f("height", 480.0))).unwrap_or((854.0, 480.0));
        let r = egui::Rect::from_min_max(app.view.to_screen(0.0, 0.0), app.view.to_screen(gw, gh));
        painter.rect_stroke(r, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 216, 77, 120)), egui::StrokeKind::Middle);
        let lr = egui::Rect::from_min_max(app.view.to_screen(0.0, 0.0), app.view.to_screen(l.f("w", 0.0), l.f("h", 0.0)));
        painter.rect_stroke(lr, 0.0, egui::Stroke::new(1.0, C::LINE3), egui::StrokeKind::Outside);
    }
    // sélection
    if let Sel::Inst(id) = &app.sel {
        if let Some(i) = app.find_inst(id) {
            if let Some(b) = inst_box(app, i) {
                let r = egui::Rect::from_min_max(app.view.to_screen(b.l.min(b.r), b.t), app.view.to_screen(b.l.max(b.r), b.b));
                painter.rect_stroke(r, 0.0, egui::Stroke::new(1.5, C::ACCENT), egui::StrokeKind::Outside);
                let hp = app.view.to_screen(b.l.max(b.r), b.b);
                painter.rect_filled(egui::Rect::from_center_size(hp, egui::vec2(9.0, 9.0)), 2.0, C::ACCENT);
                let name = app.obj_name(i.s("obj"));
                painter.text(r.left_top() - egui::vec2(0.0, 3.0), egui::Align2::LEFT_BOTTOM, format!("{}  ({:.0}, {:.0})  {:.0}×{:.0}", name, i.f("x", 0.0), i.f("y", 0.0), i.f("w", 0.0), i.f("h", 0.0)), egui::FontId::proportional(12.0), C::ACCENT_H);
            }
        }
    }
    crate::tools::overlay(app, &painter);
    // survol
    if let (Some(m), None) = (mouse, &app.view.drag) {
        let (wx, wy) = app.view.to_world(m);
        if let Some(id) = hit(app, wx, wy) {
            if app.sel != Sel::Inst(id.clone()) {
                if let Some(b) = app.find_inst(&id).and_then(|i| inst_box(app, i)) {
                    let r = egui::Rect::from_min_max(app.view.to_screen(b.l.min(b.r), b.t), app.view.to_screen(b.l.max(b.r), b.b));
                    painter.rect_stroke(r, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 90)), egui::StrokeKind::Outside);
                }
            }
        }
        let info = app.layout().map(|l| format!("{} × {} px · {} instance(s)   ", l.f("w", 0.0), l.f("h", 0.0), l.arr("instances").len())).unwrap_or_default();
        painter.text(rect.right_bottom() - egui::vec2(8.0, 6.0), egui::Align2::RIGHT_BOTTOM, format!("{}{:.0}, {:.0}", info, wx, wy), egui::FontId::monospace(11.0), C::DIM);
    }
    if app.layout().map(|l| l.arr("instances").is_empty()).unwrap_or(false) {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "Glissez un objet depuis le panneau de gauche pour le placer dans le niveau", egui::FontId::proportional(14.0), C::MUTED);
    }
}

fn zoom_at(app: &mut App, p: egui::Pos2, f: f64) {
    let (wx, wy) = app.view.to_world(p);
    app.view.zoom = (app.view.zoom * f).clamp(0.05, 16.0);
    app.view.cam = (wx - (p.x - app.view.rect.min.x) as f64 / app.view.zoom, wy - (p.y - app.view.rect.min.y) as f64 / app.view.zoom);
}
