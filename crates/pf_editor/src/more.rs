//! Éditeur de cinématiques, générateur d'effets sonores et éditeur visuel de piste (Route 3D).

use crate::theme::{self, C};
use crate::App;
use eframe::egui::{self, RichText};
use pf_core::{json, Json, JsonExt};

// =====================================================================================
// lecture d'un son dans l'éditeur

pub struct Preview {
    _s: rodio::OutputStream,
    h: rodio::OutputStreamHandle,
    sink: Option<rodio::Sink>,
}
impl Preview {
    pub fn play(app: &mut App, samples: Vec<f32>, rate: u32) {
        if app.preview.is_none() {
            if let Ok((s, h)) = rodio::OutputStream::try_default() {
                app.preview = Some(Preview { _s: s, h, sink: None });
            }
        }
        if let Some(p) = app.preview.as_mut() {
            if let Some(old) = p.sink.take() {
                old.stop();
            }
            if let Ok(sk) = rodio::Sink::try_new(&p.h) {
                sk.append(rodio::buffer::SamplesBuffer::new(1, rate, samples));
                p.sink = Some(sk);
            }
        }
    }
    pub fn play_bytes(app: &mut App, b: Vec<u8>) {
        if app.preview.is_none() {
            if let Ok((s, h)) = rodio::OutputStream::try_default() {
                app.preview = Some(Preview { _s: s, h, sink: None });
            }
        }
        if let Some(p) = app.preview.as_mut() {
            if let Some(old) = p.sink.take() {
                old.stop();
            }
            if let (Ok(sk), Ok(d)) = (rodio::Sink::try_new(&p.h), rodio::Decoder::new(std::io::Cursor::new(b))) {
                sk.append(d);
                p.sink = Some(sk);
            }
        }
    }
}

// =====================================================================================
// cinématiques

const EASES: &[(&str, &str)] = &[("inout", "Douce"), ("linear", "Linéaire"), ("in", "Accélère"), ("out", "Ralentit")];

/// (clé, libellé, type, défaut)
type F = (&'static str, &'static str, &'static str, &'static str);
fn csteps() -> Vec<(&'static str, &'static str, &'static str, Vec<F>)> {
    vec![
        ("say", "Dialogue", "💬", vec![("who", "Nom", "text", "Pixel"), ("text", "Réplique", "area", "Bonjour !"), ("pos", "Position", "pos", "bottom"), ("o", "Portrait (facultatif)", "objopt", "")]),
        ("wait", "Attendre", "⏱", vec![("d", "Durée (s)", "num", "1")]),
        ("camto", "Caméra vers un point", "🎥", vec![("x", "X", "expr", "LayoutWidth / 2"), ("y", "Y", "expr", "LayoutHeight / 2"), ("d", "Durée (s)", "num", "1.5"), ("zoom", "Zoom (%, vide = inchangé)", "numopt", ""), ("ease", "Mouvement", "ease", "inout")]),
        ("camobj", "Caméra sur un objet", "🎯", vec![("o", "Objet", "obj", ""), ("d", "Durée (s)", "num", "1"), ("zoom", "Zoom (%, vide = inchangé)", "numopt", ""), ("ease", "Mouvement", "ease", "inout")]),
        ("camback", "Caméra : retour au joueur", "↩", vec![("d", "Durée (s)", "num", "1"), ("ease", "Mouvement", "ease", "inout")]),
        ("move", "Déplacer un objet", "➜", vec![("o", "Objet", "obj", ""), ("x", "X", "expr", "100"), ("y", "Y", "expr", "0"), ("rel", "Relatif (décalage depuis sa position)", "bool", "true"), ("d", "Durée (s)", "num", "1"), ("ease", "Mouvement", "ease", "inout"), ("face", "Se tourner dans le sens du trajet", "bool", "true")]),
        ("anim", "Changer d’animation", "🎞", vec![("o", "Objet", "obj", ""), ("a", "Animation", "anim", "")]),
        ("title", "Titre à l’écran", "T", vec![("text", "Texte", "text", "Chapitre 1"), ("d", "Durée (s)", "num", "2.5")]),
        ("fade", "Fondu", "◐", vec![("mode", "Sens", "fmode", "out"), ("d", "Durée (s)", "num", "0.6"), ("color", "Couleur", "color", "#000000")]),
        ("sound", "Son", "♪", vec![("s", "Son", "sound", "powerup")]),
        ("vis", "Afficher / masquer", "👁", vec![("o", "Objet", "obj", ""), ("v", "Visible", "bool", "true")]),
        ("shake", "Secousse", "≋", vec![("a", "Intensité (px)", "num", "6"), ("d", "Durée (s)", "num", "0.4")]),
        ("call", "Appeler une fonction", "ƒ", vec![("f", "Fonction", "fn", "")]),
    ]
}
fn step_new(t: &str) -> Json {
    let mut s = json!({"t": t, "w": false});
    if let Some(d) = csteps().into_iter().find(|x| x.0 == t) {
        for f in d.3 {
            s[f.0] = match f.2 {
                "num" => json!(f.3.parse::<f64>().unwrap_or(0.0)),
                "bool" => Json::Bool(f.3 == "true"),
                _ => Json::String(f.3.to_string()),
            };
        }
    }
    s
}
fn summary(app: &App, s: &Json) -> String {
    let on = |k: &str| app.obj(s.s(k)).map(|o| o.s("name").to_string()).unwrap_or_else(|| "?".into());
    let n = |k: &str| pf_runtime::rt::jstr(s.get(k).unwrap_or(&Json::Null));
    match s.s("t") {
        "say" => format!("{}« {} »", if s.s("who").is_empty() { String::new() } else { format!("{} : ", s.s("who")) }, s.s("text").chars().take(40).collect::<String>()),
        "wait" => format!("{} s", n("d")),
        "camto" => format!("({}, {}) en {} s", n("x"), n("y"), n("d")),
        "camobj" => format!("{} en {} s", on("o"), n("d")),
        "camback" => format!("{} s", n("d")),
        "move" => format!("{}{}({}, {}) en {} s", on("o"), if pf_runtime::rt::jtruthy(s.get("rel")) { " de " } else { " vers " }, n("x"), n("y"), n("d")),
        "anim" => format!("{} → {}", on("o"), n("a")),
        "title" => format!("« {} »", s.s("text")),
        "fade" => format!("{} le noir, {} s", if s.s("mode") == "in" { "depuis" } else { "vers" }, n("d")),
        "sound" => n("s"),
        "vis" => format!("{}{}", on("o"), if s.get("v") != Some(&Json::Bool(false)) { " visible" } else { " masqué" }),
        "shake" => format!("{} px, {} s", n("a"), n("d")),
        "call" => n("f"),
        t => t.to_string(),
    }
}

