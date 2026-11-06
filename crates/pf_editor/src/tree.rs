//! Panneau Projet : niveaux, feuilles d'événements, objets, sons, variables globales.

use crate::theme::{self, C};
use crate::{App, Sel, Tab};
use eframe::egui;
use pf_core::{json, Json, JsonExt};

#[derive(Default)]
pub struct TreeState {
    pub filter: String,
}

pub const KINDS: [(&str, &str, &str); 13] = [
    ("sprite", "Sprite", "◼"),
    ("tiled", "Mosaïque", "▦"),
    ("tilemap", "Carte de tuiles", "▤"),
    ("text", "Texte", "T"),
    ("button", "Bouton", "▭"),
    ("bar", "Barre", "▬"),
    ("particles", "Particules", "✦"),
    ("leaderboard", "Classement", "☰"),
    ("road3d", "Route 3D", "◢"),
    ("minimap", "Mini-carte", "◎"),
    ("array", "Tableau", "[ ]"),
    ("dict", "Dictionnaire", "{ }"),
    ("family", "Famille", "⧉"),
];

pub fn glyph(kind: &str) -> &'static str {
    KINDS.iter().find(|k| k.0 == kind).map(|k| k.2).unwrap_or("•")
}
pub fn kind_name(kind: &str) -> &'static str {
    KINDS.iter().find(|k| k.0 == kind).map(|k| k.1).unwrap_or("Objet")
}

/// petite image PNG unie (data URL) pour les nouveaux objets
pub fn solid_png(w: u32, h: u32, rgba: [u8; 4], border: Option<[u8; 4]>) -> String {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let edge = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            let c = if edge { border.unwrap_or(rgba) } else { rgba };
            let k = ((y * w + x) * 4) as usize;
            buf[k..k + 4].copy_from_slice(&c);
        }
    }
    let mut out = Vec::new();
    {
        let enc = image::codecs::png::PngEncoder::new(&mut out);
        use image::ImageEncoder;
        let _ = enc.write_image(&buf, w, h, image::ExtendedColorType::Rgba8);
    }
    format!("data:image/png;base64,{}", pf_core::b64_encode(&out))
}

fn tileset_png() -> String {
    let cols: [[u8; 4]; 8] = [[88, 184, 79, 255], [139, 94, 60, 255], [127, 132, 145, 255], [233, 198, 120, 255], [74, 120, 200, 255], [200, 80, 90, 255], [60, 60, 70, 255], [240, 240, 240, 255]];
    let (w, h) = (64u32, 32u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let t = (y / 16) * 4 + x / 16;
            let mut c = cols[t as usize];
            if x % 16 == 0 || y % 16 == 0 {
                for v in c.iter_mut().take(3) {
                    *v = (*v as f32 * 0.8) as u8;
                }
            }
            let k = ((y * w + x) * 4) as usize;
            buf[k..k + 4].copy_from_slice(&c);
        }
    }
    let mut out = Vec::new();
    {
        use image::ImageEncoder;
        let _ = image::codecs::png::PngEncoder::new(&mut out).write_image(&buf, w, h, image::ExtendedColorType::Rgba8);
    }
    format!("data:image/png;base64,{}", pf_core::b64_encode(&out))
}

pub fn unique_name(app: &App, base: &str) -> String {
    let names: Vec<String> = app.proj.arr("objects").iter().map(|o| o.s("name").to_string()).collect();
    if !names.contains(&base.to_string()) {
        return base.to_string();
    }
    let mut k = 2;
    loop {
        let n = format!("{}{}", base, k);
        if !names.contains(&n) {
            return n;
        }
        k += 1;
    }
}

