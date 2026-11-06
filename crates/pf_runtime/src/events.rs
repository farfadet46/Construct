//! Feuilles d'événements : conditions, actions, boucles, groupes, fonctions, attentes.

use crate::coll::draw_box;
use crate::model::*;
use crate::rt::*;
use pf_core::value::cmp_op;
use pf_core::{Json, JsonExt, Val};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EvKind {
    Event,
    Group,
    Comment,
    Function,
    Lvar,
    Include,
    Other,
}

#[derive(Clone, Debug)]
pub struct Cond {
    pub id: String,
    pub o: String,
    pub ot: Option<usize>,
    pub k: String,
    pub prov: String,
    pub key: String,
    pub p: Vec<Json>,
    pub not: bool,
    pub off: bool,
}

#[derive(Clone, Debug)]
pub struct Act {
    pub o: String,
    pub ot: Option<usize>,
    pub prov: String,
    pub key: String,
    pub p: Vec<Json>,
    pub off: bool,
}

#[derive(Debug)]
pub struct Ev {
    pub id: String,
    pub kind: EvKind,
    pub off: bool,
    pub conds: Vec<Cond>,
    pub acts: Rc<Vec<Act>>,
    pub subs: Rc<Vec<Rc<Ev>>>,
    pub or: bool,
    pub name: String,
    pub active: bool,
    pub params: Vec<String>,
    pub sheet: String,
    pub is_static: bool,
    pub value: Json,
    pub text_type: bool,
}

fn split_key(k: &str) -> (String, String) {
    match k.rfind('.') {
        Some(p) => (k[..p].to_string(), k[p + 1..].to_string()),
        None => (String::new(), k.to_string()),
    }
}

impl Ev {
    pub fn parse(e: &Json, types: &HashMap<String, usize>) -> Ev {
        let kind = match e.s("kind") {
            "event" => EvKind::Event,
            "group" => EvKind::Group,
            "comment" => EvKind::Comment,
            "function" => EvKind::Function,
            "lvar" => EvKind::Lvar,
            "include" => EvKind::Include,
            _ => EvKind::Other,
        };
        let conds = e.arr("conds").iter().map(|c| {
            let k = c.s("k").to_string();
            let (prov, key) = split_key(&k);
            let o = c.s("o").to_string();
            Cond { id: c.s("id").to_string(), ot: types.get(&o).copied(), o, k, prov, key, p: c.arr("p").to_vec(), not: jtruthy(c.get("not")), off: jtruthy(c.get("off")) }
        }).collect();
        let acts = e.arr("acts").iter().map(|a| {
            let (prov, key) = split_key(a.s("k"));
            let o = a.s("o").to_string();
            Act { ot: types.get(&o).copied(), o, prov, key, p: a.arr("p").to_vec(), off: jtruthy(a.get("off")) }
        }).collect();
        let subs = e.arr("subs").iter().map(|x| Rc::new(Ev::parse(x, types))).collect();
        Ev {
            id: e.s("id").to_string(),
            kind,
            off: jtruthy(e.get("off")),
            conds,
            acts: Rc::new(acts),
            subs: Rc::new(subs),
            or: jtruthy(e.get("or")),
            name: e.s("name").to_string(),
            active: e.get("active") != Some(&Json::Bool(false)),
            params: e.arr("params").iter().map(|p| jstr(p)).collect(),
            sheet: e.s("sheet").to_string(),
            is_static: jtruthy(e.get("static")),
            value: e.get("value").cloned().unwrap_or(Json::Null),
            text_type: e.s("type") == "text",
        }
    }
}

fn pj(p: &[Json], k: usize) -> &Json {
    static NULL: Json = Json::Null;
    p.get(k).unwrap_or(&NULL)
}
fn ps(p: &[Json], k: usize) -> String {
    match p.get(k) {
        None | Some(Json::Null) => String::new(),
        Some(v) => jstr(v),
    }
}
/// String(p[k]||'défaut')
fn ps_or(p: &[Json], k: usize, d: &str) -> String {
    if jtruthy(p.get(k)) {
        jstr(&p[k])
    } else {
        d.to_string()
    }
}
fn pb(p: &[Json], k: usize) -> bool {
    jtruthy(p.get(k))
}

const LOOPS: [&str; 5] = ["repeat", "forl", "foreach", "while", "foreachitem"];

impl Runtime {
    fn xp(&mut self, p: &[Json], k: usize) -> Val {
        let v = pj(p, k).clone();
        self.x(&v)
    }
    fn xpn(&mut self, p: &[Json], k: usize) -> f64 {
        self.xp(p, k).num0()
    }
    fn tid(&self, p: &[Json], k: usize) -> Option<usize> {
        p.get(k).and_then(|v| v.as_str()).and_then(|s| self.type_by_id.get(s)).copied()
    }

    // ---------------------------------------------------------------- conditions

    fn sys_cond(&mut self, key: &str, p: &[Json], c: &Cond, sol: &mut Sol) -> bool {
        match key {
            "start" => self.first_tick,
            "tick" => true,
            "ctlopen" => self.ctl.open,
            "gamepaused" => self.gpaused,
            "hsqual" => {
                let l = self.hs_list(&ps(p, 0));
                let mx = { let v = self.xpn(p, 2).floor(); if v == 0.0 { 10.0 } else { v } }.max(1.0);
                let sc = self.xpn(p, 1);
                Runtime::hs_fits(&l, sc, mx as usize, ps(p, 3) == "asc")
            }
            "hsnamed" => self.hs_named_tick == self.tick,
            "hsopen" => self.hs_in.is_some(),
            "cineplaying" => self.cine.is_some(),
            "cineended" => self.cine_end_tick == self.tick && (!pb(p, 0) || ps(p, 0) == self.cine_end_id),
            "hassave" => self.sget(&format!("save:{}", ps_or(p, 0, "partie"))).map(|v| jtruthy(Some(&v))).unwrap_or(false),
            "hascp" => self.cp.as_ref().map(|c| c.layout == self.layout_id).unwrap_or(false),
            "dlgopen" => self.dlg.cur.is_some(),
            "dlgend" => self.dlg.end_tick == self.tick,
            "dlgchoice" => {
                let n = self.xpn(p, 0).floor();
                self.dlg.choice_tick == self.tick && (n <= 0.0 || n == self.dlg.choice)
            }
            "every" => {
                let lim = self.xpn(p, 0);
                let s = self.cstate.entry(c.id.clone()).or_default();
                s.t += DT;
                if s.t >= lim - 1e-9 {
                    s.t = 0.0;
                    true
                } else {
                    false
                }
            }
            "compare" => {
                let a = self.xp(p, 0);
                let b = self.xp(p, 2);
                cmp_op(&a, &ps(p, 1), &b)
            }
            "gcmp" => {
                let a = self.get_var(&ps(p, 0)).unwrap_or(Val::N(f64::NAN));
                let b = self.xp(p, 2);
                match self.get_var(&ps(p, 0)) {
                    Some(_) => cmp_op(&a, &ps(p, 1), &b),
                    None => cmp_undefined(&ps(p, 1), &b),
                }
            }
            "groupactive" => self.groups_by_name.get(&ps(p, 0)).map(|g| g.iter().any(|e| self.group_on(e))).unwrap_or(false),
            "chance" => {
                let v = self.xpn(p, 0);
                self.rand() * 100.0 < v
            }
            "islayout" => self.layout_id == ps(p, 0),
            "pickall" => match self.tid(p, 0) {
                Some(t) => {
                    sol.remove(&t);
                    true
                }
                None => false,
            },
            "count" => {
                let n = self.tid(p, 0).map(|t| self.alive_insts(t).len()).unwrap_or(0);
                let b = self.xp(p, 2);
                cmp_op(&Val::N(n as f64), &ps(p, 1), &b)
            }
            _ => false,
        }
    }

