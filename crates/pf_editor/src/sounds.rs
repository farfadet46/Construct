//! Bibliothèque de sons : écoute, forme d'onde, durée, utilisation, import multiple,
//! rognage, volume, fondus, export.
use crate::more::Preview;
use crate::theme::{self, C};
use crate::App;
use eframe::egui::{self, Color32};
use pf_core::{json, Json, JsonExt};
use std::collections::HashMap;

/// son décodé (échantillons entrelacés)
pub struct Dec {
    key: usize,
    pub sr: u32,
    pub ch: u16,
    pub s: Vec<f32>,
    peaks: Vec<f32>,
}
impl Dec {
    fn secs(&self) -> f64 {
        self.s.len() as f64 / self.ch.max(1) as f64 / self.sr.max(1) as f64
    }
}

#[derive(Default)]
pub struct SndLib {
    cache: HashMap<String, Option<Dec>>,
    filter: String,
    /// son en cours de modification
    cur: Option<String>,
    a: f64,
    b: f64,
    gain: f64,
    fade_in: f64,
    fade_out: f64,
    drag: Option<u8>,
    confirm_del: Option<String>,
    playing: Option<String>,
}

fn decode(data: &str) -> Option<Dec> {
    let b = pf_core::data_url_bytes(data)?;
    let d = rodio::Decoder::new(std::io::Cursor::new(b)).ok()?;
    use rodio::Source;
    let (sr, ch) = (d.sample_rate(), d.channels());
    let s: Vec<f32> = d.map(|x: i16| x as f32 / 32768.0).collect();
    let n = 300usize;
    let fr = s.len() / ch.max(1) as usize;
    let mut peaks = vec![0f32; n];
    if fr > 0 {
        for (k, p) in peaks.iter_mut().enumerate() {
            let (i0, i1) = (k * fr / n, ((k + 1) * fr / n).max(k * fr / n + 1).min(fr));
            for i in i0..i1 {
                for c in 0..ch as usize {
                    *p = p.max(s[i * ch as usize + c].abs());
                }
            }
        }
    }
    Some(Dec { key: data.len(), sr, ch, s, peaks })
}

