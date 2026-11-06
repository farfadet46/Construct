//! Le moteur de jeu (portage de `Runtime` de runtime.js).

use crate::events::{Ev, EvKind};
use crate::gfx::{parse_color, Rgba};
use crate::model::*;
use pf_core::expr::{self, Cache, Env};
use pf_core::{Json, JsonExt, Map, Val};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub const DT: f64 = 1.0 / 60.0;

pub type Sol = HashMap<usize, Rc<Vec<Slot>>>;
/// clé spéciale de la sélection « Vision »
pub const LOS_KEY: usize = 1 << 40;

#[derive(Clone, Debug)]
pub enum AudioCmd {
    Sfx { name: String, vol: f64 },
    PlayFile { id: String, vol: f64, looping: bool, tag: String },
    Music { id: String, vol: f64, fade: f64 },
    StopMusic { fade: f64 },
    StopTag(String),
    StopAll,
    Volume(f64),
    Mute(bool),
    Engine { uid: u32, v: f64, load: f64, screech: f64, rumble: f64 },
    EngineStopAll,
}

/// ce que l'hôte (l'application) fournit : sons, stockage persistant
pub trait Host {
    fn audio(&mut self, _c: AudioCmd) {}
    fn tag_playing(&self, _tag: &str) -> bool {
        false
    }
    fn storage_load(&mut self, _ns: &str) -> Map<String, Json> {
        Map::new()
    }
    fn storage_save(&mut self, _ns: &str, _m: &Map<String, Json>) {}
    /// appelé à chaque image par l'application (fondus…)
    fn tick(&mut self) {}
}
pub struct NullHost;
impl Host for NullHost {}

#[derive(Clone)]
pub struct Scope {
    pub vars: HashMap<String, Val>,
    pub fn_: bool,
    pub ret: Val,
    pub st: Vec<Rc<Ev>>,
}
pub type ScopeRef = Rc<RefCell<Scope>>;

pub struct Wait {
    pub t: f64,
    pub acts: Rc<Vec<crate::events::Act>>,
    pub from: usize,
    pub subs: Rc<Vec<Rc<Ev>>>,
    pub sol: Sol,
    pub scopes: Vec<ScopeRef>,
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub px: f64,
    pub py: f64,
    pub opacity: f64,
    pub visible: bool,
}

#[derive(Clone, Debug, Default)]
pub struct LoopFr {
    pub name: String,
    pub i: f64,
    pub val: Option<Val>,
    pub key: Option<Val>,
}

#[derive(Clone, Debug, Default)]
pub struct Mouse {
    pub sx: f64,
    pub sy: f64,
    pub down: HashSet<String>,
    pub pr: HashSet<String>,
    pub rl: HashSet<String>,
    pub psx: Option<f64>,
    pub psy: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Joint {
    pub a: Slot,
    pub b: Option<Slot>,
    pub px: f64,
    pub py: f64,
    pub len: f64,
    pub rigid: bool,
}

#[derive(Clone, Debug)]
pub struct Dialog {
    pub name: String,
    pub text: String,
    pub portrait: String,
    pub top: bool,
    pub choices: Option<Vec<String>>,
    pub shown: usize,
    pub acc: f64,
    pub sel: usize,
    pub t: f64,
    pub hit: Option<Vec<[f64; 4]>>,
}

#[derive(Default)]
pub struct DlgState {
    pub queue: std::collections::VecDeque<Dialog>,
    pub cur: Option<Dialog>,
    pub speed: f64,
    pub choice: f64,
    pub choice_text: String,
    pub choice_tick: i64,
    pub end_tick: i64,
}

#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub layout: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct Trans {
    pub ty: String,
    pub dur: f64,
    pub color: String,
}
#[derive(Clone, Debug)]
pub struct TrState {
    pub phase_out: bool,
    pub t: f64,
    pub target: String,
    pub d: f64,
    pub ty: String,
    pub color: String,
}

#[derive(Clone, Debug)]
pub struct HsInput {
    pub title: String,
    pub name: Vec<char>,
    pub def: String,
    pub max: usize,
    pub cur: usize,
    pub t: f64,
    pub hit: Option<[f64; 4]>,
}

#[derive(Clone, Debug)]
pub struct ZoomAnim {
    pub from: f64,
    pub to: f64,
    pub t: f64,
    pub d: f64,
}

#[derive(Clone, Debug)]
pub enum DataStore {
    List(Vec<Val>),
    Map(Vec<(String, Val)>),
}
impl DataStore {
    pub fn map_get(&self, k: &str) -> Option<&Val> {
        if let DataStore::Map(m) = self {
            m.iter().find(|e| e.0 == k).map(|e| &e.1)
        } else {
            None
        }
    }
    pub fn map_set(&mut self, k: String, v: Val) {
        if let DataStore::Map(m) = self {
            if let Some(e) = m.iter_mut().find(|e| e.0 == k) {
                e.1 = v;
            } else {
                m.push((k, v));
            }
        }
    }
    pub fn to_json(&self) -> Json {
        match self {
            DataStore::List(l) => Json::Array(l.iter().map(|v| v.to_json()).collect()),
            DataStore::Map(m) => {
                let mut o = Map::new();
                for (k, v) in m {
                    o.insert(k.clone(), v.to_json());
                }
                Json::Object(o)
            }
        }
    }
}

/// dval() : convertit un texte numérique en nombre
pub fn dval(v: Val) -> Val {
    match v {
        Val::N(_) => v,
        Val::S(s) => {
            if s.is_empty() {
                return Val::S(s);
            }
            let n = pf_core::value::str_to_num(&s);
            if n.is_nan() {
                Val::S(s)
            } else {
                Val::N(n)
            }
        }
    }
}

pub struct CtlDef {
    pub name: String,
    pub keys: Vec<String>,
}
#[derive(Default)]
pub struct CtlState {
    pub open: bool,
    pub sel: usize,
    pub wait: bool,
    pub hit: Option<Vec<[f64; 4]>>,
}

pub struct Runtime {
    pub p: Rc<Json>,
    pub settings: Json,
    pub w: f64,
    pub h: f64,
    pub pixel_art: bool,
    pub types: Vec<Type>,
    pub type_by_id: HashMap<String, usize>,
    pub by_name: HashMap<String, usize>,
    pub insts: Vec<Option<Box<Inst>>>,
    pub all: Vec<Slot>,
    pub uidc: u32,
    pub solids: Vec<ORef>,
    pub jts: Vec<ORef>,
    pub ladders: Vec<ORef>,
    pub poly_count: usize,
    pub joints: Vec<Joint>,
    pub pending_ropes: Vec<(Slot, f64)>,
    pub last_blk: Option<ORef>,
    pub images: Vec<Option<Rc<tiny_skia::Pixmap>>>,
    pub image_src: Vec<String>,
    pub image_idx: HashMap<String, usize>,
    pub globals: Vec<(String, Val)>,
    pub waits: Vec<Wait>,
    pub layout: Json,
    pub layout_id: String,
    pub sheet: Option<Rc<Vec<Rc<Ev>>>>,
    pub sheet_id: String,
    pub sheets: Vec<(String, Rc<Vec<Rc<Ev>>>)>,
    pub cstate: HashMap<String, CState>,
    pub tick: i64,
    pub time: f64,
    pub first_tick: bool,
    pub pending: Option<String>,
    pub cam: (f64, f64),
    pub shake_t: f64,
    pub shake_a: f64,
    pub shake_d: f64,
    pub layers: Vec<Layer>,
    pub loop_stack: Vec<LoopFr>,
    pub scopes: Vec<ScopeRef>,
    pub group_state: HashMap<String, bool>,
    pub funcs: HashMap<String, Rc<Ev>>,
    pub groups_by_name: HashMap<String, Vec<Rc<Ev>>>,
    pub call_depth: usize,
    pub include_stack: Vec<String>,
    pub mouse: Mouse,
    pub keys: HashSet<String>,
    pub kp: HashSet<String>,
    pub pkp: HashSet<String>,
    pub kr: HashSet<String>,
    pub any_press: bool,
    pub cur: Option<Slot>,
    pub cur_sol: Sol,
    pub errors: Vec<String>,
    pub cache: Cache,
    pub rng: u64,
    pub lcg: Option<i64>,
    pub host: Box<dyn Host>,
    pub storage: Map<String, Json>,
    pub storage_ns: String,
    pub music: Option<String>,
    pub muted: bool,
    pub mvol: f64,
    pub engines: HashSet<u32>,
    // effets
    pub trans: Trans,
    pub tr: Option<TrState>,
    pub dark: (f64, String),
    pub tscale: f64,
    pub sflash: Option<(String, f64, f64)>,
    pub ds: Json,
    pub dlg: DlgState,
    pub cp: Option<Checkpoint>,
    pub cp_load: Option<Option<Checkpoint>>,
    pub cam_px: f64,
    pub cam_py: f64,
    pub ialpha: f64,
    pub zoom: f64,
    pub zoom_a: Option<ZoomAnim>,
    pub cam_look: Option<(f64, f64)>,
    pub cam_b: Option<[f64; 4]>,
    pub cam_la: f64,
    pub gpaused: bool,
    pub btn_focus: Option<Slot>,
    pub data: HashMap<usize, DataStore>,
    // classements
    pub hs_in: Option<HsInput>,
    pub hs_done: bool,
    pub hs_named_tick: i64,
    pub hs_name: String,
    pub hs_rank: f64,
    pub hs_last: Option<(String, f64)>,
    // cinématiques
    pub cine: Option<crate::cine::Cine>,
    pub cine_end_tick: i64,
    pub cine_end_id: String,
    pub cine_bars: f64,
    pub cine_title: Option<(String, f64, f64)>,
    pub cine_fade: (f64, String),
    // commandes
    pub ctl_defs: Vec<CtlDef>,
    pub cbind: Vec<Vec<String>>,
    pub ctl: CtlState,
    pub phys_down: HashMap<String, bool>,
    pub pad_prev: HashMap<String, bool>,
    pub pad_nav: HashMap<String, bool>,
    pub p2v: HashMap<String, Vec<String>>,
    pub v2p: HashMap<String, Vec<String>>,
    pub blocked: HashSet<String>,
    // boucle
    pub ready: bool,
    pub splash: bool,
    pub paused: bool,
    pub step_req: bool,
    pub acc: f64,
    pub dbg_speed: f64,
    pub show_col: bool,
    pub hl: Option<u32>,
    pub ev_hits: HashMap<String, u64>,
    pub bps: HashSet<String>,
    pub brk_hit: Option<(String, i64)>,
    pub start_id: String,
    pub embedded: bool,
    pub now_ms: f64,
}