    fn obj_cond(&mut self, prov: &str, key: &str, s: Slot, p: &[Json], sol: &mut Sol) -> bool {
        let tick = self.tick;
        match (prov, key) {
            ("common", "cmpx") => {
                let x = self.inst(s).x;
                let b = self.xp(p, 1);
                cmp_op(&Val::N(x), &ps(p, 0), &b)
            }
            ("common", "cmpy") => {
                let y = self.inst(s).y;
                let b = self.xp(p, 1);
                cmp_op(&Val::N(y), &ps(p, 0), &b)
            }
            ("common", "onscreen") => {
                let i = self.inst(s);
                if i.hud {
                    return true;
                }
                let b = self.draw_box_i(s);
                let (px, py) = self.layer_pp(i.layer);
                let ox = self.cam.0 * px;
                let oy = self.cam.1 * py;
                let z = self.zf(i.layer, i.hud);
                b.r > ox && b.l < ox + self.w / z && b.b > oy && b.t < oy + self.h / z
            }
            ("common", "onlayer") => self.layers.get(self.inst(s).layer).map(|l| l.name == ps(p, 0)).unwrap_or(false),
            ("common", "pickby") => {
                let a = self.xp(p, 0);
                let b = self.xp(p, 2);
                cmp_op(&a, &ps(p, 1), &b)
            }
            ("common", "outside") => {
                let b = self.draw_box_i(s);
                b.r < 0.0 || b.l > self.lw() || b.b < 0.0 || b.t > self.lh()
            }
            ("common", "ivar") => {
                let v = self.inst(s).vars.get(&ps(p, 0)).cloned();
                let b = self.xp(p, 2);
                match v {
                    Some(v) => cmp_op(&v, &ps(p, 1), &b),
                    None => cmp_undefined(&ps(p, 1), &b),
                }
            }
            ("common", "visible") => self.inst(s).visible,
            ("sprite", "animis") => {
                let d = self.def(s);
                d.anims.get(self.inst(s).anim).map(|a| a.name == ps(p, 0)).unwrap_or(false)
            }
            ("sprite", "animend") => {
                let d = self.def(s);
                let i = self.inst(s);
                let a = d.anims.get(i.anim);
                i.anim_end == tick && (!pb(p, 0) || a.map(|a| a.name == ps(p, 0)).unwrap_or(false))
            }
            ("sprite", "mirrored") => self.inst(s).mirrored,
            ("text", "cmptext") => {
                let t = self.inst(s).text.str();
                let v = self.xp(p, 0).str();
                t == v
            }
            ("b:platform", k) => {
                let st = self.inst(s).b.platform.as_ref().unwrap();
                match k {
                    "floor" => st.floor.is_some(),
                    "onobj" => st.floor.map(|f| !self.ref_dead(f) && self.types[self.ref_type(f)].id == ps(p, 0)).unwrap_or(false),
                    "moving" => st.vx.abs() > 1.0,
                    "jumping" => st.floor.is_none() && st.climb.is_none() && st.vy < 0.0,
                    "falling" => st.floor.is_none() && st.climb.is_none() && st.vy > 0.0,
                    "onwall" => st.floor.is_none() && st.wall_t > 0.0,
                    "walljumped" => st.wj == tick,
                    "climbing" => st.climb.is_some(),
                    "landed" => st.landed,
                    "jumped" => st.jumped,
                    _ => false,
                }
            }
            ("b:patrol", k) => {
                let st = self.inst(s).b.patrol.as_ref().unwrap();
                match k {
                    "goingleft" => st.dir < 0.0,
                    "chasing" => st.chasing,
                    "spotted" => st.spot == tick,
                    _ => false,
                }
            }
            ("b:timer", k) => {
                let st = self.inst(s).b.timer.as_ref().unwrap();
                let n = ps(p, 0);
                match k {
                    "ontimer" => st.fired.get(&n) == Some(&tick),
                    "running" => st.timers.iter().any(|t| t.0 == n),
                    _ => false,
                }
            }
            ("b:eightdir", "moving") => {
                let st = self.inst(s).b.eightdir.as_ref().unwrap();
                st.vx.abs() + st.vy.abs() > 1.0
            }
            ("b:car", k) => {
                let st = self.inst(s).b.car.clone().unwrap();
                match k {
                    "moving" => st.sp.abs() > 4.0,
                    "reversing" => st.sp < -4.0,
                    "driven" => st.ctl,
                    "hit" => st.hit == tick && st.hit_v >= self.xpn(p, 0),
                    "cmpspeed" => {
                        let b = self.xp(p, 1);
                        cmp_op(&Val::N(st.sp.abs()), &ps(p, 0), &b)
                    }
                    _ => false,
                }
            }
            ("tilemap", "cmptile") => {
                let (c, r) = (self.xpn(p, 0), self.xpn(p, 1));
                let v = self.tm_get(s, c, r);
                let b = self.xpn(p, 3);
                cmp_op(&Val::N(v as f64), &ps(p, 2), &Val::N(b))
            }
            ("tilemap", "cmptileat") => {
                let (x, y) = (self.xpn(p, 0), self.xpn(p, 1));
                let (c, r) = (self.tm_col(s, x), self.tm_row(s, y));
                let v = self.tm_get(s, c, r);
                let b = self.xpn(p, 3);
                cmp_op(&Val::N(v as f64), &ps(p, 2), &Val::N(b))
            }
            ("particles", "empty") => self.inst(s).ps.as_ref().map(|p| p.parts.is_empty()).unwrap_or(true),
            ("road3d", k) => {
                let rd = match self.inst(s).rd.as_deref() {
                    Some(r) => r.clone(),
                    None => return false,
                };
                match k {
                    "lapdone" => rd.lap_t == tick,
                    "hit" => rd.hit_t == tick,
                    "offroad" => rd.x.abs() > 1.0,
                    "cmpspeed" => {
                        let v = self.rd_kmh(s);
                        let b = self.xp(p, 1);
                        cmp_op(&Val::N(v), &ps(p, 0), &b)
                    }
                    "driven" => rd.ctl,
                    _ => false,
                }
            }
            ("bar", k) => {
                let b = match &self.inst(s).bar {
                    Some(b) => b.clone(),
                    None => return false,
                };
                match k {
                    "cmpvalue" => {
                        let v = self.xp(p, 1);
                        cmp_op(&Val::N(b.v), &ps(p, 0), &v)
                    }
                    "empty" => b.v <= 0.0,
                    "full" => b.v >= b.max,
                    _ => false,
                }
            }
            ("button", k) => {
                let b = match &self.inst(s).btn {
                    Some(b) => b,
                    None => return false,
                };
                match k {
                    "clicked" => b.clicked == tick,
                    "hover" => b.hover,
                    "focused" => b.focus,
                    _ => false,
                }
            }
            ("b:tween", k) => {
                let st = self.inst(s).b.tween.as_ref().unwrap();
                let n = ps(p, 0);
                match k {
                    "done" => st.done.get(&n) == Some(&tick),
                    "running" => st.list.iter().any(|t| t.tag == n),
                    _ => false,
                }
            }
            ("b:fade", "done") => self.inst(s).b.fade.as_ref().unwrap().done == tick,
            ("b:physics", k) => {
                let b = self.inst(s).b.physics.clone().unwrap();
                match k {
                    "moving" => b.vx.abs() + b.vy.abs() > 12.0,
                    "onground" => b.ground,
                    "hit" => b.hit_t == tick && b.hit_v >= self.xpn(p, 0),
                    "roped" => self.joints.iter().any(|j| j.a == s || j.b == Some(s)),
                    _ => false,
                }
            }
            ("b:path", k) => {
                let st = self.inst(s).b.path.as_ref().unwrap();
                match k {
                    "arrived" => st.arr == tick,
                    "finished" => st.fin == tick,
                    "moving" => st.en && st.pts.len() > 1 && !(st.wt > 0.0),
                    _ => false,
                }
            }
            ("b:pin", "pinned") => self.inst(s).b.pin.as_ref().unwrap().to.map(|t| self.alive(t)).unwrap_or(false),
            ("b:drag", k) => {
                let d = self.inst(s).b.drag.as_ref().unwrap();
                match k {
                    "dragging" => d.on,
                    "start" => d.st == tick,
                    "drop" => d.dr == tick,
                    _ => false,
                }
            }
            ("b:turret", k) => {
                let t = self.inst(s).b.turret.as_ref().unwrap();
                match k {
                    "shoot" => t.fired == tick,
                    "target" => t.target.map(|x| self.alive(x)).unwrap_or(false),
                    _ => false,
                }
            }
            ("b:los", "sees") => {
                let t = match self.tid(p, 0) {
                    Some(t) => t,
                    None => return false,
                };
                let seen: Vec<Slot> = self.types[t].insts.iter().copied().filter(|&j| self.alive(j) && j != s && self.can_see(s, j)).collect();
                if !seen.is_empty() {
                    let key = LOS_KEY + t;
                    let mut prev: Vec<Slot> = sol.get(&key).map(|v| (**v).clone()).unwrap_or_default();
                    for j in &seen {
                        if !prev.contains(j) {
                            prev.push(*j);
                        }
                    }
                    sol.insert(key, Rc::new(prev));
                }
                !seen.is_empty()
            }
            _ => false,
        }
    }

    fn data_cond(&mut self, t: usize, key: &str, p: &[Json]) -> bool {
        let kind = self.types[t].def.kind;
        match (kind, key) {
            (Kind::Array, "contains") => {
                let v = dval(self.xp(p, 0));
                matches!(self.data.get(&t), Some(DataStore::List(l)) if l.iter().any(|x| x.loose_eq(&v)))
            }
            (Kind::Array, "empty") => matches!(self.data.get(&t), Some(DataStore::List(l)) if l.is_empty()),
            (Kind::Array, "cmpat") => {
                let k = self.xpn(p, 0).floor();
                let b = self.xp(p, 2);
                let v = match self.data.get(&t) {
                    Some(DataStore::List(l)) if k >= 0.0 && (k as usize) < l.len() => Some(l[k as usize].clone()),
                    _ => None,
                };
                match v {
                    Some(v) => cmp_op(&v, &ps(p, 1), &b),
                    None => cmp_undefined(&ps(p, 1), &b),
                }
            }
            (Kind::Dict, "haskey") => {
                let k = self.xp(p, 0).str();
                self.data.get(&t).and_then(|d| d.map_get(&k)).is_some()
            }
            (Kind::Dict, "empty") => matches!(self.data.get(&t), Some(DataStore::Map(m)) if m.is_empty()),
            (Kind::Dict, "cmpkey") => {
                let k = self.xp(p, 0).str();
                let b = self.xp(p, 2);
                match self.data.get(&t).and_then(|d| d.map_get(&k)).cloned() {
                    Some(v) => cmp_op(&v, &ps(p, 1), &b),
                    None => cmp_undefined(&ps(p, 1), &b),
                }
            }
            _ => false,
        }
    }

    fn pick_one(&mut self, c: &Cond, sol: &mut Sol, t: usize) -> bool {
        let list = self.picked(sol, t);
        if list.is_empty() {
            sol.insert(t, Rc::new(Vec::new()));
            return false;
        }
        let mut best = list[0];
        if c.key == "prand" {
            let k = (self.rand() * list.len() as f64) as usize;
            best = list[k.min(list.len() - 1)];
        } else {
            let x = self.xpn(&c.p, 0);
            let y = self.xpn(&c.p, 1);
            let near = c.key == "pnear";
            let mut bd = if near { f64::INFINITY } else { -1.0 };
            for &s in &list {
                let i = self.inst(s);
                let d = (i.x - x) * (i.x - x) + (i.y - y) * (i.y - y);
                if if near { d < bd } else { d > bd } {
                    bd = d;
                    best = s;
                }
            }
        }
        sol.insert(t, Rc::new(vec![best]));
        true
    }

    fn coll_cond(&mut self, c: &Cond, sol: &mut Sol, t: usize, trig: bool) -> bool {
        let o = match self.tid(&c.p, 0) {
            Some(o) => o,
            None => return false,
        };
        let a = self.picked(sol, t);
        let b = self.picked(sol, o);
        let sb: Vec<crate::coll::CShape> = b.iter().map(|&s| self.shape(s)).collect();
        if c.not && !trig {
            let mut ra = Vec::new();
            for &x in &a {
                let ba = self.shape(x);
                if !b.iter().enumerate().any(|(k, &y)| y != x && self.hits(&ba, &sb[k])) {
                    ra.push(x);
                }
            }
            let any = !ra.is_empty();
            sol.insert(t, Rc::new(ra));
            return any;
        }
        let prev = self.cstate.get(&c.id).map(|s| s.pairs.clone()).unwrap_or_default();
        let mut now = HashSet::new();
        let mut ra = Vec::new();
        let mut rb: Vec<Slot> = Vec::new();
        for &x in &a {
            let ba = self.shape(x);
            let mut any = false;
            for (m, &y) in b.iter().enumerate() {
                if x == y {
                    continue;
                }
                if self.hits(&ba, &sb[m]) {
                    let key = (self.inst(x).uid, self.inst(y).uid);
                    now.insert(key);
                    if !trig || !prev.contains(&key) {
                        any = true;
                        if !rb.contains(&y) {
                            rb.push(y);
                        }
                    }
                }
            }
            if any {
                ra.push(x);
            }
        }
        if trig {
            self.cstate.entry(c.id.clone()).or_default().pairs = now;
        }
        if ra.is_empty() {
            return false;
        }
        sol.insert(t, Rc::new(ra));
        if o != t {
            sol.insert(o, Rc::new(rb));
        }
        true
    }

