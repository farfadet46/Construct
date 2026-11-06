//! Dessin des objets (drawInst) et rendu d'une image complète du jeu.

use crate::coll::*;
use crate::gfx::*;
use crate::model::*;
use crate::rt::*;
use pf_core::{Json, JsonExt, Val};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tiny_skia::Pixmap;

pub fn font_family(f: &str, d: &'static str) -> String {
    match f {
        "sans" | "round" | "mono" | "serif" => f.to_string(),
        _ => d.to_string(),
    }
}

/// accès aux images (par index d'image du Def)
pub trait Imgs {
    fn img(&self, k: usize) -> Option<Rc<Pixmap>>;
}
impl Imgs for Vec<Option<Rc<Pixmap>>> {
    fn img(&self, k: usize) -> Option<Rc<Pixmap>> {
        self.get(k).and_then(|x| x.clone())
    }
}

/// propriétés d'affichage : définition + surcharges de style de l'instance
pub struct Props<'a> {
    o: &'a Json,
    st: Option<Json>,
}
impl<'a> Props<'a> {
    pub fn new(o: &'a Json, style: Option<&pf_core::Map<String, Json>>) -> Props<'a> {
        let st = style.filter(|m| m.values().any(|v| !v.is_null() && v.as_str() != Some(""))).map(|m| Json::Object(m.clone()));
        Props { o, st }
    }
    pub fn get(&self, k: &str) -> Option<&Json> {
        if let Some(s) = &self.st {
            if let Some(v) = s.get(k) {
                return Some(v);
            }
        }
        self.o.get(k)
    }
    /// `o.k||d`
    pub fn s(&self, k: &str, d: &str) -> String {
        match self.get(k) {
            Some(v) if jtruthy(Some(v)) => jstr(v),
            _ => d.to_string(),
        }
    }
    pub fn f_or(&self, k: &str, d: f64) -> f64 {
        match self.get(k) {
            Some(v) if jtruthy(Some(v)) => {
                let n = Val::from_json(v).num();
                if n.is_nan() {
                    d
                } else {
                    n
                }
            }
            _ => d,
        }
    }
    /// `o.k==null?d:o.k`
    pub fn f_nn(&self, k: &str, d: f64) -> f64 {
        match self.get(k) {
            None | Some(Json::Null) => d,
            Some(v) => Val::from_json(v).num(),
        }
    }
    pub fn b(&self, k: &str) -> bool {
        jtruthy(self.get(k))
    }
    pub fn not_false(&self, k: &str) -> bool {
        self.get(k) != Some(&Json::Bool(false))
    }
    pub fn c(&self, k: &str, d: &str) -> Rgba {
        parse_color(&self.s(k, d))
    }
}

thread_local! {
    static TM_CACHE: RefCell<HashMap<(usize, u64, i64, i64, usize), Rc<Pixmap>>> = RefCell::new(HashMap::new());
}

fn tm_pixmap(o: &Def, d: &TileData, img: &Pixmap, imk: usize) -> Option<Rc<Pixmap>> {
    let key = (d.data.as_ptr() as usize, d.ver, d.cols, d.rows, imk);
    if let Some(p) = TM_CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return Some(p);
    }
    let tw = o.j.f_or("tw", 16.0);
    let th = o.j.f_or("th", 16.0);
    let mut g = Gfx::new((d.cols as f64 * tw).max(1.0) as u32, (d.rows as f64 * th).max(1.0) as u32);
    g.set_smooth(false);
    let tc = ((img.width() as f64 / tw).floor()).max(1.0) as i64;
    for r in 0..d.rows {
        for k in 0..d.cols {
            let v = d.data[(r * d.cols + k) as usize];
            if v < 0 {
                continue;
            }
            g.draw_image_src(img, ((v % tc) as f64 * tw) as f32, ((v / tc) as f64 * th) as f32, tw as f32, th as f32, (k as f64 * tw) as f32, (r as f64 * th) as f32, tw as f32, th as f32);
        }
    }
    let p = Rc::new(g.pm);
    TM_CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 48 {
            c.clear();
        }
        c.insert(key, p.clone());
    });
    Some(p)
}

/// données pour la mini-carte
#[derive(Clone, Debug, Default)]
pub struct MmItem {
    pub id: String,
    pub l: f64,
    pub t: f64,
    pub r: f64,
    pub b: f64,
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub solid: bool,
}
#[derive(Clone, Debug, Default)]
pub struct MmData {
    pub w: f64,
    pub h: f64,
    pub items: Vec<MmItem>,
}

pub struct DrawCtx<'a> {
    pub imgs: &'a dyn Imgs,
    pub editor: bool,
    /// lookup d'objet pour la Route 3D (id → largeur posée, image)
    pub rd_lk: &'a dyn Fn(&str) -> Option<(f64, Rc<Pixmap>)>,
    pub mm: Option<&'a MmData>,
    pub now_ms: f64,
}