pub fn make_object(app: &App, kind: &str) -> Json {
    let name = unique_name(app, match kind {
        "sprite" => "Sprite",
        "tiled" => "Mosaique",
        "tilemap" => "Tuiles",
        "text" => "Texte",
        "button" => "Bouton",
        "bar" => "Barre",
        "particles" => "Particules",
        "leaderboard" => "Classement",
        "road3d" => "Route",
        "minimap" => "MiniCarte",
        "array" => "Tableau",
        "dict" => "Dictionnaire",
        _ => "Famille",
    });
    let id = pf_core::uid("o");
    let mut o = match kind {
        "sprite" => json!({"kind":"sprite","w":32,"h":32,"ox":16,"oy":16,"cbox":{"x":0,"y":0,"w":32,"h":32},"collision":"auto","defW":32,"defH":32,
            "anims":[{"id":pf_core::uid("a"),"name":"Défaut","speed":5,"loop":true,"frames":[solid_png(32,32,[255,138,61,255],Some([26,15,7,255]))]}],"points":[]}),
        "tiled" => json!({"kind":"tiled","w":16,"h":16,"image":solid_png(16,16,[120,140,170,255],Some([80,94,120,255])),"scale":2,"defW":128,"defH":32}),
        "tilemap" => json!({"kind":"tilemap","image":tileset_png(),"w":64,"h":32,"tw":16,"th":16,"scale":2,"tcol":{},"defW":256,"defH":128,"behaviors":[{"type":"solid","props":{}}]}),
        "text" => json!({"kind":"text","size":20,"color":"#ffffff","bold":true,"align":"left","font":"round","outline":true,"outlineColor":"#1b1f2a","text":"Texte","defW":240,"defH":32}),
        "button" => json!({"kind":"button","text":"Bouton","size":20,"font":"round","bg":"#3d6df2","fg":"#ffffff","hoverBg":"#5a86ff","hoverFg":"#ffffff","radius":10,"border":0,"borderColor":"#ffffff","shadow":true,"focusColor":"#ffd84a","defW":220,"defH":48}),
        "bar" => json!({"kind":"bar","max":100,"value":100,"fg":"#e8566a","bg":"#1b1f2a","ghostColor":"#ffffff","lowColor":"","lowAt":25,"border":2,"borderColor":"#ffffff","radius":6,"dir":"ltr","segments":0,"showText":false,"format":"{valeur} / {max}","size":14,"font":"round","textColor":"#ffffff","smooth":true,"defW":200,"defH":20}),
        "particles" => {
            let mut p = pf_core::catalog::catalog()["PTPRESETS"]["dust"].clone();
            if let Some(m) = p.as_object_mut() {
                m.remove("n");
            }
            json!({"kind":"particles","pt":p,"defW":32,"defH":32})
        }
        "leaderboard" => json!({"kind":"leaderboard","table":"scores","rows":5,"title":"Meilleurs scores","showTitle":true,"size":18,"font":"round","fg":"#ffffff","accent":"#ffd84d","bg":"#1b1f2a","bgAlpha":60,"suffix":"","defW":360,"defH":240}),
        "road3d" => json!({"kind":"road3d","track":pf_runtime::road::RD_TRACK,"segLen":200,"roadWidth":2000,"lanes":2,"rumble":3,"drawDist":160,"camHeight":1000,"fov":100,"fogDensity":4,"fog":"#9fd3f2","grassA":"#58b84f","grassB":"#4fa847","roadA":"#727280","roadB":"#6b6b73","rumbleA":"#e8566a","rumbleB":"#f4f4f4","lane":"#f4f4f4","maxKmh":220,"accelTime":5,"brakePower":1,"centrifugal":0.3,"steering":1,"offroadMax":30,"offroadDrag":1,"playerW":0.2,"spriteScale":2,"engine":true,"autoSigns":true,"minimap":false,"mmCorner":"tr","mmSize":130,"controls":true,"decor":"","decorEvery":6,"decor2":"","decor2Every":17,"seed":7,"rivalObj":"","rivals":0,"gridGap":5,"rivalMin":60,"rivalMax":85,"trafficObj":"","traffic":0,"defW":854,"defH":480}),
        "minimap" => json!({"kind":"minimap","mode":"radar","follow":"","radius":600,"shape":"round","bg":"#0b0d12","bgAlpha":75,"border":"#ffffff","borderW":2,"solids":true,"solidColor":"#8b93a6","followColor":"#ffffff","dots":[{"obj":"","color":"#ff3b4f","size":5},{"obj":"","color":"#ffd84d","size":5}],"defW":160,"defH":160}),
        "array" => json!({"kind":"array","items":[]}),
        "dict" => json!({"kind":"dict","entries":{}}),
        _ => json!({"kind":"family","members":[]}),
    };
    o["id"] = Json::String(id);
    o["name"] = Json::String(name);
    if o.get("behaviors").is_none() {
        o["behaviors"] = json!([]);
    }
    o["vars"] = json!([]);
    // place les clés id / name en tête
    let mut m = pf_core::Map::new();
    m.insert("id".into(), o["id"].clone());
    m.insert("name".into(), o["name"].clone());
    for (k, v) in o.as_object().unwrap() {
        if k != "id" && k != "name" {
            m.insert(k.clone(), v.clone());
        }
    }
    Json::Object(m)
}

