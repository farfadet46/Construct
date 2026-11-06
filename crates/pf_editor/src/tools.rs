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
    if ui.add(theme::tab(app.view.tool == Tool::Select, crate::i18n::tr("⬉ Sélection"))).clicked() {
        app.view.tool = Tool::Select;
    }
    if ui.add_enabled(is_tm, theme::tab(app.view.tool == Tool::Paint, crate::i18n::tr("✎ Peindre les tuiles"))).on_disabled_hover_text(crate::i18n::tr("Sélectionnez une carte de tuiles")).clicked() {
        app.view.tool = Tool::Paint;
    }
    if ui.add_enabled(is_path, theme::tab(app.view.tool == Tool::Path, crate::i18n::tr("〰 Trajectoire"))).on_disabled_hover_text(crate::i18n::tr("Sélectionnez une instance qui a le comportement Trajectoire")).clicked() {
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

fn tm_data(i: &Json, cols: i64, rows: i64) -> Vec<i64> {
    (0..(cols * rows) as usize).map(|k| i.arr("data").get(k).and_then(|v| v.as_f64()).map(|v| v as i64).unwrap_or(-1)).collect()
}

/// nombre de colonnes du jeu de tuiles (image)
fn tileset_cols(app: &mut App, o: &Json) -> i64 {
    let img = o.get("image").and_then(|v| v.as_str()).map(|s| app.imgs.intern(s));
    let pm = img.and_then(|k| app.imgs.list.get(k).cloned().flatten());
    pm.map(|p| ((p.width() as f64 / o.f_or("tw", 16.0)).floor() as i64).max(1)).unwrap_or(1)
}

/// case (colonne, ligne) sous un point du monde
fn cell_at(i: &Json, o: &Json, wx: f64, wy: f64) -> (i64, i64) {
    let (cw, ch) = cell_size(o);
    (((wx - i.f("x", 0.0)) / cw).floor() as i64, ((wy - i.f("y", 0.0)) / ch).floor() as i64)
}

/// tuile du tampon pour la case (x, y), le motif se répète à partir de (ax, ay)
fn pattern(app: &App, tc: i64, x: i64, y: i64, ax: i64, ay: i64) -> i64 {
    let (sw, sh) = app.view.stamp;
    app.view.tile + (y - ay).rem_euclid(sh.max(1)) * tc + (x - ax).rem_euclid(sw.max(1))
}

/// recalcule les tuiles « terrain 3×3 » autour d'une case
fn terrain_fix(data: &mut [i64], cols: i64, rows: i64, tc: i64, base: i64, c: i64, r: i64) {
    let set: Vec<i64> = (0..3).flat_map(|y| (0..3).map(move |x| base + y * tc + x)).collect();
    let is = |d: &[i64], x: i64, y: i64| x >= 0 && y >= 0 && x < cols && y < rows && set.contains(&d[(y * cols + x) as usize]);
    for y in r - 1..=r + 1 {
        for x in c - 1..=c + 1 {
            if !is(data, x, y) {
                continue;
            }
            let (n, s, w, e) = (is(data, x, y - 1), is(data, x, y + 1), is(data, x - 1, y), is(data, x + 1, y));
            let row = if !n { 0 } else if !s { 2 } else { 1 };
            let col = if !w { 0 } else if !e { 2 } else { 1 };
            data[(y * cols + x) as usize] = base + row * tc + col;
        }
    }
}

fn write_data(app: &mut App, id: &str, cols: i64, rows: i64, data: Vec<i64>) {
    if let Some(ii) = app.inst_mut(id) {
        ii["cols"] = json!(cols);
        ii["rows"] = json!(rows);
        ii["data"] = json!(data);
    }
    app.changed();
}

/// applique la peinture sur une case (pinceau, remplissage, terrain)
fn paint(app: &mut App, id: &str, cell: (i64, i64), erase: bool) {
    let (_, i, o) = match sel_inst(app) {
        Some(x) => x,
        None => return,
    };
    let (cols, rows) = tm_dims(&i, &o);
    let (c, r) = cell;
    if c < 0 || r < 0 || c >= cols || r >= rows {
        return;
    }
    let tc = tileset_cols(app, &o);
    let mut data = tm_data(&i, cols, rows);
    let old = data.clone();
    let k0 = (r * cols + c) as usize;
    match app.view.tmode {
        3 => {
            let base = app.view.tile;
            data[k0] = if erase { -1 } else { base };
            terrain_fix(&mut data, cols, rows, tc, base, c, r);
        }
        2 if !erase => {
            let from = data[k0];
            let mut seen = vec![false; data.len()];
            let mut st = vec![(c, r)];
            while let Some((x, y)) = st.pop() {
                if x < 0 || y < 0 || x >= cols || y >= rows {
                    continue;
                }
                let k = (y * cols + x) as usize;
                if seen[k] || old[k] != from {
                    continue;
                }
                seen[k] = true;
                data[k] = pattern(app, tc, x, y, c, r);
                st.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
            }
        }
        _ => {
            if erase {
                data[k0] = -1;
            } else {
                let (sw, sh) = app.view.stamp;
                for dy in 0..sh.max(1) {
                    for dx in 0..sw.max(1) {
                        let (x, y) = (c + dx, r + dy);
                        if x < cols && y < rows {
                            data[(y * cols + x) as usize] = app.view.tile + dy * tc + dx;
                        }
                    }
                }
            }
        }
    }
    if data != old {
        write_data(app, id, cols, rows, data);
    }
}

/// remplit (ou efface) un rectangle de cases avec le motif du tampon
fn paint_rect(app: &mut App, id: &str, a: (i64, i64), b: (i64, i64), erase: bool) {
    let (_, i, o) = match sel_inst(app) {
        Some(x) => x,
        None => return,
    };
    let (cols, rows) = tm_dims(&i, &o);
    let tc = tileset_cols(app, &o);
    let mut data = tm_data(&i, cols, rows);
    let (x0, x1) = (a.0.min(b.0).max(0), a.0.max(b.0).min(cols - 1));
    let (y0, y1) = (a.1.min(b.1).max(0), a.1.max(b.1).min(rows - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            data[(y * cols + x) as usize] = if erase { -1 } else { pattern(app, tc, x, y, x0, y0) };
        }
    }
    write_data(app, id, cols, rows, data);
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
            let alt = ctx.input(|x| x.modifiers.alt);
            let (_, i2, o2) = sel_inst(app).unwrap();
            let cell = cell_at(&i2, &o2, wx, wy);
            if alt {
                // pipette : reprend la tuile sous la souris
                if resp.clicked() {
                    let (cols, rows) = tm_dims(&i2, &o2);
                    if cell.0 >= 0 && cell.1 >= 0 && cell.0 < cols && cell.1 < rows {
                        let v = tm_data(&i2, cols, rows)[(cell.1 * cols + cell.0) as usize];
                        if v >= 0 {
                            app.view.tile = v;
                            app.view.stamp = (1, 1);
                            if app.view.tmode == 3 {
                                app.view.tmode = 0;
                            }
                        }
                    }
                }
                return true;
            }
            if app.view.tmode == 1 {
                let down_on = (prim || sec) && (resp.is_pointer_button_down_on() || resp.dragged());
                if down_on && app.view.trect.is_none() {
                    app.view.trect = Some((cell, sec));
                }
                if !prim && !sec {
                    if let Some((a, er)) = app.view.trect.take() {
                        app.snapshot();
                        paint_rect(app, &id, a, cell, er);
                    }
                }
                return true;
            }
            let click = resp.clicked() || resp.secondary_clicked();
            let erase = sec || resp.secondary_clicked();
            if click || ((resp.hovered() || resp.dragged()) && (prim || sec) && (resp.is_pointer_button_down_on() || resp.dragged())) {
                if !app.view.painting {
                    app.snapshot();
                    app.view.painting = true;
                    app.view.tlast = None;
                }
                // relie la case précédente à la case actuelle (pas de trous si la souris va vite)
                let from = if app.view.tmode == 2 { cell } else { app.view.tlast.unwrap_or(cell) };
                let (mut x, mut y) = from;
                let (dx, dy) = ((cell.0 - x).abs(), -(cell.1 - y).abs());
                let (sx, sy) = (if x < cell.0 { 1 } else { -1 }, if y < cell.1 { 1 } else { -1 });
                let mut err = dx + dy;
                loop {
                    paint(app, &id, (x, y), erase);
                    if (x, y) == cell {
                        break;
                    }
                    let e2 = 2 * err;
                    if e2 >= dy {
                        err += dy;
                        x += sx;
                    }
                    if e2 <= dx {
                        err += dx;
                        y += sy;
                    }
                }
                app.view.tlast = Some(cell);
            }
            if !prim && !sec {
                app.view.painting = false;
                app.view.tlast = None;
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
        let st = egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40));
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
                    // empreinte : tampon, ou rectangle en cours
                    let (mut c0, mut r0, mut c1, mut r1) = (c, r, c + 1.0, r + 1.0);
                    if let Some(((ac, ar), _)) = app.view.trect {
                        c0 = c.min(ac as f64);
                        r0 = r.min(ar as f64);
                        c1 = c.max(ac as f64) + 1.0;
                        r1 = r.max(ar as f64) + 1.0;
                    } else if app.view.tmode == 0 {
                        c1 = (c + app.view.stamp.0 as f64).min(cols as f64);
                        r1 = (r + app.view.stamp.1 as f64).min(rows as f64);
                    }
                    let a = app.view.to_screen(ix + c0 * cw, iy + r0 * ch);
                    let b = app.view.to_screen(ix + c1 * cw, iy + r1 * ch);
                    painter.rect_filled(egui::Rect::from_min_max(a, b), 0.0, egui::Color32::from_rgba_unmultiplied(255, 138, 61, 30));
                    painter.rect_stroke(egui::Rect::from_min_max(a, b), 0.0, egui::Stroke::new(2.0_f32, C::ACCENT()), egui::StrokeKind::Inside);
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
        let col = if active { C::ACCENT() } else { egui::Color32::from_rgba_unmultiplied(255, 138, 61, 110) };
        let mode = o.arr("behaviors").iter().find(|b| b.s("type") == "path").and_then(|b| b.get("props")).map(|p| p.s("mode").to_string()).unwrap_or_default();
        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], egui::Stroke::new(2.0_f32, col));
        }
        if mode == "loop" && pts.len() > 2 {
            painter.add(egui::Shape::dashed_line(&[*pts.last().unwrap(), pts[0]], egui::Stroke::new(1.5_f32, col), 6.0, 4.0));
        }
        for (k, p) in pts.iter().enumerate() {
            painter.circle(*p, if k == 0 { 4.0 } else { 6.0 }, if k == 0 { C::BG0() } else { col }, egui::Stroke::new(1.5_f32, egui::Color32::WHITE));
            if k > 0 && active {
                painter.text(*p + egui::vec2(8.0, -8.0), egui::Align2::LEFT_BOTTOM, k.to_string(), egui::FontId::proportional(11.0), egui::Color32::WHITE);
            }
        }
        if active {
            painter.text(app.view.rect.left_bottom() + egui::vec2(10.0, -10.0), egui::Align2::LEFT_BOTTOM, crate::i18n::tr("Clic : ajouter un point · glisser : déplacer · clic droit : supprimer"), egui::FontId::proportional(13.0), C::ACCENT_H());
        }
    }
    if o.s("kind") == "tilemap" && app.view.tool == Tool::Paint {
        painter.text(app.view.rect.left_bottom() + egui::vec2(10.0, -10.0), egui::Align2::LEFT_BOTTOM, match app.view.tmode {
            1 => crate::i18n::tr("Glisser : remplir un rectangle · clic droit glissé : effacer · Alt+clic : pipette"),
            2 => crate::i18n::tr("Clic : remplir la zone · clic droit : effacer · Alt+clic : pipette"),
            3 => crate::i18n::tr("Terrain : peignez, les bords et les coins se placent tout seuls · clic droit : effacer"),
            _ => crate::i18n::tr("Clic : peindre · clic droit : effacer · Alt+clic : reprendre une tuile · glisser dans la palette : tampon"),
        }, egui::FontId::proportional(13.0), C::ACCENT_H());
    }
}

