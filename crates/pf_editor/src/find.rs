//! Rechercher / remplacer dans tout le projet (événements, objets, variables, textes).

use crate::theme::{self, C};
use crate::{App, Sel, Tab};
use eframe::egui::{self, RichText};
use pf_core::{Json, JsonExt};

#[derive(Default)]
pub struct FindState {
    pub q: String,
    pub r: String,
    pub case: bool,
}

enum Hit {
    Ev { sheet: String, path: Vec<usize>, txt: String },
    Obj { id: String, txt: String },
    Global { name: String },
    Inst { layout: String, id: String, txt: String },
}

fn has(s: &str, q: &str, case: bool) -> bool {
    if case {
        s.contains(q)
    } else {
        s.to_lowercase().contains(&q.to_lowercase())
    }
}
fn repl(s: &str, q: &str, r: &str, case: bool) -> String {
    if case {
        return s.replace(q, r);
    }
    let low = s.to_lowercase();
    let ql = q.to_lowercase();
    let mut out = String::new();
    let mut i = 0;
    while let Some(k) = low[i..].find(&ql) {
        out.push_str(&s[i..i + k]);
        out.push_str(r);
        i += k + ql.len();
    }
    out.push_str(&s[i..]);
    out
}

fn search(app: &App, q: &str, case: bool) -> Vec<Hit> {
    let mut out = Vec::new();
    if q.is_empty() {
        return out;
    }
    fn walk(app: &App, l: &[Json], sheet: &str, path: &mut Vec<usize>, q: &str, case: bool, out: &mut Vec<Hit>) {
        for (k, e) in l.iter().enumerate() {
            path.push(k);
            let mut found: Option<String> = None;
            for key in ["text", "name", "desc"] {
                if has(e.s(key), q, case) {
                    found = Some(e.s(key).to_string());
                }
            }
            for (key, cond) in [("conds", true), ("acts", false)] {
                for it in e.arr(key) {
                    let p_hit = it.arr("p").iter().any(|v| v.as_str().map(|s| has(s, q, case)).unwrap_or(false));
                    let txt = crate::evsheet::ace_text(app, it, cond);
                    if p_hit || has(&txt, q, case) || has(&app.obj_name(it.s("o")), q, case) && app.obj(it.s("o")).is_some() {
                        found = Some(format!("{} {}", app.obj(it.s("o")).map(|o| o.s("name").to_string()).unwrap_or_else(|| it.s("o").to_string()), txt));
                    }
                }
            }
            if let Some(t) = found {
                out.push(Hit::Ev { sheet: sheet.to_string(), path: path.clone(), txt: t });
            }
            walk(app, e.arr("subs"), sheet, path, q, case, out);
            path.pop();
        }
    }
    for s in app.proj.arr("sheets") {
        walk(app, s.arr("events"), s.s("id"), &mut vec![], q, case, &mut out);
    }
    for o in app.proj.arr("objects") {
        if has(o.s("name"), q, case) || has(o.s("text"), q, case) {
            out.push(Hit::Obj { id: o.s("id").to_string(), txt: o.s("name").to_string() });
        }
    }
    for g in app.proj.arr("globals") {
        if has(g.s("name"), q, case) {
            out.push(Hit::Global { name: g.s("name").to_string() });
        }
    }
    for l in app.proj.arr("layouts") {
        for i in l.arr("instances") {
            if let Some(t) = i.get("text").and_then(|v| v.as_str()) {
                if has(t, q, case) {
                    out.push(Hit::Inst { layout: l.s("id").to_string(), id: i.s("id").to_string(), txt: format!("{} · « {} »", l.s("name"), t) });
                }
            }
        }
    }
    out
}