pub fn draw_inst(g: &mut Gfx, o: &Def, i: &Inst, cx: &DrawCtx) -> Option<f64> {
    let pr = Props::new(&o.j, i.style.as_ref());
    g.save();
    g.mul_alpha(i.opacity as f32);
    let mut hz = None;
    match o.kind {
        Kind::Road3d => {
            hz = crate::road::rd_draw(g, &o.j, i.x, i.y, i.w, i.h, i.rd.as_deref(), cx.rd_lk);
        }
        Kind::Minimap => draw_minimap(g, &pr, i, cx),
        Kind::Tilemap => {
            if let (Some(d), Some(k)) = (i.tm.as_ref(), o.image) {
                if let Some(im) = cx.imgs.img(k) {
                    if let Some(p) = tm_pixmap(o, d, &im, k) {
                        g.translate(i.x as f32, i.y as f32);
                        g.scale(o.scale as f32, o.scale as f32);
                        g.draw_pixmap(&p, 0.0, 0.0);
                    }
                }
            }
        }
        Kind::Button => draw_button(g, &pr, i),
        Kind::Bar => draw_bar(g, &pr, i),
        Kind::Leaderboard => draw_lb(g, &pr, i, cx.editor),
        Kind::Particles => {
            if let Some(ps) = &i.ps {
                draw_parts(g, &o.j, ps, cx);
            } else if cx.editor {
                let p = o.j.get("pt").cloned().unwrap_or(Json::Null);
                let a0 = ((p.f("angle", 0.0) + i.angle) * std::f64::consts::PI / 180.0) as f32;
                let sp = (p.f("spray", 0.0) * std::f64::consts::PI / 360.0) as f32;
                g.translate(i.x as f32, i.y as f32);
                let oc = parse_color("#ffb84d");
                g.fill_circle(0.0, 0.0, 9.0, parse_color("rgba(255,184,77,.18)"));
                if let Some(c) = circle_path(0.0, 0.0, 9.0) {
                    g.stroke_path(&c, 1.5, oc, false);
                }
                let mut pb = tiny_skia::PathBuilder::new();
                if sp < 3.1 {
                    pb.move_to(0.0, 0.0);
                    arc_to_builder(&mut pb, 0.0, 0.0, 26.0, a0 - sp, a0 + sp, false);
                    pb.close();
                } else {
                    arc_to_builder(&mut pb, 0.0, 0.0, 26.0, 0.0, std::f32::consts::TAU, true);
                }
                if let Some(p) = pb.finish() {
                    g.stroke_dash(&p, 1.5, oc, &[3.0, 3.0]);
                }
                if let Some(l) = poly_path(&[(0.0, 0.0), (a0.cos() * 20.0, a0.sin() * 20.0)], false) {
                    g.stroke_path(&l, 1.5, oc, false);
                }
                let cl: Vec<String> = p.arr("colors").iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect();
                let cl = if cl.is_empty() { vec!["#ffd84d".to_string()] } else { cl };
                for (k, c) in cl.iter().take(4).enumerate() {
                    g.fill_rect(-7.0 + k as f32 * 4.0, -2.0, 3.0, 3.0, parse_color(c));
                }
            }
        }
        Kind::Sprite => {
            let a = o.anims.get(i.anim).or_else(|| o.anims.first());
            let fr = a.and_then(|a| a.frames.get(i.frame).or_else(|| a.frames.first())).copied();
            let img = fr.and_then(|k| cx.imgs.img(k));
            g.translate(i.x as f32, i.y as f32);
            if i.angle != 0.0 {
                g.rotate((i.angle * std::f64::consts::PI / 180.0) as f32);
            }
            let sx = (i.w / o.w) as f32;
            let sy = (i.h / o.h) as f32;
            g.scale(if i.mirrored { -sx } else { sx }, sy);
            let ta = i.tint.map(|t| t.1).filter(|a| *a > 0.0).unwrap_or(0.0);
            let tf = i.tf.map(|t| if t.1 > 0.0 { t.1 / t.2 } else { 0.0 }).unwrap_or(0.0);
            let (ow, oh, ox, oy) = (o.w as f32, o.h as f32, o.ox as f32, o.oy as f32);
            match img {
                Some(im) if ta > 0.0 || tf > 0.0 => {
                    let mut tg = Gfx::new(ow.max(1.0) as u32, oh.max(1.0) as u32);
                    tg.set_smooth(g.smooth());
                    tg.draw_image(&im, 0.0, 0.0, ow, oh);
                    tg.set_blend(tiny_skia::BlendMode::SourceAtop);
                    if ta > 0.0 {
                        let c = i.tint.unwrap().0;
                        tg.set_alpha(ta as f32);
                        tg.fill_rect(0.0, 0.0, ow, oh, c);
                    }
                    if tf > 0.0 {
                        let c = i.tf.unwrap().0;
                        tg.set_alpha(tf as f32);
                        tg.fill_rect(0.0, 0.0, ow, oh, c);
                    }
                    g.draw_image(&tg.pm, -ox, -oy, ow, oh);
                }
                Some(im) => g.draw_image(&im, -ox, -oy, ow, oh),
                None => g.fill_rect(-ox, -oy, ow, oh, parse_color("rgba(255,138,61,.35)")),
            }
        }
        Kind::Tiled => {
            let im = o.image.and_then(|k| cx.imgs.img(k));
            g.translate(i.x as f32, i.y as f32);
            if i.angle != 0.0 {
                g.rotate((i.angle * std::f64::consts::PI / 180.0) as f32);
            }
            let s = o.scale as f32;
            g.scale(s, s);
            match im {
                Some(im) => g.fill_pattern(&im, 0.0, 0.0, i.w as f32 / s, i.h as f32 / s),
                None => g.fill_rect(0.0, 0.0, i.w as f32 / s, i.h as f32 / s, parse_color("rgba(120,140,170,.4)")),
            }
        }
        Kind::Text => {
            g.translate(i.x as f32, i.y as f32);
            if i.angle != 0.0 {
                g.rotate((i.angle * std::f64::consts::PI / 180.0) as f32);
            }
            let size = pr.f_or("size", 20.0);
            let f = FontSpec::new(if pr.b("bold") { 800 } else { 500 }, size, &font_family(&pr.s("font", "sans"), "sans"));
            let al = match pr.s("align", "left").as_str() {
                "center" => Align::Center,
                "right" => Align::Right,
                _ => Align::Left,
            };
            let tx = match al {
                Align::Center => i.w / 2.0,
                Align::Right => i.w,
                _ => 0.0,
            } as f32;
            let txt = match &i.text {
                Val::S(s) if s.is_empty() => String::new(),
                v => v.str(),
            };
            let color = pr.c("color", "#ffffff");
            let outline = pr.b("outline");
            let oc = pr.c("outlineColor", "#1b1f2a");
            for (n, line) in txt.split('\n').enumerate() {
                let y = (n as f64 * size * 1.2) as f32;
                if outline {
                    g.stroke_text(line, tx, y, &f, al, Baseline::Top, oc, (size * 0.16).max(2.0) as f32);
                }
                g.fill_text(line, tx, y, &f, al, Baseline::Top, color);
            }
        }
        _ => {}
    }
    g.restore();
    hz
}

fn draw_button(g: &mut Gfx, o: &Props, i: &Inst) {
    let bt = i.btn.clone().unwrap_or(BtnSt { en: true, clicked: -1, ..Default::default() });
    let en = bt.en;
    let hv = en && (bt.hover || bt.focus);
    let pr = en && bt.press;
    let rad = o.f_nn("radius", 12.0).min(i.h / 2.0).min(i.w / 2.0) as f32;
    let (w, h) = (i.w as f32, i.h as f32);
    g.translate(i.x as f32, (i.y + if pr { 1.5 } else { 0.0 }) as f32);
    if !en {
        g.mul_alpha(0.5);
    }
    if bt.focus && en {
        g.save();
        g.mul_alpha(0.9);
        g.stroke_rr(-4.0, -4.0, w + 8.0, h + 8.0, rad + 4.0, 3.0, o.c("focusColor", "#ffffff"));
        g.restore();
    }
    if o.not_false("shadow") && !pr {
        g.fill_rr(0.0, 3.0, w, h, rad, parse_color("rgba(0,0,0,.28)"));
    }
    let bg = if hv { o.s("hoverBg", &o.s("bg", "#ffa45e")) } else { o.s("bg", "#ff8a3d") };
    g.fill_rr(0.0, 0.0, w, h, rad, parse_color(&bg));
    let border = o.f_or("border", 0.0) as f32;
    if border > 0.0 {
        g.stroke_rr(border / 2.0, border / 2.0, w - border, h - border, (rad - border / 2.0).max(0.0), border, o.c("borderColor", "#1b1f2a"));
    }
    let mut bfs = o.f_or("size", 20.0);
    let fam = font_family(&o.s("font", "round"), "round");
    let txt = match &i.text {
        Val::S(s) if s.is_empty() => o.s("text", ""),
        v => v.str(),
    };
    let mut f = FontSpec::new(800, bfs, &fam);
    let bmw = g.measure_text(&f, &txt) as f64;
    let bav = i.w - 16f64.max(rad as f64);
    if bmw > bav && bmw > 0.0 {
        bfs = (bfs * bav / bmw).max(8.0);
        f = FontSpec::new(800, bfs, &fam);
    }
    let fg = if hv { o.s("hoverFg", &o.s("fg", "#1a0f07")) } else { o.s("fg", "#1a0f07") };
    g.fill_text(&txt, w / 2.0, h / 2.0 + 1.0, &f, Align::Center, Baseline::Middle, parse_color(&fg));
}

