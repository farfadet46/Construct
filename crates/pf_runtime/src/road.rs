//! Route pseudo-3D (vue arrière façon rallye) : construction de la piste, simulation, dessin, mini-carte.

use crate::gfx::{parse_color, poly_path, Gfx, Rgba};
use crate::model::*;
use crate::rt::{Runtime, AudioCmd};
use pf_core::{Json, JsonExt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const RD_TRACK: &str = "40 0 0\n60 3 20\n40 0 -20\n80 -4 0\n50 0 40\n60 5 -40\n40 -2 0\n70 0 0";

#[derive(Clone, Debug, Default)]
pub struct RdCar {
    pub id: String,
    pub z: f64,
    pub dist: f64,
    pub off: f64,
    pub spd: f64,
    pub rival: bool,
    pub sway: f64,
}

#[derive(Clone, Debug, Default)]
pub struct RoadSt {
    pub pos: f64,
    pub dist: f64,
    pub speed: f64,
    pub x: f64,
    pub lap: i64,
    pub lap_time: f64,
    pub last: f64,
    pub best: f64,
    pub lap_t: i64,
    pub hit_t: i64,
    pub ctl: bool,
    pub sim: Sim,
    pub sky: f64,
    pub steer: f64,
    pub max_kmh: f64,
    pub cars: Vec<RdCar>,
    pub hz: Option<f64>,
    pub hit_cd: f64,
    pub go: bool,
}

#[derive(Clone, Debug)]
pub struct RdSpr {
    pub id: String,
    pub off: f64,
    pub dir: f64,
    pub lvl: i32,
}

#[derive(Clone, Debug, Default)]
pub struct RdPt {
    pub y: f64,
    pub z: f64,
    pub wx: f64,
    pub cz: f64,
    pub s: f64,
    pub sx: f64,
    pub sy: f64,
    pub sw: f64,
}

#[derive(Clone, Debug)]
pub struct RdSeg {
    pub i: usize,
    pub p1: RdPt,
    pub p2: RdPt,
    pub curve: f64,
    pub col: bool,
    pub spr: Vec<RdSpr>,
}

pub struct RdBuilt {
    pub segs: Vec<RdSeg>,
    pub l: f64,
    pub len: f64,
    pub map: Vec<[f64; 2]>,
    pub mb: [f64; 4],
}

pub fn rd_num(v: Option<&Json>, d: f64) -> f64 {
    match v {
        None | Some(Json::Null) => {
            if d.is_finite() {
                // +undefined = NaN → d ; +null = 0
                if v.is_none() {
                    d
                } else {
                    0.0
                }
            } else {
                d
            }
        }
        Some(x) => {
            let n = pf_core::Val::from_json(x).num();
            if n.is_finite() {
                n
            } else {
                d
            }
        }
    }
}
fn rn(o: &Json, k: &str, d: f64) -> f64 {
    rd_num(o.get(k), d)
}
fn ease(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * (-(t * std::f64::consts::PI).cos() / 2.0 + 0.5)
}
fn ease_in(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t * t
}
fn ease_out(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * (1.0 - (1.0 - t) * (1.0 - t))
}

thread_local! {
    static RD_CACHE: RefCell<HashMap<String, Rc<RdBuilt>>> = RefCell::new(HashMap::new());
}

fn jkey(o: &Json, k: &str) -> String {
    match o.get(k) {
        None | Some(Json::Null) => String::new(),
        Some(v) => crate::rt::jstr(v),
    }
}

pub fn rd_build(o: &Json) -> Rc<RdBuilt> {
    let key = ["track", "segLen", "rumble", "decor", "decorEvery", "decor2", "decor2Every", "seed", "autoSigns"].iter().map(|k| jkey(o, k)).collect::<Vec<_>>().join("|");
    if let Some(r) = RD_CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return r;
    }
    let l = rn(o, "segLen", 200.0).max(20.0);
    let rum = (pf_core::value::js_round(rn(o, "rumble", 3.0))).max(1.0) as usize;
    let mut segs: Vec<RdSeg> = Vec::new();
    fn last_y(s: &[RdSeg]) -> f64 {
        s.last().map(|x| x.p2.y).unwrap_or(0.0)
    }
    let add = |segs: &mut Vec<RdSeg>, curve: f64, y: f64| {
        let n = segs.len();
        let ly = last_y(segs);
        segs.push(RdSeg { i: n, p1: RdPt { y: ly, z: n as f64 * l, ..Default::default() }, p2: RdPt { y, z: (n + 1) as f64 * l, ..Default::default() }, curve, col: (n / rum) % 2 == 1, spr: Vec::new() });
    };
    let mut secs: Vec<(usize, usize, f64)> = Vec::new();
    let track = match o.get("track") {
        Some(Json::String(s)) if !s.is_empty() => s.clone(),
        _ => RD_TRACK.to_string(),
    };
    for line in track.split(|c| c == '\n' || c == ';') {
        let q: Vec<f64> = line.trim().split(|c: char| c.is_whitespace() || c == ',').filter(|x| !x.is_empty()).map(|x| pf_core::value::str_to_num(x)).collect();
        if q.is_empty() || !q[0].is_finite() || q[0] <= 0.0 {
            continue;
        }
        let n = (pf_core::value::js_round(q[0]) as usize).min(2000);
        let num = |k: usize| q.get(k).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
        let c = num(1);
        let h = num(2);
        let en = n / 4;
        let lv = en;
        let hold = n - en - lv;
        let y0 = last_y(&segs);
        let y1 = y0 + h * l;
        let tot = n as f64;
        secs.push((segs.len(), n, c));
        for k in 0..en {
            add(&mut segs, ease_in(0.0, c, k as f64 / en as f64), ease(y0, y1, k as f64 / tot));
        }
        for k in 0..hold {
            add(&mut segs, c, ease(y0, y1, (en + k) as f64 / tot));
        }
        for k in 0..lv {
            add(&mut segs, ease_out(c, 0.0, k as f64 / lv as f64), ease(y0, y1, (en + hold + k) as f64 / tot));
        }
    }
    while segs.len() < 40 {
        let y = last_y(&segs);
        add(&mut segs, 0.0, y);
    }
    let y_end = last_y(&segs);
    if y_end.abs() > 1.0 {
        let m = 60.min(segs.len() / 3);
        let n = segs.len();
        let y0b = segs[n - m].p1.y;
        for k2 in 0..m {
            let sg = &mut segs[n - m + k2];
            sg.p1.y = ease(y0b, 0.0, k2 as f64 / m as f64);
            sg.p2.y = ease(y0b, 0.0, (k2 + 1) as f64 / m as f64);
        }
    }
    let mut seed = (pf_core::value::js_round(rn(o, "seed", 7.0))).max(1.0) as i64;
    let mut r = move || {
        seed = (seed * 16807) % 2147483647;
        (seed - 1) as f64 / 2147483646.0
    };
    let nseg = segs.len();
    if o.get("autoSigns") != Some(&Json::Bool(false)) {
        for (k, sc) in secs.iter().enumerate() {
            let ac = sc.2.abs();
            if ac < 1.0 {
                continue;
            }
            if k > 0 {
                let prev = secs[k - 1];
                if prev.2.signum() == sc.2.signum() && prev.2.abs() >= ac {
                    continue;
                }
            }
            let dir: f64 = if sc.2 > 0.0 { 1.0 } else { -1.0 };
            let lvl = if ac < 3.0 { 1 } else if ac < 5.0 { 2 } else { 3 };
            let side = -dir * 1.32;
            let mut addsign = |at: i64| {
                if at < 4 || at >= nseg as i64 {
                    return;
                }
                let sg = &mut segs[at as usize];
                sg.spr.retain(|q| q.off.signum() != side.signum());
                sg.spr.push(RdSpr { id: "#sign".into(), off: side, dir, lvl });
            };
            addsign(sc.0 as i64 - 26);
            addsign(sc.0 as i64 - 14);
            let mut j = 4.0;
            while j < (sc.1 as f64 * 0.6).min(48.0) {
                addsign(sc.0 as i64 + j as i64);
                j += 11.0;
            }
        }
    }
    for (dk, de) in [("decor", "decorEvery"), ("decor2", "decor2Every")] {
        let id = o.s(dk).to_string();
        let every = rn(o, de, if dk == "decor" { 6.0 } else { 17.0 });
        if id.is_empty() || every <= 0.0 {
            continue;
        }
        let stepn = (pf_core::value::js_round(every) as usize).max(1);
        let mut n = 8;
        while n < nseg {
            let side = if r() < 0.5 { -1.0 } else { 1.0 };
            let off = side * (1.25 + r() * 1.6);
            segs[n].spr.push(RdSpr { id: id.clone(), off, dir: 0.0, lvl: 0 });
            if r() < 0.5 {
                let off2 = -side * (1.3 + r() * 2.2);
                segs[n].spr.push(RdSpr { id: id.clone(), off: off2, dir: 0.0, lvl: 0 });
            }
            n += stepn;
        }
    }
    let tot: f64 = segs.iter().map(|s| s.curve * 0.55).sum();
    let dir0 = if tot < 0.0 { -1.0 } else { 1.0 };
    let extra = (dir0 * 360.0 - tot) / nseg as f64;
    let mut mp = Vec::with_capacity(nseg);
    let (mut hx, mut hy, mut ang) = (0.0, 0.0, -90.0f64);
    for sg in &segs {
        ang += sg.curve * 0.55 + extra;
        hx += (ang * std::f64::consts::PI / 180.0).cos();
        hy += (ang * std::f64::consts::PI / 180.0).sin();
        mp.push([hx, hy]);
    }
    let nn = mp.len() as f64;
    for (k, q) in mp.iter_mut().enumerate() {
        q[0] -= hx * (k + 1) as f64 / nn;
        q[1] -= hy * (k + 1) as f64 / nn;
    }
    let mut mb = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for q in &mp {
        mb[0] = mb[0].min(q[0]);
        mb[2] = mb[2].max(q[0]);
        mb[1] = mb[1].min(q[1]);
        mb[3] = mb[3].max(q[1]);
    }
    let len = nseg as f64 * l;
    let built = Rc::new(RdBuilt { segs, l, len, map: mp, mb });
    RD_CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 64 {
            c.clear();
        }
        c.insert(key, built.clone());
    });
    built
}

