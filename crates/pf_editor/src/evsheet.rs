//! Feuille d'événements : affichage et édition (conditions, actions, sous-événements, groupes,
//! fonctions, commentaires, variables locales), avec sélecteur de conditions / actions du catalogue.

use crate::theme::{self, C};
use crate::App;
use eframe::egui::{self, RichText};
use pf_core::catalog;
use pf_core::{json, Json, JsonExt};

#[derive(Default)]
pub struct EvState {
    pub sel: Option<Vec<usize>>,
    /// édition d'une condition / action : (chemin, est_condition, index)
    pub edit: Option<(Vec<usize>, bool, usize)>,
    /// sélecteur : (chemin, est_condition, fournisseur choisi)
    pub pick: Option<(Vec<usize>, bool, Option<String>)>,
    pub pick_filter: String,
    pub collapsed: std::collections::HashSet<String>,
}

const SYS_PROVS: [(&str, &str, &str); 5] = [("system", "Système", "⚙"), ("keyboard", "Clavier", "⌨"), ("mouse", "Souris", "🖱"), ("audio", "Audio", "♪"), ("storage", "Sauvegarde", "💾")];

pub fn providers_of(o: &Json, objs: &[Json]) -> Vec<String> {
    let k = o.s("kind");
    if k == "array" || k == "dict" {
        return vec![k.to_string()];
    }
    if k == "family" {
        let ms: Vec<&Json> = o.arr("members").iter().filter_map(|id| objs.iter().find(|x| Some(x.s("id")) == id.as_str())).collect();
        if ms.is_empty() {
            return vec!["common".into()];
        }
        let mut r = providers_of(ms[0], objs);
        for m in &ms[1..] {
            let q = providers_of(m, objs);
            r.retain(|p| q.contains(p));
        }
        return r;
    }
    let mut l = vec!["common".to_string()];
    if ["text", "sprite", "tilemap", "particles", "button", "leaderboard", "bar", "road3d", "minimap"].contains(&k) {
        l.push(k.to_string());
    }
    for b in o.arr("behaviors") {
        l.push(format!("b:{}", b.s("type")));
    }
    l
}

fn events_at<'a>(sheet: &'a mut Json, path: &[usize]) -> Option<&'a mut Vec<Json>> {
    let mut cur = sheet.get_mut("events")?.as_array_mut()?;
    for &k in path {
        let e = cur.get_mut(k)?;
        if e.get("subs").is_none() {
            e["subs"] = json!([]);
        }
        cur = e.get_mut("subs")?.as_array_mut()?;
    }
    Some(cur)
}
fn ev_at<'a>(sheet: &'a mut Json, path: &[usize]) -> Option<&'a mut Json> {
    let (last, parent) = path.split_last()?;
    events_at(sheet, parent)?.get_mut(*last)
}

fn sheet_mut(app: &mut App) -> Option<&mut Json> {
    let id = app.cur_sheet.clone();
    app.proj.get_mut("sheets")?.as_array_mut()?.iter_mut().find(|s| s.s("id") == id)
}

fn fmt_param(app: &App, t: &str, v: &Json, pd: &Json, k: &str) -> String {
    let s = match v {
        Json::Null => String::new(),
        v => pf_runtime::rt::jstr(v),
    };
    match t {
        "obj" | "objopt" => {
            if s.is_empty() {
                String::new()
            } else {
                app.obj_name(&s)
            }
        }
        "layout" => app.proj.arr("layouts").iter().find(|l| l.s("id") == s).map(|l| l.s("name").to_string()).unwrap_or_else(|| "?".into()),
        "cmp" => pf_core::catalog::catalog()["CMP"].as_array().and_then(|a| a.iter().find(|c| c[0].as_str() == Some(&s))).and_then(|c| c[1].as_str()).unwrap_or(&s).to_string(),
        "key" => catalog::keys().iter().find(|x| x.0 == s).map(|x| x.1.to_string()).unwrap_or(s),
        "choice" => pd.arr("o").iter().find(|o| o[0].as_str() == Some(&s)).and_then(|o| o[1].as_str()).map(|x| x.to_string()).unwrap_or(s),
        "sound" => catalog::catalog()["SOUNDS"].as_array().and_then(|a| a.iter().find(|x| x[0].as_str() == Some(&s))).and_then(|x| x[1].as_str()).unwrap_or(&s).to_string(),
        "sfile" => app.proj.arr("sounds").iter().find(|x| x.s("id") == s).map(|x| x.s("name").to_string()).unwrap_or_else(|| "(son manquant)".into()),
        "bool" => if pf_runtime::rt::jtruthy(Some(v)) { "oui".into() } else { "non".into() },
        "lname" => if s.is_empty() { "premier calque".into() } else { s },
        "gname" => format!("« {} »", s),
        "args" => format!("({})", s),
        "tag" => if s.is_empty() { String::new() } else { format!(", étiquette {}", s) },
        "anim" => format!("\"{}\"", s),
        "cine" => app.proj.arr("cines").iter().find(|c| c.s("id") == s).map(|c| c.s("name").to_string()).unwrap_or_else(|| if s.is_empty() { "toutes".into() } else { "(manquante)".into() }),
        "ipt" | "iptx" => if s.is_empty() { if k == "b:pin.pinto" { "position actuelle".into() } else { "origine".into() } } else { format!("« {} »", s) },
        _ => s,
    }
}