fn draw_bar(g: &mut Gfx, o: &Props, i: &Inst) {
    let mx0 = o.f_or("max", 100.0);
    let bv0 = o.f_nn("value", mx0 * 0.7);
    let b = i.bar.clone().unwrap_or(BarSt { v: bv0, max: mx0, shown: bv0, ghost: 0.0, gt: 0.0, col: String::new() });
    let mxv = if b.max == 0.0 { 1.0 } else { b.max };
    let fr = (b.shown / mxv).clamp(0.0, 1.0);
    let gfr = fr.max((b.ghost / mxv).min(1.0));
    let br = o.f_nn("radius", 6.0).min(i.h / 2.0).min(i.w / 2.0) as f32;
    let bd = o.f_or("border", 0.0) as f32;
    let dir = o.s("dir", "ltr");
    let seg = o.f_or("segments", 0.0).floor().max(0.0) as i32;
    let (w, h) = (i.w as f32, i.h as f32);
    g.translate(i.x as f32, i.y as f32);
    let bgc = o.c("bg", "#1b1f2a");
    g.fill_rr(0.0, 0.0, w, h, br, bgc);
    g.save();
    if let Some(p) = rr_path(bd, bd, w - bd * 2.0, h - bd * 2.0, (br - bd).max(0.0)) {
        g.clip_path(&p);
    }
    let iw = w - bd * 2.0;
    let ih = h - bd * 2.0;
    let part = |g: &mut Gfx, f: f32, c: Rgba| {
        if dir == "btt" {
            g.fill_rect(bd, bd + ih * (1.0 - f), iw, ih * f, c);
        } else if dir == "rtl" {
            g.fill_rect(bd + iw * (1.0 - f), bd, iw * f, ih, c);
        } else {
            g.fill_rect(bd, bd, iw * f, ih, c);
        }
    };
    if gfr > fr {
        part(g, gfr as f32, o.c("ghostColor", "#ffffff"));
    }
    let low_at = o.f_nn("lowAt", 25.0) / 100.0;
    let lo = o.b("lowColor") && fr <= if o.get("lowAt").map(|v| v.is_null()).unwrap_or(true) { 0.25 } else { low_at };
    let fc = if !b.col.is_empty() { parse_color(&b.col) } else if lo { o.c("lowColor", "#e8566a") } else { o.c("fg", "#e8566a") };
    part(g, fr as f32, fc);
    let hl = parse_color("rgba(255,255,255,.18)");
    if dir == "btt" {
        g.fill_rect(bd, bd, (iw * 0.3).max(1.0), ih, hl);
    } else {
        g.fill_rect(bd, bd, iw, (ih * 0.32).max(1.0), hl);
    }
    if seg > 1 {
        for sg in 1..seg {
            if dir == "btt" {
                g.fill_rect(bd, bd + ih * sg as f32 / seg as f32 - 1.0, iw, 2.0, bgc);
            } else {
                g.fill_rect(bd + iw * sg as f32 / seg as f32 - 1.0, bd, 2.0, ih, bgc);
            }
        }
    }
    g.restore();
    if bd > 0.0 {
        g.stroke_rr(bd / 2.0, bd / 2.0, w - bd, h - bd, (br - bd / 2.0).max(0.0), bd, o.c("borderColor", "#ffffff"));
    }
    if o.b("showText") {
        let bfz = o.f_or("size", 14.0).min(i.h - 4.0).max(8.0);
        let fmt = o.s("format", "{valeur} / {max}");
        let vtx = fmt.replace("{valeur}", &Val::N(pf_core::value::js_round(b.v)).str()).replace("{value}", &Val::N(pf_core::value::js_round(b.v)).str()).replace("{max}", &Val::N(pf_core::value::js_round(mxv)).str()).replace("{%}", &Val::N(pf_core::value::js_round(b.v / mxv * 100.0)).str());
        let f = FontSpec::new(800, bfz, &font_family(&o.s("font", "round"), "round"));
        g.stroke_text(&vtx, w / 2.0, h / 2.0 + 1.0, &f, Align::Center, Baseline::Middle, parse_color("rgba(0,0,0,.55)"), 3.0);
        g.fill_text(&vtx, w / 2.0, h / 2.0 + 1.0, &f, Align::Center, Baseline::Middle, o.c("textColor", "#ffffff"));
    }
}

fn draw_lb(g: &mut Gfx, o: &Props, i: &Inst, editor: bool) {
    let rows = o.f_or("rows", 5.0).max(1.0) as usize;
    let demo = vec![HsEntry { n: "PIXEL".into(), s: 42.0, d: 0.0 }, HsEntry { n: "NOVA".into(), s: 35.0, d: 0.0 }, HsEntry { n: "ZED".into(), s: 28.0, d: 0.0 }];
    let l = if i.lb.is_empty() && editor { &demo } else { &i.lb };
    let fsz = o.f_or("size", 18.0);
    let fam = font_family(&o.s("font", "round"), "round");
    let pad = pf_core::value::js_round(fsz * 0.7);
    let has_t = o.not_false("showTitle") && o.b("title");
    let th = if has_t { fsz * 1.9 } else { 0.0 };
    let rh = (i.h - pad * 2.0 - th) / rows as f64;
    g.translate(i.x as f32, i.y as f32);
    let bga = o.f_or("bgAlpha", 0.0);
    if bga > 0.0 {
        g.save();
        g.mul_alpha((bga / 100.0) as f32);
        g.fill_rr(0.0, 0.0, i.w as f32, i.h as f32, (i.h / 4.0).min(14.0) as f32, o.c("bg", "#1b1f2a"));
        g.restore();
    }
    let accent = o.c("accent", "#ffd84d");
    if has_t {
        let f = FontSpec::new(800, pf_core::value::js_round(fsz * 1.15), &fam);
        g.fill_text(&o.s("title", ""), (i.w / 2.0) as f32, (pad + th / 2.0 - fsz * 0.15) as f32, &f, Align::Center, Baseline::Middle, accent);
    }
    let fg = o.c("fg", "#ffffff");
    let suffix = o.s("suffix", "");
    for r in 0..rows {
        let e = l.get(r);
        let yy = (pad + th + rh * r as f64 + rh / 2.0) as f32;
        let hi = e.is_some() && i.lb_hi.is_some() && e.map(|e| Some(e.d) == i.lb_hi).unwrap_or(false);
        let f = FontSpec::new(if hi { 800 } else { 700 }, fsz, &fam);
        let c = if hi { accent } else { fg };
        g.save();
        if e.is_none() {
            g.mul_alpha(0.4);
        }
        g.fill_text(&format!("{}.", r + 1), pad as f32, yy, &f, Align::Left, Baseline::Middle, c);
        g.fill_text(&e.map(|e| e.n.clone()).unwrap_or_else(|| "---".into()), (pad + fsz * 2.2) as f32, yy, &f, Align::Left, Baseline::Middle, c);
        if let Some(e) = e {
            g.fill_text(&format!("{}{}", Val::N(e.s).str(), suffix), (i.w - pad) as f32, yy, &f, Align::Right, Baseline::Middle, c);
        }
        g.restore();
    }
}

pub fn draw_parts(g: &mut Gfx, o: &Json, st: &PartSt, cx: &DrawCtx) {
    let p = o.get("pt").cloned().unwrap_or(Json::Null);
    if st.parts.is_empty() {
        return;
    }
    let img = if p.s("shape") == "image" { st.img.and_then(|k| cx.imgs.img(k)) } else { None };
    g.save();
    if p.s("blend") == "add" {
        g.set_blend_add(true);
    }
    let ba = g.alpha();
    let fade = p.get("fade") != Some(&Json::Bool(false));
    let shape = p.s("shape").to_string();
    for q in &st.parts {
        let f = q.age / q.life;
        let sz = q.s0 + (q.s1 - q.s0) * f;
        if sz <= 0.0 {
            continue;
        }
        g.set_alpha(ba * if fade { (1.0 - f).max(0.0) as f32 } else { 1.0 });
        let (x, y, s) = (q.x as f32, q.y as f32, sz as f32);
        if let Some(im) = &img {
            let old = g.transform();
            g.translate(x, y);
            if q.rot != 0.0 {
                g.rotate((q.rot * std::f64::consts::PI / 180.0) as f32);
            }
            g.draw_image(im, -s / 2.0, -s / 2.0, s, s * im.height() as f32 / im.width() as f32);
            g.set_ts(old);
            continue;
        }
        if shape == "soft" {
            let a = q.c.3;
            g.fill_radial(x, y, s / 2.0, &[(0.0, q.c.with_a(a)), (0.45, q.c.with_a(a * 0.6)), (1.0, q.c.with_a(0.0))], (x - s / 2.0, y - s / 2.0, s, s));
            continue;
        }
        if shape == "circle" {
            g.fill_circle(x, y, s / 2.0, q.c);
        } else if q.rot != 0.0 {
            let old = g.transform();
            g.translate(x, y);
            g.rotate((q.rot * std::f64::consts::PI / 180.0) as f32);
            g.fill_rect(-s / 2.0, -s / 2.0, s, s, q.c);
            g.set_ts(old);
        } else {
            g.fill_rect(x - s / 2.0, y - s / 2.0, s, s, q.c);
        }
    }
    g.restore();
}