/// WAV 16 bits, n canaux
fn wav(s: &[f32], ch: u16, sr: u32) -> Vec<u8> {
    let n = s.len() as u32;
    let mut b: Vec<u8> = Vec::with_capacity(44 + s.len() * 2);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + n * 2).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&ch.to_le_bytes());
    b.extend_from_slice(&sr.to_le_bytes());
    b.extend_from_slice(&(sr * ch as u32 * 2).to_le_bytes());
    b.extend_from_slice(&(ch * 2).to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(n * 2).to_le_bytes());
    for x in s {
        b.extend_from_slice(&((x.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    b
}

/// nombre de références au son dans les événements et cinématiques
fn uses(app: &App, id: &str) -> usize {
    fn walk(v: &Json, id: &str) -> usize {
        match v {
            Json::String(s) => (s == id) as usize,
            Json::Array(a) => a.iter().map(|x| walk(x, id)).sum(),
            Json::Object(m) => m.values().map(|x| walk(x, id)).sum(),
            _ => 0,
        }
    }
    walk(app.proj.get("sheets").unwrap_or(&Json::Null), id) + walk(app.proj.get("cines").unwrap_or(&Json::Null), id)
}

fn fmt_t(t: f64) -> String {
    if t < 60.0 {
        format!("{:.2} s", t)
    } else {
        format!("{}:{:04.1}", (t / 60.0) as u32, t % 60.0)
    }
}

fn mime_of(data: &str) -> &str {
    data.strip_prefix("data:").and_then(|s| s.split(';').next()).unwrap_or("")
}

fn wave(ui: &mut egui::Ui, peaks: &[f32], size: egui::Vec2, col: Color32, sel: Option<(f64, f64)>) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let p = ui.painter_at(r);
    p.rect_filled(r, 4.0, C::BG0());
    if let Some((a, b)) = sel {
        let x0 = r.left() + r.width() * a as f32;
        let x1 = r.left() + r.width() * b as f32;
        p.rect_filled(egui::Rect::from_min_max(egui::pos2(x0, r.top()), egui::pos2(x1, r.bottom())), 0.0, Color32::from_rgba_unmultiplied(255, 138, 61, 28));
    }
    let n = peaks.len().max(1);
    let mid = r.center().y;
    for (k, v) in peaks.iter().enumerate() {
        let x = r.left() + (k as f32 + 0.5) * r.width() / n as f32;
        let h = (v * r.height() * 0.48).max(0.5);
        let inside = sel.map(|(a, b)| { let f = k as f64 / n as f64; f >= a && f <= b }).unwrap_or(true);
        p.line_segment([egui::pos2(x, mid - h), egui::pos2(x, mid + h)], egui::Stroke::new((r.width() / n as f32).max(1.0), if inside { col } else { C::DIM() }));
    }
    resp
}

impl SndLib {
    fn dec<'a>(&'a mut self, id: &str, data: &str) -> Option<&'a Dec> {
        let fresh = matches!(self.cache.get(id), Some(Some(d)) if d.key == data.len()) || matches!(self.cache.get(id), Some(None));
        if !fresh {
            self.cache.insert(id.to_string(), decode(data));
        }
        self.cache.get(id).and_then(|d| d.as_ref())
    }
    fn edit(&mut self, id: &str) {
        self.cur = Some(id.to_string());
        self.a = 0.0;
        self.b = 1.0;
        self.gain = 100.0;
        self.fade_in = 0.0;
        self.fade_out = 0.0;
    }
    /// échantillons après rognage, volume et fondus
    fn processed(&self, d: &Dec) -> Vec<f32> {
        let ch = d.ch.max(1) as usize;
        let fr = d.s.len() / ch;
        let (i0, i1) = (((self.a * fr as f64) as usize).min(fr), ((self.b * fr as f64) as usize).min(fr));
        let g = (self.gain / 100.0) as f32;
        let n = i1.saturating_sub(i0);
        let fi = (self.fade_in * d.sr as f64) as usize;
        let fo = (self.fade_out * d.sr as f64) as usize;
        let mut out = Vec::with_capacity(n * ch);
        for i in 0..n {
            let mut k = g;
            if fi > 0 && i < fi {
                k *= i as f32 / fi as f32;
            }
            if fo > 0 && n - i < fo {
                k *= (n - i) as f32 / fo as f32;
            }
            for c in 0..ch {
                out.push(d.s[(i0 + i) * ch + c] * k);
            }
        }
        out
    }
}

fn import_files(app: &mut App) {
    let Some(ps) = rfd::FileDialog::new().add_filter("Audio", &["wav", "mp3", "ogg", "flac", "m4a"]).pick_files() else { return };
    let mut added = 0;
    app.snapshot();
    for p in ps {
        let Ok(b) = std::fs::read(&p) else { continue };
        let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let mime = match ext.as_str() {
            "mp3" => "audio/mpeg",
            "ogg" => "audio/ogg",
            "flac" => "audio/flac",
            "m4a" => "audio/mp4",
            _ => "audio/wav",
        };
        let base = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "son".into());
        let names: Vec<String> = app.proj.arr("sounds").iter().map(|s| s.s("name").to_string()).collect();
        let mut name = base.clone();
        let mut k = 2;
        while names.contains(&name) {
            name = format!("{base}{k}");
            k += 1;
        }
        if !app.proj.get("sounds").map(|s| s.is_array()).unwrap_or(false) {
            app.proj["sounds"] = json!([]);
        }
        app.proj["sounds"].as_array_mut().unwrap().push(json!({"id":pf_core::uid("snd"),"name":name,"data":format!("data:{};base64,{}", mime, pf_core::b64_encode(&b))}));
        added += 1;
    }
    if added > 0 {
        app.changed();
    }
}