/// remplace dans les paramètres des conditions / actions, commentaires et textes d'instances
fn replace_all(app: &mut App, q: &str, r: &str, case: bool) -> usize {
    let mut n = 0;
    fn walk(l: &mut Vec<Json>, q: &str, r: &str, case: bool, n: &mut usize) {
        for e in l.iter_mut() {
            if e.s("kind") == "comment" && has(e.s("text"), q, case) {
                e["text"] = Json::String(repl(e.s("text"), q, r, case));
                *n += 1;
            }
            for key in ["conds", "acts"] {
                if let Some(a) = e.get_mut(key).and_then(|x| x.as_array_mut()) {
                    for it in a {
                        if let Some(ps) = it.get_mut("p").and_then(|x| x.as_array_mut()) {
                            for v in ps {
                                if let Json::String(s) = v {
                                    if has(s, q, case) {
                                        *s = repl(s, q, r, case);
                                        *n += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(s) = e.get_mut("subs").and_then(|x| x.as_array_mut()) {
                walk(s, q, r, case, n);
            }
        }
    }
    if let Some(shs) = app.proj.get_mut("sheets").and_then(|x| x.as_array_mut()) {
        for s in shs {
            if let Some(evs) = s.get_mut("events").and_then(|x| x.as_array_mut()) {
                walk(evs, q, r, case, &mut n);
            }
        }
    }
    if let Some(ls) = app.proj.get_mut("layouts").and_then(|x| x.as_array_mut()) {
        for l in ls {
            if let Some(a) = l.get_mut("instances").and_then(|x| x.as_array_mut()) {
                for i in a {
                    if let Some(Json::String(t)) = i.get_mut("text") {
                        if has(t, q, case) {
                            *t = repl(t, q, r, case);
                            n += 1;
                        }
                    }
                }
            }
        }
    }
    n
}

pub fn window(app: &mut App, ctx: &egui::Context) {
    let mut st = match app.find.take() {
        Some(s) => s,
        None => return,
    };
    let mut open = true;
    let mut goto: Option<Hit> = None;
    let mut do_rep = false;
    let (dsz, msz) = theme::fit(ctx, 560.0, 560.0);
    egui::Window::new("Rechercher / remplacer").id(egui::Id::new("find")).open(&mut open).default_size(dsz).max_size(msz).constrain(true).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Rechercher").color(C::MUTED));
            let r = ui.add(egui::TextEdit::singleline(&mut st.q).desired_width(240.0).hint_text("nom, variable, texte, expression…"));
            if r.gained_focus() || st.q.is_empty() {
                r.request_focus();
            }
            ui.checkbox(&mut st.case, "Respecter la casse");
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Remplacer par").color(C::MUTED));
            ui.add(egui::TextEdit::singleline(&mut st.r).desired_width(200.0));
            if ui.add_enabled(!st.q.is_empty(), theme::btn("Remplacer tout")).on_hover_text("Dans les conditions, actions, commentaires et textes des instances").clicked() {
                do_rep = true;
            }
        });
        ui.label(RichText::new("Pour renommer un objet ou une variable sans rien casser, modifiez plutôt son nom dans le panneau Propriétés : toutes les expressions sont mises à jour.").size(11.0).color(C::DIM));
        ui.separator();
        let hits = search(app, &st.q, st.case);
        ui.label(RichText::new(format!("{} résultat(s)", hits.len())).color(C::MUTED));
        egui::ScrollArea::vertical().max_height((ui.available_height() - 8.0).max(120.0)).show(ui, |ui| {
            for h in hits {
                let (tag, txt) = match &h {
                    Hit::Ev { sheet, txt, .. } => (format!("☰ {}", app.proj.arr("sheets").iter().find(|s| s.s("id") == sheet).map(|s| s.s("name")).unwrap_or("?")), txt.clone()),
                    Hit::Obj { txt, .. } => ("Objet".to_string(), txt.clone()),
                    Hit::Global { name } => ("Variable".to_string(), name.clone()),
                    Hit::Inst { txt, .. } => ("Instance".to_string(), txt.clone()),
                };
                let r = ui.add(egui::Button::new(RichText::new(format!("{}   {}", tag, txt)).color(C::TEXT2)).frame(false));
                if r.clicked() {
                    goto = Some(h);
                }
            }
        });
    });
    if do_rep {
        app.snapshot();
        let n = replace_all(app, &st.q, &st.r, st.case);
        app.changed();
        app.toast(format!("{} remplacement(s) — Ctrl+Z pour annuler", n), false, 0.0);
    }
    if let Some(h) = goto {
        match h {
            Hit::Ev { sheet, path, .. } => {
                app.cur_sheet = sheet.clone();
                app.tab = Tab::Events;
                app.ev.sel = Some(path);
                app.sel = Sel::Sheet(sheet);
            }
            Hit::Obj { id, .. } => app.sel = Sel::Obj(id),
            Hit::Global { name } => app.sel = Sel::Global(name),
            Hit::Inst { layout, id, .. } => {
                app.cur_layout = layout;
                app.tab = Tab::Layout;
                app.sel = Sel::Inst(id);
                app.view.fit_pending = true;
            }
        }
    }
    if open {
        app.find = Some(st);
    }
}
