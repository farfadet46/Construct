//! Outils de la vue du niveau : peinture des cartes de tuiles et édition des trajectoires.

use crate::theme::{self, C};
use crate::{App, Sel};
use eframe::egui;
use pf_core::{json, Json, JsonExt};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Select,
    Paint,
    Path,
}

/// instance sélectionnée + son objet
fn sel_inst(app: &App) -> Option<(String, Json, Json)> {
    if let Sel::Inst(id) = &app.sel {
        let i = app.find_inst(id)?.clone();
        let o = app.obj(i.s("obj"))?.clone();
        return Some((id.clone(), i, o));
    }
    None
}
fn has_path(o: &Json) -> bool {
    o.arr("behaviors").iter().any(|b| b.s("type") == "path")
}

pub fn toolbar(app: &mut App, ui: &mut egui::Ui) {
    let s = sel_inst(app);
    let is_tm = s.as_ref().map(|x| x.2.s("kind") == "tilemap").unwrap_or(false);
    let is_path = s.as_ref().map(|x| has_path(&x.2)).unwrap_or(false);
    if !is_tm && app.view.tool == Tool::Paint || !is_path && app.view.tool == Tool::Path {
        app.view.tool = Tool::Select;
    }
    if ui.add(theme::tab(app.view.tool == Tool::Select, "⬉ Sélection")).clicked() {
        app.view.tool = Tool::Select;
    }
    if ui.add_enabled(is_tm, theme::tab(app.view.tool == Tool::Paint, "🖌 Peindre les tuiles")).on_disabled_hover_text("Sélectionnez une carte de tuiles").clicked() {
        app.view.tool = Tool::Paint;
    }
    if ui.add_enabled(is_path, theme::tab(app.view.tool == Tool::Path, "〰 Trajectoire")).on_disabled_hover_text("Sélectionnez une instance qui a le comportement Trajectoire").clicked() {
        app.view.tool = Tool::Path;
    }
}

fn cell_size(o: &Json) -> (f64, f64) {
    let sc = o.f_or("scale", 1.0);
    (o.f_or("tw", 16.0) * sc, o.f_or("th", 16.0) * sc)
}
fn tm_dims(i: &Json, o: &Json) -> (i64, i64) {
    let c = { let v = i.f("cols", 0.0); if v == 0.0 { o.f_or("defCols", 8.0) } else { v } } as i64;
    let r = { let v = i.f("rows", 0.0); if v == 0.0 { o.f_or("defRows", 4.0) } else { v } } as i64;
    (c, r)
}

/// applique la peinture sur une case (et le remplissage)
fn paint(app: &mut App, id: &str, wx: f64, wy: f64, erase: bool) {
    let (_, i, o) = match sel_inst(app) {
        Some(x) => x,
        None => return,
    };
    let (cw, ch) = cell_size(&o);
    let (cols, rows) = tm_dims(&i, &o);
    let c = ((wx - i.f("x", 0.0)) / cw).floor() as i64;
    let r = ((wy - i.f("y", 0.0)) / ch).floor() as i64;
    if c < 0 || r < 0 || c >= cols || r >= rows {
        return;
    }
    let mut data: Vec<i64> = (0..(cols * rows) as usize).map(|k| i.arr("data").get(k).and_then(|v| v.as_f64()).map(|v| v as i64).unwrap_or(-1)).collect();
    let v = if erase { -1 } else { app.view.tile };
    let k0 = (r * cols + c) as usize;
    if app.view.fill && !erase {
        let from = data[k0];
        if from == v {
            return;
        }
        let mut st = vec![(c, r)];
        while let Some((x, y)) = st.pop() {
            if x < 0 || y < 0 || x >= cols || y >= rows {
                continue;
            }
            let k = (y * cols + x) as usize;
            if data[k] != from {
                continue;
            }
            data[k] = v;
            st.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
        }
    } else {
        if data[k0] == v {
            return;
        }
        data[k0] = v;
    }
    if let Some(ii) = app.inst_mut(id) {
        ii["cols"] = json!(cols);
        ii["rows"] = json!(rows);
        ii["data"] = json!(data);
    }
    app.changed();
}

