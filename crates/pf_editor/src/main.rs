//! PlatForge Studio — éditeur natif (Rust + egui).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "512"]

mod audio;
mod evsheet;
mod keys;
mod props;
mod theme;
mod export;
mod find;
mod files;
mod history;
mod i18n;
mod more;
mod particles;
mod tools;
mod tpl;
mod sounds;
mod sprite;
mod tree;
mod util;
mod view;

use eframe::egui;
use pf_core::{Json, JsonExt};
use pf_runtime::draw::Imgs;
use pf_runtime::gfx::Gfx;
use pf_runtime::model::Def;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub use theme::C;

#[derive(Clone, Debug, PartialEq)]
pub enum Sel {
    None,
    Inst(String),
    Obj(String),
    Layout(String),
    Global(String),
    Sheet(String),
    Sound(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Layout,
    Events,
}

/// cache des images décodées pour l'éditeur
#[derive(Default)]
pub struct EdImgs {
    pub list: Vec<Option<Rc<tiny_skia::Pixmap>>>,
    pub idx: HashMap<String, usize>,
}
impl EdImgs {
    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&k) = self.idx.get(s) {
            return k;
        }
        let k = self.list.len();
        self.list.push(pf_core::data_url_bytes(s).and_then(|b| pf_runtime::img::decode(&b)).map(Rc::new));
        self.idx.insert(s.to_string(), k);
        k
    }
}
impl Imgs for EdImgs {
    fn img(&self, k: usize) -> Option<Rc<tiny_skia::Pixmap>> {
        self.list.get(k).and_then(|x| x.clone())
    }
}

pub struct Play {
    pub rt: pf_runtime::Runtime,
    pub g: Gfx,
    pub tex: Option<egui::TextureHandle>,
    pub last: std::time::Instant,
    pub shift: bool,
    pub ctrl: bool,
    pub focused: bool,
    pub show_dbg: bool,
    pub gil: Option<gilrs::Gilrs>,
}

pub struct Toast {
    pub text: String,
    pub until: f64,
    pub err: bool,
}

pub struct App {
    pub proj: Json,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub ver: u64,
    pub undo: Vec<String>,
    pub redo: Vec<String>,
    pub sel: Sel,
    pub cur_layout: String,
    pub cur_sheet: String,
    pub tab: Tab,
    pub imgs: EdImgs,
    pub defs: HashMap<String, Rc<Def>>,
    pub defs_ver: u64,
    pub view: view::ViewState,
    pub ev: evsheet::EvState,
    pub play: Option<Play>,
    pub show_templates: bool,
    pub toasts: Vec<Toast>,
    pub tree: tree::TreeState,
    pub confirm: Option<files::Pending>,
    pub autosave: files::AutoSave,
    /// heure de l'interface (secondes depuis le lancement), mise à jour à chaque image
    pub clock: f64,
    pub sprite: Option<sprite::SpriteEd>,
    pub part_ed: Option<particles::PartEd>,
    pub snd_lib: Option<sounds::SndLib>,
    pub preview: Option<more::Preview>,
    pub cine_ed: Option<String>,
    pub sfx_ed: Option<more::SfxEd>,
    pub track_ed: Option<more::TrackEd>,
    pub ev_clip: Option<Json>,
    pub bps: std::collections::HashSet<String>,
    pub find: Option<find::FindState>,
    pub export: Option<export::ExportState>,
    pub help: bool,
    pub show_hist: bool,
    pub hist: history::HistCache,
    pub undo_t: Vec<f64>,
    pub redo_t: Vec<f64>,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> App {
        theme::C::LIGHT.store(load_prefs().get("light").and_then(|v| v.as_bool()).unwrap_or(false), std::sync::atomic::Ordering::Relaxed);
        {
            let pr = load_prefs();
            let en = match pr.get("lang").and_then(|v| v.as_str()) {
                Some(l) => l == "en",
                None => i18n::system_is_en(),
            };
            let en = std::env::var("PF_LANG").map(|l| l == "en").unwrap_or(en);
            i18n::set_en(en);
        }
        theme::install(&cc.egui_ctx);
        let args: Vec<String> = std::env::args().collect();
        if std::env::var("PF_LIGHT").is_ok() {
            theme::set_light(&cc.egui_ctx, true);
        }
        let mut app = App {
            proj: tpl::make("blank"),
            path: None,
            dirty: false,
            ver: 1,
            undo: Vec::new(),
            redo: Vec::new(),
            sel: Sel::None,
            cur_layout: String::new(),
            cur_sheet: String::new(),
            tab: Tab::Layout,
            imgs: EdImgs::default(),
            defs: HashMap::new(),
            defs_ver: 0,
            view: view::ViewState::default(),
            ev: evsheet::EvState::default(),
            play: None,
            show_templates: true,
            toasts: Vec::new(),
            tree: tree::TreeState::default(),
            confirm: None,
            autosave: files::AutoSave::new(),
            clock: 0.0,
            sprite: None,
            preview: None,
            cine_ed: None,
            sfx_ed: None,
            track_ed: None,
            ev_clip: None,
            bps: Default::default(),
            find: None,
            export: None,
            help: false,
            part_ed: None,
            snd_lib: None,
            show_hist: false,
            hist: Default::default(),
            undo_t: Vec::new(),
            redo_t: Vec::new(),
        };
        // `platforge jeu.json` ouvre un projet ; `platforge --play jeu.json` lance directement le jeu
        let play_only = args.iter().any(|a| a == "--play");
        if let Some(p) = args.iter().skip(1).find(|a| !a.starts_with("--")) {
            app.open_path(PathBuf::from(p));
        }
        app.fix_current();
        if let Ok(out) = std::env::var("PF_EXPORT") {
            let st = export::ExportState::new(&app);
            let _ = std::fs::write(&out, export::build_html(&app, &st.o, false));
            std::process::exit(0);
        }
        if std::env::var("PF_TOOL").is_ok() {
            app.view.tool = tools::Tool::Paint;
        }
        if std::env::var("PF_TAB").map(|v| v == "events").unwrap_or(false) {
            app.tab = Tab::Events;
        }
        if std::env::var("PF_SOUNDS").is_ok() {
            app.snd_lib = Some(Default::default());
        }
        if let Ok(o) = std::env::var("PF_PARTS") {
            if let Some(id) = app.proj.arr("objects").iter().find(|x| x.s("name") == o).map(|x| x.s("id").to_string()) {
                app.part_ed = particles::PartEd::open(&app, &id);
            }
        }
        if let Ok(o) = std::env::var("PF_SPRITE") {
            if let Some(ob) = app.proj.arr("objects").iter().find(|x| x.s("name") == o) {
                app.sprite = Some(sprite::SpriteEd::open(ob));
            }
        }
        if let Ok(o) = std::env::var("PF_INST") {
            let lay = app.proj.arr("layouts").iter().find(|l| l.arr("instances").iter().any(|i| app.obj_name(i.s("obj")) == o)).map(|l| l.s("id").to_string());
            if let Some(l) = lay {
                app.cur_layout = l;
            }
            let id = app.layout().and_then(|l| l.arr("instances").iter().find(|i| app.obj_name(i.s("obj")) == o)).map(|i| i.s("id").to_string());
            if let Some(id) = id {
                app.sel = Sel::Inst(id);
            }
        }
        if let Ok(o) = std::env::var("PF_TRACK") {
            if let Some(ob) = app.proj.arr("objects").iter().find(|x| x.s("name") == o).map(|x| x.s("id").to_string()) {
                app.ensure_defs();
                app.track_ed = Some(more::TrackEd::new(&app, &ob));
            }
        }
        if std::env::var("PF_CINE").is_ok() {
            more::new_cine(&mut app);
        }
        if std::env::var("PF_SFX").is_ok() {
            app.sfx_ed = Some(more::SfxEd::new(&app, None));
        }
        if let Ok(o) = std::env::var("PF_SEL") {
            app.sel = Sel::Obj(app.proj.arr("objects").iter().find(|x| x.s("name") == o).map(|x| x.s("id").to_string()).unwrap_or_default());
        }
        if play_only {
            app.start_play(None);
        }
        app
    }

