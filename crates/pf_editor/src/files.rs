//! Sécurité du travail : sauvegarde automatique, récupération après plantage,
//! projets récents, glisser-déposer de fichiers, confirmation avant de perdre des modifications.
use crate::i18n::{tr, trf};
use crate::theme::{self, C};
use crate::App;
use eframe::egui;
use pf_core::{json, Json, JsonExt};
use std::path::PathBuf;

const MAX_RECENT: usize = 10;

/// action en attente d'une confirmation (le projet a des modifications non enregistrées)
#[derive(Clone)]
pub enum Pending {
    Template(String),
    Open(Option<PathBuf>),
    Recover(PathBuf),
    Quit,
}

/// copie de secours laissée par une session qui ne s'est pas fermée normalement
pub struct Orphan {
    pub file: PathBuf,
    pub path: Option<String>,
    pub title: String,
    pub time: f64,
    pub size: u64,
    /// le fichier du projet a été enregistré après cette copie
    pub saved_after: bool,
}

pub struct AutoSave {
    id: String,
    /// verrou tenu pendant toute la session : une autre fenêtre sait qu'on est vivant
    _lock: Option<std::fs::File>,
    last_ver: u64,
    last_t: f64,
    pub orphans: Vec<Orphan>,
    pub show_recover: bool,
    /// l'utilisateur a confirmé la fermeture
    pub quit_ok: bool,
}

pub fn now_secs() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

fn dir() -> PathBuf {
    crate::audio::data_dir().join("autosave")
}

/// « il y a 3 min », « il y a 2 jours »
pub fn ago(t: f64) -> String {
    let s = (now_secs() - t).max(0.0) as u64;
    if s < 60 {
        tr("à l’instant").to_string()
    } else if s < 3600 {
        trf("il y a {0} min", &[(s / 60).to_string()])
    } else if s < 86400 {
        trf("il y a {0} h", &[(s / 3600).to_string()])
    } else {
        trf("il y a {0} jour(s)", &[(s / 86400).to_string()])
    }
}

impl AutoSave {
    pub fn new() -> AutoSave {
        let id = format!("{}-{}", now_secs() as u64, std::process::id());
        let d = dir();
        let _ = std::fs::create_dir_all(&d);
        let lock = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(d.join(format!("{id}.lock"))).ok();
        if let Some(f) = &lock {
            let _ = f.try_lock();
        }
        let mut a = AutoSave { id, _lock: lock, last_ver: 0, last_t: 0.0, orphans: Vec::new(), show_recover: false, quit_ok: false };
        a.scan();
        a.show_recover = !a.orphans.is_empty();
        a
    }

    fn file(&self) -> PathBuf {
        dir().join(format!("{}.json", self.id))
    }