pub fn ace_text(app: &App, it: &Json, is_cond: bool) -> String {
    let k = it.s("k");
    let d = match catalog::ace_def(!is_cond, k) {
        Some(d) => d,
        None => return format!("(inconnu : {})", k),
    };
    let t = d.s("t");
    let mut out = String::new();
    let cs: Vec<char> = t.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '{' && i + 2 < cs.len() && cs[i + 1].is_ascii_digit() && cs[i + 2] == '}' {
            let n = cs[i + 1].to_digit(10).unwrap() as usize;
            if let Some(pd) = d.arr("p").get(n) {
                out += &fmt_param(app, pd.s("t"), it.arr("p").get(n).unwrap_or(&Json::Null), pd, k);
            }
            i += 3;
            continue;
        }
        out.push(cs[i]);
        i += 1;
    }
    out
}

fn owner(app: &App, o: &str) -> (String, egui::Color32) {
    match o {
        "system" => ("Système".into(), C::SYS),
        "keyboard" => ("Clavier".into(), C::BLUE_T),
        "mouse" => ("Souris".into(), C::BLUE_T),
        "audio" => ("Audio".into(), C::GREEN_T),
        "storage" => ("Sauvegarde".into(), C::PURPLE),
        id => (app.obj(id).map(|o| format!("{} {}", crate::tree::glyph(o.s("kind")), o.s("name"))).unwrap_or_else(|| "(supprimé)".into()), C::CODE),
    }
}