pub fn cine_window(app: &mut App, ctx: &egui::Context, now: f64) {
    let id = match app.cine_ed.clone() {
        Some(i) => i,
        None => return,
    };
    let k = match app.proj.arr("cines").iter().position(|c| c.s("id") == id) {
        Some(k) => k,
        None => {
            app.cine_ed = None;
            return;
        }
    };
    let c = app.proj["cines"][k].clone();
    let mut e = c.clone();
    let mut ch = false;
    let mut open = true;
    let mut test = false;
    let mut delete = false;
    let defs = csteps();
    egui::Window::new(format!("Cinématique — {}", c.s("name"))).id(egui::Id::new("cineed")).open(&mut open).default_size([620.0, 600.0]).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Nom").color(C::MUTED));
            let mut n = e.s("name").to_string();
            if ui.add(egui::TextEdit::singleline(&mut n).desired_width(180.0)).changed() {
                e["name"] = Json::String(n);
                ch = true;
            }
            if ui.add(theme::btn_accent("▶ Tester")).on_hover_text("Lance le niveau affiché et joue la cinématique").clicked() {
                test = true;
            }
            if ui.add(theme::btn_small("Supprimer")).clicked() {
                delete = true;
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (key, lab) in [("bars", "Bandes noires"), ("lock", "Bloquer les contrôles"), ("freeze", "Figer le jeu"), ("skip", "Échap pour passer")] {
                let mut b = e.get(key) != Some(&Json::Bool(false));
                if ui.checkbox(&mut b, lab).changed() {
                    e[key] = Json::Bool(b);
                    ch = true;
                }
            }
        });
        ui.separator();
        let steps = e.arr("steps").to_vec();
        let mut act: Option<(usize, i32)> = None;
        egui::ScrollArea::vertical().max_height(440.0).show(ui, |ui| {
            for (si, s) in steps.iter().enumerate() {
                let d = defs.iter().find(|x| x.0 == s.s("t"));
                let together = pf_runtime::rt::jtruthy(s.get("w"));
                egui::Frame::new().fill(C::BG2).corner_radius(8.0).inner_margin(8.0).stroke(egui::Stroke::new(1.0, if together { C::ACCENT_SOFT } else { C::LINE })).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{} {}", d.map(|x| x.2).unwrap_or("?"), d.map(|x| x.1).unwrap_or("?"))).strong().color(C::TEXT));
                        ui.label(RichText::new(summary(app, s)).color(C::MUTED));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                act = Some((si, 0));
                            }
                            if ui.small_button("↓").clicked() {
                                act = Some((si, 1));
                            }
                            if ui.small_button("↑").clicked() {
                                act = Some((si, -1));
                            }
                            let mut w = together;
                            if si > 0 && ui.checkbox(&mut w, "en même temps").on_hover_text("Se déroule en même temps que l’étape précédente").changed() {
                                e["steps"][si]["w"] = Json::Bool(w);
                                ch = true;
                            }
                        });
                    });
                    egui::CollapsingHeader::new("Réglages").id_salt(("cs", si)).show(ui, |ui| {
                        egui::Grid::new(("csg", si)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                            for f in d.map(|x| x.3.clone()).unwrap_or_default() {
                                ui.label(RichText::new(f.1).color(C::MUTED));
                                let v = &mut e["steps"][si][f.0];
                                ch |= field(app, ui, f.2, v, s, (si, f.0));
                                ui.end_row();
                            }
                        });
                    });
                });
            }
        });
        if let Some((si, d)) = act {
            let a = e["steps"].as_array_mut().unwrap();
            if d == 0 {
                a.remove(si);
            } else {
                let nk = si as i32 + d;
                if nk >= 0 && (nk as usize) < a.len() {
                    a.swap(si, nk as usize);
                }
            }
            ch = true;
        }
        ui.menu_button("+ Ajouter une étape", |ui| {
            for d in &defs {
                if ui.button(format!("{}  {}", d.2, d.1)).clicked() {
                    if !e.get("steps").map(|x| x.is_array()).unwrap_or(false) {
                        e["steps"] = json!([]);
                    }
                    e["steps"].as_array_mut().unwrap().push(step_new(d.0));
                    ch = true;
                    ui.close_menu();
                }
            }
        });
        ui.label(RichText::new("Jouez-la depuis les événements avec l’action Système « Jouer une cinématique ».").size(11.0).color(C::DIM));
    });
    if ch {
        app.snapshot();
        app.proj["cines"][k] = e;
        app.changed();
    }
    if delete {
        app.snapshot();
        app.proj["cines"].as_array_mut().unwrap().remove(k);
        app.changed();
        app.cine_ed = None;
        app.toast("Cinématique supprimée (Ctrl+Z pour annuler).", false, now);
        return;
    }
    if test {
        let l = app.cur_layout.clone();
        app.start_play(Some(l));
        if let Some(p) = app.play.as_mut() {
            p.rt.cine_play(&id);
        }
    }
    if !open {
        app.cine_ed = None;
    }
}