#[derive(Clone, Debug, Default)]
pub struct CState {
    pub t: f64,
    pub last: i64,
    pub pairs: HashSet<(u32, u32)>,
    pub lv: Option<Val>,
}

pub struct Options {
    pub start_layout: Option<String>,
    pub splash: bool,
    pub embedded: bool,
    pub seed: Option<u64>,
    /// générateur « minimal standard » (identique au banc de test JS), pour les tests de parité
    pub lcg: Option<i64>,
}
impl Default for Options {
    fn default() -> Self {
        Options { start_layout: None, splash: false, embedded: true, seed: None, lcg: None }
    }
}

pub fn jstr(v: &Json) -> String {
    match v {
        Json::String(s) => s.clone(),
        Json::Number(n) => Val::N(n.as_f64().unwrap_or(0.0)).str(),
        Json::Bool(b) => b.to_string(),
        Json::Null => "undefined".into(),
        o => o.to_string(),
    }
}
pub fn jtruthy(v: Option<&Json>) -> bool {
    match v {
        Some(Json::Bool(b)) => *b,
        Some(Json::Number(n)) => n.as_f64().map(|x| x != 0.0 && !x.is_nan()).unwrap_or(false),
        Some(Json::String(s)) => !s.is_empty(),
        Some(Json::Null) | None => false,
        Some(_) => true,
    }
}