fn modp(a: f64, m: f64) -> f64 {
    ((a % m) + m) % m
}

impl Runtime {
    fn rd_def(&self, s: Slot) -> Json {
        self.def(s).j.clone()
    }
    pub fn rd_max(&self, s: Slot) -> f64 {
        rd_build(&self.rd_def(s)).l * 60.0
    }
    fn rd_max_kmh(&self, s: Slot) -> f64 {
        let m = self.inst(s).rd.as_ref().map(|r| r.max_kmh).unwrap_or(0.0);
        if m != 0.0 {
            m
        } else {
            rn(&self.rd_def(s), "maxKmh", 220.0)
        }
    }
    pub fn rd_kmh(&self, s: Slot) -> f64 {
        let sp = self.inst(s).rd.as_ref().map(|r| r.speed).unwrap_or(0.0);
        pf_core::value::js_round(sp / self.rd_max(s) * rn(&self.rd_def(s), "maxKmh", 220.0))
    }
    pub fn rd_curve(&self, s: Slot) -> f64 {
        let r = rd_build(&self.rd_def(s));
        let pos = self.inst(s).rd.as_ref().unwrap().pos;
        let k = (modp(pos, r.len) / r.l).floor() as usize % r.segs.len();
        r.segs[k].curve
    }
    pub fn rd_prog(&self, s: Slot) -> f64 {
        let r = rd_build(&self.rd_def(s));
        modp(self.inst(s).rd.as_ref().unwrap().pos, r.len) / r.len * 100.0
    }
    pub fn rd_rank(&self, s: Slot) -> f64 {
        let rd = self.inst(s).rd.as_ref().unwrap();
        1.0 + rd.cars.iter().filter(|c| c.rival && c.dist > rd.dist).count() as f64
    }
    pub fn rd_init(&mut self, s: Slot, reset: bool) {
        let o = self.rd_def(s);
        let r = rd_build(&o);
        let old = self.inst(s).rd.as_deref().cloned().unwrap_or_default();
        let mut st = RoadSt {
            last: old.last,
            best: if reset { old.best } else { 0.0 },
            lap_t: -1,
            hit_t: -1,
            ctl: if reset { old.ctl } else { o.get("controls") != Some(&Json::Bool(false)) },
            max_kmh: old.max_kmh,
            ..Default::default()
        };
        let mut n = pf_core::value::js_round(rn(&o, "rivals", 0.0)).clamp(0.0, 40.0) as usize;
        let cid = o.s("rivalObj").to_string();
        if cid.is_empty() || !self.type_by_id.contains_key(&cid) {
            n = 0;
        }
        for k in 0..n {
            let z = (k + 1) as f64 * rn(&o, "gridGap", 5.0) * r.l;
            let mn = rn(&o, "rivalMin", 60.0);
            let mx = rn(&o, "rivalMax", 85.0);
            st.cars.push(RdCar { id: cid.clone(), z, dist: z, off: if k % 2 == 1 { 0.45 } else { -0.45 }, spd: mn / 100.0 + (mx - mn) / 100.0 * ((k * 37 % 11) as f64 / 10.0), rival: true, sway: k as f64 * 1.7 });
        }
        let tn = pf_core::value::js_round(rn(&o, "traffic", 0.0)).clamp(0.0, 60.0) as usize;
        let tid = o.s("trafficObj").to_string();
        if !tid.is_empty() && self.type_by_id.contains_key(&tid) {
            for t2 in 0..tn {
                let z2 = (t2 + 1) as f64 * r.len / (tn + 1) as f64;
                st.cars.push(RdCar { id: tid.clone(), z: z2, dist: z2, off: ((t2 % 3) as f64 - 1.0) * 0.55, spd: 0.25 + (t2 * 13 % 7) as f64 / 20.0, rival: false, sway: t2 as f64 });
            }
        }
        self.inst_mut(s).rd = Some(Box::new(st));
    }
    fn lk_def_w(&self, id: &str, d: f64) -> Option<f64> {
        if id == "#sign" {
            return Some(50.0);
        }
        self.type_by_id.get(id).map(|&t| self.types[t].def.j.f_or("defW", d))
    }
    pub fn rd_step(&mut self, s: Slot, dt: f64) {
        let o = self.rd_def(s);
        let r = rd_build(&o);
        let n = r.segs.len();
        let l = r.l;
        let mx = self.rd_max(s) * self.rd_max_kmh(s) / rn(&o, "maxKmh", 220.0);
        let rw = rn(&o, "roadWidth", 2000.0);
        let cam_h = rn(&o, "camHeight", 1000.0);
        let depth = 1.0 / (rn(&o, "fov", 100.0) / 2.0 * std::f64::consts::PI / 180.0).tan();
        let pz = cam_h * depth;
        let iw = self.inst(s).w;
        let unit = 2.0 * cam_h / iw.max(1.0);
        let sc = unit * rn(&o, "spriteScale", 2.0);
        let lock = self.cine_lock();
        let tick = self.tick;
        let keys = self.keys.clone();
        let k = |a: &str| keys.contains(a);
        let mut st = self.inst_mut(s).rd.take().unwrap();
        let c = st.ctl && !lock;
        let sm = std::mem::take(&mut st.sim);
        let g = sm.gas || (c && (k("ArrowUp") || k("KeyW")));
        let b = sm.brake || (c && (k("ArrowDown") || k("KeyS")));
        let lf = sm.left || (c && (k("ArrowLeft") || k("KeyA")));
        let rt = sm.right || (c && (k("ArrowRight") || k("KeyD")));
        let pseg = &r.segs[((st.pos + pz) / l).floor() as usize % n];
        let sp = st.speed / mx.max(1.0);
        let dx = dt * 2.0 * sp * rn(&o, "steering", 1.0);
        st.steer = (rt as i32 - lf as i32) as f64;
        st.x += st.steer * dx;
        st.x -= dx * sp * pseg.curve * rn(&o, "centrifugal", 0.3);
        st.sky += pseg.curve * sp * dt * 60.0;
        let acc = mx / rn(&o, "accelTime", 5.0).max(0.2);
        if g && !b {
            st.speed += acc * dt;
        } else if b {
            st.speed -= mx * rn(&o, "brakePower", 1.0) * dt;
        } else {
            st.speed -= acc * dt;
        }
        let pw = rn(&o, "playerW", 0.3);
        if st.x.abs() > 1.0 {
            if st.speed > mx * rn(&o, "offroadMax", 30.0) / 100.0 {
                st.speed -= mx * 0.5 * dt * rn(&o, "offroadDrag", 1.0);
            }
            for sp0 in &pseg.spr {
                let dw = match self.lk_def_w(&sp0.id, 32.0) {
                    Some(w) => w,
                    None => continue,
                };
                let sw = dw * sc / rw;
                if !(st.hit_cd > 0.0) && rd_overlap(st.x, pw, sp0.off + sw / 2.0 * if sp0.off > 0.0 { 1.0 } else { -1.0 }, sw) {
                    st.speed = st.speed.min(mx * 0.22);
                    st.x -= (if st.x > 0.0 { 1.0 } else { -1.0 }) * 0.35;
                    st.hit_t = tick;
                    st.hit_cd = 0.7;
                    break;
                }
            }
        }
        if st.hit_cd > 0.0 {
            st.hit_cd -= dt;
        }
        st.speed = st.speed.clamp(0.0, mx.max(0.0));
        let mo = rn(&o, "maxOut", 2.5);
        st.x = st.x.clamp(-mo, mo);
        if st.ctl {
            st.go = true;
        }
        if st.go {
            for ci in 0..st.cars.len() {
                let car = st.cars[ci].clone();
                let cw = self.lk_def_w(&car.id, 48.0).map(|w| w * unit / rw).unwrap_or(0.3);
                let cs = &r.segs[(car.z / l).floor() as usize % n];
                let v = car.spd * mx * if car.rival { 1.0 - 0.15 * (cs.curve.abs() / 6.0).max(0.0) } else { 1.0 };
                let cr = &mut st.cars[ci];
                cr.z = (cr.z + v * dt) % r.len;
                cr.dist += v * dt;
                cr.sway += dt;
                cr.off += (cr.sway * 0.7).sin() * 0.12 * dt;
                cr.off = cr.off.clamp(-0.85, 0.85);
                let (cz, coff) = (cr.z, cr.off);
                let gap = modp(cz - (st.pos + pz) + r.len * 2.0, r.len);
                if gap < l * 1.2 && st.speed > v && !(st.hit_cd > 0.0) && rd_overlap(st.x, pw, coff, cw * 0.9) {
                    st.speed = v * 0.85;
                    st.x += (if st.x >= coff { 1.0 } else { -1.0 }) * 0.22;
                    st.hit_t = tick;
                    st.hit_cd = 0.5;
                }
            }
        }
        st.pos += st.speed * dt;
        st.dist += st.speed * dt;
        st.lap_time += dt;
        if o.get("engine") != Some(&Json::Bool(false)) {
            let rmax = self.rd_max(s);
            let uid = self.inst(s).uid;
            let screech = (if st.steer != 0.0 && st.speed > mx * 0.55 { (st.speed / mx - 0.55) * 2.2 } else { 0.0 }) + if st.hit_t == tick { 1.0 } else { 0.0 };
            let rumble = if st.x.abs() > 1.0 { st.speed / mx * 1.4 } else { 0.0 };
            let v = st.speed / rmax.max(1.0);
            let load = if g && st.ctl { 1.0 } else { 0.0 };
            self.engines.insert(uid);
            self.host.audio(AudioCmd::Engine { uid, v, load, screech, rumble });
        }
        if st.pos >= r.len {
            st.pos -= r.len;
            st.lap += 1;
            st.lap_t = tick;
            st.last = st.lap_time;
            if st.best == 0.0 || st.lap_time < st.best {
                st.best = st.lap_time;
            }
            st.lap_time = 0.0;
        }
        self.inst_mut(s).rd = Some(st);
    }
    pub fn rd_act(&mut self, s: Slot, k: &str, p: &[Json]) {
        match k {
            "sim" => {
                let key = p.first().map(crate::rt::jstr).unwrap_or_default();
                self.inst_mut(s).rd.as_mut().unwrap().sim.set(&key);
            }
            "controls" => self.inst_mut(s).rd.as_mut().unwrap().ctl = crate::rt::jtruthy(p.first()),
            "setspeed" => {
                let v = self.x(p.first().unwrap_or(&Json::Null)).num0();
                let mk = rn(&self.rd_def(s), "maxKmh", 220.0);
                let m = self.rd_max(s);
                self.inst_mut(s).rd.as_mut().unwrap().speed = (v / mk * m).max(0.0);
            }
            "setmax" => {
                let v = self.x(p.first().unwrap_or(&Json::Null)).num0();
                self.inst_mut(s).rd.as_mut().unwrap().max_kmh = v.max(1.0);
            }
            "reset" => self.rd_init(s, true),
            "settrack" => {
                let v = self.x(p.first().unwrap_or(&Json::Null)).str().replace(';', "\n");
                let t = self.inst(s).t;
                let mut d = (*self.types[t].def).clone();
                d.j["track"] = Json::String(v);
                self.types[t].def = Rc::new(d);
                self.rd_init(s, true);
            }
            _ => {}
        }
    }
}