    /// cherche les copies de secours des sessions mortes
    pub fn scan(&mut self) {
        self.orphans.clear();
        let Ok(rd) = std::fs::read_dir(dir()) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let Some(stem) = p.file_stem().map(|s| s.to_string_lossy().to_string()) else { continue };
            // verrou d'une session morte sans copie de secours : on nettoie
            if p.extension().map(|x| x == "lock").unwrap_or(false) && stem != self.id && !p.with_extension("json").exists() {
                let free = std::fs::OpenOptions::new().write(true).open(&p).map(|f| f.try_lock().is_ok()).unwrap_or(false);
                if free {
                    let _ = std::fs::remove_file(&p);
                }
                continue;
            }
            if p.extension().map(|x| x != "json").unwrap_or(true) || stem == self.id {
                continue;
            }
            // session encore ouverte dans une autre fenêtre ? (son verrou est tenu)
            let lp = dir().join(format!("{stem}.lock"));
            if let Ok(f) = std::fs::OpenOptions::new().write(true).open(&lp) {
                if f.try_lock().is_err() {
                    continue;
                }
                let _ = f.unlock();
            }
            let Ok(s) = std::fs::read_to_string(&p) else { continue };
            let Ok(v) = serde_json::from_str::<Json>(&s) else { continue };
            let path = v.get("path").and_then(|x| x.as_str()).map(|x| x.to_string());
            let time = v.f("time", 0.0);
            let saved_after = path.as_ref().and_then(|x| std::fs::metadata(x).ok()).and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs_f64() > time + 1.0).unwrap_or(false);
            self.orphans.push(Orphan { file: p, path, title: v.s("title").to_string(), time, size: s.len() as u64, saved_after });
        }
        self.orphans.sort_by(|a, b| b.time.partial_cmp(&a.time).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// fin de session normale : copie et verrou supprimés
    pub fn shutdown(&mut self) {
        self.clear();
        self._lock = None;
        let _ = std::fs::remove_file(dir().join(format!("{}.lock", self.id)));
    }

    /// supprime la copie de secours de cette session (projet enregistré, fermé ou abandonné)
    pub fn clear(&mut self) {
        let _ = std::fs::remove_file(self.file());
        self.last_ver = 0;
    }

    /// supprime une copie orpheline (et son verrou)
    pub fn forget(&mut self, file: &PathBuf) {
        let _ = std::fs::remove_file(file);
        let _ = std::fs::remove_file(file.with_extension("lock"));
        self.orphans.retain(|o| &o.file != file);
    }

    /// écrit tout de suite la copie de secours
    pub fn write_now(&mut self, app_proj: &Json, path: Option<&PathBuf>, ver: u64, now: f64) {
        let env = json!({
            "v": 1,
            "time": now_secs(),
            "path": path.map(|p| p.to_string_lossy().to_string()),
            "title": app_proj.get("settings").map(|s| s.s("title")).unwrap_or(""),
            "project": app_proj,
        });
        let txt = serde_json::to_string(&env).unwrap_or_default();
        let f = self.file();
        // écriture dans un fichier temporaire puis renommage : jamais de copie à moitié écrite
        std::thread::spawn(move || {
            let tmp = f.with_extension("tmp");
            if std::fs::write(&tmp, txt).is_ok() {
                let _ = std::fs::rename(&tmp, &f);
            }
        });
        self.last_ver = ver;
        self.last_t = now;
    }
}

// ---------------------------------------------------------------- préférences

pub fn autosave_minutes() -> f64 {
    if let Some(v) = std::env::var("PF_AUTOSAVE_SECS").ok().and_then(|v| v.parse::<f64>().ok()) {
        return v / 60.0;
    }
    crate::load_prefs().get("autosave").and_then(|v| v.as_f64()).unwrap_or(1.0)
}
pub fn autosave_into_file() -> bool {
    crate::load_prefs().get("autosaveFile").and_then(|v| v.as_bool()).unwrap_or(false)
}
fn set_pref(k: &str, v: Json) {
    let mut p = crate::load_prefs();
    p[k] = v;
    crate::save_prefs(&p);
}

pub fn recent() -> Vec<PathBuf> {
    crate::load_prefs().arr("recent").iter().filter_map(|v| v.as_str()).map(PathBuf::from).collect()
}
pub fn add_recent(p: &std::path::Path) {
    let p = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let mut v = recent();
    v.retain(|x| x != &p);
    v.insert(0, p);
    v.truncate(MAX_RECENT);
    set_pref("recent", json!(v.iter().map(|x| x.to_string_lossy().to_string()).collect::<Vec<_>>()));
}
fn remove_recent(p: &PathBuf) {
    let mut v = recent();
    v.retain(|x| x != p);
    set_pref("recent", json!(v.iter().map(|x| x.to_string_lossy().to_string()).collect::<Vec<_>>()));
}

// ---------------------------------------------------------------- à chaque image

