//! Géométrie et collisions : boîtes, polygones, tuiles, déplacements bloqués par les solides.

use crate::model::*;
use crate::rt::Runtime;

pub fn ovl(a: &BBox, b: &BBox) -> bool {
    a.l < b.r && a.r > b.l && a.t < b.b && a.b > b.t
}
pub fn shrink(b: &BBox, e: f64) -> BBox {
    BBox::new(b.l + e, b.t + e, b.r - e, b.b - e)
}
pub fn box_poly(b: &BBox) -> Poly {
    vec![[b.l, b.t], [b.r, b.t], [b.r, b.b], [b.l, b.b]]
}
pub fn poly_bb(p: &[[f64; 2]]) -> BBox {
    let mut b = BBox::new(f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for q in p {
        b.l = b.l.min(q[0]);
        b.r = b.r.max(q[0]);
        b.t = b.t.min(q[1]);
        b.b = b.b.max(q[1]);
    }
    b
}
pub fn pt_in_poly(x: f64, y: f64, p: &[[f64; 2]]) -> bool {
    let mut c = false;
    let n = p.len();
    if n == 0 {
        return false;
    }
    let mut j = n - 1;
    for k in 0..n {
        let a = p[k];
        let b = p[j];
        if (a[1] > y) != (b[1] > y) && x < (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0] {
            c = !c;
        }
        j = k;
    }
    c
}
fn seg_x(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let d1 = (d[0] - c[0]) * (a[1] - c[1]) - (d[1] - c[1]) * (a[0] - c[0]);
    let d2 = (d[0] - c[0]) * (b[1] - c[1]) - (d[1] - c[1]) * (b[0] - c[0]);
    let d3 = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    let d4 = (b[0] - a[0]) * (d[1] - a[1]) - (b[1] - a[1]) * (d[0] - a[0]);
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}
pub fn poly_ovl(p: &[[f64; 2]], q: &[[f64; 2]]) -> bool {
    for k in 0..p.len() {
        let a = p[k];
        let b = p[(k + 1) % p.len()];
        for m in 0..q.len() {
            if seg_x(a, b, q[m], q[(m + 1) % q.len()]) {
                return true;
            }
        }
    }
    let cp = |a: &[[f64; 2]]| {
        let (mut x, mut y) = (0.0, 0.0);
        for v in a {
            x += v[0];
            y += v[1];
        }
        [x / a.len() as f64, y / a.len() as f64]
    };
    let pc = cp(p);
    let qc = cp(q);
    pt_in_poly(pc[0], pc[1], q) || pt_in_poly(qc[0], qc[1], p) || p.iter().any(|v| pt_in_poly(v[0], v[1], q)) || q.iter().any(|v| pt_in_poly(v[0], v[1], p))
}
pub fn tile_poly(t: &str, c: f64, h: f64) -> Option<Poly> {
    Some(match t {
        "r45" => vec![[0.0, h], [c, 0.0], [c, h]],
        "l45" => vec![[0.0, 0.0], [c, h], [0.0, h]],
        "r1" => vec![[0.0, h], [c, h / 2.0], [c, h]],
        "r2" => vec![[0.0, h / 2.0], [c, 0.0], [c, h], [0.0, h]],
        "l2" => vec![[0.0, 0.0], [c, h / 2.0], [c, h], [0.0, h]],
        "l1" => vec![[0.0, h / 2.0], [c, h], [0.0, h]],
        _ => return None,
    })
}

fn surf_y(p: &[[f64; 2]], x: f64, flat: bool) -> Option<f64> {
    let mut y: Option<f64> = None;
    for k in 0..p.len() {
        let a = p[k];
        let c = p[(k + 1) % p.len()];
        if a[0] == c[0] {
            continue;
        }
        if flat && ((c[1] - a[1]) / (c[0] - a[0])).abs() > 0.25 {
            continue;
        }
        let x0 = a[0].min(c[0]);
        let x1 = a[0].max(c[0]);
        if x < x0 || x > x1 {
            continue;
        }
        let t = (x - a[0]) / (c[0] - a[0]);
        let yy = a[1] + (c[1] - a[1]) * t;
        if y.map(|v| yy < v).unwrap_or(true) {
            y = Some(yy);
        }
    }
    y
}
pub fn ground_y(p: &[[f64; 2]], b: &BBox) -> Option<f64> {
    let cx = (b.l + b.r) / 2.0;
    if let Some(g) = surf_y(p, cx, false) {
        return Some(g);
    }
    let y1 = surf_y(p, b.l + 1.0, true);
    let y2 = surf_y(p, b.r - 1.0, true);
    match (y1, y2) {
        (None, y) => y,
        (y, None) => y,
        (Some(a), Some(b)) => Some(a.min(b)),
    }
}
fn poly_hit(b: &BBox, sh: &Shape) -> bool {
    ovl(b, &sh.b) && poly_ovl(&box_poly(&shrink(b, 0.02)), &sh.p)
}
pub fn climb_of(b: &BBox, dx: f64) -> f64 {
    (b.r - b.l) / 2.0 * 1.25 + dx.abs() * 1.5 + 2.0
}

pub fn col_box(o: &Def, i: &Inst) -> BBox {
    if o.kind == Kind::Particles {
        return BBox::new(i.x - 16.0, i.y - 16.0, i.x + 16.0, i.y + 16.0);
    }
    if o.kind != Kind::Sprite {
        return BBox::new(i.x, i.y, i.x + i.w, i.y + i.h);
    }
    let c = o.cbox.unwrap_or([0.0, 0.0, o.w, o.h]);
    let sx = i.w / o.w;
    let sy = i.h / o.h;
    let mut l = (c[0] - o.ox) * sx;
    let mut r = (c[0] + c[2] - o.ox) * sx;
    if i.mirrored {
        let q = l;
        l = -r;
        r = -q;
    }
    if i.b.car.is_some() && i.angle != 0.0 {
        let t0 = (c[1] - o.oy) * sy;
        let b0 = (c[1] + c[3] - o.oy) * sy;
        let cx = (l + r) / 2.0;
        let cy = (t0 + b0) / 2.0;
        let hw = (r - l) / 2.0;
        let hh = (b0 - t0) / 2.0;
        let ra = i.angle * std::f64::consts::PI / 180.0;
        let cs = ra.cos().abs();
        let sn = ra.sin().abs();
        let k = 1.0 - 0.22 * (2.0 * ra).sin().abs();
        let ex = (cs * hw + sn * hh) * k;
        let ey = (sn * hw + cs * hh) * k;
        let rx = cx * ra.cos() - cy * ra.sin();
        let ry = cx * ra.sin() + cy * ra.cos();
        return BBox::new(i.x + rx - ex, i.y + ry - ey, i.x + rx + ex, i.y + ry + ey);
    }
    BBox::new(i.x + l, i.y + (c[1] - o.oy) * sy, i.x + r, i.y + (c[1] + c[3] - o.oy) * sy)
}

pub fn draw_box(o: &Def, i: &Inst) -> BBox {
    if o.kind == Kind::Particles {
        return BBox::new(i.x - 16.0, i.y - 16.0, i.x + 16.0, i.y + 16.0);
    }
    if o.kind != Kind::Sprite {
        return BBox::new(i.x, i.y, i.x + i.w, i.y + i.h);
    }
    let sx = i.w / o.w;
    let sy = i.h / o.h;
    let mut l = -o.ox * sx;
    let mut r = (o.w - o.ox) * sx;
    if i.mirrored {
        let q = l;
        l = -r;
        r = -q;
    }
    BBox::new(i.x + l, i.y - o.oy * sy, i.x + r, i.y + (o.h - o.oy) * sy)
}

pub fn xform_pt(o: &Def, i: &Inst, px: f64, py: f64) -> [f64; 2] {
    let mut lx = (px - o.ox) * i.w / o.w;
    let mut ly = (py - o.oy) * i.h / o.h;
    if i.mirrored {
        lx = -lx;
    }
    if i.angle != 0.0 {
        let r = i.angle * std::f64::consts::PI / 180.0;
        let (c, s) = (r.cos(), r.sin());
        let x2 = lx * c - ly * s;
        ly = lx * s + ly * c;
        lx = x2;
    }
    [i.x + lx, i.y + ly]
}

pub fn frame_poly<'a>(o: &'a Def, i: &Inst) -> Option<&'a Poly> {
    if o.kind != Kind::Sprite || !o.poly_col {
        return None;
    }
    let a = o.anims.get(i.anim).or_else(|| o.anims.first());
    let a0 = o.anims.first();
    let p = a.and_then(|a| a.poly.get(i.frame).and_then(|x| x.as_ref()).or_else(|| a.poly.first().and_then(|x| x.as_ref()))).or_else(|| a0.and_then(|a| a.poly.first().and_then(|x| x.as_ref())));
    p.filter(|p| p.len() >= 3)
}
pub fn col_poly(o: &Def, i: &Inst) -> Option<Poly> {
    frame_poly(o, i).map(|p| p.iter().map(|q| xform_pt(o, i, q[0], q[1])).collect())
}