fn rd_overlap(x1: f64, w1: f64, x2: f64, w2: f64) -> bool {
    !(x1 + w1 / 2.0 < x2 - w2 / 2.0 || x1 - w1 / 2.0 > x2 + w2 / 2.0)
}

// ---------------------------------------------------------------- dessin

thread_local! {
    static SIGNS: RefCell<HashMap<(i32, i32), Rc<tiny_skia::Pixmap>>> = RefCell::new(HashMap::new());
}

pub fn rd_sign_img(dir: f64, lvl: i32) -> Rc<tiny_skia::Pixmap> {
    let k = (dir as i32, lvl);
    if let Some(p) = SIGNS.with(|s| s.borrow().get(&k).cloned()) {
        return p;
    }
    let mut g = Gfx::new(26, 34);
    g.fill_rect(12.0, 18.0, 2.0, 16.0, parse_color("#4a5263"));
    g.fill_rect(0.0, 0.0, 26.0, 19.0, parse_color("#1b1f2a"));
    g.fill_rect(1.0, 1.0, 24.0, 17.0, parse_color("#ffd84d"));
    let w = if lvl == 1 { 7.0 } else if lvl == 2 { 13.0 } else { 19.0 };
    let x0 = 13.0 - w / 2.0;
    let dark = parse_color("#1b1f2a");
    for n in 0..lvl {
        let bx = x0 + n as f32 * 6.0;
        for j in 0..5 {
            let px = if dir > 0.0 { bx + j as f32 } else { bx + 4.0 - j as f32 };
            g.fill_rect(px.round(), 4.0 + j as f32, 2.0, 1.0, dark);
            g.fill_rect(px.round(), 13.0 - j as f32, 2.0, 1.0, dark);
        }
    }
    let p = Rc::new(g.pm);
    SIGNS.with(|s| s.borrow_mut().insert(k, p.clone()));
    p
}