/// renomme un identifiant dans toutes les expressions du projet (hors chaînes)
pub fn rename_in_exprs(p: &mut Json, old: &str, new: &str) {
    fn rep(s: &str, old: &str, new: &str) -> String {
        let cs: Vec<char> = s.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        let isw = |c: char| c.is_alphanumeric() || c == '_';
        while i < cs.len() {
            if cs[i] == '"' {
                out.push('"');
                i += 1;
                while i < cs.len() {
                    out.push(cs[i]);
                    if cs[i] == '"' {
                        if i + 1 < cs.len() && cs[i + 1] == '"' {
                            out.push('"');
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            if isw(cs[i]) && (i == 0 || !isw(cs[i - 1])) {
                let st = i;
                while i < cs.len() && isw(cs[i]) {
                    i += 1;
                }
                let w: String = cs[st..i].iter().collect();
                let after_dot = st > 0 && cs[st - 1] == '.';
                if w == old && !after_dot {
                    out += new;
                } else {
                    out += &w;
                }
                continue;
            }
            out.push(cs[i]);
            i += 1;
        }
        out
    }
    fn walk(evs: &mut Vec<Json>, old: &str, new: &str) {
        for e in evs.iter_mut() {
            for key in ["conds", "acts"] {
                if let Some(a) = e.get_mut(key).and_then(|x| x.as_array_mut()) {
                    for it in a.iter_mut() {
                        let k = it.s("k").to_string();
                        let cond = key == "conds";
                        let d = catalog::ace_def(!cond, &k).cloned();
                        if let Some(ps) = it.get_mut("p").and_then(|x| x.as_array_mut()) {
                            for (n, v) in ps.iter_mut().enumerate() {
                                let t = d.as_ref().and_then(|d| d.arr("p").get(n)).map(|pd| pd.s("t").to_string()).unwrap_or_default();
                                if let Json::String(s) = v {
                                    if t == "expr" || t == "args" {
                                        *s = rep(s, old, new);
                                    } else if (t == "gvar" || t == "ivar") && s == old {
                                        *s = new.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(s) = e.get_mut("subs").and_then(|x| x.as_array_mut()) {
                walk(s, old, new);
            }
        }
    }
    if let Some(shs) = p.get_mut("sheets").and_then(|x| x.as_array_mut()) {
        for s in shs {
            if let Some(evs) = s.get_mut("events").and_then(|x| x.as_array_mut()) {
                walk(evs, old, new);
            }
        }
    }
}

fn new_event() -> Json {
    json!({"id":pf_core::uid("e"),"kind":"event","conds":[],"acts":[],"subs":[]})
}

pub fn ui(app: &mut App, ui: &mut egui::Ui, now: f64) {
    let _ = now;
    if !app.proj.arr("sheets").iter().any(|s| s.s("id") == app.cur_sheet) {
        ui.label("Aucune feuille d’événements.");
        return;
    }
    if !ui.ctx().wants_keyboard_input() {
        let (c, v) = ui.ctx().input(|i| (i.modifiers.command && i.key_pressed(egui::Key::C), i.modifiers.command && i.key_pressed(egui::Key::V)));
        if c && app.ev.sel.is_some() {
            copy_sel(app, ui.ctx());
        }
        if v && app.ev_clip.is_some() {
            paste(app);
        }
    }
    // ---- barre d'outils
    egui::Frame::new().fill(C::BG1).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let sel = app.ev.sel.clone();
            let mut add = |app: &mut App, e: Json, sub: bool| {
                app.snapshot();
                let sel = app.ev.sel.clone();
                let sh = sheet_mut(app).unwrap();
                let (parent, at) = match &sel {
                    Some(p) if sub => (p.clone(), None),
                    Some(p) => (p[..p.len() - 1].to_vec(), Some(p[p.len() - 1] + 1)),
                    None => (vec![], None),
                };
                if let Some(list) = events_at(sh, &parent) {
                    let k = at.unwrap_or(list.len()).min(list.len());
                    list.insert(k, e);
                    let mut np = parent.clone();
                    np.push(k);
                    app.ev.sel = Some(np);
                }
                app.changed();
            };
            if ui.add(theme::btn_accent("+ Événement")).clicked() {
                add(app, new_event(), false);
            }
            if ui.add_enabled(sel.is_some(), theme::btn("+ Sous-événement")).clicked() {
                add(app, new_event(), true);
            }
            if ui.add(theme::btn("+ Commentaire")).clicked() {
                add(app, json!({"id":pf_core::uid("e"),"kind":"comment","text":"Commentaire"}), false);
            }
            if ui.add(theme::btn("+ Groupe")).clicked() {
                add(app, json!({"id":pf_core::uid("e"),"kind":"group","name":"Groupe","desc":"","active":true,"subs":[]}), false);
            }
            if ui.add(theme::btn("+ Fonction")).clicked() {
                add(app, json!({"id":pf_core::uid("e"),"kind":"function","name":"maFonction","params":[],"acts":[],"subs":[]}), false);
            }
            if ui.add(theme::btn("+ Variable locale")).clicked() {
                add(app, json!({"id":pf_core::uid("e"),"kind":"lvar","name":"local","type":"number","value":0,"static":false}), false);
            }
            if ui.add(theme::btn("+ Inclure une feuille")).clicked() {
                let other = app.proj.arr("sheets").iter().find(|s| s.s("id") != app.cur_sheet).map(|s| s.s("id").to_string()).unwrap_or_default();
                add(app, json!({"id":pf_core::uid("e"),"kind":"include","sheet":other}), false);
            }
            ui.separator();
            let has = sel.is_some();
            if ui.add_enabled(has, theme::btn_small("↑")).clicked() {
                move_sel(app, -1);
            }
            if ui.add_enabled(has, theme::btn_small("↓")).clicked() {
                move_sel(app, 1);
            }
            if ui.add_enabled(has, theme::btn_small("Activer / désactiver")).clicked() {
                if let Some(p) = sel.clone() {
                    app.snapshot();
                    if let Some(e) = ev_at(sheet_mut(app).unwrap(), &p) {
                        let off = pf_runtime::rt::jtruthy(e.get("off"));
                        e["off"] = Json::Bool(!off);
                    }
                    app.changed();
                }
            }
            if ui.add_enabled(has, theme::btn_small("Dupliquer")).clicked() {
                if let Some(p) = sel.clone() {
                    app.snapshot();
                    let sh = sheet_mut(app).unwrap();
                    if let Some(list) = events_at(sh, &p[..p.len() - 1]) {
                        let mut c = list[p[p.len() - 1]].clone();
                        reid(&mut c);
                        list.insert(p[p.len() - 1] + 1, c);
                    }
                    app.changed();
                }
            }
            if ui.add_enabled(has, theme::btn_small("Copier")).on_hover_text("Ctrl+C").clicked() {
                copy_sel(app, ui.ctx());
            }
            if ui.add_enabled(app.ev_clip.is_some(), theme::btn_small("Coller")).on_hover_text("Ctrl+V — après l’événement sélectionné").clicked() {
                paste(app);
            }
            let bp_on = sel.as_ref().and_then(|p| ev_id(app, p)).map(|id| app.bps.contains(&id)).unwrap_or(false);
            if ui.add_enabled(has, theme::btn_small(if bp_on { "● Retirer le point d’arrêt" } else { "● Point d’arrêt" })).on_hover_text("Le test se met en pause quand cet événement s’exécute").clicked() {
                if let Some(id) = sel.as_ref().and_then(|p| ev_id(app, p)) {
                    if !app.bps.remove(&id) {
                        app.bps.insert(id);
                    }
                }
            }
            if ui.add_enabled(has, theme::btn_small("Supprimer")).clicked() {
                if let Some(p) = sel.clone() {
                    app.snapshot();
                    let sh = sheet_mut(app).unwrap();
                    if let Some(list) = events_at(sh, &p[..p.len() - 1]) {
                        list.remove(p[p.len() - 1]);
                    }
                    app.ev.sel = None;
                    app.changed();
                }
            }
        });
    });
    let events = app.proj.arr("sheets").iter().find(|s| s.s("id") == app.cur_sheet).map(|s| s.arr("events").to_vec()).unwrap_or_default();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(6.0);
        if events.is_empty() {
            ui.label(RichText::new("Feuille vide : ajoutez un événement (« + Événement »), puis ses conditions (à gauche) et ses actions (à droite).").color(C::MUTED));
        }
        draw_list(app, ui, &events, &mut vec![], 0);
        ui.add_space(40.0);
    });
    edit_window(app, ui.ctx());
    pick_window(app, ui.ctx());
}

fn reid(e: &mut Json) {
    e["id"] = Json::String(pf_core::uid("e"));
    for key in ["conds", "acts"] {
        if let Some(a) = e.get_mut(key).and_then(|x| x.as_array_mut()) {
            for it in a {
                it["id"] = Json::String(pf_core::uid(if key == "conds" { "c" } else { "a" }));
            }
        }
    }
    if let Some(s) = e.get_mut("subs").and_then(|x| x.as_array_mut()) {
        for x in s {
            reid(x);
        }
    }
}

fn move_sel(app: &mut App, d: i32) {
    let p = match app.ev.sel.clone() {
        Some(p) => p,
        None => return,
    };
    app.snapshot();
    let sh = sheet_mut(app).unwrap();
    if let Some(list) = events_at(sh, &p[..p.len() - 1]) {
        let k = p[p.len() - 1] as i32;
        let nk = k + d;
        if nk >= 0 && (nk as usize) < list.len() {
            list.swap(k as usize, nk as usize);
            let mut np = p.clone();
            *np.last_mut().unwrap() = nk as usize;
            app.ev.sel = Some(np);
        }
    }
    app.changed();
}

fn draw_list(app: &mut App, ui: &mut egui::Ui, list: &[Json], path: &mut Vec<usize>, depth: usize) {
    for (k, e) in list.iter().enumerate() {
        path.push(k);
        draw_event(app, ui, e, path, depth);
        path.pop();
    }
}

fn row_frame(sel: bool, off: bool, fill: egui::Color32) -> egui::Frame {
    egui::Frame::new().fill(if off { C::BG1 } else { fill }).corner_radius(8.0).inner_margin(egui::Margin::symmetric(8, 6)).stroke(egui::Stroke::new(if sel { 1.5 } else { 1.0 }, if sel { C::ACCENT } else { C::LINE }))
}

fn draw_event(app: &mut App, ui: &mut egui::Ui, e: &Json, path: &Vec<usize>, depth: usize) {
    let sel = app.ev.sel.as_ref() == Some(path);
    let off = pf_runtime::rt::jtruthy(e.get("off"));
    let indent = depth as f32 * 22.0;
    let id = e.s("id").to_string();
    ui.horizontal(|ui| {
        ui.add_space(indent);
        ui.vertical(|ui| {
            match e.s("kind") {
                "comment" => {
                    let r = row_frame(sel, off, egui::Color32::from_rgb(0x2a, 0x26, 0x1a)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let mut t = e.s("text").to_string();
                        if sel {
                            if ui.add(egui::TextEdit::multiline(&mut t).desired_rows(1).desired_width(f32::INFINITY)).changed() {
                                set_field(app, path, "text", Json::String(t));
                            }
                        } else {
                            ui.label(RichText::new(format!("💬 {}", t)).color(egui::Color32::from_rgb(0xff, 0xd8, 0x9a)).italics());
                        }
                    });
                    if r.response.interact(egui::Sense::click()).clicked() {
                        app.ev.sel = Some(path.clone());
                    }
                }
                "group" => {
                    let collapsed = app.ev.collapsed.contains(&id);
                    let r = row_frame(sel, off, egui::Color32::from_rgb(0x1f, 0x22, 0x33)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            if ui.small_button(if collapsed { "▸" } else { "▾" }).clicked() {
                                if collapsed {
                                    app.ev.collapsed.remove(&id);
                                } else {
                                    app.ev.collapsed.insert(id.clone());
                                }
                            }
                            let mut act = e.get("active") != Some(&Json::Bool(false));
                            if ui.checkbox(&mut act, "").on_hover_text("Groupe actif au démarrage").changed() {
                                set_field(app, path, "active", Json::Bool(act));
                            }
                            if sel {
                                let mut n = e.s("name").to_string();
                                if ui.add(egui::TextEdit::singleline(&mut n).desired_width(160.0)).changed() {
                                    set_field(app, path, "name", Json::String(n));
                                }
                                let mut d = e.s("desc").to_string();
                                if ui.add(egui::TextEdit::singleline(&mut d).hint_text("description").desired_width(f32::INFINITY)).changed() {
                                    set_field(app, path, "desc", Json::String(d));
                                }
                            } else {
                                ui.label(RichText::new(format!("📁 {}", e.s("name"))).strong().color(C::PURPLE));
                                ui.label(RichText::new(e.s("desc")).color(C::DIM));
                            }
                        });
                    });
                    if r.response.interact(egui::Sense::click()).clicked() {
                        app.ev.sel = Some(path.clone());
                    }
                    if !collapsed {
                        let subs = e.arr("subs").to_vec();
                        let mut p = path.clone();
                        draw_list(app, ui, &subs, &mut p, 1);
                    }
                }
                "lvar" => {
                    let r = row_frame(sel, off, C::BG2).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Variable locale").color(C::MUTED));
                            let mut n = e.s("name").to_string();
                            if ui.add(egui::TextEdit::singleline(&mut n).desired_width(100.0)).changed() {
                                set_field(app, path, "name", Json::String(n));
                            }
                            ui.label("=");
                            let mut v = pf_runtime::rt::jstr(e.get("value").unwrap_or(&json!(0)));
                            if ui.add(egui::TextEdit::singleline(&mut v).desired_width(80.0)).changed() {
                                let tx = e.s("type") == "text";
                                set_field(app, path, "value", if tx { Json::String(v) } else { json!(pf_core::Val::S(v).num0()) });
                            }
                            let mut tx = e.s("type") == "text";
                            if ui.checkbox(&mut tx, "texte").changed() {
                                set_field(app, path, "type", Json::String(if tx { "text" } else { "number" }.into()));
                            }
                            let mut st = pf_runtime::rt::jtruthy(e.get("static"));
                            if ui.checkbox(&mut st, "statique").on_hover_text("Garde sa valeur d’une image à l’autre").changed() {
                                set_field(app, path, "static", Json::Bool(st));
                            }
                        });
                    });
                    if r.response.interact(egui::Sense::click()).clicked() {
                        app.ev.sel = Some(path.clone());
                    }
                }
                "include" => {
                    let r = row_frame(sel, off, C::BG2).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Inclure la feuille").color(C::MUTED));
                            let cur = e.s("sheet").to_string();
                            let nm = app.proj.arr("sheets").iter().find(|s| s.s("id") == cur).map(|s| s.s("name").to_string()).unwrap_or_else(|| "?".into());
                            egui::ComboBox::from_id_salt(("inc", &id)).selected_text(nm).show_ui(ui, |ui| {
                                for s in app.proj.arr("sheets").to_vec() {
                                    if s.s("id") != app.cur_sheet && ui.selectable_label(s.s("id") == cur, s.s("name")).clicked() {
                                        set_field(app, path, "sheet", Json::String(s.s("id").to_string()));
                                    }
                                }
                            });
                        });
                    });
                    if r.response.interact(egui::Sense::click()).clicked() {
                        app.ev.sel = Some(path.clone());
                    }
                }
                kind => {
                    let is_fn = kind == "function";
                    let r = row_frame(sel, off, C::BG2).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if app.bps.contains(&id) {
                            ui.label(RichText::new("● point d’arrêt").size(11.0).color(C::DANGER));
                        }
                        if is_fn {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("ƒ Fonction").strong().color(C::ACCENT));
                                let mut n = e.s("name").to_string();
                                if ui.add(egui::TextEdit::singleline(&mut n).desired_width(140.0)).changed() {
                                    set_field(app, path, "name", Json::String(n));
                                }
                                ui.label(RichText::new("paramètres :").color(C::MUTED));
                                let mut ps = e.arr("params").iter().map(pf_runtime::rt::jstr).collect::<Vec<_>>().join(", ");
                                if ui.add(egui::TextEdit::singleline(&mut ps).desired_width(160.0)).changed() {
                                    let v: Vec<String> = ps.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                                    set_field(app, path, "params", json!(v));
                                }
                            });
                        }
                        ui.columns(2, |cols| {
                            // conditions
                            if !is_fn {
                                let conds = e.arr("conds").to_vec();
                                let or = pf_runtime::rt::jtruthy(e.get("or"));
                                for (ci, c) in conds.iter().enumerate() {
                                    if ci > 0 && or {
                                        cols[0].label(RichText::new("OU").size(10.0).color(C::ACCENT));
                                    }
                                    ace_row(app, &mut cols[0], c, path, true, ci);
                                }
                                if conds.is_empty() {
                                    cols[0].label(RichText::new("À chaque image (aucune condition)").color(C::DIM).italics());
                                }
                                cols[0].horizontal(|ui| {
                                    if ui.add(egui::Button::new(RichText::new("+ condition").size(12.0).color(C::COND)).frame(false)).clicked() {
                                        app.ev.sel = Some(path.clone());
                                        app.ev.pick = Some((path.clone(), true, None));
                                        app.ev.pick_filter.clear();
                                    }
                                    if conds.len() > 1 {
                                        let mut o2 = or;
                                        if ui.checkbox(&mut o2, RichText::new("bloc OU").size(11.0)).changed() {
                                            set_field(app, path, "or", Json::Bool(o2));
                                        }
                                    }
                                });
                            } else {
                                cols[0].label(RichText::new("Appelée par l’action « Appeler une fonction » ou call(\"nom\", …)").size(11.0).color(C::DIM));
                            }
                            // actions
                            let acts = e.arr("acts").to_vec();
                            for (ai, a) in acts.iter().enumerate() {
                                ace_row(app, &mut cols[1], a, path, false, ai);
                            }
                            if cols[1].add(egui::Button::new(RichText::new("+ action").size(12.0).color(C::ACT)).frame(false)).clicked() {
                                app.ev.sel = Some(path.clone());
                                app.ev.pick = Some((path.clone(), false, None));
                                app.ev.pick_filter.clear();
                            }
                        });
                    });
                    if r.response.interact(egui::Sense::click()).clicked() {
                        app.ev.sel = Some(path.clone());
                    }
                    let subs = e.arr("subs").to_vec();
                    if !subs.is_empty() {
                        let mut p = path.clone();
                        draw_list(app, ui, &subs, &mut p, 1);
                    }
                }
            }
        });
    });
    ui.add_space(3.0);
}