/// crée une instance de l'objet dans le niveau courant
pub fn add_instance(app: &mut App, oid: &str, x: f64, y: f64) {
    let o = match app.obj(oid) {
        Some(o) => o.clone(),
        None => return,
    };
    let k = o.s("kind");
    if k == "array" || k == "dict" || k == "family" {
        return;
    }
    app.snapshot();
    let layer = app.view.active_layer.clone().or_else(|| app.layout().and_then(|l| l.arr("layers").iter().find(|y| y.f("parallaxX", 100.0) == 100.0).or_else(|| l.arr("layers").first()).map(|y| y.s("id").to_string())));
    let id = pf_core::uid("i");
    let (w, h) = (o.f_or("defW", 32.0), o.f_or("defH", 32.0));
    let mut i = json!({"id":id,"obj":oid,"x":x.round(),"y":y.round(),"w":w,"h":h,"angle":0,"mirrored":false,"opacity":1,"visible":true});
    if let Some(l) = layer {
        i["layer"] = Json::String(l);
    }
    if let Some(l) = app.layout_mut() {
        if l.get("instances").is_none() {
            l["instances"] = json!([]);
        }
        l["instances"].as_array_mut().unwrap().push(i);
    }
    app.sel = Sel::Inst(id);
    app.changed();
}

