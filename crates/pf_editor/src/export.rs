//! Export du jeu : page web unique (HTML), jeu web installable (PWA, zip) ou application de bureau.
//! Les jeux web utilisent le moteur JavaScript d'origine (runtime.js), identique au moteur natif.

use crate::theme::{self, C};
use crate::App;
use eframe::egui::{self, RichText};
use pf_core::{json, Json, JsonExt};
use pf_runtime::gfx::{parse_color, Align, Baseline, FontSpec, Gfx};

const RUNTIME_JS: &str = include_str!("../assets/runtime.js");

pub struct ExportState {
    pub format: String,
    pub o: Json,
}

impl ExportState {
    pub fn new(app: &App) -> ExportState {
        let s = app.proj.get("settings").cloned().unwrap_or(json!({}));
        let mut o = json!({"format":"html","touch":true,"splash":true,"bg":"#0d0f13","accent":"#ff8a3d","showTitle":true,"logo":"","icon":""});
        if let Some(e) = s.get("export").and_then(|e| e.as_object()) {
            for (k, v) in e {
                o[k] = v.clone();
            }
        }
        o["title"] = Json::String(if s.s("title").is_empty() { crate::i18n::tr("Mon jeu").into() } else { s.s("title").to_string() });
        let format = o.s("format").to_string();
        ExportState { format: if format.is_empty() { "html".into() } else { format }, o }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn png(g: &Gfx) -> Vec<u8> {
    let rgba = pf_runtime::img::to_rgba(&g.pm);
    let mut out = Vec::new();
    use image::ImageEncoder;
    let _ = image::codecs::png::PngEncoder::new(&mut out).write_image(&rgba, g.width(), g.height(), image::ExtendedColorType::Rgba8);
    out
}

/// icône carrée : sprite choisi sur le fond, sinon l'initiale du titre
fn icon(app: &App, ex: &Json, size: u32) -> Vec<u8> {
    let mut g = Gfx::new(size, size);
    let sz = size as f32;
    g.fill_rect(0.0, 0.0, sz, sz, parse_color(if ex.s("bg").is_empty() { "#0d0f13" } else { ex.s("bg") }));
    let im = app.obj(ex.s("icon")).and_then(|o| o.arr("anims").first().and_then(|a| a.arr("frames").first()).and_then(|f| f.as_str())).and_then(|u| pf_core::data_url_bytes(u)).and_then(|b| pf_runtime::img::decode(&b));
    match im {
        Some(im) => {
            g.set_smooth(false);
            let (w, h) = (im.width() as f32, im.height() as f32);
            let mut k = (sz * 0.72 / w.max(h)).floor();
            if k < 1.0 {
                k = sz * 0.72 / w.max(h);
            }
            g.draw_image(&im, (sz - w * k) / 2.0, (sz - h * k) / 2.0, w * k, h * k);
        }
        None => {
            g.fill_rr(sz * 0.1, sz * 0.1, sz * 0.8, sz * 0.8, sz * 0.18, parse_color(if ex.s("accent").is_empty() { "#ff8a3d" } else { ex.s("accent") }));
            let t = ex.s("title").trim().chars().next().unwrap_or('J').to_uppercase().to_string();
            g.fill_text(&t, sz / 2.0, sz * 0.54, &FontSpec::new(800, (sz * 0.5) as f64, "sans"), Align::Center, Baseline::Middle, parse_color("#ffffff"));
        }
    }
    png(&g)
}

pub fn build_html(app: &App, ex: &Json, pwa: bool) -> String {
    let mut data = app.proj.clone();
    data["settings"]["title"] = Json::String(ex.s("title").to_string());
    data["settings"]["export"] = json!({"bg":ex.s("bg"),"accent":ex.s("accent"),"showTitle":ex.get("showTitle") != Some(&Json::Bool(false)),"logo":ex.s("logo")});
    let js = serde_json::to_string(&data).unwrap_or_default().replace('<', "\\u003c");
    let ico = format!("data:image/png;base64,{}", pf_core::b64_encode(&icon(app, ex, 64)));
    let bg = esc(if ex.s("bg").is_empty() { "#0d0f13" } else { ex.s("bg") });
    let head = crate::i18n::trf("<link rel=\"icon\" href=\"{0}\">\n{1}<meta name=\"theme-color\" content=\"{2}\">\n", &[format!("{}", if pwa { "icon-192.png".to_string() } else { ico }), format!("{}", if pwa { "<link rel=\"manifest\" href=\"manifest.webmanifest\">\n<link rel=\"apple-touch-icon\" href=\"icon-192.png\">\n<meta name=\"apple-mobile-web-app-capable\" content=\"yes\">\n" } else { "" }), format!("{}", bg)]);
    let b = |k: &str| if pf_runtime::rt::jtruthy(ex.get(k)) { "true" } else { "false" };
    crate::i18n::trf("<!DOCTYPE html>\n<html lang=\"fr\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no\">\n<meta name=\"generator\" content=\"PlatForge Studio (Rust)\">\n<title>{0}</title>\n{1}<style>html,body{{margin:0;height:100%;background:{2};overflow:hidden;-webkit-user-select:none;user-select:none;-webkit-tap-highlight-color:transparent}}#game{{position:fixed;inset:0;width:100%;height:100%;display:block;touch-action:none;outline:none}}</style>\n</head>\n<body>\n<canvas id=\"game\" tabindex=\"0\"></canvas>\n<script type=\"application/json\" id=\"platforge-data\">{3}</script>\n<script>{4}</script>\n<script>PF.Runtime(document.getElementById(\"game\"),JSON.parse(document.getElementById(\"platforge-data\").textContent),{{touch:{5},splash:{6}}});document.getElementById(\"game\").focus();{7}</script>\n</body>\n</html>\n", &[format!("{}", esc(ex.s("title"))), format!("{}", head), format!("{}", bg), format!("{}", js), format!("{}", RUNTIME_JS), format!("{}", b("touch")), format!("{}", b("splash")), format!("{}", if pwa { "if(\"serviceWorker\" in navigator&&location.protocol!==\"file:\")navigator.serviceWorker.register(\"sw.js\").catch(function(){});" } else { "" })])
}

// ---- zip (stockage simple, sans compression)
fn crc32(d: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in d {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}
pub fn make_zip(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let off = out.len() as u32;
        let crc = crc32(data);
        let n = name.as_bytes();
        let mut h = Vec::new();
        h.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        h.extend_from_slice(&20u16.to_le_bytes());
        h.extend_from_slice(&0x0800u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0x5821u16.to_le_bytes());
        h.extend_from_slice(&crc.to_le_bytes());
        h.extend_from_slice(&(data.len() as u32).to_le_bytes());
        h.extend_from_slice(&(data.len() as u32).to_le_bytes());
        h.extend_from_slice(&(n.len() as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&h);
        out.extend_from_slice(n);
        out.extend_from_slice(data);
        let mut c = Vec::new();
        c.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&0x0800u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&0x5821u16.to_le_bytes());
        c.extend_from_slice(&crc.to_le_bytes());
        c.extend_from_slice(&(data.len() as u32).to_le_bytes());
        c.extend_from_slice(&(data.len() as u32).to_le_bytes());
        c.extend_from_slice(&(n.len() as u16).to_le_bytes());
        c.extend_from_slice(&[0u8; 12]);
        c.extend_from_slice(&off.to_le_bytes());
        c.extend_from_slice(n);
        central.extend_from_slice(&c);
    }
    let coff = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&coff.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn do_export(app: &mut App, st: &ExportState, now: f64) {
    let ex = &st.o;
    let name = crate::util::slug(ex.s("title"));
    let r: Result<String, String> = match st.format.as_str() {
        "html" => match rfd::FileDialog::new().add_filter(crate::i18n::tr("Page web"), &["html"]).set_file_name(format!("{}.html", name)).save_file() {
            Some(p) => std::fs::write(&p, build_html(app, ex, false)).map(|_| p.display().to_string()).map_err(|e| e.to_string()),
            None => return,
        },
        "pwa" => match rfd::FileDialog::new().add_filter(crate::i18n::tr("Archive zip"), &["zip"]).set_file_name(format!("{}-web.zip", name)).save_file() {
            Some(p) => {
                let man = json!({"name":ex.s("title"),"short_name":ex.s("title").chars().take(12).collect::<String>(),"start_url":".","scope":".","display":"fullscreen","orientation":"landscape","background_color":ex.s("bg"),"theme_color":ex.s("bg"),"icons":[{"src":"icon-192.png","sizes":"192x192","type":"image/png"},{"src":"icon-512.png","sizes":"512x512","type":"image/png","purpose":"any maskable"}]});
                let ver = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0));
                let sw = crate::i18n::trf("var C='{0}-{1}',F=['./','index.html','manifest.webmanifest','icon-192.png','icon-512.png'];\nself.addEventListener('install',function(e){{e.waitUntil(caches.open(C).then(function(c){{return c.addAll(F);}}).then(function(){{return self.skipWaiting();}}));}});\nself.addEventListener('activate',function(e){{e.waitUntil(caches.keys().then(function(k){{return Promise.all(k.filter(function(x){{return x!==C;}}).map(function(x){{return caches.delete(x);}}));}}).then(function(){{return self.clients.claim();}}));}});\nself.addEventListener('fetch',function(e){{e.respondWith(caches.match(e.request).then(function(r){{return r||fetch(e.request);}}));}});\n", &[format!("{}", name), format!("{}", ver)]);
                let readme = crate::i18n::trf("« {0} » — jeu créé avec PlatForge Studio\n\nCe dossier est un jeu web installable (PWA).\n\n• Pour jouer tout de suite : ouvrez index.html dans un navigateur.\n• Pour le publier : envoyez TOUS les fichiers sur un hébergement web en HTTPS\n  (itch.io en « HTML », GitHub Pages, Netlify, votre site…).\n  Une fois en ligne, le navigateur propose « Installer » : le jeu s’ouvre alors\n  comme une application et fonctionne même sans connexion.\n", &[format!("{}", ex.s("title"))]);
                let files = vec![
                    ("index.html".to_string(), build_html(app, ex, true).into_bytes()),
                    ("manifest.webmanifest".to_string(), serde_json::to_string_pretty(&man).unwrap_or_default().into_bytes()),
                    ("sw.js".to_string(), sw.into_bytes()),
                    ("icon-192.png".to_string(), icon(app, ex, 192)),
                    ("icon-512.png".to_string(), icon(app, ex, 512)),
                    (crate::i18n::tr("LISEZMOI.txt").to_string(), readme.into_bytes()),
                ];
                std::fs::write(&p, make_zip(&files)).map(|_| p.display().to_string()).map_err(|e| e.to_string())
            }
            None => return,
        },
        _ => {
            app.export_game(now);
            return;
        }
    };
    match r {
        Ok(p) => app.toast(crate::i18n::trf("Jeu exporté : {0}", &[format!("{}", p)]), false, now),
        Err(e) => app.toast(crate::i18n::trf("Export impossible : {0}", &[format!("{}", e)]), true, now),
    }
}

pub fn window(app: &mut App, ctx: &egui::Context, now: f64) {
    let mut st = match app.export.take() {
        Some(s) => s,
        None => return,
    };
    let mut open = true;
    let mut go = false;
    let (dsz, msz) = theme::fit(ctx, 520.0, 560.0);
    egui::Window::new(crate::i18n::tr("Exporter le jeu")).id(egui::Id::new("export")).open(&mut open).collapsible(false).default_width(dsz.x).max_size(msz).vscroll(true).constrain(true).show(ctx, |ui| {
        for (k, n, d) in [
            ("html", crate::i18n::tr("Page web (un seul fichier .html)"), crate::i18n::tr("Le plus simple : un fichier à ouvrir dans n’importe quel navigateur, à envoyer par mail ou à déposer sur itch.io.")),
            ("pwa", crate::i18n::tr("Jeu web installable (PWA, .zip)"), crate::i18n::tr("À mettre en ligne (itch.io, GitHub Pages, Netlify…) : le navigateur propose « Installer » et le jeu marche hors connexion, aussi sur mobile.")),
            ("native", crate::i18n::tr("Application de bureau"), crate::i18n::tr("Une copie de PlatForge qui lance directement votre jeu, pour ce système (Windows, macOS ou Linux).")),
        ] {
            ui.radio_value(&mut st.format, k.to_string(), RichText::new(n).strong());
            ui.label(RichText::new(d).size(11.0).color(C::DIM()));
            ui.add_space(4.0);
        }
        ui.separator();
        if st.format != "native" {
            egui::Grid::new("exg").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(RichText::new(crate::i18n::tr("Titre")).color(C::MUTED()));
                let mut t = st.o.s("title").to_string();
                if ui.add(egui::TextEdit::singleline(&mut t).desired_width(240.0)).changed() {
                    st.o["title"] = Json::String(t);
                }
                ui.end_row();
                for (k, l) in [("bg", crate::i18n::tr("Couleur de fond")), ("accent", crate::i18n::tr("Couleur d’accent"))] {
                    ui.label(RichText::new(l).color(C::MUTED()));
                    let mut c = theme::color32(st.o.s(k));
                    if ui.color_edit_button_srgba(&mut c).changed() {
                        st.o[k] = Json::String(format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()));
                    }
                    ui.end_row();
                }
                for (k, l) in [("splash", crate::i18n::tr("Écran « Cliquez pour jouer » (nécessaire pour le son sur le web)")), ("touch", crate::i18n::tr("Boutons tactiles sur mobile")), ("showTitle", crate::i18n::tr("Afficher le titre au chargement"))] {
                    ui.label(RichText::new(l).color(C::MUTED()));
                    let mut b = st.o.get(k) != Some(&Json::Bool(false));
                    if ui.checkbox(&mut b, "").changed() {
                        st.o[k] = Json::Bool(b);
                    }
                    ui.end_row();
                }
                for (k, l) in [("logo", crate::i18n::tr("Logo de l’écran de chargement")), ("icon", crate::i18n::tr("Icône du jeu"))] {
                    ui.label(RichText::new(l).color(C::MUTED()));
                    let cur = st.o.s(k).to_string();
                    let nm = app.obj(&cur).map(|o| o.s("name").to_string()).unwrap_or_else(|| crate::i18n::tr("(aucun)").into());
                    egui::ComboBox::from_id_salt(k).selected_text(nm).show_ui(ui, |ui| {
                        if ui.selectable_label(cur.is_empty(), crate::i18n::tr("(aucun)")).clicked() {
                            st.o[k] = json!("");
                        }
                        for o in app.proj.arr("objects").iter().filter(|o| o.s("kind") == "sprite") {
                            if ui.selectable_label(o.s("id") == cur, o.s("name")).clicked() {
                                st.o[k] = Json::String(o.s("id").to_string());
                            }
                        }
                    });
                    ui.end_row();
                }
            });
        }
        ui.add_space(6.0);
        if ui.add(theme::btn_accent(crate::i18n::tr("Exporter…"))).clicked() {
            go = true;
        }
    });
    if go {
        // mémorise les réglages dans le projet (comme la version HTML)
        let mut e = st.o.clone();
        e["format"] = Json::String(st.format.clone());
        if let Some(m) = e.as_object_mut() {
            m.remove("title");
        }
        if app.proj.get("settings").and_then(|s| s.get("export")) != Some(&e) {
            app.snapshot();
            app.proj["settings"]["export"] = e;
            app.changed();
        }
        do_export(app, &st, now);
        return;
    }
    if open {
        app.export = Some(st);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn zip_ok() {
        let z = super::make_zip(&[("a.txt".into(), b"bonjour".to_vec()), ("dossier/b.bin".into(), vec![1, 2, 3])]);
        std::fs::write("/tmp/pf_test.zip", z).unwrap();
    }
}