fn ace_row(app: &mut App, ui: &mut egui::Ui, it: &Json, path: &Vec<usize>, cond: bool, idx: usize) {
    let (on, col) = owner(app, it.s("o"));
    let txt = ace_text(app, it, cond);
    let not = cond && pf_runtime::rt::jtruthy(it.get("not"));
    let off = pf_runtime::rt::jtruthy(it.get("off"));
    let r = egui::Frame::new().fill(C::BG3).corner_radius(6.0).inner_margin(egui::Margin::symmetric(6, 3)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(on).size(12.0).color(col));
            if not {
                ui.label(RichText::new("NON").size(11.0).strong().color(C::DANGER));
            }
            let mut rt = RichText::new(txt).color(if off { C::DIM } else { C::TEXT });
            if off {
                rt = rt.strikethrough();
            }
            ui.label(rt);
        });
    });
    let resp = r.response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        app.ev.sel = Some(path.clone());
        app.ev.edit = Some((path.clone(), cond, idx));
    }
    resp.context_menu(|ui| {
        if ui.button("Modifier…").clicked() {
            app.ev.edit = Some((path.clone(), cond, idx));
            ui.close_menu();
        }
        if ui.button(if off { "Activer" } else { "Désactiver" }).clicked() {
            ace_set(app, path, cond, idx, "off", Json::Bool(!off));
            ui.close_menu();
        }
        if cond && ui.button(if not { "Ne plus inverser" } else { "Inverser (NON)" }).clicked() {
            ace_set(app, path, cond, idx, "not", Json::Bool(!not));
            ui.close_menu();
        }
        if ui.button("Supprimer").clicked() {
            app.snapshot();
            if let Some(e) = ev_at(sheet_mut(app).unwrap(), path) {
                if let Some(a) = e.get_mut(if cond { "conds" } else { "acts" }).and_then(|x| x.as_array_mut()) {
                    if idx < a.len() {
                        a.remove(idx);
                    }
                }
            }
            app.changed();
            ui.close_menu();
        }
    });
}