fn field(app: &App, ui: &mut egui::Ui, t: &str, v: &mut Json, step: &Json, salt: (usize, &str)) -> bool {
    let mut ch = false;
    let cur = match &*v {
        Json::Null => String::new(),
        x => pf_runtime::rt::jstr(x),
    };
    let mut combo = |ui: &mut egui::Ui, v: &mut Json, items: Vec<(String, String)>| {
        let lab = items.iter().find(|x| x.0 == cur).map(|x| x.1.clone()).unwrap_or_else(|| if cur.is_empty() { "(choisir)".into() } else { cur.clone() });
        egui::ComboBox::from_id_salt(salt).selected_text(lab).show_ui(ui, |ui| {
            for (k, n) in items {
                if ui.selectable_label(k == cur, n).clicked() {
                    *v = Json::String(k);
                    ch = true;
                }
            }
        });
    };
    match t {
        "num" => {
            let mut x = pf_core::Val::from_json(v).num0();
            if ui.add(egui::DragValue::new(&mut x).speed(0.05)).changed() {
                *v = json!(x);
                ch = true;
            }
        }
        "bool" => {
            let mut b = pf_runtime::rt::jtruthy(Some(v));
            if ui.checkbox(&mut b, "").changed() {
                *v = Json::Bool(b);
                ch = true;
            }
        }
        "color" => {
            let mut c = theme::color32(if cur.is_empty() { "#000000" } else { &cur });
            if ui.color_edit_button_srgba(&mut c).changed() {
                *v = Json::String(format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()));
                ch = true;
            }
        }
        "area" => {
            let mut s = cur.clone();
            if ui.add(egui::TextEdit::multiline(&mut s).desired_rows(2).desired_width(300.0)).changed() {
                *v = Json::String(s);
                ch = true;
            }
        }
        "pos" => combo(ui, v, vec![("bottom".into(), "En bas".into()), ("top".into(), "En haut".into())]),
        "fmode" => combo(ui, v, vec![("out".into(), "Vers le noir".into()), ("in".into(), "Depuis le noir".into())]),
        "ease" => combo(ui, v, EASES.iter().map(|e| (e.0.to_string(), e.1.to_string())).collect()),
        "obj" | "objopt" => {
            let mut items = if t == "objopt" { vec![(String::new(), "(aucun)".to_string())] } else { vec![] };
            items.extend(app.proj.arr("objects").iter().filter(|o| !["array", "dict", "family"].contains(&o.s("kind"))).map(|o| (o.s("id").to_string(), o.s("name").to_string())));
            combo(ui, v, items);
        }
        "anim" => {
            let items = app.obj(step.s("o")).map(|o| o.arr("anims").iter().map(|a| (a.s("name").to_string(), a.s("name").to_string())).collect()).unwrap_or_default();
            combo(ui, v, items);
        }
        "sound" => {
            let mut items: Vec<(String, String)> = pf_core::catalog::catalog()["SOUNDS"].as_array().unwrap().iter().map(|s| (s[0].as_str().unwrap().to_string(), s[1].as_str().unwrap().to_string())).collect();
            items.extend(app.proj.arr("sounds").iter().map(|s| (format!("file:{}", s.s("id")), format!("♪ {}", s.s("name")))));
            combo(ui, v, items);
        }
        "fn" => {
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
            combo(ui, v, items);
        }
        _ => {
            let mut s = cur.clone();
            let mono = t == "expr";
            let mut te = egui::TextEdit::singleline(&mut s).desired_width(220.0);
            if mono {
                te = te.font(egui::TextStyle::Monospace);
            }
            if ui.add(te).changed() {
                *v = if t == "numopt" && !s.trim().is_empty() { json!(pf_core::Val::S(s.clone()).num0()) } else { Json::String(s.clone()) };
                ch = true;
            }
            if mono {
                if let Err(er) = pf_core::expr::parse(&s) {
                    ui.label(RichText::new(format!("⚠ {}", er)).size(11.0).color(C::DANGER));
                }
            }
        }
    }
    ch
}

pub fn new_cine(app: &mut App) {
    app.snapshot();
    if !app.proj.get("cines").map(|c| c.is_array()).unwrap_or(false) {
        app.proj["cines"] = json!([]);
    }
    let names: Vec<String> = app.proj.arr("cines").iter().map(|c| c.s("name").to_string()).collect();
    let mut nm = "Cinematique".to_string();
    let mut k = 2;
    while names.contains(&nm) {
        nm = format!("Cinematique{}", k);
        k += 1;
    }
    let id = pf_core::uid("ci");
    let c = json!({"id":id,"name":nm,"bars":true,"lock":true,"skip":true,"freeze":true,"steps":[step_new("title"), step_new("wait")]});
    app.proj["cines"].as_array_mut().unwrap().push(c);
    app.cine_ed = Some(id);
    app.changed();
}

// =====================================================================================
// générateur d'effets sonores (même synthèse que la version HTML)

const SFXW: &[(&str, &str)] = &[("square", "Carré"), ("saw", "Dent de scie"), ("triangle", "Triangle"), ("sine", "Sinus"), ("noise", "Bruit")];
const SFXP: &[(&str, &str, &str, f64, f64, f64)] = &[
    ("freq", "Fréquence de départ", "Hz", 40.0, 2400.0, 1.0),
    ("freqEnd", "Fréquence d’arrivée", "Hz", 20.0, 2400.0, 1.0),
    ("attack", "Attaque", "s", 0.0, 1.0, 0.005),
    ("sustain", "Maintien", "s", 0.0, 1.5, 0.005),
    ("decay", "Déclin", "s", 0.01, 2.0, 0.005),
    ("punch", "Punch", "%", 0.0, 100.0, 1.0),
    ("vibDepth", "Vibrato (profondeur)", "½ tons", 0.0, 6.0, 0.1),
    ("vibSpeed", "Vibrato (vitesse)", "Hz", 0.0, 30.0, 0.5),
    ("arp", "Saut de hauteur", "½ tons", -24.0, 24.0, 1.0),
    ("arpT", "Moment du saut", "s", 0.0, 1.5, 0.01),
    ("duty", "Largeur d’onde (carré)", "%", 5.0, 50.0, 1.0),
    ("cutoff", "Filtre passe-bas", "Hz", 200.0, 20000.0, 50.0),
    ("volume", "Volume", "%", 0.0, 100.0, 1.0),
];

fn sfx_defaults() -> Json {
    json!({"wave":"square","freq":440,"freqEnd":440,"attack":0,"sustain":0.08,"decay":0.2,"punch":0,"vibDepth":0,"vibSpeed":0,"arp":0,"arpT":0,"duty":50,"cutoff":20000,"volume":60})
}

