//! Panneau Propriétés (instance, objet, niveau, variable, paramètres du jeu) et débogueur.

use crate::theme::{self, C};
use crate::{App, Play, Sel};
use eframe::egui;
use pf_core::{json, Json, JsonExt};

/// largeur jusqu'au bord visible du panneau : un champ « pleine largeur » ne doit jamais
/// élargir le panneau (sinon il grandit dès que la colonne des libellés change de largeur)
fn fill_w(ui: &egui::Ui) -> f32 {
    (content_right(ui) - ui.cursor().left()).max(40.0)
}

/// bord droit utile du panneau, mémorisé avant la zone de défilement
/// (la zone de défilement, elle, s'élargit avec son contenu : on ne peut pas s'y fier)
fn content_right(ui: &egui::Ui) -> f32 {
    ui.ctx().data(|d| d.get_temp::<f32>(egui::Id::new("props_right"))).unwrap_or_else(|| ui.clip_rect().right() - 16.0)
}

pub fn jdisp(v: &Json) -> String {
    match v {
        Json::String(s) => format!("\"{}\"", s),
        Json::Null => "0".into(),
        v => pf_runtime::rt::jstr(v),
    }
}

fn label_for(k: &str) -> &str {
    match k {
        "name" => crate::i18n::tr("Nom"),
        "text" => crate::i18n::tr("Texte"),
        "size" => crate::i18n::tr("Taille du texte"),
        "color" => crate::i18n::tr("Couleur"),
        "bold" => crate::i18n::tr("Gras"),
        "align" => crate::i18n::tr("Alignement"),
        "font" => crate::i18n::tr("Police"),
        "outline" => crate::i18n::tr("Contour"),
        "outlineColor" => crate::i18n::tr("Couleur du contour"),
        "defW" => crate::i18n::tr("Largeur par défaut"),
        "defH" => crate::i18n::tr("Hauteur par défaut"),
        "scale" => crate::i18n::tr("Échelle"),
        "w" => crate::i18n::tr("Largeur de l’image"),
        "h" => crate::i18n::tr("Hauteur de l’image"),
        "ox" => crate::i18n::tr("Origine X"),
        "oy" => crate::i18n::tr("Origine Y"),
        "collision" => crate::i18n::tr("Collision"),
        "bg" => crate::i18n::tr("Fond"),
        "fg" => crate::i18n::tr("Premier plan"),
        "hoverBg" => crate::i18n::tr("Fond (survol)"),
        "hoverFg" => crate::i18n::tr("Texte (survol)"),
        "radius" => crate::i18n::tr("Arrondi / rayon"),
        "border" => crate::i18n::tr("Bordure"),
        "borderColor" => crate::i18n::tr("Couleur de bordure"),
        "shadow" => crate::i18n::tr("Ombre"),
        "focusColor" => crate::i18n::tr("Couleur du focus"),
        "max" => crate::i18n::tr("Maximum"),
        "value" => crate::i18n::tr("Valeur"),
        "ghostColor" => crate::i18n::tr("Couleur de la traîne"),
        "lowColor" => crate::i18n::tr("Couleur si presque vide"),
        "lowAt" => crate::i18n::tr("Presque vide sous (%)"),
        "dir" => crate::i18n::tr("Direction"),
        "segments" => crate::i18n::tr("Segments"),
        "showText" => crate::i18n::tr("Afficher la valeur"),
        "format" => crate::i18n::tr("Format du texte"),
        "textColor" => crate::i18n::tr("Couleur du texte"),
        "smooth" => crate::i18n::tr("Animation douce"),
        "table" => crate::i18n::tr("Classement"),
        "rows" => crate::i18n::tr("Lignes"),
        "title" => crate::i18n::tr("Titre"),
        "showTitle" => crate::i18n::tr("Afficher le titre"),
        "accent" => crate::i18n::tr("Couleur d’accent"),
        "bgAlpha" => crate::i18n::tr("Opacité du fond (%)"),
        "suffix" => crate::i18n::tr("Suffixe"),
        "tw" => crate::i18n::tr("Largeur d’une tuile"),
        "th" => crate::i18n::tr("Hauteur d’une tuile"),
        "track" => crate::i18n::tr("Piste (longueur courbe hauteur)"),
        "mode" => crate::i18n::tr("Mode"),
        "follow" => crate::i18n::tr("Suivre l’objet"),
        "shape" => crate::i18n::tr("Forme"),
        "solids" => crate::i18n::tr("Afficher les solides"),
        "solidColor" => crate::i18n::tr("Couleur des solides"),
        "followColor" => crate::i18n::tr("Couleur du suivi"),
        "borderW" => crate::i18n::tr("Épaisseur de bordure"),
        _ => k,
    }
}

const SKIP: [&str; 13] = ["id", "kind", "anims", "image", "behaviors", "vars", "pt", "cbox", "points", "tcol", "dots", "members", "items"];

fn is_color_key(k: &str, v: &str) -> bool {
    v.starts_with('#') && (v.len() == 7 || v.len() == 4) || (v.is_empty() && (k.to_lowercase().contains("color") || k == "lowColor"))
}

/// éditeur générique d'un champ JSON ; renvoie true si modifié
fn field(ui: &mut egui::Ui, k: &str, v: &mut Json, label: &str) -> bool {
    let mut ch = false;
    ui.label(egui::RichText::new(label).color(C::MUTED()));
    match v {
        Json::Bool(b) => {
            ch |= ui.checkbox(b, "").changed();
        }
        Json::Number(n) => {
            let mut x = n.as_f64().unwrap_or(0.0);
            let sp = if x.abs() < 5.0 && x.fract() != 0.0 { 0.01 } else { 1.0 };
            let r = ui.add(egui::DragValue::new(&mut x).speed(sp));
            if r.changed() {
                *v = json!(x);
                ch = true;
            }
        }
        Json::String(s) => {
            if is_color_key(k, s) {
                ui.horizontal(|ui| {
                    let mut c = theme::color32(if s.is_empty() { "#000000" } else { s });
                    if ui.color_edit_button_srgba(&mut c).changed() {
                        *s = format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b());
                        ch = true;
                    }
                    ch |= ui.add(egui::TextEdit::singleline(s).desired_width(80.0)).changed();
                });
            } else if k == "track" || k == "text" && s.contains('\n') || s.len() > 40 {
                ch |= ui.add(egui::TextEdit::multiline(s).desired_rows(3).desired_width(fill_w(ui)).font(egui::TextStyle::Monospace)).changed();
            } else {
                ch |= ui.add(egui::TextEdit::singleline(s).desired_width(fill_w(ui))).changed();
            }
        }
        Json::Null => {
            ui.label("—");
        }
        other => {
            let mut s = other.to_string();
            if ui.add(egui::TextEdit::multiline(&mut s).desired_rows(2).font(egui::TextStyle::Monospace)).changed() {
                if let Ok(j) = serde_json::from_str(&s) {
                    *other = j;
                    ch = true;
                }
            }
        }
    }
    ui.end_row();
    ch
}