pub fn img_point(o: &Def, i: &Inst, nm: Option<&pf_core::Val>) -> (f64, f64) {
    let (mut lx, mut ly) = (0.0, 0.0);
    if o.kind == Kind::Sprite {
        if let Some(nm) = nm {
            let s = nm.str();
            if !s.is_empty() {
                let key = if matches!(nm, pf_core::Val::N(_)) || s.chars().all(|c| c.is_ascii_digit()) {
                    let k = nm.num0() as i64 - 1;
                    if k >= 0 {
                        o.points.get(k as usize).cloned()
                    } else {
                        None
                    }
                } else if o.points.iter().any(|p| *p == s) {
                    Some(s)
                } else {
                    None
                };
                if let Some(key) = key {
                    let a = o.anims.get(i.anim).or_else(|| o.anims.first());
                    let loc = a.and_then(|a| a.pts.get(i.frame).and_then(|m| m.get(&key)).or_else(|| a.pts.first().and_then(|m| m.get(&key)))).copied().unwrap_or([o.ox, o.oy]);
                    lx = (loc[0] - o.ox) * i.w / o.w;
                    ly = (loc[1] - o.oy) * i.h / o.h;
                }
            }
        }
    }
    if i.mirrored {
        lx = -lx;
    }
    if i.angle != 0.0 {
        let r = i.angle * std::f64::consts::PI / 180.0;
        let (c, s) = (r.cos(), r.sin());
        let x2 = lx * c - ly * s;
        ly = lx * s + ly * c;
        lx = x2;
    }
    (i.x + lx, i.y + ly)
}