struct Rng(u64);
impl Rng {
    fn f(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn r(&mut self, a: f64, b: f64) -> f64 {
        a + self.f() * (b - a)
    }
    fn pick<'a>(&mut self, l: &[&'a str]) -> &'a str {
        l[((self.f() * l.len() as f64) as usize).min(l.len() - 1)]
    }
}
fn seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(9) | 1
}

const PRESETS: &[(&str, &str)] = &[("coin", "Pièce"), ("jump", "Saut"), ("laser", "Laser"), ("explosion", "Explosion"), ("powerup", "Bonus"), ("hurt", "Choc"), ("blip", "Clic"), ("step", "Pas"), ("random", "Hasard")];

fn sfx_preset(k: &str) -> Json {
    let mut r = Rng(seed());
    let mut p = sfx_defaults();
    let set = |p: &mut Json, kv: Vec<(&str, Json)>| {
        for (k, v) in kv {
            p[k] = v;
        }
    };
    match k {
        "coin" => {
            let f = r.r(700.0, 1300.0);
            let w = r.pick(&["square", "triangle", "sine"]);
            let arp = [5.0, 7.0, 12.0][((r.f() * 3.0) as usize).min(2)];
            set(&mut p, vec![("wave", json!(w)), ("freq", json!(f)), ("freqEnd", json!(f)), ("sustain", json!(r.r(0.02, 0.08))), ("decay", json!(r.r(0.12, 0.3))), ("punch", json!(r.r(20.0, 60.0))), ("arp", json!(arp)), ("arpT", json!(r.r(0.04, 0.09))), ("duty", json!(r.r(30.0, 50.0)))]);
        }
        "jump" => {
            let f = r.r(250.0, 520.0);
            let w = r.pick(&["square", "triangle"]);
            set(&mut p, vec![("wave", json!(w)), ("freq", json!(f)), ("freqEnd", json!(f * r.r(1.8, 3.0))), ("sustain", json!(r.r(0.04, 0.1))), ("decay", json!(r.r(0.08, 0.2))), ("duty", json!(r.r(20.0, 50.0)))]);
        }
        "laser" => {
            let f = r.r(800.0, 1800.0);
            let w = r.pick(&["square", "saw", "sine"]);
            set(&mut p, vec![("wave", json!(w)), ("freq", json!(f)), ("freqEnd", json!(f * r.r(0.15, 0.4))), ("sustain", json!(r.r(0.03, 0.12))), ("decay", json!(r.r(0.06, 0.2))), ("duty", json!(r.r(10.0, 50.0))), ("punch", json!(r.r(0.0, 40.0)))]);
        }
        "explosion" => {
            let f = r.r(80.0, 300.0);
            set(&mut p, vec![("wave", json!("noise")), ("freq", json!(f)), ("freqEnd", json!(f * r.r(0.2, 0.6))), ("sustain", json!(r.r(0.08, 0.3))), ("decay", json!(r.r(0.3, 0.8))), ("punch", json!(r.r(30.0, 80.0))), ("cutoff", json!(r.r(1500.0, 6000.0))), ("volume", json!(70))]);
        }
        "powerup" => {
            let f = r.r(200.0, 450.0);
            let w = r.pick(&["square", "triangle", "saw"]);
            set(&mut p, vec![("wave", json!(w)), ("freq", json!(f)), ("freqEnd", json!(f * r.r(2.0, 4.0))), ("sustain", json!(r.r(0.15, 0.3))), ("decay", json!(r.r(0.15, 0.35))), ("vibDepth", json!(r.r(0.0, 1.5))), ("vibSpeed", json!(r.r(8.0, 18.0))), ("duty", json!(r.r(30.0, 50.0)))]);
        }
        "hurt" => {
            let f = r.r(200.0, 600.0);
            let w = r.pick(&["square", "saw", "noise"]);
            set(&mut p, vec![("wave", json!(w)), ("freq", json!(f)), ("freqEnd", json!(f * r.r(0.2, 0.5))), ("sustain", json!(r.r(0.02, 0.08))), ("decay", json!(r.r(0.1, 0.25))), ("punch", json!(r.r(20.0, 60.0))), ("cutoff", json!(r.r(3000.0, 12000.0)))]);
        }
        "blip" => {
            let f = r.r(400.0, 1400.0);
            let w = r.pick(&["square", "sine"]);
            set(&mut p, vec![("wave", json!(w)), ("freq", json!(f)), ("freqEnd", json!(f)), ("sustain", json!(r.r(0.02, 0.05))), ("decay", json!(r.r(0.02, 0.06))), ("duty", json!(r.r(25.0, 50.0))), ("volume", json!(50))]);
        }
        "step" => {
            let f = r.r(60.0, 160.0);
            set(&mut p, vec![("wave", json!("noise")), ("freq", json!(f)), ("freqEnd", json!(f * 0.7)), ("sustain", json!(0.01)), ("decay", json!(r.r(0.04, 0.09))), ("cutoff", json!(r.r(800.0, 2500.0))), ("volume", json!(55))]);
        }
        _ => {
            let w = r.pick(&["square", "saw", "triangle", "sine", "noise"]);
            p["wave"] = json!(w);
            for d in SFXP {
                if d.0 != "volume" {
                    p[d.0] = json!(r.r(d.3, d.4));
                }
            }
            p["decay"] = json!(r.r(0.08, 0.6));
            p["sustain"] = json!(r.r(0.0, 0.3));
            p["attack"] = json!(r.r(0.0, 0.08));
            p["vibDepth"] = json!(r.r(0.0, 2.0));
            p["arp"] = json!(r.r(-12.0, 12.0).round());
            p["cutoff"] = json!(r.r(2000.0, 20000.0));
        }
    }
    for d in SFXP {
        let v = p.f(d.0, 0.0);
        p[d.0] = json!(((v / d.5).round() * d.5).clamp(d.3, d.4));
    }
    p
}