/// éditeur d'un paramètre de comportement d'après le catalogue
fn beh_param(ui: &mut egui::Ui, app: &App, def: &Json, v: &mut Json) -> bool {
    let t = def.s("t");
    let mut ch = false;
    match t {
        "bool" => {
            let mut b = pf_runtime::rt::jtruthy(Some(v));
            if ui.checkbox(&mut b, "").changed() {
                *v = Json::Bool(b);
                ch = true;
            }
        }
        "select" => {
            let cur = pf_runtime::rt::jstr(v);
            let opts = def.arr("o").to_vec();
            let lab = opts.iter().find(|o| o[0].as_str() == Some(&cur)).and_then(|o| o[1].as_str()).unwrap_or(&cur).to_string();
            egui::ComboBox::from_id_salt(def as *const _ as usize).selected_text(lab).show_ui(ui, |ui| {
                for o in &opts {
                    if ui.selectable_label(o[0].as_str() == Some(&cur), o[1].as_str().unwrap_or("")).clicked() {
                        *v = o[0].clone();
                        ch = true;
                    }
                }
            });
        }
        "obj" => {
            let cur = pf_runtime::rt::jstr(v);
            egui::ComboBox::from_id_salt(def as *const _ as usize).selected_text(if cur.is_empty() || cur == "undefined" { crate::i18n::tr("(aucun)").to_string() } else { cur.clone() }).show_ui(ui, |ui| {
                if ui.selectable_label(cur.is_empty(), crate::i18n::tr("(aucun)")).clicked() {
                    *v = Json::String(String::new());
                    ch = true;
                }
                for o in app.proj.arr("objects") {
                    if ui.selectable_label(cur == o.s("name"), o.s("name")).clicked() {
                        *v = Json::String(o.s("name").to_string());
                        ch = true;
                    }
                }
            });
        }
        "color" => {
            let s = pf_runtime::rt::jstr(v);
            let mut c = theme::color32(&s);
            if ui.color_edit_button_srgba(&mut c).changed() {
                *v = Json::String(format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()));
                ch = true;
            }
        }
        _ => {
            let mut x = pf_core::Val::from_json(v).num0();
            let mut dv = egui::DragValue::new(&mut x).speed(def.f("step", 1.0).max(0.01));
            if def.has("min") && def.has("max") {
                dv = dv.range(def.f("min", 0.0)..=def.f("max", 1e9));
            }
            if ui.add(dv).changed() {
                *v = json!(x);
                ch = true;
            }
        }
    }
    ch
}

pub fn panel(app: &mut App, ctx: &egui::Context, now: f64) {
    let w0 = crate::util::panel_width(ctx, "props");
    egui::SidePanel::right("props").default_width(300.0).min_width(240.0).frame(egui::Frame::new().fill(C::BG1()).inner_margin(egui::Margin::symmetric(10, 8)).stroke(egui::Stroke::new(1.0_f32, C::LINE()))).show(ctx, |ui| {
        let right = ui.max_rect().right() - 4.0;
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("props_right"), right));
        // max_width : le contenu ne peut pas élargir le panneau (il serait coupé, pas agrandi)
        egui::ScrollArea::vertical().max_width(ui.available_width()).auto_shrink([false, false]).show(ui, |ui| {
            match app.sel.clone() {
                Sel::Inst(id) => inst_props(app, ui, &id),
                Sel::Obj(id) => obj_props(app, ui, &id, now),
                Sel::Layout(_) => layout_props(app, ui),
                Sel::Global(n) => global_props(app, ui, &n),
                Sel::Sheet(id) => sheet_props(app, ui, &id),
                Sel::Sound(id) => sound_props(app, ui, &id),
                Sel::None => settings_props(app, ui),
            }
        });
    });
    crate::util::keep_panel_width(ctx, "props", w0, true);
}

fn header(ui: &mut egui::Ui, icon: &str, title: &str, sub: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).size(18.0).color(C::ACCENT()));
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).strong().size(15.0).color(C::TEXT()));
            ui.label(egui::RichText::new(sub).size(11.0).color(C::DIM()));
        });
    });
    ui.separator();
}

fn edit_inst<F: FnOnce(&mut Json)>(app: &mut App, id: &str, f: F) {
    app.snapshot();
    if let Some(i) = app.inst_mut(id) {
        f(i);
    }
    app.changed();
}