fn set_field(app: &mut App, path: &[usize], k: &str, v: Json) {
    app.snapshot();
    if let Some(e) = ev_at(sheet_mut(app).unwrap(), path) {
        e[k] = v;
    }
    app.changed();
}
fn ace_set(app: &mut App, path: &[usize], cond: bool, idx: usize, k: &str, v: Json) {
    app.snapshot();
    if let Some(e) = ev_at(sheet_mut(app).unwrap(), path) {
        if let Some(it) = e.get_mut(if cond { "conds" } else { "acts" }).and_then(|x| x.get_mut(idx)) {
            it[k] = v;
        }
    }
    app.changed();
}

/// éditeur d'un paramètre de condition / action
fn param_ui(app: &App, ui: &mut egui::Ui, pd: &Json, v: &mut Json, owner_id: &str) -> bool {
    let t = pd.s("t");
    let mut ch = false;
    let cur = match &*v {
        Json::Null => String::new(),
        x => pf_runtime::rt::jstr(x),
    };
    let combo = |ui: &mut egui::Ui, v: &mut Json, items: Vec<(String, String)>, ch: &mut bool| {
        let lab = items.iter().find(|x| x.0 == cur).map(|x| x.1.clone()).unwrap_or_else(|| if cur.is_empty() { "(choisir)".into() } else { cur.clone() });
        egui::ComboBox::from_id_salt(pd as *const _ as usize).selected_text(lab).width(220.0).show_ui(ui, |ui| {
            for (k, n) in items {
                if ui.selectable_label(k == cur, n).clicked() {
                    *v = Json::String(k);
                    *ch = true;
                }
            }
        });
    };
    match t {
        "expr" | "args" => {
            let mut s = cur.clone();
            let r = ui.add(egui::TextEdit::singleline(&mut s).font(egui::TextStyle::Monospace).desired_width(260.0));
            if r.changed() {
                *v = Json::String(s.clone());
                ch = true;
            }
            if t == "expr" {
                if let Err(e) = pf_core::expr::parse(&s) {
                    ui.label(RichText::new(format!("⚠ {}", e)).size(11.0).color(C::DANGER));
                }
            }
        }
        "cmp" => {
            let items = catalog::catalog()["CMP"].as_array().unwrap().iter().map(|c| (c[0].as_str().unwrap().to_string(), c[1].as_str().unwrap().to_string())).collect();
            combo(ui, v, items, &mut ch);
        }
        "key" => {
            let items = catalog::keys().into_iter().map(|(k, n)| (k.to_string(), n.to_string())).collect();
            combo(ui, v, items, &mut ch);
        }
        "obj" | "objopt" => {
            let mut items: Vec<(String, String)> = if t == "objopt" { vec![(String::new(), "(aucun)".into())] } else { vec![] };
            items.extend(app.proj.arr("objects").iter().map(|o| (o.s("id").to_string(), format!("{} {}", crate::tree::glyph(o.s("kind")), o.s("name")))));
            combo(ui, v, items, &mut ch);
        }
        "layout" => {
            let items = app.proj.arr("layouts").iter().map(|l| (l.s("id").to_string(), l.s("name").to_string())).collect();
            combo(ui, v, items, &mut ch);
        }
        "lname" => {
            let mut items = vec![(String::new(), "(premier calque)".to_string())];
            let mut seen = std::collections::HashSet::new();
            for l in app.proj.arr("layouts") {
                for y in l.arr("layers") {
                    if seen.insert(y.s("name").to_string()) {
                        items.push((y.s("name").to_string(), y.s("name").to_string()));
                    }
                }
            }
            combo(ui, v, items, &mut ch);
        }
        "choice" => {
            let items = pd.arr("o").iter().map(|o| (pf_runtime::rt::jstr(&o[0]), o[1].as_str().unwrap_or("").to_string())).collect();
            combo(ui, v, items, &mut ch);
        }
        "sound" => {
            let items = catalog::catalog()["SOUNDS"].as_array().unwrap().iter().map(|s| (s[0].as_str().unwrap().to_string(), s[1].as_str().unwrap().to_string())).collect();
            combo(ui, v, items, &mut ch);
        }
        "sfile" => {
            let items = app.proj.arr("sounds").iter().map(|s| (s.s("id").to_string(), s.s("name").to_string())).collect();
            combo(ui, v, items, &mut ch);
        }
        "gvar" => {
            let items = app.proj.arr("globals").iter().map(|g| (g.s("name").to_string(), g.s("name").to_string())).collect::<Vec<_>>();
            ui.horizontal(|ui| {
                combo(ui, v, items, &mut ch);
                let mut s = cur.clone();
                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(90.0)).on_hover_text("ou une variable locale / un paramètre de fonction").changed() {
                    *v = Json::String(s);
                    ch = true;
                }
            });
        }
        "ivar" => {
            let items = app.obj(owner_id).map(|o| o.arr("vars").iter().map(|x| (x.s("name").to_string(), x.s("name").to_string())).collect()).unwrap_or_default();
            combo(ui, v, items, &mut ch);
        }
        "anim" => {
            let items = app.obj(owner_id).map(|o| o.arr("anims").iter().map(|x| (x.s("name").to_string(), x.s("name").to_string())).collect()).unwrap_or_default();
            combo(ui, v, items, &mut ch);
        }
        "fname" => {
            let mut items = Vec::new();
            fn walk(l: &[Json], out: &mut Vec<(String, String)>) {
                for e in l {
                    if e.s("kind") == "function" {
                        out.push((e.s("name").to_string(), e.s("name").to_string()));
                    }
                    walk(e.arr("subs"), out);
                }
            }
            for s in app.proj.arr("sheets") {
                walk(s.arr("events"), &mut items);
            }
            combo(ui, v, items, &mut ch);
        }
        "gname" => {
            let mut items = Vec::new();
            fn walk(l: &[Json], out: &mut Vec<(String, String)>) {
                for e in l {
                    if e.s("kind") == "group" {
                        out.push((e.s("name").to_string(), e.s("name").to_string()));
                    }
                    walk(e.arr("subs"), out);
                }
            }
            for s in app.proj.arr("sheets") {
                walk(s.arr("events"), &mut items);
            }
            combo(ui, v, items, &mut ch);
        }
        "cine" => {
            let mut items = vec![(String::new(), "(toutes)".to_string())];
            items.extend(app.proj.arr("cines").iter().map(|c| (c.s("id").to_string(), c.s("name").to_string())));
            combo(ui, v, items, &mut ch);
        }
        "bool" => {
            let mut b = pf_runtime::rt::jtruthy(Some(v));
            if ui.checkbox(&mut b, "").changed() {
                *v = Json::Bool(b);
                ch = true;
            }
        }
        "color" => {
            let mut c = theme::color32(if cur.is_empty() { "#ffffff" } else { &cur });
            if ui.color_edit_button_srgba(&mut c).changed() {
                *v = Json::String(format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()));
                ch = true;
            }
        }
        _ => {
            let mut s = cur.clone();
            if ui.add(egui::TextEdit::singleline(&mut s).desired_width(220.0)).changed() {
                *v = Json::String(s);
                ch = true;
            }
        }
    }
    ch
}

