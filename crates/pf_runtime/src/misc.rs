//! Boutons, barres, classements, dialogues et particules.

use crate::gfx::parse_color;
use crate::model::*;
use crate::rt::*;
use pf_core::{Json, JsonExt};

const CHARS: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.!";

impl Runtime {
    // ---------------------------------------------------------------- barres
    pub fn bar_set(&mut self, s: Slot, v: f64) {
        let b = match self.inst_mut(s).bar.as_mut() {
            Some(b) => b,
            None => return,
        };
        let v = v.clamp(0.0, b.max);
        if v < b.v {
            b.gt = 0.45;
            if b.ghost < b.v {
                b.ghost = b.v;
            }
        }
        b.v = v;
    }
    pub fn bar_step(&mut self, s: Slot, dt: f64) {
        let sm = self.def(s).j.get("smooth") != Some(&Json::Bool(false));
        let b = self.inst_mut(s).bar.as_mut().unwrap();
        b.shown = if sm { b.shown + (b.v - b.shown) * (dt * 12.0).min(1.0) } else { b.v };
        if b.gt > 0.0 {
            b.gt -= dt;
        } else {
            b.ghost += (b.shown - b.ghost) * (dt * 6.0).min(1.0);
        }
        if b.ghost < b.shown {
            b.ghost = b.shown;
        }
    }

    // ---------------------------------------------------------------- classements
    pub fn hs_list(&self, t: &str) -> Vec<HsEntry> {
        let t = if t.is_empty() { "scores" } else { t };
        match self.sget(&format!("hs:{}", t)) {
            Some(Json::Array(a)) => a.iter().map(|e| HsEntry { n: crate::rt::jstr(e.get("n").unwrap_or(&Json::Null)), s: e.f("s", 0.0), d: e.f("d", 0.0) }).collect(),
            _ => Vec::new(),
        }
    }
    pub fn hs_fits(l: &[HsEntry], s: f64, mx: usize, asc: bool) -> bool {
        if l.len() < mx {
            return true;
        }
        let w = &l[mx - 1];
        if asc {
            s < w.s
        } else {
            s > w.s
        }
    }
    pub fn hs_refresh(&mut self) {
        for s in self.all.clone() {
            if self.def(s).kind == Kind::Leaderboard {
                let t = self.inst(s).lbt.clone();
                let l = self.hs_list(&t);
                let hi = self.hs_last.as_ref().filter(|h| h.0 == t).map(|h| h.1);
                let i = self.inst_mut(s);
                i.lb = l;
                i.lb_hi = hi;
            }
        }
    }
    fn hs_confirm(&mut self) {
        let q = match &self.hs_in {
            Some(q) => q.clone(),
            None => return,
        };
        let name: String = q.name.iter().collect();
        let v = if !name.trim().is_empty() {
            name.trim().to_string()
        } else if !q.def.trim().is_empty() {
            q.def.trim().to_string()
        } else {
            "Joueur".to_string()
        };
        self.hs_name = v.clone();
        self.sset("hs:name", Json::String(v));
        self.hs_done = true;
    }
    fn hs_cycle(&mut self, d: i32) {
        let q = self.hs_in.as_mut().unwrap();
        let chars: Vec<char> = CHARS.chars().collect();
        let ch = q.name.get(q.cur).copied().unwrap_or(' ');
        let up = ch.to_uppercase().next().unwrap_or(ch);
        let k = chars.iter().position(|&c| c == up);
        let n = chars.len() as i32;
        let k = match k {
            None => 0,
            Some(k) => ((k as i32 + d + n) % n) as usize,
        };
        let nc = chars[k];
        if q.cur >= q.name.len() {
            if q.name.len() >= q.max {
                return;
            }
            q.name.push(nc);
        } else {
            q.name[q.cur] = nc;
        }
    }
    pub fn hs_key(&mut self, key: &str, ctrl: bool) {
        if self.hs_done {
            return;
        }
        match key {
            "Enter" => return self.hs_confirm(),
            "Escape" => {
                let q = self.hs_in.as_mut().unwrap();
                if q.name.iter().collect::<String>().trim().is_empty() {
                    q.name = q.def.chars().collect();
                }
                return self.hs_confirm();
            }
            _ => {}
        }
        if key == "ArrowUp" {
            self.hs_cycle(1);
        } else if key == "ArrowDown" {
            self.hs_cycle(-1);
        }
        let q = self.hs_in.as_mut().unwrap();
        match key {
            "Backspace" => {
                if q.cur > 0 {
                    q.name.remove(q.cur - 1);
                    q.cur -= 1;
                }
            }
            "Delete" => {
                if q.cur < q.name.len() {
                    q.name.remove(q.cur);
                }
            }
            "ArrowLeft" => q.cur = q.cur.saturating_sub(1),
            "ArrowRight" => q.cur = (q.cur + 1).min(q.name.len()).min(q.max.saturating_sub(1)),
            "Home" => q.cur = 0,
            "End" => q.cur = q.name.len(),
            "ArrowUp" | "ArrowDown" => {}
            k if k.chars().count() == 1 && !ctrl && q.name.len() < q.max => {
                q.name.insert(q.cur.min(q.name.len()), k.chars().next().unwrap());
                q.cur += 1;
            }
            _ => {}
        }
        q.t = 0.0;
    }
    pub fn hs_step(&mut self, dt: f64) {
        self.hs_in.as_mut().unwrap().t += dt;
        let nk = self.nav_keys();
        if nk.contains("Enter") {
            self.hs_confirm();
        } else {
            if nk.contains("ArrowUp") || nk.contains("KeyW") {
                self.hs_cycle(1);
            }
            if nk.contains("ArrowDown") || nk.contains("KeyS") {
                self.hs_cycle(-1);
            }
            let q = self.hs_in.as_mut().unwrap();
            if nk.contains("ArrowLeft") || nk.contains("KeyA") {
                q.cur = q.cur.saturating_sub(1);
            }
            if nk.contains("ArrowRight") || nk.contains("KeyD") {
                q.cur = (q.cur + 1).min(q.name.len()).min(q.max.saturating_sub(1));
            }
        }
        self.kp.clear();
        self.pkp.clear();
        self.any_press = false;
        self.mouse.pr.clear();
    }
    pub fn hs_click(&mut self) {
        let h = match self.hs_in.as_ref().and_then(|q| q.hit) {
            Some(h) => h,
            None => return,
        };
        if self.mouse.sx >= h[0] && self.mouse.sx <= h[2] && self.mouse.sy >= h[1] && self.mouse.sy <= h[3] {
            self.hs_confirm();
        }
    }