fn inst_props(app: &mut App, ui: &mut egui::Ui, id: &str) {
    let i = match app.find_inst(id) {
        Some(i) => i.clone(),
        None => return,
    };
    let o = app.obj(i.s("obj")).cloned().unwrap_or(Json::Null);
    header(ui, crate::tree::glyph(o.s("kind")), o.s("name"), crate::i18n::tr("Instance dans le niveau"));
    if ui.add(theme::btn_small(crate::i18n::tr("Modifier l’objet →"))).clicked() {
        app.sel = Sel::Obj(o.s("id").to_string());
        return;
    }
    egui::Grid::new("ip").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        for (k, lab) in [("x", "X"), ("y", "Y"), ("w", crate::i18n::tr("Largeur")), ("h", crate::i18n::tr("Hauteur")), ("angle", crate::i18n::tr("Angle (°)"))] {
            ui.label(egui::RichText::new(lab).color(C::MUTED()));
            let mut v = i.f(k, 0.0);
            let r = ui.add(egui::DragValue::new(&mut v).speed(1.0));
            if r.drag_started() || (r.changed() && !r.dragged()) {
                app.snapshot();
            }
            if r.changed() {
                if let Some(ii) = app.inst_mut(id) {
                    ii[k] = json!(v);
                }
                app.changed();
            }
            ui.end_row();
        }
        ui.label(egui::RichText::new(crate::i18n::tr("Opacité (%)")).color(C::MUTED()));
        let mut op = i.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0) * 100.0;
        if ui.add(egui::Slider::new(&mut op, 0.0..=100.0)).changed() {
            if let Some(ii) = app.inst_mut(id) {
                ii["opacity"] = json!(op / 100.0);
            }
            app.changed();
        }
        ui.end_row();
        for (k, lab, def) in [("visible", crate::i18n::tr("Visible"), true), ("mirrored", crate::i18n::tr("Miroir"), false), ("hud", crate::i18n::tr("Fixe à l’écran (interface)"), false), ("off", crate::i18n::tr("Désactivée (absente du jeu)"), false)] {
            ui.label(egui::RichText::new(lab).color(C::MUTED()));
            let mut b = i.get(k).map(|v| v.as_bool().unwrap_or(def)).unwrap_or(def);
            if ui.checkbox(&mut b, "").changed() {
                edit_inst(app, id, |ii| ii[k] = Json::Bool(b));
            }
            ui.end_row();
        }
        // calque
        if let Some(l) = app.layout().cloned() {
            ui.label(egui::RichText::new(crate::i18n::tr("Calque")).color(C::MUTED()));
            let cur = i.s("layer").to_string();
            let name = l.arr("layers").iter().find(|y| y.s("id") == cur).or_else(|| l.arr("layers").first()).map(|y| y.s("name").to_string()).unwrap_or_default();
            egui::ComboBox::from_id_salt("ilayer").selected_text(name).show_ui(ui, |ui| {
                for y in l.arr("layers") {
                    if ui.selectable_label(y.s("id") == cur, y.s("name")).clicked() {
                        let yid = y.s("id").to_string();
                        edit_inst(app, id, |ii| ii["layer"] = Json::String(yid));
                    }
                }
            });
            ui.end_row();
        }
        let k = o.s("kind");
        if k == "text" || k == "button" {
            ui.label(egui::RichText::new(crate::i18n::tr("Texte")).color(C::MUTED()));
            let mut t = i.get("text").map(pf_runtime::rt::jstr).unwrap_or_else(|| o.s("text").to_string());
            if ui.add(egui::TextEdit::multiline(&mut t).desired_rows(2)).changed() {
                if let Some(ii) = app.inst_mut(id) {
                    ii["text"] = Json::String(t);
                }
                app.changed();
            }
            ui.end_row();
        }
        if k == "bar" {
            ui.label(egui::RichText::new(crate::i18n::tr("Valeur de départ")).color(C::MUTED()));
            let mut v = i.get("value").and_then(|v| v.as_f64()).unwrap_or(o.f("value", 100.0));
            if ui.add(egui::DragValue::new(&mut v)).changed() {
                if let Some(ii) = app.inst_mut(id) {
                    ii["value"] = json!(v);
                }
                app.changed();
            }
            ui.end_row();
        }
    });
    if o.s("kind") == "tilemap" {
        crate::tools::palette(app, ui, id);
    }
    if o.arr("behaviors").iter().any(|b| b.s("type") == "path") {
        crate::tools::path_panel(app, ui, id);
    }
    // variables d'instance
    let vars = o.arr("vars").to_vec();
    if !vars.is_empty() {
        theme::section(ui, crate::i18n::tr("Variables de cette instance"));
        egui::Grid::new("ivars").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
            for v in vars {
                let n = v.s("name").to_string();
                ui.label(egui::RichText::new(&n).color(C::MUTED()));
                let cur = i.get("vars").and_then(|m| m.get(&n)).cloned().unwrap_or_else(|| v.get("value").cloned().unwrap_or(json!(0)));
                let mut s = pf_runtime::rt::jstr(&cur);
                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(120.0)).changed() {
                    let nv = if v.s("type") == "text" { Json::String(s) } else { json!(pf_core::Val::S(s).num0()) };
                    if let Some(ii) = app.inst_mut(id) {
                        if !ii.get("vars").map(|x| x.is_object()).unwrap_or(false) {
                            ii["vars"] = json!({});
                        }
                        ii["vars"][&n] = nv;
                    }
                    app.changed();
                }
                ui.end_row();
            }
        });
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.add(theme::btn_small(crate::i18n::tr("Dupliquer"))).clicked() {
            app.duplicate_selection();
        }
        if ui.add(theme::btn_small(crate::i18n::tr("Au premier plan"))).clicked() {
            app.snapshot();
            let id = id.to_string();
            if let Some(a) = app.layout_mut().and_then(|l| l.get_mut("instances")).and_then(|v| v.as_array_mut()) {
                if let Some(k) = a.iter().position(|x| x.s("id") == id) {
                    let x = a.remove(k);
                    a.push(x);
                }
            }
            app.changed();
        }
        if ui.add(theme::btn_small(crate::i18n::tr("Supprimer"))).clicked() {
            app.delete_selection();
        }
    });
}