pub fn interact(app: &mut App, resp: &egui::Response, ctx: &egui::Context) -> bool {
    if app.view.tool == Tool::Select {
        return false;
    }
    let (id, i, _o) = match sel_inst(app) {
        Some(x) => x,
        None => return false,
    };
    let (mid, space) = ctx.input(|x| (x.pointer.middle_down(), x.key_down(egui::Key::Space)));
    if mid || space {
        return false; // défilement normal
    }
    let pos = resp.interact_pointer_pos().or(resp.hover_pos());
    let (prim, sec, pressed) = ctx.input(|x| (x.pointer.primary_down(), x.pointer.secondary_down(), x.pointer.any_pressed()));
    let p = match pos {
        Some(p) => p,
        None => return true,
    };
    let (wx, wy) = app.view.to_world(p);
    match app.view.tool {
        Tool::Paint => {
            if resp.hovered() || resp.dragged() {
                if (prim || sec) && (resp.is_pointer_button_down_on() || resp.dragged()) {
                    if !app.view.painting {
                        app.snapshot();
                        app.view.painting = true;
                    }
                    paint(app, &id, wx, wy, sec);
                } else if !prim && !sec {
                    app.view.painting = false;
                }
            }
            let _ = pressed;
        }
        Tool::Path => {
            let (ix, iy) = (i.f("x", 0.0), i.f("y", 0.0));
            let pts: Vec<[f64; 2]> = i.arr("path").iter().map(|q| [q[0].as_f64().unwrap_or(0.0), q[1].as_f64().unwrap_or(0.0)]).collect();
            let near = |app: &App| pts.iter().position(|q| (app.view.to_screen(ix + q[0], iy + q[1]) - p).length() < 9.0);
            let snap = |app: &App, v: f64| if app.view.snap && !ctx.input(|x| x.modifiers.alt) { (v / app.view.grid_size).round() * app.view.grid_size } else { v.round() };
            if resp.drag_started() {
                if let Some(k) = near(app) {
                    app.snapshot();
                    app.view.tdrag = Some(k);
                }
            }
            if resp.dragged() {
                if let Some(k) = app.view.tdrag {
                    let (nx, ny) = (snap(app, wx - ix), snap(app, wy - iy));
                    if let Some(ii) = app.inst_mut(&id) {
                        ii["path"][k] = json!([nx, ny]);
                    }
                    app.changed();
                }
            }
            if resp.drag_stopped() {
                app.view.tdrag = None;
            }
            if resp.clicked() && near(app).is_none() {
                app.snapshot();
                let (nx, ny) = (snap(app, wx - ix), snap(app, wy - iy));
                if let Some(ii) = app.inst_mut(&id) {
                    if !ii.get("path").map(|x| x.is_array()).unwrap_or(false) {
                        ii["path"] = json!([]);
                    }
                    ii["path"].as_array_mut().unwrap().push(json!([nx, ny]));
                }
                app.changed();
            }
            if resp.secondary_clicked() {
                if let Some(k) = near(app) {
                    app.snapshot();
                    if let Some(ii) = app.inst_mut(&id) {
                        ii["path"].as_array_mut().unwrap().remove(k);
                    }
                    app.changed();
                }
            }
        }
        Tool::Select => {}
    }
    true
}

