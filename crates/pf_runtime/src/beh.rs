//! Comportements : plateforme, patrouille, trajectoire, physique, 8 directions, voiture…

use crate::coll::*;
use crate::model::*;
use crate::rt::{AudioCmd, Joint, Runtime, DT};

pub fn approach(v: f64, t: f64, d: f64) -> f64 {
    if v < t {
        (v + d).min(t)
    } else {
        (v - d).max(t)
    }
}

pub fn ease(k: &str, t: f64) -> f64 {
    match k {
        "linear" => t,
        "in" => t * t * t,
        "out" => 1.0 - (1.0 - t).powi(3),
        "back" => {
            let c = 1.70158;
            1.0 + (c + 1.0) * (t - 1.0).powi(3) + c * (t - 1.0).powi(2)
        }
        "elastic" => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                2f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * 2.0 * std::f64::consts::PI / 3.0).sin() + 1.0
            }
        }
        "bounce" => {
            let n = 7.5625;
            let d = 2.75;
            if t < 1.0 / d {
                n * t * t
            } else if t < 2.0 / d {
                let t = t - 1.5 / d;
                n * t * t + 0.75
            } else if t < 2.5 / d {
                let t = t - 2.25 / d;
                n * t * t + 0.9375
            } else {
                let t = t - 2.625 / d;
                n * t * t + 0.984375
            }
        }
        _ => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }
    }
}

struct PushChain {
    l: Vec<Slot>,
    r: Vec<(Slot, Slot, i32)>,
    m: f64,
}

#[derive(Clone, Copy)]
struct PShape {
    c: bool,
    x: f64,
    y: f64,
    r: f64,
    b: BBox,
}
#[derive(Clone, Copy)]
struct Manifold {
    nx: f64,
    ny: f64,
    d: f64,
}