fn edit_window(app: &mut App, ctx: &egui::Context) {
    let (path, cond, idx) = match app.ev.edit.clone() {
        Some(x) => x,
        None => return,
    };
    let it = app.proj.arr("sheets").iter().find(|s| s.s("id") == app.cur_sheet).and_then(|s| {
        let mut l = s.arr("events");
        let mut e: Option<&Json> = None;
        for (n, &k) in path.iter().enumerate() {
            e = l.get(k);
            if n + 1 < path.len() {
                l = e?.arr("subs");
            }
        }
        e.and_then(|e| e.arr(if cond { "conds" } else { "acts" }).get(idx)).cloned()
    });
    let it = match it {
        Some(i) => i,
        None => {
            app.ev.edit = None;
            return;
        }
    };
    let d = catalog::ace_def(!cond, it.s("k")).cloned().unwrap_or(Json::Null);
    let mut open = true;
    let mut e2 = it.clone();
    let mut ch = false;
    let mut close = false;
    egui::Window::new(if cond { "Condition" } else { "Action" }).id(egui::Id::new("aceedit")).open(&mut open).collapsible(false).resizable(false).default_width(420.0).max_size(theme::fit(ctx, 420.0, 600.0).1).vscroll(true).constrain(true).show(ctx, |ui| {
        let (on, col) = owner(app, it.s("o"));
        ui.label(RichText::new(on).color(col));
        ui.label(RichText::new(d.s("n")).strong().size(15.0).color(C::TEXT));
        if !d.s("hint").is_empty() {
            ui.label(RichText::new(d.s("hint")).size(11.0).color(C::DIM));
        }
        ui.separator();
        egui::Grid::new("aceps").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
            for (n, pd) in d.arr("p").iter().enumerate() {
                ui.label(RichText::new(pd.s("n")).color(C::MUTED));
                if e2.get("p").map(|p| !p.is_array()).unwrap_or(true) {
                    e2["p"] = json!([]);
                }
                while e2["p"].as_array().unwrap().len() <= n {
                    e2["p"].as_array_mut().unwrap().push(pd.get("d").cloned().unwrap_or(json!("")));
                }
                let owner_id = it.s("o").to_string();
                ch |= param_ui(app, ui, pd, &mut e2["p"][n], &owner_id);
                ui.end_row();
            }
            if cond {
                ui.label(RichText::new("Inverser (NON)").color(C::MUTED));
                let mut b = pf_runtime::rt::jtruthy(e2.get("not"));
                if ui.checkbox(&mut b, "").changed() {
                    e2["not"] = Json::Bool(b);
                    ch = true;
                }
                ui.end_row();
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.add(theme::btn_accent("Terminé")).clicked() {
                close = true;
            }
            if ui.add(theme::btn("Supprimer")).clicked() {
                app.snapshot();
                if let Some(e) = ev_at(sheet_mut(app).unwrap(), &path) {
                    if let Some(a) = e.get_mut(if cond { "conds" } else { "acts" }).and_then(|x| x.as_array_mut()) {
                        if idx < a.len() {
                            a.remove(idx);
                        }
                    }
                }
                app.changed();
                close = true;
            }
        });
    });
    if ch && !close {
        app.snapshot();
        if let Some(e) = ev_at(sheet_mut(app).unwrap(), &path) {
            if let Some(x) = e.get_mut(if cond { "conds" } else { "acts" }).and_then(|x| x.get_mut(idx)) {
                *x = e2;
            }
        }
        app.changed();
    }
    if !open || close {
        app.ev.edit = None;
    }
}