    // ---------------------------------------------------------------- dialogues
    pub fn dlg_push(&mut self, d: Dialog) {
        self.dlg.queue.push_back(d);
        if self.dlg.cur.is_none() {
            self.dlg_next();
        }
    }
    fn dlg_next(&mut self) {
        self.dlg.cur = self.dlg.queue.pop_front();
    }
    pub fn dlg_step(&mut self, dt: f64) {
        if self.dlg.cur.is_none() {
            return;
        }
        let nk0 = self.nav_keys();
        let mut adv = ["Space", "Enter", "KeyE", "NumpadEnter"].iter().any(|k| nk0.contains(*k));
        let clk = self.mouse.pr.contains("left");
        let blip = crate::rt::jtruthy(self.ds.get("blip"));
        let speed = self.dlg.speed;
        let (mx, my) = (self.mouse.sx, self.mouse.sy);
        let mut blips = 0;
        let mut finish: Option<Option<(usize, String)>> = None; // Some(None) = suivante ; Some(Some(choix))
        {
            let d = self.dlg.cur.as_mut().unwrap();
            d.t += dt;
            let chars: Vec<char> = d.text.chars().collect();
            let len = chars.len();
            if d.shown < len {
                if speed <= 0.0 {
                    d.shown = len;
                } else {
                    d.acc += dt * speed;
                    while d.acc >= 1.0 && d.shown < len {
                        d.acc -= 1.0;
                        let ch = chars[d.shown];
                        d.shown += 1;
                        if ".!?…".contains(ch) {
                            d.acc -= 6.0;
                        } else if ",;:".contains(ch) {
                            d.acc -= 2.5;
                        }
                        if blip && d.shown % 3 == 0 && !ch.is_whitespace() {
                            blips += 1;
                        }
                    }
                }
                if (adv || clk) && d.t > 0.12 {
                    d.shown = len;
                }
            } else if let Some(ch) = d.choices.clone() {
                let up = nk0.contains("ArrowUp") || nk0.contains("KeyW");
                let dn = nk0.contains("ArrowDown") || nk0.contains("KeyS");
                if up {
                    d.sel = (d.sel + ch.len() - 1) % ch.len();
                }
                if dn {
                    d.sel = (d.sel + 1) % ch.len();
                }
                if clk {
                    if let Some(h) = &d.hit {
                        for (q, h) in h.iter().enumerate() {
                            if mx >= h[0] && mx <= h[2] && my >= h[1] && my <= h[3] {
                                d.sel = q;
                                adv = true;
                            }
                        }
                    }
                }
                if adv && d.t > 0.15 {
                    finish = Some(Some((d.sel, ch[d.sel].clone())));
                }
            } else if (adv || clk) && d.t > 0.15 {
                finish = Some(None);
            }
        }
        for _ in 0..blips {
            self.sfx_v("blip", 0.25);
        }
        if let Some(f) = finish {
            if let Some((sel, txt)) = f {
                self.dlg.choice = (sel + 1) as f64;
                self.dlg.choice_text = txt;
                self.dlg.choice_tick = self.tick;
                self.sfx("blip");
            }
            self.dlg_next();
            if self.dlg.cur.is_none() {
                self.dlg.end_tick = self.tick;
            }
        }
        self.kp.clear();
        self.pkp.clear();
        self.mouse.pr.clear();
        self.any_press = false;
    }