pub fn panel(app: &mut App, ctx: &egui::Context, now: f64) {
    egui::SidePanel::left("tree").default_width(250.0).min_width(200.0).frame(egui::Frame::new().fill(C::BG1()).inner_margin(egui::Margin::symmetric(10, 8)).stroke(egui::Stroke::new(1.0_f32, C::LINE()))).show(ctx, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // ---- niveaux
            ui.horizontal(|ui| {
                theme::section(ui, "Niveaux");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::btn_small("+")).on_hover_text("Nouveau niveau").clicked() {
                        app.snapshot();
                        let n = app.proj.arr("layouts").len() + 1;
                        let sid = pf_core::uid("s");
                        let lid = pf_core::uid("l");
                        let w = app.proj.get("settings").map(|s| s.f("width", 854.0)).unwrap_or(854.0);
                        let h = app.proj.get("settings").map(|s| s.f("height", 480.0)).unwrap_or(480.0);
                        app.proj["sheets"].as_array_mut().map(|a| a.push(json!({"id":sid,"name":format!("Événements {}", n),"events":[]})));
                        app.proj["layouts"].as_array_mut().map(|a| a.push(json!({"id":lid,"name":format!("Niveau {}", n),"w":w,"h":h,"bg":"#7ec8f2","sheet":sid,"layers":[{"id":pf_core::uid("y"),"name":"Principal","parallaxX":100,"parallaxY":100,"opacity":100,"visible":true}],"instances":[]})));
                        app.cur_layout = lid.clone();
                        app.cur_sheet = sid;
                        app.sel = Sel::Layout(lid);
                        app.view.fit_pending = true;
                        app.changed();
                    }
                });
            });
            let start = app.proj.get("settings").map(|s| s.s("startLayout").to_string()).unwrap_or_default();
            for l in app.proj.arr("layouts").to_vec() {
                let id = l.s("id").to_string();
                let on = id == app.cur_layout;
                let star = if id == start || (start.is_empty() && app.proj.arr("layouts").first().map(|x| x.s("id")) == Some(&id)) { "  ★" } else { "" };
                let r = ui.add(egui::SelectableLabel::new(on, egui::RichText::new(format!("▦  {}{}", l.s("name"), star)).color(if on { C::TEXT() } else { C::TEXT2() })));
                if r.clicked() {
                    app.cur_layout = id.clone();
                    let sh = l.s("sheet").to_string();
                    if !sh.is_empty() {
                        app.cur_sheet = sh;
                    }
                    app.sel = Sel::Layout(id.clone());
                    app.tab = Tab::Layout;
                    app.view.fit_pending = true;
                }
                r.context_menu(|ui| {
                    if ui.button("Niveau de départ").clicked() {
                        app.snapshot();
                        app.proj["settings"]["startLayout"] = Json::String(id.clone());
                        app.changed();
                        ui.close_menu();
                    }
                    if ui.button("Dupliquer").clicked() {
                        app.snapshot();
                        let mut c = l.clone();
                        c["id"] = Json::String(pf_core::uid("l"));
                        c["name"] = Json::String(format!("{} (copie)", l.s("name")));
                        if let Some(a) = c.get_mut("instances").and_then(|v| v.as_array_mut()) {
                            for i in a {
                                i["id"] = Json::String(pf_core::uid("i"));
                            }
                        }
                        app.proj["layouts"].as_array_mut().unwrap().push(c);
                        app.changed();
                        ui.close_menu();
                    }
                    if ui.add_enabled(app.proj.arr("layouts").len() > 1, egui::Button::new("Supprimer")).clicked() {
                        app.snapshot();
                        app.proj["layouts"].as_array_mut().unwrap().retain(|x| x.s("id") != id);
                        app.fix_current();
                        app.changed();
                        ui.close_menu();
                    }
                });
            }
            // ---- feuilles
            ui.horizontal(|ui| {
                theme::section(ui, "Feuilles d’événements");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::btn_small("+")).on_hover_text("Nouvelle feuille (à inclure ou à associer à un niveau)").clicked() {
                        app.snapshot();
                        let sid = pf_core::uid("s");
                        let n = app.proj.arr("sheets").len() + 1;
                        app.proj["sheets"].as_array_mut().map(|a| a.push(json!({"id":sid,"name":format!("Feuille {}", n),"events":[]})));
                        app.cur_sheet = sid.clone();
                        app.sel = Sel::Sheet(sid);
                        app.tab = Tab::Events;
                        app.changed();
                    }
                });
            });
            for s in app.proj.arr("sheets").to_vec() {
                let id = s.s("id").to_string();
                let on = id == app.cur_sheet && app.tab == Tab::Events;
                let n = count_events(s.arr("events"));
                let r = ui.add(egui::SelectableLabel::new(on, egui::RichText::new(format!("☰  {}   ", s.s("name"))).color(if on { C::TEXT() } else { C::TEXT2() }))).on_hover_text(format!("{} événement(s)", n));
                if r.clicked() {
                    app.cur_sheet = id.clone();
                    app.sel = Sel::Sheet(id.clone());
                    app.tab = Tab::Events;
                }
            }
            // ---- objets
            ui.horizontal(|ui| {
                theme::section(ui, "Objets");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button(egui::RichText::new("+").size(12.0), |ui| {
                        for (k, n, g) in KINDS {
                            if ui.button(format!("{}  {}", g, n)).clicked() {
                                app.snapshot();
                                let o = make_object(app, k);
                                let id = o.s("id").to_string();
                                app.proj["objects"].as_array_mut().unwrap().push(o);
                                app.sel = Sel::Obj(id);
                                app.changed();
                                ui.close_menu();
                            }
                        }
                    });
                });
            });
            ui.add(egui::TextEdit::singleline(&mut app.tree.filter).hint_text("🔍 Filtrer les objets").desired_width(f32::INFINITY));
            let filt = app.tree.filter.to_lowercase();
            for o in app.proj.arr("objects").to_vec() {
                if !filt.is_empty() && !o.s("name").to_lowercase().contains(&filt) {
                    continue;
                }
                let id = o.s("id").to_string();
                let on = app.sel == Sel::Obj(id.clone());
                let txt = egui::RichText::new(format!("{}  {}", glyph(o.s("kind")), o.s("name"))).color(if on { C::TEXT() } else { C::TEXT2() });
                let r = ui.dnd_drag_source(egui::Id::new(("obj", &id)), id.clone(), |ui| {
                    ui.add(egui::SelectableLabel::new(on, txt));
                });
                let resp = r.response;
                if resp.clicked() {
                    app.sel = Sel::Obj(id.clone());
                }
                if resp.double_clicked() && o.s("kind") == "sprite" {
                    app.sprite = Some(crate::sprite::SpriteEd::open(&o));
                }
                if resp.double_clicked() && o.s("kind") == "particles" {
                    app.part_ed = crate::particles::PartEd::open(app, &id);
                }
                let n_inst = app.layout().map(|l| l.arr("instances").iter().filter(|i| i.s("obj") == id).count()).unwrap_or(0);
                let resp = resp.on_hover_text(format!("{} — {} instance(s) dans ce niveau\nGlissez-le dans le niveau pour en placer une.{}", kind_name(o.s("kind")), n_inst, match o.s("kind") { "sprite" => "\nDouble-clic : éditeur de sprites.", "particles" => "\nDouble-clic : éditeur de particules.", _ => "" }));
                resp.context_menu(|ui| {
                    if ui.button("Placer au centre de la vue").clicked() {
                        let c = app.view.center_world();
                        add_instance(app, &id, c.0, c.1);
                        ui.close_menu();
                    }
                    if ui.button("Dupliquer l’objet").clicked() {
                        app.snapshot();
                        let mut c = o.clone();
                        c["id"] = Json::String(pf_core::uid("o"));
                        c["name"] = Json::String(unique_name(app, o.s("name")));
                        app.proj["objects"].as_array_mut().unwrap().push(c);
                        app.changed();
                        ui.close_menu();
                    }
                    if ui.button("Supprimer l’objet (et ses instances)").clicked() {
                        app.snapshot();
                        app.proj["objects"].as_array_mut().unwrap().retain(|x| x.s("id") != id);
                        if let Some(ls) = app.proj.get_mut("layouts").and_then(|l| l.as_array_mut()) {
                            for l in ls {
                                if let Some(a) = l.get_mut("instances").and_then(|v| v.as_array_mut()) {
                                    a.retain(|i| i.s("obj") != id);
                                }
                            }
                        }
                        app.sel = Sel::None;
                        app.changed();
                        app.toast("Objet supprimé (Ctrl+Z pour annuler).", false, now);
                        ui.close_menu();
                    }
                });
            }
            // ---- cinématiques
            ui.horizontal(|ui| {
                theme::section(ui, "Cinématiques");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::btn_small("+")).on_hover_text("Nouvelle cinématique").clicked() {
                        crate::more::new_cine(app);
                    }
                });
            });
            for c in app.proj.arr("cines").to_vec() {
                let on = app.cine_ed.as_deref() == Some(c.s("id"));
                if ui.add(egui::SelectableLabel::new(on, format!("🎬  {}", c.s("name")))).clicked() {
                    app.cine_ed = Some(c.s("id").to_string());
                }
            }
            // ---- sons
            ui.horizontal(|ui| {
                theme::section(ui, "Sons");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::btn_small("☰")).on_hover_text("Bibliothèque de sons : écouter, rogner, importer plusieurs fichiers…").clicked() {
                        app.snd_lib = Some(Default::default());
                    }
                    if ui.add(theme::btn_small("✦")).on_hover_text("Créer un effet sonore (générateur)").clicked() {
                        app.sfx_ed = Some(crate::more::SfxEd::new(app, None));
                    }
                    if ui.add(theme::btn_small("+")).on_hover_text("Importer un fichier audio (wav, mp3, ogg)").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("Audio", &["wav", "mp3", "ogg", "flac", "m4a"]).pick_file() {
                            if let Ok(b) = std::fs::read(&p) {
                                let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                                let mime = match ext.as_str() {
                                    "mp3" => "audio/mpeg",
                                    "ogg" => "audio/ogg",
                                    "flac" => "audio/flac",
                                    "m4a" => "audio/mp4",
                                    _ => "audio/wav",
                                };
                                app.snapshot();
                                let name = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "son".into());
                                if app.proj.get("sounds").is_none() {
                                    app.proj["sounds"] = json!([]);
                                }
                                app.proj["sounds"].as_array_mut().unwrap().push(json!({"id":pf_core::uid("snd"),"name":name,"data":format!("data:{};base64,{}", mime, pf_core::b64_encode(&b))}));
                                app.changed();
                            }
                        }
                    }
                });
            });
            for s in app.proj.arr("sounds").to_vec() {
                let id = s.s("id").to_string();
                let on = app.sel == Sel::Sound(id.clone());
                if ui.add(egui::SelectableLabel::new(on, format!("♪  {}", s.s("name")))).clicked() {
                    app.sel = Sel::Sound(id);
                }
            }
            // ---- variables globales
            ui.horizontal(|ui| {
                theme::section(ui, "Variables globales");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::btn_small("+")).clicked() {
                        app.snapshot();
                        let mut k = 1;
                        let names: Vec<String> = app.proj.arr("globals").iter().map(|g| g.s("name").to_string()).collect();
                        while names.contains(&format!("Variable{}", k)) {
                            k += 1;
                        }
                        let n = format!("Variable{}", k);
                        if app.proj.get("globals").is_none() {
                            app.proj["globals"] = json!([]);
                        }
                        app.proj["globals"].as_array_mut().unwrap().push(json!({"name":n,"type":"number","value":0}));
                        app.sel = Sel::Global(n);
                        app.changed();
                    }
                });
            });
            for g in app.proj.arr("globals").to_vec() {
                let n = g.s("name").to_string();
                let on = app.sel == Sel::Global(n.clone());
                let v = crate::props::jdisp(g.get("value").unwrap_or(&Json::Null));
                if ui.add(egui::SelectableLabel::new(on, egui::RichText::new(format!("𝑥  {} = {}", n, v)))).clicked() {
                    app.sel = Sel::Global(n);
                }
            }
            ui.add_space(10.0);
            if ui.add(egui::SelectableLabel::new(false, egui::RichText::new("⚙  Paramètres du jeu").color(C::TEXT2()))).clicked() {
                app.sel = Sel::None;
            }
        });
    });
}

pub fn count_events(l: &[Json]) -> usize {
    l.iter().map(|e| 1 + count_events(e.arr("subs"))).sum()
}