fn pick_window(app: &mut App, ctx: &egui::Context) {
    let (path, cond, prov) = match app.ev.pick.clone() {
        Some(x) => x,
        None => return,
    };
    let mut open = true;
    let mut chosen: Option<(String, String)> = None;
    let mut new_prov: Option<Option<String>> = None;
    egui::Window::new(if cond { "Ajouter une condition" } else { "Ajouter une action" }).id(egui::Id::new("acepick")).open(&mut open).collapsible(false).default_width(520.0).default_height(theme::fit(ctx, 520.0, 520.0).0.y).max_size(theme::fit(ctx, 520.0, 520.0).1).constrain(true).show(ctx, |ui| {
        match &prov {
            None => {
                ui.label(RichText::new("1. Choisissez l’objet concerné").color(C::MUTED));
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for (k, n, g) in SYS_PROVS {
                        if !cond && k == "keyboard" {
                            continue;
                        }
                        let tab = if cond { catalog::conds() } else { catalog::acts() };
                        if !tab.contains_key(k) {
                            continue;
                        }
                        if ui.add(theme::btn(&format!("{}  {}", g, n))).clicked() {
                            new_prov = Some(Some(k.to_string()));
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().max_height((ui.available_height() - 20.0).clamp(120.0, 380.0)).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for o in app.proj.arr("objects") {
                            if ui.add(theme::btn(&format!("{}  {}", crate::tree::glyph(o.s("kind")), o.s("name")))).clicked() {
                                new_prov = Some(Some(o.s("id").to_string()));
                            }
                        }
                    });
                });
            }
            Some(p) => {
                let (on, col) = owner(app, p);
                ui.horizontal(|ui| {
                    if ui.small_button("←").clicked() {
                        new_prov = Some(None);
                    }
                    ui.label(RichText::new(format!("2. {} : choisissez {}", on, if cond { "la condition" } else { "l’action" })).color(col));
                });
                ui.add(egui::TextEdit::singleline(&mut app.ev.pick_filter).hint_text("🔍 Rechercher").desired_width(f32::INFINITY));
                let tab = if cond { catalog::conds() } else { catalog::acts() };
                let provs: Vec<String> = if SYS_PROVS.iter().any(|x| x.0 == p) {
                    vec![p.clone()]
                } else {
                    app.obj(p).map(|o| providers_of(o, app.proj.arr("objects"))).unwrap_or_default()
                };
                let f = app.ev.pick_filter.to_lowercase();
                egui::ScrollArea::vertical().max_height((ui.available_height() - 20.0).clamp(120.0, 420.0)).show(ui, |ui| {
                    let mut cats: Vec<(String, Vec<(String, String, String)>)> = Vec::new();
                    for pv in &provs {
                        if let Some(m) = tab.get(pv).and_then(|m| m.as_object()) {
                            for (k, d) in m {
                                if !f.is_empty() && !d.s("n").to_lowercase().contains(&f) && !d.s("t").to_lowercase().contains(&f) {
                                    continue;
                                }
                                let cat = if d.s("cat").is_empty() {
                                    if let Some(b) = pv.strip_prefix("b:") {
                                        catalog::behaviors().get(b).map(|x| x.s("n").to_string()).unwrap_or_default()
                                    } else {
                                        catalog::provname(pv).to_string()
                                    }
                                } else {
                                    d.s("cat").to_string()
                                };
                                let full = format!("{}.{}", pv, k);
                                match cats.iter_mut().find(|c| c.0 == cat) {
                                    Some(c) => c.1.push((full, d.s("n").to_string(), d.s("hint").to_string())),
                                    None => cats.push((cat, vec![(full, d.s("n").to_string(), d.s("hint").to_string())])),
                                }
                            }
                        }
                    }
                    for (cat, items) in cats {
                        ui.label(RichText::new(cat.to_uppercase()).size(11.0).strong().color(C::DIM));
                        for (k, n, h) in items {
                            let mut r = ui.add(egui::Button::new(RichText::new(&n).color(C::TEXT)).frame(false));
                            if !h.is_empty() {
                                r = r.on_hover_text(h);
                            }
                            if r.clicked() {
                                chosen = Some((p.clone(), k));
                            }
                        }
                        ui.add_space(4.0);
                    }
                });
            }
        }
    });
    if let Some(np) = new_prov {
        app.ev.pick = Some((path.clone(), cond, np));
        app.ev.pick_filter.clear();
    }
    if let Some((o, k)) = chosen {
        let d = catalog::ace_def(!cond, &k).cloned().unwrap_or(Json::Null);
        let ps: Vec<Json> = d.arr("p").iter().map(|pd| match pd.get("d") {
            Some(v) => v.clone(),
            None => {
                if pd.s("t") == "obj" {
                    Json::String(app.proj.arr("objects").first().map(|o| o.s("id").to_string()).unwrap_or_default())
                } else if pd.s("t") == "layout" {
                    Json::String(app.proj.arr("layouts").first().map(|o| o.s("id").to_string()).unwrap_or_default())
                } else {
                    Json::String(String::new())
                }
            }
        }).collect();
        let mut it = json!({"id":pf_core::uid(if cond { "c" } else { "a" }),"o":o,"k":k,"p":ps});
        if cond {
            it["not"] = Json::Bool(false);
        }
        app.snapshot();
        let mut idx = 0;
        if let Some(e) = ev_at(sheet_mut(app).unwrap(), &path) {
            let key = if cond { "conds" } else { "acts" };
            if !e.get(key).map(|x| x.is_array()).unwrap_or(false) {
                e[key] = json!([]);
            }
            let a = e[key].as_array_mut().unwrap();
            a.push(it);
            idx = a.len() - 1;
        }
        app.changed();
        app.ev.pick = None;
        if !d.arr("p").is_empty() {
            app.ev.edit = Some((path, cond, idx));
        }
        return;
    }
    if !open {
        app.ev.pick = None;
    }
}