fn draw_minimap(g: &mut Gfx, o: &Props, i: &Inst, cx: &DrawCtx) {
    let (w, h) = (i.w as f32, i.h as f32);
    let round = o.s("shape", "") != "rect";
    g.save();
    g.translate(i.x as f32, i.y as f32);
    let shape = if round {
        let mut pb = tiny_skia::PathBuilder::new();
        if let Some(r) = tiny_skia::Rect::from_xywh(0.0, 0.0, w.max(0.01), h.max(0.01)) {
            pb.push_oval(r);
        }
        pb.finish()
    } else {
        rr_path(0.0, 0.0, w, h, (w / 8.0).min(12.0))
    };
    g.save();
    if let Some(p) = &shape {
        g.clip_path(p);
    }
    let ba = (o.f_nn("bgAlpha", 75.0) / 100.0) as f32;
    g.save();
    g.mul_alpha(ba);
    g.fill_rect(0.0, 0.0, w, h, o.c("bg", "#0b0d12"));
    g.restore();
    if let Some(d) = cx.mm {
        let mode = o.s("mode", "");
        let follow = o.s("follow", "");
        let fo = if mode != "level" && !follow.is_empty() { d.items.iter().find(|it| it.id == follow) } else { None };
        let (ccx, ccy, sc) = match fo {
            Some(fo) => {
                let rr = i.mm_r.unwrap_or_else(|| o.f_or("radius", 500.0)).max(50.0);
                (fo.x, fo.y, w.min(h) as f64 / 2.0 / rr)
            }
            None => (d.w / 2.0, d.h / 2.0, (w as f64 / d.w.max(1.0)).min(h as f64 / d.h.max(1.0))),
        };
        let xx = |v: f64| (w as f64 / 2.0 + (v - ccx) * sc) as f32;
        let yy = |v: f64| (h as f64 / 2.0 + (v - ccy) * sc) as f32;
        if o.not_false("solids") {
            let sc_c = o.c("solidColor", "#8b93a6");
            for it in &d.items {
                if !it.solid {
                    continue;
                }
                let (x1, y1, x2, y2) = (xx(it.l), yy(it.t), xx(it.r), yy(it.b));
                if x2 < 0.0 || y2 < 0.0 || x1 > w || y1 > h {
                    continue;
                }
                g.fill_rect(x1, y1, (x2 - x1).max(1.0), (y2 - y1).max(1.0), sc_c);
            }
        }
        if mode == "level" || fo.is_none() {
            if let Some(r) = tiny_skia::Rect::from_xywh(xx(0.0) + 0.5, yy(0.0) + 0.5, (d.w * sc - 1.0).max(0.1) as f32, (d.h * sc - 1.0).max(0.1) as f32) {
                g.stroke_path(&tiny_skia::PathBuilder::from_rect(r), 1.0, parse_color("rgba(255,255,255,.18)"), false);
            }
        }
        let dots = o.get("dots").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        for dt in dots.iter().filter(|d| d.is_object() && !d.s("obj").is_empty()) {
            let r = Val::from_json(dt.get("size").unwrap_or(&Json::Null)).num0();
            let r = if r == 0.0 { 4.0 } else { r }.max(1.5) as f32;
            let blink = jtruthy(dt.get("blink")) && ((cx.now_ms / 300.0).floor() as i64) % 2 == 1;
            let colr = if blink { Rgba::WHITE } else { parse_color(if dt.s("color").is_empty() { "#ff3b4f" } else { dt.s("color") }) };
            for it in &d.items {
                if it.id != dt.s("obj") || fo.map(|f| std::ptr::eq(f, it)).unwrap_or(false) {
                    continue;
                }
                let (mut x, mut y) = (xx(it.x), yy(it.y));
                let m = r + 2.0;
                if mode != "level" && fo.is_some() && (x < -m || y < -m || x > w + m || y > h + m) {
                    let ang = (y - h / 2.0).atan2(x - w / 2.0);
                    let rr = w.min(h) / 2.0 - r - 3.0;
                    x = w / 2.0 + ang.cos() * rr;
                    y = h / 2.0 + ang.sin() * rr;
                }
                g.fill_circle(x, y, r, colr);
            }
        }
        if let Some(fo) = fo {
            let (fx, fy) = (xx(fo.x), yy(fo.y));
            let a = (fo.angle * std::f64::consts::PI / 180.0) as f32;
            let ar = (w.min(h) / 18.0).max(5.0);
            let old = g.transform();
            g.translate(fx, fy);
            g.rotate(a);
            if let Some(p) = poly_path(&[(ar, 0.0), (-ar * 0.7, ar * 0.65), (-ar * 0.35, 0.0), (-ar * 0.7, -ar * 0.65)], true) {
                g.fill_path(&p, o.c("followColor", "#ffffff"));
                g.stroke_path(&p, 1.5, parse_color("#1b1f2a"), false);
            }
            g.set_ts(old);
        }
    } else {
        g.save();
        g.mul_alpha(0.5);
        g.fill_text("Mini-carte", w / 2.0, h / 2.0, &FontSpec::new(700, 12.0, "sans"), Align::Center, Baseline::Middle, Rgba::WHITE);
        g.restore();
    }
    g.restore();
    if let Some(p) = &shape {
        g.save();
        g.mul_alpha(0.8);
        g.stroke_path(p, o.f_or("borderW", 2.0).max(1.0) as f32, o.c("border", "#ffffff"), false);
        g.restore();
    }
    g.restore();
}

impl Inst {
    /// instance « à plat » pour l'aperçu dans l'éditeur (sans moteur)
    pub fn preview(t: usize, d: &Def, j: &Json) -> Inst {
        let mut i = Inst {
            uid: 0,
            t,
            x: j.f("x", 0.0),
            y: j.f("y", 0.0),
            w: { let v = j.f("w", 0.0); if v == 0.0 { d.j.f_or("defW", if d.w != 0.0 { d.w } else { 32.0 }) } else { v } },
            h: { let v = j.f("h", 0.0); if v == 0.0 { d.j.f_or("defH", if d.h != 0.0 { d.h } else { 32.0 }) } else { v } },
            angle: j.f("angle", 0.0),
            mirrored: jtruthy(j.get("mirrored")),
            opacity: j.get("opacity").filter(|v| !v.is_null()).map(|v| Val::from_json(v).num()).unwrap_or(1.0),
            visible: j.get("visible") != Some(&Json::Bool(false)),
            hud: false,
            text: match j.get("text").filter(|v| !v.is_null()) {
                Some(v) => Val::from_json(v),
                None => Val::S(d.j.s("text").to_string()),
            },
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
            style: j.get("style").and_then(|s| s.as_object()).cloned(),
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
        if d.kind == Kind::Tilemap {
            let cols = { let v = j.f("cols", 0.0); if v == 0.0 { d.j.f_or("defCols", 8.0) } else { v } } as i64;
            let rows = { let v = j.f("rows", 0.0); if v == 0.0 { d.j.f_or("defRows", 4.0) } else { v } } as i64;
            let dt = j.arr("data");
            let data = (0..(cols * rows).max(0) as usize).map(|q| dt.get(q).and_then(|v| v.as_f64()).map(|v| v as i64).unwrap_or(-1)).collect();
            i.tm = Some(TileData { cols, rows, data, ..Default::default() });
            i.w = cols as f64 * d.j.f_or("tw", 16.0) * d.scale;
            i.h = rows as f64 * d.j.f_or("th", 16.0) * d.scale;
        }
        if d.kind == Kind::Bar {
            let mx = d.j.f_or("max", 100.0);
            let v = j.get("value").filter(|v| !v.is_null()).map(|v| Val::from_json(v).num()).unwrap_or_else(|| d.j.get("value").filter(|v| !v.is_null()).map(|v| Val::from_json(v).num()).unwrap_or(mx * 0.7));
            i.bar = Some(BarSt { v, max: mx, shown: v, ghost: 0.0, gt: 0.0, col: String::new() });
        }
        i
    }
}

// ---------------------------------------------------------------- rendu complet

impl Runtime {
    pub fn mm_source(&self, me: Slot) -> MmData {
        let mut out = Vec::new();
        for &j in &self.all {
            let ji = self.inst(j);
            if ji.dead || j == me || ji.hud {
                continue;
            }
            if let Some(l) = self.layers.get(ji.layer) {
                if l.px == 0.0 && l.py == 0.0 {
                    continue;
                }
            }
            let b = self.box_i(j);
            out.push(MmItem { id: self.types[ji.t].id.clone(), l: b.l, t: b.t, r: b.r, b: b.b, x: (b.l + b.r) / 2.0, y: (b.t + b.b) / 2.0, angle: ji.angle, solid: self.types[ji.t].solid });
        }
        MmData { w: self.lw(), h: self.lh(), items: out }
    }