fn project(p: &mut RdPt, cx: f64, cy: f64, cz: f64, depth: f64, w: f64, h: f64, rw: f64) {
    let x = -cx;
    let y = p.y - cy;
    let z = p.z - cz;
    p.cz = z;
    p.s = depth / z;
    p.sx = pf_core::value::js_round(w / 2.0 + p.s * (x + p.wx) * w / 2.0);
    p.sy = pf_core::value::js_round(h / 2.0 - p.s * y * h / 2.0);
    p.sw = pf_core::value::js_round(p.s * rw * w / 2.0);
}
fn quad(g: &mut Gfx, x1: f64, y1: f64, w1: f64, x2: f64, y2: f64, w2: f64, c: Rgba) {
    if let Some(p) = poly_path(&[((x1 - w1) as f32, y1 as f32), ((x2 - w2) as f32, y2 as f32), ((x2 + w2) as f32, y2 as f32), ((x1 + w1) as f32, y1 as f32)], true) {
        g.fill_path(&p, c);
    }
}
fn col(o: &Json, k: &str, d: &str) -> Rgba {
    let s = o.s(k);
    parse_color(if s.is_empty() { d } else { s })
}

/// lookup : id d'objet → (largeur posée defW, image)
pub type RdLookup<'a> = &'a dyn Fn(&str) -> Option<(f64, Rc<tiny_skia::Pixmap>)>;