    // ---------------------------------------------------------------- boutons
    pub fn btn_step(&mut self) {
        let mut bl: Vec<Slot> = self.all.iter().copied().filter(|&s| {
            let i = self.inst(s);
            !i.dead && i.btn.as_ref().map(|b| b.en).unwrap_or(false) && i.visible && self.layers.get(i.layer).map(|l| l.visible).unwrap_or(true)
        }).collect();
        for &s in &self.all.clone() {
            if let Some(b) = self.inst_mut(s).btn.as_mut() {
                b.hover = false;
                b.press = false;
            }
        }
        if let Some(f) = self.btn_focus {
            if !bl.contains(&f) {
                if let Some(b) = self.insts[f as usize].as_mut().and_then(|i| i.btn.as_mut()) {
                    b.focus = false;
                }
                self.btn_focus = None;
            }
        }
        if bl.is_empty() {
            return;
        }
        let tick = self.tick;
        let mut hov = None;
        for &s in bl.iter().rev() {
            if self.mouse_over(s) {
                hov = Some(s);
                break;
            }
        }
        if let Some(h) = hov {
            let down = self.mouse.down.contains("left");
            let pr = self.mouse.pr.contains("left");
            let b = self.inst_mut(h).btn.as_mut().unwrap();
            b.hover = true;
            if down {
                b.press = true;
            }
            if pr {
                b.clicked = tick;
                self.mouse.pr.clear();
                self.sfx("blip");
            }
        }
        let nav = self.gpaused || !self.all.iter().any(|&s| {
            let i = self.inst(s);
            !i.dead && (i.b.platform.as_ref().map(|p| p.en && p.controls).unwrap_or(false) || i.b.eightdir.as_ref().map(|p| p.en && p.controls).unwrap_or(false) || i.b.car.as_ref().map(|p| p.en && p.ctl).unwrap_or(false))
        });
        if !nav {
            return;
        }
        let pos: Vec<(f64, f64)> = bl.iter().map(|&s| (self.inst(s).x, self.inst(s).y)).collect();
        let mut idx: Vec<usize> = (0..bl.len()).collect();
        idx.sort_by(|&a, &b| {
            let (pa, pb) = (pos[a], pos[b]);
            let d = if (pa.1 - pb.1).abs() > 4.0 { pa.1 - pb.1 } else { pa.0 - pb.0 };
            d.partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
        });
        bl = idx.into_iter().map(|k| bl[k]).collect();
        if self.btn_focus.is_none() {
            self.btn_focus = Some(bl[0]);
            self.inst_mut(bl[0]).btn.as_mut().unwrap().focus = true;
        }
        let f = self.btn_focus.unwrap();
        let cur0 = bl.iter().position(|&x| x == f).map(|k| k as i64).unwrap_or(-1);
        let nk = self.nav_keys();
        let mv = if nk.contains("ArrowDown") || nk.contains("KeyS") || nk.contains("ArrowRight") || nk.contains("KeyD") {
            1
        } else if nk.contains("ArrowUp") || nk.contains("KeyW") || nk.contains("ArrowLeft") || nk.contains("KeyA") {
            -1
        } else {
            0
        };
        if mv != 0 {
            self.inst_mut(f).btn.as_mut().unwrap().focus = false;
            let n = bl.len() as i64;
            let nf = bl[(((cur0 + mv) % n + n) % n) as usize];
            self.btn_focus = Some(nf);
            self.inst_mut(nf).btn.as_mut().unwrap().focus = true;
            self.sfx("blip");
            self.kp.clear();
            self.pkp.clear();
            self.any_press = false;
        }
        if nk.contains("Enter") || nk.contains("Space") || nk.contains("NumpadEnter") {
            let f = self.btn_focus.unwrap();
            let b = self.inst_mut(f).btn.as_mut().unwrap();
            b.clicked = tick;
            b.press = true;
            self.kp.clear();
            self.pkp.clear();
            self.any_press = false;
            self.sfx("blip");
        }
    }