pub fn tick(app: &mut App, ctx: &egui::Context, now: f64) {
    // fermeture de la fenêtre : on demande si des modifications ne sont pas enregistrées
    if ctx.input(|i| i.viewport().close_requested()) && app.dirty && !app.autosave.quit_ok {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        app.confirm = Some(Pending::Quit);
    }
    // sauvegarde automatique
    let mins = autosave_minutes();
    if mins > 0.0 && app.dirty && app.ver != app.autosave.last_ver && app.play.is_none() {
        let due = app.autosave.last_t + mins * 60.0;
        if now >= due || app.autosave.last_t == 0.0 && now >= mins * 60.0 {
            if autosave_into_file() && app.path.is_some() {
                app.save_quiet(now);
            } else {
                let (proj, path, ver) = (app.proj.clone(), app.path.clone(), app.ver);
                app.autosave.write_now(&proj, path.as_ref(), ver, now);
            }
            app.autosave.last_t = now;
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64((due - now).clamp(0.5, 60.0)));
        }
    }
    // fichiers glissés sur la fenêtre
    let mut dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
    if now > 1.5 {
        if let Some(v) = std::env::var_os("PF_DROP") {
            // test : simule un dépôt une seule fois
            std::env::remove_var("PF_DROP");
            dropped = std::env::split_paths(&v).collect();
        }
    }
    if !dropped.is_empty() {
        drop_files(app, ctx, dropped, now);
    }
    let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
    if hovering {
        let r = ctx.screen_rect();
        let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("dropov")));
        p.rect_filled(r, 0.0, egui::Color32::from_rgba_unmultiplied(10, 12, 18, 170));
        p.rect_stroke(r.shrink(12.0), 16.0, egui::Stroke::new(3.0_f32, C::ACCENT()), egui::StrokeKind::Inside);
        p.text(r.center(), egui::Align2::CENTER_CENTER, tr("Déposez ici : projet (.json) à ouvrir, images en sprites, sons dans le projet"), egui::FontId::proportional(20.0), egui::Color32::WHITE);
    }
}

fn drop_files(app: &mut App, ctx: &egui::Context, files: Vec<PathBuf>, now: f64) {
    let ext = |p: &PathBuf| p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    if let Some(p) = files.iter().find(|p| ["json", "pfproj", "platforge"].contains(&ext(p).as_str())) {
        app.request(Pending::Open(Some(p.clone())));
        return;
    }
    let images: Vec<&PathBuf> = files.iter().filter(|p| ["png", "jpg", "jpeg", "gif", "webp", "bmp"].contains(&ext(p).as_str())).collect();
    let sounds: Vec<PathBuf> = files.iter().filter(|p| ["wav", "mp3", "ogg", "flac", "m4a"].contains(&ext(p).as_str())).cloned().collect();
    let mut n_img = 0;
    if !images.is_empty() {
        app.snapshot();
        let pos = ctx.input(|i| i.pointer.latest_pos());
        let mut last = None;
        for (k, p) in images.iter().enumerate() {
            let Ok(b) = std::fs::read(p) else { continue };
            let Ok(img) = image::load_from_memory(&b) else { continue };
            let rgba = img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            let mut out = Vec::new();
            use image::ImageEncoder;
            let _ = image::codecs::png::PngEncoder::new(&mut out).write_image(&rgba, w, h, image::ExtendedColorType::Rgba8);
            let url = format!("data:image/png;base64,{}", pf_core::b64_encode(&out));
            let mut o = crate::tree::make_object(app, "sprite");
            let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Sprite".into());
            let name: String = stem.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
            o["name"] = json!(crate::tree::unique_name(app, if name.is_empty() { "Sprite" } else { &name }));
            o["anims"][0]["frames"] = json!([url]);
            for (k2, v) in [("w", w), ("h", h), ("defW", w), ("defH", h), ("ox", w / 2), ("oy", h / 2)] {
                o[k2] = json!(v);
            }
            o["cbox"] = json!({"x":0,"y":0,"w":w,"h":h});
            let id = o.s("id").to_string();
            app.proj["objects"].as_array_mut().unwrap().push(o);
            app.changed();
            // dans la vue du niveau : une instance là où on a lâché le fichier
            if app.tab == crate::Tab::Layout {
                let (x, y) = match pos {
                    Some(p) if app.view.rect.contains(p) => app.view.to_world(p),
                    _ => app.view.center_world(),
                };
                crate::tree::add_instance(app, &id, x + k as f64 * 24.0, y + k as f64 * 24.0);
            }
            last = Some(id);
            n_img += 1;
        }
        if let Some(id) = last {
            app.sel = crate::Sel::Obj(id);
        }
    }
    let n_snd = if sounds.is_empty() { 0 } else { crate::sounds::import_paths(app, sounds) };
    if n_img + n_snd > 0 {
        app.toast(trf("Ajouté au projet : {0} sprite(s), {1} son(s).", &[n_img.to_string(), n_snd.to_string()]), false, now);
    } else {
        app.toast(tr("Ce type de fichier n’est pas pris en charge."), true, now);
    }
}