    pub fn toast(&mut self, t: impl Into<String>, err: bool, now: f64) {
        let now = now.max(self.clock);
        self.toasts.push(Toast { text: t.into(), until: now + if err { 5.0 } else { 2.6 }, err });
    }

    /// avant une modification : mémorise l'état pour Annuler
    pub fn snapshot(&mut self) {
        let s = serde_json::to_string(&self.proj).unwrap_or_default();
        if self.undo.last() != Some(&s) {
            self.undo.push(s);
            self.undo_t.push(history::now());
            if self.undo.len() > 100 {
                self.undo.remove(0);
                self.undo_t.remove(0);
            }
        }
        self.redo.clear();
        self.redo_t.clear();
    }
    pub fn changed(&mut self) {
        self.dirty = true;
        self.ver += 1;
    }
    pub fn do_undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            let t = self.undo_t.pop().unwrap_or(0.0);
            self.redo_t.push(t);
            self.redo.push(serde_json::to_string(&self.proj).unwrap_or_default());
            self.proj = serde_json::from_str(&s).unwrap_or(Json::Null);
            self.changed();
            self.fix_current();
        }
    }
    pub fn do_redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            let t = self.redo_t.pop().unwrap_or(0.0);
            self.undo_t.push(t);
            self.undo.push(serde_json::to_string(&self.proj).unwrap_or_default());
            self.proj = serde_json::from_str(&s).unwrap_or(Json::Null);
            self.changed();
            self.fix_current();
        }
    }

    /// garde un niveau / une feuille valides
    pub fn fix_current(&mut self) {
        let ls = self.proj.arr("layouts");
        if !ls.iter().any(|l| l.s("id") == self.cur_layout) {
            self.cur_layout = ls.first().map(|l| l.s("id").to_string()).unwrap_or_default();
            self.view.fit_pending = true;
        }
        let sh = self.layout().map(|l| l.s("sheet").to_string()).unwrap_or_default();
        if !self.proj.arr("sheets").iter().any(|s| s.s("id") == self.cur_sheet) {
            self.cur_sheet = if sh.is_empty() { self.proj.arr("sheets").first().map(|s| s.s("id").to_string()).unwrap_or_default() } else { sh };
        }
        if let Sel::Inst(id) = &self.sel {
            if self.find_inst(id).is_none() {
                self.sel = Sel::None;
            }
        }
    }

    pub fn layout(&self) -> Option<&Json> {
        self.proj.arr("layouts").iter().find(|l| l.s("id") == self.cur_layout)
    }
    pub fn layout_mut(&mut self) -> Option<&mut Json> {
        let id = self.cur_layout.clone();
        self.proj.get_mut("layouts")?.as_array_mut()?.iter_mut().find(|l| l.s("id") == id)
    }
    pub fn obj(&self, id: &str) -> Option<&Json> {
        self.proj.arr("objects").iter().find(|o| o.s("id") == id)
    }
    pub fn obj_mut(&mut self, id: &str) -> Option<&mut Json> {
        self.proj.get_mut("objects")?.as_array_mut()?.iter_mut().find(|o| o.s("id") == id)
    }
    pub fn obj_name(&self, id: &str) -> String {
        self.obj(id).map(|o| o.s("name").to_string()).unwrap_or_else(|| "?".into())
    }
    pub fn find_inst(&self, id: &str) -> Option<&Json> {
        self.layout().and_then(|l| l.arr("instances").iter().find(|i| i.s("id") == id))
    }
    pub fn inst_mut(&mut self, id: &str) -> Option<&mut Json> {
        let id = id.to_string();
        self.layout_mut()?.get_mut("instances")?.as_array_mut()?.iter_mut().find(|i| i.s("id") == id)
    }

    /// (re)construit les définitions d'objets utilisées par l'aperçu
    pub fn ensure_defs(&mut self) {
        if self.defs_ver == self.ver {
            return;
        }
        self.defs_ver = self.ver;
        let objs = self.proj.arr("objects").to_vec();
        let mut defs = HashMap::new();
        for o in &objs {
            let d = {
                let imgs = &mut self.imgs;
                let mut f = |s: &str| imgs.intern(s);
                Def::parse(o, &mut f)
            };
            defs.insert(o.s("id").to_string(), Rc::new(d));
        }
        self.defs = defs;
        self.view.dirty = true;
    }

    // ---------------------------------------------------------------- fichiers

    pub fn load_project(&mut self, p: Json, path: Option<PathBuf>) {
        self.autosave.clear();
        self.proj = p;
        self.path = path;
        self.dirty = false;
        self.undo.clear();
        self.redo.clear();
        self.undo_t.clear();
        self.redo_t.clear();
        self.sel = Sel::None;
        self.cur_layout = String::new();
        self.cur_sheet = String::new();
        self.ver += 1;
        self.fix_current();
        self.view.fit_pending = true;
        self.show_templates = false;
        self.play = None;
    }
    pub fn open_path(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str::<Json>(&s).map_err(|e| e.to_string())) {
            Ok(p) if p.get("objects").is_some() => {
                files::add_recent(&path);
                self.load_project(p, Some(path));
            }
            Ok(_) => self.toast(crate::i18n::tr("Ce fichier n’est pas un projet PlatForge."), true, 0.0),
            Err(e) => self.toast(crate::i18n::trf("Impossible d’ouvrir : {0}", &[format!("{}", e)]), true, 0.0),
        }
    }
    pub fn open_dialog(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter(crate::i18n::tr("Projet PlatForge"), &["json", "pfproj", "platforge"]).pick_file() {
            self.open_path(p);
        }
    }
    pub fn save(&mut self, as_new: bool, now: f64) {
        let path = if as_new || self.path.is_none() {
            let name = format!("{}.json", util::slug(self.proj.get("settings").map(|s| s.s("title")).unwrap_or("jeu")));
            match rfd::FileDialog::new().add_filter(crate::i18n::tr("Projet PlatForge"), &["json"]).set_file_name(name).save_file() {
                Some(p) => p,
                None => return,
            }
        } else {
            self.path.clone().unwrap()
        };
        match std::fs::write(&path, serde_json::to_string_pretty(&self.proj).unwrap_or_default()) {
            Ok(_) => {
                self.path = Some(path.clone());
                self.dirty = false;
                self.autosave.clear();
                files::add_recent(&path);
                self.toast(crate::i18n::trf("Enregistré : {0}", &[format!("{}", path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default())]), false, now);
            }
            Err(e) => self.toast(crate::i18n::trf("Échec de l’enregistrement : {0}", &[format!("{}", e)]), true, now),
        }
    }
    /// enregistrement automatique dans le fichier du projet (sans boîte de dialogue)
    pub fn save_quiet(&mut self, now: f64) {
        let Some(path) = self.path.clone() else { return };
        let tmp = path.with_extension("json.tmp");
        let ok = std::fs::write(&tmp, serde_json::to_string_pretty(&self.proj).unwrap_or_default()).and_then(|_| std::fs::rename(&tmp, &path));
        match ok {
            Ok(_) => {
                self.dirty = false;
                self.autosave.clear();
                self.toast(crate::i18n::tr("Enregistré automatiquement."), false, now);
            }
            Err(e) => self.toast(crate::i18n::trf("Échec de l’enregistrement : {0}", &[e.to_string()]), true, now),
        }
    }
    /// demande une action ; si le projet a des modifications non enregistrées, on confirme d'abord
    pub fn request(&mut self, p: files::Pending) {
        self.show_templates = false;
        if self.dirty {
            self.confirm = Some(p);
        } else {
            self.run_pending_now(p);
        }
    }
    fn run_pending_now(&mut self, p: files::Pending) {
        match p {
            files::Pending::Template(t) => self.load_project(tpl::make(&t), None),
            files::Pending::Open(Some(path)) => self.open_path(path),
            files::Pending::Open(None) => self.open_dialog(),
            files::Pending::Recover(f) => {
                let env = std::fs::read_to_string(&f).ok().and_then(|s| serde_json::from_str::<Json>(&s).ok());
                match env.and_then(|v| v.get("project").cloned().map(|p| (p, v.get("path").and_then(|x| x.as_str()).map(PathBuf::from)))) {
                    Some((p, path)) if p.get("objects").is_some() => {
                        self.load_project(p, path);
                        self.dirty = true;
                        self.ver += 1;
                        // nouvelle copie de secours tout de suite, puis on jette l'ancienne
                        let (proj, path, ver) = (self.proj.clone(), self.path.clone(), self.ver);
                        self.autosave.write_now(&proj, path.as_ref(), ver, 0.0);
                        self.autosave.forget(&f);
                        self.toast(crate::i18n::tr("Projet récupéré. Pensez à l’enregistrer (Ctrl+S)."), false, 0.0);
                    }
                    _ => {
                        self.toast(crate::i18n::tr("Cette copie de secours est illisible."), true, 0.0);
                        self.autosave.forget(&f);
                    }
                }
            }
            files::Pending::Quit => {}
        }
    }
    pub fn run_pending(&mut self, ctx: &egui::Context, p: files::Pending) {
        if let files::Pending::Quit = p {
            self.autosave.quit_ok = true;
            self.autosave.clear();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            self.run_pending_now(p);
        }
    }
    /// jeu autonome : copie de l'application + le projet, lancé en mode joueur
    pub fn export_game(&mut self, now: f64) {
        let title = self.proj.get("settings").map(|s| s.s("title").to_string()).unwrap_or_else(|| "jeu".into());
        let dir = match rfd::FileDialog::new().set_title(crate::i18n::tr("Dossier où exporter le jeu")).pick_folder() {
            Some(d) => d.join(util::slug(&title)),
            None => return,
        };
        let r = (|| -> Result<PathBuf, String> {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let ext = exe.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
            let dst = dir.join(format!("{}{}", util::slug(&title), ext));
            std::fs::copy(&exe, &dst).map_err(|e| e.to_string())?;
            std::fs::write(dir.join("jeu.json"), serde_json::to_string(&self.proj).unwrap_or_default()).map_err(|e| e.to_string())?;
            let launch = if cfg!(windows) { crate::i18n::trf("@echo off\r\nstart \"\" \"%~dp0{0}{1}\" --play \"%~dp0jeu.json\"\r\n", &[format!("{}", util::slug(&title)), format!("{}", ext)]) } else { format!("#!/bin/sh\ncd \"$(dirname \"$0\")\"\n./{} --play jeu.json\n", util::slug(&title)) };
            let lp = dir.join(if cfg!(windows) { crate::i18n::tr("Jouer.bat") } else { "jouer.sh" });
            std::fs::write(&lp, launch).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&lp, std::fs::Permissions::from_mode(0o755));
            }
            Ok(dir.clone())
        })();
        match r {
            Ok(d) => self.toast(crate::i18n::trf("Jeu exporté dans {0}", &[format!("{}", d.display())]), false, now),
            Err(e) => self.toast(crate::i18n::trf("Export impossible : {0}", &[format!("{}", e)]), true, now),
        }
    }

    // ---------------------------------------------------------------- tester

    pub fn start_play(&mut self, layout: Option<String>) {
        let host = audio::AudioHost::new(&self.proj);
        let rt = pf_runtime::Runtime::new(self.proj.clone(), pf_runtime::Options { start_layout: layout, splash: false, embedded: false, seed: None, lcg: None }, Box::new(host));
        let mut rt = rt;
        rt.bps = self.bps.clone();
        let (w, h) = (rt.w as u32, rt.h as u32);
        self.play = Some(Play { rt, g: Gfx::new(w, h), tex: None, last: std::time::Instant::now(), shift: false, ctrl: false, focused: true, show_dbg: false, gil: gilrs::Gilrs::new().ok() });
    }
    pub fn stop_play(&mut self) {
        if let Some(mut p) = self.play.take() {
            p.rt.stop();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _ = frame;
        let now = ctx.input(|i| i.time);
        self.clock = now;
        self.ensure_defs();
        files::tick(self, ctx, now);
        let title = crate::i18n::trf("{0}{1} — PlatForge Studio", &[format!("{}", if self.dirty { "• " } else { "" }), format!("{}", self.proj.get("settings").map(|s| s.s("title")).unwrap_or(crate::i18n::tr("Jeu")))]);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        // raccourcis globaux
        if self.play.is_none() && self.sprite.is_none() && !crate::util::typing(ctx) {
            let (z, y, s, f5, del, dup) = ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift, i.modifiers.command && (i.key_pressed(egui::Key::Y) || (i.modifiers.shift && i.key_pressed(egui::Key::Z))), i.modifiers.command && i.key_pressed(egui::Key::S), i.key_pressed(egui::Key::F5), i.key_pressed(egui::Key::Delete), i.modifiers.command && i.key_pressed(egui::Key::D)));
            if z {
                self.do_undo();
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::H)) {
                self.show_hist = !self.show_hist;
            }
            if y {
                self.do_redo();
            }
            if s {
                self.save(false, now);
            }
            if f5 {
                self.start_play(None);
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
                self.find = Some(find::FindState::default());
            }
            if del {
                self.delete_selection();
            }
            if dup {
                self.duplicate_selection();
            }
        }

        self.top_bar(ctx, now);
        if self.play.is_some() {
            self.play_ui(ctx);
        } else {
            tree::panel(self, ctx, now);
            props::panel(self, ctx, now);
            egui::CentralPanel::default().frame(egui::Frame::new().fill(C::BG0())).show(ctx, |ui| {
                self.tabs(ui);
                match self.tab {
                    Tab::Layout => view::ui(self, ui, now),
                    Tab::Events => evsheet::ui(self, ui, now),
                }
            });
        }
        if self.play.is_none() {
            if self.sprite.is_some() {
                sprite::window(self, ctx, now);
            }
            more::cine_window(self, ctx, now);
            particles::window(self, ctx, now);
            sounds::window(self, ctx, now);
            more::sfx_window(self, ctx, now);
            more::track_window(self, ctx);
            find::window(self, ctx);
            export::window(self, ctx, now);
            if self.help {
                help_window(self, ctx);
            }
            history::window(self, ctx);
        }
        if self.show_templates {
            self.templates_window(ctx);
        }
        if self.play.is_none() {
            files::recover_window(self, ctx);
        }
        files::confirm_window(self, ctx, now);
        self.draw_toasts(ctx, now);
        if self.play.is_some() {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_play();
        // fermeture normale : la copie de secours n'est plus utile
        self.autosave.shutdown();
    }
}