/// forme de collision : polygone, tuiles, ou boîte
pub enum CShape {
    B(BBox),
    P(Poly, BBox),
    T(BBox, Slot),
}
impl CShape {
    pub fn bb(&self) -> &BBox {
        match self {
            CShape::B(b) | CShape::P(_, b) | CShape::T(b, _) => b,
        }
    }
}

impl Runtime {
    pub fn box_i(&self, s: Slot) -> BBox {
        let i = self.inst(s);
        col_box(&self.types[i.t].def, i)
    }
    pub fn box_r(&self, r: ORef) -> BBox {
        match r {
            ORef::I(s) => self.box_i(s),
            ORef::V(o, k) => self.inst(o).tm.as_ref().and_then(|t| t.vt.all.get(k as usize)).map(|v| v.vb).unwrap_or_default(),
        }
    }
    pub fn draw_box_i(&self, s: Slot) -> BBox {
        let i = self.inst(s);
        draw_box(&self.types[i.t].def, i)
    }
    pub fn ref_dead(&self, r: ORef) -> bool {
        match r {
            ORef::I(s) | ORef::V(s, _) => !self.alive(s),
        }
    }
    pub fn ref_type(&self, r: ORef) -> usize {
        match r {
            ORef::I(s) | ORef::V(s, _) => self.inst(s).t,
        }
    }
    pub fn ref_phys(&self, r: ORef) -> bool {
        match r {
            ORef::I(s) => self.inst(s).b.physics.is_some(),
            _ => false,
        }
    }
    pub fn ref_is_ladder(&self, r: ORef) -> bool {
        self.types[self.ref_type(r)].ladder
    }
    /// sshape : forme polygonale d'un solide (None = simple boîte)
    pub fn sshape(&self, r: ORef) -> Option<Shape> {
        match r {
            ORef::V(o, k) => self.inst(o).tm.as_ref().and_then(|t| t.vt.all.get(k as usize)).and_then(|v| v.sh.clone()),
            ORef::I(s) => {
                let i = self.inst(s);
                let d = &self.types[i.t].def;
                if !d.poly_col || d.kind != Kind::Sprite {
                    return None;
                }
                col_poly(d, i).map(|p| {
                    let b = poly_bb(&p);
                    Shape { p, b }
                })
            }
        }
    }
    pub fn has_sshape(&self, r: ORef) -> bool {
        match r {
            ORef::V(o, k) => self.inst(o).tm.as_ref().and_then(|t| t.vt.all.get(k as usize)).map(|v| v.sh.is_some()).unwrap_or(false),
            ORef::I(s) => {
                let i = self.inst(s);
                let d = &self.types[i.t].def;
                d.poly_col && d.kind == Kind::Sprite && frame_poly(d, i).is_some()
            }
        }
    }
    pub fn img_point(&self, s: Slot, nm: Option<&pf_core::Val>) -> (f64, f64) {
        let i = self.inst(s);
        img_point(&self.types[i.t].def, i, nm)
    }