pub fn sfx_synth(p: &Json, sr: u32) -> Vec<f32> {
    let srf = sr as f64;
    let a = p.f("attack", 0.0).max(0.0);
    let s = p.f("sustain", 0.0).max(0.0);
    let d = p.f("decay", 0.2).max(0.01);
    let len = a + s + d;
    let n = (len * srf).ceil() as usize;
    let mut out = vec![0f32; n];
    let mut r = Rng(0x2545_F491_4F6C_DD1D);
    let mut noise = [0f64; 32];
    for x in noise.iter_mut() {
        *x = r.f() * 2.0 - 1.0;
    }
    let al = 1.0 - (-2.0 * std::f64::consts::PI * p.f("cutoff", 20000.0).min(srf / 2.0) / srf).exp();
    let f0 = p.f("freq", 440.0).max(1.0);
    let f1 = p.f("freqEnd", 440.0).max(1.0);
    let duty = p.f("duty", 50.0) / 100.0;
    let vol = p.f("volume", 60.0) / 100.0;
    let punch = p.f("punch", 0.0);
    let (vd, vs, arp, arpt) = (p.f("vibDepth", 0.0), p.f("vibSpeed", 0.0), p.f("arp", 0.0), p.f("arpT", 0.0));
    let wave = p.s("wave").to_string();
    let (mut ph, mut y, mut last) = (0.0f64, 0.0f64, -1i64);
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f64 / srf;
        let env = if t < a { if a > 0.0 { t / a } else { 1.0 } } else if t < a + s { 1.0 + (punch / 100.0) * (1.0 - (t - a) / if s == 0.0 { 1.0 } else { s }) } else { 1.0 - (t - a - s) / d };
        let mut f = f0 * (f1 / f0).powf(t / len);
        if vd > 0.0 && vs > 0.0 {
            f *= 2f64.powf(vd * (2.0 * std::f64::consts::PI * vs * t).sin() / 12.0);
        }
        if arp != 0.0 && t >= arpt {
            f *= 2f64.powf(arp / 12.0);
        }
        ph += f / srf;
        let fr = ph - ph.floor();
        let v = match wave.as_str() {
            "saw" => 2.0 * fr - 1.0,
            "triangle" => if fr < 0.5 { 4.0 * fr - 1.0 } else { 3.0 - 4.0 * fr },
            "sine" => (2.0 * std::f64::consts::PI * fr).sin(),
            "noise" => {
                let seg = (ph * 32.0).floor() as i64;
                if seg != last {
                    last = seg;
                    noise[(seg & 31) as usize] = r.f() * 2.0 - 1.0;
                }
                noise[(seg & 31) as usize]
            }
            _ => if fr < duty { 1.0 } else { -1.0 },
        };
        y += al * (v - y);
        *o = (y * env * vol * 0.8).clamp(-1.0, 1.0) as f32;
    }
    out
}

pub fn wav_url(data: &[f32], sr: u32) -> String {
    let n = data.len() as u32;
    let mut b: Vec<u8> = Vec::with_capacity(44 + data.len() * 2);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + n * 2).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&sr.to_le_bytes());
    b.extend_from_slice(&(sr * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(n * 2).to_le_bytes());
    for v in data {
        b.extend_from_slice(&((v * 32767.0).round() as i16).to_le_bytes());
    }
    format!("data:audio/wav;base64,{}", pf_core::b64_encode(&b))
}

pub struct SfxEd {
    pub sound: Option<String>,
    pub p: Json,
    pub name: String,
    data: Vec<f32>,
    dirty: bool,
}
impl SfxEd {
    pub fn new(app: &App, sound: Option<String>) -> SfxEd {
        let s = sound.as_ref().and_then(|id| app.proj.arr("sounds").iter().find(|x| x.s("id") == id)).cloned();
        let mut p = sfx_defaults();
        if let Some(sp) = s.as_ref().and_then(|s| s.get("sfx")).and_then(|v| v.as_object()) {
            for (k, v) in sp {
                p[k] = v.clone();
            }
        } else if s.is_none() {
            p = sfx_preset("coin");
        }
        let name = s.map(|s| s.s("name").to_string()).unwrap_or_else(|| {
            let names: Vec<String> = app.proj.arr("sounds").iter().map(|x| x.s("name").to_string()).collect();
            let mut k = 1;
            while names.contains(&format!("Effet{}", k)) {
                k += 1;
            }
            format!("Effet{}", k)
        });
        SfxEd { sound, p, name, data: Vec::new(), dirty: true }
    }
}