    fn rd_lookup(&self) -> impl Fn(&str) -> Option<(f64, Rc<Pixmap>)> + '_ {
        move |id: &str| {
            let &t = self.type_by_id.get(id)?;
            let d = &self.types[t].def;
            if d.kind != Kind::Sprite {
                return None;
            }
            let k = *d.anims.first()?.frames.first()?;
            let im = self.images.img(k)?;
            Some((d.j.f_or("defW", 0.0), im))
        }
    }

    fn draw_one(&mut self, g: &mut Gfx, s: Slot) {
        let t = self.inst(s).t;
        let def = self.types[t].def.clone();
        let mm = if def.kind == Kind::Minimap { Some(self.mm_source(s)) } else { None };
        let hz = {
            let lk = self.rd_lookup();
            let cx = DrawCtx { imgs: &self.images, editor: false, rd_lk: &lk, mm: mm.as_ref(), now_ms: self.now_ms };
            draw_inst(g, &def, self.inst(s), &cx)
        };
        if let (Some(hz), Some(rd)) = (hz, self.inst_mut(s).rd.as_mut()) {
            rd.hz = Some(hz);
        }
    }

    /// dessine une image du jeu dans `g` (taille quelconque, mise à l'échelle avec bandes noires)
    pub fn render(&mut self, g: &mut Gfx) {
        let (cw, ch) = (g.width() as f64, g.height() as f64);
        let (w, h) = (self.w, self.h);
        g.reset();
        g.clear(Rgba::BLACK);
        let s = (cw / w).min(ch / h);
        let ox = ((cw - w * s) / 2.0).round();
        let oy = ((ch - h * s) / 2.0).round();
        if !self.ready {
            self.draw_loader(g, s, ox, oy);
            return;
        }
        g.save();
        g.clip_rect(ox as f32, oy as f32, (w * s) as f32, (h * s) as f32);
        g.fill_rect(ox as f32, oy as f32, (w * s) as f32, (h * s) as f32, parse_color(if self.layout.s("bg").is_empty() { "#000" } else { self.layout.s("bg") }));
        g.set_smooth(!self.pixel_art);
        let (mut sx, mut sy) = (0.0, 0.0);
        if self.shake_t > 0.0 {
            let m = self.shake_a * (self.shake_t / self.shake_d);
            sx = (self.rand() * 2.0 - 1.0) * m;
            sy = (self.rand() * 2.0 - 1.0) * m;
        }
        let al = self.ialpha;
        let near = (self.cam.0 - self.cam_px).abs() < 200.0 && (self.cam.1 - self.cam_py).abs() < 200.0;
        let icx = if near { self.cam_px + (self.cam.0 - self.cam_px) * al } else { self.cam.0 };
        let icy = if near { self.cam_py + (self.cam.1 - self.cam_py) * al } else { self.cam.1 };
        let mut cx = icx + sx;
        let mut cy = icy + sy;
        if self.pixel_art {
            cx = (cx * s).round() / s;
            cy = (cy * s).round() / s;
        }
        let mut dark_done = false;
        for li in 0..self.layers.len() {
            let ly = self.layers[li].clone();
            if !dark_done && self.dark.0 > 0.0 && ly.px == 0.0 && ly.py == 0.0 {
                dark_done = true;
                self.draw_dark(g, s, ox, oy, cx, cy);
            }
            if !ly.visible || ly.opacity <= 0.0 {
                continue;
            }
            let lz = self.zf(li, false);
            let ls = s * lz;
            let mut lx = cx * ly.px;
            let mut lyy = cy * ly.py;
            if self.pixel_art {
                lx = (lx * ls).round() / ls;
                lyy = (lyy * ls).round() / ls;
            }
            g.set_alpha(ly.opacity as f32);
            for s_ in self.all.clone() {
                let (dead, layer, vis, flash, hud) = {
                    let i = self.inst(s_);
                    (i.dead, i.layer, i.visible, i.flash, i.hud)
                };
                if dead || layer != li || !vis {
                    continue;
                }
                if flash > 0.0 && ((flash * 12.0).floor() as i64) % 2 == 1 {
                    continue;
                }
                let hx = if hud { 0.0 } else { lx };
                let hy = if hud { 0.0 } else { lyy };
                let is = if hud { s } else { ls };
                g.set_transform(is as f32, 0.0, 0.0, is as f32, (ox - hx * is) as f32, (oy - hy * is) as f32);
                let (rx, ry, ipx, ipy) = {
                    let i = self.inst(s_);
                    (i.x, i.y, i.px, i.py)
                };
                if al < 1.0 && (rx - ipx).abs() < 120.0 && (ry - ipy).abs() < 120.0 {
                    let i = self.inst_mut(s_);
                    i.x = ipx + (rx - ipx) * al;
                    i.y = ipy + (ry - ipy) * al;
                }
                let b = self.draw_box_i(s_);
                let vz = if hud { 1.0 } else { lz };
                let (ang, has_ps) = {
                    let i = self.inst(s_);
                    (i.angle, i.ps.is_some())
                };
                if !(ang == 0.0 && !has_ps && (b.r < hx - 8.0 || b.l > hx + w / vz + 8.0 || b.b < hy - 8.0 || b.t > hy + h / vz + 8.0)) {
                    self.draw_one(g, s_);
                }
                let i = self.inst_mut(s_);
                i.x = rx;
                i.y = ry;
            }
            if !self.joints.is_empty() {
                g.set_transform(ls as f32, 0.0, 0.0, ls as f32, (ox - lx * ls) as f32, (oy - lyy * ls) as f32);
                self.draw_joints(g, li);
            }
        }
        g.set_alpha(1.0);
        if !dark_done && self.dark.0 > 0.0 {
            self.draw_dark(g, s, ox, oy, cx, cy);
        }
        let base = |g: &mut Gfx| g.set_transform(s as f32, 0.0, 0.0, s as f32, ox as f32, oy as f32);
        base(g);
        if let Some((c, t, d)) = &self.sflash {
            if *t > 0.0 {
                g.save();
                g.set_alpha((t / d * 0.85) as f32);
                g.fill_rect(0.0, 0.0, w as f32, h as f32, parse_color(c));
                g.restore();
            }
        }
        if self.show_col {
            self.draw_collisions(g, s, ox, oy, cx, cy);
            base(g);
        }
        if let Some(hl) = self.hl {
            if let Some(&hs) = self.all.iter().find(|&&x| self.inst(x).uid == hl && !self.inst(x).dead) {
                let i = self.inst(hs);
                let (px, py) = self.layer_pp(i.layer);
                let hb = self.draw_box_i(hs);
                let hx2 = if i.hud { 0.0 } else { cx * px };
                let hy2 = if i.hud { 0.0 } else { cy * py };
                g.set_transform(s as f32, 0.0, 0.0, s as f32, (ox - hx2 * s) as f32, (oy - hy2 * s) as f32);
                if let Some(r) = tiny_skia::Rect::from_ltrb(hb.l as f32, hb.t as f32, hb.r as f32, hb.b as f32) {
                    g.stroke_dash(&tiny_skia::PathBuilder::from_rect(r), (2.0 / s) as f32, parse_color("#ff4dd2"), &[(4.0 / s) as f32, (3.0 / s) as f32]);
                }
                base(g);
            }
        }
        base(g);
        self.draw_cine(g);
        if self.dlg.cur.is_some() {
            self.draw_dialog(g);
        }
        if self.hs_in.is_some() {
            self.draw_hs(g);
        }
        if self.ctl.open {
            self.draw_ctl(g);
        }
        if let Some(tr) = &self.tr {
            let pr = if tr.phase_out { (tr.t / tr.d).min(1.0) } else { (1.0 - tr.t / tr.d).max(0.0) };
            let c = parse_color(&tr.color);
            match tr.ty.as_str() {
                "iris" => {
                    let r = (w.hypot(h) / 2.0 * (1.0 - pr)).max(0.0) as f32;
                    let mut pb = tiny_skia::PathBuilder::new();
                    if let Some(rc) = tiny_skia::Rect::from_xywh(0.0, 0.0, w as f32, h as f32) {
                        pb.push_rect(rc);
                    }
                    if r > 0.0 {
                        if let Some(rc) = tiny_skia::Rect::from_xywh(w as f32 / 2.0 - r, h as f32 / 2.0 - r, r * 2.0, r * 2.0) {
                            pb.push_oval(rc);
                        }
                    }
                    if let Some(p) = pb.finish() {
                        g.fill_path_eo(&p, c);
                    }
                }
                "wipe" => {
                    if tr.phase_out {
                        g.fill_rect(0.0, 0.0, (w * pr) as f32, h as f32, c);
                    } else {
                        g.fill_rect((w - w * pr) as f32, 0.0, (w * pr) as f32, h as f32, c);
                    }
                }
                _ => {
                    g.save();
                    g.set_alpha(pr as f32);
                    g.fill_rect(0.0, 0.0, w as f32, h as f32, c);
                    g.restore();
                }
            }
        }
        if self.muted {
            g.save();
            g.set_alpha(0.6);
            g.fill_text("son coupé (M)", (w - 8.0) as f32, (h - 6.0) as f32, &FontSpec::new(600, 11.0, "sans"), Align::Right, Baseline::Bottom, Rgba::WHITE);
            g.restore();
        }
        g.restore();
    }

