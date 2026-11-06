//! Données du moteur : types d'objets, instances et états des comportements.

use crate::gfx::Rgba;
use pf_core::{Json, JsonExt, Map, Val};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub type Slot = u32;

/// référence à un solide : une instance, ou une tuile d'une carte de tuiles (owner, index)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ORef {
    I(Slot),
    V(Slot, u32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BBox {
    pub l: f64,
    pub t: f64,
    pub r: f64,
    pub b: f64,
}
impl BBox {
    pub fn new(l: f64, t: f64, r: f64, b: f64) -> BBox {
        BBox { l, t, r, b }
    }
}

pub type Poly = Vec<[f64; 2]>;

#[derive(Clone, Debug)]
pub struct Shape {
    pub p: Poly,
    pub b: BBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Sprite,
    Tiled,
    Text,
    Tilemap,
    Particles,
    Button,
    Bar,
    Leaderboard,
    Road3d,
    Minimap,
    Family,
    Array,
    Dict,
    Other,
}

impl Kind {
    pub fn from(s: &str) -> Kind {
        match s {
            "sprite" => Kind::Sprite,
            "tiled" => Kind::Tiled,
            "text" => Kind::Text,
            "tilemap" => Kind::Tilemap,
            "particles" => Kind::Particles,
            "button" => Kind::Button,
            "bar" => Kind::Bar,
            "leaderboard" => Kind::Leaderboard,
            "road3d" => Kind::Road3d,
            "minimap" => Kind::Minimap,
            "family" => Kind::Family,
            "array" => Kind::Array,
            "dict" => Kind::Dict,
            _ => Kind::Other,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Anim {
    pub name: String,
    pub frames: Vec<usize>,
    pub speed: f64,
    pub looping: bool,
    pub poly: Vec<Option<Poly>>,
    pub pts: Vec<HashMap<String, [f64; 2]>>,
}

#[derive(Clone, Debug)]
pub struct VarDef {
    pub name: String,
    pub text: bool,
    pub value: Json,
}

/// définition d'un objet (lecture du JSON + informations précalculées)
#[derive(Clone, Debug)]
pub struct Def {
    pub j: Json,
    pub kind: Kind,
    pub w: f64,
    pub h: f64,
    pub ox: f64,
    pub oy: f64,
    pub cbox: Option<[f64; 4]>,
    pub poly_col: bool,
    pub anims: Vec<Anim>,
    pub points: Vec<String>,
    pub image: Option<usize>,
    pub scale: f64,
    pub vars: Vec<VarDef>,
}

impl Def {
    pub fn parse(j: &Json, img: &mut dyn FnMut(&str) -> usize) -> Def {
        let kind = Kind::from(j.s("kind"));
        let mut anims = Vec::new();
        for a in j.arr("anims") {
            let frames: Vec<usize> = a.arr("frames").iter().map(|f| img(f.as_str().unwrap_or(""))).collect();
            let poly = a.arr("poly").iter().map(|p| {
                let v: Poly = p.as_array().map(|q| q.iter().map(|pt| [pt[0].as_f64().unwrap_or(0.0), pt[1].as_f64().unwrap_or(0.0)]).collect()).unwrap_or_default();
                if v.len() >= 3 {
                    Some(v)
                } else {
                    None
                }
            }).collect();
            let pts = a.arr("pts").iter().map(|m| {
                let mut h = HashMap::new();
                if let Some(o) = m.as_object() {
                    for (k, v) in o {
                        h.insert(k.clone(), [v[0].as_f64().unwrap_or(0.0), v[1].as_f64().unwrap_or(0.0)]);
                    }
                }
                h
            }).collect();
            anims.push(Anim { name: a.s("name").to_string(), frames, speed: a.f("speed", 0.0), looping: a.get("loop") != Some(&Json::Bool(false)), poly, pts });
        }
        let cbox = j.get("cbox").filter(|c| c.is_object()).map(|c| [c.f("x", 0.0), c.f("y", 0.0), c.f("w", 0.0), c.f("h", 0.0)]);
        let image = match kind {
            Kind::Tiled | Kind::Tilemap if j.has("image") => Some(img(j.s("image"))),
            _ => None,
        };
        let vars = j.arr("vars").iter().map(|v| VarDef { name: v.s("name").to_string(), text: v.s("type") == "text", value: v.get("value").cloned().unwrap_or(Json::Null) }).collect();
        Def {
            kind,
            w: j.f("w", 0.0),
            h: j.f("h", 0.0),
            ox: j.f("ox", 0.0),
            oy: j.f("oy", 0.0),
            cbox,
            poly_col: j.s("collision") == "poly",
            anims,
            points: j.arr("points").iter().map(|p| p.s("name").to_string()).collect(),
            image,
            scale: j.f_or("scale", 1.0),
            vars,
            j: j.clone(),
        }
    }
    pub fn s(&self, k: &str) -> &str {
        self.j.s(k)
    }
    pub fn f(&self, k: &str, d: f64) -> f64 {
        self.j.f(k, d)
    }
}

pub struct Type {
    pub def: Rc<Def>,
    pub id: String,
    pub name: String,
    pub insts: Vec<Slot>,
    pub bh: HashMap<String, Map<String, Json>>,
    pub fam: bool,
    pub mem: HashSet<usize>,
    pub mem_list: Vec<usize>,
    pub fams: Vec<usize>,
    pub tpl_path: Option<Vec<[f64; 2]>>,
    pub solid: bool,
    pub jumpthru: bool,
    pub ladder: bool,
}

impl Type {
    pub fn has(&self, b: &str) -> bool {
        self.bh.contains_key(b)
    }
}

// ------------------------------------------------------------------ paramètres de comportement

pub fn pf(m: &Map<String, Json>, k: &str) -> f64 {
    match m.get(k) {
        Some(Json::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Json::String(s)) => Val::S(s.clone()).num0(),
        Some(Json::Bool(b)) => *b as i32 as f64,
        _ => 0.0,
    }
}
pub fn pb(m: &Map<String, Json>, k: &str) -> bool {
    match m.get(k) {
        Some(Json::Bool(b)) => *b,
        Some(Json::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Json::String(s)) => !s.is_empty(),
        _ => false,
    }
}
pub fn ps(m: &Map<String, Json>, k: &str) -> String {
    match m.get(k) {
        Some(Json::String(s)) => s.clone(),
        Some(Json::Number(n)) => Val::N(n.as_f64().unwrap_or(0.0)).str(),
        Some(Json::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sim {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub jump: bool,
    pub gas: bool,
    pub brake: bool,
}
impl Sim {
    pub fn set(&mut self, k: &str) {
        match k {
            "left" => self.left = true,
            "right" => self.right = true,
            "up" => self.up = true,
            "down" => self.down = true,
            "jump" => self.jump = true,
            "gas" => self.gas = true,
            "brake" => self.brake = true,
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlatformSt {
    pub max_speed: f64,
    pub accel: f64,
    pub decel: f64,
    pub jump: f64,
    pub gravity: f64,
    pub max_fall: f64,
    pub double_jump: bool,
    pub wall_jump: bool,
    pub wall_slide: f64,
    pub wall_kick: f64,
    pub climb_speed: f64,
    pub controls: bool,
    pub vx: f64,
    pub vy: f64,
    pub floor: Option<ORef>,
    pub coyote: f64,
    pub buf: f64,
    pub jumps: i32,
    pub landed: bool,
    pub jumped: bool,
    pub sim: Sim,
    pub en: bool,
    pub hold: bool,
    pub lock: f64,
    pub wall_dir: f64,
    pub wall_t: f64,
    pub wj: i64,
    pub climb: Option<ORef>,
    pub pushing: Option<ORef>,
}

#[derive(Clone, Debug, Default)]
pub struct PatrolSt {
    pub speed: f64,
    pub ledges: bool,
    pub flip: bool,
    pub gravity: f64,
    pub chase: String,
    pub range: f64,
    pub chase_speed: f64,
    pub jump: f64,
    pub dir: f64,
    pub vy: f64,
    pub floor: Option<ORef>,
    pub chasing: bool,
    pub spot: i64,
}

#[derive(Clone, Debug, Default)]
pub struct SineSt {
    pub vertical: bool,
    pub period: f64,
    pub magnitude: f64,
    pub bx: f64,
    pub by: f64,
    pub t: f64,
    pub en: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PathSt {
    pub speed: f64,
    pub mode: String,
    pub pause: f64,
    pub ease: bool,
    pub rotate: bool,
    pub bx: f64,
    pub by: f64,
    pub pts: Vec<[f64; 2]>,
    pub k: usize,
    pub nx: usize,
    pub dir: i32,
    pub u: f64,
    pub wt: f64,
    pub en: bool,
    pub arr: i64,
    pub fin: i64,
    pub last: usize,
}

#[derive(Clone, Debug, Default)]
pub struct TimerOne {
    pub left: f64,
    pub dur: f64,
    pub reg: bool,
}
#[derive(Clone, Debug, Default)]
pub struct TimerSt {
    pub timers: Vec<(String, TimerOne)>,
    pub fired: HashMap<String, i64>,
}

#[derive(Clone, Debug, Default)]
pub struct BulletSt {
    pub speed: f64,
    pub angle: f64,
    pub gravity: f64,
    pub vx: f64,
    pub vy: f64,
}

#[derive(Clone, Debug, Default)]
pub struct EightSt {
    pub max_speed: f64,
    pub accel: f64,
    pub decel: f64,
    pub dirs: String,
    pub rotate: bool,
    pub controls: bool,
    pub vx: f64,
    pub vy: f64,
    pub sim: Sim,
    pub en: bool,
}

#[derive(Clone, Debug, Default)]
pub struct CarSt {
    pub max_speed: f64,
    pub accel: f64,
    pub brake: f64,
    pub friction: f64,
    pub reverse: f64,
    pub steer: f64,
    pub drift: f64,
    pub engine: bool,
    pub sp: f64,
    pub vx: f64,
    pub vy: f64,
    pub sim: Sim,
    pub en: bool,
    pub ctl: bool,
    pub hit: i64,
    pub hit_v: f64,
}

#[derive(Clone, Debug)]
pub struct Tween {
    pub prop: String,
    pub from: f64,
    pub to: f64,
    pub d: f64,
    pub t: f64,
    pub ease: String,
    pub mode: String,
    pub tag: String,
}
#[derive(Clone, Debug, Default)]
pub struct TweenSt {
    pub list: Vec<Tween>,
    pub done: HashMap<String, i64>,
}

#[derive(Clone, Debug, Default)]
pub struct PhysSt {
    pub circle: bool,
    pub density: f64,
    pub friction: f64,
    pub bounce: f64,
    pub gravity: f64,
    pub pushable: bool,
    pub vx: f64,
    pub vy: f64,
    pub fixed: bool,
    pub ground: bool,
    pub hit_t: i64,
    pub hit_v: f64,
    pub push_t: i64,
}

#[derive(Clone, Debug, Default)]
pub struct FadeSt {
    pub fade_in: f64,
    pub wait: f64,
    pub fade_out: f64,
    pub destroy: bool,
    pub t: f64,
    pub run: bool,
    pub base: f64,
    pub done: i64,
}

#[derive(Clone, Debug, Default)]
pub struct PinSt {
    pub mirror: bool,
    pub angle: bool,
    pub destroy: bool,
    pub to: Option<Slot>,
    pub pt: String,
    pub ox: f64,
    pub oy: f64,
    pub oa: f64,
    pub mi: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DragSt {
    pub axes: String,
    pub on: bool,
    pub en: bool,
    pub st: i64,
    pub dr: i64,
    pub gx: f64,
    pub gy: f64,
}

#[derive(Clone, Debug, Default)]
pub struct TurretSt {
    pub range: f64,
    pub rate: f64,
    pub rot_speed: f64,
    pub rotate: bool,
    pub types: Vec<usize>,
    pub target: Option<Slot>,
    pub cd: f64,
    pub fired: i64,
}

#[derive(Clone, Debug, Default)]
pub struct LosSt {
    pub range: f64,
    pub cone: f64,
}

#[derive(Clone, Debug, Default)]
pub struct LightSt {
    pub radius: f64,
    pub intensity: f64,
    pub color: String,
    pub glow: f64,
    pub flicker: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ScrollSt {
    pub smooth: f64,
    pub dead_w: f64,
    pub dead_h: f64,
    pub lookahead: f64,
    pub offset_y: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Behs {
    pub platform: Option<PlatformSt>,
    pub patrol: Option<PatrolSt>,
    pub sine: Option<SineSt>,
    pub path: Option<PathSt>,
    pub timer: Option<TimerSt>,
    pub bullet: Option<BulletSt>,
    pub eightdir: Option<EightSt>,
    pub car: Option<CarSt>,
    pub tween: Option<TweenSt>,
    pub physics: Option<PhysSt>,
    pub fade: Option<FadeSt>,
    pub pin: Option<PinSt>,
    pub drag: Option<DragSt>,
    pub turret: Option<TurretSt>,
    pub los: Option<LosSt>,
    pub light: Option<LightSt>,
    pub scrollto: Option<ScrollSt>,
    pub rotate: Option<f64>,
    pub bound: Option<bool>,
    pub destroy: bool,
}

impl Behs {
    pub fn has(&self, k: &str) -> bool {
        match k {
            "platform" => self.platform.is_some(),
            "patrol" => self.patrol.is_some(),
            "sine" => self.sine.is_some(),
            "path" => self.path.is_some(),
            "timer" => self.timer.is_some(),
            "bullet" => self.bullet.is_some(),
            "eightdir" => self.eightdir.is_some(),
            "car" => self.car.is_some(),
            "tween" => self.tween.is_some(),
            "physics" => self.physics.is_some(),
            "fade" => self.fade.is_some(),
            "pin" => self.pin.is_some(),
            "drag" => self.drag.is_some(),
            "turret" => self.turret.is_some(),
            "los" => self.los.is_some(),
            "light" => self.light.is_some(),
            "scrollto" => self.scrollto.is_some(),
            "rotate" => self.rotate.is_some(),
            "bound" => self.bound.is_some(),
            "destroy" => self.destroy,
            // comportements sans état (solid, jumpthru, ladder) : présents sur le type
            _ => false,
        }
    }
}

// ------------------------------------------------------------------ objets spéciaux

#[derive(Clone, Debug, Default)]
pub struct VirtSolid {
    pub vb: BBox,
    pub sh: Option<Shape>,
}

#[derive(Clone, Debug, Default)]
pub struct TileOut {
    pub rect: Vec<u32>,
    pub jt: Vec<u32>,
    pub poly: Vec<u32>,
    pub all: Vec<VirtSolid>,
}

#[derive(Clone, Debug, Default)]
pub struct TileData {
    pub cols: i64,
    pub rows: i64,
    pub data: Vec<i64>,
    pub ver: u64,
    pub vk: Option<(f64, f64, u64, i64, i64, f64, f64)>,
    pub vt: TileOut,
}

#[derive(Clone, Debug, Default)]
pub struct BarSt {
    pub v: f64,
    pub max: f64,
    pub shown: f64,
    pub ghost: f64,
    pub gt: f64,
    pub col: String,
}

#[derive(Clone, Debug, Default)]
pub struct BtnSt {
    pub hover: bool,
    pub press: bool,
    pub focus: bool,
    pub en: bool,
    pub clicked: i64,
}

#[derive(Clone, Debug)]
pub struct Part {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub age: f64,
    pub life: f64,
    pub s0: f64,
    pub s1: f64,
    pub c: Rgba,
    pub rot: f64,
    pub spin: f64,
}

#[derive(Clone, Debug, Default)]
pub struct PartSt {
    pub parts: Vec<Part>,
    pub acc: f64,
    pub active: bool,
    pub started: bool,
    pub rate: Option<f64>,
    pub img: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct HsEntry {
    pub n: String,
    pub s: f64,
    pub d: f64,
}

pub struct Inst {
    pub uid: u32,
    pub t: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub angle: f64,
    pub mirrored: bool,
    pub opacity: f64,
    pub visible: bool,
    pub hud: bool,
    pub text: Val,
    pub layer: usize,
    pub vars: HashMap<String, Val>,
    pub anim: usize,
    pub frame: usize,
    pub ft: f64,
    pub anim_end: i64,
    pub dead: bool,
    pub kill: bool,
    pub flash: f64,
    pub b: Behs,
    pub tm: Option<TileData>,
    pub bar: Option<BarSt>,
    pub btn: Option<BtnSt>,
    pub ps: Option<PartSt>,
    pub style: Option<Map<String, Json>>,
    pub tint: Option<(Rgba, f64)>,
    pub tf: Option<(Rgba, f64, f64)>,
    pub px: f64,
    pub py: f64,
    pub dx: f64,
    pub dy: f64,
    pub sf: Option<ORef>,
    pub nojt: bool,
    pub plv: i32,
    pub pmv: f64,
    pub lbt: String,
    pub lb: Vec<HsEntry>,
    pub lb_hi: Option<f64>,
    pub mm_r: Option<f64>,
    pub rd: Option<Box<crate::road::RoadSt>>,
}
