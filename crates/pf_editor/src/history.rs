//! Historique visuel des modifications (Annuler / Rétablir illustrés)
use eframe::egui;
use crate::theme::{self, C};
use crate::App;
use serde_json::Value as Json;

#[derive(Default)]
pub struct HistCache {
    key: (usize, u64),
    descs: Vec<String>,
}

pub fn now() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

fn ago(t: f64) -> String {
    let s = (now() - t).max(0.0) as u64;
    if s < 5 {
        "à l’instant".into()
    } else if s < 60 {
        format!("il y a {s} s")
    } else if s < 3600 {
        format!("il y a {} min", s / 60)
    } else {
        format!("il y a {} h", s / 3600)
    }
}

fn name_of(v: &Json) -> String {
    v.get("name").and_then(|n| n.as_str()).or_else(|| v.get("id").and_then(|n| n.as_str())).unwrap_or("?").to_string()
}

fn key_of(v: &Json) -> String {
    v.get("id").and_then(|n| n.as_str()).or_else(|| v.get("name").and_then(|n| n.as_str())).unwrap_or("").to_string()
}

fn label(k: &str) -> (&'static str, &'static str) {
    match k {
        "objects" => ("Objet", "Objets"),
        "layouts" => ("Niveau", "Niveaux"),
        "sheets" => ("Feuille", "Feuilles"),
        "sounds" => ("Son", "Sons"),
        "cines" => ("Cinématique", "Cinématiques"),
        "globals" => ("Variable globale", "Variables globales"),
        _ => ("Élément", "Éléments"),
    }
}

fn ids_in(list: Option<&Json>, out: &mut Vec<String>) {
    for e in list.and_then(|e| e.as_array()).map(|a| a.as_slice()).unwrap_or(&[]) {
        out.push(e.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string());
        ids_in(e.get("subs"), out);
    }
}

fn event_ids(sheet: &Json) -> Vec<String> {
    let mut v = Vec::new();
    ids_in(sheet.get("events"), &mut v);
    v
}

fn diff_item(k: &str, a: &Json, b: &Json) -> String {
    let n = name_of(b);
    match k {
        "layouts" => {
            let ia = a.get("instances").and_then(|x| x.as_array()).map(|x| x.len()).unwrap_or(0);
            let ib = b.get("instances").and_then(|x| x.as_array()).map(|x| x.len()).unwrap_or(0);
            if ib > ia {
                format!("Niveau « {n} » : {} instance(s) ajoutée(s)", ib - ia)
            } else if ib < ia {
                format!("Niveau « {n} » : {} instance(s) supprimée(s)", ia - ib)
            } else if a.get("instances") != b.get("instances") {
                format!("Niveau « {n} » : instance déplacée ou modifiée")
            } else if a.get("layers") != b.get("layers") {
                format!("Niveau « {n} » : calques / tuiles modifiés")
            } else {
                format!("Niveau « {n} » : propriétés modifiées")
            }
        }
        "sheets" => {
            let ia = event_ids(a);
            let ib = event_ids(b);
            let (ea, eb) = (ia.len(), ib.len());
            let mut sa = ia.clone();
            let mut sb = ib.clone();
            sa.sort();
            sb.sort();
            if sa == sb && ia != ib {
                format!("Feuille « {n} » : événement déplacé")
            } else if eb > ea {
                format!("Feuille « {n} » : {} événement(s) ajouté(s)", eb - ea)
            } else if eb < ea {
                format!("Feuille « {n} » : {} événement(s) supprimé(s)", ea - eb)
            } else {
                format!("Feuille « {n} » : événements modifiés")
            }
        }
        "objects" => {
            if a.get("anims") != b.get("anims") {
                format!("Objet « {n} » : sprite / animations modifiés")
            } else if a.get("name") != b.get("name") {
                format!("Objet « {} » renommé en « {n} »", name_of(a))
            } else {
                format!("Objet « {n} » : propriétés modifiées")
            }
        }
        _ => format!("{} « {n} » modifié(e)", label(k).0),
    }
}