pub fn overlay(app: &App, painter: &egui::Painter) {
    let (_, i, o) = match sel_inst(app) {
        Some(x) => x,
        None => return,
    };
    let (ix, iy) = (i.f("x", 0.0), i.f("y", 0.0));
    if o.s("kind") == "tilemap" && app.view.tool == Tool::Paint {
        let (cw, ch) = cell_size(&o);
        let (cols, rows) = tm_dims(&i, &o);
        let st = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40));
        for c in 0..=cols {
            painter.line_segment([app.view.to_screen(ix + c as f64 * cw, iy), app.view.to_screen(ix + c as f64 * cw, iy + rows as f64 * ch)], st);
        }
        for r in 0..=rows {
            painter.line_segment([app.view.to_screen(ix, iy + r as f64 * ch), app.view.to_screen(ix + cols as f64 * cw, iy + r as f64 * ch)], st);
        }
        if let Some(m) = painter.ctx().input(|x| x.pointer.hover_pos()) {
            if app.view.rect.contains(m) {
                let (wx, wy) = app.view.to_world(m);
                let c = ((wx - ix) / cw).floor();
                let r = ((wy - iy) / ch).floor();
                if c >= 0.0 && r >= 0.0 && (c as i64) < cols && (r as i64) < rows {
                    let a = app.view.to_screen(ix + c * cw, iy + r * ch);
                    let b = app.view.to_screen(ix + (c + 1.0) * cw, iy + (r + 1.0) * ch);
                    painter.rect_stroke(egui::Rect::from_min_max(a, b), 0.0, egui::Stroke::new(2.0, C::ACCENT), egui::StrokeKind::Inside);
                }
            }
        }
    }
    if has_path(&o) {
        let mut pts = vec![app.view.to_screen(ix, iy)];
        for q in i.arr("path") {
            pts.push(app.view.to_screen(ix + q[0].as_f64().unwrap_or(0.0), iy + q[1].as_f64().unwrap_or(0.0)));
        }
        let active = app.view.tool == Tool::Path;
        let col = if active { C::ACCENT } else { egui::Color32::from_rgba_unmultiplied(255, 138, 61, 110) };
        let mode = o.arr("behaviors").iter().find(|b| b.s("type") == "path").and_then(|b| b.get("props")).map(|p| p.s("mode").to_string()).unwrap_or_default();
        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], egui::Stroke::new(2.0, col));
        }
        if mode == "loop" && pts.len() > 2 {
            painter.add(egui::Shape::dashed_line(&[*pts.last().unwrap(), pts[0]], egui::Stroke::new(1.5, col), 6.0, 4.0));
        }
        for (k, p) in pts.iter().enumerate() {
            painter.circle(*p, if k == 0 { 4.0 } else { 6.0 }, if k == 0 { C::BG0 } else { col }, egui::Stroke::new(1.5, egui::Color32::WHITE));
            if k > 0 && active {
                painter.text(*p + egui::vec2(8.0, -8.0), egui::Align2::LEFT_BOTTOM, k.to_string(), egui::FontId::proportional(11.0), egui::Color32::WHITE);
            }
        }
        if active {
            painter.text(app.view.rect.left_bottom() + egui::vec2(10.0, -10.0), egui::Align2::LEFT_BOTTOM, "Clic : ajouter un point · glisser : déplacer · clic droit : supprimer", egui::FontId::proportional(13.0), C::ACCENT_H);
        }
    }
    if o.s("kind") == "tilemap" && app.view.tool == Tool::Paint {
        painter.text(app.view.rect.left_bottom() + egui::vec2(10.0, -10.0), egui::Align2::LEFT_BOTTOM, "Clic : peindre · clic droit : effacer · choisissez la tuile dans le panneau de droite", egui::FontId::proportional(13.0), C::ACCENT_H);
    }
}