impl App {
    fn top_bar(&mut self, ctx: &egui::Context, now: f64) {
        egui::TopBottomPanel::top("top").exact_height(46.0).frame(egui::Frame::new().fill(C::BG1()).inner_margin(egui::Margin::symmetric(10, 6)).stroke(egui::Stroke::new(1.0_f32, C::LINE()))).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("◆").color(C::ACCENT()).size(20.0));
                ui.label(egui::RichText::new("PlatForge").strong().size(16.0).color(C::TEXT()));
                ui.label(egui::RichText::new("Studio").size(16.0).color(C::MUTED()));
                ui.add_space(14.0);
                if self.play.is_some() {
                    if ui.add(theme::btn_accent(crate::i18n::tr("■  Arrêter (Échap)"))).clicked() {
                        self.stop_play();
                    }
                    let restart = ui.add(theme::btn(crate::i18n::tr("↻  Recommencer"))).clicked();
                    if restart {
                        if let Some(p) = self.play.as_mut() {
                            p.rt.restart();
                        }
                    }
                    if let Some(p) = self.play.as_mut() {
                        let pl = if p.rt.paused { crate::i18n::tr("▶  Reprendre") } else { crate::i18n::tr("❚❚  Pause") };
                        if ui.add(theme::btn(pl)).clicked() {
                            p.rt.paused = !p.rt.paused;
                        }
                        if p.rt.paused && ui.add(theme::btn(crate::i18n::tr("⏭  Image suivante"))).clicked() {
                            p.rt.step_req = true;
                        }
                        ui.toggle_value(&mut p.show_dbg, crate::i18n::tr("Débogueur"));
                        ui.toggle_value(&mut p.rt.show_col, crate::i18n::tr("Collisions"));
                        ui.label(egui::RichText::new(crate::i18n::trf("Niveau : {0}", &[format!("{}", p.rt.layout_name())])).color(C::MUTED()));
                        if let (Some((_, t)), true) = (&p.rt.brk_hit, p.rt.paused) {
                            ui.label(egui::RichText::new(crate::i18n::trf("● Point d’arrêt atteint (image {0}) — « Reprendre » pour continuer", &[format!("{}", t)])).color(C::DANGER()));
                        }
                    }
                    return;
                }
                ui.menu_button(egui::RichText::new(crate::i18n::tr("Fichier")).color(C::TEXT2()), |ui| {
                    if ui.button(crate::i18n::tr("Nouveau depuis un modèle…")).clicked() {
                        self.show_templates = true;
                        ui.close_menu();
                    }
                    if ui.button(crate::i18n::tr("Ouvrir…  (Ctrl+O)")).clicked() {
                        ui.close_menu();
                        self.request(files::Pending::Open(None));
                    }
                    files::recent_menu(self, ui);
                    ui.separator();
                    if ui.button(crate::i18n::tr("Enregistrer  (Ctrl+S)")).clicked() {
                        ui.close_menu();
                        self.save(false, now);
                    }
                    if ui.button(crate::i18n::tr("Enregistrer sous…")).clicked() {
                        ui.close_menu();
                        self.save(true, now);
                    }
                    ui.separator();
                    if ui.button(crate::i18n::tr("Exporter le jeu (web, installable ou application)…")).clicked() {
                        ui.close_menu();
                        self.export = Some(export::ExportState::new(self));
                    }
                });
                ui.menu_button(egui::RichText::new(crate::i18n::tr("Édition")).color(C::TEXT2()), |ui| {
                    if ui.add_enabled(!self.undo.is_empty(), egui::Button::new(crate::i18n::tr("Annuler  (Ctrl+Z)"))).clicked() {
                        self.do_undo();
                        ui.close_menu();
                    }
                    if ui.add_enabled(!self.redo.is_empty(), egui::Button::new(crate::i18n::tr("Rétablir  (Ctrl+Y)"))).clicked() {
                        self.do_redo();
                        ui.close_menu();
                    }
                    if ui.button(crate::i18n::tr("Historique des modifications…  (Ctrl+H)")).clicked() {
                        self.show_hist = true;
                        ui.close_menu();
                    }
                    if ui.button(crate::i18n::tr("Rechercher / remplacer…  (Ctrl+F)")).clicked() {
                        self.find = Some(find::FindState::default());
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(crate::i18n::tr("Dupliquer la sélection  (Ctrl+D)")).clicked() {
                        self.duplicate_selection();
                        ui.close_menu();
                    }
                    if ui.button(crate::i18n::tr("Supprimer la sélection  (Suppr)")).clicked() {
                        self.delete_selection();
                        ui.close_menu();
                    }
                });
                if ui.add(egui::Button::new(egui::RichText::new(crate::i18n::tr("Aide")).color(C::TEXT2())).frame(false)).clicked() {
                    self.help = !self.help;
                }
                ui.add_space(8.0);
                if ui.add(theme::btn_accent(crate::i18n::tr("▶  Tester  (F5)"))).on_hover_text(crate::i18n::tr("Lance le jeu depuis le niveau de départ")).clicked() {
                    self.start_play(None);
                }
                if ui.add(theme::btn(crate::i18n::tr("▶  Ce niveau"))).on_hover_text(crate::i18n::tr("Lance le jeu sur le niveau affiché")).clicked() {
                    let l = self.cur_layout.clone();
                    self.start_play(Some(l));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let en = i18n::is_en();
                    if ui.add(egui::Button::new(egui::RichText::new(if en { "EN" } else { "FR" }).small()).frame(false)).on_hover_text(if en { "Interface : English — cliquer pour le français" } else { "Interface : français — click for English" }).clicked() {
                        i18n::set_en(!en);
                        let mut p = load_prefs();
                        p["lang"] = Json::String(if en { "fr" } else { "en" }.into());
                        save_prefs(&p);
                    }
                    let light = theme::C::LIGHT.load(std::sync::atomic::Ordering::Relaxed);
                    if ui.add(egui::Button::new(if light { "☾" } else { "☀" }).frame(false)).on_hover_text(if light { crate::i18n::tr("Passer au thème sombre") } else { crate::i18n::tr("Passer au thème clair") }).clicked() {
                        theme::set_light(ui.ctx(), !light);
                        let mut p = load_prefs();
                        p["light"] = Json::Bool(!light);
                        save_prefs(&p);
                    }
                    let name = self.path.as_ref().and_then(|p| p.file_name()).map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| crate::i18n::tr("projet non enregistré").into());
                    ui.label(egui::RichText::new(name).color(C::DIM()));
                });
            });
        });
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) && self.play.is_none() {
            self.request(files::Pending::Open(None));
        }
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new().fill(C::BG1()).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
            ui.horizontal(|ui| {
                let lname = self.layout().map(|l| l.s("name").to_string()).unwrap_or_default();
                let sname = self.proj.arr("sheets").iter().find(|s| s.s("id") == self.cur_sheet).map(|s| s.s("name").to_string()).unwrap_or_default();
                if ui.add(theme::tab(self.tab == Tab::Layout, &crate::i18n::trf("▦  Niveau : {0}", &[format!("{}", lname)]))).clicked() {
                    self.tab = Tab::Layout;
                }
                if ui.add(theme::tab(self.tab == Tab::Events, &crate::i18n::trf("☰  Événements : {0}", &[format!("{}", sname)]))).clicked() {
                    self.tab = Tab::Events;
                }
            });
        });
    }

    fn templates_window(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let (dsz, msz) = theme::fit(ctx, 760.0, 640.0);
        egui::Window::new(crate::i18n::tr("Nouveau projet")).open(&mut open).collapsible(false).resizable(false).default_width(dsz.x).max_size(msz).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(egui::RichText::new(crate::i18n::tr("Choisissez un point de départ : chaque modèle est un jeu complet que vous pouvez modifier.")).color(C::MUTED()));
            ui.add_space(8.0);
            egui::ScrollArea::vertical().max_height((msz.y - 110.0).min(520.0)).show(ui, |ui| {
                let two = msz.x > 740.0;
                egui::Grid::new("tpls").num_columns(if two { 2 } else { 1 }).spacing([10.0, 10.0]).show(ui, |ui| {
                    for (k, t) in tpl::LIST.iter().enumerate() {
                        let r = egui::Frame::new().fill(C::BG2()).corner_radius(10.0).stroke(egui::Stroke::new(1.0_f32, C::LINE2())).inner_margin(12.0).show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(340.0);
                                ui.set_height(58.0);
                                ui.label(egui::RichText::new(crate::i18n::tr(t.1)).strong().size(15.0).color(C::TEXT()));
                                ui.add(egui::Label::new(egui::RichText::new(crate::i18n::tr(t.2)).color(C::MUTED())).wrap());
                            });
                        });
                        let resp = r.response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
                        if resp.hovered() {
                            ui.painter().rect_stroke(resp.rect, 10.0, egui::Stroke::new(1.5_f32, C::ACCENT()), egui::StrokeKind::Inside);
                        }
                        if resp.clicked() {
                            self.request(files::Pending::Template(t.0.to_string()));
                        }
                        if !two || k % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button(crate::i18n::tr("Ouvrir un projet existant…")).clicked() {
                    self.request(files::Pending::Open(None));
                }
            });
            files::recent_list(self, ui);
        });
        if !open {
            self.show_templates = false;
        }
    }

    fn draw_toasts(&mut self, ctx: &egui::Context, now: f64) {
        self.toasts.retain(|t| t.until == 0.0 || t.until > now);
        for t in &mut self.toasts {
            if t.until < 1.0 {
                t.until = now + 4.0;
            }
        }
        if self.toasts.is_empty() {
            return;
        }
        egui::Area::new(egui::Id::new("toasts")).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0]).show(ctx, |ui| {
            for t in &self.toasts {
                egui::Frame::new().fill(if t.err { egui::Color32::from_rgb(90, 30, 36) } else { C::BG3() }).corner_radius(8.0).inner_margin(egui::Margin::symmetric(14, 8)).stroke(egui::Stroke::new(1.0_f32, if t.err { C::DANGER() } else { C::LINE2() })).show(ui, |ui| {
                    ui.label(egui::RichText::new(&t.text).color(C::TEXT()));
                });
            }
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }

    pub fn delete_selection(&mut self) {
        match self.sel.clone() {
            Sel::Inst(id) => {
                self.snapshot();
                if let Some(l) = self.layout_mut() {
                    if let Some(a) = l.get_mut("instances").and_then(|v| v.as_array_mut()) {
                        a.retain(|i| i.s("id") != id);
                    }
                }
                self.sel = Sel::None;
                self.changed();
            }
            _ => {}
        }
    }
    pub fn duplicate_selection(&mut self) {
        if let Sel::Inst(id) = self.sel.clone() {
            if let Some(mut j) = self.find_inst(&id).cloned() {
                self.snapshot();
                let nid = pf_core::uid("i");
                j["id"] = Json::String(nid.clone());
                j["x"] = pf_core::json!(j.f("x", 0.0) + 16.0);
                j["y"] = pf_core::json!(j.f("y", 0.0) + 16.0);
                if let Some(l) = self.layout_mut() {
                    if let Some(a) = l.get_mut("instances").and_then(|v| v.as_array_mut()) {
                        a.push(j);
                    }
                }
                self.sel = Sel::Inst(nid);
                self.changed();
            }
        }
    }

    fn play_ui(&mut self, ctx: &egui::Context) {
        let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        // Échap arrête le test, sauf si le jeu l'utilise (dialogue, cinématique, menu des touches, saisie du nom)
        if esc {
            let p = self.play.as_ref().unwrap();
            let used = p.rt.dlg.cur.is_some() || p.rt.cine.is_some() || p.rt.ctl.open || p.rt.hs_in.is_some();
            if !used {
                self.stop_play();
                return;
            }
        }
        let p = self.play.as_mut().unwrap();
        keys::feed(ctx, p);
        let mut gil = p.gil.take();
        keys::pad(&mut gil, p);
        p.gil = gil;
        let dt = p.last.elapsed().as_secs_f64();
        p.last = std::time::Instant::now();
        p.rt.frame(dt);
        if p.show_dbg {
            egui::SidePanel::right("dbg").default_width(300.0).frame(egui::Frame::new().fill(C::BG1()).inner_margin(10.0)).show(ctx, |ui| {
                props::debugger(ui, p);
            });
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::BLACK)).show(ctx, |ui| {
            let avail = ui.available_size();
            let ppp = ctx.pixels_per_point();
            let (pw, ph) = ((avail.x * ppp).round().max(1.0) as u32, (avail.y * ppp).round().max(1.0) as u32);
            // rendu à la taille réelle de la zone (net), limité à 2× la résolution du jeu
            let k = ((pw as f64 / p.rt.w).min(ph as f64 / p.rt.h)).min(2.0).max(0.25);
            let (gw, gh) = ((p.rt.w * k).round() as u32, (p.rt.h * k).round() as u32);
            p.g.resize(gw, gh);
            p.rt.render(&mut p.g);
            let rgba = pf_runtime::img::to_rgba(&p.g.pm);
            let im = egui::ColorImage::from_rgba_unmultiplied([gw as usize, gh as usize], &rgba);
            let opts = if p.rt.pixel_art { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR };
            match &mut p.tex {
                Some(t) => t.set(im, opts),
                None => p.tex = Some(ctx.load_texture("game", im, opts)),
            }
            let s = (avail.x / gw as f32).min(avail.y / gh as f32);
            let size = egui::vec2(gw as f32 * s, gh as f32 * s);
            let rect = egui::Rect::from_center_size(ui.max_rect().center(), size);
            let resp = ui.allocate_rect(ui.max_rect(), egui::Sense::click_and_drag());
            ui.painter().image(p.tex.as_ref().unwrap().id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
            // souris → coordonnées du jeu
            let (rw, rh) = (p.rt.w, p.rt.h);
            let to_game = move |pos: egui::Pos2| (((pos.x - rect.min.x) / size.x) as f64 * rw, ((pos.y - rect.min.y) / size.y) as f64 * rh);
            if let Some(pos) = resp.hover_pos().or(ctx.input(|i| i.pointer.latest_pos())) {
                let (x, y) = to_game(pos);
                p.rt.mouse_move(x, y);
            }
            ctx.input(|i| {
                for e in &i.events {
                    if let egui::Event::PointerButton { pos, button, pressed, .. } = e {
                        let b = match button {
                            egui::PointerButton::Primary => "left",
                            egui::PointerButton::Secondary => "right",
                            egui::PointerButton::Middle => "middle",
                            _ => continue,
                        };
                        if *pressed {
                            if rect.contains(*pos) {
                                let (x, y) = to_game(*pos);
                                p.rt.mouse_down(b, x, y);
                            }
                        } else {
                            p.rt.mouse_up(b);
                        }
                    }
                }
            });
            if !p.rt.errors.is_empty() {
                let txt = crate::i18n::trf("Erreur d’expression : {0}", &[format!("{}", p.rt.errors.last().unwrap())]);
                ui.painter().text(rect.left_bottom() + egui::vec2(8.0, -8.0), egui::Align2::LEFT_BOTTOM, txt, egui::FontId::proportional(13.0), C::DANGER());
            }
        });
        p.rt.host.tick();
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 880.0]).with_min_inner_size([900.0, 560.0]).with_title("PlatForge Studio").with_app_id("platforge-studio"),
        ..Default::default()
    };
    eframe::run_native("PlatForge Studio", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

fn load_prefs() -> Json {
    std::fs::read_to_string(audio::data_dir().join("editor.json")).ok().and_then(|s| serde_json::from_str::<Json>(&s).ok()).filter(|v| v.is_object()).unwrap_or_else(|| serde_json::json!({}))
}

fn save_prefs(p: &Json) {
    let d = audio::data_dir();
    let _ = std::fs::create_dir_all(&d);
    let _ = std::fs::write(d.join("editor.json"), serde_json::to_string_pretty(p).unwrap_or_default());
}

fn help_window(app: &mut App, ctx: &egui::Context) {
    let mut open = true;
    let (dsz, msz) = theme::fit(ctx, 620.0, 620.0);
    egui::Window::new(crate::i18n::tr("Aide")).open(&mut open).default_size(dsz).max_size(msz).vscroll(true).constrain(true).show(ctx, |ui| {
        let sec = |ui: &mut egui::Ui, t: &str, lines: &[&str]| {
            ui.label(egui::RichText::new(t).strong().size(15.0).color(C::ACCENT()));
            for l in lines {
                ui.label(egui::RichText::new(format!("•  {}", l)).color(C::TEXT2()));
            }
            ui.add_space(8.0);
        };
        sec(ui, crate::i18n::tr("Démarrer"), &[
            crate::i18n::tr("Fichier ▸ Nouveau depuis un modèle : chaque modèle est un jeu complet à modifier."),
            crate::i18n::tr("▶ Tester (F5) lance le jeu ; Échap revient à l’éditeur. « Ce niveau » démarre sur le niveau affiché."),
            crate::i18n::tr("Les projets sont des fichiers .json, compatibles avec la version HTML de PlatForge."),
            crate::i18n::tr("Le bouton FR / EN en haut à droite change la langue de l’interface ; ☀ / ☾ le thème."),
            crate::i18n::tr("Fichier ▸ Projets récents ; le travail est copié automatiquement chaque minute et proposé au redémarrage après un plantage."),
            crate::i18n::tr("Glissez sur la fenêtre un projet (.json) pour l’ouvrir, des images pour en faire des sprites, des sons pour les importer."),
        ]);
        sec(ui, crate::i18n::tr("Le niveau"), &[
            crate::i18n::tr("Glissez un objet du panneau de gauche dans le niveau pour en placer une instance."),
            crate::i18n::tr("Clic : sélectionner · glisser : déplacer · carré orange : redimensionner (Maj = proportions)."),
            crate::i18n::tr("Molette : zoom · clic du milieu, clic droit ou Espace + glisser : se déplacer."),
            crate::i18n::tr("Flèches : déplacer la sélection de 1 px (Maj : d’une case de grille) · Alt : sans aimantation."),
            crate::i18n::tr("Ctrl+D : dupliquer · Suppr : supprimer · double-clic : propriétés de l’objet."),
            crate::i18n::tr("Carte de tuiles sélectionnée : « Peindre les tuiles » (clic droit = effacer)."),
            crate::i18n::tr("Instance avec le comportement Trajectoire : « Trajectoire » pour placer les points."),
        ]);
        sec(ui, crate::i18n::tr("Les événements"), &[
            crate::i18n::tr("Un événement = des conditions (à gauche) qui, si elles sont toutes vraies, déclenchent les actions (à droite)."),
            crate::i18n::tr("Les sous-événements ne s’exécutent que si leur parent est vrai ; « Sinon » joue si l’événement précédent ne l’a pas été."),
            crate::i18n::tr("Les expressions acceptent + - * / ^ %, & (texte), = <> < > <= >=, et / ou / non, a ? b : c,"),
            crate::i18n::tr("les variables (Score), les propriétés (Joueur.X, Joueur.Speed) et des fonctions (random, round, distance…)."),
            crate::i18n::tr("Ctrl+C / Ctrl+V copient et collent des événements, même entre feuilles. « ● Point d’arrêt » met le test en pause."),
        ]);
        sec(ui, crate::i18n::tr("Les éditeurs"), &[
            crate::i18n::tr("Sprite : double-clic sur l’objet. S sélection, B crayon, E gomme, G remplir, I pipette, L ligne, R rectangle,"),
            crate::i18n::tr("U ellipse, O origine, C boîte de collision, P polygone, N points d’image, Ctrl+Z annuler."),
            crate::i18n::tr("Sprite, sélection : glisser dedans pour la déplacer (Alt : copie), Ctrl+C / X / V (aussi d’un sprite à l’autre), flèches, Suppr."),
            crate::i18n::tr("Sprite : « Plein » pour les formes pleines, « Sym. » pour dessiner en miroir, « Transformer » pour retourner, tourner, contourer…"),
            crate::i18n::tr("Tuiles : pinceau, rectangle, remplissage ou terrain automatique ; glisser dans la palette = tampon de plusieurs tuiles ; Alt+clic = pipette."),
            crate::i18n::tr("Particules : double-clic sur l’objet pour l’éditeur avec aperçu en direct."),
            crate::i18n::tr("Sons : ☰ ouvre la bibliothèque (écoute, rognage, volume, fondus, import multiple), ✦ crée un effet sonore."),
            crate::i18n::tr("Cinématiques : + dans la rubrique Cinématiques ; jouez-les avec l’action « Jouer une cinématique »."),
            crate::i18n::tr("Route 3D : « Éditeur visuel de la piste » dans ses propriétés."),
        ]);
        sec(ui, crate::i18n::tr("Raccourcis"), &[
            crate::i18n::tr("Ctrl+S enregistrer · Ctrl+O ouvrir · Ctrl+Z / Ctrl+Y annuler / rétablir · Ctrl+H historique · Ctrl+F rechercher · F5 tester."),
            crate::i18n::tr("Pendant le jeu : M coupe le son."),
        ]);
        sec(ui, crate::i18n::tr("Publier"), &[
            crate::i18n::tr("Fichier ▸ Exporter : page web (.html), jeu web installable (.zip pour itch.io, GitHub Pages…) ou application de bureau."),
        ]);
    });
    if !open {
        app.help = false;
    }
}