/// décrit ce qui a changé entre deux états du projet
pub fn describe(a: &Json, b: &Json) -> String {
    let (Some(oa), Some(ob)) = (a.as_object(), b.as_object()) else { return "Projet modifié".into() };
    let mut out: Vec<String> = Vec::new();
    for k in ["objects", "layouts", "sheets", "sounds", "cines", "globals"] {
        let la = oa.get(k).and_then(|x| x.as_array()).cloned().unwrap_or_default();
        let lb = ob.get(k).and_then(|x| x.as_array()).cloned().unwrap_or_default();
        if la == lb {
            continue;
        }
        let (one, _) = label(k);
        let added: Vec<&Json> = lb.iter().filter(|x| !la.iter().any(|y| key_of(y) == key_of(x))).collect();
        let removed: Vec<&Json> = la.iter().filter(|x| !lb.iter().any(|y| key_of(y) == key_of(x))).collect();
        for x in &added {
            out.push(format!("{one} « {} » ajouté(e)", name_of(x)));
        }
        for x in &removed {
            out.push(format!("{one} « {} » supprimé(e)", name_of(x)));
        }
        for x in &lb {
            if let Some(y) = la.iter().find(|y| key_of(y) == key_of(x)) {
                if y != x {
                    out.push(diff_item(k, y, x));
                }
            }
        }
        if added.is_empty() && removed.is_empty() && out.is_empty() {
            out.push(format!("{} : ordre modifié", label(k).1));
        }
    }
    if oa.get("settings") != ob.get("settings") {
        out.push("Paramètres du projet modifiés".into());
    }
    if oa.get("name") != ob.get("name") {
        out.push("Projet renommé".into());
    }
    if out.is_empty() {
        return "Modification mineure".into();
    }
    if out.len() > 2 {
        let n = out.len() - 2;
        out.truncate(2);
        out.push(format!("+{n} autre(s)"));
    }
    out.join(" · ")
}

fn rebuild(app: &App) -> Vec<String> {
    let states: Vec<Json> = app.undo.iter().map(|s| serde_json::from_str(s).unwrap_or(Json::Null)).collect();
    (0..states.len()).map(|i| describe(&states[i], if i + 1 < states.len() { &states[i + 1] } else { &app.proj })).collect()
}

pub fn window(app: &mut App, ctx: &egui::Context) {
    if !app.show_hist {
        return;
    }
    let key = (app.undo.len(), app.ver);
    if app.hist.key != key {
        app.hist.descs = rebuild(app);
        app.hist.key = key;
    }
    let (dsz, msz) = theme::fit(ctx, 380.0, 460.0);
    let mut open = true;
    let mut target: Option<usize> = None;
    let mut redo_n = 0usize;
    egui::Window::new("Historique").open(&mut open).default_size(dsz).max_size(msz).constrain(true).show(ctx, |ui| {
        ui.label(egui::RichText::new("Cliquez sur une étape pour revenir à l’état d’avant. Rien n’est perdu : Rétablir (Ctrl+Y) refait les étapes.").color(C::DIM()).small());
        ui.separator();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // étapes annulées (rétablissables), de la plus récente à la plus ancienne
            let nr = app.redo.len();
            for j in 0..nr {
                let txt = egui::RichText::new(format!("↷  étape annulée n°{}", nr - j)).color(C::DIM()).italics();
                if ui.add(egui::SelectableLabel::new(false, txt)).on_hover_text("Rétablir jusqu’ici").clicked() {
                    redo_n = nr - j;
                }
            }
            ui.add(egui::SelectableLabel::new(true, egui::RichText::new("●  État actuel").strong().color(C::ACCENT())));
            let n = app.hist.descs.len();
            for i in (0..n).rev() {
                let t = app.undo_t.get(i).copied().unwrap_or(0.0);
                let r = ui.add(egui::SelectableLabel::new(false, format!("↶  {}", app.hist.descs[i])));
                let r = if t > 0.0 { r.on_hover_text(format!("{} — cliquer pour annuler jusqu’à avant cette étape", ago(t))) } else { r };
                if r.clicked() {
                    target = Some(i);
                }
                if t > 0.0 {
                    ui.label(egui::RichText::new(format!("      {}", ago(t))).color(C::DIM()).small());
                }
            }
            if n == 0 && nr == 0 {
                ui.label(egui::RichText::new("Aucune modification pour l’instant.").color(C::DIM()));
            }
        });
    });
    if let Some(i) = target {
        let k = app.undo.len() - i;
        for _ in 0..k {
            app.do_undo();
        }
    }
    for _ in 0..redo_n {
        app.do_redo();
    }
    if !open {
        app.show_hist = false;
    }
}