    // ---------------------------------------------------------------- tuiles

    pub fn tm_cell(&self, s: Slot) -> (f64, f64) {
        let o = &self.types[self.inst(s).t].def;
        (o.j.f_or2("tw", 16.0) * o.scale, o.j.f_or2("th", 16.0) * o.scale)
    }
    /// (re)construit le cache des tuiles solides
    pub fn tm_tiles(&mut self, s: Slot) {
        let (c, h) = self.tm_cell(s);
        let i = self.inst(s);
        let o = self.types[i.t].def.clone();
        let d = i.tm.as_ref().unwrap();
        let key = (i.x, i.y, d.ver, d.cols, d.rows, c, h);
        if d.vk == Some(key) {
            return;
        }
        let (ix, iy) = (i.x, i.y);
        let (cols, rows) = (d.cols, d.rows);
        let data = d.data.clone();
        let tc = o.j.get("tcol").cloned().unwrap_or(pf_core::Json::Null);
        let tw = o.j.f_or2("tw", 16.0);
        let th = o.j.f_or2("th", 16.0);
        let mut out = TileOut::default();
        for r in 0..rows {
            let mut run: Option<(bool, i64)> = None; // (solide ?, colonne de départ)
            for cc in 0..=cols {
                let v = if cc < cols { data[(r * cols + cc) as usize] } else { -1 };
                let ty: Option<pf_core::Json> = if v < 0 {
                    None
                } else {
                    Some(tc.get(v.to_string()).cloned().unwrap_or_else(|| pf_core::Json::String("s".into())))
                };
                let tys = ty.as_ref().and_then(|t| t.as_str()).map(|s| s.to_string());
                let rk = match tys.as_deref() {
                    Some("s") => Some(true),
                    Some("j") => Some(false),
                    _ => None,
                };
                if let Some(rn) = run {
                    if Some(rn.0) != rk {
                        let vs = VirtSolid { vb: BBox::new(ix + rn.1 as f64 * c, iy + r as f64 * h, ix + cc as f64 * c, iy + (r + 1) as f64 * h), sh: None };
                        let k = out.all.len() as u32;
                        if rn.0 {
                            out.rect.push(k);
                        } else {
                            out.jt.push(k);
                        }
                        out.all.push(vs);
                        run = None;
                    }
                }
                if let Some(rk) = rk {
                    if run.is_none() {
                        run = Some((rk, cc));
                    }
                }
                if let Some(ty) = &ty {
                    if rk.is_none() && tys.as_deref() != Some("n") {
                        let pp: Option<Poly> = if ty.is_object() {
                            let pts = ty.get("p").and_then(|p| p.as_array()).filter(|a| a.len() >= 3);
                            pts.map(|a| a.iter().map(|q| [q[0].as_f64().unwrap_or(0.0) * c / tw, q[1].as_f64().unwrap_or(0.0) * h / th]).collect())
                        } else {
                            tile_poly(tys.as_deref().unwrap_or(""), c, h)
                        };
                        if let Some(pp) = pp {
                            let pq: Poly = pp.iter().map(|q| [ix + cc as f64 * c + q[0], iy + r as f64 * h + q[1]]).collect();
                            let bb = poly_bb(&pq);
                            let k = out.all.len() as u32;
                            out.poly.push(k);
                            out.all.push(VirtSolid { vb: bb, sh: Some(Shape { p: pq, b: bb }) });
                        }
                    }
                }
            }
        }
        let d = self.inst_mut(s).tm.as_mut().unwrap();
        d.vk = Some(key);
        d.vt = out;
    }
    pub fn tm_get(&self, s: Slot, c: f64, r: f64) -> i64 {
        let d = match &self.inst(s).tm {
            Some(d) => d,
            None => return -1,
        };
        let (c, r) = (c.floor(), r.floor());
        if c < 0.0 || r < 0.0 || c >= d.cols as f64 || r >= d.rows as f64 {
            return -1;
        }
        d.data[(r as i64 * d.cols + c as i64) as usize]
    }
    pub fn tm_set(&mut self, s: Slot, c: f64, r: f64, v: f64) {
        let d = match self.inst_mut(s).tm.as_mut() {
            Some(d) => d,
            None => return,
        };
        let (c, r) = (c.floor(), r.floor());
        if c < 0.0 || r < 0.0 || c >= d.cols as f64 || r >= d.rows as f64 {
            return;
        }
        let v = v.floor();
        d.data[(r as i64 * d.cols + c as i64) as usize] = if v.is_nan() || v < 0.0 { -1 } else { v as i64 };
        d.ver += 1;
    }
    pub fn tm_col(&self, s: Slot, x: f64) -> f64 {
        ((x - self.inst(s).x) / self.tm_cell(s).0).floor()
    }
    pub fn tm_row(&self, s: Slot, y: f64) -> f64 {
        ((y - self.inst(s).y) / self.tm_cell(s).1).floor()
    }