    // ---------------------------------------------------------------- particules
    pub fn pt_emit(&mut self, s: Slot, n: usize) {
        let d = self.def(s);
        let p = d.j.get("pt").cloned().unwrap_or(Json::Null);
        let (x, y, ang, mir) = {
            let i = self.inst(s);
            (i.x, i.y, i.angle, i.mirrored)
        };
        let cols: Vec<String> = p.arr("colors").iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect();
        let cols = if cols.is_empty() { vec!["#ffffff".to_string()] } else { cols };
        let mut base = p.f("angle", 0.0) + ang;
        if mir {
            base = 180.0 - base;
        }
        let spray = p.f("spray", 0.0);
        let speed = p.f("speed", 0.0);
        let speed_r = p.f("speedRand", 0.0);
        let life = p.f_or("life", 1.0);
        let life_r = p.f("lifeRand", 0.0);
        let size_r = p.f("sizeRand", 0.0);
        let width = p.f("width", 0.0);
        let s0 = if p.has("size0") { p.f("size0", 4.0) } else { 4.0 };
        let s1 = if p.has("size1") { p.f("size1", 0.0) } else { 0.0 };
        let spin = p.f("spin", 0.0);
        let mut out = Vec::new();
        let have = self.inst(s).ps.as_ref().map(|p| p.parts.len()).unwrap_or(0);
        for _ in 0..n {
            if have + out.len() >= 3000 {
                break;
            }
            let a = (base + (self.rand() - 0.5) * spray) * std::f64::consts::PI / 180.0;
            let sp = (speed + (self.rand() * 2.0 - 1.0) * speed_r).max(0.0);
            let lf = (life + (self.rand() * 2.0 - 1.0) * life_r).max(0.05);
            let sr = (self.rand() * 2.0 - 1.0) * size_r;
            let wx = if width != 0.0 { (self.rand() - 0.5) * width } else { 0.0 };
            let ci = ((self.rand() * cols.len() as f64) as usize).min(cols.len() - 1);
            let rot = if spin != 0.0 { self.rand() * 360.0 } else { 0.0 };
            let sg = if self.rand() < 0.5 { -1.0 } else { 1.0 };
            out.push(Part { x: x + wx, y, vx: a.cos() * sp, vy: a.sin() * sp, age: 0.0, life: lf, s0: (s0 + sr).max(0.0), s1: (s1 + sr * 0.5).max(0.0), c: parse_color(&cols[ci]), rot, spin: spin * sg });
        }
        if let Some(ps) = self.inst_mut(s).ps.as_mut() {
            ps.parts.extend(out);
        }
    }
    pub fn emit_parts(&mut self, s: Slot, n: usize) {
        self.pt_emit(s, n);
    }
    pub fn ps_step(&mut self, s: Slot, dt: f64) {
        let d = self.def(s);
        let p = d.j.get("pt").cloned().unwrap_or(Json::Null);
        let burst = p.s("mode") == "burst";
        if burst {
            if !self.inst(s).ps.as_ref().unwrap().started {
                self.inst_mut(s).ps.as_mut().unwrap().started = true;
                let n = p.f("burst", 0.0).max(0.0) as usize;
                self.pt_emit(s, n);
            }
        } else if self.inst(s).ps.as_ref().unwrap().active {
            let rate = self.inst(s).ps.as_ref().unwrap().rate.unwrap_or_else(|| p.f("rate", 0.0));
            let st = self.inst_mut(s).ps.as_mut().unwrap();
            st.acc += rate * dt;
            let n = st.acc.floor();
            st.acc -= n;
            if n > 0.0 {
                self.pt_emit(s, (n as usize).min(300));
            }
        }
        let g = p.f("gravity", 0.0);
        let dr = (1.0 - p.f("drag", 0.0) * dt).max(0.0);
        let destroy_end = p.get("destroyEnd") != Some(&Json::Bool(false));
        let i = self.inst_mut(s);
        let st = i.ps.as_mut().unwrap();
        st.parts.retain_mut(|q| {
            q.age += dt;
            if q.age >= q.life {
                return false;
            }
            q.vx *= dr;
            q.vy = q.vy * dr + g * dt;
            q.x += q.vx * dt;
            q.y += q.vy * dt;
            if q.spin != 0.0 {
                q.rot += q.spin * dt;
            }
            true
        });
        if burst && st.started && st.parts.is_empty() && destroy_end {
            i.kill = true;
        }
    }
}