pub fn rd_draw(g: &mut Gfx, o: &Json, x0: f64, y0: f64, w: f64, h: f64, st: Option<&RoadSt>, lk: RdLookup) -> Option<f64> {
    let r = rd_build(o);
    let n = r.segs.len();
    let def = RoadSt::default();
    let st = st.unwrap_or(&def);
    let l = r.l;
    let rw = rn(o, "roadWidth", 2000.0);
    let cam_h = rn(o, "camHeight", 1000.0);
    let depth = 1.0 / (rn(o, "fov", 100.0) / 2.0 * std::f64::consts::PI / 180.0).tan();
    let pz = cam_h * depth;
    let dd = (pf_core::value::js_round(rn(o, "drawDist", 160.0)).max(20.0) as usize).min(n - 1);
    let lanes = pf_core::value::js_round(rn(o, "lanes", 2.0)).max(1.0);
    let pos = modp(st.pos, r.len);
    let base = &r.segs[(pos / l).floor() as usize % n];
    let bp = (pos % l) / l;
    let ps = &r.segs[((pos + pz) / l).floor() as usize % n];
    let pp = ((pos + pz) % l) / l;
    let p_y = ps.p1.y + (ps.p2.y - ps.p1.y) * pp;
    g.save();
    g.translate(x0 as f32, y0 as f32);
    g.clip_rect(0.0, 0.0, w as f32, h as f32);
    let mut segs: Vec<RdSeg> = Vec::with_capacity(dd);
    let mut fogs = Vec::with_capacity(dd);
    let mut clips = Vec::with_capacity(dd);
    let mut vis: Vec<usize> = Vec::new();
    let mut maxy = h;
    let mut x = 0.0;
    let mut dx = -(base.curve * bp);
    let fog_c = col(o, "fog", "#9fd3f2");
    let fog_d = rn(o, "fogDensity", 4.0);
    let cx = st.x * rw;
    for k in 0..dd {
        let mut sg = r.segs[(base.i + k) % n].clone();
        let looped = sg.i < base.i;
        let cz = pos - if looped { r.len } else { 0.0 };
        let kk = k as f64 / dd as f64;
        fogs.push(1.0 / (kk * kk * fog_d).exp());
        clips.push(maxy);
        sg.p1.wx = x;
        sg.p2.wx = x + dx;
        project(&mut sg.p1, cx, p_y + cam_h, cz, depth, w, h, rw);
        project(&mut sg.p2, cx, p_y + cam_h, cz, depth, w, h, rw);
        x += dx;
        dx += sg.curve;
        let ok = !(sg.p1.cz <= depth || sg.p2.sy >= sg.p1.sy || sg.p2.sy >= maxy);
        if ok {
            vis.push(k);
            maxy = sg.p1.sy;
        }
        segs.push(sg);
    }
    let hz = maxy;
    let (ga, gb) = (col(o, "grassA", "#58b84f"), col(o, "grassB", "#4fa847"));
    let (ra, rb) = (col(o, "rumbleA", "#e8566a"), col(o, "rumbleB", "#f4f4f4"));
    let (roa, rob) = (col(o, "roadA", "#727280"), col(o, "roadB", "#6b6b73"));
    let lane_c = col(o, "lane", "#f4f4f4");
    let no_lane = o.get("lane") == Some(&Json::String(String::new()));
    for &k in vis.iter().rev() {
        let sg = &segs[k];
        let (a, b) = (&sg.p1, &sg.p2);
        let alt = sg.col;
        let start = sg.i < 2;
        g.fill_rect(0.0, b.sy as f32, w as f32, (a.sy - b.sy + 1.0) as f32, if alt { gb } else { ga });
        let r1 = a.sw / (2.0 * lanes).max(6.0);
        let r2 = b.sw / (2.0 * lanes).max(6.0);
        let rc = if alt { rb } else { ra };
        quad(g, a.sx - a.sw - r1, a.sy, r1, b.sx - b.sw - r2, b.sy, r2, rc);
        quad(g, a.sx + a.sw + r1, a.sy, r1, b.sx + b.sw + r2, b.sy, r2, rc);
        quad(g, a.sx, a.sy, a.sw, b.sx, b.sy, b.sw, if alt { rob } else { roa });
        if start {
            for ck in 0..10 {
                let u0 = ck as f64 / 10.0 * 2.0 - 1.0;
                let u1 = (ck + 1) as f64 / 10.0 * 2.0 - 1.0;
                let c = if (ck + sg.i) % 2 == 1 { parse_color("#f4f4f4") } else { parse_color("#1b1f2a") };
                if let Some(p) = poly_path(&[((a.sx + a.sw * u0) as f32, a.sy as f32), ((b.sx + b.sw * u0) as f32, b.sy as f32), ((b.sx + b.sw * u1) as f32, b.sy as f32), ((a.sx + a.sw * u1) as f32, a.sy as f32)], true) {
                    g.fill_path(&p, c);
                }
            }
        }
        if !alt && !no_lane && lanes > 1.0 {
            let l1 = a.sw / (8.0 * lanes).max(32.0);
            let l2 = b.sw / (8.0 * lanes).max(32.0);
            let lw1 = a.sw * 2.0 / lanes;
            let lw2 = b.sw * 2.0 / lanes;
            let mut ln = 1.0;
            while ln < lanes {
                quad(g, a.sx - a.sw + lw1 * ln, a.sy, l1, b.sx - b.sw + lw2 * ln, b.sy, l2, lane_c);
                ln += 1.0;
            }
        }
        if fogs[k] < 1.0 {
            g.save();
            g.mul_alpha((1.0 - fogs[k]) as f32);
            g.fill_rect(0.0, b.sy as f32, w as f32, (a.sy - b.sy + 1.0) as f32, fog_c);
            g.restore();
        }
    }
    let unit = 2.0 * cam_h / w;
    let dsc = rn(o, "spriteScale", 2.0);
    let old_smooth = g.smooth();
    g.set_smooth(false);
    let spr = |g: &mut Gfx, id: &str, k: usize, off: f64, is_car: bool, sp: Option<&RdSpr>| {
        let (dw0, im) = if id == "#sign" {
            let sp = sp.unwrap();
            (50.0, rd_sign_img(sp.dir, sp.lvl))
        } else {
            match lk(id) {
                Some(v) => v,
                None => return,
            }
        };
        let seg = &segs[k];
        let a = &seg.p1;
        if a.s == 0.0 || a.cz <= depth {
            return;
        }
        let (iw, ih) = (im.width() as f64, im.height() as f64);
        let ww = (if dw0 != 0.0 { dw0 } else { iw }) * unit * if is_car { 1.0 } else { dsc };
        let dw = ww * a.s * w / 2.0;
        let dh = dw * ih / iw;
        let dxp = a.sx + a.s * off * rw * w / 2.0 - dw / 2.0;
        let dyp = a.sy - dh;
        let clip = clips[k];
        let clip_h = if clip != 0.0 { (dyp + dh - clip).max(0.0) } else { 0.0 };
        if clip_h >= dh || dw < 1.0 {
            return;
        }
        g.save();
        g.mul_alpha((0.3 + fogs[k]).min(1.0) as f32);
        g.draw_image_src(&im, 0.0, 0.0, iw as f32, (ih - ih * clip_h / dh) as f32, dxp as f32, dyp as f32, dw as f32, (dh - clip_h) as f32);
        g.restore();
    };
    for k in (1..dd).rev() {
        let si = segs[k].i;
        for q in segs[k].spr.clone() {
            spr(g, &q.id, k, q.off, false, Some(&q));
        }
        for car in &st.cars {
            if (car.z / l).floor() as usize % n == si {
                spr(g, &car.id, k, car.off, true, None);
            }
        }
    }
    g.set_smooth(old_smooth);
    if crate::rt::jtruthy(o.get("minimap")) {
        rd_minimap(g, o, &r, st, w, h, pz);
    }
    g.restore();
    Some(hz)
}