fn box_box(a: &BBox, b: &BBox) -> Option<Manifold> {
    let ox = a.r.min(b.r) - a.l.max(b.l);
    let oy = a.b.min(b.b) - a.t.max(b.t);
    if ox <= 0.0 || oy <= 0.0 {
        return None;
    }
    let acx = (a.l + a.r) / 2.0;
    let bcx = (b.l + b.r) / 2.0;
    let acy = (a.t + a.b) / 2.0;
    let bcy = (b.t + b.b) / 2.0;
    if ox < oy {
        return Some(Manifold { nx: if acx < bcx { -1.0 } else { 1.0 }, ny: 0.0, d: ox });
    }
    Some(Manifold { nx: 0.0, ny: if acy < bcy { -1.0 } else { 1.0 }, d: oy })
}
fn circ_box(c: &PShape, b: &BBox) -> Option<Manifold> {
    let qx = b.l.max(c.x.min(b.r));
    let qy = b.t.max(c.y.min(b.b));
    let dx = c.x - qx;
    let dy = c.y - qy;
    let d2 = dx * dx + dy * dy;
    if d2 > c.r * c.r {
        return None;
    }
    if d2 > 1e-9 {
        let d = d2.sqrt();
        return Some(Manifold { nx: dx / d, ny: dy / d, d: c.r - d });
    }
    box_box(&BBox::new(c.x - c.r, c.y - c.r, c.x + c.r, c.y + c.r), b)
}
fn circ_poly(c: &PShape, p: &[[f64; 2]]) -> Option<Manifold> {
    let mut best: Option<[f64; 2]> = None;
    let mut bd = 1e9;
    let inside = pt_in_poly(c.x, c.y, p);
    for k in 0..p.len() {
        let a = p[k];
        let b = p[(k + 1) % p.len()];
        let ex = b[0] - a[0];
        let ey = b[1] - a[1];
        let l2 = ex * ex + ey * ey;
        let t = if l2 != 0.0 { (((c.x - a[0]) * ex + (c.y - a[1]) * ey) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let qx = a[0] + ex * t;
        let qy = a[1] + ey * t;
        let d = (c.x - qx).hypot(c.y - qy);
        if d < bd {
            bd = d;
            best = Some([qx, qy]);
        }
    }
    let best = best?;
    if !inside && bd >= c.r {
        return None;
    }
    let mut nx = c.x - best[0];
    let mut ny = c.y - best[1];
    let l = { let l = nx.hypot(ny); if l == 0.0 { 1.0 } else { l } };
    nx /= l;
    ny /= l;
    if inside {
        return Some(Manifold { nx: -nx, ny: -ny, d: c.r + bd });
    }
    Some(Manifold { nx, ny, d: c.r - bd })
}
fn manifold(sa: &PShape, sb: &PShape, poly_b: Option<&[[f64; 2]]>) -> Option<Manifold> {
    if sa.c {
        if let Some(p) = poly_b {
            return circ_poly(sa, p);
        }
        if sb.c {
            let dx = sa.x - sb.x;
            let dy = sa.y - sb.y;
            let mut d = dx.hypot(dy);
            let rr = sa.r + sb.r;
            if d >= rr {
                return None;
            }
            if d == 0.0 {
                d = 0.0001;
            }
            return Some(Manifold { nx: dx / d, ny: dy / d, d: rr - d });
        }
        return circ_box(sa, &sb.b);
    }
    if sb.c {
        return circ_box(sb, &sa.b).map(|m| Manifold { nx: -m.nx, ny: -m.ny, d: m.d });
    }
    let bb = match poly_b {
        Some(p) => poly_bb(p),
        None => sb.b,
    };
    box_box(&sa.b, &bb)
}

impl Runtime {
    fn can_push(&self, b: Option<ORef>) -> bool {
        match b {
            Some(ORef::I(s)) => {
                let i = self.inst(s);
                matches!(&i.b.physics, Some(ph) if !ph.fixed && ph.pushable && !i.dead)
            }
            _ => false,
        }
    }
    fn phys_ground(&self, s: Slot) -> bool {
        self.inst(s).b.physics.as_ref().map(|p| p.ground).unwrap_or(false)
    }
    fn push_chain(&mut self, b: Slot, dir: f64, reach: f64) -> PushChain {
        let mut l = vec![b];
        let mut m = self.phys_mass(b);
        while l.len() < 12 {
            let c = *l.last().unwrap();
            let ox = self.inst(c).x;
            let g = self.phys_ground(c);
            self.move_x(c, dir * (reach + 0.5), g);
            let nb = self.last_blk;
            self.inst_mut(c).x = ox;
            if !self.can_push(nb) {
                break;
            }
            let nb = match nb {
                Some(ORef::I(s)) => s,
                _ => break,
            };
            if l.contains(&nb) {
                break;
            }
            l.push(nb);
            m += self.phys_mass(nb);
        }
        for &q in &l {
            self.inst_mut(q).plv = 0;
        }
        let mut rid: Vec<(Slot, Slot, i32)> = Vec::new();
        let mut seen = l.clone();
        let bl: Vec<Slot> = self.all.iter().copied().filter(|&o| self.can_push(Some(ORef::I(o))) && !seen.contains(&o)).collect();
        let mut k = 0;
        while k < seen.len() && seen.len() < 30 {
            let sb = self.box_i(seen[k]);
            for &o in &bl {
                if seen.contains(&o) {
                    continue;
                }
                let ob = self.box_i(o);
                if ob.r > sb.l + 1.0 && ob.l < sb.r - 1.0 && (ob.b - sb.t).abs() <= 2.5 {
                    seen.push(o);
                    let lv = self.inst(seen[k]).plv + 1;
                    rid.push((o, seen[k], lv));
                    self.inst_mut(o).plv = lv;
                    m += self.phys_mass(o);
                }
            }
            k += 1;
        }
        let xs: Vec<f64> = rid.iter().map(|q| self.inst(q.0).x).collect();
        let mut idx: Vec<usize> = (0..rid.len()).collect();
        idx.sort_by(|&a, &c| {
            let r = rid[a].2.cmp(&rid[c].2);
            if r != std::cmp::Ordering::Equal {
                return r;
            }
            ((xs[c] - xs[a]) * dir).partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
        });
        let rid = idx.into_iter().map(|k| rid[k]).collect();
        PushChain { l, r: rid, m }
    }
    fn push_move(&mut self, ch: &PushChain, want: f64) -> Slot {
        for k in (0..ch.l.len()).rev() {
            let b = ch.l[k];
            let x0 = self.inst(b).x;
            let g = self.phys_ground(b);
            self.move_x(b, want, g);
            let i = self.inst_mut(b);
            i.pmv = i.x - x0;
        }
        for &(o, sup, _) in &ch.r {
            let d = self.inst(sup).pmv;
            let x1 = self.inst(o).x;
            if d != 0.0 {
                self.move_x(o, d, true);
            }
            let i = self.inst_mut(o);
            i.pmv = i.x - x1;
        }
        let last = *ch.l.last().unwrap();
        let mut all = ch.l.clone();
        all.extend(ch.r.iter().map(|q| q.0));
        let tick = self.tick;
        for b in all {
            let i = self.inst_mut(b);
            let pmv = i.pmv;
            if let Some(ph) = i.b.physics.as_mut() {
                if pmv.abs() > 0.001 {
                    ph.vx = if ph.circle && b == last { pmv / DT } else { 0.0 };
                    ph.push_t = tick;
                }
            }
            i.pmv = 0.0;
        }
        ch.l[0]
    }

    fn key(&self, k: &str) -> bool {
        self.keys.contains(k)
    }
    fn kpr(&self, k: &str) -> bool {
        self.kp.contains(k)
    }

    pub fn platform(&mut self, s: Slot, dt: f64) {
        let lock = self.cine_lock();
        let mut st = self.inst_mut(s).b.platform.take().unwrap();
        self.platform_inner(s, dt, &mut st, lock);
        st.sim = Sim::default();
        self.inst_mut(s).b.platform = Some(st);
    }
    fn platform_inner(&mut self, s: Slot, dt: f64, st: &mut PlatformSt, lock: bool) {
        st.landed = false;
        st.jumped = false;
        if !st.en {
            return;
        }
        let c = st.controls && !lock;
        let l = st.sim.left || (c && (self.key("ArrowLeft") || self.key("KeyA")));
        let r = st.sim.right || (c && (self.key("ArrowRight") || self.key("KeyD")));
        let j = st.hold || (c && (self.key("ArrowUp") || self.key("KeyW") || self.key("Space")));
        let jp = st.sim.jump || (c && (self.kpr("ArrowUp") || self.kpr("KeyW") || self.kpr("Space")));
        let u = st.sim.up || (c && (self.key("ArrowUp") || self.key("KeyW")));
        let d = st.sim.down || (c && (self.key("ArrowDown") || self.key("KeyS")));
        if let Some(f) = st.floor {
            if !self.ref_dead(f) {
                if let ORef::I(fs) = f {
                    let (fdx, fdy) = (self.inst(fs).dx, self.inst(fs).dy);
                    if fdy != 0.0 {
                        self.inst_mut(s).y += fdy;
                    }
                    if fdx != 0.0 {
                        self.move_x(s, fdx, false);
                    }
                }
            }
        }
        let mut dir = (r as i32 - l as i32) as f64;
        if st.lock > 0.0 {
            st.lock -= dt;
            if st.wall_dir != 0.0 {
                dir = -st.wall_dir;
            }
        }
        if st.climb_speed > 0.0 && !self.ladders.is_empty() {
            let b0 = self.box_i(s);
            let cx0 = (b0.l + b0.r) / 2.0;
            let in_l = |lb: &BBox| cx0 >= lb.l - 2.0 && cx0 <= lb.r + 2.0;
            if st.climb.is_none() {
                for ld in self.ladders.clone() {
                    if self.ref_dead(ld) {
                        continue;
                    }
                    let lb = self.box_r(ld);
                    if !in_l(&lb) {
                        continue;
                    }
                    if u && b0.b > lb.t + 2.0 && b0.t < lb.b {
                        st.climb = Some(ld);
                        break;
                    }
                    if d && st.floor == Some(ld) && (b0.b - lb.t).abs() <= 3.0 {
                        st.climb = Some(ld);
                        self.inst_mut(s).y += 3.0;
                        break;
                    }
                }
            }
            if let Some(cl) = st.climb {
                let clb = self.box_r(cl);
                let js = st.sim.jump || (c && self.kpr("Space"));
                if self.ref_dead(cl) || !in_l(&clb) || b0.t >= clb.b {
                    st.climb = None;
                } else if js {
                    st.climb = None;
                    st.vy = -st.jump * 0.85;
                    st.jumped = true;
                    st.floor = None;
                    st.jumps = 1;
                    st.buf = 0.0;
                } else {
                    let cs = st.climb_speed;
                    let cvy = (d as i32 - u as i32) as f64 * cs;
                    st.vx = dir * cs * 0.7;
                    st.vy = 0.0;
                    st.floor = None;
                    self.inst_mut(s).nojt = true;
                    self.move_x(s, st.vx * dt, false);
                    let hit = self.move_y(s, cvy * dt);
                    self.inst_mut(s).nojt = false;
                    let bb = self.box_i(s);
                    if cvy < 0.0 && bb.b <= clb.t + 0.5 {
                        self.inst_mut(s).y += clb.t - bb.b;
                        st.climb = None;
                        st.floor = Some(cl);
                    } else if cvy > 0.0 && hit.is_some() {
                        st.climb = None;
                        st.floor = hit;
                    }
                    st.coyote = 0.0;
                    return;
                }
            }
        }
        if dir != 0.0 {
            let mut a = st.accel;
            if st.vx * dir < 0.0 {
                a += st.decel;
            }
            st.vx = approach(st.vx, dir * st.max_speed, a * dt);
        } else {
            st.vx = approach(st.vx, 0.0, st.decel * dt);
        }
        if st.vx.abs() > st.max_speed {
            st.vx = approach(st.vx, dir * st.max_speed, st.decel * dt);
        }
        st.coyote = if st.floor.is_some() { 0.1 } else { st.coyote - dt };
        if jp {
            st.buf = 0.12;
        } else {
            st.buf -= dt;
        }
        if st.wall_t > 0.0 {
            st.wall_t -= dt;
        }
        if st.buf > 0.0 && st.wall_jump && st.floor.is_none() && st.coyote <= 0.0 && st.wall_t > 0.0 {
            st.vy = -st.jump;
            st.vx = -st.wall_dir * st.wall_kick;
            st.lock = 0.16;
            st.jumped = true;
            st.wj = self.tick;
            st.buf = 0.0;
            st.wall_t = 0.0;
            st.jumps = 1;
            st.hold = st.sim.jump;
        }
        if st.buf > 0.0 {
            if st.floor.is_some() || st.coyote > 0.0 {
                st.vy = -st.jump;
                st.jumped = true;
                st.buf = 0.0;
                st.coyote = 0.0;
                st.floor = None;
                st.jumps = 1;
                st.hold = st.sim.jump;
            } else if st.double_jump && st.jumps < 2 {
                st.vy = -st.jump * 0.9;
                st.jumped = true;
                st.buf = 0.0;
                st.jumps = 2;
                st.hold = st.sim.jump;
            }
        }
        if st.vy >= 0.0 {
            st.hold = false;
        }
        let mut g = st.gravity;
        if st.vy < 0.0 && !j && !st.hold {
            g *= 2.4;
        }
        st.vy = (st.vy + g * dt).min(st.max_fall);
        let was = st.floor.is_some();
        let gr = was && st.vy >= 0.0;
        self.inst_mut(s).sf = None;
        let pvx = st.vx;
        let hx0 = self.inst(s).x;
        let dxw = st.vx * dt;
        let wall_hit = self.move_x(s, dxw, gr);
        st.pushing = None;
        if wall_hit != 0 {
            let pb = self.last_blk;
            let pushable = match pb {
                Some(ORef::I(b)) => matches!(&self.inst(b).b.physics, Some(ph) if !ph.fixed && ph.pushable),
                _ => false,
            };
            if pushable && dir != 0.0 && dir as i32 == wall_hit && st.floor != pb {
                let pbs = match pb {
                    Some(ORef::I(b)) => b,
                    _ => unreachable!(),
                };
                let rem = dxw - (self.inst(s).x - hx0);
                let ch = self.push_chain(pbs, dir, rem.abs());
                let psp = (st.max_speed * 0.75).min(260.0 / ch.m.max(0.2).sqrt());
                let want = dir * rem.abs().min(psp * dt);
                let bx0 = self.inst(pbs).x;
                if want != 0.0 {
                    self.push_move(&ch, want);
                    let mv = self.inst(pbs).x - bx0;
                    if mv.abs() > 0.001 {
                        self.move_x(s, mv, gr);
                    }
                }
                st.pushing = pb;
                st.vx = dir * pvx.abs().min(psp);
            } else {
                st.vx = 0.0;
            }
        }
        if st.floor.is_none() && dir != 0.0 && (st.wall_jump || st.wall_slide > 0.0) {
            let mut blk = wall_hit != 0 && st.vx == 0.0;
            if !blk {
                let ox = self.inst(s).x;
                if self.move_x(s, dir * 0.6, false) != 0 {
                    blk = true;
                }
                self.inst_mut(s).x = ox;
            }
            if blk && (st.lock <= 0.0 || dir != -st.wall_dir) {
                st.wall_t = 0.1;
                st.wall_dir = dir;
            }
        }
        if st.wall_slide > 0.0 && st.wall_t > 0.0 && st.floor.is_none() && st.vy > st.wall_slide && dir == st.wall_dir {
            st.vy = st.wall_slide;
        }
        let f = self.move_y(s, st.vy * dt);
        if st.vy > 0.0 {
            st.floor = f;
            if f.is_some() {
                st.vy = 0.0;
            }
        } else {
            st.floor = None;
            if f.is_some() && st.vy < 0.0 {
                st.vy = 0.0;
            }
        }
        if st.floor.is_none() && gr {
            let bx = self.box_i(s);
            let gs = self.ground_snap(s, climb_of(&bx, st.vx * dt));
            if gs.is_some() {
                st.floor = gs;
                st.vy = 0.0;
            }
        }
        let sf = self.inst(s).sf;
        if st.floor.is_none() && sf.is_some() && st.vy >= 0.0 {
            st.floor = sf;
            st.vy = 0.0;
        }
        if st.floor.is_some() && !was {
            st.landed = true;
            st.jumps = 0;
        }
    }

    pub fn path_step(&mut self, s: Slot, dt: f64) {
        let tick = self.tick;
        let i = self.inst_mut(s);
        let st = i.b.path.as_mut().unwrap();
        let n = st.pts.len();
        if !st.en || n < 2 {
            return;
        }
        if st.wt > 0.0 {
            st.wt -= dt;
            return;
        }
        let a = st.pts[st.k];
        let b = st.pts[st.nx];
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        st.u += if len > 0.001 { st.speed * dt / len } else { 1.0 };
        if st.u >= 1.0 {
            st.u = 0.0;
            st.last = st.nx;
            st.arr = tick;
            st.wt = st.pause;
            match st.mode.as_str() {
                "loop" => {
                    st.k = st.nx;
                    st.nx = (st.nx + 1) % n;
                }
                "once" => {
                    if st.nx >= n - 1 {
                        st.en = false;
                        st.fin = tick;
                        st.k = st.nx;
                    } else {
                        st.k = st.nx;
                        st.nx += 1;
                    }
                }
                _ => {
                    st.k = st.nx;
                    if st.nx >= n - 1 {
                        st.dir = -1;
                    } else if st.nx == 0 {
                        st.dir = 1;
                    }
                    st.nx = (st.k as i64 + st.dir as i64).max(0) as usize;
                }
            }
            let (bx, by) = (st.bx, st.by);
            i.x = bx + b[0];
            i.y = by + b[1];
            return;
        }
        let e = if st.ease { st.u * st.u * (3.0 - 2.0 * st.u) } else { st.u };
        let (bx, by, rot) = (st.bx, st.by, st.rotate);
        i.x = bx + a[0] + (b[0] - a[0]) * e;
        i.y = by + a[1] + (b[1] - a[1]) * e;
        if rot && len > 0.001 {
            i.angle = (b[1] - a[1]).atan2(b[0] - a[0]) * 180.0 / std::f64::consts::PI;
        }
    }

    // ---------------------------------------------------------------- physique

    fn phys_c(&self, s: Slot) -> (f64, f64) {
        self.center(s)
    }
    pub fn phys_mass(&self, s: Slot) -> f64 {
        let b = self.box_i(s);
        let d = self.inst(s).b.physics.as_ref().map(|p| p.density).unwrap_or(1.0);
        ((b.r - b.l) * (b.b - b.t) / 1024.0 * d).max(0.01)
    }
    fn phys_inv(&self, s: Slot) -> f64 {
        match &self.inst(s).b.physics {
            None => 0.0,
            Some(p) if p.fixed => 0.0,
            _ => 1.0 / self.phys_mass(s),
        }
    }
    fn phys_shape(&self, s: Slot) -> PShape {
        let b = self.box_i(s);
        let c = self.inst(s).b.physics.as_ref().map(|p| p.circle).unwrap_or(false);
        PShape { c, x: (b.l + b.r) / 2.0, y: (b.t + b.b) / 2.0, r: (b.r - b.l).min(b.b - b.t) / 2.0, b }
    }
    /// B : autre corps (instance ou tuile) ; is_static : on ne déplace pas B
    fn phys_resolve(&mut self, a: Slot, b: Option<ORef>, m: Manifold, ia: f64, ib: f64, is_static: bool) {
        let tot = ia + ib;
        if tot <= 0.0 {
            return;
        }
        let corr = (m.d - 0.05).max(0.0) / tot * 0.9;
        {
            let i = self.inst_mut(a);
            i.x += m.nx * corr * ia;
            i.y += m.ny * corr * ia;
        }
        let bs = match b {
            Some(ORef::I(s)) => Some(s),
            _ => None,
        };
        let pb_has = bs.map(|s| self.inst(s).b.physics.is_some()).unwrap_or(false);
        if let Some(bsl) = bs {
            if !is_static && ib > 0.0 {
                let i = self.inst_mut(bsl);
                i.x -= m.nx * corr * ib;
                i.y -= m.ny * corr * ib;
            }
        }
        let (bvx, bvy) = if pb_has {
            let p = self.inst(bs.unwrap()).b.physics.as_ref().unwrap();
            (p.vx, p.vy)
        } else if let Some(bsl) = bs {
            let i = self.inst(bsl);
            (if i.dx != 0.0 { i.dx * 60.0 } else { 0.0 }, if i.dy != 0.0 { i.dy * 60.0 } else { 0.0 })
        } else {
            (0.0, 0.0)
        };
        let tick = self.tick;
        let (pb_bounce, pb_fric) = if pb_has {
            let p = self.inst(bs.unwrap()).b.physics.as_ref().unwrap();
            (Some(p.bounce), Some(p.friction))
        } else {
            (None, None)
        };
        let pa = self.inst_mut(a).b.physics.as_mut().unwrap();
        let rvx = pa.vx - bvx;
        let rvy = pa.vy - bvy;
        let vn = rvx * m.nx + rvy * m.ny;
        if m.ny < -0.5 {
            pa.ground = true;
        }
        let pa_bounce = pa.bounce;
        let pa_fric = pa.friction;
        if pb_has && m.ny > 0.5 {
            self.inst_mut(bs.unwrap()).b.physics.as_mut().unwrap().ground = true;
        }
        if vn >= 0.0 {
            return;
        }
        let mut e = pa_bounce.max(pb_bounce.unwrap_or(0.0));
        if -vn < 60.0 {
            e = 0.0;
        }
        let jj = -(1.0 + e) * vn / tot;
        let tx = -m.ny;
        let ty = m.nx;
        let vt = rvx * tx + rvy * ty;
        let mu = (pa_fric * pb_fric.unwrap_or(0.6)).sqrt();
        let mut jt = -vt / tot;
        let lim = jj.abs() * mu;
        jt = jt.clamp(-lim, lim);
        {
            let pa = self.inst_mut(a).b.physics.as_mut().unwrap();
            pa.vx += jj * m.nx * ia;
            pa.vy += jj * m.ny * ia;
            if -vn > pa.hit_v || pa.hit_t != tick {
                pa.hit_t = tick;
                pa.hit_v = -vn;
            }
            pa.vx += jt * tx * ia;
            pa.vy += jt * ty * ia;
        }
        if pb_has {
            let pb = self.inst_mut(bs.unwrap()).b.physics.as_mut().unwrap();
            if ib > 0.0 {
                pb.vx -= jj * m.nx * ib;
                pb.vy -= jj * m.ny * ib;
            }
            pb.hit_t = tick;
            pb.hit_v = pb.hit_v.max(-vn);
            if ib > 0.0 {
                pb.vx -= jt * tx * ib;
                pb.vy -= jt * ty * ib;
            }
        }
    }
    pub fn phys_step(&mut self, dt: f64) {
        let bl: Vec<Slot> = self.all.iter().copied().filter(|&s| {
            let i = self.inst(s);
            !i.dead && i.b.physics.is_some()
        }).collect();
        if bl.is_empty() && self.joints.is_empty() {
            return;
        }
        let joints: Vec<Joint> = self.joints.iter().filter(|j| self.alive(j.a) && j.b.map(|b| self.alive(b)).unwrap_or(true)).cloned().collect();
        self.joints = joints;
        let sub = 2;
        let h = dt / sub as f64;
        for _ in 0..sub {
            for &s in &bl {
                let i = self.inst_mut(s);
                let ph = i.b.physics.as_mut().unwrap();
                ph.ground = false;
                if ph.fixed {
                    ph.vx = 0.0;
                    ph.vy = 0.0;
                    continue;
                }
                ph.vy = (ph.vy + ph.gravity * h).min(1600.0);
                ph.vx *= 0.9995;
                ph.vy *= 0.9995;
                let (vx, vy) = (ph.vx, ph.vy);
                i.x += vx * h;
                i.y += vy * h;
            }
            let stat: Vec<ORef> = self.solids.iter().copied().filter(|&so| !self.ref_dead(so) && !self.ref_phys(so)).collect();
            let chars: Vec<Slot> = self.all.iter().copied().filter(|&c| {
                let i = self.inst(c);
                !i.dead && (i.b.platform.is_some() || i.b.patrol.is_some() || i.b.eightdir.is_some() || i.b.car.is_some())
            }).collect();
            for _ in 0..3 {
                for &s in &bl {
                    if self.inst(s).b.physics.as_ref().unwrap().fixed {
                        continue;
                    }
                    let ia = self.phys_inv(s);
                    for &so in &stat {
                        let sa = self.phys_shape(s);
                        let sh = self.sshape(so);
                        let sb = sh.as_ref().map(|x| x.b).unwrap_or_else(|| self.box_r(so));
                        if sa.b.r < sb.l || sa.b.l > sb.r || sa.b.b < sb.t || sa.b.t > sb.b {
                            continue;
                        }
                        let m = manifold(&sa, &PShape { c: false, x: 0.0, y: 0.0, r: 0.0, b: sb }, sh.as_ref().map(|x| x.p.as_slice()));
                        if let Some(m) = m {
                            self.phys_resolve(s, Some(so), m, ia, 0.0, true);
                        }
                    }
                    for so in self.jts.clone() {
                        if self.ref_dead(so) || so == ORef::I(s) || self.ref_is_ladder(so) {
                            continue;
                        }
                        let sa = self.phys_shape(s);
                        let sb = self.box_r(so);
                        let vy = self.inst(s).b.physics.as_ref().unwrap().vy;
                        if vy < 0.0 || sa.b.b > sb.t + 6f64.max(vy * h + 2.0) || sa.b.r <= sb.l || sa.b.l >= sb.r || sa.b.b < sb.t {
                            continue;
                        }
                        self.phys_resolve(s, Some(so), Manifold { nx: 0.0, ny: -1.0, d: sa.b.b - sb.t }, ia, 0.0, true);
                    }
                    for &c in &chars {
                        let ci = self.inst(c);
                        let fl = ci.b.platform.as_ref().map(|p| p.floor).or_else(|| ci.b.patrol.as_ref().map(|p| p.floor)).flatten();
                        if fl == Some(ORef::I(s)) {
                            continue;
                        }
                        let sa = self.phys_shape(s);
                        let cb = self.box_i(c);
                        if let Some(m) = manifold(&sa, &PShape { c: false, x: 0.0, y: 0.0, r: 0.0, b: cb }, None) {
                            if m.ny < 0.3 {
                                self.phys_resolve(s, Some(ORef::I(c)), m, ia, 0.0, true);
                            }
                        }
                    }
                }
                for a in 0..bl.len() {
                    for b2 in a + 1..bl.len() {
                        let (aa, cc) = (bl[a], bl[b2]);
                        let ia2 = self.phys_inv(aa);
                        let ib2 = self.phys_inv(cc);
                        if ia2 + ib2 <= 0.0 {
                            continue;
                        }
                        if let Some(m2) = manifold(&self.phys_shape(aa), &self.phys_shape(cc), None) {
                            if ia2 > 0.0 {
                                self.phys_resolve(aa, Some(ORef::I(cc)), m2, ia2, ib2, false);
                            } else {
                                self.phys_resolve(cc, Some(ORef::I(aa)), Manifold { nx: -m2.nx, ny: -m2.ny, d: m2.d }, ib2, 0.0, true);
                            }
                        }
                    }
                }
                for j in self.joints.clone() {
                    let ca = self.phys_c(j.a);
                    let ia3 = self.phys_inv(j.a);
                    let (bx, by, ib3) = match j.b {
                        Some(b) => {
                            let cb = self.phys_c(b);
                            (cb.0, cb.1, self.phys_inv(b))
                        }
                        None => (j.px, j.py, 0.0),
                    };
                    let dx = ca.0 - bx;
                    let dy = ca.1 - by;
                    let d = { let d = dx.hypot(dy); if d == 0.0 { 0.0001 } else { d } };
                    if d <= j.len && !j.rigid {
                        continue;
                    }
                    let tot = ia3 + ib3;
                    if tot <= 0.0 {
                        continue;
                    }
                    let nx = dx / d;
                    let ny = dy / d;
                    let err = d - j.len;
                    {
                        let i = self.inst_mut(j.a);
                        i.x -= nx * err * ia3 / tot;
                        i.y -= ny * err * ia3 / tot;
                    }
                    if let Some(b) = j.b {
                        let i = self.inst_mut(b);
                        i.x += nx * err * ib3 / tot;
                        i.y += ny * err * ib3 / tot;
                    }
                    let pav = self.inst(j.a).b.physics.as_ref().map(|p| (p.vx, p.vy));
                    let pbv = j.b.and_then(|b| self.inst(b).b.physics.as_ref().map(|p| (p.vx, p.vy)));
                    let rv = (pav.map(|p| p.0).unwrap_or(0.0) - pbv.map(|p| p.0).unwrap_or(0.0)) * nx + (pav.map(|p| p.1).unwrap_or(0.0) - pbv.map(|p| p.1).unwrap_or(0.0)) * ny;
                    if rv > 0.0 || j.rigid {
                        let jj = rv / tot;
                        if let Some(p) = self.inst_mut(j.a).b.physics.as_mut() {
                            p.vx -= jj * nx * ia3;
                            p.vy -= jj * ny * ia3;
                        }
                        if let Some(b) = j.b {
                            if let Some(p) = self.inst_mut(b).b.physics.as_mut() {
                                p.vx += jj * nx * ib3;
                                p.vy += jj * ny * ib3;
                            }
                        }
                    }
                }
            }
        }
        let (lw, lh) = (self.lw(), self.lh());
        for &s in &bl {
            let bb = self.box_i(s);
            let roped = self.joints.iter().any(|j| j.a == s || j.b == Some(s));
            let i = self.inst_mut(s);
            let ph = i.b.physics.as_mut().unwrap();
            if bb.l < 0.0 {
                i.x -= bb.l;
                ph.vx = ph.vx.abs() * ph.bounce;
            } else if bb.r > lw {
                i.x -= bb.r - lw;
                ph.vx = -ph.vx.abs() * ph.bounce;
            }
            if bb.t > lh + 400.0 && !roped {
                i.dead = true;
            }
            if ph.ground && ph.vx.abs() < 4.0 {
                ph.vx = 0.0;
            }
            if ph.circle && !ph.fixed {
                let r = i.w.min(i.h) / 2.0;
                if ph.ground || ph.vx.abs() > 1.0 {
                    i.angle = (i.angle + ph.vx * dt / r.max(1.0) * 180.0 / std::f64::consts::PI) % 360.0;
                }
            }
        }
    }

    fn ledge_ok(&self, s: Slot, dir: f64) -> bool {
        let b = self.box_i(s);
        let x = if dir > 0.0 { b.r + 2.0 } else { b.l - 2.0 };
        let drop = if self.poly_count > 0 { (b.r - b.l) / 2.0 * 1.25 + 8.0 } else { 4.0 };
        let mut dy = 2.0;
        while dy <= drop {
            if self.point_solid(x, b.b + dy) {
                return true;
            }
            dy += 2.0;
        }
        false
    }
    pub fn patrol(&mut self, s: Slot, dt: f64) {
        let mut st = self.inst_mut(s).b.patrol.take().unwrap();
        st.vy = (st.vy + st.gravity * dt).min(1200.0);
        let mut tgt = None;
        let mut spd = st.speed;
        let mut mv = st.dir;
        let b0 = self.box_i(s);
        let cx = (b0.l + b0.r) / 2.0;
        let cy = (b0.t + b0.b) / 2.0;
        let was = st.chasing;
        st.chasing = false;
        if !st.chase.is_empty() {
            if let Some(&tt) = self.by_name.get(&st.chase) {
                let mut bd = 1e9;
                for &o in &self.types[tt].insts {
                    let oi = self.inst(o);
                    if oi.dead || o == s || !oi.visible {
                        continue;
                    }
                    let ob = self.box_i(o);
                    let d = ((ob.l + ob.r) / 2.0 - cx).hypot((ob.t + ob.b) / 2.0 - cy);
                    if d < bd {
                        bd = d;
                        tgt = Some(o);
                    }
                }
                if tgt.is_some() && bd > st.range * if was { 1.25 } else { 1.0 } {
                    tgt = None;
                }
            }
        }
        if let Some(tg) = tgt {
            st.chasing = true;
            if !was {
                st.spot = self.tick;
            }
            let tb = self.box_i(tg);
            let dx = (tb.l + tb.r) / 2.0 - cx;
            if dx.abs() > 6.0 {
                st.dir = if dx > 0.0 { 1.0 } else { -1.0 };
            }
            mv = if dx.abs() > 6.0 { st.dir } else { 0.0 };
            spd = st.chase_speed;
            if st.floor.is_some() && mv != 0.0 && !self.ledge_ok(s, mv) && !(tb.t > b0.b + 8.0) {
                if st.jump > 0.0 {
                    st.vy = -st.jump;
                    st.floor = None;
                } else {
                    mv = 0.0;
                }
            }
        }
        let pg = st.floor.is_some();
        if mv != 0.0 && self.move_x(s, mv * spd * dt, pg) != 0 {
            if tgt.is_some() {
                if pg && st.jump > 0.0 && st.vy >= 0.0 {
                    st.vy = -st.jump;
                    st.floor = None;
                }
            } else {
                st.dir *= -1.0;
            }
        }
        let f = self.move_y(s, st.vy * dt);
        if st.vy > 0.0 {
            st.floor = f;
            if f.is_some() {
                st.vy = 0.0;
            }
        } else if f.is_some() {
            st.vy = 0.0;
        } else if st.vy < 0.0 {
            st.floor = None;
        }
        if st.floor.is_none() && pg && st.vy >= 0.0 {
            let bx = self.box_i(s);
            let gs = self.ground_snap(s, climb_of(&bx, spd * dt));
            if gs.is_some() {
                st.floor = gs;
                st.vy = 0.0;
            }
        }
        if tgt.is_none() && st.floor.is_some() && st.ledges && !self.ledge_ok(s, st.dir) {
            st.dir *= -1.0;
        }
        let sprite = self.def(s).kind == Kind::Sprite;
        let i = self.inst_mut(s);
        if st.flip && sprite {
            i.mirrored = st.dir < 0.0;
        }
        i.b.patrol = Some(st);
    }

    pub fn animate(&mut self, s: Slot, dt: f64) {
        let tick = self.tick;
        let d = self.def(s);
        if d.kind != Kind::Sprite {
            return;
        }
        let i = self.inst_mut(s);
        let a = match d.anims.get(i.anim) {
            Some(a) => a,
            None => return,
        };
        if a.frames.len() < 2 || !(a.speed > 0.0) {
            return;
        }
        i.ft += dt * a.speed;
        while i.ft >= 1.0 {
            i.ft -= 1.0;
            i.frame += 1;
            if i.frame >= a.frames.len() {
                if a.looping {
                    i.frame = 0;
                } else {
                    i.frame = a.frames.len() - 1;
                    i.ft = 0.0;
                    if i.anim_end != tick {
                        i.anim_end = tick;
                    }
                    break;
                }
            }
        }
    }

    pub fn tprop(i: &Inst, pr: &str) -> f64 {
        match pr {
            "x" => i.x,
            "y" => i.y,
            "w" => i.w,
            "h" => i.h,
            "angle" => i.angle,
            "opacity" => i.opacity * 100.0,
            _ => 0.0,
        }
    }
    pub fn tset(i: &mut Inst, pr: &str, v: f64) {
        match pr {
            "x" => i.x = v,
            "y" => i.y = v,
            "w" => i.w = v,
            "h" => i.h = v,
            "angle" => i.angle = v,
            "opacity" => i.opacity = (v / 100.0).clamp(0.0, 1.0),
            _ => {}
        }
        if pr == "x" || pr == "y" {
            let (x, y) = (i.x, i.y);
            if let Some(sn) = i.b.sine.as_mut() {
                sn.bx = x;
                sn.by = y;
            }
        }
    }

    pub fn behave(&mut self, s: Slot, dt: f64) {
        let tick = self.tick;
        let lock = self.cine_lock();
        if self.inst(s).ps.is_some() {
            self.ps_step(s, dt);
        }
        if self.inst(s).rd.is_some() {
            self.rd_step(s, dt);
        }
        // 8 directions
        if self.inst(s).b.eightdir.as_ref().map(|e| e.en).unwrap_or(false) {
            let mut st = self.inst_mut(s).b.eightdir.take().unwrap();
            let c = st.controls && !lock;
            let sm = st.sim.clone();
            let mut ix = ((sm.right || (c && (self.key("ArrowRight") || self.key("KeyD")))) as i32 - (sm.left || (c && (self.key("ArrowLeft") || self.key("KeyA")))) as i32) as f64;
            let mut iy = ((sm.down || (c && (self.key("ArrowDown") || self.key("KeyS")))) as i32 - (sm.up || (c && (self.key("ArrowUp") || self.key("KeyW")))) as i32) as f64;
            if st.dirs == "h" {
                iy = 0.0;
            } else if st.dirs == "v" {
                ix = 0.0;
            } else if st.dirs == "4" && ix != 0.0 && iy != 0.0 {
                iy = 0.0;
            }
            let l = ix.hypot(iy);
            if l != 0.0 {
                ix /= l;
                iy /= l;
            }
            let tx = ix * st.max_speed;
            let ty = iy * st.max_speed;
            let ax = (if l != 0.0 { st.accel } else { st.decel }) * dt;
            let dvx = tx - st.vx;
            let dvy = ty - st.vy;
            let dv = dvx.hypot(dvy);
            if dv <= ax {
                st.vx = tx;
                st.vy = ty;
            } else {
                st.vx += dvx / dv * ax;
                st.vy += dvy / dv * ax;
            }
            let h = self.slide(s, st.vx * dt, st.vy * dt);
            if h.0 {
                st.vx = 0.0;
            }
            if h.1 {
                st.vy = 0.0;
            }
            if st.rotate && st.vx.hypot(st.vy) > 1.0 {
                self.inst_mut(s).angle = st.vy.atan2(st.vx) * 180.0 / std::f64::consts::PI;
            }
            st.sim = Sim::default();
            self.inst_mut(s).b.eightdir = Some(st);
        }
        // voiture
        if self.inst(s).b.car.as_ref().map(|e| e.en).unwrap_or(false) {
            let mut st = self.inst_mut(s).b.car.take().unwrap();
            let cc = st.ctl && !lock;
            let cs = st.sim.clone();
            let g = cs.gas || (cc && (self.key("ArrowUp") || self.key("KeyW")));
            let br = cs.brake || (cc && (self.key("ArrowDown") || self.key("KeyS")));
            let tl = cs.left || (cc && (self.key("ArrowLeft") || self.key("KeyA")));
            let trr = cs.right || (cc && (self.key("ArrowRight") || self.key("KeyD")));
            if g && !br {
                st.sp += (if st.sp < 0.0 { st.brake } else { st.accel }) * dt;
            } else if br && !g {
                st.sp -= (if st.sp > 0.0 { st.brake } else { st.accel * 0.6 }) * dt;
            } else {
                st.sp = approach(st.sp, 0.0, st.friction * dt);
            }
            st.sp = st.sp.min(st.max_speed).max(-st.reverse);
            let tdir = (trr as i32 - tl as i32) as f64;
            if tdir != 0.0 && st.sp.abs() > 2.0 {
                let a0 = self.inst(s).angle;
                let f = (st.sp.abs() / (st.max_speed.max(1.0) * 0.3)).min(1.0);
                // l'état voiture est retiré : on le remet le temps de tester le blocage (colBox tourné)
                self.inst_mut(s).angle = a0 + tdir * st.steer * f * (if st.sp < 0.0 { -1.0 } else { 1.0 }) * dt;
                self.inst_mut(s).b.car = Some(st.clone());
                if self.blocked_any(s) {
                    self.inst_mut(s).angle = a0;
                }
                self.inst_mut(s).b.car = None;
            }
            {
                let i = self.inst_mut(s);
                i.angle = (i.angle % 360.0 + 360.0) % 360.0;
            }
            let ra = self.inst(s).angle * std::f64::consts::PI / 180.0;
            let hx = ra.cos();
            let hy = ra.sin();
            let grip = ((1.0 - st.drift) * 14.0 * dt + 0.02).min(1.0);
            st.vx += (hx * st.sp - st.vx) * grip;
            st.vy += (hy * st.sp - st.vy) * grip;
            let spd = st.vx.hypot(st.vy);
            self.inst_mut(s).b.car = Some(st.clone());
            let hc = self.slide(s, st.vx * dt, st.vy * dt);
            if st.ctl && st.engine {
                let vf = st.sp.abs() / st.max_speed.max(1.0);
                let mut slip = (st.vy.atan2(st.vx) - ra).abs();
                if slip > std::f64::consts::PI {
                    slip = 2.0 * std::f64::consts::PI - slip;
                }
                let uid = self.inst(s).uid;
                let screech = if spd > 120.0 { (slip * 2.5).min(1.0) * (spd / st.max_speed) } else { 0.0 };
                self.eng_set(uid, vf, if g { 1.0 } else { 0.0 }, screech, 0.0);
            }
            if hc.0 || hc.1 {
                if spd > 20.0 && st.hit != tick - 1 {
                    st.hit = tick;
                    st.hit_v = spd;
                }
                if hc.0 {
                    st.vx *= -0.25;
                }
                if hc.1 {
                    st.vy *= -0.25;
                }
                st.sp *= -0.25;
            }
            st.sim = Sim::default();
            self.inst_mut(s).b.car = Some(st);
        }
        // glisser-déposer
        if self.inst(s).b.drag.as_ref().map(|d| d.en && d.on).unwrap_or(false) {
            if !self.mouse.down.contains("left") {
                let d = self.inst_mut(s).b.drag.as_mut().unwrap();
                d.on = false;
                d.dr = tick;
            } else {
                let mw = self.mouse_world(s);
                let i = self.inst_mut(s);
                let d = i.b.drag.as_ref().unwrap().clone();
                if d.axes != "v" {
                    i.x = mw.0 - d.gx;
                }
                if d.axes != "h" {
                    i.y = mw.1 - d.gy;
                }
            }
        }
        // tourelle
        if self.inst(s).b.turret.is_some() {
            let st = self.inst(s).b.turret.clone().unwrap();
            let mut best = None;
            let mut bd = st.range;
            let me = self.center(s);
            for &id in &st.types {
                for &j in &self.types[id].insts {
                    if !self.alive(j) || j == s {
                        continue;
                    }
                    let c2 = self.center(j);
                    let dd = (c2.0 - me.0).hypot(c2.1 - me.1);
                    if dd <= bd {
                        bd = dd;
                        best = Some(j);
                    }
                }
            }
            let mut aimed = true;
            let tc = best.map(|b| self.center(b));
            let i = self.inst_mut(s);
            let st = i.b.turret.as_mut().unwrap();
            st.target = best;
            st.cd = (st.cd - dt).max(0.0);
            if let (Some(tc), true) = (tc, st.rotate) {
                let want = (tc.1 - me.1).atan2(tc.0 - me.0) * 180.0 / std::f64::consts::PI;
                let df = ((want - i.angle) % 360.0 + 540.0) % 360.0 - 180.0;
                let mx = if st.rot_speed > 0.0 { st.rot_speed * dt } else { 360.0 };
                i.angle += df.clamp(-mx, mx);
                aimed = df.abs() <= 12f64.max(mx);
            }
            if best.is_some() && aimed && st.cd <= 0.0 {
                st.fired = tick;
                st.cd = st.rate;
            }
        }
        // interpolations
        if self.inst(s).b.tween.as_ref().map(|t| !t.list.is_empty()).unwrap_or(false) {
            let i = self.inst_mut(s);
            let mut tw = i.b.tween.take().unwrap();
            let mut keep = Vec::new();
            for mut t in std::mem::take(&mut tw.list) {
                t.t += dt;
                let k = (t.t / t.d).min(1.0);
                Runtime::tset(i, &t.prop, t.from + (t.to - t.from) * ease(&t.ease, k));
                if k < 1.0 {
                    keep.push(t);
                    continue;
                }
                tw.done.insert(t.tag.clone(), tick);
                if t.mode == "loop" {
                    t.t = 0.0;
                    keep.push(t);
                    continue;
                }
                if t.mode == "pingpong" {
                    t.t = 0.0;
                    std::mem::swap(&mut t.from, &mut t.to);
                    keep.push(t);
                    continue;
                }
            }
            tw.list = keep;
            i.b.tween = Some(tw);
        }
        // fondu
        if self.inst(s).b.fade.as_ref().map(|f| f.run).unwrap_or(false) {
            let i = self.inst_mut(s);
            let st = i.b.fade.as_mut().unwrap();
            st.t += dt;
            let t1 = st.fade_in;
            let t2 = t1 + st.wait;
            let t3 = t2 + st.fade_out;
            if st.t < t1 {
                i.opacity = st.base * st.t / t1;
            } else if st.t < t2 {
                i.opacity = st.base;
            } else if st.t < t3 {
                i.opacity = st.base * (1.0 - (st.t - t2) / st.fade_out);
            } else {
                i.opacity = if st.fade_out > 0.0 { 0.0 } else { st.base };
                st.run = false;
                st.done = tick;
                if st.destroy {
                    i.kill = true;
                }
            }
        }
    }

    pub fn pin_step(&mut self, s: Slot) {
        let st = self.inst(s).b.pin.clone().unwrap();
        let j = match st.to {
            Some(j) => j,
            None => return,
        };
        if !self.alive(j) {
            let i = self.inst_mut(s);
            i.b.pin.as_mut().unwrap().to = None;
            if st.destroy {
                i.dead = true;
            }
            return;
        }
        let (jm, ja, jx, jy) = {
            let ji = self.inst(j);
            (ji.mirrored, ji.angle, ji.x, ji.y)
        };
        let q = if !st.pt.is_empty() { Some(self.img_point(j, Some(&pf_core::Val::S(st.pt.clone())))) } else { None };
        let i = self.inst_mut(s);
        if st.mirror {
            i.mirrored = if st.mi { !jm } else { jm };
        }
        if st.angle {
            i.angle = ja + st.oa;
        }
        if let Some(q) = q {
            i.x = q.0;
            i.y = q.1;
            return;
        }
        let mut ux = st.ox;
        let mut uy = st.oy;
        if st.mirror && jm {
            ux = -ux;
        }
        if st.angle && ja != 0.0 {
            let r = ja * std::f64::consts::PI / 180.0;
            let (c, sn) = (r.cos(), r.sin());
            let x2 = ux * c - uy * sn;
            uy = ux * sn + uy * c;
            ux = x2;
        }
        i.x = jx + ux;
        i.y = jy + uy;
    }

    pub fn eng_set(&mut self, uid: u32, v: f64, load: f64, screech: f64, rumble: f64) {
        self.engines.insert(uid);
        self.host.audio(AudioCmd::Engine { uid, v, load, screech, rumble });
    }
}