    pub fn eval_cond(&mut self, c: &Cond, sol: &mut Sol, prev_ok: bool) -> bool {
        if c.off {
            return true;
        }
        self.cur_sol = sol.clone();
        self.cur = None;
        let key = c.key.as_str();
        let p = &c.p;
        match c.o.as_str() {
            "system" => {
                if key == "else" {
                    return !prev_ok;
                }
                if key == "once" {
                    let tick = self.tick;
                    let s = self.cstate.entry(c.id.clone()).or_insert_with(|| CState { last: -5, ..Default::default() });
                    let r = s.last != tick - 1;
                    s.last = tick;
                    return r;
                }
                let r = self.sys_cond(key, p, c, sol);
                return r != c.not;
            }
            "keyboard" => {
                let k = ps(p, 0);
                let r = match key {
                    "pressed" => self.kp.contains(&k),
                    "down" => self.keys.contains(&k),
                    "released" => self.kr.contains(&k),
                    "any" => self.any_press,
                    _ => false,
                };
                return r != c.not;
            }
            "audio" => {
                let r = match key {
                    "musicplaying" => self.music.is_some(),
                    "tagplaying" => self.host.tag_playing(&ps(p, 0)),
                    _ => false,
                };
                return r != c.not;
            }
            "storage" => {
                let r = key == "exists" && self.sget(&ps(p, 0)).is_some();
                return r != c.not;
            }
            "mouse" => {
                if key == "clickobj" || key == "overobj" {
                    let mt = match self.tid(p, if key == "clickobj" { 1 } else { 0 }) {
                        Some(t) => t,
                        None => return false,
                    };
                    if key == "clickobj" && !self.mouse.pr.contains(&ps(p, 0)) {
                        return false;
                    }
                    let inv = c.not && key == "overobj";
                    let hl: Vec<Slot> = self.picked(sol, mt).into_iter().filter(|&s| self.mouse_over(s) != inv).collect();
                    let any = !hl.is_empty();
                    sol.insert(mt, Rc::new(hl));
                    return any;
                }
                let b = ps(p, 0);
                let r = match key {
                    "click" => self.mouse.pr.contains(&b),
                    "down" => self.mouse.down.contains(&b),
                    "released" => self.mouse.rl.contains(&b),
                    _ => false,
                };
                return r != c.not;
            }
            _ => {}
        }
        let t = match c.ot {
            Some(t) => t,
            None => return false,
        };
        let kind = self.types[t].def.kind;
        if kind == Kind::Array || kind == Kind::Dict {
            let r = self.data_cond(t, key, p);
            return r != c.not;
        }
        if key == "collide" || key == "overlap" {
            return self.coll_cond(c, sol, t, key == "collide");
        }
        if key == "pnear" || key == "pfar" || key == "prand" {
            return self.pick_one(c, sol, t);
        }
        let prov = c.prov.as_str();
        let list = self.picked(sol, t);
        let lt = if prov == "b:los" { self.tid(p, 0) } else { None };
        if let Some(lt) = lt {
            sol.remove(&(LOS_KEY + lt));
        }
        let bname = prov.strip_prefix("b:");
        let mut res = Vec::new();
        for s in list {
            if let Some(b) = bname {
                if !self.inst(s).b.has(b) {
                    continue;
                }
            }
            self.cur = Some(s);
            if self.obj_cond(prov, key, s, p, sol) != c.not {
                res.push(s);
            }
        }
        if let Some(lt) = lt {
            let seen = sol.remove(&(LOS_KEY + lt));
            if let Some(seen) = seen {
                if !c.not && lt != t {
                    sol.insert(lt, seen);
                }
            }
        }
        self.cur = None;
        let any = !res.is_empty();
        sol.insert(t, Rc::new(res));
        any
    }

    // ---------------------------------------------------------------- actions

    fn sys_act(&mut self, key: &str, p: &[Json], sol: &mut Sol) {
        match key {
            "gset" => {
                let n = ps(p, 0);
                let v = self.xp(p, 1);
                let old = self.get_var(&n);
                let nv = match old {
                    Some(o) => Val::typed_like(&o, v),
                    None => Val::N(v.num0()),
                };
                self.set_var(&n, nv);
            }
            "gadd" => {
                let n = ps(p, 0);
                let o = self.get_var(&n);
                let v = self.xp(p, 1);
                let nv = match o {
                    Some(Val::S(s)) => Val::S(s).add(&v),
                    Some(o) => Val::N(o.num0() + v.num0()),
                    None => Val::N(v.num0()),
                };
                self.set_var(&n, nv);
            }
            "gsub" => {
                let n = ps(p, 0);
                let o = self.get_var(&n).map(|v| v.num0()).unwrap_or(0.0);
                let v = self.xpn(p, 1);
                self.set_var(&n, Val::N(o - v));
            }
            "call" => {
                let a = ps(p, 1);
                let args = if a.trim().is_empty() { Vec::new() } else { self.eval_args(&a) };
                self.run_func(&ps(p, 0), args);
            }
            "retval" => {
                let v = self.xp(p, 0);
                for s in self.scopes.iter().rev() {
                    if s.borrow().fn_ {
                        s.borrow_mut().ret = v;
                        return;
                    }
                }
            }
            "setgroup" => {
                let mode = ps(p, 1);
                for g in self.groups_by_name.get(&ps(p, 0)).cloned().unwrap_or_default() {
                    let v = if mode == "toggle" { !self.group_on(&g) } else { mode == "on" };
                    self.group_state.insert(g.id.clone(), v);
                }
            }
            "resetg" => self.reset_g(),
            "gotolayout" => {
                if pb(p, 0) {
                    self.pending = Some(ps(p, 0));
                }
            }
            "controlsmenu" => self.ctl_open(),
            "pause" => {
                let m = ps(p, 0);
                self.gpaused = if m == "pause" { true } else if m == "resume" { false } else { !self.gpaused };
            }
            "savegame" => {
                let mut g = pf_core::Map::new();
                for (k, v) in &self.globals {
                    g.insert(k.clone(), v.to_json());
                }
                let mut d = pf_core::Map::new();
                for (k, v) in &self.data {
                    d.insert(self.types[*k].id.clone(), match v {
                        DataStore::List(_) => pf_core::json!({"list": v.to_json()}),
                        DataStore::Map(_) => pf_core::json!({"map": v.to_json()}),
                    });
                }
                let cp = self.cp.as_ref().map(|c| pf_core::json!({"layout": c.layout, "x": c.x, "y": c.y})).unwrap_or(Json::Null);
                let sv = pf_core::json!({"layout": self.layout_id, "globals": g, "data": d, "cp": cp, "at": self.now_ms});
                self.sset(&format!("save:{}", ps_or(p, 0, "partie")), sv);
            }
            "loadgame" => {
                let sv = match self.sget(&format!("save:{}", ps_or(p, 0, "partie"))) {
                    Some(v) if jtruthy(Some(&v)) => v,
                    _ => return,
                };
                let lay = sv.s("layout").to_string();
                if !self.p.arr("layouts").iter().any(|l| l.s("id") == lay) {
                    return;
                }
                if let Some(g) = sv.get("globals").and_then(|g| g.as_object()) {
                    for (k, v) in g {
                        if self.global(k).is_some() {
                            self.set_global(k, Val::from_json(v));
                        }
                    }
                }
                if let Some(d) = sv.get("data").and_then(|g| g.as_object()) {
                    for (k, v) in d {
                        if let Some(&t) = self.type_by_id.get(k) {
                            if self.data.contains_key(&t) {
                                let ds = if let Some(l) = v.get("list").and_then(|l| l.as_array()) {
                                    DataStore::List(l.iter().map(Val::from_json).collect())
                                } else if let Some(m) = v.get("map").and_then(|m| m.as_object()) {
                                    DataStore::Map(m.iter().map(|(k, v)| (k.clone(), Val::from_json(v))).collect())
                                } else {
                                    continue;
                                };
                                self.data.insert(t, ds);
                            }
                        }
                    }
                }
                self.cp_load = Some(sv.get("cp").filter(|c| c.is_object()).map(|c| Checkpoint { layout: c.s("layout").to_string(), x: c.f("x", 0.0), y: c.f("y", 0.0) }));
                self.pending = Some(lay);
            }
            "hsadd" => {
                let tb = ps_or(p, 0, "scores");
                let mut l = self.hs_list(&tb);
                let sc = self.xpn(p, 1);
                let nm0 = self.xp(p, 2).str();
                let nm = if nm0.trim().is_empty() { "???".to_string() } else { nm0.trim().to_string() };
                let mx = { let v = self.xpn(p, 3).floor(); if v == 0.0 { 10.0 } else { v } }.max(1.0) as usize;
                let asc = ps(p, 4) == "asc";
                l.truncate(mx);
                if !Runtime::hs_fits(&l, sc, mx, asc) {
                    self.hs_rank = 0.0;
                    return;
                }
                let e = HsEntry { n: nm.chars().take(24).collect(), s: sc, d: self.now_ms + self.tick as f64 };
                let mut k = 0;
                while k < l.len() && (if asc { l[k].s <= sc } else { l[k].s >= sc }) {
                    k += 1;
                }
                let d = e.d;
                l.insert(k, e);
                l.truncate(mx);
                self.hs_rank = (k + 1) as f64;
                self.hs_last = Some((tb.clone(), d));
                let js = Json::Array(l.iter().map(|e| pf_core::json!({"n": e.n, "s": e.s, "d": e.d})).collect());
                self.sset(&format!("hs:{}", tb), js);
                self.hs_refresh();
            }
            "hsask" => {
                let mx = { let v = self.xpn(p, 2).floor(); if v == 0.0 { 10.0 } else { v } }.clamp(1.0, 24.0) as usize;
                let nm: String = self.xp(p, 1).str().chars().take(mx).collect();
                let title = self.xp(p, 0).str();
                self.hs_in = Some(HsInput { title, name: nm.chars().collect(), def: nm.clone(), max: mx, cur: nm.chars().count(), t: 0.0, hit: None });
            }
            "hsclear" => {
                self.sdel(&format!("hs:{}", ps_or(p, 0, "scores")));
                self.hs_refresh();
            }
            "cineplay" => self.cine_play(&ps(p, 0)),
            "cinestop" => self.cine_skip(),
            "deletesave" => self.sdel(&format!("save:{}", ps_or(p, 0, "partie"))),
            "camzoom" => {
                let z = (self.xpn(p, 0) / 100.0).clamp(0.1, 10.0);
                let d = self.xpn(p, 1).max(0.0);
                if d <= 0.0 {
                    self.zoom = z;
                    self.zoom_a = None;
                } else {
                    self.zoom_a = Some(ZoomAnim { from: self.zoom, to: z, t: 0.0, d });
                }
            }
            "camlook" => {
                let x = self.xpn(p, 0);
                let y = self.xpn(p, 1);
                self.cam_look = Some((x, y));
            }
            "camfollow" => self.cam_look = None,
            "cambounds" => {
                let v = [self.xpn(p, 0), self.xpn(p, 1), self.xpn(p, 2), self.xpn(p, 3)];
                self.cam_b = Some(v);
            }
            "camnobounds" => self.cam_b = None,
            "setcp" => {
                let x = self.xpn(p, 0);
                let y = self.xpn(p, 1);
                self.cp = Some(Checkpoint { layout: self.layout_id.clone(), x, y });
            }
            "clearcp" => self.cp = None,
            "respawn" => {
                let t = match self.tid(p, 0) {
                    Some(t) => t,
                    None => return,
                };
                let cp = match &self.cp {
                    Some(c) if c.layout == self.layout_id => c.clone(),
                    _ => return,
                };
                if let Some(&s) = self.alive_insts(t).first() {
                    let i = self.inst_mut(s);
                    i.x = cp.x;
                    i.y = cp.y;
                    if let Some(pl) = i.b.platform.as_mut() {
                        pl.vx = 0.0;
                        pl.vy = 0.0;
                        pl.floor = None;
                        pl.hold = false;
                    }
                    if let Some(e) = i.b.eightdir.as_mut() {
                        e.vx = 0.0;
                        e.vy = 0.0;
                    }
                    if let Some(sn) = i.b.sine.as_mut() {
                        sn.bx = cp.x;
                        sn.by = cp.y;
                    }
                }
                self.update_cam(0.0);
            }
            "transition" => {
                let d = self.xpn(p, 1).max(0.0);
                self.trans = Trans { ty: ps_or(p, 0, "fade"), dur: d, color: ps_or(p, 2, "#000000") };
            }
            "darkness" => {
                let a = (self.xpn(p, 0) / 100.0).clamp(0.0, 1.0);
                self.dark = (a, ps_or(p, 1, "#0b0b1e"));
            }
            "timescale" => self.tscale = (self.xpn(p, 0) / 100.0).clamp(0.0, 4.0),
            "screenflash" => {
                let d = self.xpn(p, 1).max(0.01);
                self.sflash = Some((ps_or(p, 0, "#ffffff"), d, d));
            }
            "say" => {
                let name = self.xp(p, 0).str();
                let text = self.xp(p, 1).str();
                self.dlg_push(Dialog { name, text, portrait: ps(p, 2), top: ps(p, 3) == "top", choices: None, shown: 0, acc: 0.0, sel: 0, t: 0.0, hit: None });
            }
            "ask" => {
                let ch: Vec<String> = self.xp(p, 2).str().split('|').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                let name = self.xp(p, 0).str();
                let text = self.xp(p, 1).str();
                self.dlg_push(Dialog { name, text, portrait: ps(p, 3), top: ps(p, 4) == "top", choices: Some(if ch.is_empty() { vec!["OK".into()] } else { ch }), shown: 0, acc: 0.0, sel: 0, t: 0.0, hit: None });
            }
            "dlgclose" => {
                self.dlg.queue.clear();
                if self.dlg.cur.is_some() {
                    self.dlg.cur = None;
                    self.dlg.end_tick = self.tick;
                }
            }
            "dlgspeed" => self.dlg.speed = self.xpn(p, 0).max(0.0),
            "next" => {
                let ls = self.p.arr("layouts");
                let k = ls.iter().position(|l| l.s("id") == self.layout_id).map(|k| k as i64).unwrap_or(-1);
                if !ls.is_empty() {
                    self.pending = Some(ls[((k + 1) as usize) % ls.len()].s("id").to_string());
                }
            }
            "restart" => self.pending = Some(self.layout_id.clone()),
            "create" => {
                let t = match self.tid(p, 0) {
                    Some(t) => t,
                    None => return,
                };
                let li = if pb(p, 3) { self.layer_idx(&ps(p, 3)) } else { None };
                let x = self.xpn(p, 1);
                let y = self.xpn(p, 2);
                if let Some(i) = self.create_inst(t, None, x, y, Some(li.unwrap_or(0))) {
                    sol.insert(t, Rc::new(vec![i]));
                }
            }
            "layervis" => {
                if let Some(k) = self.layer_idx(&ps(p, 0)) {
                    self.layers[k].visible = pb(p, 1);
                }
            }
            "layeropa" => {
                if let Some(k) = self.layer_idx(&ps(p, 0)) {
                    self.layers[k].opacity = (self.xpn(p, 1) / 100.0).clamp(0.0, 1.0);
                }
            }
            "shake" => {
                self.shake_a = self.xpn(p, 0);
                let d = self.xpn(p, 1).max(0.01);
                self.shake_d = d;
                self.shake_t = d;
            }
            _ => {}
        }
    }