/// palette de tuiles (panneau Propriétés d'une carte de tuiles)
pub fn palette(app: &mut App, ui: &mut egui::Ui, inst_id: &str) {
    let (_, i, o) = match sel_inst(app) {
        Some(x) => x,
        None => return,
    };
    let _ = inst_id;
    theme::section(ui, crate::i18n::tr("Carte de tuiles"));
    let (cols, rows) = tm_dims(&i, &o);
    let mut nc = cols;
    let mut nr = rows;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(crate::i18n::tr("Colonnes")).color(C::MUTED()));
        ui.add(egui::DragValue::new(&mut nc).range(1..=1000));
        ui.label(egui::RichText::new(crate::i18n::tr("Lignes")).color(C::MUTED()));
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
        if ui.add(theme::btn_accent(crate::i18n::tr("✎ Peindre les tuiles"))).clicked() {
            app.view.tool = Tool::Paint;
        }
    }
    ui.horizontal_wrapped(|ui| {
        for (m, n, tip) in [(0u8, crate::i18n::tr("✎ Pinceau"), crate::i18n::tr("Peint la tuile (ou le tampon) choisie")), (1, crate::i18n::tr("▭ Rectangle"), crate::i18n::tr("Glisser pour remplir un rectangle")), (2, crate::i18n::tr("◍ Remplir"), crate::i18n::tr("Remplit une zone de tuiles identiques")), (3, crate::i18n::tr("◢ Terrain"), crate::i18n::tr("Terrain automatique : choisissez le coin haut-gauche d’un bloc 3×3 de tuiles (bords et coins)"))] {
            if ui.add(theme::tab(app.view.tmode == m, n)).on_hover_text(tip).clicked() {
                app.view.tmode = m;
                app.view.tool = Tool::Paint;
                if m == 3 {
                    app.view.stamp = (3, 3);
                }
            }
        }
    });
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
    ui.label(egui::RichText::new(crate::i18n::tr("Clic : une tuile · glisser : plusieurs tuiles (tampon)")).size(11.0).color(C::DIM()));
    egui::Grid::new("tiles").spacing([3.0, 3.0]).show(ui, |ui| {
        for r in 0..tr {
            for c in 0..tc {
                let v = r * tc + c;
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(cell, cell), egui::Sense::click_and_drag());
                let uv = egui::Rect::from_min_max(egui::pos2((c as f64 * tw / pm.width() as f64) as f32, (r as f64 * th / pm.height() as f64) as f32), egui::pos2(((c + 1) as f64 * tw / pm.width() as f64) as f32, ((r + 1) as f64 * th / pm.height() as f64) as f32));
                ui.painter().rect_filled(rect, 3.0, C::BG0());
                ui.painter().image(tex.id(), rect.shrink(2.0), uv, egui::Color32::WHITE);
                let (sc, sr) = (app.view.tile % tc, app.view.tile / tc);
                let (sw, sh) = app.view.stamp;
                if c >= sc && c < sc + sw && r >= sr && r < sr + sh {
                    ui.painter().rect_filled(rect, 3.0, egui::Color32::from_rgba_unmultiplied(255, 138, 61, 40));
                    ui.painter().rect_stroke(rect, 3.0, egui::Stroke::new(2.0_f32, C::ACCENT()), egui::StrokeKind::Inside);
                }
                if resp.drag_started() && app.view.tmode != 3 {
                    app.view.pal_anchor = Some((c, r));
                }
                if let Some((ac, ar)) = app.view.pal_anchor {
                    let over = ui.ctx().input(|i| i.pointer.interact_pos()).map(|p| rect.contains(p)).unwrap_or(false);
                    if over {
                        app.view.tile = ar.min(r) * tc + ac.min(c);
                        app.view.stamp = ((ac - c).abs() + 1, (ar - r).abs() + 1);
                        app.view.tool = Tool::Paint;
                    }
                }
                if resp.clicked() {
                    app.view.tile = v;
                    app.view.stamp = if app.view.tmode == 3 { (3, 3) } else { (1, 1) };
                    app.view.tool = Tool::Paint;
                }
            }
            ui.end_row();
        }
    });
    if !ui.ctx().input(|i| i.pointer.any_down()) {
        app.view.pal_anchor = None;
    }
    if app.view.tmode == 3 {
        ui.label(egui::RichText::new(crate::i18n::tr("Terrain : le bloc 3×3 doit contenir, dans l’ordre, coin haut-gauche, bord haut, coin haut-droit, puis bords gauche / centre / droit, puis la rangée du bas.")).size(11.0).color(C::DIM()));
    } else if app.view.stamp != (1, 1) {
        ui.label(egui::RichText::new(crate::i18n::trf("Tampon de {0} × {1} tuiles", &[format!("{}", app.view.stamp.0), format!("{}", app.view.stamp.1)])).size(11.0).color(C::ACCENT()));
    }
    // type de collision de la tuile choisie
    let types = pf_core::catalog::ui_catalog()["TILETYPES"].as_array().cloned().unwrap_or_default();
    let cur = o.get("tcol").and_then(|t| t.get(app.view.tile.to_string())).and_then(|v| v.as_str()).unwrap_or("s").to_string();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(crate::i18n::trf("Tuile n°{0} :", &[format!("{}", app.view.tile)])).color(C::MUTED()));
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
    theme::section(ui, crate::i18n::tr("Trajectoire"));
    let n = i.arr("path").len();
    ui.label(egui::RichText::new(crate::i18n::trf("{0} point(s) en plus du départ", &[format!("{}", n)])).color(C::MUTED()));
    ui.horizontal(|ui| {
        if ui.add(if app.view.tool == Tool::Path { theme::btn(crate::i18n::tr("Terminer")) } else { theme::btn_accent(crate::i18n::tr("〰 Modifier la trajectoire")) }).clicked() {
            app.view.tool = if app.view.tool == Tool::Path { Tool::Select } else { Tool::Path };
        }
        if ui.add_enabled(n > 0, theme::btn_small(crate::i18n::tr("Effacer"))).clicked() {
            app.snapshot();
            if let Some(ii) = app.inst_mut(inst_id) {
                ii["path"] = json!([]);
            }
            app.changed();
        }
    });
}