// ---------------------------------------------------------------- fenêtres

/// confirmation « enregistrer les modifications ? »
pub fn confirm_window(app: &mut App, ctx: &egui::Context, now: f64) {
    let Some(p) = app.confirm.clone() else { return };
    let mut close = false;
    egui::Window::new(tr("Modifications non enregistrées")).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        let title = app.proj.get("settings").map(|s| s.s("title").to_string()).unwrap_or_default();
        ui.label(trf("Enregistrer les modifications de « {0} » ?", &[title]));
        ui.label(egui::RichText::new(match p {
            Pending::Quit => tr("Sinon, elles seront perdues à la fermeture."),
            _ => tr("Sinon, elles seront perdues."),
        }).color(C::MUTED()));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.add(theme::btn_accent(tr("Enregistrer"))).clicked() {
                app.save(false, now);
                if !app.dirty {
                    close = true;
                    app.run_pending(ctx, p.clone());
                }
            }
            if ui.add(theme::btn(tr("Ne pas enregistrer"))).clicked() {
                close = true;
                app.dirty = false;
                app.autosave.clear();
                app.run_pending(ctx, p.clone());
            }
            if ui.add(theme::btn(tr("Annuler"))).clicked() {
                close = true;
            }
        });
    });
    if close {
        app.confirm = None;
    }
}

/// récupération après une fermeture inattendue
pub fn recover_window(app: &mut App, ctx: &egui::Context) {
    if !app.autosave.show_recover {
        return;
    }
    if app.autosave.orphans.is_empty() {
        app.autosave.show_recover = false;
        return;
    }
    let mut open = true;
    let mut act: Option<(PathBuf, bool)> = None;
    let (dsz, msz) = theme::fit(ctx, 560.0, 380.0);
    egui::Window::new(tr("Récupérer un projet")).open(&mut open).collapsible(false).default_size(dsz).max_size(msz).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        ui.label(tr("PlatForge ne s’est pas fermé normalement. Ces copies de secours contiennent du travail non enregistré :"));
        ui.add_space(6.0);
        egui::ScrollArea::vertical().max_height(msz.y - 120.0).show(ui, |ui| {
            for o in &app.autosave.orphans {
                egui::Frame::new().fill(C::BG2()).corner_radius(8.0).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(if o.title.is_empty() { tr("(sans titre)") } else { &o.title }).strong().color(C::TEXT()));
                            let file = o.path.as_ref().and_then(|p| PathBuf::from(p).file_name().map(|f| f.to_string_lossy().to_string())).unwrap_or_else(|| tr("projet jamais enregistré").to_string());
                            ui.label(egui::RichText::new(format!("{} · {} · {} Ko", file, ago(o.time), o.size / 1024)).size(11.0).color(C::MUTED()));
                            if o.saved_after {
                                ui.label(egui::RichText::new(tr("Le fichier du projet a été enregistré plus tard : cette copie est sans doute dépassée.")).size(11.0).color(C::DIM()));
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(theme::btn(tr("Supprimer"))).on_hover_text(tr("Jeter cette copie de secours")).clicked() {
                                act = Some((o.file.clone(), false));
                            }
                            if ui.add(theme::btn_accent(tr("Récupérer"))).clicked() {
                                act = Some((o.file.clone(), true));
                            }
                        });
                    });
                });
                ui.add_space(4.0);
            }
        });
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tr("« Plus tard » garde les copies : elles seront proposées au prochain lancement.")).size(11.0).color(C::DIM()));
        if ui.add(theme::btn(tr("Plus tard"))).clicked() {
            app.autosave.show_recover = false;
        }
    });
    if !open {
        app.autosave.show_recover = false;
    }
    match act {
        Some((f, true)) => app.request(Pending::Recover(f)),
        Some((f, false)) => app.autosave.forget(&f),
        None => {}
    }
}