    /// évalue « a, b, c » comme la liste d'arguments __a(...)
    fn eval_args(&mut self, src: &str) -> Vec<Val> {
        let e = self.cache.get(&format!("__a({})", src));
        match &*e {
            Ok(pf_core::expr::Expr::Call(_, args)) => {
                let args = args.clone();
                args.iter().map(|a| pf_core::expr::eval(a, self)).collect()
            }
            Ok(_) => Vec::new(),
            Err(m) => {
                if self.errors.len() < 20 && !self.errors.contains(m) {
                    self.errors.push(m.clone());
                }
                Vec::new()
            }
        }
    }

    fn data_act(&mut self, t: usize, key: &str, p: &[Json]) {
        let kind = self.types[t].def.kind;
        match (kind, key) {
            (Kind::Array, "push") => {
                let v = dval(self.xp(p, 0));
                if let Some(DataStore::List(l)) = self.data.get_mut(&t) {
                    if ps(p, 1) == "start" {
                        l.insert(0, v);
                    } else {
                        l.push(v);
                    }
                }
            }
            (Kind::Array, "set") => {
                let k = self.xpn(p, 0).floor();
                let v = dval(self.xp(p, 1));
                if let Some(DataStore::List(l)) = self.data.get_mut(&t) {
                    if k >= 0.0 && k < 100000.0 {
                        let k = k as usize;
                        while l.len() < k {
                            l.push(Val::N(0.0));
                        }
                        if k < l.len() {
                            l[k] = v;
                        } else {
                            l.push(v);
                        }
                    }
                }
            }
            (Kind::Array, "insert") => {
                let k = self.xpn(p, 0).floor();
                let v = dval(self.xp(p, 1));
                if let Some(DataStore::List(l)) = self.data.get_mut(&t) {
                    let k = k.clamp(0.0, l.len() as f64) as usize;
                    l.insert(k, v);
                }
            }
            (Kind::Array, "removeat") => {
                let k = self.xpn(p, 0).floor();
                if let Some(DataStore::List(l)) = self.data.get_mut(&t) {
                    if k >= 0.0 && (k as usize) < l.len() {
                        l.remove(k as usize);
                    }
                }
            }
            (Kind::Array, "removeval") => {
                let v = dval(self.xp(p, 0));
                if let Some(DataStore::List(l)) = self.data.get_mut(&t) {
                    if let Some(k) = l.iter().position(|x| x.loose_eq(&v)) {
                        l.remove(k);
                    }
                }
            }
            (Kind::Array, "clear") => {
                self.data.insert(t, DataStore::List(Vec::new()));
            }
            (Kind::Array, "sort") => {
                let desc = ps(p, 0) == "desc";
                if let Some(DataStore::List(l)) = self.data.get_mut(&t) {
                    l.sort_by(|a, b| {
                        let r = match (a, b) {
                            (Val::N(x), Val::N(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            _ => a.str().to_lowercase().cmp(&b.str().to_lowercase()).then_with(|| a.str().cmp(&b.str())),
                        };
                        if desc {
                            r.reverse()
                        } else {
                            r
                        }
                    });
                }
            }
            (Kind::Array, "shuffle") => {
                let n = match self.data.get(&t) {
                    Some(DataStore::List(l)) => l.len(),
                    _ => 0,
                };
                let mut sw = Vec::new();
                for k in (1..n).rev() {
                    let j = (self.rand() * (k + 1) as f64) as usize;
                    sw.push((k, j.min(k)));
                }
                if let Some(DataStore::List(l)) = self.data.get_mut(&t) {
                    for (k, j) in sw {
                        l.swap(k, j);
                    }
                }
            }
            (Kind::Array, "save") => {
                let v = self.data.get(&t).map(|d| d.to_json()).unwrap_or(Json::Null);
                self.sset(&format!("arr:{}", ps(p, 0)), v);
            }
            (Kind::Array, "load") => {
                if let Some(Json::Array(a)) = self.sget(&format!("arr:{}", ps(p, 0))) {
                    self.data.insert(t, DataStore::List(a.iter().map(Val::from_json).collect()));
                }
            }
            (Kind::Dict, "set") => {
                let k = self.xp(p, 0).str();
                let v = dval(self.xp(p, 1));
                if let Some(d) = self.data.get_mut(&t) {
                    d.map_set(k, v);
                }
            }
            (Kind::Dict, "add") => {
                let k = self.xp(p, 0).str();
                let o = self.data.get(&t).and_then(|d| d.map_get(&k)).cloned();
                let v = self.xp(p, 1);
                let nv = match o {
                    Some(Val::S(s)) => Val::S(s).add(&v),
                    Some(o) => Val::N(o.num0() + v.num0()),
                    None => Val::N(v.num0()),
                };
                if let Some(d) = self.data.get_mut(&t) {
                    d.map_set(k, nv);
                }
            }
            (Kind::Dict, "remove") => {
                let k = self.xp(p, 0).str();
                if let Some(DataStore::Map(m)) = self.data.get_mut(&t) {
                    m.retain(|e| e.0 != k);
                }
            }
            (Kind::Dict, "clear") => {
                self.data.insert(t, DataStore::Map(Vec::new()));
            }
            (Kind::Dict, "save") => {
                let v = self.data.get(&t).map(|d| d.to_json()).unwrap_or(Json::Null);
                self.sset(&format!("dict:{}", ps(p, 0)), v);
            }
            (Kind::Dict, "load") => {
                if let Some(Json::Object(o)) = self.sget(&format!("dict:{}", ps(p, 0))) {
                    self.data.insert(t, DataStore::Map(o.iter().map(|(k, v)| (k.clone(), Val::from_json(v))).collect()));
                }
            }
            _ => {}
        }
    }

    fn obj_act(&mut self, prov: &str, key: &str, s: Slot, p: &[Json], sol: &mut Sol) {
        let tick = self.tick;
        match (prov, key) {
            ("common", "destroy") => self.inst_mut(s).dead = true,
            ("common", "spawn") => {
                if let Some(t) = self.tid(p, 0) {
                    let (x, y, l) = {
                        let i = self.inst(s);
                        (i.x, i.y, i.layer)
                    };
                    if let Some(n) = self.create_inst(t, None, x, y, Some(l)) {
                        sol.insert(t, Rc::new(vec![n]));
                    }
                }
            }
            ("common", "tolayer") => {
                if let Some(k) = self.layer_idx(&ps(p, 0)) {
                    self.inst_mut(s).layer = k;
                }
            }
            ("common", "posto") => {
                let t = match self.tid(p, 0) {
                    Some(t) => t,
                    None => return,
                };
                let j = sol.get(&t).and_then(|v| v.iter().copied().find(|&x| self.alive(x))).or_else(|| self.alive_insts(t).first().copied());
                if let Some(j) = j {
                    let nm = Val::S(ps(p, 1).trim().to_string());
                    let q = self.img_point(j, Some(&nm));
                    let i = self.inst_mut(s);
                    i.x = q.0;
                    i.y = q.1;
                }
            }
            ("common", "setpos") => {
                let x = self.xpn(p, 0);
                let y = self.xpn(p, 1);
                let i = self.inst_mut(s);
                i.x = x;
                i.y = y;
            }
            ("common", "setx") => {
                let v = self.xpn(p, 0);
                self.inst_mut(s).x = v;
            }
            ("common", "sety") => {
                let v = self.xpn(p, 0);
                self.inst_mut(s).y = v;
            }
            ("common", "move") => {
                let x = self.xpn(p, 0);
                let y = self.xpn(p, 1);
                let i = self.inst_mut(s);
                i.x += x;
                i.y += y;
            }
            ("common", "setsize") => {
                let w = self.xpn(p, 0);
                let h = self.xpn(p, 1);
                let i = self.inst_mut(s);
                i.w = w;
                i.h = h;
            }
            ("common", "setangle") => {
                let v = self.xpn(p, 0);
                self.inst_mut(s).angle = v;
            }
            ("common", "setstyle") => {
                let k = ps(p, 0);
                let v = if jtruthy(p.get(1)) { jstr(&p[1]) } else { String::new() };
                let i = self.inst_mut(s);
                let mut st = i.style.clone().unwrap_or_default();
                st.insert(k, Json::String(v));
                i.style = Some(st);
            }
            ("common", "setvis") => self.inst_mut(s).visible = pb(p, 0),
            ("common", "setopa") => {
                let v = (self.xpn(p, 0) / 100.0).clamp(0.0, 1.0);
                self.inst_mut(s).opacity = v;
            }
            ("common", "flash") => {
                let v = self.xpn(p, 0);
                self.inst_mut(s).flash = v;
            }
            ("common", "front") => {
                if let Some(k) = self.all.iter().position(|&x| x == s) {
                    self.all.remove(k);
                    self.all.push(s);
                }
            }
            ("common", "ivset") => {
                let n = ps(p, 0);
                let v = self.xp(p, 1);
                let i = self.inst_mut(s);
                let nv = match i.vars.get(&n) {
                    Some(o) => Val::typed_like(o, v),
                    None => Val::N(v.num0()),
                };
                i.vars.insert(n, nv);
            }
            ("common", "tint") => {
                let a = (self.xpn(p, 1) / 100.0).clamp(0.0, 1.0);
                let c = crate::gfx::parse_color(&ps_or(p, 0, "#ffffff"));
                self.inst_mut(s).tint = if a > 0.0 { Some((c, a)) } else { None };
            }
            ("common", "tintflash") => {
                let d = self.xpn(p, 1).max(0.01);
                let c = crate::gfx::parse_color(&ps_or(p, 0, "#ffffff"));
                self.inst_mut(s).tf = Some((c, d, d));
            }
            ("common", "ivadd") => {
                let n = ps(p, 0);
                let v = self.xp(p, 1);
                let i = self.inst_mut(s);
                let nv = match i.vars.get(&n) {
                    Some(Val::S(o)) => Val::S(o.clone()).add(&v),
                    Some(o) => Val::N(o.num0() + v.num0()),
                    None => Val::N(v.num0()),
                };
                i.vars.insert(n, nv);
            }
            ("common", "ivsub") => {
                let n = ps(p, 0);
                let v = self.xpn(p, 1);
                let i = self.inst_mut(s);
                let o = i.vars.get(&n).map(|v| v.num0()).unwrap_or(0.0);
                i.vars.insert(n, Val::N(o - v));
            }
            ("sprite", "setanim") => {
                let d = self.def(s);
                let n = ps(p, 0);
                if let Some(k) = d.anims.iter().position(|a| a.name == n) {
                    let i = self.inst_mut(s);
                    if k != i.anim {
                        i.anim = k;
                        i.frame = 0;
                        i.ft = 0.0;
                    }
                }
            }
            ("sprite", "mirror") => self.inst_mut(s).mirrored = pb(p, 0),
            ("sprite", "spawnpt") => {
                let t = match self.tid(p, 0) {
                    Some(t) => t,
                    None => return,
                };
                let nm = p.get(1).map(Val::from_json);
                let q = self.img_point(s, nm.as_ref());
                let (mir, ang, lay) = {
                    let i = self.inst(s);
                    (i.mirrored, i.angle, i.layer)
                };
                if let Some(n) = self.create_inst(t, None, q.0, q.1, Some(lay)) {
                    if ps(p, 2) != "none" {
                        let ni = self.inst_mut(n);
                        ni.mirrored = mir;
                        ni.angle = ang;
                        if let Some(u) = ni.b.bullet.as_mut() {
                            let mut a = u.angle + ang;
                            if mir {
                                a = 180.0 - a;
                            }
                            let a = a * std::f64::consts::PI / 180.0;
                            u.vx = a.cos() * u.speed;
                            u.vy = a.sin() * u.speed;
                            ni.angle = if mir { 0.0 } else { ang };
                        }
                    }
                    sol.insert(t, Rc::new(vec![n]));
                }
            }
            ("text", "settext") => {
                let v = self.xp(p, 0).str();
                self.inst_mut(s).text = Val::S(v);
            }
            ("text", "append") => {
                let v = self.xp(p, 0);
                let i = self.inst_mut(s);
                i.text = Val::S(i.text.str()).add(&v);
            }
            ("b:platform", k) => {
                let v = if matches!(k, "setvy" | "setvx" | "setmax" | "setjump") { self.xpn(p, 0) } else { 0.0 };
                let st = self.inst_mut(s).b.platform.as_mut().unwrap();
                match k {
                    "simjump" => st.sim.jump = true,
                    "simleft" => st.sim.left = true,
                    "simright" => st.sim.right = true,
                    "setvy" => {
                        st.vy = v;
                        if st.vy < 0.0 {
                            st.floor = None;
                            st.hold = true;
                        }
                    }
                    "setvx" => st.vx = v,
                    "setmax" => st.max_speed = v,
                    "setjump" => st.jump = v,
                    "enable" => st.en = pb(p, 0),
                    "controls" => st.controls = pb(p, 0),
                    _ => {}
                }
            }
            ("b:patrol", k) => {
                let v = if k == "speed" { self.xpn(p, 0) } else { 0.0 };
                let name = self.tid(p, 0).map(|t| self.types[t].name.clone()).unwrap_or_default();
                let st = self.inst_mut(s).b.patrol.as_mut().unwrap();
                match k {
                    "setchase" => st.chase = name,
                    "reverse" => st.dir *= -1.0,
                    "speed" => st.speed = v,
                    _ => {}
                }
            }
            ("b:bullet", "speed") => {
                let v = self.xpn(p, 0);
                let u = self.inst_mut(s).b.bullet.as_mut().unwrap();
                let a = u.vy.atan2(u.vx);
                u.speed = v;
                u.vx = a.cos() * v;
                u.vy = a.sin() * v;
            }
            ("b:bullet", "angle") => {
                let a = self.xpn(p, 0) * std::f64::consts::PI / 180.0;
                let u = self.inst_mut(s).b.bullet.as_mut().unwrap();
                u.vx = a.cos() * u.speed;
                u.vy = a.sin() * u.speed;
            }
            ("b:bullet", "bounceoff") => {
                let t = match self.tid(p, 0) {
                    Some(t) => t,
                    None => return,
                };
                let a = self.box_i(s);
                let mut best: Option<(BBox, f64, f64)> = None;
                let mut bo = 0.0;
                for o in self.picked(sol, t) {
                    if o == s || !self.alive(o) {
                        continue;
                    }
                    let b = self.box_i(o);
                    let ox = a.r.min(b.r) - a.l.max(b.l);
                    let oy = a.b.min(b.b) - a.t.max(b.t);
                    if ox > 0.0 && oy > 0.0 && ox * oy > bo {
                        bo = ox * oy;
                        best = Some((b, ox, oy));
                    }
                }
                if let Some((b, ox, oy)) = best {
                    let acx = (a.l + a.r) / 2.0;
                    let acy = (a.t + a.b) / 2.0;
                    let i = self.inst_mut(s);
                    if ox < oy {
                        let dx = if acx < (b.l + b.r) / 2.0 { -ox } else { ox };
                        i.x += dx;
                        let u = i.b.bullet.as_mut().unwrap();
                        u.vx = if dx < 0.0 { -u.vx.abs() } else { u.vx.abs() };
                    } else {
                        let dy = if acy < (b.t + b.b) / 2.0 { -oy } else { oy };
                        i.y += dy;
                        let u = i.b.bullet.as_mut().unwrap();
                        u.vy = if dy < 0.0 { -u.vy.abs() } else { u.vy.abs() };
                    }
                }
            }
            ("b:sine", "active") => self.inst_mut(s).b.sine.as_mut().unwrap().en = pb(p, 0),
            ("b:physics", k) => match k {
                "impulse" => {
                    let a = self.xpn(p, 0) * std::f64::consts::PI / 180.0;
                    let f = self.xpn(p, 1);
                    let b = self.inst_mut(s).b.physics.as_mut().unwrap();
                    b.vx += a.cos() * f;
                    b.vy += a.sin() * f;
                }
                "setvel" => {
                    let x = self.xpn(p, 0);
                    let y = self.xpn(p, 1);
                    let b = self.inst_mut(s).b.physics.as_mut().unwrap();
                    b.vx = x;
                    b.vy = y;
                }
                "immovable" => {
                    let b = self.inst_mut(s).b.physics.as_mut().unwrap();
                    b.fixed = pb(p, 0);
                    b.vx = 0.0;
                    b.vy = 0.0;
                }
                "ropeto" => {
                    let t = match self.tid(p, 0) {
                        Some(t) => t,
                        None => return,
                    };
                    let o = self.picked(sol, t).into_iter().find(|&x| x != s);
                    if let Some(o) = o {
                        let l = self.xpn(p, 1);
                        let ca = self.center(s);
                        let cb = self.center(o);
                        self.joints.push(Joint { a: s, b: Some(o), px: 0.0, py: 0.0, len: if l > 0.0 { l } else { (ca.0 - cb.0).hypot(ca.1 - cb.1) }, rigid: pb(p, 2) });
                    }
                }
                "ropepoint" => {
                    let x = self.xpn(p, 0);
                    let y = self.xpn(p, 1);
                    let l = self.xpn(p, 2);
                    let ca = self.center(s);
                    self.joints.push(Joint { a: s, b: None, px: x, py: y, len: if l > 0.0 { l } else { (ca.0 - x).hypot(ca.1 - y) }, rigid: false });
                }
                "cutropes" => self.joints.retain(|j| j.a != s && j.b != Some(s)),
                _ => {}
            },
            ("b:path", k) => {
                let v = if k == "speed" { self.xpn(p, 0) } else { 0.0 };
                let i = self.inst_mut(s);
                let st = i.b.path.as_mut().unwrap();
                match k {
                    "enable" => st.en = pb(p, 0),
                    "speed" => st.speed = v,
                    "restart" => {
                        st.k = 0;
                        st.nx = 1;
                        st.dir = 1;
                        st.u = 0.0;
                        st.wt = 0.0;
                        st.en = true;
                        let (bx, by) = (st.bx, st.by);
                        i.x = bx;
                        i.y = by;
                    }
                    _ => {}
                }
            }
            ("tilemap", "settile") => {
                let (c, r, v) = (self.xpn(p, 0), self.xpn(p, 1), self.xpn(p, 2));
                self.tm_set(s, c, r, v);
            }
            ("tilemap", "settileat") => {
                let (x, y, v) = (self.xpn(p, 0), self.xpn(p, 1), self.xpn(p, 2));
                let (c, r) = (self.tm_col(s, x), self.tm_row(s, y));
                self.tm_set(s, c, r, v);
            }
            ("tilemap", "fillrect") => {
                let mut c0 = self.xpn(p, 0).floor();
                let mut r0 = self.xpn(p, 1).floor();
                let mut c1 = self.xpn(p, 2).floor();
                let mut r1 = self.xpn(p, 3).floor();
                let v = self.xpn(p, 4);
                if c1 < c0 {
                    std::mem::swap(&mut c0, &mut c1);
                }
                if r1 < r0 {
                    std::mem::swap(&mut r0, &mut r1);
                }
                let mut r = r0;
                while r <= r1 {
                    let mut c = c0;
                    while c <= c1 {
                        self.tm_set(s, c, r, v);
                        c += 1.0;
                    }
                    r += 1.0;
                }
            }
            ("minimap", "setradius") => {
                let v = self.xpn(p, 0).max(50.0);
                self.inst_mut(s).mm_r = Some(v);
            }
            ("road3d", k) => self.rd_act(s, k, p),
            ("bar", k) => {
                let v = if k != "setcolor" { self.xpn(p, 0) } else { 0.0 };
                match k {
                    "setvalue" => self.bar_set(s, v),
                    "addvalue" => {
                        let c = self.inst(s).bar.as_ref().map(|b| b.v).unwrap_or(0.0);
                        self.bar_set(s, c + v);
                    }
                    "setmax" => {
                        let c = {
                            let b = self.inst_mut(s).bar.as_mut().unwrap();
                            b.max = v.max(0.0001);
                            b.v
                        };
                        self.bar_set(s, c);
                    }
                    "setcolor" => {
                        let c = if jtruthy(p.first()) { jstr(&p[0]) } else { String::new() };
                        self.inst_mut(s).bar.as_mut().unwrap().col = c;
                    }
                    _ => {}
                }
            }
            ("leaderboard", "settable") => {
                let t = self.xp(p, 0).str();
                let l = self.hs_list(&t);
                let i = self.inst_mut(s);
                i.lbt = t;
                i.lb = l;
            }
            ("button", k) => match k {
                "settext" => {
                    let v = self.xp(p, 0).str();
                    self.inst_mut(s).text = Val::S(v);
                }
                "enable" => {
                    let en = pb(p, 0);
                    self.inst_mut(s).btn.as_mut().unwrap().en = en;
                    if !en && self.btn_focus == Some(s) {
                        self.inst_mut(s).btn.as_mut().unwrap().focus = false;
                        self.btn_focus = None;
                    }
                }
                "focus" => {
                    if let Some(f) = self.btn_focus {
                        if let Some(b) = self.insts[f as usize].as_mut().and_then(|i| i.btn.as_mut()) {
                            b.focus = false;
                        }
                    }
                    self.btn_focus = Some(s);
                    self.inst_mut(s).btn.as_mut().unwrap().focus = true;
                }
                _ => {}
            },
            ("particles", k) => match k {
                "burst" => {
                    let n = self.xpn(p, 0).floor().clamp(0.0, 500.0) as usize;
                    self.emit_parts(s, n);
                }
                "active" => self.inst_mut(s).ps.as_mut().unwrap().active = pb(p, 0),
                "setrate" => {
                    let v = self.xpn(p, 0).max(0.0);
                    self.inst_mut(s).ps.as_mut().unwrap().rate = Some(v);
                }
                "clear" => self.inst_mut(s).ps.as_mut().unwrap().parts.clear(),
                _ => {}
            },
            ("b:car", k) => {
                let v = if k == "setspeed" || k == "setmax" { self.xpn(p, 0) } else { 0.0 };
                let c = self.inst_mut(s).b.car.as_mut().unwrap();
                match k {
                    "sim" => c.sim.set(&ps(p, 0)),
                    "setspeed" => c.sp = v,
                    "setmax" => c.max_speed = v,
                    "stop" => {
                        c.sp = 0.0;
                        c.vx = 0.0;
                        c.vy = 0.0;
                    }
                    "controls" => c.ctl = pb(p, 0),
                    "enable" => c.en = pb(p, 0),
                    _ => {}
                }
            }
            ("b:eightdir", k) => {
                let v = if k == "setmax" { self.xpn(p, 0) } else { 0.0 };
                let c = self.inst_mut(s).b.eightdir.as_mut().unwrap();
                match k {
                    "sim" => c.sim.set(&ps(p, 0)),
                    "setmax" => c.max_speed = v,
                    "stop" => {
                        c.vx = 0.0;
                        c.vy = 0.0;
                    }
                    "enable" => c.en = pb(p, 0),
                    _ => {}
                }
            }
            ("b:tween", "start") => {
                let pr = ps(p, 0);
                let to = self.xpn(p, 1);
                let d = self.xpn(p, 2).max(0.0);
                let i = self.inst_mut(s);
                let from = Runtime::tprop(i, &pr);
                let st = i.b.tween.as_mut().unwrap();
                st.list.retain(|t| t.prop != pr);
                if d <= 0.0 {
                    Runtime::tset(i, &pr, to);
                    i.b.tween.as_mut().unwrap().done.insert(ps(p, 5), tick + 1);
                    return;
                }
                st.list.push(Tween { prop: pr, from, to, d, t: 0.0, ease: ps_or(p, 3, "inout"), mode: ps_or(p, 4, "once"), tag: ps_or(p, 5, "tween") });
            }
            ("b:tween", "stop") => {
                let tag = ps(p, 0);
                self.inst_mut(s).b.tween.as_mut().unwrap().list.retain(|t| t.tag != tag);
            }
            ("b:tween", "stopall") => self.inst_mut(s).b.tween.as_mut().unwrap().list.clear(),
            ("b:fade", "start") => {
                let i = self.inst_mut(s);
                let f = i.b.fade.as_mut().unwrap();
                f.t = 0.0;
                f.run = true;
                if f.base == 0.0 {
                    f.base = 1.0;
                }
                let (fi, base) = (f.fade_in, f.base);
                i.opacity = if fi > 0.0 { 0.0 } else { base };
            }
            ("b:fade", "out") => {
                let d = self.xpn(p, 0).max(0.01);
                let i = self.inst_mut(s);
                let op = i.opacity;
                let f = i.b.fade.as_mut().unwrap();
                f.base = op;
                f.fade_in = 0.0;
                f.wait = 0.0;
                f.fade_out = d;
                f.t = 0.0;
                f.run = true;
            }
            ("b:pin", "pinto") => {
                let t = match self.tid(p, 0) {
                    Some(t) => t,
                    None => return,
                };
                let j = sol.get(&t).and_then(|v| v.iter().copied().find(|&x| self.alive(x))).or_else(|| self.alive_insts(t).first().copied());
                let j = match j {
                    Some(j) if j != s => j,
                    _ => return,
                };
                let nm = ps(p, 1).trim().to_string();
                let (jx, jy, ja, jm) = {
                    let ji = self.inst(j);
                    (ji.x, ji.y, ji.angle, ji.mirrored)
                };
                let i = self.inst_mut(s);
                let (ix, iy, ia, im) = (i.x, i.y, i.angle, i.mirrored);
                let st = i.b.pin.as_mut().unwrap();
                st.to = Some(j);
                st.pt = nm;
                let mut ux = ix - jx;
                let mut uy = iy - jy;
                if st.angle && ja != 0.0 {
                    let r = -ja * std::f64::consts::PI / 180.0;
                    let (c, sn) = (r.cos(), r.sin());
                    let x2 = ux * c - uy * sn;
                    uy = ux * sn + uy * c;
                    ux = x2;
                }
                if st.mirror && jm {
                    ux = -ux;
                }
                st.ox = ux;
                st.oy = uy;
                st.oa = ia - ja;
                st.mi = im != jm;
            }
            ("b:pin", "unpin") => self.inst_mut(s).b.pin.as_mut().unwrap().to = None,
            ("b:drag", "drop") => {
                let d = self.inst_mut(s).b.drag.as_mut().unwrap();
                if d.on {
                    d.on = false;
                    d.dr = tick + 1;
                }
            }
            ("b:drag", "enable") => {
                let d = self.inst_mut(s).b.drag.as_mut().unwrap();
                d.en = pb(p, 0);
                if !d.en {
                    d.on = false;
                }
            }
            ("b:turret", k) => {
                let v = if k == "setrange" { self.xpn(p, 0) } else { 0.0 };
                let tt = self.tid(p, 0);
                let st = self.inst_mut(s).b.turret.as_mut().unwrap();
                match k {
                    "addtarget" => {
                        if let Some(tt) = tt {
                            if !st.types.contains(&tt) {
                                st.types.push(tt);
                            }
                        }
                    }
                    "cleartargets" => {
                        st.types.clear();
                        st.target = None;
                    }
                    "setrange" => st.range = v,
                    _ => {}
                }
            }
            ("b:los", "setrange") => {
                let v = self.xpn(p, 0);
                self.inst_mut(s).b.los.as_mut().unwrap().range = v;
            }
            ("b:timer", k) => {
                let n = ps(p, 0);
                let d = if k == "start" { self.xpn(p, 1).max(0.0) } else { 0.0 };
                let st = self.inst_mut(s).b.timer.as_mut().unwrap();
                match k {
                    "start" => {
                        let one = TimerOne { left: d, dur: d, reg: ps(p, 2) == "regular" };
                        if let Some(e) = st.timers.iter_mut().find(|e| e.0 == n) {
                            e.1 = one;
                        } else {
                            st.timers.push((n, one));
                        }
                    }
                    "stop" => st.timers.retain(|e| e.0 != n),
                    "stopall" => st.timers.clear(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn run_acts(&mut self, acts: &Rc<Vec<Act>>, sol: &mut Sol, from: usize, subs: &Rc<Vec<Rc<Ev>>>) -> bool {
        let mut n = from;
        while n < acts.len() {
            let a = &acts[n];
            n += 1;
            if a.off {
                continue;
            }
            self.cur_sol = sol.clone();
            self.cur = None;
            let key = a.key.as_str();
            let p = &a.p;
            if a.o == "system" && key == "wait" {
                let d = self.xpn(p, 0).max(0.0);
                self.waits.push(Wait { t: self.time + d, acts: acts.clone(), from: n, subs: subs.clone(), sol: sol.clone(), scopes: self.scopes.clone() });
                return true;
            }
            match a.o.as_str() {
                "system" => self.sys_act(key, p, sol),
                "audio" => match key {
                    "play" => self.sfx(&ps(p, 0)),
                    "playfile" => {
                        let v = self.xpn(p, 1) / 100.0;
                        let tag = ps(p, 3);
                        self.host.audio(AudioCmd::PlayFile { id: ps(p, 0), vol: v, looping: pb(p, 2), tag });
                    }
                    "music" => {
                        let v = self.xpn(p, 1) / 100.0;
                        let f = self.xpn(p, 2);
                        let id = ps(p, 0);
                        self.music = Some(id.clone());
                        self.host.audio(AudioCmd::Music { id, vol: v, fade: f });
                    }
                    "stopmusic" => {
                        let f = self.xpn(p, 0);
                        self.music = None;
                        self.host.audio(AudioCmd::StopMusic { fade: f });
                    }
                    "stoptag" => self.host.audio(AudioCmd::StopTag(ps(p, 0))),
                    "stopall" => {
                        self.music = None;
                        self.host.audio(AudioCmd::StopAll);
                    }
                    "volume" => {
                        self.mvol = (self.xpn(p, 0) / 100.0).max(0.0);
                        let v = self.mvol;
                        self.host.audio(AudioCmd::Volume(v));
                    }
                    _ => {}
                },
                "storage" => match key {
                    "savevar" => {
                        let v = self.global(&ps(p, 0)).map(|v| v.to_json()).unwrap_or(Json::Null);
                        self.sset(&ps(p, 0), v);
                    }
                    "loadvar" => {
                        let k = ps(p, 0);
                        if let Some(sv) = self.sget(&k) {
                            if let Some(old) = self.global(&k).cloned() {
                                let nv = if old.is_str() { Val::S(Val::from_json(&sv).str()) } else { Val::N(Val::from_json(&sv).num0()) };
                                self.set_global(&k, nv);
                            }
                        }
                    }
                    "save" => {
                        let v = self.xp(p, 1).to_json();
                        self.sset(&ps(p, 0), v);
                    }
                    "remove" => self.sdel(&ps(p, 0)),
                    "clear" => self.sclear(),
                    _ => {}
                },
                "mouse" => {}
                _ => {
                    if let Some(t) = a.ot {
                        let kind = self.types[t].def.kind;
                        if kind == Kind::Array || kind == Kind::Dict {
                            self.data_act(t, key, p);
                        } else {
                            let prov = a.prov.as_str();
                            let bname = prov.strip_prefix("b:");
                            let list = self.picked(sol, t);
                            for s in list {
                                if !self.alive(s) && !matches!(prov, "common") {
                                    // JS agit aussi sur les instances détruites dans la même action ; on garde ce comportement
                                }
                                if let Some(b) = bname {
                                    if !self.inst(s).b.has(b) {
                                        continue;
                                    }
                                }
                                self.cur = Some(s);
                                self.cur_sol = sol.clone();
                                self.obj_act(prov, key, s, p, sol);
                            }
                            self.cur = None;
                        }
                    }
                }
            }
            if self.pending.is_some() {
                return true;
            }
        }
        false
    }

    pub fn run_waits(&mut self) {
        if self.waits.is_empty() {
            return;
        }
        let time = self.time;
        let (due, keep): (Vec<Wait>, Vec<Wait>) = std::mem::take(&mut self.waits).into_iter().partition(|w| w.t <= time + 1e-9);
        self.waits = keep;
        for w in due {
            if self.pending.is_some() {
                break;
            }
            let mut sol = Sol::new();
            for (k, v) in &w.sol {
                sol.insert(*k, Rc::new(v.iter().copied().filter(|&s| self.alive(s)).collect()));
            }
            let saved = std::mem::replace(&mut self.scopes, w.scopes.clone());
            if !self.run_acts(&w.acts, &mut sol, w.from, &w.subs) && !w.subs.is_empty() {
                let subs = w.subs.clone();
                self.run_list(&subs, &sol);
            }
            self.scopes = saved;
        }
    }

    fn finish_event(&mut self, ev: &Rc<Ev>, sol: &mut Sol) -> bool {
        *self.ev_hits.entry(ev.id.clone()).or_insert(0) += 1;
        if self.bps.contains(&ev.id) && !self.paused {
            self.paused = true;
            self.brk_hit = Some((ev.id.clone(), self.tick));
        }
        let waited = self.run_acts(&ev.acts, sol, 0, &ev.subs);
        if !waited && !ev.subs.is_empty() && self.pending.is_none() {
            let subs = ev.subs.clone();
            self.run_list(&subs, sol);
        }
        true
    }

    fn run_or(&mut self, ev: &Rc<Ev>, sol: &mut Sol, prev_ok: bool) -> bool {
        let mut any = false;
        let mut union: HashMap<usize, Vec<Slot>> = HashMap::new();
        let mut full: HashSet<usize> = HashSet::new();
        for c in &ev.conds {
            if c.off {
                continue;
            }
            let mut s2 = sol.clone();
            if self.eval_cond(c, &mut s2, prev_ok) {
                any = true;
                for (k, v) in &s2 {
                    if sol.get(k).map(|x| Rc::ptr_eq(x, v)).unwrap_or(false) {
                        continue;
                    }
                    let u = union.entry(*k).or_default();
                    for &i in v.iter() {
                        if !u.contains(&i) {
                            u.push(i);
                        }
                    }
                }
                for t in 0..self.types.len() {
                    let same = match (s2.get(&t), sol.get(&t)) {
                        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                        (None, None) => true,
                        _ => false,
                    };
                    if same {
                        full.insert(t);
                    }
                }
            }
        }
        if !any {
            return false;
        }
        for (k, v) in union {
            if !full.contains(&k) {
                sol.insert(k, Rc::new(v));
            }
        }
        self.finish_event(ev, sol)
    }

    pub fn group_on(&self, g: &Ev) -> bool {
        match self.group_state.get(&g.id) {
            Some(v) => *v,
            None => g.active,
        }
    }

    pub fn run_func(&mut self, name: &str, args: Vec<Val>) -> Val {
        let f = match self.funcs.get(name) {
            Some(f) => f.clone(),
            None => return Val::N(0.0),
        };
        if self.call_depth > 64 {
            return Val::N(0.0);
        }
        let mut vars = HashMap::new();
        for (k, pn) in f.params.iter().enumerate() {
            vars.insert(pn.clone(), args.get(k).cloned().unwrap_or(Val::N(0.0)));
        }
        let sc = Rc::new(RefCell::new(Scope { vars, fn_: true, ret: Val::N(0.0), st: Vec::new() }));
        let s_cur = self.cur;
        let s_sol = self.cur_sol.clone();
        self.scopes.push(sc.clone());
        self.call_depth += 1;
        let mut sol = Sol::new();
        if !self.run_acts(&f.acts, &mut sol, 0, &f.subs) && !f.subs.is_empty() {
            let subs = f.subs.clone();
            self.run_list(&subs, &sol);
        }
        self.call_depth -= 1;
        if let Some(k) = self.scopes.iter().rposition(|x| Rc::ptr_eq(x, &sc)) {
            self.scopes.remove(k);
        }
        self.cur = s_cur;
        self.cur_sol = s_sol;
        let r = sc.borrow().ret.clone();
        r
    }

    fn run_event(&mut self, ev: &Rc<Ev>, sol: &mut Sol, from: usize, prev_ok: bool) -> bool {
        if ev.or && from == 0 {
            return self.run_or(ev, sol, prev_ok);
        }
        for n in from..ev.conds.len() {
            let c = &ev.conds[n];
            if !c.off && c.o == "system" && LOOPS.contains(&c.key.as_str()) {
                let c = c.clone();
                return self.run_loop(ev, sol, n, &c, prev_ok);
            }
            if !self.eval_cond(c, sol, prev_ok) {
                return false;
            }
        }
        self.finish_event(ev, sol)
    }

    fn run_loop(&mut self, ev: &Rc<Ev>, sol: &mut Sol, n: usize, c: &Cond, prev_ok: bool) -> bool {
        let p = &c.p;
        let mut any = false;
        self.loop_stack.push(LoopFr::default());
        self.cur_sol = sol.clone();
        self.cur = None;
        let top = self.loop_stack.len() - 1;
        match c.key.as_str() {
            "repeat" => {
                let nn = self.xpn(p, 0).floor().min(100000.0);
                let mut k = 0.0;
                while k < nn && self.pending.is_none() {
                    self.loop_stack[top].i = k;
                    let mut s2 = sol.clone();
                    if self.run_event(ev, &mut s2, n + 1, prev_ok) {
                        any = true;
                    }
                    k += 1.0;
                }
            }
            "forl" => {
                self.loop_stack[top].name = ps(p, 0);
                let a = pf_core::value::js_round(self.xpn(p, 1));
                let b = pf_core::value::js_round(self.xpn(p, 2));
                let st = if a <= b { 1.0 } else { -1.0 };
                let mut k = a;
                let mut guard = 0;
                while (if st > 0.0 { k <= b } else { k >= b }) && self.pending.is_none() && guard < 100000 {
                    self.loop_stack[top].i = k;
                    let mut s2 = sol.clone();
                    if self.run_event(ev, &mut s2, n + 1, prev_ok) {
                        any = true;
                    }
                    k += st;
                    guard += 1;
                }
            }
            "foreach" => {
                if let Some(t) = self.tid(p, 0) {
                    let list = self.picked(sol, t);
                    for (k, &it) in list.iter().enumerate() {
                        if self.pending.is_some() {
                            break;
                        }
                        if !self.alive(it) {
                            continue;
                        }
                        self.loop_stack[top].i = k as f64;
                        let mut s2 = sol.clone();
                        s2.insert(t, Rc::new(vec![it]));
                        if self.run_event(ev, &mut s2, n + 1, prev_ok) {
                            any = true;
                        }
                    }
                }
            }
            "foreachitem" => {
                if let Some(t) = self.tid(p, 0) {
                    if let Some(d) = self.data.get(&t).cloned() {
                        match d {
                            DataStore::List(l) => {
                                for (k, v) in l.iter().enumerate() {
                                    if self.pending.is_some() {
                                        break;
                                    }
                                    let f = &mut self.loop_stack[top];
                                    f.i = k as f64;
                                    f.val = Some(v.clone());
                                    f.key = Some(Val::N(k as f64));
                                    let mut s2 = sol.clone();
                                    if self.run_event(ev, &mut s2, n + 1, prev_ok) {
                                        any = true;
                                    }
                                }
                            }
                            DataStore::Map(m) => {
                                for (k, (key, v)) in m.iter().enumerate() {
                                    if self.pending.is_some() {
                                        break;
                                    }
                                    let f = &mut self.loop_stack[top];
                                    f.i = k as f64;
                                    f.key = Some(Val::S(key.clone()));
                                    f.val = Some(v.clone());
                                    let mut s2 = sol.clone();
                                    if self.run_event(ev, &mut s2, n + 1, prev_ok) {
                                        any = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "while" => {
                let mut k = 0;
                while k < 10000 && self.pending.is_none() {
                    self.loop_stack[top].i = k as f64;
                    let mut s2 = sol.clone();
                    if !self.run_event(ev, &mut s2, n + 1, prev_ok) {
                        break;
                    }
                    any = true;
                    k += 1;
                }
            }
            _ => {}
        }
        self.loop_stack.pop();
        any
    }

    pub fn run_list(&mut self, list: &Rc<Vec<Rc<Ev>>>, parent: &Sol) {
        let mut sc: Option<ScopeRef> = None;
        for ev in list.iter() {
            if ev.kind == EvKind::Lvar && !ev.name.is_empty() {
                let s = sc.get_or_insert_with(|| Rc::new(RefCell::new(Scope { vars: HashMap::new(), fn_: false, ret: Val::N(0.0), st: Vec::new() })));
                let key = format!("lv:{}", ev.id);
                let stv = self.cstate.get(&key).and_then(|c| c.lv.clone());
                let v = if ev.is_static && stv.is_some() {
                    stv.unwrap()
                } else if ev.text_type {
                    Val::S(if ev.value.is_null() { String::new() } else { jstr(&ev.value) })
                } else {
                    Val::N(Val::from_json(&ev.value).num0())
                };
                let mut b = s.borrow_mut();
                b.vars.insert(ev.name.clone(), v);
                if ev.is_static {
                    b.st.push(ev.clone());
                }
            }
        }
        if let Some(s) = &sc {
            self.scopes.push(s.clone());
        }
        let mut prev_ok = true;
        for ev in list.iter() {
            if ev.off {
                continue;
            }
            match ev.kind {
                EvKind::Event => {
                    let mut s2 = parent.clone();
                    prev_ok = self.run_event(ev, &mut s2, 0, prev_ok);
                }
                EvKind::Group => {
                    if self.group_on(ev) {
                        let subs = ev.subs.clone();
                        self.run_list(&subs, parent);
                    }
                }
                EvKind::Include => {
                    let sh = self.sheets.iter().find(|s| s.0 == ev.sheet).cloned();
                    if let Some((sid, evs)) = sh {
                        if !self.include_stack.contains(&sid) && sid != self.sheet_id {
                            self.include_stack.push(sid);
                            self.run_list(&evs, parent);
                            self.include_stack.pop();
                        }
                    }
                }
                _ => {}
            }
            if self.pending.is_some() {
                break;
            }
        }
        if let Some(s) = sc {
            let b = s.borrow();
            for e in &b.st {
                let v = b.vars.get(&e.name).cloned();
                self.cstate.entry(format!("lv:{}", e.id)).or_default().lv = v;
            }
            drop(b);
            if let Some(k) = self.scopes.iter().rposition(|x| Rc::ptr_eq(x, &s)) {
                self.scopes.remove(k);
            }
        }
    }

    // ---------------------------------------------------------------- pas de simulation

    pub fn step(&mut self, dt: f64) {
        self.cam_px = self.cam.0;
        self.cam_py = self.cam.1;
        self.tick += 1;
        if self.hs_done {
            self.hs_done = false;
            self.hs_in = None;
            self.hs_named_tick = self.tick;
        }
        if self.hs_in.is_some() && !self.ctl.open {
            self.hs_step(dt);
        }
        if !self.ctl.open && !self.gpaused && self.hs_in.is_none() && !self.tr.as_ref().map(|t| t.phase_out).unwrap_or(false) {
            self.cine_step(dt);
        }
        let mut frozen = self.dlg.cur.is_some() || self.hs_in.is_some() || self.cine.as_ref().map(|c| c.def.get("freeze") != Some(&Json::Bool(false))).unwrap_or(false);
        if frozen && !self.ctl.open {
            self.dlg_step(dt);
        }
        frozen = frozen || self.dlg.cur.is_some() || self.ctl.open;
        if !frozen || self.gpaused {
            self.btn_step();
        }
        frozen = frozen || self.gpaused;
        if self.ctl.open {
            self.kp.clear();
            self.mouse.pr.clear();
        }
        if !frozen {
            self.time += dt;
        }
        self.solids.clear();
        self.jts.clear();
        self.ladders.clear();
        self.poly_count = 0;
        let all = self.all.clone();
        for &s in &all {
            {
                let i = self.inst_mut(s);
                i.px = i.x;
                i.py = i.y;
                if i.dead {
                    continue;
                }
            }
            let t = self.inst(s).t;
            if self.inst(s).tm.is_some() {
                if self.types[t].solid {
                    self.tm_tiles(s);
                    let vt = &self.inst(s).tm.as_ref().unwrap().vt;
                    let (r, p, j) = (vt.rect.clone(), vt.poly.clone(), vt.jt.clone());
                    for k in r {
                        self.solids.push(ORef::V(s, k));
                    }
                    for k in p {
                        self.solids.push(ORef::V(s, k));
                        self.poly_count += 1;
                    }
                    for k in j {
                        self.jts.push(ORef::V(s, k));
                    }
                }
                continue;
            }
            let ty = &self.types[t];
            if ty.solid {
                self.solids.push(ORef::I(s));
                if ty.def.poly_col && ty.def.kind == Kind::Sprite {
                    self.poly_count += 1;
                }
            }
            if ty.jumpthru {
                self.jts.push(ORef::I(s));
            }
            if ty.ladder {
                self.jts.push(ORef::I(s));
                self.ladders.push(ORef::I(s));
            }
            if self.inst(s).b.physics.is_some() && !ty.solid {
                self.solids.push(ORef::I(s));
            }
        }
        if !frozen {
            for &s in &all {
                if !self.alive(s) {
                    continue;
                }
                let i = self.inst_mut(s);
                if let Some(sn) = i.b.sine.as_mut() {
                    if sn.en {
                        sn.t += dt;
                        let off = (sn.t * std::f64::consts::PI * 2.0 / sn.period.max(0.05)).sin() * sn.magnitude;
                        if sn.vertical {
                            i.y = sn.by + off;
                        } else {
                            i.x = sn.bx + off;
                        }
                    }
                }
                if i.b.path.is_some() {
                    self.path_step(s, dt);
                }
                let i = self.inst_mut(s);
                if let Some(u) = i.b.bullet.as_mut() {
                    u.vy += u.gravity * dt;
                    let (vx, vy) = (u.vx, u.vy);
                    i.x += vx * dt;
                    i.y += vy * dt;
                }
                if let Some(r) = i.b.rotate {
                    i.angle += r * dt;
                }
            }
            if !self.pending_ropes.is_empty() {
                for (s, len) in std::mem::take(&mut self.pending_ropes) {
                    if !self.alive(s) {
                        continue;
                    }
                    let c = self.center(s);
                    self.joints.push(Joint { a: s, b: None, px: c.0, py: c.1 - len, len, rigid: false });
                }
            }
            self.phys_step(dt);
        }
        for &s in &all {
            let i = self.inst_mut(s);
            i.dx = i.x - i.px;
            i.dy = i.y - i.py;
        }
        if !frozen {
            for &s in &all {
                if !self.alive(s) {
                    continue;
                }
                if self.inst(s).b.patrol.is_some() {
                    self.patrol(s, dt);
                }
                if self.inst(s).b.platform.is_some() {
                    self.platform(s, dt);
                }
            }
            if self.mouse.pr.contains("left") && self.mouse.psx.is_some() {
                let (msx, msy) = (self.mouse.sx, self.mouse.sy);
                self.mouse.sx = self.mouse.psx.unwrap();
                self.mouse.sy = self.mouse.psy.unwrap();
                for &s in self.all.clone().iter().rev() {
                    let ok = {
                        let i = self.inst(s);
                        !i.dead && i.b.drag.as_ref().map(|d| d.en).unwrap_or(false)
                    };
                    if !ok {
                        continue;
                    }
                    if self.mouse_over(s) {
                        let mw = self.mouse_world(s);
                        let tick = self.tick;
                        let i = self.inst_mut(s);
                        let (x, y) = (i.x, i.y);
                        let dg = i.b.drag.as_mut().unwrap();
                        dg.on = true;
                        dg.st = tick;
                        dg.gx = mw.0 - x;
                        dg.gy = mw.1 - y;
                        break;
                    }
                }
                self.mouse.sx = msx;
                self.mouse.sy = msy;
            }
            let all2 = self.all.clone();
            for &s in &all2 {
                if self.alive(s) {
                    self.behave(s, dt);
                }
            }
            for &s in &all2 {
                if self.alive(s) && self.inst(s).b.pin.is_some() {
                    self.pin_step(s);
                }
            }
        }
        if let Some(f) = self.sflash.as_mut() {
            if f.1 > 0.0 {
                f.1 = (f.1 - dt).max(0.0);
            }
        }
        let (lw, lh) = (self.lw(), self.lh());
        let tick = self.tick;
        for &s in &self.all.clone() {
            if !self.alive(s) {
                continue;
            }
            {
                let i = self.inst_mut(s);
                if let Some(tf) = i.tf.as_mut() {
                    if tf.1 > 0.0 {
                        tf.1 = (tf.1 - dt).max(0.0);
                    }
                }
            }
            if frozen {
                self.animate(s, dt);
                continue;
            }
            let d = self.def(s);
            if let Some(bottom) = self.inst(s).b.bound {
                let bb = draw_box(&d, self.inst(s));
                let i = self.inst_mut(s);
                if bb.l < 0.0 {
                    i.x -= bb.l;
                }
                if bb.r > lw {
                    i.x -= bb.r - lw;
                }
                if bb.t < 0.0 {
                    i.y -= bb.t;
                }
                if bottom && bb.b > lh {
                    i.y -= bb.b - lh;
                }
            }
            if self.inst(s).b.destroy {
                let db = draw_box(&d, self.inst(s));
                if db.r < -64.0 || db.l > lw + 64.0 || db.b < -400.0 || db.t > lh + 64.0 {
                    self.inst_mut(s).dead = true;
                }
            }
            if let Some(tm) = self.inst_mut(s).b.timer.as_mut() {
                let mut k = 0;
                while k < tm.timers.len() {
                    let (nm, q) = &mut tm.timers[k];
                    q.left -= dt;
                    if q.left <= 1e-9 {
                        tm.fired.insert(nm.clone(), tick);
                        if q.reg && q.dur > 0.0 {
                            q.left += q.dur;
                        } else {
                            tm.timers.remove(k);
                            continue;
                        }
                    }
                    k += 1;
                }
            }
            self.animate(s, dt);
            if self.inst(s).bar.is_some() {
                self.bar_step(s, dt);
            }
            let i = self.inst_mut(s);
            if i.flash > 0.0 {
                i.flash = (i.flash - dt).max(0.0);
            }
        }
        self.run_waits();
        if let (Some(sh), true) = (self.sheet.clone(), self.pending.is_none()) {
            self.run_list(&sh, &Sol::new());
        }
        for &s in &self.all.clone() {
            if let Some(i) = self.insts[s as usize].as_deref_mut() {
                if i.kill {
                    i.dead = true;
                }
            }
        }
        self.first_tick = false;
        self.kp.clear();
        self.pkp.clear();
        self.kr.clear();
        self.any_press = false;
        self.mouse.pr.clear();
        self.mouse.rl.clear();
        if self.all.iter().any(|&s| !self.alive(s)) {
            let dead: Vec<Slot> = self.all.iter().copied().filter(|&s| !self.alive(s)).collect();
            let alive: Vec<Slot> = self.all.iter().copied().filter(|&s| self.alive(s)).collect();
            self.all = alive;
            for t in 0..self.types.len() {
                let l: Vec<Slot> = self.types[t].insts.iter().copied().filter(|&s| self.alive(s)).collect();
                self.types[t].insts = l;
            }
            let _ = dead;
        }
        if self.shake_t > 0.0 {
            self.shake_t = (self.shake_t - dt).max(0.0);
        }
        self.update_cam(dt);
        if let Some(pl) = self.pending.take() {
            if self.trans.ty != "none" && self.trans.dur > 0.0 && self.tr.is_none() {
                self.tr = Some(TrState { phase_out: true, t: 0.0, target: pl, d: self.trans.dur / 2.0, ty: self.trans.ty.clone(), color: self.trans.color.clone() });
            } else {
                self.load_layout(&pl);
            }
        }
    }
}

/// comparaison avec une valeur JS `undefined` (variable inexistante)
fn cmp_undefined(op: &str, b: &Val) -> bool {
    // undefined == x est faux, undefined != x est vrai, les comparaisons d'ordre sont fausses
    let _ = b;
    op == "<>"
}