fn obj_props(app: &mut App, ui: &mut egui::Ui, id: &str, now: f64) {
    let o = match app.obj(id) {
        Some(o) => o.clone(),
        None => return,
    };
    let kind = o.s("kind").to_string();
    header(ui, crate::tree::glyph(&kind), o.s("name"), crate::tree::kind_name(&kind));
    let mut edited = o.clone();
    let mut ch = false;
    let mut rename: Option<(String, String)> = None;
    egui::Grid::new("op").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        if let Some(m) = edited.as_object_mut() {
            let keys: Vec<String> = m.keys().cloned().collect();
            for k in keys {
                if SKIP.contains(&k.as_str()) {
                    continue;
                }
                let old_name = if k == "name" { m["name"].as_str().map(|s| s.to_string()) } else { None };
                if field(ui, &k, m.get_mut(&k).unwrap(), label_for(&k)) {
                    ch = true;
                    if let Some(on) = old_name {
                        rename = Some((on, m["name"].as_str().unwrap_or("").to_string()));
                    }
                }
            }
        }
    });
    // particules
    if kind == "particles" {
        theme::section(ui, crate::i18n::tr("Émetteur de particules"));
        if ui.add(theme::btn_accent(crate::i18n::tr("✦  Éditeur de particules (aperçu en direct)"))).clicked() {
            app.part_ed = crate::particles::PartEd::open(app, id);
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(crate::i18n::tr("Préréglage :")).color(C::MUTED()));
            if let Some(pr) = pf_core::catalog::ui_catalog()["PTPRESETS"].as_object() {
                for (k, p) in pr {
                    if ui.add(theme::btn_small(p.s("n"))).clicked() {
                        let mut q = p.clone();
                        q.as_object_mut().unwrap().remove("n");
                        let _ = k;
                        edited["pt"] = q;
                        ch = true;
                    }
                }
            }
        });
        egui::CollapsingHeader::new(crate::i18n::tr("Réglages bruts")).id_salt("ptraw").show(ui, |ui| {
            egui::Grid::new("pt").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                if let Some(m) = edited.get_mut("pt").and_then(|p| p.as_object_mut()) {
                    let keys: Vec<String> = m.keys().cloned().collect();
                    for k in keys {
                        ch |= field(ui, &k, m.get_mut(&k).unwrap(), &k);
                    }
                }
            });
        });
    }
    if kind == "array" {
        theme::section(ui, crate::i18n::tr("Éléments (JSON)"));
        let mut s = edited.get("items").map(|v| v.to_string()).unwrap_or_else(|| "[]".into());
        if ui.add(egui::TextEdit::multiline(&mut s).desired_rows(3).font(egui::TextStyle::Monospace).desired_width(fill_w(ui))).changed() {
            if let Ok(j) = serde_json::from_str::<Json>(&s) {
                if j.is_array() {
                    edited["items"] = j;
                    ch = true;
                }
            }
        }
    }
    if kind == "dict" {
        theme::section(ui, crate::i18n::tr("Entrées (JSON)"));
        let mut s = edited.get("entries").map(|v| v.to_string()).unwrap_or_else(|| "{}".into());
        if ui.add(egui::TextEdit::multiline(&mut s).desired_rows(3).font(egui::TextStyle::Monospace).desired_width(fill_w(ui))).changed() {
            if let Ok(j) = serde_json::from_str::<Json>(&s) {
                if j.is_object() {
                    edited["entries"] = j;
                    ch = true;
                }
            }
        }
    }
    if kind == "family" {
        theme::section(ui, crate::i18n::tr("Membres de la famille"));
        let mem: Vec<String> = edited.arr("members").iter().filter_map(|m| m.as_str().map(|s| s.to_string())).collect();
        for ob in app.proj.arr("objects") {
            let k2 = ob.s("kind");
            if k2 == "family" || k2 == "array" || k2 == "dict" {
                continue;
            }
            let mut on = mem.contains(&ob.s("id").to_string());
            if ui.checkbox(&mut on, ob.s("name")).changed() {
                let mut m2 = mem.clone();
                if on {
                    m2.push(ob.s("id").to_string());
                } else {
                    m2.retain(|x| x != ob.s("id"));
                }
                edited["members"] = json!(m2);
                ch = true;
            }
        }
    }
    if kind == "road3d" {
        if ui.add(theme::btn_accent(crate::i18n::tr("🛣  Éditeur visuel de la piste"))).clicked() {
            app.track_ed = Some(crate::more::TrackEd::new(app, id));
        }
    }
    if kind == "sprite" {
        if ui.add(theme::btn_accent(crate::i18n::tr("✏  Ouvrir l’éditeur de sprites"))).clicked() {
            app.sprite = Some(crate::sprite::SpriteEd::open(&o));
        }
        theme::section(ui, crate::i18n::tr("Animations"));
        let anims = edited.arr("anims").to_vec();
        for (k, a) in anims.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(crate::i18n::trf("{0} — {1} image(s), {2} img/s{3}", &[format!("{}", a.s("name")), format!("{}", a.arr("frames").len()), format!("{}", a.f("speed", 0.0)), format!("{}", if a.get("loop") == Some(&Json::Bool(false)) { "" } else { crate::i18n::tr(", en boucle") })])).color(C::TEXT2()));
                if ui.add(theme::btn_small(crate::i18n::tr("Image…"))).on_hover_text(crate::i18n::tr("Remplacer la première image (PNG)")).clicked() {
                    if let Some(p) = rfd::FileDialog::new().add_filter(crate::i18n::tr("Image"), &["png", "jpg", "jpeg", "gif", "webp"]).pick_file() {
                        if let Ok(b) = std::fs::read(&p) {
                            if let Ok(img) = image::load_from_memory(&b) {
                                let mut out = Vec::new();
                                let rgba = img.to_rgba8();
                                use image::ImageEncoder;
                                let _ = image::codecs::png::PngEncoder::new(&mut out).write_image(&rgba, rgba.width(), rgba.height(), image::ExtendedColorType::Rgba8);
                                let url = format!("data:image/png;base64,{}", pf_core::b64_encode(&out));
                                edited["anims"][k]["frames"] = json!([url]);
                                if k == 0 {
                                    edited["w"] = json!(rgba.width());
                                    edited["h"] = json!(rgba.height());
                                    edited["ox"] = json!(rgba.width() / 2);
                                    edited["oy"] = json!(rgba.height() / 2);
                                    edited["cbox"] = json!({"x":0,"y":0,"w":rgba.width(),"h":rgba.height()});
                                }
                                ch = true;
                            }
                        }
                    }
                }
            });
        }
    }
    if kind == "tiled" || kind == "tilemap" {
        if ui.add(theme::btn_small(crate::i18n::tr("Remplacer l’image…"))).clicked() {
            if let Some(p) = rfd::FileDialog::new().add_filter(crate::i18n::tr("Image"), &["png", "jpg", "jpeg", "gif", "webp"]).pick_file() {
                if let Ok(b) = std::fs::read(&p) {
                    let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                    let mime = if ext == "png" { "image/png" } else if ext == "gif" { "image/gif" } else if ext == "webp" { "image/webp" } else { "image/jpeg" };
                    edited["image"] = Json::String(format!("data:{};base64,{}", mime, pf_core::b64_encode(&b)));
                    ch = true;
                }
            }
        }
    }
    // comportements
    if !["array", "dict"].contains(&kind.as_str()) {
        theme::section(ui, crate::i18n::tr("Comportements"));
        let bhs = edited.arr("behaviors").to_vec();
        let cat = pf_core::catalog::ui_behaviors();
        let mut remove = None;
        for (bi, b) in bhs.iter().enumerate() {
            let t = b.s("type");
            let d = cat.get(t).cloned().unwrap_or(Json::Null);
            egui::Frame::new().fill(C::BG2()).corner_radius(8.0).inner_margin(8.0).stroke(egui::Stroke::new(1.0_f32, C::LINE())).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(d.s("n")).strong().color(C::TEXT()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✕").on_hover_text(crate::i18n::tr("Retirer")).clicked() {
                            remove = Some(bi);
                        }
                    });
                });
                ui.label(egui::RichText::new(d.s("d")).size(11.0).color(C::DIM()));
                if let Some(ps) = d.get("p").and_then(|p| p.as_object()) {
                    egui::Grid::new(("bp", bi)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                        for (pk, pd) in ps {
                            ui.label(egui::RichText::new(pd.s("l")).size(12.0).color(C::MUTED()));
                            let mut v = b.get("props").and_then(|p| p.get(pk)).cloned().unwrap_or_else(|| pd.get("d").cloned().unwrap_or(Json::Null));
                            if beh_param(ui, app, pd, &mut v) {
                                if !edited["behaviors"][bi].get("props").map(|p| p.is_object()).unwrap_or(false) {
                                    edited["behaviors"][bi]["props"] = json!({});
                                }
                                edited["behaviors"][bi]["props"][pk] = v;
                                ch = true;
                            }
                            ui.end_row();
                        }
                    });
                }
            });
        }
        if let Some(k) = remove {
            edited["behaviors"].as_array_mut().unwrap().remove(k);
            ch = true;
        }
        ui.menu_button(crate::i18n::tr("+ Ajouter un comportement"), |ui| {
            for (k, d) in cat {
                if bhs.iter().any(|b| b.s("type") == k) {
                    continue;
                }
                if ui.button(d.s("n")).on_hover_text(d.s("d")).clicked() {
                    if !edited.get("behaviors").map(|b| b.is_array()).unwrap_or(false) {
                        edited["behaviors"] = json!([]);
                    }
                    edited["behaviors"].as_array_mut().unwrap().push(json!({"type":k,"props":pf_core::catalog::bdefaults(k)}));
                    ch = true;
                    ui.close_menu();
                }
            }
        });
        // variables d'instance
        theme::section(ui, crate::i18n::tr("Variables d’instance"));
        let vars = edited.arr("vars").to_vec();
        let mut del = None;
        egui::Grid::new("ov").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
            for (k, v) in vars.iter().enumerate() {
                let mut n = v.s("name").to_string();
                if ui.add(egui::TextEdit::singleline(&mut n).desired_width(90.0)).changed() {
                    edited["vars"][k]["name"] = Json::String(n);
                    ch = true;
                }
                let mut tx = v.s("type") == "text";
                if ui.checkbox(&mut tx, crate::i18n::tr("texte")).changed() {
                    edited["vars"][k]["type"] = Json::String(if tx { "text" } else { "number" }.into());
                    ch = true;
                }
                let mut val = pf_runtime::rt::jstr(v.get("value").unwrap_or(&json!(0)));
                if ui.add(egui::TextEdit::singleline(&mut val).desired_width(60.0)).changed() {
                    edited["vars"][k]["value"] = if tx { Json::String(val) } else { json!(pf_core::Val::S(val).num0()) };
                    ch = true;
                }
                if ui.small_button("✕").clicked() {
                    del = Some(k);
                }
                ui.end_row();
            }
        });
        if let Some(k) = del {
            edited["vars"].as_array_mut().unwrap().remove(k);
            ch = true;
        }
        if ui.add(theme::btn_small(crate::i18n::tr("+ Variable"))).clicked() {
            if !edited.get("vars").map(|b| b.is_array()).unwrap_or(false) {
                edited["vars"] = json!([]);
            }
            let n = edited.arr("vars").len() + 1;
            edited["vars"].as_array_mut().unwrap().push(json!({"name":format!("var{}", n),"type":"number","value":0}));
            ch = true;
        }
    }
    if ch {
        if let Some((a, b)) = &rename {
            if a != b && app.proj.arr("objects").iter().any(|x| x.s("name") == b && x.s("id") != id) {
                app.toast(crate::i18n::trf("Le nom « {0} » est déjà utilisé.", &[format!("{}", b)]), true, now);
                return;
            }
        }
        app.snapshot();
        if let Some(t) = app.obj_mut(id) {
            *t = edited;
        }
        if let Some((a, b)) = rename {
            if !b.is_empty() && a != b {
                crate::evsheet::rename_in_exprs(&mut app.proj, &a, &b);
            }
        }
        app.changed();
    }
}

