//! Sélection multiple d'instances dans le niveau : aligner, répartir, réglages communs,
//! copier / coller (y compris d'un niveau à l'autre).
use crate::i18n::{tr, trf};
use crate::theme::{self, C};
use crate::view::inst_box;
use crate::{App, Sel};
use eframe::egui;
use pf_core::{json, Json, JsonExt};

/// instances sélectionnées (la principale comprise)
pub fn sel_insts(app: &App) -> Vec<String> {
    if !app.multi.is_empty() {
        return app.multi.clone();
    }
    match &app.sel {
        Sel::Inst(id) => vec![id.clone()],
        _ => vec![],
    }
}

/// garde la liste cohérente avec la sélection principale (changée ailleurs : arbre, propriétés…)
pub fn normalize(app: &mut App) {
    match &app.sel {
        Sel::Inst(id) => {
            if !app.multi.contains(id) {
                app.multi.clear();
            }
        }
        _ => app.multi.clear(),
    }
    // instances supprimées ou d'un autre niveau
    if !app.multi.is_empty() {
        let ids: Vec<String> = app.layout().map(|l| l.arr("instances").iter().map(|i| i.s("id").to_string()).collect()).unwrap_or_default();
        app.multi.retain(|m| ids.contains(m));
        if app.multi.len() == 1 {
            app.multi.clear();
        }
    }
}

/// sélectionne une liste d'instances (la dernière devient la principale)
pub fn select(app: &mut App, ids: Vec<String>) {
    match ids.len() {
        0 => {
            app.multi.clear();
            app.sel = Sel::Layout(app.cur_layout.clone());
        }
        1 => {
            app.multi.clear();
            app.sel = Sel::Inst(ids[0].clone());
        }
        _ => {
            app.sel = Sel::Inst(ids.last().unwrap().clone());
            app.multi = ids;
        }
    }
}

/// Maj+clic : ajoute ou retire une instance de la sélection
pub fn toggle(app: &mut App, id: &str) {
    let mut cur = sel_insts(app);
    if let Some(k) = cur.iter().position(|x| x == id) {
        cur.remove(k);
    } else {
        cur.push(id.to_string());
    }
    select(app, cur);
}

/// boîtes (gauche, haut, droite, bas) des instances, en coordonnées du monde
fn boxes(app: &App, ids: &[String]) -> Vec<(String, f64, f64, f64, f64)> {
    ids.iter()
        .filter_map(|id| {
            let i = app.find_inst(id)?;
            let b = inst_box(app, i)?;
            Some((id.clone(), b.l.min(b.r), b.t, b.l.max(b.r), b.b))
        })
        .collect()
}

/// boîte englobant toute la sélection
pub fn group_box(app: &App, ids: &[String]) -> Option<(f64, f64, f64, f64)> {
    let b = boxes(app, ids);
    if b.is_empty() {
        return None;
    }
    Some(b.iter().fold((f64::MAX, f64::MAX, f64::MIN, f64::MIN), |a, x| (a.0.min(x.1), a.1.min(x.2), a.2.max(x.3), a.3.max(x.4))))
}

fn shift_inst(app: &mut App, id: &str, dx: f64, dy: f64) {
    if let Some(i) = app.inst_mut(id) {
        i["x"] = json!((i.f("x", 0.0) + dx).round());
        i["y"] = json!((i.f("y", 0.0) + dy).round());
    }
}

#[derive(Clone, Copy)]
pub enum Align {
    Left,
    CenterX,
    Right,
    Top,
    CenterY,
    Bottom,
}

pub fn align(app: &mut App, how: Align) {
    let ids = sel_insts(app);
    let b = boxes(app, &ids);
    let Some(g) = group_box(app, &ids) else { return };
    app.snapshot();
    for (id, l, t, r, bt) in b {
        let (dx, dy) = match how {
            Align::Left => (g.0 - l, 0.0),
            Align::Right => (g.2 - r, 0.0),
            Align::CenterX => ((g.0 + g.2) / 2.0 - (l + r) / 2.0, 0.0),
            Align::Top => (0.0, g.1 - t),
            Align::Bottom => (0.0, g.3 - bt),
            Align::CenterY => (0.0, (g.1 + g.3) / 2.0 - (t + bt) / 2.0),
        };
        shift_inst(app, &id, dx, dy);
    }
    app.changed();
}