pub fn sfx_window(app: &mut App, ctx: &egui::Context, now: f64) {
    let mut ed = match app.sfx_ed.take() {
        Some(e) => e,
        None => return,
    };
    let mut open = true;
    let mut close = false;
    let mut play = false;
    let mut save = false;
    if ed.dirty {
        ed.data = sfx_synth(&ed.p, 22050);
        ed.dirty = false;
    }
    egui::Window::new(if ed.sound.is_some() { "Modifier l’effet sonore" } else { "Créer un effet sonore" }).id(egui::Id::new("sfxed")).open(&mut open).default_width(640.0).show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (k, n) in PRESETS {
                if ui.add(theme::btn_small(n)).clicked() {
                    ed.p = sfx_preset(k);
                    ed.dirty = true;
                    play = true;
                }
            }
            if ui.add(theme::btn_small("🔀 Varier")).clicked() {
                let mut r = Rng(seed());
                for d in SFXP {
                    if d.0 == "volume" || r.f() < 0.5 {
                        continue;
                    }
                    let rr = (d.4 - d.3) * 0.06;
                    let v = ed.p.f(d.0, 0.0) + r.r(-rr, rr);
                    ed.p[d.0] = json!(((v / d.5).round() * d.5).clamp(d.3, d.4));
                }
                ed.dirty = true;
                play = true;
            }
        });
        // forme du son
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 80.0), egui::Sense::hover());
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 6.0, C::BG0);
        p.line_segment([rect.left_center(), rect.right_center()], egui::Stroke::new(1.0, C::LINE2));
        let n = ed.data.len().max(1);
        let w = rect.width() as usize;
        for x in 0..w {
            let a = x * n / w;
            let b = ((x + 1) * n / w + 1).min(n);
            let (mut mn, mut mx) = (0f32, 0f32);
            for v in &ed.data[a.min(n - 1)..b.max(a + 1).min(ed.data.len().max(1))] {
                mn = mn.min(*v);
                mx = mx.max(*v);
            }
            let h = rect.height() / 2.0 * 1.2;
            p.line_segment([egui::pos2(rect.left() + x as f32, rect.center().y - mx * h), egui::pos2(rect.left() + x as f32, rect.center().y - mn * h + 1.0)], egui::Stroke::new(1.0, C::ACCENT));
        }
        ui.horizontal(|ui| {
            if ui.add(theme::btn_accent("▶ Écouter")).clicked() {
                play = true;
            }
            ui.label(RichText::new(format!("Durée {:.2} s", ed.data.len() as f64 / 22050.0)).color(C::MUTED));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Forme d’onde").color(C::MUTED));
            for (k, n) in SFXW {
                if ui.add(theme::tab(ed.p.s("wave") == *k, n)).clicked() {
                    ed.p["wave"] = json!(k);
                    ed.dirty = true;
                }
            }
        });
        egui::Grid::new("sfxp").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
            for d in SFXP {
                let off = (d.0 == "duty" && ed.p.s("wave") != "square") || (d.0 == "arpT" && ed.p.f("arp", 0.0) == 0.0) || (d.0 == "vibSpeed" && ed.p.f("vibDepth", 0.0) == 0.0);
                ui.label(RichText::new(d.1).color(if off { C::DIM } else { C::MUTED }));
                let mut v = ed.p.f(d.0, 0.0);
                let r = ui.add_enabled(!off, egui::Slider::new(&mut v, d.3..=d.4).step_by(d.5).suffix(format!(" {}", d.2)));
                if r.changed() {
                    ed.p[d.0] = json!(v);
                    ed.dirty = true;
                }
                if r.drag_stopped() {
                    play = true;
                }
                ui.end_row();
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Nom du son").color(C::MUTED));
            ui.add(egui::TextEdit::singleline(&mut ed.name).desired_width(160.0));
            if ui.add(theme::btn_accent(if ed.sound.is_some() { "Enregistrer" } else { "Ajouter au projet" })).clicked() {
                save = true;
            }
            if ui.add(theme::btn("Annuler")).clicked() {
                close = true;
            }
        });
    });
    if ed.dirty {
        ed.data = sfx_synth(&ed.p, 22050);
        ed.dirty = false;
    }
    if play {
        Preview::play(app, ed.data.clone(), 22050);
    }
    if save {
        let nm = ed.name.trim().to_string();
        let clash = app.proj.arr("sounds").iter().any(|s| s.s("name") == nm && Some(s.s("id").to_string()) != ed.sound);
        if nm.is_empty() || clash {
            app.toast(if nm.is_empty() { "Nom obligatoire." } else { "Ce nom de son est déjà utilisé." }, true, now);
            app.sfx_ed = Some(ed);
            return;
        }
        app.snapshot();
        if !app.proj.get("sounds").map(|s| s.is_array()).unwrap_or(false) {
            app.proj["sounds"] = json!([]);
        }
        let url = wav_url(&ed.data, 22050);
        match &ed.sound {
            Some(id) => {
                if let Some(s) = app.proj["sounds"].as_array_mut().unwrap().iter_mut().find(|s| s.s("id") == id) {
                    s["name"] = json!(nm);
                    s["data"] = json!(url);
                    s["sfx"] = ed.p.clone();
                }
            }
            None => {
                let id = pf_core::uid("snd");
                app.proj["sounds"].as_array_mut().unwrap().push(json!({"id":id,"name":nm,"data":url,"sfx":ed.p}));
                app.sel = crate::Sel::Sound(id);
            }
        }
        app.changed();
        app.toast("Effet sonore enregistré.", false, now);
        return;
    }
    if open && !close {
        app.sfx_ed = Some(ed);
    }
}

// =====================================================================================
// éditeur visuel de piste

const TK_PRESETS: &[(&str, &str, &[[f64; 3]])] = &[
    ("Balade facile", "Longues lignes droites, virages doux, petites bosses", &[[60.0, 0.0, 0.0], [60.0, 2.0, 0.0], [50.0, 0.0, 25.0], [60.0, -2.0, -25.0], [80.0, 0.0, 0.0], [60.0, 2.0, 0.0], [50.0, -2.0, 0.0], [60.0, 0.0, 0.0]]),
    ("Rallye en forêt", "Enchaînement de virages et de bosses", &[[30.0, 0.0, 0.0], [50.0, 2.0, 25.0], [40.0, 4.0, 0.0], [60.0, -4.0, -25.0], [30.0, 0.0, 50.0], [50.0, 6.0, 0.0], [40.0, -6.0, -50.0], [60.0, 0.0, 25.0], [40.0, -2.0, -25.0], [80.0, 4.0, 25.0], [30.0, -6.0, 0.0], [50.0, 0.0, -25.0]]),
    ("Col de montagne", "Grandes montées, descentes et épingles", &[[40.0, 0.0, 0.0], [80.0, 2.0, 50.0], [40.0, -6.0, 0.0], [80.0, -2.0, 50.0], [40.0, 6.0, 0.0], [60.0, 0.0, -50.0], [50.0, -4.0, -50.0], [60.0, 4.0, 0.0], [50.0, 0.0, 0.0]]),
    ("Ovale de vitesse", "Deux lignes droites et deux grands virages", &[[120.0, 0.0, 0.0], [80.0, 4.0, 0.0], [120.0, 0.0, 0.0], [80.0, 4.0, 0.0]]),
    ("Slalom", "Virages alternés gauche / droite, très technique", &[[30.0, 0.0, 0.0], [30.0, -4.0, 0.0], [30.0, 4.0, 0.0], [30.0, -4.0, 0.0], [30.0, 4.0, 0.0], [40.0, 0.0, 25.0], [30.0, -6.0, 0.0], [30.0, 6.0, -25.0], [50.0, 0.0, 0.0]]),
];