fn layout_props(app: &mut App, ui: &mut egui::Ui) {
    let l = match app.layout() {
        Some(l) => l.clone(),
        None => return,
    };
    header(ui, "▦", l.s("name"), crate::i18n::tr("Niveau"));
    let mut e = l.clone();
    let mut ch = false;
    egui::Grid::new("lp").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ch |= field(ui, "name", &mut e["name"], crate::i18n::tr("Nom"));
        if e.get("w").is_none() {
            e["w"] = json!(854);
        }
        if e.get("h").is_none() {
            e["h"] = json!(480);
        }
        ch |= field(ui, "w", &mut e["w"], crate::i18n::tr("Largeur (px)"));
        ch |= field(ui, "h", &mut e["h"], crate::i18n::tr("Hauteur (px)"));
        if e.get("bg").is_none() {
            e["bg"] = json!("#000000");
        }
        ch |= field(ui, "bg", &mut e["bg"], crate::i18n::tr("Couleur de fond"));
        ui.label(egui::RichText::new(crate::i18n::tr("Feuille d’événements")).color(C::MUTED()));
        let cur = e.s("sheet").to_string();
        let nm = app.proj.arr("sheets").iter().find(|s| s.s("id") == cur).map(|s| s.s("name").to_string()).unwrap_or_else(|| crate::i18n::tr("(aucune)").into());
        egui::ComboBox::from_id_salt("lsheet").selected_text(nm).show_ui(ui, |ui| {
            if ui.selectable_label(cur.is_empty(), crate::i18n::tr("(aucune)")).clicked() {
                e["sheet"] = json!("");
                ch = true;
            }
            for s in app.proj.arr("sheets") {
                if ui.selectable_label(s.s("id") == cur, s.s("name")).clicked() {
                    e["sheet"] = Json::String(s.s("id").to_string());
                    ch = true;
                }
            }
        });
        ui.end_row();
    });
    theme::section(ui, crate::i18n::tr("Calques (de l’arrière vers l’avant)"));
    let layers = e.arr("layers").to_vec();
    let mut mv: Option<(usize, i32)> = None;
    let mut del = None;
    for (k, y) in layers.iter().enumerate() {
        egui::Frame::new().fill(C::BG2()).corner_radius(8.0).inner_margin(6.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut n = y.s("name").to_string();
                if ui.add(egui::TextEdit::singleline(&mut n).desired_width(110.0)).changed() {
                    e["layers"][k]["name"] = Json::String(n);
                    ch = true;
                }
                if ui.small_button("↑").clicked() && k > 0 {
                    mv = Some((k, -1));
                }
                if ui.small_button("↓").clicked() && k + 1 < layers.len() {
                    mv = Some((k, 1));
                }
                if ui.small_button("✕").clicked() && layers.len() > 1 {
                    del = Some(k);
                }
            });
            ui.horizontal(|ui| {
                let mut px = y.f("parallaxX", 100.0);
                ui.label(egui::RichText::new(crate::i18n::tr("Parallaxe %")).size(11.0).color(C::MUTED()));
                if ui.add(egui::DragValue::new(&mut px).range(0.0..=400.0)).on_hover_text(crate::i18n::tr("0 = fixe à l’écran (interface), 100 = normal, <100 = arrière-plan lointain")).changed() {
                    e["layers"][k]["parallaxX"] = json!(px);
                    e["layers"][k]["parallaxY"] = json!(px);
                    ch = true;
                }
                let mut op = y.f("opacity", 100.0);
                ui.label(egui::RichText::new(crate::i18n::tr("Opacité")).size(11.0).color(C::MUTED()));
                if ui.add(egui::DragValue::new(&mut op).range(0.0..=100.0)).changed() {
                    e["layers"][k]["opacity"] = json!(op);
                    ch = true;
                }
                let mut vis = y.get("visible") != Some(&Json::Bool(false));
                if ui.checkbox(&mut vis, crate::i18n::tr("visible")).changed() {
                    e["layers"][k]["visible"] = Json::Bool(vis);
                    ch = true;
                }
            });
        });
    }
    if let Some((k, d)) = mv {
        let a = e["layers"].as_array_mut().unwrap();
        a.swap(k, (k as i32 + d) as usize);
        ch = true;
    }
    if let Some(k) = del {
        e["layers"].as_array_mut().unwrap().remove(k);
        ch = true;
    }
    if ui.add(theme::btn_small(crate::i18n::tr("+ Calque"))).clicked() {
        if !e.get("layers").map(|x| x.is_array()).unwrap_or(false) {
            e["layers"] = json!([]);
        }
        let n = e.arr("layers").len() + 1;
        e["layers"].as_array_mut().unwrap().push(json!({"id":pf_core::uid("y"),"name":crate::i18n::trf("Calque {0}", &[format!("{}", n)]),"parallaxX":100,"parallaxY":100,"opacity":100,"visible":true}));
        ch = true;
    }
    if ch {
        app.snapshot();
        if let Some(t) = app.layout_mut() {
            *t = e;
        }
        app.changed();
    }
}