/// répartit avec des écarts égaux entre les objets (les deux extrêmes ne bougent pas)
pub fn distribute(app: &mut App, horizontal: bool) {
    let ids = sel_insts(app);
    let mut b = boxes(app, &ids);
    if b.len() < 3 {
        return;
    }
    if horizontal {
        b.sort_by(|a, c| (a.1 + a.3).partial_cmp(&(c.1 + c.3)).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        b.sort_by(|a, c| (a.2 + a.4).partial_cmp(&(c.2 + c.4)).unwrap_or(std::cmp::Ordering::Equal));
    }
    let n = b.len();
    let (start, end) = if horizontal { (b[0].1, b[n - 1].3) } else { (b[0].2, b[n - 1].4) };
    let total: f64 = b.iter().map(|x| if horizontal { x.3 - x.1 } else { x.4 - x.2 }).sum();
    let gap = (end - start - total) / (n - 1) as f64;
    app.snapshot();
    let mut pos = start;
    for (id, l, t, r, bt) in b {
        let size = if horizontal { r - l } else { bt - t };
        let d = pos - if horizontal { l } else { t };
        if horizontal {
            shift_inst(app, &id, d, 0.0);
        } else {
            shift_inst(app, &id, 0.0, d);
        }
        pos += size + gap;
    }
    app.changed();
}

pub fn move_all(app: &mut App, dx: f64, dy: f64) {
    let ids = sel_insts(app);
    if ids.is_empty() {
        return;
    }
    app.snapshot();
    for id in ids {
        shift_inst(app, &id, dx, dy);
    }
    app.changed();
}

pub fn delete_all(app: &mut App) {
    let ids = sel_insts(app);
    if ids.is_empty() {
        return;
    }
    app.snapshot();
    if let Some(a) = app.layout_mut().and_then(|l| l.get_mut("instances")).and_then(|v| v.as_array_mut()) {
        a.retain(|i| !ids.iter().any(|x| x == i.s("id")));
    }
    app.multi.clear();
    app.sel = Sel::Layout(app.cur_layout.clone());
    app.changed();
}

/// ajoute des copies d'instances au niveau courant, décalées de (dx, dy) ; renvoie les nouveaux ids
fn insert_copies(app: &mut App, src: &[Json], dx: f64, dy: f64) -> Vec<String> {
    let layers: Vec<String> = app.layout().map(|l| l.arr("layers").iter().map(|y| y.s("id").to_string()).collect()).unwrap_or_default();
    let mut out = Vec::new();
    let mut add = Vec::new();
    for j in src {
        if app.obj(j.s("obj")).is_none() {
            continue;
        }
        let mut c = j.clone();
        let nid = pf_core::uid("i");
        c["id"] = json!(nid);
        c["x"] = json!((j.f("x", 0.0) + dx).round());
        c["y"] = json!((j.f("y", 0.0) + dy).round());
        // calque absent de ce niveau : calque actif, sinon automatique
        if !layers.iter().any(|l| l == c.s("layer")) {
            match &app.view.active_layer {
                Some(a) if layers.contains(a) => c["layer"] = json!(a),
                _ => {
                    if let Some(m) = c.as_object_mut() {
                        m.remove("layer");
                    }
                }
            }
        }
        out.push(nid);
        add.push(c);
    }
    if let Some(l) = app.layout_mut() {
        if l.get("instances").is_none() {
            l["instances"] = json!([]);
        }
        l["instances"].as_array_mut().unwrap().extend(add);
    }
    out
}

pub fn duplicate_all(app: &mut App) {
    let ids = sel_insts(app);
    let src: Vec<Json> = ids.iter().filter_map(|id| app.find_inst(id).cloned()).collect();
    if src.is_empty() {
        return;
    }
    app.snapshot();
    let new = insert_copies(app, &src, 16.0, 16.0);
    select(app, new);
    app.changed();
}

pub fn copy(app: &mut App) -> usize {
    let ids = sel_insts(app);
    app.inst_clip = ids.iter().filter_map(|id| app.find_inst(id).cloned()).collect();
    app.inst_clip.len()
}

/// colle le presse-papiers : le groupe est centré sur `at` (souris), sinon décalé de 16 px
pub fn paste(app: &mut App, at: Option<(f64, f64)>) {
    if app.inst_clip.is_empty() {
        return;
    }
    let src = app.inst_clip.clone();
    let (dx, dy) = match at {
        Some((mx, my)) => {
            let n = src.len() as f64;
            let cx = src.iter().map(|j| j.f("x", 0.0)).sum::<f64>() / n;
            let cy = src.iter().map(|j| j.f("y", 0.0)).sum::<f64>() / n;
            let (mut dx, mut dy) = (mx - cx, my - cy);
            if app.view.snap {
                let gs = app.view.grid_size;
                dx = (dx / gs).round() * gs;
                dy = (dy / gs).round() * gs;
            }
            (dx, dy)
        }
        None => (16.0, 16.0),
    };
    app.snapshot();
    let new = insert_copies(app, &src, dx, dy);
    select(app, new);
    app.changed();
}

/// toutes les instances visibles et non verrouillées du niveau
pub fn select_all(app: &mut App) {
    let Some(l) = app.layout() else { return };
    let first = l.arr("layers").first().map(|y| y.s("id").to_string()).unwrap_or_default();
    let ids: Vec<String> = l
        .arr("instances")
        .iter()
        .filter(|i| {
            let ly = if l.arr("layers").iter().any(|y| y.s("id") == i.s("layer")) { i.s("layer").to_string() } else { first.clone() };
            !app.view.hidden.contains(&ly) && !app.view.locked.contains(&ly)
        })
        .map(|i| i.s("id").to_string())
        .collect();
    select(app, ids);
}

/// panneau Propriétés quand plusieurs instances sont sélectionnées
pub fn props(app: &mut App, ui: &mut egui::Ui) {
    let ids = sel_insts(app);
    let n = ids.len();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("▣").size(18.0).color(C::ACCENT()));
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(trf("{0} instances sélectionnées", &[n.to_string()])).strong().size(15.0).color(C::TEXT()));
            // résumé par objet
            let mut cnt: Vec<(String, usize)> = Vec::new();
            for id in &ids {
                if let Some(o) = app.find_inst(id).map(|i| app.obj_name(i.s("obj"))) {
                    match cnt.iter_mut().find(|c| c.0 == o) {
                        Some(c) => c.1 += 1,
                        None => cnt.push((o, 1)),
                    }
                }
            }
            let s = cnt.iter().map(|(o, k)| if *k > 1 { format!("{o} ×{k}") } else { o.clone() }).collect::<Vec<_>>().join(", ");
            ui.label(egui::RichText::new(s).size(11.0).color(C::DIM()));
        });
    });
    ui.separator();
    theme::section(ui, tr("Aligner"));
    ui.horizontal_wrapped(|ui| {
        for (a, t, tip) in [(Align::Left, "⇤", tr("Aligner à gauche")), (Align::CenterX, "↔", tr("Centrer horizontalement")), (Align::Right, "⇥", tr("Aligner à droite")), (Align::Top, "↑", tr("Aligner en haut")), (Align::CenterY, "↕", tr("Centrer verticalement")), (Align::Bottom, "↓", tr("Aligner en bas"))] {
            if ui.add(theme::btn_small(t)).on_hover_text(tip).clicked() {
                align(app, a);
            }
        }
    });
    theme::section(ui, tr("Répartir"));
    ui.horizontal_wrapped(|ui| {
        let ok = n >= 3;
        if ui.add_enabled(ok, theme::btn_small(tr("Horizontalement"))).on_hover_text(tr("Écarts égaux de gauche à droite")).on_disabled_hover_text(tr("Au moins 3 instances")).clicked() {
            distribute(app, true);
        }
        if ui.add_enabled(ok, theme::btn_small(tr("Verticalement"))).on_hover_text(tr("Écarts égaux de haut en bas")).on_disabled_hover_text(tr("Au moins 3 instances")).clicked() {
            distribute(app, false);
        }
    });
    theme::section(ui, tr("Pour toutes"));
    let first = ids.first().and_then(|id| app.find_inst(id)).cloned().unwrap_or(json!({}));
    let mut set: Option<(&str, Json)> = None;
    egui::Grid::new("multi").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ui.label(egui::RichText::new(tr("Calque")).color(C::MUTED()));
        let layers: Vec<(String, String)> = app.layout().map(|l| l.arr("layers").iter().map(|y| (y.s("id").to_string(), y.s("name").to_string())).collect()).unwrap_or_default();
        let same = ids.iter().filter_map(|id| app.find_inst(id)).all(|i| i.s("layer") == first.s("layer"));
        let cur = if same { layers.iter().find(|y| y.0 == first.s("layer")).map(|y| y.1.clone()).unwrap_or_else(|| tr("(premier calque)").into()) } else { tr("(différents)").into() };
        egui::ComboBox::from_id_salt("mlayer").selected_text(cur).show_ui(ui, |ui| {
            for (lid, ln) in &layers {
                if ui.selectable_label(false, ln).clicked() {
                    set = Some(("layer", json!(lid)));
                }
            }
        });
        ui.end_row();
        ui.label(egui::RichText::new(tr("Opacité (%)")).color(C::MUTED()));
        let mut op = first.f("opacity", 1.0) * 100.0;
        if ui.add(egui::DragValue::new(&mut op).range(0.0..=100.0)).changed() {
            set = Some(("opacity", json!((op / 100.0 * 1000.0).round() / 1000.0)));
        }
        ui.end_row();
        ui.label(egui::RichText::new(tr("Visible")).color(C::MUTED()));
        let mut v = first.get("visible") != Some(&Json::Bool(false));
        if ui.checkbox(&mut v, "").changed() {
            set = Some(("visible", json!(v)));
        }
        ui.end_row();
        ui.label(egui::RichText::new(tr("Miroir")).color(C::MUTED()));
        let mut m = first.get("mirrored") == Some(&Json::Bool(true));
        if ui.checkbox(&mut m, "").changed() {
            set = Some(("mirrored", json!(m)));
        }
        ui.end_row();
        ui.label(egui::RichText::new(tr("Angle (°)")).color(C::MUTED()));
        let mut a = first.f("angle", 0.0);
        if ui.add(egui::DragValue::new(&mut a).speed(1.0)).changed() {
            set = Some(("angle", json!(a)));
        }
        ui.end_row();
    });
    if let Some((k, v)) = set {
        app.snapshot();
        for id in &ids {
            if let Some(i) = app.inst_mut(id) {
                i[k] = v.clone();
            }
        }
        app.changed();
    }
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        if ui.add(theme::btn_small(tr("Dupliquer"))).on_hover_text(tr("Ctrl+D")).clicked() {
            duplicate_all(app);
        }
        if ui.add(theme::btn_small(tr("Copier"))).on_hover_text(tr("Ctrl+C — on peut coller dans un autre niveau")).clicked() {
            copy(app);
        }
        if ui.add(theme::btn_small(tr("Supprimer"))).on_hover_text(tr("Suppr")).clicked() {
            delete_all(app);
        }
        if ui.add(theme::btn_small(tr("Désélectionner"))).on_hover_text(tr("Échap")).clicked() {
            select(app, vec![]);
        }
    });
    ui.add_space(6.0);
    ui.label(egui::RichText::new(tr("Maj+clic : ajouter / retirer une instance · glisser dans le vide : sélection rectangle · Ctrl+A : tout sélectionner")).size(11.0).color(C::DIM()));
}