fn tk_parse(t: &str) -> Vec<[f64; 3]> {
    t.split(|c| c == '\n' || c == ';').filter_map(|l| {
        let q: Vec<f64> = l.trim().split(|c: char| c.is_whitespace() || c == ',').filter(|x| !x.is_empty()).map(pf_core::value::str_to_num).collect();
        if q.is_empty() || !q[0].is_finite() || q[0] <= 0.0 {
            return None;
        }
        Some([q[0].round(), q.get(1).copied().filter(|v| v.is_finite()).unwrap_or(0.0), q.get(2).copied().filter(|v| v.is_finite()).unwrap_or(0.0)])
    }).collect()
}
fn tk_text(s: &[[f64; 3]]) -> String {
    s.iter().map(|q| format!("{} {} {}", q[0], pf_core::Val::N(q[1]).str(), pf_core::Val::N(q[2]).str())).collect::<Vec<_>>().join("\n")
}
fn tk_desc(q: &[f64; 3]) -> String {
    let len = if q[0] < 30.0 { "Très court" } else if q[0] < 60.0 { "Court" } else if q[0] < 105.0 { "" } else if q[0] < 165.0 { "Long" } else { "Très long" };
    let ac = q[1].abs();
    let side = if q[1] < 0.0 { "à gauche" } else { "à droite" };
    let cur = if ac < 0.5 { "ligne droite".to_string() } else if ac < 3.0 { format!("virage léger {}", side) } else if ac < 5.0 { format!("virage {}", side) } else { format!("virage serré {}", side) };
    let hl = if q[2] >= 40.0 { " qui monte fort" } else if q[2] >= 10.0 { " qui monte" } else if q[2] <= -40.0 { " qui descend fort" } else if q[2] <= -10.0 { " qui descend" } else { "" };
    format!("{}{}{}", if len.is_empty() { String::new() } else { format!("{} : ", len) }, cur, hl)
}

pub struct TrackEd {
    pub oid: String,
    secs: Vec<[f64; 3]>,
    pos: f64,
    tex: Option<(u64, egui::TextureHandle)>,
    ver: u64,
    playing: bool,
    g: Option<pf_runtime::gfx::Gfx>,
}
impl TrackEd {
    pub fn new(app: &App, oid: &str) -> TrackEd {
        let t = app.obj(oid).map(|o| if o.s("track").is_empty() { pf_runtime::road::RD_TRACK.to_string() } else { o.s("track").to_string() }).unwrap_or_default();
        TrackEd { oid: oid.to_string(), secs: tk_parse(&t), pos: 0.0, tex: None, ver: 1, playing: false, g: None }
    }
}