    fn draw_joints(&self, g: &mut Gfx, li: usize) {
        for j in &self.joints {
            if !self.alive(j.a) || self.inst(j.a).layer != li {
                continue;
            }
            let a = self.center(j.a);
            let b = match j.b {
                Some(b) if self.alive(b) => self.center(b),
                Some(_) => continue,
                None => (j.px, j.py),
            };
            if let Some(p) = poly_path(&[(a.0 as f32, a.1 as f32), (b.0 as f32, b.1 as f32)], false) {
                g.stroke_path(&p, 2.5, parse_color("#5a3a22"), true);
                g.stroke_path(&p, 1.2, parse_color("#c98f4f"), true);
            }
            if j.b.is_none() {
                g.fill_rect(b.0 as f32 - 4.0, b.1 as f32 - 3.0, 8.0, 6.0, parse_color("#3b3f4a"));
            }
        }
    }

    fn draw_collisions(&mut self, g: &mut Gfx, s: f64, ox: f64, oy: f64, cx: f64, cy: f64) {
        for sl in self.all.clone() {
            let (dead, vis, kind, layer, hud) = {
                let i = self.inst(sl);
                (i.dead, i.visible, self.types[i.t].def.kind, i.layer, i.hud)
            };
            if dead || !vis || kind == Kind::Text {
                continue;
            }
            let (px, py) = self.layer_pp(layer);
            let cs2 = s * self.zf(layer, hud);
            g.set_transform(cs2 as f32, 0.0, 0.0, cs2 as f32, (ox - if hud { 0.0 } else { cx * px } * cs2) as f32, (oy - if hud { 0.0 } else { cy * py } * cs2) as f32);
            let sh = self.shape(sl);
            let t = &self.types[self.inst(sl).t];
            let i = self.inst(sl);
            let col = if t.solid {
                "#3ddc84"
            } else if t.jumpthru {
                "#4da3ff"
            } else if i.b.platform.is_some() {
                "#ff8a3d"
            } else if i.b.patrol.is_some() {
                "#ff4d6a"
            } else {
                "rgba(255,255,255,.75)"
            };
            let c = parse_color(col);
            let pp: Vec<(f32, f32)> = match &sh {
                CShape::P(p, _) => p.iter().map(|q| (q[0] as f32, q[1] as f32)).collect(),
                _ => box_poly(sh.bb()).iter().map(|q| (q[0] as f32, q[1] as f32)).collect(),
            };
            if let Some(p) = poly_path(&pp, true) {
                g.save();
                g.set_alpha(0.18);
                g.fill_path(&p, c);
                g.restore();
                g.stroke_path(&p, (1.5 / s) as f32, c, false);
            }
            if i.b.platform.as_ref().map(|p| p.floor.is_some()).unwrap_or(false) {
                let b = sh.bb();
                g.fill_rect(b.l as f32, (b.b - 1.5 / s) as f32, (b.r - b.l) as f32, (3.0 / s) as f32, parse_color("#ffd84d"));
            }
        }
    }

    fn draw_dark(&mut self, g: &mut Gfx, s: f64, ox: f64, oy: f64, cx: f64, cy: f64) {
        let w = (self.w * s).ceil() as u32;
        let h = (self.h * s).ceil() as u32;
        let mut d = Gfx::new(w, h);
        d.set_alpha(self.dark.0 as f32);
        d.fill_rect(0.0, 0.0, w as f32, h as f32, parse_color(&self.dark.1));
        d.set_alpha(1.0);
        d.set_blend(tiny_skia::BlendMode::DestinationOut);
        let mut glows = Vec::new();
        for &sl in &self.all {
            let i = self.inst(sl);
            let l = match &i.b.light {
                Some(l) if !i.dead && i.visible => l,
                _ => continue,
            };
            let (px, py) = self.layer_pp(i.layer);
            let bx = self.draw_box_i(sl);
            let wx = (bx.l + bx.r) / 2.0;
            let wy = (bx.t + bx.b) / 2.0;
            let lzz = self.zf(i.layer, i.hud);
            let sx2 = (wx - if i.hud { 0.0 } else { cx * px }) * s * lzz;
            let sy2 = (wy - if i.hud { 0.0 } else { cy * py }) * s * lzz;
            let uid = i.uid as f64;
            let r = l.radius * s * lzz * if l.flicker { 1.0 + (self.time * 13.0 + uid).sin() * 0.04 + (self.time * 7.3 + uid * 3.0).sin() * 0.03 } else { 1.0 };
            if r <= 0.0 {
                continue;
            }
            let k = (l.intensity / 100.0) as f32;
            let (x, y, r) = (sx2 as f32, sy2 as f32, r as f32);
            d.fill_radial(x, y, r, &[(0.0, Rgba(0.0, 0.0, 0.0, k)), (0.55, Rgba(0.0, 0.0, 0.0, k * 0.7)), (1.0, Rgba(0.0, 0.0, 0.0, 0.0))], (x - r, y - r, r * 2.0, r * 2.0));
            if l.glow > 0.0 {
                glows.push((x, y, r, if l.color.is_empty() { "#ffd9a0".to_string() } else { l.color.clone() }, (l.glow / 100.0) as f32 * k));
            }
        }
        g.save();
        g.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        g.set_alpha(1.0);
        g.draw_pixmap(&d.pm, ox as f32, oy as f32);
        if !glows.is_empty() {
            g.set_blend_add(true);
            for (x, y, r, c, a) in glows {
                let cc = parse_color(&c);
                let (x, y) = (ox as f32 + x, oy as f32 + y);
                g.fill_radial(x, y, r * 0.8, &[(0.0, cc.with_a(a * 0.6)), (1.0, cc.with_a(0.0))], (x - r, y - r, r * 2.0, r * 2.0));
            }
        }
        g.restore();
    }

    fn ds(&self, k: &str, d: &str) -> String {
        match self.ds.get(k) {
            Some(v) if jtruthy(Some(v)) => jstr(v),
            _ => d.to_string(),
        }
    }