fn rd_minimap(g: &mut Gfx, o: &Json, r: &RdBuilt, st: &RoadSt, w: f64, h: f64, pz: f64) {
    let sz = rn(o, "mmSize", 130.0).max(60.0);
    let pad = 12.0;
    let cr = if o.s("mmCorner").is_empty() { "tr" } else { o.s("mmCorner") };
    let x0 = if cr.chars().nth(1) == Some('l') { pad } else { w - sz - pad };
    let y0 = if cr.starts_with('t') { pad } else { h - sz - pad };
    let b = r.mb;
    let sc = (sz - 22.0) / (b[2] - b[0]).max(b[3] - b[1]).max(1e-6);
    let ox = x0 + sz / 2.0 - (b[0] + b[2]) / 2.0 * sc;
    let oy = y0 + sz / 2.0 - (b[1] + b[3]) / 2.0 * sc;
    let n = r.map.len() as i64;
    let pt = |k: i64| {
        let q = r.map[(((k % n) + n) % n) as usize];
        ((ox + q[0] * sc) as f32, (oy + q[1] * sc) as f32)
    };
    g.save();
    g.set_alpha(1.0);
    g.fill_rr(x0 as f32, y0 as f32, sz as f32, sz as f32, 10.0, parse_color("rgba(15,18,26,.72)"));
    g.stroke_rr(x0 as f32, y0 as f32, sz as f32, sz as f32, 10.0, 1.0, parse_color("rgba(255,255,255,.25)"));
    let mut pts = Vec::new();
    let mut k = 0;
    while k <= n {
        pts.push(pt(k));
        k += 2;
    }
    if let Some(p) = poly_path(&pts, true) {
        g.stroke_path(&p, 6.0, parse_color("rgba(0,0,0,.6)"), true);
        g.stroke_path(&p, 3.0, col(o, "roadA", "#b48a52"), true);
    }
    let s0 = pt(0);
    g.fill_rect(s0.0 - 3.0, s0.1 - 3.0, 6.0, 6.0, Rgba::WHITE);
    for car in &st.cars {
        let p = pt((car.z / r.l).floor() as i64);
        g.fill_circle(p.0, p.1, 3.2, if car.rival { parse_color("#4da3ff") } else { parse_color("#c2c3c7") });
    }
    let me = pt((modp(st.pos + pz, r.len) / r.l).floor() as i64);
    g.fill_circle(me.0, me.1, 4.6, parse_color("#ff3b4f"));
    if let Some(c) = crate::gfx::circle_path(me.0, me.1, 4.6) {
        g.stroke_path(&c, 1.5, Rgba::WHITE, false);
    }
    g.restore();
}
