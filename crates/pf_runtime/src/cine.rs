//! Cinématiques (suite d'étapes : dialogues, caméra, déplacements, fondus, titres…).

use crate::model::*;
use crate::rt::*;
use pf_core::{Json, JsonExt};

pub struct CineRun {
    pub s: Json,
    pub t: f64,
    pub d: f64,
    pub done: bool,
    pub say: bool,
    pub f: (f64, f64),
    pub to: (f64, f64),
    pub o: Option<Slot>,
    pub from_a: f64,
    pub to_a: f64,
}

pub struct Cine {
    pub def: Json,
    pub k: usize,
    pub run: Vec<CineRun>,
    pub follow: Option<Slot>,
}

fn cine_ease(e: f64, k: &str) -> f64 {
    match k {
        "linear" => e,
        "in" => e * e,
        "out" => 1.0 - (1.0 - e) * (1.0 - e),
        _ => {
            if e < 0.5 {
                2.0 * e * e
            } else {
                1.0 - (-2.0 * e + 2.0).powi(2) / 2.0
            }
        }
    }
}

impl Runtime {
    pub fn cine_lock(&self) -> bool {
        self.cine.as_ref().map(|c| c.def.get("lock") != Some(&Json::Bool(false))).unwrap_or(false)
    }
    fn cine_inst(&self, id: &str) -> Option<Slot> {
        self.type_by_id.get(id).and_then(|&t| self.alive_insts(t).first().copied())
    }
    fn cine_center(&self, s: Slot) -> (f64, f64) {
        self.center(s)
    }
    fn cam_center(&self) -> (f64, f64) {
        if let Some(c) = self.cam_look {
            return c;
        }
        (self.cam.0 + self.w / self.zoom / 2.0, self.cam.1 + self.h / self.zoom / 2.0)
    }
    fn hero_center(&self) -> (f64, f64) {
        let tg: Vec<Slot> = self.all.iter().copied().filter(|&s| self.alive(s) && self.inst(s).b.scrollto.is_some()).collect();
        if tg.is_empty() {
            return self.cam_center();
        }
        let (mut x, mut y) = (0.0, 0.0);
        for &s in &tg {
            let c = self.cine_center(s);
            x += c.0;
            y += c.1;
        }
        let oy = self.inst(tg[0]).b.scrollto.as_ref().map(|s| s.offset_y).unwrap_or(0.0);
        (x / tg.len() as f64, y / tg.len() as f64 + oy)
    }
    fn cine_zoom(&mut self, z: f64, d: f64) {
        if !(z > 0.0) {
            return;
        }
        let z = (z / 100.0).clamp(0.1, 10.0);
        if d <= 0.0 {
            self.zoom = z;
            self.zoom_a = None;
        } else {
            self.zoom_a = Some(ZoomAnim { from: self.zoom, to: z, t: 0.0, d });
        }
    }
    pub fn cine_play(&mut self, id: &str) {
        let p = self.p.clone();
        let d = match p.arr("cines").iter().find(|c| c.s("id") == id || c.s("name") == id) {
            Some(d) => d.clone(),
            None => return,
        };
        if self.cine.is_some() {
            self.cine_skip();
        }
        self.cine = Some(Cine { def: d, k: 0, run: Vec::new(), follow: None });
        self.cine_next();
    }
    fn cine_next(&mut self) {
        loop {
            let (steps, k) = match &self.cine {
                Some(c) => (c.def.arr("steps").to_vec(), c.k),
                None => return,
            };
            if k >= steps.len() {
                self.cine_end();
                return;
            }
            let mut g = vec![steps[k].clone()];
            let mut k2 = k + 1;
            while k2 < steps.len() && crate::rt::jtruthy(steps[k2].get("w")) {
                g.push(steps[k2].clone());
                k2 += 1;
            }
            self.cine.as_mut().unwrap().k = k2;
            let run: Vec<CineRun> = g.into_iter().map(|s| self.cine_start(s)).collect();
            let all_done = run.iter().all(|r| r.done);
            if let Some(c) = self.cine.as_mut() {
                c.run = run;
            } else {
                return;
            }
            if !all_done {
                return;
            }
        }
    }
    fn cine_start(&mut self, st: Json) -> CineRun {
        let d = Val0::n(&st, "d").max(0.0);
        let mut r = CineRun { s: st.clone(), t: 0.0, d, done: false, say: false, f: (0.0, 0.0), to: (0.0, 0.0), o: None, from_a: 0.0, to_a: 0.0 };
        let ty = st.s("t");
        match ty {
            "say" => {
                self.dlg_push(Dialog { name: jstr_or(&st, "who"), text: jstr_or(&st, "text"), portrait: st.s("o").to_string(), top: st.s("pos") == "top", choices: None, shown: 0, acc: 0.0, sel: 0, t: 0.0, hit: None });
                r.say = true;
            }
            "camto" => {
                r.f = self.cam_center();
                let x = self.x(st.get("x").unwrap_or(&Json::Null)).num0();
                let y = self.x(st.get("y").unwrap_or(&Json::Null)).num0();
                r.to = (x, y);
                if let Some(c) = self.cine.as_mut() {
                    c.follow = None;
                }
                if crate::rt::jtruthy(st.get("zoom")) {
                    self.cine_zoom(Val0::n(&st, "zoom"), r.d);
                }
            }
            "camobj" => {
                r.f = self.cam_center();
                r.o = self.cine_inst(st.s("o"));
                let o = r.o;
                if let Some(c) = self.cine.as_mut() {
                    c.follow = o;
                }
                if crate::rt::jtruthy(st.get("zoom")) {
                    self.cine_zoom(Val0::n(&st, "zoom"), r.d);
                }
            }
            "camback" => {
                r.f = self.cam_center();
                if let Some(c) = self.cine.as_mut() {
                    c.follow = None;
                }
                if self.zoom != 1.0 {
                    self.cine_zoom(100.0, r.d);
                }
            }
            "move" => match self.cine_inst(st.s("o")) {
                None => r.done = true,
                Some(o) => {
                    r.o = Some(o);
                    let (ox, oy) = (self.inst(o).x, self.inst(o).y);
                    r.f = (ox, oy);
                    let tx = self.x(st.get("x").unwrap_or(&Json::Null)).num0();
                    let ty2 = self.x(st.get("y").unwrap_or(&Json::Null)).num0();
                    r.to = if crate::rt::jtruthy(st.get("rel")) { (ox + tx, oy + ty2) } else { (tx, ty2) };
                    let sprite = self.def(o).kind == Kind::Sprite;
                    let i = self.inst_mut(o);
                    if sprite && st.get("face") != Some(&Json::Bool(false)) && (r.to.0 - ox).abs() > 0.5 {
                        i.mirrored = r.to.0 < ox;
                    }
                    if let Some(p) = i.b.platform.as_mut() {
                        p.vx = 0.0;
                        p.vy = 0.0;
                    }
                }
            },
            "anim" => {
                if let Some(o) = self.cine_inst(st.s("o")) {
                    let d = self.def(o);
                    if d.kind == Kind::Sprite {
                        if let Some(ai) = d.anims.iter().position(|a| a.name == st.s("a")) {
                            let i = self.inst_mut(o);
                            if ai != i.anim {
                                i.anim = ai;
                                i.frame = 0;
                                i.ft = 0.0;
                            }
                        }
                    }
                }
                r.done = true;
            }
            "title" => self.cine_title = Some((jstr_or(&st, "text"), 0.0, r.d.max(0.5))),
            "fade" => {
                r.from_a = self.cine_fade.0;
                r.to_a = if st.s("mode") == "in" { 0.0 } else { 1.0 };
                self.cine_fade.1 = if st.s("color").is_empty() { "#000000".into() } else { st.s("color").to_string() };
                if r.d <= 0.0 {
                    self.cine_fade.0 = r.to_a;
                    r.done = true;
                }
            }
            "sound" => {
                let s = st.s("s").to_string();
                if !s.is_empty() {
                    if let Some(f) = s.strip_prefix("file:") {
                        self.host.audio(AudioCmd::PlayFile { id: f.to_string(), vol: 1.0, looping: false, tag: String::new() });
                    } else {
                        self.sfx(&s);
                    }
                }
                r.done = true;
            }
            "vis" => {
                if let Some(&t) = self.type_by_id.get(st.s("o")) {
                    let v = st.get("v") != Some(&Json::Bool(false));
                    for s in self.types[t].insts.clone() {
                        self.inst_mut(s).visible = v;
                    }
                }
                r.done = true;
            }
            "call" => {
                let f = st.s("f").to_string();
                if !f.is_empty() {
                    self.run_func(&f, Vec::new());
                }
                r.done = true;
            }
            "shake" => {
                self.shake_a = { let a = Val0::n(&st, "a"); if a == 0.0 { 6.0 } else { a } };
                self.shake_d = r.d.max(0.01);
                self.shake_t = self.shake_d;
            }
            _ => {}
        }
        if !r.say && r.d <= 0.0 && ["wait", "camto", "camobj", "camback", "move", "title", "shake"].contains(&ty) {
            self.cine_apply(&r, 1.0);
            r.done = true;
        }
        r
    }
    fn cine_apply(&mut self, r: &CineRun, e: f64) {
        let st = &r.s;
        let k = cine_ease(e, st.s("ease"));
        match st.s("t") {
            "camto" => self.cam_look = Some((r.f.0 + (r.to.0 - r.f.0) * k, r.f.1 + (r.to.1 - r.f.1) * k)),
            "camobj" => {
                if let Some(o) = r.o.filter(|&o| self.alive(o)) {
                    let c = self.cine_center(o);
                    self.cam_look = Some((r.f.0 + (c.0 - r.f.0) * k, r.f.1 + (c.1 - r.f.1) * k));
                }
            }
            "camback" => {
                let h = self.hero_center();
                self.cam_look = Some((r.f.0 + (h.0 - r.f.0) * k, r.f.1 + (h.1 - r.f.1) * k));
                if e >= 1.0 {
                    self.cam_look = None;
                }
            }
            "move" => {
                if let Some(o) = r.o.filter(|&o| self.alive(o)) {
                    let i = self.inst_mut(o);
                    i.x = r.f.0 + (r.to.0 - r.f.0) * k;
                    i.y = r.f.1 + (r.to.1 - r.f.1) * k;
                }
            }
            "fade" => self.cine_fade.0 = r.from_a + (r.to_a - r.from_a) * k,
            _ => {}
        }
    }
    pub fn cine_step(&mut self, dt: f64) {
        if self.cine.is_none() {
            if self.cine_bars > 0.0 {
                self.cine_bars = (self.cine_bars - dt * 3.0).max(0.0);
            }
            if let Some(t) = self.cine_title.as_mut() {
                t.1 += dt;
                if t.1 >= t.2 {
                    self.cine_title = None;
                }
            }
            return;
        }
        let def = self.cine.as_ref().unwrap().def.clone();
        if def.get("bars") != Some(&Json::Bool(false)) {
            self.cine_bars = (self.cine_bars + dt * 3.0).min(1.0);
        }
        if let Some(t) = self.cine_title.as_mut() {
            t.1 += dt;
            if t.1 >= t.2 {
                self.cine_title = None;
            }
        }
        if def.get("skip") != Some(&Json::Bool(false)) && self.pkp.contains("Escape") {
            self.pkp.clear();
            self.kp.clear();
            self.cine_skip();
            return;
        }
        let mut busy_cam = false;
        let mut run = std::mem::take(&mut self.cine.as_mut().unwrap().run);
        for r in run.iter_mut() {
            if r.done {
                continue;
            }
            r.t += dt;
            if r.say {
                if r.t > 0.05 && self.dlg.cur.is_none() && self.dlg.queue.is_empty() {
                    r.done = true;
                }
                continue;
            }
            let e = if r.d > 0.0 { (r.t / r.d).min(1.0) } else { 1.0 };
            self.cine_apply(r, e);
            if ["camto", "camobj", "camback"].contains(&r.s.s("t")) {
                busy_cam = true;
            }
            if e >= 1.0 {
                r.done = true;
            }
        }
        if let Some(c) = self.cine.as_mut() {
            c.run = run;
        } else {
            return;
        }
        if !busy_cam {
            if let Some(f) = self.cine.as_ref().unwrap().follow {
                if !self.alive(f) {
                    self.cine.as_mut().unwrap().follow = None;
                } else {
                    self.cam_look = Some(self.cine_center(f));
                }
            }
        }
        if self.cine.as_ref().map(|c| c.run.iter().all(|r| r.done)).unwrap_or(false) {
            self.cine_next();
        }
    }
    pub fn cine_skip(&mut self) {
        let c = match self.cine.take() {
            Some(c) => c,
            None => return,
        };
        for r in &c.run {
            if !r.done && !r.say {
                self.cine_apply(r, 1.0);
            }
        }
        let steps = c.def.arr("steps").to_vec();
        let k0 = c.k;
        self.cine = Some(Cine { def: c.def.clone(), k: c.k, run: Vec::new(), follow: c.follow });
        for st in steps.iter().skip(k0) {
            if !["move", "anim", "vis", "fade", "call"].contains(&st.s("t")) {
                continue;
            }
            let r = self.cine_start(st.clone());
            if !r.done {
                self.cine_apply(&r, 1.0);
            }
        }
        self.dlg.queue.clear();
        if self.dlg.cur.is_some() {
            self.dlg.cur = None;
            self.dlg.end_tick = self.tick;
        }
        self.cine_title = None;
        self.cine_end();
    }
    fn cine_end(&mut self) {
        let c = match self.cine.take() {
            Some(c) => c,
            None => return,
        };
        self.cine_end_id = c.def.s("id").to_string();
        self.cine_end_tick = self.tick + 1;
        self.cam_look = None;
        if self.zoom != 1.0 {
            self.cine_zoom(100.0, 0.4);
        }
    }
}

struct Val0;
impl Val0 {
    fn n(j: &Json, k: &str) -> f64 {
        j.get(k).map(|v| pf_core::Val::from_json(v).num0()).unwrap_or(0.0)
    }
}
fn jstr_or(j: &Json, k: &str) -> String {
    match j.get(k) {
        None | Some(Json::Null) => String::new(),
        Some(v) => crate::rt::jstr(v),
    }
}