    /// forme pour les conditions de collision / chevauchement
    pub fn shape(&mut self, s: Slot) -> CShape {
        if self.inst(s).tm.is_some() {
            self.tm_tiles(s);
            return CShape::T(self.box_i(s), s);
        }
        let i = self.inst(s);
        let d = &self.types[i.t].def;
        match col_poly(d, i) {
            Some(p) => {
                let b = poly_bb(&p);
                CShape::P(p, b)
            }
            None => CShape::B(col_box(d, i)),
        }
    }
    pub fn hits(&self, a: &CShape, b: &CShape) -> bool {
        if !ovl(a.bb(), b.bb()) {
            return false;
        }
        if let CShape::T(_, ts) = a {
            return self.tiles_hit(*ts, b);
        }
        if let CShape::T(_, ts) = b {
            return self.tiles_hit(*ts, a);
        }
        match (a, b) {
            (CShape::B(_), CShape::B(_)) => true,
            _ => {
                let pa = match a {
                    CShape::P(p, _) => p.clone(),
                    _ => box_poly(a.bb()),
                };
                let pb = match b {
                    CShape::P(p, _) => p.clone(),
                    _ => box_poly(b.bb()),
                };
                poly_ovl(&pa, &pb)
            }
        }
    }
    fn tiles_hit(&self, ts: Slot, o: &CShape) -> bool {
        let d = match &self.inst(ts).tm {
            Some(d) => d,
            None => return false,
        };
        d.vt.all.iter().any(|v| {
            let vs = match &v.sh {
                Some(sh) => CShape::P(sh.p.clone(), sh.b),
                None => CShape::B(v.vb),
            };
            self.hits(o, &vs)
        })
    }

    // ---------------------------------------------------------------- déplacements