/// sous-menu Fichier ▸ Projets récents
pub fn recent_menu(app: &mut App, ui: &mut egui::Ui) {
    let list = recent();
    ui.menu_button(tr("Projets récents"), |ui| {
        if list.is_empty() {
            ui.label(egui::RichText::new(tr("Aucun pour l’instant")).color(C::DIM()));
        }
        for p in &list {
            let name = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
            let exists = p.exists();
            let r = ui.add_enabled(exists, egui::Button::new(if exists { name.clone() } else { trf("{0} (introuvable)", &[name.clone()]) }));
            let r = r.on_hover_text(p.to_string_lossy()).on_disabled_hover_text(p.to_string_lossy());
            if r.clicked() {
                app.request(Pending::Open(Some(p.clone())));
                ui.close_menu();
            }
            if !exists {
                r.context_menu(|ui| {
                    if ui.button(tr("Retirer de la liste")).clicked() {
                        remove_recent(p);
                        ui.close_menu();
                    }
                });
            }
        }
        if !list.is_empty() {
            ui.separator();
            if ui.button(tr("Vider la liste")).clicked() {
                set_pref("recent", json!([]));
                ui.close_menu();
            }
        }
    });
    ui.menu_button(tr("Sauvegarde automatique"), |ui| {
        let cur = autosave_minutes();
        for (m, n) in [(0.0, tr("Désactivée")), (1.0, tr("Toutes les minutes")), (2.0, tr("Toutes les 2 minutes")), (5.0, tr("Toutes les 5 minutes")), (10.0, tr("Toutes les 10 minutes"))] {
            if ui.radio(cur == m, n).clicked() {
                set_pref("autosave", json!(m));
            }
        }
        ui.separator();
        let mut f = autosave_into_file();
        if ui.checkbox(&mut f, tr("Enregistrer aussi dans le fichier du projet")).on_hover_text(tr("Sinon, une copie de secours est gardée à part : votre fichier n’est modifié que quand vous enregistrez.")).changed() {
            set_pref("autosaveFile", json!(f));
        }
        ui.separator();
        if ui.button(tr("Copies de secours…")).clicked() {
            app.autosave.scan();
            app.autosave.show_recover = true;
            if app.autosave.orphans.is_empty() {
                app.toast(tr("Aucune copie de secours à récupérer."), false, 0.0);
            }
            ui.close_menu();
        }
    });
}

/// liste compacte des projets récents (fenêtre « Nouveau projet »)
pub fn recent_list(app: &mut App, ui: &mut egui::Ui) {
    let list: Vec<PathBuf> = recent().into_iter().filter(|p| p.exists()).take(6).collect();
    if list.is_empty() {
        return;
    }
    ui.label(egui::RichText::new(tr("Projets récents")).color(C::MUTED()));
    ui.horizontal_wrapped(|ui| {
        for p in list {
            let name = p.file_stem().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
            if ui.add(theme::btn_small(&name)).on_hover_text(p.to_string_lossy()).clicked() {
                app.request(Pending::Open(Some(p.clone())));
            }
        }
    });
}