    fn draw_dialog(&mut self, g: &mut Gfx) {
        let (w, h) = (self.w as f32, self.h as f32);
        let fs = (self.h / 27.0).round().max(14.0) as f32;
        let lh = (fs * 1.35).round();
        let pad = (fs * 0.9).round();
        let m = 12.0;
        let fam = font_family(&self.ds("font", "sans"), "sans");
        let bg = parse_color(&self.ds("bg", "#1b1f2a"));
        let fg = parse_color(&self.ds("fg", "#ffffff"));
        let accent = parse_color(&self.ds("accent", "#ffd84d"));
        let d = self.dlg.cur.clone().unwrap();
        let por = if !d.portrait.is_empty() {
            self.type_by_id.get(&d.portrait).and_then(|&t| {
                let def = &self.types[t].def;
                if def.kind == Kind::Sprite {
                    def.anims.first().and_then(|a| a.frames.first()).and_then(|&k| self.images.img(k))
                } else {
                    None
                }
            })
        } else {
            None
        };
        let f = FontSpec::new(600, fs as f64, &fam);
        let bw = w - m * 2.0;
        let ps = if por.is_some() { lh * 3.0 + pad } else { 0.0 };
        let tx = m + pad + if por.is_some() { ps + pad * 0.6 } else { 0.0 };
        let max_w = bw - (tx - m) - pad;
        let full = d.text.clone();
        let lines = wrap_lines(g, &f, &full, max_w);
        let full_len = full.chars().count();
        let nch = if d.shown >= full_len { d.choices.as_ref().map(|c| c.len()).unwrap_or(0) } else { 0 };
        let bh = (lines.len() as f32 + if nch > 0 { nch as f32 + 0.3 } else { 0.0 }).max(3.0) * lh + pad * 2.0;
        let by = if d.top { (h * 0.13).round() + if !d.name.is_empty() { fs * 0.4 } else { 0.0 } } else { h - m - bh };
        g.save();
        g.save();
        g.set_alpha(0.94);
        g.fill_rr(m, by, bw, bh, 10.0, bg);
        g.set_alpha(0.55);
        g.stroke_rr(m + 1.0, by + 1.0, bw - 2.0, bh - 2.0, 9.0, 2.0, accent);
        g.restore();
        if let Some(por) = &por {
            let (pw, ph) = (por.width() as f32, por.height() as f32);
            let pk = (ps / pw).min(ps / ph);
            g.fill_rr(m + pad, by + pad, ps, ps, 8.0, parse_color("rgba(255,255,255,.06)"));
            let sm = g.smooth();
            g.set_smooth(!self.pixel_art);
            g.draw_image(por, m + pad + (ps - pw * pk) / 2.0, by + pad + (ps - ph * pk) / 2.0, pw * pk, ph * pk);
            g.set_smooth(sm);
        }
        if !d.name.is_empty() {
            let nf = FontSpec::new(800, (fs * 0.85).round() as f64, &fam);
            let nw = g.measure_text(&nf, &d.name) + pad * 1.4;
            let nh = (fs * 1.5).round();
            let nx = m + pad * 0.6;
            let ny = by - nh * 0.6;
            g.fill_rr(nx, ny, nw, nh, 7.0, accent);
            g.fill_text(&d.name, nx + pad * 0.7, ny + nh / 2.0 + 1.0, &nf, Align::Left, Baseline::Middle, bg);
        }
        let mut left = d.shown as i64;
        let ty = by + pad;
        for (k, l) in lines.iter().enumerate() {
            if left <= 0 {
                break;
            }
            let part: String = l.chars().take(left as usize).collect();
            g.fill_text(&part, tx, ty + k as f32 * lh, &f, Align::Left, Baseline::Top, fg);
            left -= l.chars().count() as i64 + 1;
        }
        let mut hit = None;
        if nch > 0 {
            let mut hv = Vec::new();
            let cy0 = by + pad + (lines.len().max(1) as f32) * lh + pad * 0.4;
            for (q, ch) in d.choices.as_ref().unwrap().iter().enumerate() {
                let yy = cy0 + q as f32 * lh;
                let on = q == d.sel;
                if on {
                    g.save();
                    g.set_alpha(0.18);
                    g.fill_rr(tx - 8.0, yy - 3.0, max_w + 8.0, lh, 6.0, accent);
                    g.restore();
                }
                g.fill_text(&format!("{}{}", if on { "▶ " } else { "   " }, ch), tx, yy, &f, Align::Left, Baseline::Top, if on { accent } else { fg });
                hv.push([(tx - 8.0) as f64, (yy - 3.0) as f64, (tx + max_w) as f64, (yy + lh - 3.0) as f64]);
            }
            hit = Some(hv);
        } else if d.shown >= full_len && ((d.t * 2.5).floor() as i64) % 2 == 0 {
            let ax = m + bw - pad - 8.0;
            let ay = by + bh - pad - 6.0;
            if let Some(p) = poly_path(&[(ax, ay), (ax + 12.0, ay), (ax + 6.0, ay + 7.0)], true) {
                g.fill_path(&p, accent);
            }
        }
        g.restore();
        if let Some(c) = self.dlg.cur.as_mut() {
            c.hit = hit;
        }
    }

    fn draw_hs(&mut self, g: &mut Gfx) {
        let q = self.hs_in.clone().unwrap();
        let (w, h) = (self.w as f32, self.h as f32);
        let fs = (self.h / 26.0).round().max(14.0) as f32;
        let fam = font_family(&self.ds("font", "sans"), "sans");
        let bg = parse_color(&self.ds("bg", "#1b1f2a"));
        let fg = parse_color(&self.ds("fg", "#ffffff"));
        let accent = parse_color(&self.ds("accent", "#ffd84d"));
        let pw = (w - 40.0).min((fs * (q.max as f32 * 1.05 + 6.0)).max(380.0));
        let ph = fs * 9.4;
        let px = (w - pw) / 2.0;
        let py = (h - ph) / 2.0;
        g.save();
        g.fill_rect(0.0, 0.0, w, h, parse_color("rgba(0,0,0,.6)"));
        g.fill_rr(px, py, pw, ph, 14.0, bg);
        g.save();
        g.set_alpha(0.6);
        g.stroke_rr(px + 1.0, py + 1.0, pw - 2.0, ph - 2.0, 13.0, 2.0, accent);
        g.restore();
        g.fill_text(&q.title, w / 2.0, py + fs * 1.4, &FontSpec::new(800, (fs * 1.3).round() as f64, &fam), Align::Center, Baseline::Middle, accent);
        let bw = fs * (q.max as f32 * 0.95 + 1.6);
        let bx = w / 2.0 - bw / 2.0;
        let by = py + fs * 2.7;
        let bh = fs * 2.1;
        g.fill_rr(bx, by, bw, bh, 8.0, parse_color("rgba(255,255,255,.08)"));
        g.stroke_rr(bx, by, bw, bh, 8.0, 1.5, parse_color("rgba(255,255,255,.25)"));
        let mf = FontSpec::new(800, (fs * 1.2).round() as f64, "mono");
        let cw = g.measure_text(&mf, "M");
        let tw = cw * q.max as f32;
        let tx = w / 2.0 - tw / 2.0;
        let ty = by + bh / 2.0;
        for k in 0..q.max {
            let (ch, c) = if k < q.name.len() { (q.name[k].to_string(), fg) } else { ("_".to_string(), parse_color("rgba(255,255,255,.18)")) };
            g.fill_text(&ch, tx + k as f32 * cw, ty, &mf, Align::Left, Baseline::Middle, c);
        }
        if ((q.t * 2.5).floor() as i64) % 2 == 0 {
            g.fill_rect(tx + q.cur as f32 * cw, ty + fs * 0.62, cw * 0.9, 3.0, accent);
        }
        let okw = fs * 6.0;
        let okh = fs * 1.7;
        let okx = w / 2.0 - okw / 2.0;
        let oky = py + ph - fs * 2.6;
        g.fill_rr(okx, oky, okw, okh, 8.0, accent);
        g.fill_text("Valider", w / 2.0, oky + okh / 2.0 + 1.0, &FontSpec::new(800, (fs * 0.95).round() as f64, &fam), Align::Center, Baseline::Middle, bg);
        g.save();
        g.set_alpha(0.55);
        g.fill_text("Tapez votre nom · ↑↓ changer la lettre · Entrée valider", w / 2.0, by + bh + fs * 0.85, &FontSpec::new(500, (fs * 0.68).round() as f64, &fam), Align::Center, Baseline::Middle, fg);
        g.restore();
        g.restore();
        if let Some(h) = self.hs_in.as_mut() {
            h.hit = Some([okx as f64, oky as f64, (okx + okw) as f64, (oky + okh) as f64]);
        }
    }