impl Runtime {
    pub fn new(p: Json, opts: Options, host: Box<dyn Host>) -> Runtime {
        let p = Rc::new(p);
        let mut settings = pf_core::json!({"width":854,"height":480,"pixelArt":true,"title":"Mon jeu"});
        if let Some(o) = p.get("settings").and_then(|s| s.as_object()) {
            for (k, v) in o {
                settings[k] = v.clone();
            }
        }
        let w = settings.f("width", 854.0);
        let h = settings.f("height", 480.0);
        let pixel_art = jtruthy(settings.get("pixelArt"));
        let mut rt = Runtime {
            p: p.clone(),
            w,
            h,
            pixel_art,
            types: Vec::new(),
            type_by_id: HashMap::new(),
            by_name: HashMap::new(),
            insts: Vec::new(),
            all: Vec::new(),
            uidc: 1,
            solids: Vec::new(),
            jts: Vec::new(),
            ladders: Vec::new(),
            poly_count: 0,
            joints: Vec::new(),
            pending_ropes: Vec::new(),
            last_blk: None,
            images: Vec::new(),
            image_src: Vec::new(),
            image_idx: HashMap::new(),
            globals: Vec::new(),
            waits: Vec::new(),
            layout: Json::Null,
            layout_id: String::new(),
            sheet: None,
            sheet_id: String::new(),
            sheets: Vec::new(),
            cstate: HashMap::new(),
            tick: 0,
            time: 0.0,
            first_tick: true,
            pending: None,
            cam: (0.0, 0.0),
            shake_t: 0.0,
            shake_a: 0.0,
            shake_d: 1.0,
            layers: Vec::new(),
            loop_stack: Vec::new(),
            scopes: Vec::new(),
            group_state: HashMap::new(),
            funcs: HashMap::new(),
            groups_by_name: HashMap::new(),
            call_depth: 0,
            include_stack: Vec::new(),
            mouse: Mouse { sx: -9999.0, sy: -9999.0, ..Default::default() },
            keys: HashSet::new(),
            kp: HashSet::new(),
            pkp: HashSet::new(),
            kr: HashSet::new(),
            any_press: false,
            cur: None,
            cur_sol: Sol::new(),
            errors: Vec::new(),
            cache: Cache::default(),
            rng: opts.seed.unwrap_or_else(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7)) | 1,
            lcg: opts.lcg,
            host,
            storage: Map::new(),
            storage_ns: format!("pf:{}:", settings.s("title")),
            music: None,
            muted: false,
            mvol: 1.0,
            engines: HashSet::new(),
            trans: Trans { ty: "fade".into(), dur: 0.5, color: "#000000".into() },
            tr: None,
            dark: (0.0, "#0b0b1e".into()),
            tscale: 1.0,
            sflash: None,
            ds: pf_core::json!({"bg":"#1b1f2a","fg":"#ffffff","accent":"#ffd84d","font":"round","blip":true}),
            dlg: DlgState { speed: 45.0, choice_tick: -1, end_tick: -1, ..Default::default() },
            cp: None,
            cp_load: None,
            cam_px: 0.0,
            cam_py: 0.0,
            ialpha: 1.0,
            zoom: 1.0,
            zoom_a: None,
            cam_look: None,
            cam_b: None,
            cam_la: 0.0,
            gpaused: false,
            btn_focus: None,
            data: HashMap::new(),
            hs_in: None,
            hs_done: false,
            hs_named_tick: -1,
            hs_name: String::new(),
            hs_rank: 0.0,
            hs_last: None,
            cine: None,
            cine_end_tick: -1,
            cine_end_id: String::new(),
            cine_bars: 0.0,
            cine_title: None,
            cine_fade: (0.0, "#000000".into()),
            ctl_defs: Vec::new(),
            cbind: Vec::new(),
            ctl: CtlState::default(),
            phys_down: HashMap::new(),
            pad_prev: HashMap::new(),
            pad_nav: HashMap::new(),
            p2v: HashMap::new(),
            v2p: HashMap::new(),
            blocked: HashSet::new(),
            ready: false,
            splash: opts.splash,
            paused: false,
            step_req: false,
            acc: 0.0,
            dbg_speed: 1.0,
            show_col: false,
            hl: None,
            ev_hits: HashMap::new(),
            bps: HashSet::new(),
            brk_hit: None,
            start_id: String::new(),
            embedded: opts.embedded,
            now_ms: 0.0,
            settings,
        };
        if let Some(t) = rt.settings.get("transition").and_then(|t| t.as_object()) {
            if let Some(v) = t.get("type").and_then(|v| v.as_str()) {
                rt.trans.ty = v.to_string();
            }
            if let Some(v) = t.get("dur").and_then(|v| v.as_f64()) {
                rt.trans.dur = v;
            }
            if let Some(v) = t.get("color").and_then(|v| v.as_str()) {
                rt.trans.color = v.to_string();
            }
        }
        if let Some(d) = rt.settings.get("dialog").and_then(|t| t.as_object()) {
            for (k, v) in d {
                rt.ds[k] = v.clone();
            }
        }
        rt.storage = rt.host.storage_load(&rt.storage_ns.clone());
        // types
        let objs: Vec<Json> = p.arr("objects").to_vec();
        for o in &objs {
            let def = {
                let mut f = |s: &str| rt.intern_image(s);
                Def::parse(o, &mut f)
            };
            let mut bh = HashMap::new();
            for b in o.arr("behaviors") {
                let t = b.s("type").to_string();
                let mut m = pf_core::catalog::bdefaults(&t);
                if let Some(pp) = b.get("props").and_then(|v| v.as_object()) {
                    for (k, v) in pp {
                        m.insert(k.clone(), v.clone());
                    }
                }
                bh.insert(t, m);
            }
            let idx = rt.types.len();
            let id = o.s("id").to_string();
            let name = o.s("name").to_string();
            rt.type_by_id.insert(id.clone(), idx);
            rt.by_name.insert(name.clone(), idx);
            rt.types.push(Type {
                solid: bh.contains_key("solid"),
                jumpthru: bh.contains_key("jumpthru"),
                ladder: bh.contains_key("ladder"),
                def: Rc::new(def),
                id,
                name,
                insts: Vec::new(),
                bh,
                fam: false,
                mem: HashSet::new(),
                mem_list: Vec::new(),
                fams: Vec::new(),
                tpl_path: None,
            });
        }
        for (k, o) in objs.iter().enumerate() {
            if o.s("kind") != "family" {
                continue;
            }
            rt.types[k].fam = true;
            for m in o.arr("members") {
                if let Some(&mi) = m.as_str().and_then(|s| rt.type_by_id.get(s)) {
                    if rt.types[mi].def.kind != Kind::Family {
                        rt.types[k].mem.insert(mi);
                        rt.types[k].mem_list.push(mi);
                        rt.types[mi].fams.push(k);
                    }
                }
            }
        }
        rt.reset_g();
        // feuilles d'événements
        for s in p.arr("sheets") {
            let evs = Rc::new(s.arr("events").iter().map(|e| Rc::new(Ev::parse(e, &rt.type_by_id))).collect::<Vec<_>>());
            rt.sheets.push((s.s("id").to_string(), evs));
        }
        fn walk(rt: &mut Runtime, list: &[Rc<Ev>]) {
            for e in list {
                if e.kind == EvKind::Function && !e.name.is_empty() {
                    rt.funcs.insert(e.name.clone(), e.clone());
                }
                if e.kind == EvKind::Group {
                    rt.groups_by_name.entry(e.name.clone()).or_default().push(e.clone());
                    let subs = e.subs.clone();
                    walk(rt, &subs);
                }
            }
        }
        let sheets: Vec<Rc<Vec<Rc<Ev>>>> = rt.sheets.iter().map(|s| s.1.clone()).collect();
        for s in sheets {
            walk(&mut rt, &s);
        }
        rt.hs_name = rt.sget("hs:name").map(|v| jstr(&v)).unwrap_or_default();
        rt.data_init();
        rt.ctl_init();
        rt.start_id = opts.start_layout.clone().or_else(|| rt.settings.get("startLayout").and_then(|v| v.as_str()).map(|s| s.to_string())).filter(|s| !s.is_empty()).unwrap_or_else(|| p.arr("layouts").first().map(|l| l.s("id").to_string()).unwrap_or_default());
        if !rt.splash {
            rt.boot();
        }
        rt
    }

    pub fn boot(&mut self) {
        self.ready = true;
        let id = self.start_id.clone();
        self.load_layout(&id);
    }

    fn intern_image(&mut self, s: &str) -> usize {
        if let Some(&k) = self.image_idx.get(s) {
            return k;
        }
        let k = self.images.len();
        let pm = pf_core::data_url_bytes(s).and_then(|b| crate::img::decode(&b)).map(Rc::new);
        self.images.push(pm);
        self.image_src.push(s.to_string());
        self.image_idx.insert(s.to_string(), k);
        k
    }
    pub fn image(&self, k: usize) -> Option<&Rc<tiny_skia::Pixmap>> {
        self.images.get(k).and_then(|x| x.as_ref())
    }

    pub fn reset_g(&mut self) {
        let p = self.p.clone();
        let mut g = Vec::new();
        for v in p.arr("globals") {
            let val = if v.s("type") == "text" {
                Val::S(match v.get("value") {
                    None | Some(Json::Null) => String::new(),
                    Some(x) => jstr(x),
                })
            } else {
                Val::N(Val::from_json(v.get("value").unwrap_or(&Json::Null)).num0())
            };
            let name = v.s("name").to_string();
            if let Some(e) = g.iter_mut().find(|e: &&mut (String, Val)| e.0 == name) {
                e.1 = val;
            } else {
                g.push((name, val));
            }
        }
        self.globals = g;
    }
    pub fn global(&self, n: &str) -> Option<&Val> {
        self.globals.iter().find(|e| e.0 == n).map(|e| &e.1)
    }
    pub fn set_global(&mut self, n: &str, v: Val) {
        if let Some(e) = self.globals.iter_mut().find(|e| e.0 == n) {
            e.1 = v;
        } else {
            self.globals.push((n.to_string(), v));
        }
    }
    pub fn find_var_scope(&self, n: &str) -> Option<ScopeRef> {
        for s in self.scopes.iter().rev() {
            if s.borrow().vars.contains_key(n) {
                return Some(s.clone());
            }
        }
        None
    }
    pub fn get_var(&self, n: &str) -> Option<Val> {
        if let Some(s) = self.find_var_scope(n) {
            return s.borrow().vars.get(n).cloned();
        }
        self.global(n).cloned()
    }
    pub fn set_var(&mut self, n: &str, v: Val) {
        if let Some(s) = self.find_var_scope(n) {
            s.borrow_mut().vars.insert(n.to_string(), v);
        } else {
            self.set_global(n, v);
        }
    }

    pub fn rand(&mut self) -> f64 {
        if let Some(s) = self.lcg.as_mut() {
            *s = (*s * 16807) % 2147483647;
            return (*s - 1) as f64 / 2147483646.0;
        }
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        ((x.wrapping_mul(0x2545F4914F6CDD1D) >> 11) as f64) / ((1u64 << 53) as f64)
    }

    // ---------------------------------------------------------------- stockage

    pub fn sget(&self, k: &str) -> Option<Json> {
        self.storage.get(k).cloned()
    }
    pub fn sset(&mut self, k: &str, v: Json) {
        self.storage.insert(k.to_string(), v);
        let ns = self.storage_ns.clone();
        let st = self.storage.clone();
        self.host.storage_save(&ns, &st);
    }
    pub fn sdel(&mut self, k: &str) {
        self.storage.remove(k);
        let ns = self.storage_ns.clone();
        let st = self.storage.clone();
        self.host.storage_save(&ns, &st);
    }
    pub fn sclear(&mut self) {
        self.storage.clear();
        let ns = self.storage_ns.clone();
        self.host.storage_save(&ns, &Map::new());
    }

    // ---------------------------------------------------------------- audio
    pub fn sfx(&mut self, n: &str) {
        self.sfx_v(n, 1.0);
    }
    pub fn sfx_v(&mut self, n: &str, vol: f64) {
        self.host.audio(AudioCmd::Sfx { name: n.to_string(), vol });
    }

    // ---------------------------------------------------------------- instances

    pub fn inst(&self, s: Slot) -> &Inst {
        self.insts[s as usize].as_deref().expect("instance")
    }
    pub fn inst_mut(&mut self, s: Slot) -> &mut Inst {
        self.insts[s as usize].as_deref_mut().expect("instance")
    }
    pub fn get(&self, s: Slot) -> Option<&Inst> {
        self.insts.get(s as usize).and_then(|x| x.as_deref())
    }
    pub fn alive(&self, s: Slot) -> bool {
        self.get(s).map(|i| !i.dead).unwrap_or(false)
    }
    pub fn def(&self, s: Slot) -> Rc<Def> {
        self.types[self.inst(s).t].def.clone()
    }
    pub fn in_type(&self, s: Slot, t: usize) -> bool {
        let it = self.inst(s).t;
        it == t || (self.types[t].fam && self.types[t].mem.contains(&it))
    }
    pub fn picked(&self, sol: &Sol, t: usize) -> Vec<Slot> {
        match sol.get(&t) {
            Some(v) => v.iter().copied().filter(|&s| self.alive(s)).collect(),
            None => self.types[t].insts.iter().copied().filter(|&s| self.alive(s)).collect(),
        }
    }
    pub fn alive_insts(&self, t: usize) -> Vec<Slot> {
        self.types[t].insts.iter().copied().filter(|&s| self.alive(s)).collect()
    }
    pub fn layer_idx(&self, nm: &str) -> Option<usize> {
        self.layers.iter().position(|l| l.name == nm)
    }
    pub fn main_layer(&self) -> Layer {
        for l in &self.layers {
            if l.px == 1.0 && l.py == 1.0 {
                return l.clone();
            }
        }
        self.layers.first().cloned().unwrap_or(Layer { id: "_".into(), name: String::new(), px: 1.0, py: 1.0, opacity: 1.0, visible: true })
    }
    pub fn zf(&self, li: usize, hud: bool) -> f64 {
        let l = self.layers.get(li);
        if hud || l.map(|l| l.px == 0.0 && l.py == 0.0).unwrap_or(false) {
            1.0
        } else {
            self.zoom
        }
    }
    pub fn layer_pp(&self, li: usize) -> (f64, f64) {
        self.layers.get(li).map(|l| (l.px, l.py)).unwrap_or((1.0, 1.0))
    }

    pub fn create_inst(&mut self, t: usize, d: Option<&Json>, x: f64, y: f64, li: Option<usize>) -> Option<Slot> {
        let mut t = t;
        if self.types[t].fam {
            if self.types[t].mem_list.is_empty() {
                return None;
            }
            let k = (self.rand() * self.types[t].mem_list.len() as f64) as usize;
            t = self.types[t].mem_list[k.min(self.types[t].mem_list.len() - 1)];
        }
        let o = self.types[t].def.clone();
        let dw = d.map(|d| d.f("w", 0.0)).unwrap_or(0.0);
        let dh = d.map(|d| d.f("h", 0.0)).unwrap_or(0.0);
        let mut w = if dw != 0.0 { dw } else { o.j.f_or("defW", 0.0) };
        if w == 0.0 {
            w = if o.w != 0.0 { o.w } else { 32.0 };
        }
        let mut h = if dh != 0.0 { dh } else { o.j.f_or("defH", 0.0) };
        if h == 0.0 {
            h = if o.h != 0.0 { o.h } else { 32.0 };
        }
        let text = match d.and_then(|d| d.get("text")).filter(|v| !v.is_null()) {
            Some(v) => Val::from_json(v),
            None => match o.j.get("text") {
                Some(v) if jtruthy(Some(v)) => Val::from_json(v),
                _ => Val::S(String::new()),
            },
        };
        let uid = self.uidc;
        self.uidc += 1;
        let mut i = Inst {
            uid,
            t,
            x: d.map(|d| d.f("x", 0.0)).unwrap_or(x),
            y: d.map(|d| d.f("y", 0.0)).unwrap_or(y),
            w,
            h,
            angle: d.map(|d| d.f("angle", 0.0)).unwrap_or(0.0),
            mirrored: d.map(|d| jtruthy(d.get("mirrored"))).unwrap_or(false),
            opacity: d.and_then(|d| d.get("opacity")).filter(|v| !v.is_null()).map(|v| Val::from_json(v).num()).unwrap_or(1.0),
            visible: d.map(|d| d.get("visible") != Some(&Json::Bool(false))).unwrap_or(true),
            hud: d.map(|d| jtruthy(d.get("hud"))).unwrap_or(false),
            text,
            layer: 0,
            vars: HashMap::new(),
            anim: 0,
            frame: 0,
            ft: 0.0,
            anim_end: -1,
            dead: false,
            kill: false,
            flash: 0.0,
            b: Behs::default(),
            tm: None,
            bar: None,
            btn: None,
            ps: None,
            style: None,
            tint: None,
            tf: None,
            px: 0.0,
            py: 0.0,
            dx: 0.0,
            dy: 0.0,
            sf: None,
            nojt: false,
            plv: 0,
            pmv: 0.0,
            lbt: String::new(),
            lb: Vec::new(),
            lb_hi: None,
            mm_r: None,
            rd: None,
        };
        i.px = i.x;
        i.py = i.y;
        if let Some(d) = d {
            let lid = d.s("layer");
            i.layer = self.layers.iter().position(|l| l.id == lid).unwrap_or(0);
        } else if let Some(li) = li {
            i.layer = li;
        }
        let jo = &o.j;
        match o.kind {
            Kind::Tilemap => {
                let cols = d.map(|d| d.f_or("cols", 0.0)).filter(|v| *v != 0.0).unwrap_or_else(|| jo.f_or("defCols", 8.0)) as i64;
                let rows = d.map(|d| d.f_or("rows", 0.0)).filter(|v| *v != 0.0).unwrap_or_else(|| jo.f_or("defRows", 4.0)) as i64;
                let dt = d.map(|d| d.arr("data").to_vec()).unwrap_or_default();
                let n = (cols * rows).max(0) as usize;
                let data: Vec<i64> = (0..n).map(|q| dt.get(q).filter(|v| !v.is_null()).map(|v| v.as_f64().unwrap_or(-1.0) as i64).unwrap_or(-1)).collect();
                i.tm = Some(TileData { cols, rows, data, ..Default::default() });
                i.w = cols as f64 * jo.f_or("tw", 16.0) * o.scale;
                i.h = rows as f64 * jo.f_or("th", 16.0) * o.scale;
            }
            Kind::Road3d => {
                i.rd = Some(Box::new(crate::road::RoadSt::default()));
            }
            Kind::Bar => {
                let bmx = { let m = jo.f("max", 100.0); if m == 0.0 || m.is_nan() { 100.0 } else { m } }.max(0.0001);
                let mut bv = d.and_then(|d| d.get("value")).filter(|v| !v.is_null()).map(|v| Val::from_json(v).num()).or_else(|| jo.get("value").filter(|v| !v.is_null()).map(|v| Val::from_json(v).num())).unwrap_or(bmx);
                bv = bv.min(bmx).max(0.0);
                i.bar = Some(BarSt { v: bv, max: bmx, shown: bv, ghost: bv, gt: 0.0, col: String::new() });
            }
            Kind::Leaderboard => {
                i.lbt = d.map(|d| d.s("table").to_string()).filter(|s| !s.is_empty()).or_else(|| Some(jo.s("table").to_string()).filter(|s| !s.is_empty())).unwrap_or_else(|| "scores".into());
                i.lb = self.hs_list(&i.lbt.clone());
                i.lb_hi = self.hs_last.as_ref().filter(|h| h.0 == i.lbt).map(|h| h.1);
            }
            Kind::Button => {
                i.btn = Some(BtnSt { en: true, clicked: -1, ..Default::default() });
                if d.map(|d| d.get("text").map(|v| v.is_null()).unwrap_or(true)).unwrap_or(false) {
                    i.text = Val::S(jo.s("text").to_string());
                }
            }
            Kind::Particles => {
                let im = jo.get("pt").map(|p| p.s("image").to_string()).filter(|s| !s.is_empty()).and_then(|id| self.type_by_id.get(&id).copied()).and_then(|ti| {
                    let d = &self.types[ti].def;
                    if d.kind == Kind::Sprite {
                        d.anims.first().and_then(|a| a.frames.first().copied())
                    } else {
                        None
                    }
                });
                i.ps = Some(PartSt { active: true, img: im, ..Default::default() });
            }
            _ => {}
        }
        if let Some(st) = d.and_then(|d| d.get("style")).and_then(|v| v.as_object()) {
            i.style = Some(st.clone());
        }
        // variables d'instance (+ celles des familles)
        let mut ivd = o.vars.clone();
        for &ft in &self.types[t].fams.clone() {
            for v in &self.types[ft].def.vars {
                if !ivd.iter().any(|x| x.name == v.name) {
                    ivd.push(v.clone());
                }
            }
        }
        for v in &ivd {
            let val = d.and_then(|d| d.get("vars")).and_then(|m| m.get(&v.name)).filter(|x| !x.is_null()).cloned().unwrap_or_else(|| v.value.clone());
            let vv = if v.text {
                Val::S(if val.is_null() { String::new() } else { jstr(&val) })
            } else {
                Val::N(Val::from_json(&val).num0())
            };
            i.vars.insert(v.name.clone(), vv);
        }
        // comportements
        let bh = self.types[t].bh.clone();
        for (k, p) in &bh {
            match k.as_str() {
                "platform" => {
                    i.b.platform = Some(PlatformSt {
                        max_speed: pf(p, "maxSpeed"),
                        accel: pf(p, "accel"),
                        decel: pf(p, "decel"),
                        jump: pf(p, "jump"),
                        gravity: pf(p, "gravity"),
                        max_fall: pf(p, "maxFall"),
                        double_jump: pb(p, "doubleJump"),
                        wall_jump: pb(p, "wallJump"),
                        wall_slide: pf(p, "wallSlide"),
                        wall_kick: pf(p, "wallKick"),
                        climb_speed: pf(p, "climbSpeed"),
                        controls: pb(p, "controls"),
                        en: true,
                        wj: -1,
                        ..Default::default()
                    })
                }
                "patrol" => {
                    i.b.patrol = Some(PatrolSt {
                        speed: pf(p, "speed"),
                        ledges: pb(p, "ledges"),
                        flip: pb(p, "flip"),
                        gravity: pf(p, "gravity"),
                        chase: ps(p, "chase"),
                        range: pf(p, "range"),
                        chase_speed: pf(p, "chaseSpeed"),
                        jump: pf(p, "jump"),
                        dir: if pb(p, "startLeft") { -1.0 } else { 1.0 },
                        spot: -1,
                        ..Default::default()
                    })
                }
                "sine" => i.b.sine = Some(SineSt { vertical: ps(p, "axis") == "v", period: pf(p, "period"), magnitude: pf(p, "magnitude"), bx: i.x, by: i.y, t: 0.0, en: true }),
                "path" => {
                    let dpath: Option<Vec<[f64; 2]>> = d.and_then(|d| d.get("path")).and_then(|v| v.as_array()).filter(|a| !a.is_empty()).map(|a| a.iter().map(|q| [Val::from_json(&q[0]).num0(), Val::from_json(&q[1]).num0()]).collect());
                    if let Some(dp) = &dpath {
                        if self.types[t].tpl_path.is_none() {
                            self.types[t].tpl_path = Some(dp.clone());
                        }
                    }
                    let src = if d.is_some() { dpath.clone().unwrap_or_default() } else { self.types[t].tpl_path.clone().unwrap_or_default() };
                    let mut pts = vec![[0.0, 0.0]];
                    pts.extend(src);
                    i.b.path = Some(PathSt { speed: pf(p, "speed"), mode: ps(p, "mode"), pause: pf(p, "pause"), ease: pb(p, "ease"), rotate: pb(p, "rotate"), bx: i.x, by: i.y, pts, k: 0, nx: 1, dir: 1, u: 0.0, wt: 0.0, en: pb(p, "auto"), arr: -1, fin: -1, last: 0 });
                }
                "timer" => i.b.timer = Some(TimerSt::default()),
                "bullet" => {
                    let a = pf(p, "angle") * std::f64::consts::PI / 180.0;
                    let sp = pf(p, "speed");
                    i.b.bullet = Some(BulletSt { speed: sp, angle: pf(p, "angle"), gravity: pf(p, "gravity"), vx: a.cos() * sp, vy: a.sin() * sp });
                }
                "eightdir" => i.b.eightdir = Some(EightSt { max_speed: pf(p, "maxSpeed"), accel: pf(p, "accel"), decel: pf(p, "decel"), dirs: ps(p, "dirs"), rotate: pb(p, "rotate"), controls: pb(p, "controls"), en: true, ..Default::default() }),
                "car" => {
                    i.b.car = Some(CarSt {
                        max_speed: pf(p, "maxSpeed"),
                        accel: pf(p, "accel"),
                        brake: pf(p, "brake"),
                        friction: pf(p, "friction"),
                        reverse: pf(p, "reverse"),
                        steer: pf(p, "steer"),
                        drift: pf(p, "drift"),
                        engine: p.get("engine") != Some(&Json::Bool(false)),
                        en: true,
                        ctl: pb(p, "controls"),
                        hit: -1,
                        ..Default::default()
                    })
                }
                "tween" => i.b.tween = Some(TweenSt::default()),
                "physics" => {
                    i.b.physics = Some(PhysSt {
                        circle: ps(p, "shape") == "circle",
                        density: pf(p, "density"),
                        friction: pf(p, "friction"),
                        bounce: pf(p, "bounce"),
                        gravity: pf(p, "gravity"),
                        pushable: pb(p, "pushable"),
                        fixed: pb(p, "immovable"),
                        hit_t: -1,
                        push_t: -1,
                        ..Default::default()
                    });
                }
                "fade" => {
                    let run = pb(p, "auto");
                    i.b.fade = Some(FadeSt { fade_in: pf(p, "fadeIn"), wait: pf(p, "wait"), fade_out: pf(p, "fadeOut"), destroy: pb(p, "destroy"), t: 0.0, run, base: i.opacity, done: -1 });
                    if run && pf(p, "fadeIn") > 0.0 {
                        i.opacity = 0.0;
                    }
                }
                "pin" => i.b.pin = Some(PinSt { mirror: pb(p, "mirror"), angle: pb(p, "angle"), destroy: pb(p, "destroy"), ..Default::default() }),
                "drag" => i.b.drag = Some(DragSt { axes: ps(p, "axes"), en: true, st: -1, dr: -1, ..Default::default() }),
                "turret" => i.b.turret = Some(TurretSt { range: pf(p, "range"), rate: pf(p, "rate"), rot_speed: pf(p, "rotSpeed"), rotate: pb(p, "rotate"), fired: -1, ..Default::default() }),
                "los" => i.b.los = Some(LosSt { range: pf(p, "range"), cone: pf(p, "cone") }),
                "light" => i.b.light = Some(LightSt { radius: pf(p, "radius"), intensity: if p.get("intensity").map(|v| v.is_null()).unwrap_or(true) { 100.0 } else { pf(p, "intensity") }, color: ps(p, "color"), glow: pf(p, "glow"), flicker: pb(p, "flicker") }),
                "scrollto" => i.b.scrollto = Some(ScrollSt { smooth: pf(p, "smooth"), dead_w: pf(p, "deadW"), dead_h: pf(p, "deadH"), lookahead: pf(p, "lookahead"), offset_y: pf(p, "offsetY") }),
                "rotate" => i.b.rotate = Some(pf(p, "speed")),
                "bound" => i.b.bound = Some(pb(p, "bottom")),
                "destroy" => i.b.destroy = true,
                _ => {}
            }
        }
        let rope = bh.get("physics").map(|p| pf(p, "rope")).unwrap_or(0.0);
        let slot = self.insts.len() as Slot;
        self.insts.push(Some(Box::new(i)));
        if rope > 0.0 {
            self.pending_ropes.push((slot, rope));
        }
        if o.kind == Kind::Road3d {
            self.rd_init(slot, false);
        }
        self.types[t].insts.push(slot);
        for ft in self.types[t].fams.clone() {
            self.types[ft].insts.push(slot);
        }
        self.all.push(slot);
        Some(slot)
    }

    // ---------------------------------------------------------------- niveau

    pub fn load_layout(&mut self, id: &str) {
        self.host.audio(AudioCmd::EngineStopAll);
        self.engines.clear();
        self.joints.clear();
        self.pending_ropes.clear();
        self.cine = None;
        self.cine_bars = 0.0;
        self.cine_title = None;
        self.cine_fade.0 = 0.0;
        if let Some(cl) = self.cp_load.take() {
            self.cp = cl.filter(|c| c.layout == id);
        }
        if self.cp.as_ref().map(|c| c.layout != id).unwrap_or(false) {
            self.cp = None;
        }
        self.gpaused = false;
        self.btn_focus = None;
        self.zoom = 1.0;
        self.zoom_a = None;
        self.cam_look = None;
        self.cam_b = None;
        self.cam_la = 0.0;
        self.dark.0 = 0.0;
        self.tscale = 1.0;
        self.dlg.queue.clear();
        self.dlg.cur = None;
        let p = self.p.clone();
        let layouts = p.arr("layouts");
        let l = layouts.iter().find(|l| l.s("id") == id).or_else(|| layouts.first()).cloned().unwrap_or(Json::Null);
        self.layout_id = l.s("id").to_string();
        let sid = l.s("sheet").to_string();
        self.sheet = self.sheets.iter().find(|s| s.0 == sid).map(|s| s.1.clone());
        self.sheet_id = if self.sheet.is_some() { sid } else { String::new() };
        self.all.clear();
        self.insts.clear();
        for t in &mut self.types {
            t.insts.clear();
            t.tpl_path = None;
        }
        self.cstate.clear();
        self.waits.clear();
        self.loop_stack.clear();
        self.scopes.clear();
        self.group_state.clear();
        self.first_tick = true;
        self.shake_t = 0.0;
        let ls = l.arr("layers");
        self.layers = if ls.is_empty() {
            vec![Layer { id: "_".into(), name: "Principal".into(), px: 1.0, py: 1.0, opacity: 1.0, visible: true }]
        } else {
            ls.iter().map(|y| {
                let g = |k: &str| y.get(k).filter(|v| !v.is_null()).map(|v| Val::from_json(v).num()).unwrap_or(100.0) / 100.0;
                Layer { id: y.s("id").to_string(), name: y.s("name").to_string(), px: g("parallaxX"), py: g("parallaxY"), opacity: g("opacity"), visible: y.get("visible") != Some(&Json::Bool(false)) }
            }).collect()
        };
        self.layout = l.clone();
        for d in l.arr("instances") {
            if jtruthy(d.get("off")) {
                continue;
            }
            if let Some(&t) = self.type_by_id.get(d.s("obj")) {
                self.create_inst(t, Some(d), 0.0, 0.0, None);
            }
        }
        self.update_cam(0.0);
    }
    pub fn lw(&self) -> f64 {
        self.layout.f("w", 0.0)
    }
    pub fn lh(&self) -> f64 {
        self.layout.f("h", 0.0)
    }

    pub fn data_init(&mut self) {
        self.data.clear();
        for (k, t) in self.types.iter().enumerate() {
            match t.def.kind {
                Kind::Array => {
                    self.data.insert(k, DataStore::List(t.def.j.arr("items").iter().map(Val::from_json).collect()));
                }
                Kind::Dict => {
                    let m = t.def.j.get("entries").and_then(|e| e.as_object()).map(|o| o.iter().map(|(k, v)| (k.clone(), Val::from_json(v))).collect()).unwrap_or_default();
                    self.data.insert(k, DataStore::Map(m));
                }
                _ => {}
            }
        }
    }

    // ---------------------------------------------------------------- expressions

    pub fn x(&mut self, src: &Json) -> Val {
        let s = match src {
            Json::Null => String::new(),
            v => jstr(v),
        };
        self.xs(&s)
    }
    pub fn xs(&mut self, s: &str) -> Val {
        let e = self.cache.get(s);
        match &*e {
            Ok(ex) => expr::eval(ex, self),
            Err(m) => {
                if self.errors.len() < 20 && !self.errors.contains(m) {
                    self.errors.push(m.clone());
                }
                Val::N(0.0)
            }
        }
    }
    pub fn xn(&mut self, src: &Json) -> f64 {
        self.x(src).num0()
    }

    pub fn prop(&self, s: Slot, p: &str) -> Val {
        let i = self.inst(s);
        let o = &self.types[i.t].def;
        let rd = i.rd.as_deref();
        let b = &i.b;
        let n = |v: f64| Val::N(v);
        match p {
            "X" => n(i.x),
            "Y" => n(i.y),
            "Width" => n(i.w),
            "Height" => n(i.h),
            "Angle" => n(i.angle),
            "Opacity" => n(pf_core::value::js_round(i.opacity * 100.0)),
            "UID" => n(i.uid as f64),
            "AnimationName" => Val::S(if o.kind == Kind::Sprite { o.anims.get(i.anim).map(|a| a.name.clone()).unwrap_or_default() } else { String::new() }),
            "AnimationFrame" => n(i.frame as f64),
            "Text" => i.text.clone(),
            "Distance" => n(rd.map(|r| r.dist).unwrap_or(0.0)),
            "Lap" => n(rd.map(|r| r.lap as f64).unwrap_or(0.0)),
            "LapTime" => n(rd.map(|r| r.lap_time).unwrap_or(0.0)),
            "LastLap" => n(rd.map(|r| r.last).unwrap_or(0.0)),
            "BestLap" => n(rd.map(|r| r.best).unwrap_or(0.0)),
            "Rank" => n(if rd.is_some() { self.rd_rank(s) } else { 0.0 }),
            "Rivals" => n(rd.map(|r| r.cars.len() as f64).unwrap_or(0.0)),
            "PlayerX" => n(rd.map(|r| r.x).unwrap_or(0.0)),
            "Curve" => n(if rd.is_some() { self.rd_curve(s) } else { 0.0 }),
            "SkyOffset" => n(rd.map(|r| r.sky).unwrap_or(0.0)),
            "Progress" => n(if rd.is_some() { self.rd_prog(s) } else { 0.0 }),
            "Steer" => n(rd.map(|r| r.steer).unwrap_or(0.0)),
            "Horizon" => n(rd.map(|r| i.y + r.hz.unwrap_or(i.h / 2.0)).unwrap_or(0.0)),
            "Speed" => n(if rd.is_some() {
                self.rd_kmh(s)
            } else if let Some(c) = &b.car {
                c.sp.abs()
            } else if let Some(e) = &b.eightdir {
                e.vx.hypot(e.vy)
            } else if let Some(pl) = &b.platform {
                pl.vx.abs()
            } else if let Some(u) = &b.bullet {
                u.vx.hypot(u.vy)
            } else if let Some(ph) = &b.physics {
                ph.vx.hypot(ph.vy)
            } else {
                0.0
            }),
            "VectorX" => n(b.car.as_ref().map(|c| c.vx).or_else(|| b.platform.as_ref().map(|c| c.vx)).or_else(|| b.bullet.as_ref().map(|c| c.vx)).or_else(|| b.physics.as_ref().map(|c| c.vx)).unwrap_or(0.0)),
            "VectorY" => n(b.car.as_ref().map(|c| c.vy).or_else(|| b.platform.as_ref().map(|c| c.vy)).or_else(|| b.bullet.as_ref().map(|c| c.vy)).or_else(|| b.physics.as_ref().map(|c| c.vy)).unwrap_or(0.0)),
            "IsOnFloor" => n(if b.platform.as_ref().map(|p| p.floor.is_some()).unwrap_or(false) { 1.0 } else { 0.0 }),
            "PathPoint" => n(b.path.as_ref().map(|p| p.last as f64).unwrap_or(0.0)),
            "Value" => n(i.bar.as_ref().map(|b| b.v).unwrap_or(0.0)),
            "MaxValue" => n(i.bar.as_ref().map(|b| b.max).unwrap_or(0.0)),
            "ParticleCount" => n(i.ps.as_ref().map(|p| p.parts.len() as f64).unwrap_or(0.0)),
            "Columns" => n(i.tm.as_ref().map(|t| t.cols as f64).unwrap_or(0.0)),
            "Rows" => n(i.tm.as_ref().map(|t| t.rows as f64).unwrap_or(0.0)),
            _ => i.vars.get(p).cloned().unwrap_or(Val::N(0.0)),
        }
    }

    fn expr_target(&self, n: &str) -> Option<Slot> {
        if n == "Self" {
            return self.cur;
        }
        let &t = self.by_name.get(n)?;
        if let Some(c) = self.cur {
            if self.in_type(c, t) {
                return Some(c);
            }
        }
        self.picked(&self.cur_sol, t).first().copied()
    }

    // ---------------------------------------------------------------- caméra

    pub fn update_cam(&mut self, dt: f64) {
        let (w, h) = (self.w, self.h);
        if let (Some(za), true) = (self.zoom_a.as_mut(), dt != 0.0) {
            za.t += dt;
            let zk = (za.t / za.d).min(1.0);
            let ze = if zk < 0.5 { 2.0 * zk * zk } else { 1.0 - (-2.0 * zk + 2.0).powi(2) / 2.0 };
            let ocx = self.cam.0 + w / self.zoom / 2.0;
            let ocy = self.cam.1 + h / self.zoom / 2.0;
            self.zoom = za.from + (za.to - za.from) * ze;
            self.cam.0 = ocx - w / self.zoom / 2.0;
            self.cam.1 = ocy - h / self.zoom / 2.0;
            if zk >= 1.0 {
                self.zoom_a = None;
            }
        }
        let vw = w / self.zoom;
        let vh = h / self.zoom;
        let tg: Vec<Slot> = self.all.iter().copied().filter(|&s| {
            let i = self.inst(s);
            !i.dead && i.b.scrollto.is_some()
        }).collect();
        let p0 = tg.first().map(|&s| self.inst(s).b.scrollto.clone().unwrap()).unwrap_or_default();
        let mut tx = None;
        let mut ty = 0.0;
        if let Some(cl) = self.cam_look {
            tx = Some(cl.0);
            ty = cl.1;
        } else if !tg.is_empty() {
            let (mut x, mut y) = (0.0, 0.0);
            for &s in &tg {
                let b = self.box_i(s);
                x += (b.l + b.r) / 2.0;
                y += (b.t + b.b) / 2.0;
            }
            let mut txx = x / tg.len() as f64;
            ty = y / tg.len() as f64 + p0.offset_y;
            if p0.lookahead != 0.0 {
                let i0 = self.inst(tg[0]);
                let mv = if let Some(p) = &i0.b.platform { p.vx } else if let Some(e) = &i0.b.eightdir { e.vx } else { (i0.x - i0.px) * 60.0 };
                let dir = if mv.abs() > 20.0 { mv.signum() } else { 0.0 };
                let f = if dt != 0.0 { (dt * 2.5).min(1.0) } else { 1.0 };
                self.cam_la += (dir * p0.lookahead - self.cam_la) * f;
                txx += self.cam_la;
            }
            tx = Some(txx);
        }
        if let Some(tx) = tx {
            let dx = tx - (self.cam.0 + vw / 2.0);
            let dy = ty - (self.cam.1 + vh / 2.0);
            let dzw = p0.dead_w / 2.0;
            let dzh = p0.dead_h / 2.0;
            let mx = if dx.abs() > dzw { dx - if dx > 0.0 { dzw } else { -dzw } } else { 0.0 };
            let my = if dy.abs() > dzh { dy - if dy > 0.0 { dzh } else { -dzh } } else { 0.0 };
            let sm = if self.cam_look.is_some() { p0.smooth.max(60.0) } else { p0.smooth };
            let f = if dt == 0.0 || sm <= 0.0 { 1.0 } else { 1.0 - (-dt * (30.0 * (1.0 - sm / 100.0).powi(2) + 1.0)).exp() };
            self.cam.0 += mx * f;
            self.cam.1 += my * f;
        }
        let (bx, by, bw, bh) = match self.cam_b {
            Some(b) => (b[0], b[1], b[2], b[3]),
            None => (0.0, 0.0, self.lw(), self.lh()),
        };
        self.cam.0 = if bw <= vw { bx + (bw - vw) / 2.0 } else { self.cam.0.min(bx + bw - vw).max(bx) };
        self.cam.1 = if bh <= vh { by + (bh - vh) / 2.0 } else { self.cam.1.min(by + bh - vh).max(by) };
    }

    // ---------------------------------------------------------------- entrées

    pub fn ctl_init(&mut self) {
        self.ctl_defs = self.settings.arr("controls").iter().filter(|c| !c.arr("keys").is_empty()).map(|c| CtlDef { name: c.s("name").to_string(), keys: c.arr("keys").iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect() }).collect();
        let saved = self.sget(&format!("pfkeys:{}", self.settings.s("title")));
        self.cbind = self.ctl_defs.iter().map(|c| {
            saved.as_ref().and_then(|s| s.get(&c.name)).and_then(|v| v.as_array()).filter(|a| !a.is_empty()).map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_else(|| c.keys.clone())
        }).collect();
        self.ctl_map();
    }
    pub fn ctl_save(&mut self) {
        let mut o = Map::new();
        for (k, c) in self.ctl_defs.iter().enumerate() {
            o.insert(c.name.clone(), Json::Array(self.cbind[k].iter().map(|s| Json::String(s.clone())).collect()));
        }
        let key = format!("pfkeys:{}", self.settings.s("title"));
        self.sset(&key, Json::Object(o));
    }
    pub fn ctl_map(&mut self) {
        self.p2v.clear();
        self.v2p.clear();
        self.blocked.clear();
        for (k, c) in self.ctl_defs.iter().enumerate() {
            for v in &c.keys {
                self.blocked.insert(v.clone());
            }
            for ph in &self.cbind[k] {
                let e = self.p2v.entry(ph.clone()).or_default();
                for v in &c.keys {
                    if !e.contains(v) {
                        e.push(v.clone());
                    }
                }
                for v in &c.keys {
                    let e2 = self.v2p.entry(v.clone()).or_default();
                    if !e2.contains(ph) {
                        e2.push(ph.clone());
                    }
                }
            }
        }
    }
    fn ctl_press(&mut self, code: &str, down: bool) {
        let vs = match self.p2v.get(code) {
            Some(v) => v.clone(),
            None => {
                if self.blocked.contains(code) {
                    return;
                }
                vec![code.to_string()]
            }
        };
        self.phys_down.insert(code.to_string(), down);
        for v in vs {
            let any = match self.v2p.get(&v) {
                Some(phs) => phs.iter().any(|ph| *self.phys_down.get(ph).unwrap_or(&false)),
                None => down,
            };
            if any && !self.keys.contains(&v) {
                self.keys.insert(v.clone());
                self.kp.insert(v.clone());
                self.any_press = true;
            } else if !any && self.keys.contains(&v) {
                self.keys.remove(&v);
                self.kr.insert(v.clone());
            }
        }
    }
    pub fn ctl_open(&mut self) {
        if self.ctl_defs.is_empty() {
            return;
        }
        self.ctl.open = true;
        self.ctl.sel = 0;
        self.ctl.wait = false;
        self.keys.clear();
        self.phys_down.clear();
    }

    /// touche enfoncée : `code` = KeyboardEvent.code, `key` = KeyboardEvent.key
    pub fn key_down(&mut self, code: &str, key: &str, ctrl: bool) {
        if self.splash {
            self.splash = false;
            self.boot();
            return;
        }
        if self.hs_in.is_some() && !self.ctl.open {
            self.hs_key(key, ctrl);
            return;
        }
        if self.ctl.open {
            self.ctl_key(code);
            return;
        }
        self.pkp.insert(code.to_string());
        if !self.ctl_defs.is_empty() {
            self.ctl_press(code, true);
        } else {
            self.keys.insert(code.to_string());
            self.kp.insert(code.to_string());
            self.any_press = true;
        }
        if code == "KeyM" && !self.embedded {
            self.muted = !self.muted;
            let m = self.muted;
            self.host.audio(AudioCmd::Mute(m));
        }
    }
    pub fn key_up(&mut self, code: &str) {
        if self.ctl.open {
            return;
        }
        if !self.ctl_defs.is_empty() {
            self.ctl_press(code, false);
            return;
        }
        self.keys.remove(code);
        self.kr.insert(code.to_string());
    }
    /// manette : état des touches simulées (pollPad de la version HTML)
    pub fn pad_input(&mut self, st: &[(&str, bool)], nav: &[(&str, bool)]) {
        for (k, on) in st {
            let was = *self.pad_prev.get(*k).unwrap_or(&false);
            if *on && !was {
                self.keys.insert(k.to_string());
                self.kp.insert(k.to_string());
                self.any_press = true;
            } else if !*on && was {
                self.keys.remove(*k);
                self.kr.insert(k.to_string());
            }
            self.pad_prev.insert(k.to_string(), *on);
        }
        for (k, on) in nav {
            let was = *self.pad_nav.get(*k).unwrap_or(&false);
            if *on && !was {
                self.pkp.insert(k.to_string());
            }
            self.pad_nav.insert(k.to_string(), *on);
        }
    }
    pub fn blur(&mut self) {
        self.keys.clear();
        self.phys_down.clear();
    }
    pub fn mouse_move(&mut self, sx: f64, sy: f64) {
        self.mouse.sx = sx;
        self.mouse.sy = sy;
    }
    pub fn mouse_down(&mut self, b: &str, sx: f64, sy: f64) {
        self.mouse_move(sx, sy);
        if self.splash {
            self.splash = false;
            self.boot();
            return;
        }
        if self.ctl.open {
            self.ctl_click();
            return;
        }
        if self.hs_in.is_some() {
            self.hs_click();
            return;
        }
        self.mouse.down.insert(b.to_string());
        self.mouse.pr.insert(b.to_string());
        self.mouse.psx = Some(sx);
        self.mouse.psy = Some(sy);
    }
    pub fn mouse_up(&mut self, b: &str) {
        if self.mouse.down.remove(b) {
            self.mouse.rl.insert(b.to_string());
        }
    }
    fn ctl_key(&mut self, c: &str) {
        let n = self.ctl_defs.len();
        if self.ctl.wait {
            if c == "Escape" {
                self.ctl.wait = false;
                return;
            }
            let sel = self.ctl.sel;
            for k in 0..n {
                if k != sel {
                    self.cbind[k].retain(|ph| ph != c);
                }
            }
            self.cbind[sel] = vec![c.to_string()];
            for k in 0..n {
                if self.cbind[k].is_empty() {
                    self.cbind[k] = self.ctl_defs[k].keys.iter().take(1).filter(|d| *d != c).cloned().collect();
                }
            }
            self.ctl.wait = false;
            self.ctl_map();
            self.ctl_save();
            self.sfx("blip");
            return;
        }
        match c {
            "ArrowUp" | "KeyW" => self.ctl.sel = (self.ctl.sel + n) % (n + 1),
            "ArrowDown" | "KeyS" => self.ctl.sel = (self.ctl.sel + 1) % (n + 1),
            "Enter" | "Space" | "NumpadEnter" => {
                if self.ctl.sel == n {
                    self.ctl.open = false;
                } else {
                    self.ctl.wait = true;
                }
            }
            "Backspace" | "Delete" => {
                if self.ctl.sel < n {
                    self.cbind[self.ctl.sel] = self.ctl_defs[self.ctl.sel].keys.clone();
                    self.ctl_map();
                    self.ctl_save();
                }
            }
            "Escape" | "KeyC" => self.ctl.open = false,
            _ => {}
        }
    }
    fn ctl_click(&mut self) {
        let hit = match &self.ctl.hit {
            Some(h) => h.clone(),
            None => return,
        };
        let (mx, my) = (self.mouse.sx, self.mouse.sy);
        for (q, h) in hit.iter().enumerate() {
            if mx >= h[0] && mx <= h[2] && my >= h[1] && my <= h[3] {
                self.ctl.sel = q;
                if q == self.ctl_defs.len() {
                    self.ctl.open = false;
                } else {
                    self.ctl.wait = true;
                }
                return;
            }
        }
    }
    pub fn key_label(code: &str) -> String {
        for (k, l) in pf_core::catalog::keys() {
            if k == code {
                return l.to_string();
            }
        }
        if let Some(r) = code.strip_prefix("Key") {
            return r.to_string();
        }
        if let Some(r) = code.strip_prefix("Digit") {
            return r.to_string();
        }
        code.to_string()
    }
    pub fn nav_keys(&self) -> HashSet<String> {
        const NAVK: [&str; 13] = ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "KeyW", "KeyA", "KeyS", "KeyD", "Space", "Enter", "NumpadEnter", "KeyE", "Escape"];
        let mut r = HashSet::new();
        for ph in &self.pkp {
            if NAVK.contains(&ph.as_str()) {
                r.insert(ph.clone());
                continue;
            }
            let vs = self.p2v.get(ph).cloned().unwrap_or_else(|| vec![ph.clone()]);
            if vs.iter().any(|v| v == "Space" || v == "Enter") {
                r.insert("Enter".into());
                continue;
            }
            for v in vs {
                r.insert(v);
            }
        }
        r
    }

    // ---------------------------------------------------------------- boucle principale

    /// avance le jeu de `dt` secondes réelles (équivalent de frame())
    pub fn frame(&mut self, dt_real: f64) {
        self.now_ms += dt_real * 1000.0;
        if self.ready && (self.paused || self.step_req) {
            self.acc = 0.0;
            self.ialpha = 1.0;
            if self.step_req {
                self.step_req = false;
                self.step(DT);
            }
        } else if self.ready {
            let dt = dt_real.max(0.0).min(0.1);
            if let Some(tr) = self.tr.as_mut() {
                tr.t += dt;
                if tr.phase_out {
                    if tr.t >= tr.d {
                        let tg = tr.target.clone();
                        self.load_layout(&tg);
                        if let Some(tr) = self.tr.as_mut() {
                            tr.phase_out = false;
                            tr.t = 0.0;
                        }
                        self.acc = 0.0;
                    }
                } else if tr.t >= tr.d {
                    self.tr = None;
                }
            }
            if self.tr.as_ref().map(|t| !t.phase_out).unwrap_or(true) {
                let ts = if self.dlg.cur.is_some() || self.ctl.open { 1.0 } else { self.tscale };
                self.acc += dt * self.dbg_speed * ts;
                let mut k = 0;
                while self.acc >= DT && k < 6 && !self.tr.as_ref().map(|t| t.phase_out).unwrap_or(false) {
                    self.step(DT);
                    self.acc -= DT;
                    k += 1;
                }
                if k >= 6 {
                    self.acc = 0.0;
                }
            }
            self.ialpha = if self.paused || self.dlg.cur.is_some() || self.ctl.open || self.tr.as_ref().map(|t| t.phase_out).unwrap_or(false) { 1.0 } else { (self.acc / DT).clamp(0.0, 1.0) };
        } else {
            self.ialpha = 1.0;
        }
        self.eng_idle();
    }

    fn eng_idle(&mut self) {}

    pub fn restart(&mut self) {
        if !self.ready {
            return;
        }
        self.host.audio(AudioCmd::StopAll);
        self.music = None;
        self.reset_g();
        self.data_init();
        self.cp = None;
        self.tr = None;
        let id = if self.layout_id.is_empty() { self.start_id.clone() } else { self.layout_id.clone() };
        self.load_layout(&id);
    }
    pub fn layout_name(&self) -> String {
        self.layout.s("name").to_string()
    }
    pub fn stop(&mut self) {
        self.host.audio(AudioCmd::EngineStopAll);
        self.host.audio(AudioCmd::StopAll);
    }

    /// nombre d'instances vivantes d'un objet (par nom)
    pub fn count(&self, name: &str) -> usize {
        self.by_name.get(name).map(|&t| self.alive_insts(t).len()).unwrap_or(0)
    }
    pub fn find(&self, name: &str) -> Vec<&Inst> {
        self.by_name.get(name).map(|&t| self.alive_insts(t).into_iter().map(|s| self.inst(s)).collect()).unwrap_or_default()
    }
    pub fn color(s: &str) -> Rgba {
        parse_color(s)
    }
}