fn global_props(app: &mut App, ui: &mut egui::Ui, n: &str) {
    let k = match app.proj.arr("globals").iter().position(|g| g.s("name") == n) {
        Some(k) => k,
        None => return,
    };
    let g = app.proj["globals"][k].clone();
    header(ui, "𝑥", n, crate::i18n::tr("Variable globale"));
    let mut e = g.clone();
    let mut ch = false;
    egui::Grid::new("gp").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ch |= field(ui, "name", &mut e["name"], crate::i18n::tr("Nom"));
        ui.label(egui::RichText::new(crate::i18n::tr("Type")).color(C::MUTED()));
        let mut tx = e.s("type") == "text";
        if ui.checkbox(&mut tx, crate::i18n::tr("texte")).changed() {
            e["type"] = Json::String(if tx { "text".into() } else { "number".into() });
            e["value"] = if tx { Json::String(pf_runtime::rt::jstr(e.get("value").unwrap_or(&json!(0)))) } else { json!(0) };
            ch = true;
        }
        ui.end_row();
        if e.get("value").is_none() {
            e["value"] = json!(0);
        }
        ch |= field(ui, "value", &mut e["value"], crate::i18n::tr("Valeur de départ"));
    });
    if ui.add(theme::btn_small(crate::i18n::tr("Supprimer la variable"))).clicked() {
        app.snapshot();
        app.proj["globals"].as_array_mut().unwrap().remove(k);
        app.sel = crate::Sel::None;
        app.changed();
        return;
    }
    if ch {
        app.snapshot();
        let old = g.s("name").to_string();
        let new = e.s("name").to_string();
        app.proj["globals"][k] = e;
        if old != new && !new.is_empty() {
            crate::evsheet::rename_in_exprs(&mut app.proj, &old, &new);
            app.sel = Sel::Global(new);
        }
        app.changed();
    }
}