    fn draw_ctl(&mut self, g: &mut Gfx) {
        let (w, h) = (self.w as f32, self.h as f32);
        let fs = (self.h / 32.0).round().max(13.0) as f32;
        let lh = (fs * 2.1).round();
        let pw = (w - 40.0).min(460.0);
        let rows = self.ctl_defs.len() + 1;
        let ph = rows as f32 * lh + fs * 4.6;
        let px = (w - pw) / 2.0;
        let py = ((h - ph) / 2.0).max(10.0);
        let fam = font_family(&self.ds("font", "sans"), "sans");
        let bg = parse_color(&self.ds("bg", "#1b1f2a"));
        let fg = parse_color(&self.ds("fg", "#ffffff"));
        let accent = parse_color(&self.ds("accent", "#ffd84d"));
        g.save();
        g.fill_rect(0.0, 0.0, w, h, parse_color("rgba(0,0,0,.55)"));
        g.save();
        g.set_alpha(0.97);
        g.fill_rr(px, py, pw, ph, 12.0, bg);
        g.set_alpha(0.5);
        g.stroke_rr(px + 1.0, py + 1.0, pw - 2.0, ph - 2.0, 11.0, 2.0, accent);
        g.restore();
        g.fill_text("Touches", px + fs * 1.2, py + fs * 0.9, &FontSpec::new(800, (fs * 1.25).round() as f64, &fam), Align::Left, Baseline::Top, fg);
        g.save();
        g.set_alpha(0.6);
        g.fill_text("↑ ↓ choisir · Entrée changer · Retour : par défaut · Échap fermer", px + fs * 1.2, py + fs * 2.5, &FontSpec::new(500, (fs * 0.78).round() as f64, &fam), Align::Left, Baseline::Top, fg);
        g.restore();
        let mut hit = Vec::new();
        let y0 = py + fs * 3.8;
        let blink = ((self.now_ms / 400.0).floor() as i64) % 2 == 1;
        for k in 0..rows {
            let y = y0 + k as f32 * lh;
            let on = k == self.ctl.sel;
            if on {
                g.save();
                g.set_alpha(0.16);
                g.fill_rr(px + fs * 0.6, y, pw - fs * 1.2, lh - 6.0, 8.0, accent);
                g.restore();
            }
            let f = FontSpec::new(700, fs as f64, &fam);
            let ym = y + (lh - 6.0) / 2.0;
            if k == self.ctl_defs.len() {
                g.fill_text("Fermer", w / 2.0, ym, &f, Align::Center, Baseline::Middle, if on { accent } else { fg });
            } else {
                g.fill_text(&self.ctl_defs[k].name, px + fs * 1.3, ym, &f, Align::Left, Baseline::Middle, if on { accent } else { fg });
                let waiting = self.ctl.wait && on;
                let lab = if waiting { "Appuyez sur une touche…".to_string() } else { self.cbind[k].iter().map(|c| Runtime::key_label(c)).collect::<Vec<_>>().join(" / ") };
                g.save();
                g.set_alpha(if waiting { if blink { 1.0 } else { 0.5 } } else { 0.85 });
                g.fill_text(&lab, px + pw - fs * 1.3, ym, &FontSpec::new(600, fs as f64, &fam), Align::Right, Baseline::Middle, if waiting { accent } else { fg });
                g.restore();
            }
            hit.push([(px + fs * 0.6) as f64, y as f64, (px + pw - fs * 0.6) as f64, (y + lh - 6.0) as f64]);
        }
        g.restore();
        self.ctl.hit = Some(hit);
    }

    fn draw_cine(&mut self, g: &mut Gfx) {
        let (w, h) = (self.w as f32, self.h as f32);
        if self.cine_fade.0 > 0.0 {
            g.save();
            g.set_alpha(self.cine_fade.0.min(1.0) as f32);
            g.fill_rect(0.0, 0.0, w, h, parse_color(&self.cine_fade.1));
            g.restore();
        }
        if self.cine_bars > 0.0 {
            let k = if self.cine_bars < 1.0 { 1.0 - (1.0 - self.cine_bars).powi(3) } else { 1.0 };
            let bh = (self.h * 0.11 * k).round() as f32;
            g.fill_rect(0.0, 0.0, w, bh, Rgba::BLACK);
            g.fill_rect(0.0, h - bh, w, bh, Rgba::BLACK);
        }
        if let Some((text, t, d)) = &self.cine_title {
            let a = (t / 0.4).min((d - t) / 0.4).clamp(0.0, 1.0) as f32;
            let fs = (self.h / 14.0).round();
            let fam = font_family(&self.ds("font", "round"), "round");
            let f = FontSpec::new(800, fs, &fam);
            g.save();
            g.set_alpha(a);
            g.stroke_text(text, w / 2.0, h * 0.72, &f, Align::Center, Baseline::Middle, parse_color("rgba(0,0,0,.75)"), (fs * 0.14).max(3.0) as f32);
            g.fill_text(text, w / 2.0, h * 0.72, &f, Align::Center, Baseline::Middle, Rgba::WHITE);
            g.restore();
        }
        if self.cine.as_ref().map(|c| c.def.get("skip") != Some(&Json::Bool(false))).unwrap_or(false) {
            g.save();
            g.set_alpha(0.6);
            g.fill_text("Échap : passer", w - 12.0, h - 8.0, &FontSpec::new(600, (self.h / 40.0).round().max(10.0), "sans"), Align::Right, Baseline::Bottom, Rgba::WHITE);
            g.restore();
        }
    }

    fn draw_loader(&mut self, g: &mut Gfx, s: f64, ox: f64, oy: f64) {
        let ex = self.settings.get("export").cloned().unwrap_or(Json::Null);
        let gs = |k: &str, d: &str| if ex.s(k).is_empty() { d.to_string() } else { ex.s(k).to_string() };
        g.set_transform(s as f32, 0.0, 0.0, s as f32, ox as f32, oy as f32);
        let (w, h) = (self.w as f32, self.h as f32);
        g.fill_rect(0.0, 0.0, w, h, parse_color(&gs("bg", "#0d0f13")));
        let cy = h / 2.0;
        let logo = ex.s("logo").to_string();
        if !logo.is_empty() {
            if let Some(&t) = self.type_by_id.get(&logo) {
                if let Some(im) = self.types[t].def.anims.first().and_then(|a| a.frames.first()).and_then(|&k| self.images.img(k)) {
                    let k = (96.0 / im.width() as f32).min(96.0 / im.height() as f32);
                    let sm = g.smooth();
                    g.set_smooth(!self.pixel_art);
                    g.draw_image(&im, w / 2.0 - im.width() as f32 * k / 2.0, cy - 70.0 - im.height() as f32 * k / 2.0, im.width() as f32 * k, im.height() as f32 * k);
                    g.set_smooth(sm);
                }
            }
        }
        if ex.get("showTitle") != Some(&Json::Bool(false)) {
            g.fill_text(self.settings.s("title"), w / 2.0, cy, &FontSpec::new(800, (self.h / 16.0).round(), "round"), Align::Center, Baseline::Middle, Rgba::WHITE);
        }
        if self.splash {
            g.save();
            g.set_alpha((0.55 + 0.45 * (self.now_ms / 300.0).sin()) as f32);
            g.fill_text("Cliquez ou appuyez sur une touche pour jouer", w / 2.0, cy + h / 9.0, &FontSpec::new(700, (self.h / 26.0).round(), "round"), Align::Center, Baseline::Middle, parse_color(&gs("accent", "#ff8a3d")));
            g.restore();
        }
    }
}