pub fn track_window(app: &mut App, ctx: &egui::Context) {
    let mut ed = match app.track_ed.take() {
        Some(e) => e,
        None => return,
    };
    let o = match app.obj(&ed.oid) {
        Some(o) => o.clone(),
        None => return,
    };
    let mut open = true;
    let mut ch = false;
    egui::Window::new(format!("Éditeur de piste — {}", o.s("name"))).id(egui::Id::new("tked")).open(&mut open).default_size([900.0, 620.0]).show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Modèle :").color(C::MUTED));
            for (n, d, t) in TK_PRESETS {
                if ui.add(theme::btn_small(n)).on_hover_text(*d).clicked() {
                    ed.secs = t.to_vec();
                    ch = true;
                }
            }
        });
        ui.horizontal_top(|ui| {
            // ---- tronçons
            ui.vertical(|ui| {
                ui.set_width(420.0);
                theme::section(ui, "Tronçons (dans l’ordre du circuit)");
                let mut act: Option<(usize, i32)> = None;
                egui::ScrollArea::vertical().max_height(470.0).show(ui, |ui| {
                    for k in 0..ed.secs.len() {
                        egui::Frame::new().fill(C::BG2).corner_radius(6.0).inner_margin(6.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{}.", k + 1)).color(C::DIM));
                                ui.label(RichText::new(tk_desc(&ed.secs[k])).color(C::TEXT2));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("✕").clicked() {
                                        act = Some((k, 0));
                                    }
                                    if ui.small_button("↓").clicked() {
                                        act = Some((k, 1));
                                    }
                                    if ui.small_button("↑").clicked() {
                                        act = Some((k, -1));
                                    }
                                    if ui.small_button("copie").on_hover_text("Dupliquer").clicked() {
                                        act = Some((k, 2));
                                    }
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Longueur").size(11.0).color(C::MUTED));
                                ch |= ui.add(egui::DragValue::new(&mut ed.secs[k][0]).range(5.0..=2000.0).speed(1.0)).changed();
                                ui.label(RichText::new("Virage").size(11.0).color(C::MUTED));
                                ch |= ui.add(egui::Slider::new(&mut ed.secs[k][1], -8.0..=8.0).step_by(0.5).show_value(true)).on_hover_text("négatif = à gauche, positif = à droite").changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Relief").size(11.0).color(C::MUTED));
                                ch |= ui.add(egui::Slider::new(&mut ed.secs[k][2], -80.0..=80.0).step_by(5.0)).on_hover_text("positif = monte, négatif = descend").changed();
                                if ui.add(theme::btn_small("Voir")).on_hover_text("Placer la caméra au début de ce tronçon").clicked() {
                                    let before: f64 = ed.secs[..k].iter().map(|q| q[0]).sum();
                                    ed.pos = before * o.f_or("segLen", 200.0);
                                    ed.ver += 1;
                                }
                            });
                        });
                    }
                });
                if let Some((k, d)) = act {
                    match d {
                        0 => {
                            if ed.secs.len() > 1 {
                                ed.secs.remove(k);
                            }
                        }
                        2 => {
                            let q = ed.secs[k];
                            ed.secs.insert(k + 1, q);
                        }
                        _ => {
                            let nk = k as i32 + d;
                            if nk >= 0 && (nk as usize) < ed.secs.len() {
                                ed.secs.swap(k, nk as usize);
                            }
                        }
                    }
                    ch = true;
                }
                ui.horizontal(|ui| {
                    if ui.add(theme::btn_small("+ Ligne droite")).clicked() {
                        ed.secs.push([60.0, 0.0, 0.0]);
                        ch = true;
                    }
                    if ui.add(theme::btn_small("+ Virage à gauche")).clicked() {
                        ed.secs.push([50.0, -3.0, 0.0]);
                        ch = true;
                    }
                    if ui.add(theme::btn_small("+ Virage à droite")).clicked() {
                        ed.secs.push([50.0, 3.0, 0.0]);
                        ch = true;
                    }
                    if ui.add(theme::btn_small("+ Bosse")).clicked() {
                        ed.secs.push([40.0, 0.0, 30.0]);
                        ch = true;
                    }
                });
            });
            ui.separator();
            // ---- aperçus
            ui.vertical(|ui| {
                let mut oj = o.clone();
                oj["track"] = Json::String(tk_text(&ed.secs));
                oj["minimap"] = Json::Bool(false);
                let r = pf_runtime::road::rd_build(&oj);
                theme::section(ui, "Vue du pilote");
                ui.horizontal(|ui| {
                    if ui.add(theme::btn_small(if ed.playing { "❚❚ Pause" } else { "▶ Rouler" })).clicked() {
                        ed.playing = !ed.playing;
                    }
                    let mut p = ed.pos / r.len.max(1.0) * 100.0;
                    if ui.add(egui::Slider::new(&mut p, 0.0..=100.0).suffix(" %").show_value(false)).changed() {
                        ed.pos = p / 100.0 * r.len;
                        ed.ver += 1;
                    }
                });
                if ed.playing {
                    ed.pos = (ed.pos + r.l * 60.0 * 0.6 * ctx.input(|i| i.stable_dt as f64)) % r.len;
                    ed.ver += 1;
                    ctx.request_repaint();
                }
                let (w, h) = (420u32, 236u32);
                if ed.tex.as_ref().map(|t| t.0 != ed.ver).unwrap_or(true) {
                    let g = ed.g.get_or_insert_with(|| pf_runtime::gfx::Gfx::new(w, h));
                    g.reset();
                    g.fill_rect(0.0, 0.0, w as f32, h as f32, pf_runtime::gfx::parse_color("#7ec8f2"));
                    let st = pf_runtime::road::RoadSt { pos: ed.pos, ..Default::default() };
                    let imgs = &mut app.imgs;
                    let defs = &app.defs;
                    let lk = |id: &str| -> Option<(f64, std::rc::Rc<tiny_skia::Pixmap>)> {
                        let d = defs.get(id)?;
                        let k = *d.anims.first()?.frames.first()?;
                        use pf_runtime::draw::Imgs;
                        Some((d.j.f_or("defW", 0.0), imgs.img(k)?))
                    };
                    let sc = w as f64 / o.f_or("defW", 854.0);
                    g.save();
                    g.scale(sc as f32, sc as f32);
                    pf_runtime::road::rd_draw(g, &oj, 0.0, 0.0, o.f_or("defW", 854.0), h as f64 / sc, Some(&st), &lk);
                    g.restore();
                    let rgba = pf_runtime::img::to_rgba(&g.pm);
                    let t = ctx.load_texture("tkview", egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba), egui::TextureOptions::LINEAR);
                    ed.tex = Some((ed.ver, t));
                }
                ui.image((ed.tex.as_ref().unwrap().1.id(), egui::vec2(w as f32, h as f32)));
                // vue de dessus
                theme::section(ui, "Vue de dessus et relief");
                let (rect, _) = ui.allocate_exact_size(egui::vec2(w as f32, 150.0), egui::Sense::hover());
                let p = ui.painter_at(rect);
                p.rect_filled(rect, 6.0, C::BG0);
                let mb = r.mb;
                let sc = ((rect.width() - 20.0) as f64 / (mb[2] - mb[0]).max(1e-6)).min((rect.height() - 20.0) as f64 / (mb[3] - mb[1]).max(1e-6));
                let cx = rect.center().x as f64 - (mb[0] + mb[2]) / 2.0 * sc;
                let cy = rect.center().y as f64 - (mb[1] + mb[3]) / 2.0 * sc;
                let pts: Vec<egui::Pos2> = r.map.iter().map(|q| egui::pos2((cx + q[0] * sc) as f32, (cy + q[1] * sc) as f32)).collect();
                let mut closed = pts.clone();
                if let Some(f) = pts.first() {
                    closed.push(*f);
                }
                p.add(egui::Shape::line(closed, egui::Stroke::new(3.0, theme::color32(if o.s("roadA").is_empty() { "#727280" } else { o.s("roadA") }))));
                if let Some(f) = pts.first() {
                    p.rect_filled(egui::Rect::from_center_size(*f, egui::vec2(6.0, 6.0)), 1.0, egui::Color32::WHITE);
                }
                let k = ((ed.pos / r.l).floor() as usize) % pts.len().max(1);
                if let Some(me) = pts.get(k) {
                    p.circle_filled(*me, 5.0, egui::Color32::from_rgb(0xff, 0x3b, 0x4f));
                }
                let (rect2, _) = ui.allocate_exact_size(egui::vec2(w as f32, 60.0), egui::Sense::hover());
                let p2 = ui.painter_at(rect2);
                p2.rect_filled(rect2, 6.0, C::BG0);
                let ys: Vec<f64> = r.segs.iter().map(|s| s.p1.y).collect();
                let (mn, mx) = ys.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, &y| (a.0.min(y), a.1.max(y)));
                let span = (mx - mn).max(1.0);
                let prof: Vec<egui::Pos2> = ys.iter().enumerate().map(|(i, y)| egui::pos2(rect2.left() + 6.0 + (i as f32 / ys.len().max(1) as f32) * (rect2.width() - 12.0), rect2.bottom() - 8.0 - ((y - mn) / span) as f32 * (rect2.height() - 16.0))).collect();
                p2.add(egui::Shape::line(prof, egui::Stroke::new(2.0, C::OK)));
                let x = rect2.left() + 6.0 + (k as f32 / ys.len().max(1) as f32) * (rect2.width() - 12.0);
                p2.line_segment([egui::pos2(x, rect2.top()), egui::pos2(x, rect2.bottom())], egui::Stroke::new(1.0, egui::Color32::from_rgb(0xff, 0x3b, 0x4f)));
                ui.label(RichText::new(format!("{} tronçons · {} segments · un tour ≈ {:.0} s à fond", ed.secs.len(), r.segs.len(), r.len / (r.l * 60.0))).size(11.0).color(C::DIM));
            });
        });
    });
    if ch {
        ed.ver += 1;
        app.snapshot();
        let t = tk_text(&ed.secs);
        let oid = ed.oid.clone();
        if let Some(ob) = app.obj_mut(&oid) {
            ob["track"] = Json::String(t);
        }
        app.changed();
    }
    if open {
        app.track_ed = Some(ed);
    }
}