fn sheet_props(app: &mut App, ui: &mut egui::Ui, id: &str) {
    let k = match app.proj.arr("sheets").iter().position(|s| s.s("id") == id) {
        Some(k) => k,
        None => return,
    };
    let s = app.proj["sheets"][k].clone();
    header(ui, "☰", s.s("name"), crate::i18n::tr("Feuille d’événements"));
    let mut e = s.clone();
    egui::Grid::new("sp").num_columns(2).show(ui, |ui| {
        if field(ui, "name", &mut e["name"], crate::i18n::tr("Nom")) {
            app.snapshot();
            app.proj["sheets"][k] = e.clone();
            app.changed();
        }
    });
    ui.label(egui::RichText::new(crate::i18n::trf("{0} événement(s)", &[format!("{}", crate::tree::count_events(s.arr("events")))])).color(C::DIM()));
    let used = app.proj.arr("layouts").iter().any(|l| l.s("sheet") == id);
    if ui.add_enabled(!used, theme::btn_small(crate::i18n::tr("Supprimer la feuille"))).on_disabled_hover_text(crate::i18n::tr("Utilisée par un niveau")).clicked() {
        app.snapshot();
        app.proj["sheets"].as_array_mut().unwrap().remove(k);
        app.sel = Sel::None;
        app.fix_current();
        app.changed();
    }
}

fn sound_props(app: &mut App, ui: &mut egui::Ui, id: &str) {
    let k = match app.proj.arr("sounds").iter().position(|s| s.s("id") == id) {
        Some(k) => k,
        None => return,
    };
    let s = app.proj["sounds"][k].clone();
    header(ui, "♪", s.s("name"), crate::i18n::tr("Son importé"));
    let mut e = s.clone();
    egui::Grid::new("snd").num_columns(2).show(ui, |ui| {
        if field(ui, "name", &mut e["name"], crate::i18n::tr("Nom")) {
            app.snapshot();
            app.proj["sounds"][k] = e.clone();
            app.changed();
        }
    });
    ui.label(egui::RichText::new(crate::i18n::trf("{0} Ko", &[format!("{}", s.s("data").len() * 3 / 4 / 1024)])).color(C::DIM()));
    ui.horizontal(|ui| {
        if ui.add(theme::btn_accent(crate::i18n::tr("▶ Écouter"))).clicked() {
            if let Some(b) = pf_core::data_url_bytes(s.s("data")) {
                crate::more::Preview::play_bytes(app, b);
            }
        }
        if s.get("sfx").map(|x| x.is_object()).unwrap_or(false) && ui.add(theme::btn(crate::i18n::tr("✦ Modifier l’effet"))).clicked() {
            app.sfx_ed = Some(crate::more::SfxEd::new(app, Some(id.to_string())));
        }
    });
    if ui.add(theme::btn(crate::i18n::tr("☰ Bibliothèque de sons (rogner, volume…)"))).clicked() {
        app.snd_lib = Some(Default::default());
    }
    if ui.add(theme::btn_small(crate::i18n::tr("Supprimer le son"))).clicked() {
        app.snapshot();
        app.proj["sounds"].as_array_mut().unwrap().remove(k);
        app.sel = Sel::None;
        app.changed();
    }
}