/// palette de tuiles (panneau Propriétés d'une carte de tuiles)
pub fn palette(app: &mut App, ui: &mut egui::Ui, inst_id: &str) {
    let (_, i, o) = match sel_inst(app) {
        Some(x) => x,
        None => return,
    };
    let _ = inst_id;
    theme::section(ui, "Carte de tuiles");
    let (cols, rows) = tm_dims(&i, &o);
    let mut nc = cols;
    let mut nr = rows;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Colonnes").color(C::MUTED));
        ui.add(egui::DragValue::new(&mut nc).range(1..=1000));
        ui.label(egui::RichText::new("Lignes").color(C::MUTED));
        ui.add(egui::DragValue::new(&mut nr).range(1..=1000));
    });
    if nc != cols || nr != rows {
        app.snapshot();
        let old: Vec<i64> = (0..(cols * rows) as usize).map(|k| i.arr("data").get(k).and_then(|v| v.as_f64()).map(|v| v as i64).unwrap_or(-1)).collect();
        let mut data = vec![-1i64; (nc * nr) as usize];
        for r in 0..rows.min(nr) {
            for c in 0..cols.min(nc) {
                data[(r * nc + c) as usize] = old[(r * cols + c) as usize];
            }
        }
        let (cw, ch) = cell_size(&o);
        if let Some(ii) = app.inst_mut(inst_id) {
            ii["cols"] = json!(nc);
            ii["rows"] = json!(nr);
            ii["data"] = json!(data);
            ii["w"] = json!(nc as f64 * cw);
            ii["h"] = json!(nr as f64 * ch);
        }
        app.changed();
    }
    if app.view.tool != Tool::Paint {
        if ui.add(theme::btn_accent("🖌 Peindre les tuiles")).clicked() {
            app.view.tool = Tool::Paint;
        }
    }
    ui.checkbox(&mut app.view.fill, "Remplir une zone (pot de peinture)");
    // image du jeu de tuiles
    let img = o.get("image").and_then(|v| v.as_str()).map(|s| app.imgs.intern(s));
    let pm = img.and_then(|k| app.imgs.list.get(k).cloned().flatten());
    let pm = match pm {
        Some(p) => p,
        None => return,
    };
    let (tw, th) = (o.f_or("tw", 16.0), o.f_or("th", 16.0));
    let tc = ((pm.width() as f64 / tw).floor() as i64).max(1);
    let tr = ((pm.height() as f64 / th).floor() as i64).max(1);
    let key = format!("tileset:{}", o.s("id"));
    let tex = ui.ctx().memory_mut(|m| m.data.get_temp::<(egui::TextureHandle, usize)>(egui::Id::new(&key))).filter(|t| Some(t.1) == img);
    let tex = match tex {
        Some(t) => t.0,
        None => {
            let rgba = pf_runtime::img::to_rgba(&pm);
            let t = ui.ctx().load_texture(&key, egui::ColorImage::from_rgba_unmultiplied([pm.width() as usize, pm.height() as usize], &rgba), egui::TextureOptions::NEAREST);
            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new(&key), (t.clone(), img.unwrap())));
            t
        }
    };
    let cell = 34.0f32;
    ui.label(egui::RichText::new("Tuile à peindre (clic droit dans le niveau : effacer)").size(11.0).color(C::DIM));
    egui::Grid::new("tiles").spacing([3.0, 3.0]).show(ui, |ui| {
        for r in 0..tr {
            for c in 0..tc {
                let v = r * tc + c;
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(cell, cell), egui::Sense::click());
                let uv = egui::Rect::from_min_max(egui::pos2((c as f64 * tw / pm.width() as f64) as f32, (r as f64 * th / pm.height() as f64) as f32), egui::pos2(((c + 1) as f64 * tw / pm.width() as f64) as f32, ((r + 1) as f64 * th / pm.height() as f64) as f32));
                ui.painter().rect_filled(rect, 3.0, C::BG0);
                ui.painter().image(tex.id(), rect.shrink(2.0), uv, egui::Color32::WHITE);
                if v == app.view.tile {
                    ui.painter().rect_stroke(rect, 3.0, egui::Stroke::new(2.0, C::ACCENT), egui::StrokeKind::Inside);
                }
                if resp.clicked() {
                    app.view.tile = v;
                    app.view.tool = Tool::Paint;
                }
            }
            ui.end_row();
        }
    });
    // type de collision de la tuile choisie
    let types = pf_core::catalog::catalog()["TILETYPES"].as_array().cloned().unwrap_or_default();
    let cur = o.get("tcol").and_then(|t| t.get(app.view.tile.to_string())).and_then(|v| v.as_str()).unwrap_or("s").to_string();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Tuile n°{} :", app.view.tile)).color(C::MUTED));
        let lab = types.iter().find(|t| t[0].as_str() == Some(&cur)).and_then(|t| t[1].as_str()).unwrap_or(&cur).to_string();
        egui::ComboBox::from_id_salt("tcol").selected_text(lab).show_ui(ui, |ui| {
            for t in &types {
                let k = t[0].as_str().unwrap_or("s");
                if ui.selectable_label(k == cur, t[1].as_str().unwrap_or("")).clicked() {
                    let oid = o.s("id").to_string();
                    let tile = app.view.tile.to_string();
                    app.snapshot();
                    if let Some(ob) = app.obj_mut(&oid) {
                        if !ob.get("tcol").map(|x| x.is_object()).unwrap_or(false) {
                            ob["tcol"] = json!({});
                        }
                        ob["tcol"][tile] = json!(k);
                    }
                    app.changed();
                }
            }
        });
    });
}

/// réglages de la trajectoire (panneau Propriétés)
pub fn path_panel(app: &mut App, ui: &mut egui::Ui, inst_id: &str) {
    let (_, i, _o) = match sel_inst(app) {
        Some(x) => x,
        None => return,
    };
    theme::section(ui, "Trajectoire");
    let n = i.arr("path").len();
    ui.label(egui::RichText::new(format!("{} point(s) en plus du départ", n)).color(C::MUTED));
    ui.horizontal(|ui| {
        if ui.add(if app.view.tool == Tool::Path { theme::btn("Terminer") } else { theme::btn_accent("〰 Modifier la trajectoire") }).clicked() {
            app.view.tool = if app.view.tool == Tool::Path { Tool::Select } else { Tool::Path };
        }
        if ui.add_enabled(n > 0, theme::btn_small("Effacer")).clicked() {
            app.snapshot();
            if let Some(ii) = app.inst_mut(inst_id) {
                ii["path"] = json!([]);
            }
            app.changed();
        }
    });
}