fn ev_id(app: &App, path: &[usize]) -> Option<String> {
    let s = app.proj.arr("sheets").iter().find(|s| s.s("id") == app.cur_sheet)?;
    let mut l = s.arr("events");
    let mut e: Option<&Json> = None;
    for (n, &k) in path.iter().enumerate() {
        e = l.get(k);
        if n + 1 < path.len() {
            l = e?.arr("subs");
        }
    }
    e.map(|e| e.s("id").to_string())
}

fn copy_sel(app: &mut App, ctx: &egui::Context) {
    let p = match app.ev.sel.clone() {
        Some(p) => p,
        None => return,
    };
    let s = app.proj.arr("sheets").iter().find(|s| s.s("id") == app.cur_sheet).cloned();
    if let Some(mut s) = s {
        if let Some(e) = ev_at(&mut s, &p) {
            let e = e.clone();
            ctx.copy_text(serde_json::to_string_pretty(&e).unwrap_or_default());
            app.ev_clip = Some(e);
        }
    }
}

fn paste(app: &mut App) {
    let mut e = match app.ev_clip.clone() {
        Some(e) => e,
        None => return,
    };
    reid(&mut e);
    app.snapshot();
    let sel = app.ev.sel.clone();
    let sh = sheet_mut(app).unwrap();
    let (parent, at) = match &sel {
        Some(p) => (p[..p.len() - 1].to_vec(), Some(p[p.len() - 1] + 1)),
        None => (vec![], None),
    };
    if let Some(list) = events_at(sh, &parent) {
        let k = at.unwrap_or(list.len()).min(list.len());
        list.insert(k, e);
        let mut np = parent;
        np.push(k);
        app.ev.sel = Some(np);
    }
    app.changed();
}