fn settings_props(app: &mut App, ui: &mut egui::Ui) {
    header(ui, "⚙", crate::i18n::tr("Paramètres du jeu"), crate::i18n::tr("Titre, taille de l’écran, transitions, dialogues, commandes"));
    let mut s = app.proj.get("settings").cloned().unwrap_or(json!({}));
    let mut ch = false;
    for (k, d) in [("title", json!(crate::i18n::tr("Mon jeu"))), ("width", json!(854)), ("height", json!(480)), ("pixelArt", json!(true))] {
        if s.get(k).is_none() {
            s[k] = d;
        }
    }
    egui::Grid::new("set").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ch |= field(ui, "title", &mut s["title"], crate::i18n::tr("Titre"));
        ch |= field(ui, "width", &mut s["width"], crate::i18n::tr("Largeur de l’écran"));
        ch |= field(ui, "height", &mut s["height"], crate::i18n::tr("Hauteur de l’écran"));
        ch |= field(ui, "pixelArt", &mut s["pixelArt"], crate::i18n::tr("Pixel art (sans lissage)"));
        ui.label(egui::RichText::new(crate::i18n::tr("Niveau de départ")).color(C::MUTED()));
        let cur = s.s("startLayout").to_string();
        let nm = app.proj.arr("layouts").iter().find(|l| l.s("id") == cur).map(|l| l.s("name").to_string()).unwrap_or_else(|| crate::i18n::tr("(le premier)").into());
        egui::ComboBox::from_id_salt("start").selected_text(nm).show_ui(ui, |ui| {
            for l in app.proj.arr("layouts") {
                if ui.selectable_label(l.s("id") == cur, l.s("name")).clicked() {
                    s["startLayout"] = Json::String(l.s("id").to_string());
                    ch = true;
                }
            }
        });
        ui.end_row();
    });
    theme::section(ui, crate::i18n::tr("Transition entre niveaux"));
    if !s.get("transition").map(|t| t.is_object()).unwrap_or(false) {
        s["transition"] = json!({"type":"fade","dur":0.5,"color":"#000000"});
    }
    egui::Grid::new("tr").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ui.label(egui::RichText::new(crate::i18n::tr("Type")).color(C::MUTED()));
        let cur = s["transition"].s("type").to_string();
        egui::ComboBox::from_id_salt("trt").selected_text(match cur.as_str() { "none" => crate::i18n::tr("aucune"), "iris" => crate::i18n::tr("cercle"), "wipe" => crate::i18n::tr("balayage"), _ => crate::i18n::tr("fondu") }).show_ui(ui, |ui| {
            for (k, n) in [("none", crate::i18n::tr("aucune")), ("fade", crate::i18n::tr("fondu")), ("iris", crate::i18n::tr("cercle")), ("wipe", crate::i18n::tr("balayage"))] {
                if ui.selectable_label(cur == k, n).clicked() {
                    s["transition"]["type"] = json!(k);
                    ch = true;
                }
            }
        });
        ui.end_row();
        ch |= field(ui, "dur", &mut s["transition"]["dur"], crate::i18n::tr("Durée (s)"));
        ch |= field(ui, "color", &mut s["transition"]["color"], crate::i18n::tr("Couleur"));
    });
    theme::section(ui, crate::i18n::tr("Dialogues"));
    if !s.get("dialog").map(|t| t.is_object()).unwrap_or(false) {
        s["dialog"] = json!({"bg":"#1b1f2a","fg":"#ffffff","accent":"#ffd84d","font":"round","blip":true});
    }
    egui::Grid::new("dl").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        for (k, l) in [("bg", crate::i18n::tr("Fond")), ("fg", crate::i18n::tr("Texte")), ("accent", crate::i18n::tr("Accent")), ("font", crate::i18n::tr("Police")), ("blip", crate::i18n::tr("Bip en écrivant"))] {
            if s["dialog"].get(k).is_none() {
                s["dialog"][k] = json!("");
            }
            ch |= field(ui, k, &mut s["dialog"][k], l);
        }
    });
    theme::section(ui, crate::i18n::tr("Commandes du jeu (touches modifiables par le joueur)"));
    let ctl = s.arr("controls").to_vec();
    let mut del = None;
    for (k, c) in ctl.iter().enumerate() {
        ui.horizontal(|ui| {
            // largeurs relatives : la ligne ne doit jamais dépasser le panneau (sinon il s'élargit)
            let avail = (content_right(ui) - ui.cursor().left()).max(120.0);
            let mut n = c.s("name").to_string();
            if ui.add(egui::TextEdit::singleline(&mut n).desired_width((avail * 0.36).max(50.0))).changed() {
                s["controls"][k]["name"] = Json::String(n);
                ch = true;
            }
            // de droite à gauche : d'abord ✕, puis le champ des touches prend la place restante
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    del = Some(k);
                }
                let mut keys = c.arr("keys").iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" ");
                let w = (ui.available_width() - 8.0).max(50.0);
                if ui.add(egui::TextEdit::singleline(&mut keys).desired_width(w)).on_hover_text(crate::i18n::tr("Codes de touches séparés par des espaces (ArrowLeft KeyA…)")).changed() {
                    s["controls"][k]["keys"] = json!(keys.split_whitespace().collect::<Vec<_>>());
                    ch = true;
                }
            });
        });
    }
    if let Some(k) = del {
        s["controls"].as_array_mut().unwrap().remove(k);
        ch = true;
    }
    if ui.add(theme::btn_small(crate::i18n::tr("+ Commande"))).clicked() {
        if !s.get("controls").map(|x| x.is_array()).unwrap_or(false) {
            s["controls"] = json!([]);
        }
        s["controls"].as_array_mut().unwrap().push(json!({"name":crate::i18n::tr("Action"),"keys":["Space"]}));
        ch = true;
    }
    if ch {
        app.snapshot();
        app.proj["settings"] = s;
        app.changed();
    }
}

/// panneau du débogueur pendant le test
pub fn debugger(ui: &mut egui::Ui, p: &mut Play) {
    let rt = &mut p.rt;
    ui.label(egui::RichText::new(crate::i18n::tr("Débogueur")).strong().size(15.0).color(C::TEXT()));
    ui.label(egui::RichText::new(crate::i18n::trf("Niveau « {0} » · image {1} · {2} s · {3} objet(s)", &[format!("{}", rt.layout_name()), format!("{}", rt.tick), format!("{:.1}", rt.time), format!("{}", rt.all.len())])).color(C::MUTED()));
    ui.horizontal(|ui| {
        ui.label(crate::i18n::tr("Vitesse"));
        ui.add(egui::Slider::new(&mut rt.dbg_speed, 0.05..=4.0).logarithmic(true));
    });
    if let Some((id, t)) = &rt.brk_hit {
        ui.colored_label(C::ACCENT(), crate::i18n::trf("Point d’arrêt atteint ({0} à l’image {1})", &[format!("{}", id), format!("{}", t)]));
    }
    if !rt.errors.is_empty() {
        theme::section(ui, crate::i18n::tr("Erreurs"));
        for e in &rt.errors {
            ui.colored_label(C::DANGER(), e);
        }
    }
    theme::section(ui, crate::i18n::tr("Variables globales"));
    egui::Grid::new("dg").num_columns(2).show(ui, |ui| {
        let gl = rt.globals.clone();
        for (k, v) in gl {
            ui.label(egui::RichText::new(&k).color(C::MUTED()));
            let mut s = v.str();
            if ui.add(egui::TextEdit::singleline(&mut s).desired_width(110.0)).changed() {
                let nv = if v.is_str() { pf_core::Val::S(s) } else { pf_core::Val::N(pf_core::Val::S(s).num0()) };
                rt.set_global(&k, nv);
            }
            ui.end_row();
        }
    });
    theme::section(ui, crate::i18n::tr("Objets"));
    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
        for t in 0..rt.types.len() {
            let n = rt.alive_insts(t).len();
            if n == 0 || rt.types[t].fam {
                continue;
            }
            let name = rt.types[t].name.clone();
            egui::CollapsingHeader::new(format!("{} ({})", name, n)).id_salt(("dt", t)).show(ui, |ui| {
                for s in rt.alive_insts(t).into_iter().take(30) {
                    let i = rt.inst(s);
                    let mut line = format!("#{}  x={:.1} y={:.1}", i.uid, i.x, i.y);
                    if let Some(pl) = &i.b.platform {
                        line += &crate::i18n::trf("  vx={0} vy={1}{2}", &[format!("{:.0}", pl.vx), format!("{:.0}", pl.vy), format!("{}", if pl.floor.is_some() { crate::i18n::tr(" au sol") } else { "" })]);
                    }
                    for (k, v) in &i.vars {
                        line += &format!("  {}={}", k, v);
                    }
                    let uid = i.uid;
                    let r = ui.label(egui::RichText::new(line).monospace().size(11.0).color(C::TEXT2()));
                    if r.hovered() {
                        rt.hl = Some(uid);
                    }
                }
            });
        }
    });
}
