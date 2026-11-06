//! Petite surface de dessin façon « Canvas 2D » au-dessus de tiny-skia :
//! transformations, opacité globale, rectangles arrondis, images, motifs, dégradés et texte.

use ab_glyph::{Font, FontRef, GlyphId, OutlineCurve};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tiny_skia::*;

pub use tiny_skia::{Color, Pixmap, Transform};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl Rgba {
    pub fn color(&self, alpha: f32) -> Color {
        Color::from_rgba(self.0.clamp(0.0, 1.0), self.1.clamp(0.0, 1.0), self.2.clamp(0.0, 1.0), (self.3 * alpha).clamp(0.0, 1.0)).unwrap_or(Color::BLACK)
    }
    pub fn with_a(self, a: f32) -> Rgba {
        Rgba(self.0, self.1, self.2, a)
    }
    pub const WHITE: Rgba = Rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Rgba = Rgba(0.0, 0.0, 0.0, 1.0);
    pub const NONE: Rgba = Rgba(0.0, 0.0, 0.0, 0.0);
}

fn hexv(s: &str) -> Option<u32> {
    u32::from_str_radix(s, 16).ok()
}

/// couleurs CSS : #rgb #rgba #rrggbb #rrggbbaa rgb() rgba() et quelques noms
pub fn parse_color(s: &str) -> Rgba {
    let s = s.trim();
    if let Some(h) = s.strip_prefix('#') {
        let n = h.len();
        if let Some(v) = hexv(h) {
            return match n {
                3 => Rgba(((v >> 8) & 15) as f32 / 15.0, ((v >> 4) & 15) as f32 / 15.0, (v & 15) as f32 / 15.0, 1.0),
                4 => Rgba(((v >> 12) & 15) as f32 / 15.0, ((v >> 8) & 15) as f32 / 15.0, ((v >> 4) & 15) as f32 / 15.0, (v & 15) as f32 / 15.0),
                6 => Rgba(((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0, 1.0),
                8 => Rgba(((v >> 24) & 255) as f32 / 255.0, ((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0),
                _ => Rgba::BLACK,
            };
        }
        return Rgba::BLACK;
    }
    let l = s.to_ascii_lowercase();
    if l.starts_with("rgb") {
        if let (Some(a), Some(b)) = (l.find('('), l.rfind(')')) {
            let parts: Vec<f32> = l[a + 1..b].split(|c| c == ',' || c == '/' || c == ' ').filter(|x| !x.trim().is_empty()).map(|x| {
                let x = x.trim();
                if let Some(p) = x.strip_suffix('%') {
                    p.parse::<f32>().unwrap_or(0.0) / 100.0 * 255.0
                } else {
                    x.parse::<f32>().unwrap_or(0.0)
                }
            }).collect();
            let g = |i: usize| parts.get(i).copied().unwrap_or(0.0);
            let alpha = if parts.len() > 3 { g(3) } else { 1.0 };
            return Rgba(g(0) / 255.0, g(1) / 255.0, g(2) / 255.0, alpha);
        }
    }
    match l.as_str() {
        "transparent" => Rgba::NONE,
        "white" => Rgba::WHITE,
        "black" => Rgba::BLACK,
        "red" => Rgba(1.0, 0.0, 0.0, 1.0),
        "green" => Rgba(0.0, 0.5, 0.0, 1.0),
        "blue" => Rgba(0.0, 0.0, 1.0, 1.0),
        "yellow" => Rgba(1.0, 1.0, 0.0, 1.0),
        "orange" => Rgba(1.0, 0.647, 0.0, 1.0),
        "gray" | "grey" => Rgba(0.5, 0.5, 0.5, 1.0),
        _ => Rgba::BLACK,
    }
}

/// lighten() de la version HTML
pub fn lighten(h: &str, k: f64) -> String {
    let t = h.trim_start_matches('#');
    if t.len() != 6 {
        return h.to_string();
    }
    let n = match u32::from_str_radix(t, 16) {
        Ok(n) => n,
        Err(_) => return h.to_string(),
    };
    let f = |v: u32| -> u32 { ((v as f64 + (255.0 - v as f64) * k) + 0.5).floor() as u32 };
    format!("#{:02x}{:02x}{:02x}", f(n >> 16 & 255), f(n >> 8 & 255), f(n & 255))
}

// ---------------------------------------------------------------- polices

pub struct FontFace {
    pub font: FontRef<'static>,
    upem: f32,
    asc: f32,
    desc: f32,
    cache: RefCell<HashMap<GlyphId, Option<Path>>>,
}

impl FontFace {
    fn new(data: &'static [u8]) -> FontFace {
        let font = FontRef::try_from_slice(data).expect("font");
        let upem = font.units_per_em().unwrap_or(1000.0);
        let asc = font.ascent_unscaled();
        let desc = font.descent_unscaled();
        FontFace { font, upem, asc, desc, cache: RefCell::new(HashMap::new()) }
    }
    fn glyph_path(&self, id: GlyphId) -> Option<Path> {
        if let Some(p) = self.cache.borrow().get(&id) {
            return p.clone();
        }
        let p = self.font.outline(id).and_then(|o| {
            let mut pb = PathBuilder::new();
            let mut last: Option<(f32, f32)> = None;
            for c in &o.curves {
                let (s, e) = match c {
                    OutlineCurve::Line(a, b) => ((a.x, a.y), (b.x, b.y)),
                    OutlineCurve::Quad(a, _, b) => ((a.x, a.y), (b.x, b.y)),
                    OutlineCurve::Cubic(a, _, _, b) => ((a.x, a.y), (b.x, b.y)),
                };
                if last.map(|l| (l.0 - s.0).abs() > 0.01 || (l.1 - s.1).abs() > 0.01).unwrap_or(true) {
                    if last.is_some() {
                        pb.close();
                    }
                    pb.move_to(s.0, -s.1);
                }
                match c {
                    OutlineCurve::Line(_, b) => pb.line_to(b.x, -b.y),
                    OutlineCurve::Quad(_, c1, b) => pb.quad_to(c1.x, -c1.y, b.x, -b.y),
                    OutlineCurve::Cubic(_, c1, c2, b) => pb.cubic_to(c1.x, -c1.y, c2.x, -c2.y, b.x, -b.y),
                }
                last = Some(e);
            }
            if last.is_some() {
                pb.close();
            }
            pb.finish()
        });
        self.cache.borrow_mut().insert(id, p.clone());
        p
    }
    /// part de l'ascendant dans le cadratin (pour textBaseline top/middle/bottom)
    fn asc_n(&self) -> f32 {
        let t = self.asc - self.desc;
        if t <= 0.0 {
            0.8
        } else {
            self.asc / t
        }
    }
}

pub struct Fonts {
    faces: Vec<FontFace>,
}

static FONT_DATA: [&[u8]; 8] = [
    include_bytes!("../../../assets/fonts/DejaVuSans.ttf"),
    include_bytes!("../../../assets/fonts/DejaVuSans-Bold.ttf"),
    include_bytes!("../../../assets/fonts/Poppins-Medium.ttf"),
    include_bytes!("../../../assets/fonts/Poppins-Bold.ttf"),
    include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf"),
    include_bytes!("../../../assets/fonts/DejaVuSansMono-Bold.ttf"),
    include_bytes!("../../../assets/fonts/DejaVuSerif.ttf"),
    include_bytes!("../../../assets/fonts/DejaVuSerif-Bold.ttf"),
];

pub fn font_bytes(family: &str, bold: bool) -> &'static [u8] {
    FONT_DATA[family_index(family) * 2 + bold as usize]
}

fn family_index(f: &str) -> usize {
    match f {
        "round" => 1,
        "mono" => 2,
        "serif" => 3,
        _ => 0,
    }
}

thread_local! {
    static FONTS: Rc<Fonts> = Rc::new(Fonts { faces: FONT_DATA.iter().map(|d| FontFace::new(d)).collect() });
}

pub fn fonts() -> Rc<Fonts> {
    FONTS.with(|f| f.clone())
}

#[derive(Clone, Debug)]
pub struct FontSpec {
    pub family: String,
    pub bold: bool,
    pub size: f32,
}

impl FontSpec {
    /// poids CSS ('800', '700', '600', '500')
    pub fn new(weight: u32, size: f64, family: &str) -> FontSpec {
        FontSpec { family: family.to_string(), bold: weight >= 700, size: size as f32 }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Baseline {
    Top,
    Middle,
    Alphabetic,
    Bottom,
}

impl Fonts {
    fn face(&self, f: &FontSpec) -> &FontFace {
        &self.faces[family_index(&f.family) * 2 + f.bold as usize]
    }
    pub fn measure(&self, f: &FontSpec, text: &str) -> f32 {
        let face = self.face(f);
        let k = f.size / face.upem;
        let mut w = 0.0;
        let mut prev: Option<GlyphId> = None;
        for ch in text.chars() {
            let id = face.font.glyph_id(ch);
            if let Some(p) = prev {
                w += face.font.kern_unscaled(p, id);
            }
            w += face.font.h_advance_unscaled(id);
            prev = Some(id);
        }
        w * k
    }
    /// chemin du texte (origine = point d'ancrage selon align/baseline)
    pub fn text_path(&self, f: &FontSpec, text: &str, align: Align, base: Baseline) -> Option<Path> {
        let face = self.face(f);
        let k = f.size / face.upem;
        let w = self.measure(f, text);
        let x0 = match align {
            Align::Left => 0.0,
            Align::Center => -w / 2.0,
            Align::Right => -w,
        };
        let an = face.asc_n();
        let y0 = match base {
            Baseline::Top => an * f.size,
            Baseline::Middle => (an - 0.5) * f.size,
            Baseline::Alphabetic => 0.0,
            Baseline::Bottom => (an - 1.0) * f.size,
        };
        let mut pb = PathBuilder::new();
        let mut x = 0.0f32;
        let mut prev: Option<GlyphId> = None;
        let mut any = false;
        for ch in text.chars() {
            let id = face.font.glyph_id(ch);
            if let Some(p) = prev {
                x += face.font.kern_unscaled(p, id);
            }
            if let Some(gp) = face.glyph_path(id) {
                let ts = Transform::from_row(k, 0.0, 0.0, k, x0 + x * k, y0);
                if let Some(tp) = gp.transform(ts) {
                    pb.push_path(&tp);
                    any = true;
                }
            }
            x += face.font.h_advance_unscaled(id);
            prev = Some(id);
        }
        if !any {
            return None;
        }
        pb.finish()
    }
}

// ---------------------------------------------------------------- surface

#[derive(Clone)]
struct State {
    ts: Transform,
    alpha: f32,
    blend: BlendMode,
    clip: Option<Rc<Mask>>,
    smooth: bool,
}

pub struct Gfx {
    pub pm: Pixmap,
    st: State,
    stack: Vec<State>,
    pub fonts: Rc<Fonts>,
}

pub fn rr_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    if r <= 0.01 {
        let rc = Rect::from_xywh(x, y, w.max(0.0001), h.max(0.0001))?;
        return Some(PathBuilder::from_rect(rc));
    }
    // arcTo approximé par des courbes de Bézier (k = 0.5523)
    let k = 0.552_284_8 * r;
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}

pub fn circle_path(cx: f32, cy: f32, r: f32) -> Option<Path> {
    PathBuilder::from_circle(cx, cy, r.max(0.0001))
}

pub fn poly_path(pts: &[(f32, f32)], close: bool) -> Option<Path> {
    let mut pb = PathBuilder::new();
    for (k, p) in pts.iter().enumerate() {
        if k == 0 {
            pb.move_to(p.0, p.1);
        } else {
            pb.line_to(p.0, p.1);
        }
    }
    if close {
        pb.close();
    }
    pb.finish()
}

/// arc à la manière de ctx.arc (angles en radians, sens horaire)
pub fn arc_to_builder(pb: &mut PathBuilder, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, move_first: bool) {
    let mut sweep = a1 - a0;
    if sweep.abs() > std::f32::consts::TAU {
        sweep = std::f32::consts::TAU * sweep.signum();
    }
    let n = ((sweep.abs() / (std::f32::consts::PI / 2.0)).ceil() as usize).max(1);
    let d = sweep / n as f32;
    let (sx, sy) = (cx + r * a0.cos(), cy + r * a0.sin());
    if move_first {
        pb.move_to(sx, sy);
    } else {
        pb.line_to(sx, sy);
    }
    let k = 4.0 / 3.0 * (d / 4.0).tan();
    let mut a = a0;
    for _ in 0..n {
        let b = a + d;
        let (x0, y0) = (cx + r * a.cos(), cy + r * a.sin());
        let (x3, y3) = (cx + r * b.cos(), cy + r * b.sin());
        let (x1, y1) = (x0 - k * r * a.sin(), y0 + k * r * a.cos());
        let (x2, y2) = (x3 + k * r * b.sin(), y3 - k * r * b.cos());
        pb.cubic_to(x1, y1, x2, y2, x3, y3);
        a = b;
    }
}

impl Gfx {
    pub fn new(w: u32, h: u32) -> Gfx {
        Gfx {
            pm: Pixmap::new(w.max(1), h.max(1)).unwrap(),
            st: State { ts: Transform::identity(), alpha: 1.0, blend: BlendMode::SourceOver, clip: None, smooth: true },
            stack: Vec::new(),
            fonts: fonts(),
        }
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        let (w, h) = (w.max(1), h.max(1));
        if self.pm.width() != w || self.pm.height() != h {
            self.pm = Pixmap::new(w, h).unwrap();
        }
        self.reset();
    }
    pub fn reset(&mut self) {
        self.stack.clear();
        self.st = State { ts: Transform::identity(), alpha: 1.0, blend: BlendMode::SourceOver, clip: None, smooth: true };
    }
    pub fn width(&self) -> u32 {
        self.pm.width()
    }
    pub fn height(&self) -> u32 {
        self.pm.height()
    }
    pub fn save(&mut self) {
        self.stack.push(self.st.clone());
    }
    pub fn restore(&mut self) {
        if let Some(s) = self.stack.pop() {
            self.st = s;
        }
    }
    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.st.ts = Transform::from_row(a, b, c, d, e, f);
    }
    pub fn transform(&self) -> Transform {
        self.st.ts
    }
    pub fn set_ts(&mut self, t: Transform) {
        self.st.ts = t;
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.st.ts = self.st.ts.pre_translate(x, y);
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        self.st.ts = self.st.ts.pre_scale(x, y);
    }
    pub fn rotate(&mut self, rad: f32) {
        self.st.ts = self.st.ts.pre_concat(Transform::from_rotate(rad.to_degrees()));
    }
    pub fn alpha(&self) -> f32 {
        self.st.alpha
    }
    pub fn set_alpha(&mut self, a: f32) {
        self.st.alpha = a.clamp(0.0, 1.0);
    }
    pub fn mul_alpha(&mut self, a: f32) {
        self.st.alpha = (self.st.alpha * a).clamp(0.0, 1.0);
    }
    pub fn set_blend_add(&mut self, add: bool) {
        self.st.blend = if add { BlendMode::Plus } else { BlendMode::SourceOver };
    }
    pub fn set_blend(&mut self, b: BlendMode) {
        self.st.blend = b;
    }
    pub fn set_smooth(&mut self, s: bool) {
        self.st.smooth = s;
    }
    pub fn smooth(&self) -> bool {
        self.st.smooth
    }
    /// clip rectangulaire (dans le repère courant), combiné au clip existant
    pub fn clip_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if let Some(p) = Rect::from_xywh(x, y, w.max(0.001), h.max(0.001)).map(PathBuilder::from_rect) {
            self.clip_path(&p);
        }
    }
    pub fn clip_path(&mut self, p: &Path) {
        let mut m = match &self.st.clip {
            Some(c) => (**c).clone(),
            None => {
                let mut m = Mask::new(self.pm.width(), self.pm.height()).unwrap();
                m.fill_path(&PathBuilder::from_rect(Rect::from_xywh(0.0, 0.0, self.pm.width() as f32, self.pm.height() as f32).unwrap()), FillRule::Winding, false, Transform::identity());
                m
            }
        };
        m.intersect_path(p, FillRule::Winding, true, self.st.ts);
        self.st.clip = Some(Rc::new(m));
    }
    fn paint_color(&self, c: Rgba) -> Paint<'static> {
        let mut p = Paint::default();
        p.set_color(c.color(self.st.alpha));
        p.anti_alias = true;
        p.blend_mode = self.st.blend;
        p
    }
    pub fn clear(&mut self, c: Rgba) {
        self.pm.fill(c.color(1.0));
    }
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        if c.3 <= 0.0 || self.st.alpha <= 0.0 || w <= 0.0 || h <= 0.0 {
            return;
        }
        if let Some(r) = Rect::from_xywh(x, y, w, h) {
            let mut p = self.paint_color(c);
            // pas d'anticrénelage sur les rectangles alignés (comme fillRect)
            if self.st.ts.is_scale_translate() {
                p.anti_alias = false;
                let ts = self.st.ts;
                // arrondi au pixel près pour éviter les joints visibles entre tuiles
                let l = x * ts.sx + ts.tx;
                let t = y * ts.sy + ts.ty;
                let rr = (x + w) * ts.sx + ts.tx;
                let b = (y + h) * ts.sy + ts.ty;
                let (l, rr) = if l < rr { (l, rr) } else { (rr, l) };
                let (t, b) = if t < b { (t, b) } else { (b, t) };
                if let Some(r2) = Rect::from_ltrb(l.round(), t.round(), rr.round().max(l.round() + 1.0), b.round().max(t.round() + 1.0)) {
                    let clip = self.st.clip.clone();
                    self.pm.fill_rect(r2, &p, Transform::identity(), clip.as_deref());
                    return;
                }
            }
            let clip = self.st.clip.clone();
            self.pm.fill_rect(r, &p, self.st.ts, clip.as_deref());
        }
    }
    pub fn fill_path(&mut self, path: &Path, c: Rgba) {
        if c.3 <= 0.0 || self.st.alpha <= 0.0 {
            return;
        }
        let p = self.paint_color(c);
        let clip = self.st.clip.clone();
        self.pm.fill_path(path, &p, FillRule::Winding, self.st.ts, clip.as_deref());
    }
    pub fn fill_path_eo(&mut self, path: &Path, c: Rgba) {
        let p = self.paint_color(c);
        let clip = self.st.clip.clone();
        self.pm.fill_path(path, &p, FillRule::EvenOdd, self.st.ts, clip.as_deref());
    }
    pub fn stroke_path(&mut self, path: &Path, width: f32, c: Rgba, round: bool) {
        if c.3 <= 0.0 || self.st.alpha <= 0.0 || width <= 0.0 {
            return;
        }
        let p = self.paint_color(c);
        let mut s = Stroke { width, ..Default::default() };
        if round {
            s.line_join = LineJoin::Round;
            s.line_cap = LineCap::Round;
        } else {
            s.line_join = LineJoin::Miter;
        }
        let clip = self.st.clip.clone();
        self.pm.stroke_path(path, &p, &s, self.st.ts, clip.as_deref());
    }
    pub fn stroke_dash(&mut self, path: &Path, width: f32, c: Rgba, dash: &[f32]) {
        let p = self.paint_color(c);
        let s = Stroke { width, dash: StrokeDash::new(dash.to_vec(), 0.0), ..Default::default() };
        let clip = self.st.clip.clone();
        self.pm.stroke_path(path, &p, &s, self.st.ts, clip.as_deref());
    }
    pub fn fill_rr(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, c: Rgba) {
        if let Some(p) = rr_path(x, y, w, h, r) {
            self.fill_path(&p, c);
        }
    }
    pub fn stroke_rr(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, width: f32, c: Rgba) {
        if let Some(p) = rr_path(x, y, w, h, r) {
            self.stroke_path(&p, width, c, false);
        }
    }
    pub fn fill_circle(&mut self, x: f32, y: f32, r: f32, c: Rgba) {
        if r <= 0.0 {
            return;
        }
        if let Some(p) = circle_path(x, y, r) {
            self.fill_path(&p, c);
        }
    }
    /// drawImage(img, sx,sy,sw,sh, dx,dy,dw,dh)
    pub fn draw_image_src(&mut self, img: &Pixmap, sx: f32, sy: f32, sw: f32, sh: f32, dx: f32, dy: f32, dw: f32, dh: f32) {
        if self.st.alpha <= 0.0 || dw == 0.0 || dh == 0.0 || sw <= 0.0 || sh <= 0.0 {
            return;
        }
        let (dx, dw) = if dw < 0.0 { (dx + dw, -dw) } else { (dx, dw) };
        let (dy, dh) = if dh < 0.0 { (dy + dh, -dh) } else { (dy, dh) };
        let r = match Rect::from_xywh(dx, dy, dw, dh) {
            Some(r) => r,
            None => return,
        };
        let q = if self.st.smooth { FilterQuality::Bilinear } else { FilterQuality::Nearest };
        let pt = Transform::from_row(dw / sw, 0.0, 0.0, dh / sh, dx - sx * dw / sw, dy - sy * dh / sh);
        let pat = Pattern::new(img.as_ref(), SpreadMode::Pad, q, self.st.alpha, pt);
        let mut p = Paint::default();
        p.shader = pat;
        p.blend_mode = self.st.blend;
        p.anti_alias = !self.st.ts.is_scale_translate();
        let clip = self.st.clip.clone();
        self.pm.fill_rect(r, &p, self.st.ts, clip.as_deref());
    }
    pub fn draw_image(&mut self, img: &Pixmap, dx: f32, dy: f32, dw: f32, dh: f32) {
        self.draw_image_src(img, 0.0, 0.0, img.width() as f32, img.height() as f32, dx, dy, dw, dh);
    }
    /// remplit un rectangle avec une image répétée (createPattern 'repeat')
    pub fn fill_pattern(&mut self, img: &Pixmap, x: f32, y: f32, w: f32, h: f32) {
        if self.st.alpha <= 0.0 || w <= 0.0 || h <= 0.0 {
            return;
        }
        let r = match Rect::from_xywh(x, y, w, h) {
            Some(r) => r,
            None => return,
        };
        let q = if self.st.smooth { FilterQuality::Bilinear } else { FilterQuality::Nearest };
        let pat = Pattern::new(img.as_ref(), SpreadMode::Repeat, q, self.st.alpha, Transform::identity());
        let mut p = Paint::default();
        p.shader = pat;
        p.blend_mode = self.st.blend;
        p.anti_alias = false;
        let clip = self.st.clip.clone();
        self.pm.fill_rect(r, &p, self.st.ts, clip.as_deref());
    }
    /// dégradé radial (stops : position, couleur)
    pub fn fill_radial(&mut self, cx: f32, cy: f32, r: f32, stops: &[(f32, Rgba)], rect: (f32, f32, f32, f32)) {
        if r <= 0.0 {
            return;
        }
        let st: Vec<GradientStop> = stops.iter().map(|(p, c)| GradientStop::new(*p, c.color(self.st.alpha))).collect();
        let sh = match RadialGradient::new(Point::from_xy(cx, cy), Point::from_xy(cx, cy), r, st, SpreadMode::Pad, Transform::identity()) {
            Some(s) => s,
            None => return,
        };
        let mut p = Paint::default();
        p.shader = sh;
        p.blend_mode = self.st.blend;
        if let Some(rc) = Rect::from_xywh(rect.0, rect.1, rect.2, rect.3) {
            let clip = self.st.clip.clone();
            self.pm.fill_rect(rc, &p, self.st.ts, clip.as_deref());
        }
    }
    pub fn fill_linear(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, stops: &[(f32, Rgba)], rect: (f32, f32, f32, f32)) {
        let st: Vec<GradientStop> = stops.iter().map(|(p, c)| GradientStop::new(*p, c.color(self.st.alpha))).collect();
        let sh = match LinearGradient::new(Point::from_xy(x0, y0), Point::from_xy(x1, y1), st, SpreadMode::Pad, Transform::identity()) {
            Some(s) => s,
            None => return,
        };
        let mut p = Paint::default();
        p.shader = sh;
        p.blend_mode = self.st.blend;
        if let Some(rc) = Rect::from_xywh(rect.0, rect.1, rect.2, rect.3) {
            let clip = self.st.clip.clone();
            self.pm.fill_rect(rc, &p, self.st.ts, clip.as_deref());
        }
    }
    pub fn measure_text(&self, f: &FontSpec, t: &str) -> f32 {
        self.fonts.measure(f, t)
    }
    pub fn fill_text(&mut self, t: &str, x: f32, y: f32, f: &FontSpec, al: Align, bl: Baseline, c: Rgba) {
        if t.is_empty() || c.3 <= 0.0 || self.st.alpha <= 0.0 {
            return;
        }
        let fonts = self.fonts.clone();
        if let Some(p) = fonts.text_path(f, t, al, bl) {
            let old = self.st.ts;
            self.st.ts = old.pre_translate(x, y);
            self.fill_path(&p, c);
            self.st.ts = old;
        }
    }
    pub fn stroke_text(&mut self, t: &str, x: f32, y: f32, f: &FontSpec, al: Align, bl: Baseline, c: Rgba, width: f32) {
        if t.is_empty() || c.3 <= 0.0 || self.st.alpha <= 0.0 {
            return;
        }
        let fonts = self.fonts.clone();
        if let Some(p) = fonts.text_path(f, t, al, bl) {
            let old = self.st.ts;
            self.st.ts = old.pre_translate(x, y);
            self.stroke_path(&p, width, c, true);
            self.st.ts = old;
        }
    }
    /// dessine une autre surface (repère courant)
    pub fn draw_pixmap(&mut self, src: &Pixmap, x: f32, y: f32) {
        self.draw_image(src, x, y, src.width() as f32, src.height() as f32);
    }
}

/// décode une image PNG/JPEG… (data URL) en Pixmap prémultipliée
pub fn decode_image(bytes: &[u8]) -> Option<Pixmap> {
    if bytes.starts_with(b"\x89PNG") {
        return Pixmap::decode_png(bytes).ok();
    }
    None
}

/// texte découpé en lignes (wrapLines)
pub fn wrap_lines(g: &Gfx, f: &FontSpec, text: &str, max_w: f32) -> Vec<String> {
    let mut out = Vec::new();
    for par in text.split('\n') {
        let mut line = String::new();
        for w in par.split(' ') {
            let t = if line.is_empty() { w.to_string() } else { format!("{} {}", line, w) };
            if g.measure_text(f, &t) > max_w && !line.is_empty() {
                out.push(std::mem::take(&mut line));
                line = w.to_string();
            } else {
                line = t;
            }
        }
        out.push(line);
    }
    out
}