impl Env for Runtime {
    fn global(&mut self, n: &str) -> Val {
        if let Some(s) = self.find_var_scope(n) {
            return s.borrow().vars.get(n).cloned().unwrap_or_default();
        }
        if let Some(v) = Runtime::global(self, n) {
            return v.clone();
        }
        let nn = |v: f64| Val::N(v);
        match n {
            "newline" => Val::S("\n".into()),
            "dt" => nn(DT),
            "time" => nn(self.time),
            "TickCount" => nn(self.tick as f64),
            "LayoutWidth" => nn(self.lw()),
            "LayoutHeight" => nn(self.lh()),
            "LayoutName" => Val::S(self.layout_name()),
            "ViewportWidth" => nn(self.w),
            "ViewportHeight" => nn(self.h),
            "ScrollX" => nn(self.cam.0),
            "ScrollY" => nn(self.cam.1),
            "pi" => nn(std::f64::consts::PI),
            "PlayerName" => Val::S(self.hs_name.clone()),
            "LastRank" => nn(self.hs_rank),
            "CurValue" => {
                for f in self.loop_stack.iter().rev() {
                    if let Some(v) = &f.val {
                        return v.clone();
                    }
                }
                nn(0.0)
            }
            "CurKey" => {
                for f in self.loop_stack.iter().rev() {
                    if let Some(v) = &f.key {
                        return v.clone();
                    }
                }
                Val::S(String::new())
            }
            "CameraZoom" => nn(pf_core::value::js_round(self.zoom * 100.0)),
            "CheckpointX" => nn(self.cp.as_ref().filter(|c| c.layout == self.layout_id).map(|c| c.x).unwrap_or(0.0)),
            "CheckpointY" => nn(self.cp.as_ref().filter(|c| c.layout == self.layout_id).map(|c| c.y).unwrap_or(0.0)),
            "DialogueChoice" => nn(self.dlg.choice),
            "DialogueChoiceText" => Val::S(self.dlg.choice_text.clone()),
            "TimeScale" => nn(pf_core::value::js_round(self.tscale * 100.0)),
            "loopindex" => nn(self.loop_stack.last().map(|f| f.i).unwrap_or(0.0)),
            _ => nn(0.0),
        }
    }
    fn obj(&mut self, n: &str, p: &str) -> Val {
        if let Some(&t) = self.by_name.get(n) {
            if let Some(d) = self.data.get(&t) {
                return match (p, d) {
                    ("Length", DataStore::List(l)) => Val::N(l.len() as f64),
                    ("Length", DataStore::Map(m)) => Val::N(m.len() as f64),
                    ("KeyCount", DataStore::List(l)) => Val::N(l.len() as f64),
                    ("KeyCount", DataStore::Map(m)) => Val::N(m.len() as f64),
                    ("AsJSON", d) => Val::S(d.to_json().to_string()),
                    _ => Val::N(0.0),
                };
            }
        }
        if n == "Mouse" {
            let ml = self.main_layer();
            return Val::N(match p {
                "ScreenX" => self.mouse.sx,
                "ScreenY" => self.mouse.sy,
                "X" => self.mouse.sx / self.zoom + self.cam.0 * ml.px,
                "Y" => self.mouse.sy / self.zoom + self.cam.1 * ml.py,
                _ => 0.0,
            });
        }
        if n != "Self" {
            let t = match self.by_name.get(n) {
                Some(&t) => t,
                None => return Val::N(0.0),
            };
            if p == "Count" {
                return Val::N(self.alive_insts(t).len() as f64);
            }
        }
        match self.expr_target(n) {
            Some(s) => self.prop(s, p),
            None => Val::N(0.0),
        }
    }
    fn meth(&mut self, n: &str, p: &str, a: &[Val]) -> Val {
        if let Some(&t) = self.by_name.get(n) {
            if let Some(d) = self.data.get(&t) {
                let a0 = a.first().cloned().unwrap_or(Val::N(f64::NAN));
                return match (p, d) {
                    ("At", DataStore::List(l)) => {
                        let k = a0.num0().floor();
                        if k >= 0.0 && (k as usize) < l.len() {
                            l[k as usize].clone()
                        } else {
                            Val::N(0.0)
                        }
                    }
                    ("At", _) => Val::N(0.0),
                    ("IndexOf", DataStore::List(l)) => {
                        let iv = dval(a0);
                        Val::N(l.iter().position(|x| x.loose_eq(&iv)).map(|k| k as f64).unwrap_or(-1.0))
                    }
                    ("IndexOf", _) => Val::N(-1.0),
                    ("Get", DataStore::Map(_)) => d.map_get(&a0.str()).cloned().unwrap_or(Val::N(0.0)),
                    ("Has", DataStore::Map(_)) => Val::from(d.map_get(&a0.str()).is_some()),
                    _ => Val::N(0.0),
                };
            }
        }
        if n != "Self" && !self.by_name.contains_key(n) {
            return Val::N(0.0);
        }
        let s = match self.expr_target(n) {
            Some(s) => s,
            None => return Val::N(0.0),
        };
        let an = |k: usize| a.get(k).map(|v| v.num0()).unwrap_or(0.0);
        match p {
            "TileAt" => Val::N(self.tm_get(s, an(0), an(1)) as f64),
            "TileAtXY" => {
                if self.inst(s).tm.is_some() {
                    let (c, r) = (self.tm_col(s, an(0)), self.tm_row(s, an(1)));
                    Val::N(self.tm_get(s, c, r) as f64)
                } else {
                    Val::N(-1.0)
                }
            }
            "ColumnAt" => Val::N(if self.inst(s).tm.is_some() { self.tm_col(s, an(0)) } else { 0.0 }),
            "RowAt" => Val::N(if self.inst(s).tm.is_some() { self.tm_row(s, an(0)) } else { 0.0 }),
            _ => {
                let nm = a.first().cloned();
                let q = self.img_point(s, nm.as_ref());
                match p {
                    "ImagePointX" => Val::N(q.0),
                    "ImagePointY" => Val::N(q.1),
                    _ => Val::N(0.0),
                }
            }
        }
    }
    fn func(&mut self, name: &str, a: &[Val]) -> Option<Val> {
        match name {
            "call" => {
                let nm = a.first().map(|v| v.str()).unwrap_or_else(|| "undefined".into());
                Some(self.run_func(&nm, a.get(1..).unwrap_or(&[]).to_vec()))
            }
            "loopindex" => {
                let n = a.first();
                for f in self.loop_stack.iter().rev() {
                    if n.is_none() || f.name == n.unwrap().str() {
                        return Some(Val::N(f.i));
                    }
                }
                Some(Val::N(0.0))
            }
            "storage" => {
                let k = a.first().map(|v| v.str()).unwrap_or_else(|| "undefined".into());
                Some(match self.sget(&k) {
                    Some(v) => Val::from_json(&v),
                    None => a.get(1).cloned().unwrap_or(Val::N(0.0)),
                })
            }
            "highscore" | "highscorename" | "highscorecount" => {
                let t = a.first().map(|v| v.str()).unwrap_or_default();
                let l = self.hs_list(&t);
                if name == "highscorecount" {
                    return Some(Val::N(l.len() as f64));
                }
                let r = a.get(1).map(|v| v.num0()).unwrap_or(0.0).floor() - 1.0;
                let e = if r >= 0.0 { l.get(r as usize) } else { None };
                Some(if name == "highscore" { Val::N(e.map(|e| e.s).unwrap_or(0.0)) } else { Val::S(e.map(|e| e.n.clone()).unwrap_or_default()) })
            }
            _ => None,
        }
    }
    fn random(&mut self) -> f64 {
        self.rand()
    }
}