pub fn window(app: &mut App, ctx: &egui::Context, now: f64) {
    let mut lib = match app.snd_lib.take() {
        Some(l) => l,
        None => return,
    };
    let mut open = true;
    let (dsz, msz) = theme::fit(ctx, 760.0, 600.0);
    let sounds: Vec<Json> = app.proj.arr("sounds").to_vec();
    if !Preview::playing(app) {
        lib.playing = None;
    }
    egui::Window::new("Bibliothèque de sons").id(egui::Id::new("sndlib")).open(&mut open).default_size(dsz).max_size(msz).min_size([480.0, 300.0]).resizable(true).constrain(true).show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.add(theme::btn_accent("+ Importer…")).on_hover_text("Plusieurs fichiers à la fois (wav, mp3, ogg, flac, m4a)").clicked() {
                import_files(app);
            }
            if ui.add(theme::btn("✦ Nouvel effet")).on_hover_text("Générateur d’effets sonores").clicked() {
                app.sfx_ed = Some(crate::more::SfxEd::new(app, None));
            }
            ui.add(egui::TextEdit::singleline(&mut lib.filter).hint_text("Filtrer…").desired_width(140.0));
            let total: usize = sounds.iter().map(|s| s.s("data").len() * 3 / 4).sum();
            ui.label(egui::RichText::new(format!("{} son(s) · {:.1} Mo", sounds.len(), total as f64 / 1048576.0)).color(C::DIM()));
        });
        ui.separator();
        let list_h = if lib.cur.is_some() { (ui.available_height() - 230.0).max(120.0) } else { ui.available_height() };
        egui::ScrollArea::vertical().id_salt("sndl").max_height(list_h).auto_shrink([false, true]).show(ui, |ui| {
            if sounds.is_empty() {
                ui.label(egui::RichText::new("Aucun son. Importez des fichiers ou créez un effet.").color(C::DIM()));
            }
            let f = lib.filter.to_lowercase();
            for s in &sounds {
                let id = s.s("id").to_string();
                if !f.is_empty() && !s.s("name").to_lowercase().contains(&f) {
                    continue;
                }
                let n_use = uses(app, &id);
                let sel = lib.cur.as_deref() == Some(&id);
                egui::Frame::new().fill(if sel { C::BG3() } else { C::BG2() }).corner_radius(6.0).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let is_play = lib.playing.as_deref() == Some(&id);
                        if ui.add(theme::btn_small(if is_play { "■" } else { "▶" })).on_hover_text(if is_play { "Arrêter" } else { "Écouter" }).clicked() {
                            if is_play {
                                Preview::stop(app);
                                lib.playing = None;
                            } else if let Some(b) = pf_core::data_url_bytes(s.s("data")) {
                                Preview::play_bytes(app, b);
                                lib.playing = Some(id.clone());
                            }
                        }
                        let mut nm = s.s("name").to_string();
                        if ui.add(egui::TextEdit::singleline(&mut nm).desired_width(150.0)).changed() && !nm.trim().is_empty() {
                            if let Some(x) = app.proj["sounds"].as_array_mut().and_then(|a| a.iter_mut().find(|x| x.s("id") == id)) {
                                x["name"] = json!(nm);
                            }
                            app.changed();
                        }
                        let info = lib.dec(&id, s.s("data")).map(|d| (d.peaks.clone(), d.secs(), d.ch, d.sr));
                        match &info {
                            Some((pk, _, _, _)) => {
                                let r = wave(ui, pk, egui::vec2(170.0, 26.0), if s.get("sfx").map(|x| x.is_object()).unwrap_or(false) { C::PURPLE() } else { C::ACCENT() }, None);
                                if r.clicked() {
                                    lib.edit(&id);
                                }
                            }
                            None => {
                                ui.label(egui::RichText::new("(illisible)").color(C::DANGER()));
                            }
                        }
                        ui.vertical(|ui| {
                            let fmt = mime_of(s.s("data")).trim_start_matches("audio/").replace("mpeg", "mp3").replace("x-wav", "wav");
                            let kb = s.s("data").len() * 3 / 4 / 1024;
                            if let Some((_, t, ch, sr)) = info {
                                ui.label(egui::RichText::new(format!("{} · {} · {} Ko", fmt_t(t), fmt, kb)).size(11.0).color(C::MUTED())).on_hover_text(format!("{} Hz, {}", sr, if ch == 1 { "mono".to_string() } else { format!("{ch} canaux") }));
                            }
                            ui.label(egui::RichText::new(if n_use == 0 { "inutilisé".to_string() } else { format!("utilisé {}×", n_use) }).size(11.0).color(if n_use == 0 { C::DIM() } else { C::COND() }));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(theme::btn_small("✕")).on_hover_text("Supprimer").clicked() {
                                lib.confirm_del = Some(id.clone());
                            }
                            if ui.add(theme::btn_small("⬇")).on_hover_text("Enregistrer le fichier sur l’ordinateur").clicked() {
                                let ext = match mime_of(s.s("data")) { "audio/mpeg" => "mp3", "audio/ogg" => "ogg", "audio/flac" => "flac", "audio/mp4" => "m4a", _ => "wav" };
                                if let (Some(p), Some(b)) = (rfd::FileDialog::new().set_file_name(format!("{}.{}", s.s("name"), ext)).save_file(), pf_core::data_url_bytes(s.s("data"))) {
                                    let _ = std::fs::write(p, b);
                                }
                            }
                            if s.get("sfx").map(|x| x.is_object()).unwrap_or(false) && ui.add(theme::btn_small("✦")).on_hover_text("Modifier l’effet dans le générateur").clicked() {
                                app.sfx_ed = Some(crate::more::SfxEd::new(app, Some(id.clone())));
                            }
                            if ui.add(theme::btn_small("✂")).on_hover_text("Rogner, volume, fondus").clicked() {
                                if sel {
                                    lib.cur = None;
                                } else {
                                    lib.edit(&id);
                                }
                            }
                        });
                    });
                });
                ui.add_space(3.0);
            }
        });
        // ---- modification du son choisi
        let cur = lib.cur.clone().and_then(|id| sounds.iter().find(|s| s.s("id") == id).cloned());
        if let Some(s) = cur {
            let id = s.s("id").to_string();
            ui.separator();
            let Some((pk, secs, sr, ch)) = lib.dec(&id, s.s("data")).map(|d| (d.peaks.clone(), d.secs(), d.sr, d.ch)) else { return };
            ui.horizontal(|ui| {
                theme::section(ui, &format!("Modifier « {} »", s.s("name")));
                ui.label(egui::RichText::new(format!("garde {} sur {}", fmt_t((lib.b - lib.a) * secs), fmt_t(secs))).color(C::DIM()));
            });
            let w = ui.available_width().max(200.0);
            let r = wave(ui, &pk, egui::vec2(w, 90.0), C::ACCENT(), Some((lib.a, lib.b)));
            // poignées de début et de fin
            let fx = |f: f64| r.rect.left() + r.rect.width() * f as f32;
            for (k, f) in [(0u8, lib.a), (1, lib.b)] {
                let x = fx(f);
                ui.painter().line_segment([egui::pos2(x, r.rect.top()), egui::pos2(x, r.rect.bottom())], egui::Stroke::new(2.0_f32, C::ACCENT()));
                ui.painter().rect_filled(egui::Rect::from_center_size(egui::pos2(x, r.rect.top() + 6.0), egui::vec2(10.0, 12.0)), 2.0, C::ACCENT());
                let _ = k;
            }
            if let Some(p) = r.interact_pointer_pos() {
                let f = ((p.x - r.rect.left()) / r.rect.width()).clamp(0.0, 1.0) as f64;
                if r.drag_started() {
                    lib.drag = Some(if (f - lib.a).abs() <= (f - lib.b).abs() { 0 } else { 1 });
                }
                match lib.drag {
                    Some(0) => lib.a = f.min(lib.b - 0.002),
                    Some(_) => lib.b = f.max(lib.a + 0.002),
                    None => {}
                }
            }
            if r.drag_stopped() {
                lib.drag = None;
            }
            r.on_hover_text("Glissez les poignées pour garder seulement une partie du son");
            egui::Grid::new("sndfx").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(egui::RichText::new("Volume").color(C::MUTED()));
                ui.add(egui::Slider::new(&mut lib.gain, 0.0..=400.0).suffix(" %"));
                ui.label(egui::RichText::new("Fondu d’entrée").color(C::MUTED()));
                ui.add(egui::Slider::new(&mut lib.fade_in, 0.0..=(secs * (lib.b - lib.a)).max(0.01)).suffix(" s").step_by(0.01));
                ui.end_row();
                if ui.add(theme::btn_small("Normaliser")).on_hover_text("Règle le volume pour que le pic atteigne le maximum sans saturer").clicked() {
                    if let Some(d) = lib.cache.get(&id).and_then(|d| d.as_ref()) {
                        let ch = d.ch.max(1) as usize;
                        let fr = d.s.len() / ch;
                        let (i0, i1) = ((lib.a * fr as f64) as usize, ((lib.b * fr as f64) as usize).min(fr));
                        let pk = d.s[i0 * ch..i1 * ch].iter().fold(0f32, |m, x| m.max(x.abs()));
                        if pk > 0.0 {
                            lib.gain = (0.98 / pk as f64 * 100.0).round().min(400.0);
                        }
                    }
                }
                ui.label("");
                ui.label(egui::RichText::new("Fondu de sortie").color(C::MUTED()));
                ui.add(egui::Slider::new(&mut lib.fade_out, 0.0..=(secs * (lib.b - lib.a)).max(0.01)).suffix(" s").step_by(0.01));
                ui.end_row();
            });
            ui.horizontal(|ui| {
                if ui.add(theme::btn("▶ Écouter le résultat")).clicked() {
                    if let Some(d) = lib.cache.get(&id).and_then(|d| d.as_ref()) {
                        let out = lib.processed(d);
                        Preview::play_ch(app, out, ch, sr);
                    }
                }
                if ui.add(theme::btn_accent("Appliquer")).on_hover_text("Remplace le son du projet (enregistré en WAV)").clicked() {
                    if let Some(d) = lib.cache.get(&id).and_then(|d| d.as_ref()) {
                        let out = lib.processed(d);
                        let url = format!("data:audio/wav;base64,{}", pf_core::b64_encode(&wav(&out, ch, sr)));
                        app.snapshot();
                        if let Some(x) = app.proj["sounds"].as_array_mut().and_then(|a| a.iter_mut().find(|x| x.s("id") == id)) {
                            x["data"] = json!(url);
                            if let Some(m) = x.as_object_mut() {
                                m.remove("sfx");
                            }
                        }
                        app.changed();
                        lib.edit(&id);
                        app.toast("Son modifié (Ctrl+Z pour revenir en arrière).", false, now);
                    }
                }
                if ui.add(theme::btn("Annuler")).clicked() {
                    lib.cur = None;
                }
            });
        }
    });
    // confirmation de suppression
    if let Some(id) = lib.confirm_del.clone() {
        let n = uses(app, &id);
        let name = sounds.iter().find(|s| s.s("id") == id).map(|s| s.s("name").to_string()).unwrap_or_default();
        egui::Window::new("Supprimer le son ?").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(format!("« {} » sera retiré du projet.", name));
            if n > 0 {
                ui.label(egui::RichText::new(format!("Attention : il est utilisé {} fois dans les événements ou les cinématiques.", n)).color(C::DANGER()));
            }
            ui.horizontal(|ui| {
                if ui.add(theme::btn_accent("Supprimer")).clicked() {
                    app.snapshot();
                    if let Some(a) = app.proj["sounds"].as_array_mut() {
                        a.retain(|x| x.s("id") != id);
                    }
                    if lib.cur.as_deref() == Some(&id) {
                        lib.cur = None;
                    }
                    lib.cache.remove(&id);
                    app.changed();
                    lib.confirm_del = None;
                }
                if ui.add(theme::btn("Annuler")).clicked() {
                    lib.confirm_del = None;
                }
            });
        });
    }
    if open {
        app.snd_lib = Some(lib);
    } else {
        Preview::stop(app);
    }
}