    fn poly_block(&self, s: Slot, lift: f64) -> Option<ORef> {
        let mut b = self.box_i(s);
        if lift != 0.0 {
            b.b -= lift;
        }
        if b.b <= b.t {
            return None;
        }
        for &so in &self.solids {
            if so == ORef::I(s) || self.ref_dead(so) {
                continue;
            }
            if let Some(sh) = self.sshape(so) {
                if poly_hit(&b, &sh) {
                    return Some(so);
                }
            }
        }
        None
    }
    fn ovl0(&self, a: &BBox, so: ORef) -> bool {
        if !self.ref_phys(so) {
            return false;
        }
        let q = self.box_r(so);
        a.r > q.l + 2.0 && a.l < q.r - 2.0 && a.b > q.t + 2.0 && a.t < q.b - 2.0
    }
    pub fn move_x(&mut self, s: Slot, dx: f64, grounded: bool) -> i32 {
        self.last_blk = None;
        if dx == 0.0 {
            return 0;
        }
        let x0 = self.inst(s).x;
        let pre = self.poly_count > 0 && !grounded && self.poly_block(s, 0.0).is_some();
        let b0 = self.box_i(s);
        self.inst_mut(s).x += dx;
        let mut b = self.box_i(s);
        let mut hit = 0;
        let step_up = if grounded { if self.poly_count > 0 { climb_of(&b, dx) } else { 4.0 } } else { 0.0 };
        let sols = self.solids.clone();
        let iphys = self.inst(s).b.physics.is_some();
        for so in sols {
            if so == ORef::I(s) || self.ref_dead(so) || self.has_sshape(so) || self.ovl0(&b0, so) {
                continue;
            }
            let sb = self.box_r(so);
            let sphys = self.ref_phys(so);
            let phy = iphys || sphys;
            let tol = if phy { 1.5 } else { 0.01 };
            if phy && (if dx > 0.0 { sb.l + sb.r < b0.l + b0.r } else { sb.l + sb.r > b0.l + b0.r }) {
                continue;
            }
            if b.r > sb.l && b.l < sb.r && b.b > sb.t + tol && b.t < sb.b - tol {
                if step_up != 0.0 && b.b - sb.t <= (if sphys { 4.0 } else { step_up }) {
                    let oy = self.inst(s).y;
                    self.inst_mut(s).y -= b.b - sb.t;
                    if !self.blocked_any(s) {
                        b = self.box_i(s);
                        self.inst_mut(s).sf = Some(so);
                        continue;
                    }
                    self.inst_mut(s).y = oy;
                }
                {
                    let i = self.inst_mut(s);
                    if dx > 0.0 {
                        i.x -= b.r - sb.l;
                    } else {
                        i.x += sb.r - b.l;
                    }
                    if (i.x - x0) * dx < 0.0 {
                        i.x = x0;
                    }
                }
                b = self.box_i(s);
                hit = if dx > 0.0 { 1 } else { -1 };
                self.last_blk = Some(so);
            }
        }
        if self.poly_count == 0 {
            return hit;
        }
        let climb = if grounded || pre { climb_of(&b, dx) } else { 0.0 };
        if self.poly_block(s, climb).is_some() {
            let (mut lo, mut hi) = (0.0, self.inst(s).x - x0);
            let base = x0;
            for _ in 0..10 {
                let mid = (lo + hi) / 2.0;
                self.inst_mut(s).x = base + mid;
                if self.poly_block(s, climb).is_some() {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            self.inst_mut(s).x = base + lo;
            hit = if dx > 0.0 { 1 } else { -1 };
        }
        if grounded {
            b = self.box_i(s);
            let mut best: Option<f64> = None;
            let mut bs = None;
            for &so in &self.solids {
                if so == ORef::I(s) || self.ref_dead(so) {
                    continue;
                }
                let sh = match self.sshape(so) {
                    Some(sh) => sh,
                    None => continue,
                };
                if sh.b.r < b.l || sh.b.l > b.r {
                    continue;
                }
                if let Some(g) = ground_y(&sh.p, &b) {
                    if g < b.b && b.b - g <= climb && best.map(|v| g < v).unwrap_or(true) {
                        best = Some(g);
                        bs = Some(so);
                    }
                }
            }
            if let (Some(bs), Some(best)) = (bs, best) {
                let i = self.inst_mut(s);
                i.y -= b.b - best;
                i.sf = Some(bs);
            }
        }
        hit
    }
    pub fn move_y(&mut self, s: Slot, dy: f64) -> Option<ORef> {
        if dy == 0.0 {
            return None;
        }
        let b0 = self.box_i(s);
        let pb = b0.b;
        let y0 = self.inst(s).y;
        self.inst_mut(s).y += dy;
        let mut b = self.box_i(s);
        let mut res = None;
        let sols = self.solids.clone();
        if dy > 0.0 {
            for so in sols {
                if so == ORef::I(s) || self.ref_dead(so) {
                    continue;
                }
                if self.ref_phys(so) && pb > self.box_r(so).t + 3f64.max(dy + 1.0) {
                    continue;
                }
                if let Some(sh) = self.sshape(so) {
                    if !ovl(&BBox::new(b.l, b.t, b.r, b.b + 0.01), &sh.b) {
                        continue;
                    }
                    if let Some(g) = ground_y(&sh.p, &b) {
                        if b.b >= g - 0.001 && pb <= g + 2.0 {
                            self.inst_mut(s).y -= b.b - g;
                            b = self.box_i(s);
                            res = Some(so);
                        }
                    }
                    continue;
                }
                let sb = self.box_r(so);
                if b.r > sb.l + 0.01 && b.l < sb.r - 0.01 && b.b > sb.t && b.t < sb.b {
                    self.inst_mut(s).y -= b.b - sb.t;
                    b = self.box_i(s);
                    res = Some(so);
                }
            }
            let nojt = self.inst(s).nojt;
            for so in self.jts.clone() {
                if so == ORef::I(s) || self.ref_dead(so) || (nojt && self.ref_is_ladder(so)) {
                    continue;
                }
                let sb = self.box_r(so);
                if b.r > sb.l + 0.01 && b.l < sb.r - 0.01 && pb <= sb.t + 0.5 && b.b >= sb.t {
                    self.inst_mut(s).y -= b.b - sb.t;
                    b = self.box_i(s);
                    res = Some(so);
                }
            }
        } else {
            for so in sols {
                if so == ORef::I(s) || self.ref_dead(so) || self.has_sshape(so) {
                    continue;
                }
                if self.ref_phys(so) && b0.t < self.box_r(so).b - 3f64.max(1.0 - dy) {
                    continue;
                }
                let sb = self.box_r(so);
                if b.r > sb.l + 0.01 && b.l < sb.r - 0.01 && b.t < sb.b && b.b > sb.t {
                    self.inst_mut(s).y += sb.b - b.t;
                    b = self.box_i(s);
                    res = Some(so);
                }
            }
            if self.poly_count > 0 {
                let c0 = climb_of(&b, 0.0);
                if let Some(hs) = self.poly_block(s, c0) {
                    let (mut lo, mut hi) = (0.0, self.inst(s).y - y0);
                    for _ in 0..10 {
                        let mid = (lo + hi) / 2.0;
                        self.inst_mut(s).y = y0 + mid;
                        if self.poly_block(s, c0).is_some() {
                            hi = mid;
                        } else {
                            lo = mid;
                        }
                    }
                    self.inst_mut(s).y = y0 + lo;
                    res = Some(hs);
                }
            }
        }
        res
    }
    pub fn ground_snap(&mut self, s: Slot, maxd: f64) -> Option<ORef> {
        if self.poly_count == 0 {
            return None;
        }
        let b = self.box_i(s);
        let mut best: Option<f64> = None;
        let mut bs = None;
        for &so in &self.solids {
            if so == ORef::I(s) || self.ref_dead(so) {
                continue;
            }
            let sh = match self.sshape(so) {
                Some(sh) => sh,
                None => continue,
            };
            if sh.b.r < b.l || sh.b.l > b.r {
                continue;
            }
            if let Some(g) = ground_y(&sh.p, &b) {
                if g >= b.b - 0.001 && g - b.b <= maxd && best.map(|v| g < v).unwrap_or(true) {
                    best = Some(g);
                    bs = Some(so);
                }
            }
        }
        if let Some(best) = best {
            self.inst_mut(s).y += best - b.b;
        }
        bs
    }
    pub fn blocked_any(&self, s: Slot) -> bool {
        let b = shrink(&self.box_i(s), 0.02);
        for &so in &self.solids {
            if so == ORef::I(s) || self.ref_dead(so) {
                continue;
            }
            let hit = match self.sshape(so) {
                Some(sh) => poly_hit(&b, &sh),
                None => ovl(&b, &self.box_r(so)),
            };
            if hit {
                return true;
            }
        }
        false
    }
    pub fn slide(&mut self, s: Slot, dx: f64, dy: f64) -> (bool, bool) {
        let mut hx = false;
        let mut hy = false;
        for (axis, d) in [(0, dx), (1, dy)] {
            if d == 0.0 {
                continue;
            }
            let get = |rt: &Runtime| if axis == 0 { rt.inst(s).x } else { rt.inst(s).y };
            let set = |rt: &mut Runtime, v: f64| {
                if axis == 0 {
                    rt.inst_mut(s).x = v
                } else {
                    rt.inst_mut(s).y = v
                }
            };
            let o = get(self);
            set(self, o + d);
            if self.blocked_any(s) {
                let (mut lo, mut hi) = (0.0, d);
                for _ in 0..10 {
                    let m = (lo + hi) / 2.0;
                    set(self, o + m);
                    if self.blocked_any(s) {
                        hi = m;
                    } else {
                        lo = m;
                    }
                }
                set(self, o + lo);
                if axis == 0 {
                    hx = true;
                } else {
                    hy = true;
                }
            }
        }
        (hx, hy)
    }
    pub fn point_solid(&self, x: f64, y: f64) -> bool {
        for &so in &self.solids {
            if self.ref_dead(so) {
                continue;
            }
            if let Some(sh) = self.sshape(so) {
                if x >= sh.b.l && x <= sh.b.r && y >= sh.b.t && y <= sh.b.b && pt_in_poly(x, y, &sh.p) {
                    return true;
                }
                continue;
            }
            let b = self.box_r(so);
            if x >= b.l && x < b.r && y >= b.t && y < b.b {
                return true;
            }
        }
        for &so in &self.jts {
            if self.ref_dead(so) {
                continue;
            }
            let b = self.box_r(so);
            if x >= b.l && x < b.r && y >= b.t && y < b.b {
                return true;
            }
        }
        false
    }
    pub fn center(&self, s: Slot) -> (f64, f64) {
        let b = self.box_i(s);
        ((b.l + b.r) / 2.0, (b.t + b.b) / 2.0)
    }
    fn ray_free(&self, a: (f64, f64), b: (f64, f64), me: Slot, other: Slot) -> bool {
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let l = dx.hypot(dy);
        let n = (l / 6.0).ceil() as i64;
        for k in 1..n {
            let x = a.0 + dx * k as f64 / n as f64;
            let y = a.1 + dy * k as f64 / n as f64;
            for &so in &self.solids {
                if so == ORef::I(me) || so == ORef::I(other) || self.ref_dead(so) {
                    continue;
                }
                if let Some(sh) = self.sshape(so) {
                    if x >= sh.b.l && x <= sh.b.r && y >= sh.b.t && y <= sh.b.b && pt_in_poly(x, y, &sh.p) {
                        return false;
                    }
                } else {
                    let q = self.box_r(so);
                    if x >= q.l && x < q.r && y >= q.t && y < q.b {
                        return false;
                    }
                }
            }
        }
        true
    }
    pub fn can_see(&self, i: Slot, j: Slot) -> bool {
        let st = match &self.inst(i).b.los {
            Some(s) => s.clone(),
            None => return false,
        };
        let a = self.center(i);
        let b = self.center(j);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        if dx.hypot(dy) > st.range {
            return false;
        }
        if st.cone < 360.0 {
            let ii = self.inst(i);
            let face = if ii.mirrored { 180.0 - ii.angle } else { ii.angle };
            let ang = dy.atan2(dx) * 180.0 / std::f64::consts::PI;
            let df = ((ang - face) % 360.0 + 540.0) % 360.0 - 180.0;
            if df.abs() > st.cone / 2.0 {
                return false;
            }
        }
        self.ray_free(a, b, i, j)
    }
    pub fn mouse_world(&self, s: Slot) -> (f64, f64) {
        let i = self.inst(s);
        let (px, py) = self.layer_pp(i.layer);
        let z = self.zf(i.layer, i.hud);
        (self.mouse.sx / z + if i.hud { 0.0 } else { self.cam.0 * px }, self.mouse.sy / z + if i.hud { 0.0 } else { self.cam.1 * py })
    }
    pub fn mouse_over(&self, s: Slot) -> bool {
        let i = self.inst(s);
        let (px, py) = self.layer_pp(i.layer);
        let z = self.zf(i.layer, i.hud);
        let x = self.mouse.sx / z + if i.hud { 0.0 } else { self.cam.0 * px };
        let y = self.mouse.sy / z + if i.hud { 0.0 } else { self.cam.1 * py };
        let d = &self.types[i.t].def;
        let b = draw_box(d, i);
        let lv = self.layers.get(i.layer).map(|l| l.visible).unwrap_or(true);
        if !(i.visible && lv && x >= b.l && x <= b.r && y >= b.t && y <= b.b) {
            return false;
        }
        match col_poly(d, i) {
            Some(p) => pt_in_poly(x, y, &p),
            None => true,
        }
    }
}

/// `o.k||d` pour la lecture JSON
pub trait JF {
    fn f_or2(&self, k: &str, d: f64) -> f64;
}
impl JF for pf_core::Json {
    fn f_or2(&self, k: &str, d: f64) -> f64 {
        use pf_core::JsonExt;
        self.f_or(k, d)
    }
}
