//! Éditeur de sprites : dessin au pixel, animations et images, origine, boîte de collision,
//! import d'images et de planches.

use crate::theme::{self, C};
use crate::App;
use eframe::egui::{self, Color32};
use pf_core::{json, Json, JsonExt};

#[derive(Clone, Copy, PartialEq)]
pub enum STool {
    Pencil,
    Eraser,
    Fill,
    Picker,
    Line,
    Rect,
    Origin,
    BoxCol,
    Poly,
    Point,
}

#[derive(Clone)]
pub struct AnimEd {
    pub src: Json,
    pub name: String,
    pub speed: f64,
    pub looping: bool,
    pub frames: Vec<Vec<u8>>,
    /// polygone de collision et points d'image, une entrée par image
    pub poly: Vec<Option<Vec<[f64; 2]>>>,
    pub pts: Vec<Json>,
}

impl AnimEd {
    fn new(name: String, f: Vec<u8>) -> AnimEd {
        AnimEd { src: json!({"id":pf_core::uid("a")}), name, speed: 5.0, looping: true, frames: vec![f], poly: vec![None], pts: vec![json!({})] }
    }
    /// garde poly / pts alignés sur les images
    fn fix(&mut self) {
        let n = self.frames.len();
        self.poly.resize(n, None);
        self.pts.resize(n, json!({}));
    }
    fn ins(&mut self, k: usize, f: Vec<u8>, from: Option<usize>) {
        self.fix();
        let (p, q) = match from {
            Some(i) => (self.poly[i].clone(), self.pts[i].clone()),
            None => (self.poly.first().cloned().flatten(), json!({})),
        };
        self.frames.insert(k, f);
        self.poly.insert(k, p);
        self.pts.insert(k, q);
    }
    fn rem(&mut self, k: usize) {
        self.fix();
        self.frames.remove(k);
        self.poly.remove(k);
        self.pts.remove(k);
    }
    fn swp(&mut self, a: usize, b: usize) {
        self.fix();
        self.frames.swap(a, b);
        self.poly.swap(a, b);
        self.pts.swap(a, b);
    }
}

pub struct SpriteEd {
    pub oid: String,
    pub name: String,
    pub w: u32,
    pub h: u32,
    pub anims: Vec<AnimEd>,
    pub ca: usize,
    pub cf: usize,
    pub tool: STool,
    pub color: Color32,
    pub brush: u32,
    pub zoom: f32,
    pub ox: f64,
    pub oy: f64,
    pub cbox: [f64; 4],
    pub collision: String,
    undo: Vec<(Vec<AnimEd>, u32, u32)>,
    redo: Vec<(Vec<AnimEd>, u32, u32)>,
    ver: u64,
    tex: Option<(u64, usize, usize, egui::TextureHandle)>,
    thumbs: Vec<(u64, usize, usize, egui::TextureHandle)>,
    last: Option<(i32, i32)>,
    start: Option<(i32, i32)>,
    pub onion: bool,
    pub grid: bool,
    nw: u32,
    nh: u32,
    sheet_cols: u32,
    sheet_rows: u32,
    recent: Vec<Color32>,
    pub dirty: bool,
    pub points: Vec<String>,
    cur_pt: usize,
    pdrag: Option<usize>,
}

const PAL: [u32; 24] = [
    0x000000, 0x1d2b53, 0x7e2553, 0x008751, 0xab5236, 0x5f574f, 0xc2c3c7, 0xfff1e8, 0xff004d, 0xffa300, 0xffec27, 0x00e436, 0x29adff, 0x83769c, 0xff77a8, 0xffccaa, 0xff8a3d, 0x58b84f, 0x7ec8f2, 0x3d6df2, 0xe8566a, 0xffd84d, 0x1b1f2a, 0xffffff,
];

fn decode(url: &str, w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    if let Some(img) = pf_core::data_url_bytes(url).and_then(|b| image::load_from_memory(&b).ok()) {
        let im = img.to_rgba8();
        for y in 0..h.min(im.height()) {
            for x in 0..w.min(im.width()) {
                let p = im.get_pixel(x, y).0;
                let k = ((y * w + x) * 4) as usize;
                out[k..k + 4].copy_from_slice(&p);
            }
        }
    }
    out
}
fn encode(px: &[u8], w: u32, h: u32) -> String {
    let mut out = Vec::new();
    {
        use image::ImageEncoder;
        let _ = image::codecs::png::PngEncoder::new(&mut out).write_image(px, w, h, image::ExtendedColorType::Rgba8);
    }
    format!("data:image/png;base64,{}", pf_core::b64_encode(&out))
}

impl SpriteEd {
    pub fn open(o: &Json) -> SpriteEd {
        let w = o.f_or("w", 32.0).max(1.0) as u32;
        let h = o.f_or("h", 32.0).max(1.0) as u32;
        let mut anims: Vec<AnimEd> = o.arr("anims").iter().map(|a| AnimEd {
            src: a.clone(),
            name: a.s("name").to_string(),
            speed: a.f("speed", 5.0),
            looping: a.get("loop") != Some(&Json::Bool(false)),
            frames: a.arr("frames").iter().map(|f| decode(f.as_str().unwrap_or(""), w, h)).collect(),
            poly: a.arr("poly").iter().map(|p| p.as_array().filter(|v| v.len() >= 3).map(|v| v.iter().map(|q| [q[0].as_f64().unwrap_or(0.0), q[1].as_f64().unwrap_or(0.0)]).collect())).collect(),
            pts: a.arr("pts").iter().map(|m| if m.is_object() { m.clone() } else { json!({}) }).collect(),
        }).collect();
        if anims.is_empty() {
            anims.push(AnimEd::new("Défaut".into(), vec![0u8; (w * h * 4) as usize]));
        }
        for a in &mut anims {
            if a.frames.is_empty() {
                a.frames.push(vec![0u8; (w * h * 4) as usize]);
            }
            a.fix();
        }
        let cb = o.get("cbox").filter(|c| c.is_object()).map(|c| [c.f("x", 0.0), c.f("y", 0.0), c.f("w", w as f64), c.f("h", h as f64)]).unwrap_or([0.0, 0.0, w as f64, h as f64]);
        let zoom = (300.0 / w.max(h) as f32).floor().clamp(2.0, 32.0);
        SpriteEd {
            oid: o.s("id").to_string(),
            name: o.s("name").to_string(),
            w,
            h,
            anims,
            ca: 0,
            cf: 0,
            tool: STool::Pencil,
            color: Color32::from_rgb(0xff, 0x8a, 0x3d),
            brush: 1,
            zoom,
            ox: o.f("ox", (w / 2) as f64),
            oy: o.f("oy", (h / 2) as f64),
            cbox: cb,
            collision: if o.s("collision").is_empty() { "auto".into() } else { o.s("collision").to_string() },
            undo: Vec::new(),
            redo: Vec::new(),
            ver: 1,
            tex: None,
            thumbs: Vec::new(),
            last: None,
            start: None,
            onion: false,
            grid: true,
            nw: w,
            nh: h,
            sheet_cols: 4,
            sheet_rows: 1,
            recent: Vec::new(),
            dirty: false,
            points: o.arr("points").iter().map(|p| p.s("name").to_string()).filter(|n| !n.is_empty()).collect(),
            cur_pt: 0,
            pdrag: None,
        }
    }
    fn px(&mut self) -> &mut Vec<u8> {
        &mut self.anims[self.ca].frames[self.cf]
    }
    fn push_undo_meta(&mut self) {
        self.push_undo();
    }
    fn push_undo(&mut self) {
        self.undo.push((self.anims.clone(), self.w, self.h));
        if self.undo.len() > 60 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }
    fn touch(&mut self) {
        self.ver += 1;
    }
    fn set(&mut self, x: i32, y: i32, c: [u8; 4]) {
        let (w, h, b) = (self.w as i32, self.h as i32, self.brush as i32);
        let o = (b - 1) / 2;
        for dy in 0..b {
            for dx in 0..b {
                let (px, py) = (x + dx - o, y + dy - o);
                if px >= 0 && py >= 0 && px < w && py < h {
                    let k = ((py * w + px) * 4) as usize;
                    self.px()[k..k + 4].copy_from_slice(&c);
                }
            }
        }
    }
    fn get(&self, x: i32, y: i32) -> [u8; 4] {
        let k = ((y * self.w as i32 + x) * 4) as usize;
        let p = &self.anims[self.ca].frames[self.cf];
        [p[k], p[k + 1], p[k + 2], p[k + 3]]
    }
    fn line(&mut self, a: (i32, i32), b: (i32, i32), c: [u8; 4]) {
        let (mut x0, mut y0) = a;
        let (dx, dy) = ((b.0 - x0).abs(), -(b.1 - y0).abs());
        let (sx, sy) = (if x0 < b.0 { 1 } else { -1 }, if y0 < b.1 { 1 } else { -1 });
        let mut err = dx + dy;
        loop {
            self.set(x0, y0, c);
            if x0 == b.0 && y0 == b.1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
    fn fill(&mut self, x: i32, y: i32, c: [u8; 4]) {
        let from = self.get(x, y);
        if from == c {
            return;
        }
        let (w, h) = (self.w as i32, self.h as i32);
        let mut st = vec![(x, y)];
        let b = self.brush;
        self.brush = 1;
        while let Some((x, y)) = st.pop() {
            if x < 0 || y < 0 || x >= w || y >= h || self.get(x, y) != from {
                continue;
            }
            self.set(x, y, c);
            st.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
        }
        self.brush = b;
    }
    fn resize(&mut self, nw: u32, nh: u32) {
        for a in &mut self.anims {
            for f in &mut a.frames {
                let mut n = vec![0u8; (nw * nh * 4) as usize];
                for y in 0..self.h.min(nh) {
                    for x in 0..self.w.min(nw) {
                        let k0 = ((y * self.w + x) * 4) as usize;
                        let k1 = ((y * nw + x) * 4) as usize;
                        n[k1..k1 + 4].copy_from_slice(&f[k0..k0 + 4]);
                    }
                }
                *f = n;
            }
        }
        if self.cbox == [0.0, 0.0, self.w as f64, self.h as f64] {
            self.cbox = [0.0, 0.0, nw as f64, nh as f64];
        }
        self.w = nw;
        self.h = nh;
        self.ox = self.ox.min(nw as f64);
        self.oy = self.oy.min(nh as f64);
        self.touch();
    }
    fn import(&mut self, path: &std::path::Path, sheet: bool) -> Result<(), String> {
        let b = std::fs::read(path).map_err(|e| e.to_string())?;
        let im = image::load_from_memory(&b).map_err(|e| e.to_string())?.to_rgba8();
        self.push_undo();
        let (cols, rows) = if sheet { (self.sheet_cols.max(1), self.sheet_rows.max(1)) } else { (1, 1) };
        let (fw, fh) = (im.width() / cols, im.height() / rows);
        if fw != self.w || fh != self.h {
            self.resize(fw.max(1), fh.max(1));
            self.nw = self.w;
            self.nh = self.h;
            self.ox = (fw / 2) as f64;
            self.oy = (fh / 2) as f64;
            self.cbox = [0.0, 0.0, fw as f64, fh as f64];
        }
        let mut frames = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let mut f = vec![0u8; (fw * fh * 4) as usize];
                for y in 0..fh {
                    for x in 0..fw {
                        let p = im.get_pixel(c * fw + x, r * fh + y).0;
                        let k = ((y * fw + x) * 4) as usize;
                        f[k..k + 4].copy_from_slice(&p);
                    }
                }
                frames.push(f);
            }
        }
        if sheet {
            self.anims[self.ca].frames = frames;
            self.anims[self.ca].fix();
            self.cf = 0;
        } else {
            self.anims[self.ca].frames[self.cf] = frames.remove(0);
        }
        self.touch();
        Ok(())
    }
    /// écrit le résultat dans l'objet du projet
    fn apply(&self, o: &mut Json) {
        let anims: Vec<Json> = self.anims.iter().map(|a| {
            let mut j = a.src.clone();
            if !j.is_object() {
                j = json!({});
            }
            if j.get("id").is_none() {
                j["id"] = Json::String(pf_core::uid("a"));
            }
            j["name"] = Json::String(a.name.clone());
            j["speed"] = json!(a.speed);
            j["loop"] = Json::Bool(a.looping);
            j["frames"] = Json::Array(a.frames.iter().map(|f| Json::String(encode(f, self.w, self.h))).collect());
            let n = a.frames.len();
            if a.poly.iter().take(n).any(|p| p.is_some()) {
                j["poly"] = Json::Array((0..n).map(|k| match a.poly.get(k).cloned().flatten() {
                    Some(p) => json!(p.iter().map(|q| json!([q[0], q[1]])).collect::<Vec<_>>()),
                    None => json!([]),
                }).collect());
            } else if let Some(m) = j.as_object_mut() {
                m.remove("poly");
            }
            j["pts"] = Json::Array((0..n).map(|k| a.pts.get(k).cloned().unwrap_or(json!({}))).collect());
            j
        }).collect();
        let old_w = o.f("w", self.w as f64);
        let old_dw = o.f_or("defW", self.w as f64);
        let k = if old_w > 0.0 { old_dw / old_w } else { 1.0 };
        o["anims"] = Json::Array(anims);
        o["w"] = json!(self.w);
        o["h"] = json!(self.h);
        o["ox"] = json!(self.ox);
        o["oy"] = json!(self.oy);
        o["cbox"] = json!({"x":self.cbox[0],"y":self.cbox[1],"w":self.cbox[2],"h":self.cbox[3]});
        o["collision"] = Json::String(self.collision.clone());
        o["points"] = Json::Array(self.points.iter().map(|n| json!({"name": n})).collect());
        if (self.w as f64 - old_w).abs() > 0.1 {
            o["defW"] = json!((self.w as f64 * k).round());
            o["defH"] = json!((self.h as f64 * k).round());
        }
    }
}

fn tex_of(ctx: &egui::Context, px: &[u8], w: u32, h: u32, name: &str) -> egui::TextureHandle {
    ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], px), egui::TextureOptions::NEAREST)
}

pub fn window(app: &mut App, ctx: &egui::Context, now: f64) {
    let mut ed = match app.sprite.take() {
        Some(e) => e,
        None => return,
    };
    let mut open = true;
    let mut close = false;
    let mut save = false;
    let (dsz, msz) = theme::fit(ctx, 1000.0, 680.0);
    egui::Window::new(format!("Éditeur de sprites — {}", ed.name)).id(egui::Id::new("spred")).open(&mut open).default_size(dsz).max_size(msz).min_size([560.0, 360.0]).resizable(true).collapsible(false).constrain(true).show(ctx, |ui| {
        // ---- barre du haut
        ui.horizontal_wrapped(|ui| {
            for (t, n, tip) in [(STool::Pencil, "✏ Crayon", "B"), (STool::Eraser, "⌫ Gomme", "E"), (STool::Fill, "🪣 Remplir", "G"), (STool::Picker, "💧 Pipette", "I"), (STool::Line, "／ Ligne", "L"), (STool::Rect, "▭ Rectangle", "R"), (STool::Origin, "✛ Origine", "O"), (STool::BoxCol, "⬚ Boîte", "C"), (STool::Poly, "⬠ Polygone", "P"), (STool::Point, "◆ Points", "N")] {
                if ui.add(theme::tab(ed.tool == t, n)).on_hover_text(format!("Raccourci : {}", tip)).clicked() {
                    ed.tool = t;
                }
            }
            ui.separator();
            ui.label(egui::RichText::new("Taille").color(C::MUTED()));
            ui.add(egui::DragValue::new(&mut ed.brush).range(1..=8));
            ui.separator();
            if ui.add_enabled(!ed.undo.is_empty(), theme::btn_small("↶")).on_hover_text("Annuler (Ctrl+Z)").clicked() {
                undo(&mut ed);
            }
            if ui.add_enabled(!ed.redo.is_empty(), theme::btn_small("↷")).on_hover_text("Rétablir (Ctrl+Y)").clicked() {
                redo(&mut ed);
            }
            ui.toggle_value(&mut ed.grid, "Grille");
            ui.toggle_value(&mut ed.onion, "Pelure d’oignon");
            ui.label(egui::RichText::new("Zoom").color(C::MUTED()));
            ui.add(egui::Slider::new(&mut ed.zoom, 1.0..=48.0).integer());
            ui.separator();
            if ui.add(theme::btn_accent("Enregistrer")).on_hover_text("Enregistrer dans le projet").clicked() {
                save = true;
            }
            if ui.add(theme::btn("Enregistrer et fermer")).clicked() {
                save = true;
                close = true;
            }
            if ui.add(theme::btn("Fermer")).on_hover_text("Fermer sans enregistrer").clicked() {
                close = true;
            }
            if ed.dirty {
                ui.label(egui::RichText::new("●").color(C::ACCENT())).on_hover_text("Modifications non enregistrées");
            }
        });
        ui.separator();
        // raccourcis
        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                for (k, t) in [(egui::Key::B, STool::Pencil), (egui::Key::E, STool::Eraser), (egui::Key::G, STool::Fill), (egui::Key::I, STool::Picker), (egui::Key::L, STool::Line), (egui::Key::R, STool::Rect), (egui::Key::O, STool::Origin), (egui::Key::C, STool::BoxCol), (egui::Key::P, STool::Poly), (egui::Key::N, STool::Point)] {
                    if i.key_pressed(k) && !i.modifiers.command {
                        ed.tool = t;
                    }
                }
            });
            let (z, y) = ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift, i.modifiers.command && (i.key_pressed(egui::Key::Y) || i.modifiers.shift && i.key_pressed(egui::Key::Z))));
            if z {
                undo(&mut ed);
            }
            if y {
                redo(&mut ed);
            }
        }
        ui.horizontal_top(|ui| {
            // ---- gauche : animations et couleurs
            let left_h = ui.available_height().max(200.0);
            egui::ScrollArea::vertical().id_salt("spleft").max_height(left_h).auto_shrink([true, false]).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(210.0);
                theme::section(ui, "Animations");
                let mut del = None;
                for k in 0..ed.anims.len() {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(k == ed.ca, &ed.anims[k].name).clicked() {
                            ed.ca = k;
                            ed.cf = 0;
                        }
                        if ed.anims.len() > 1 && ui.small_button("✕").clicked() {
                            del = Some(k);
                        }
                    });
                }
                if let Some(k) = del {
                    ed.push_undo();
                    ed.anims.remove(k);
                    ed.ca = ed.ca.min(ed.anims.len() - 1);
                    ed.cf = 0;
                    ed.touch();
                }
                if ui.add(theme::btn_small("+ Animation")).clicked() {
                    ed.push_undo();
                    let f = vec![0u8; (ed.w * ed.h * 4) as usize];
                    let n = format!("Anim{}", ed.anims.len() + 1);
                    ed.anims.push(AnimEd::new(n, f));
                    ed.ca = ed.anims.len() - 1;
                    ed.cf = 0;
                    ed.touch();
                }
                let a = &mut ed.anims[ed.ca];
                egui::Grid::new("anp").num_columns(2).show(ui, |ui| {
                    ui.label(egui::RichText::new("Nom").color(C::MUTED()));
                    if ui.add(egui::TextEdit::singleline(&mut a.name).desired_width(110.0)).changed() {
                        ed.dirty = true;
                    }
                    ui.end_row();
                    ui.label(egui::RichText::new("Images / s").color(C::MUTED()));
                    if ui.add(egui::DragValue::new(&mut a.speed).range(0.0..=60.0)).changed() {
                        ed.dirty = true;
                    }
                    ui.end_row();
                    ui.label(egui::RichText::new("En boucle").color(C::MUTED()));
                    if ui.checkbox(&mut a.looping, "").changed() {
                        ed.dirty = true;
                    }
                    ui.end_row();
                });
                theme::section(ui, "Couleur");
                ui.horizontal(|ui| {
                    ui.color_edit_button_srgba(&mut ed.color);
                    ui.label(egui::RichText::new(format!("#{:02x}{:02x}{:02x}", ed.color.r(), ed.color.g(), ed.color.b())).monospace().color(C::MUTED()));
                });
                egui::Grid::new("pal").spacing([2.0, 2.0]).show(ui, |ui| {
                    let mut cols: Vec<Color32> = PAL.iter().map(|c| Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, *c as u8)).collect();
                    cols.extend(ed.recent.iter().copied());
                    for (k, c) in cols.iter().enumerate() {
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                        ui.painter().rect_filled(r, 3.0, *c);
                        if *c == ed.color {
                            ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Inside);
                        }
                        if resp.clicked() {
                            ed.color = *c;
                            if ed.tool == STool::Eraser || ed.tool == STool::Picker {
                                ed.tool = STool::Pencil;
                            }
                        }
                        if k % 8 == 7 {
                            ui.end_row();
                        }
                    }
                });
                theme::section(ui, "Taille de l’image");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut ed.nw).range(1..=512));
                    ui.label("×");
                    ui.add(egui::DragValue::new(&mut ed.nh).range(1..=512));
                    if ui.add_enabled(ed.nw != ed.w || ed.nh != ed.h, theme::btn_small("Appliquer")).clicked() {
                        ed.push_undo();
                        let (a, b) = (ed.nw, ed.nh);
                        ed.resize(a, b);
                    }
                });
                theme::section(ui, "Importer");
                if ui.add(theme::btn_small("Image dans cette image…")).clicked() {
                    if let Some(p) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "gif", "webp", "bmp"]).pick_file() {
                        let _ = ed.import(&p, false);
                    }
                }
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Planche :").color(C::MUTED()));
                    ui.add(egui::DragValue::new(&mut ed.sheet_cols).range(1..=64)).on_hover_text("colonnes");
                    ui.label("×");
                    ui.add(egui::DragValue::new(&mut ed.sheet_rows).range(1..=64)).on_hover_text("lignes");
                });
                if ui.add(theme::btn_small("Découper une planche…")).on_hover_text("Remplace les images de l’animation par les cases de la planche").clicked() {
                    if let Some(p) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "gif", "webp", "bmp"]).pick_file() {
                        let _ = ed.import(&p, true);
                    }
                }
                theme::section(ui, "Collision");
                egui::ComboBox::from_id_salt("colmode").selected_text(match ed.collision.as_str() { "custom" => "boîte réglée", "poly" => "polygone", _ => "image entière" }).show_ui(ui, |ui| {
                    for (k, n) in [("auto", "image entière"), ("custom", "boîte réglée"), ("poly", "polygone (gardé tel quel)")] {
                        if ui.selectable_label(ed.collision == k, n).clicked() {
                            ed.collision = k.into();
                            ed.dirty = true;
                        }
                    }
                });
                if ed.collision == "poly" || ed.tool == STool::Poly {
                    let (ca, cf) = (ed.ca, ed.cf);
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(theme::btn_small("Depuis la boîte")).on_hover_text("Crée un polygone à partir de la boîte de collision").clicked() {
                            ed.push_undo_meta();
                            let b = ed.cbox;
                            ed.anims[ca].poly[cf] = Some(vec![[b[0], b[1]], [b[0] + b[2], b[1]], [b[0] + b[2], b[1] + b[3]], [b[0], b[1] + b[3]]]);
                            ed.collision = "poly".into();
                        }
                        if ui.add(theme::btn_small("Copier sur toutes les images")).clicked() {
                            ed.push_undo_meta();
                            let p = ed.anims[ca].poly[cf].clone();
                            for a in ed.anims.iter_mut() {
                                for q in a.poly.iter_mut() {
                                    *q = p.clone();
                                }
                            }
                        }
                        if ui.add(theme::btn_small("Effacer")).clicked() {
                            ed.push_undo_meta();
                            ed.anims[ca].poly[cf] = None;
                        }
                    });
                    ui.label(egui::RichText::new("Outil Polygone : clic = ajouter un sommet, glisser = déplacer, clic droit = supprimer").size(11.0).color(C::DIM()));
                }
                theme::section(ui, "Points d’image");
                let mut delp = None;
                for k in 0..ed.points.len() {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(k == ed.cur_pt, format!("◆ {}", ed.points[k])).clicked() {
                            ed.cur_pt = k;
                            ed.tool = STool::Point;
                        }
                        if ui.small_button("✕").clicked() {
                            delp = Some(k);
                        }
                    });
                }
                if let Some(k) = delp {
                    ed.push_undo_meta();
                    let n = ed.points.remove(k);
                    for a in ed.anims.iter_mut() {
                        for m in a.pts.iter_mut() {
                            if let Some(o) = m.as_object_mut() {
                                o.remove(&n);
                            }
                        }
                    }
                    ed.cur_pt = 0;
                }
                if let Some(n) = ed.points.get(ed.cur_pt).cloned() {
                    let mut nn = n.clone();
                    if ui.add(egui::TextEdit::singleline(&mut nn).desired_width(140.0)).changed() && !nn.is_empty() && !ed.points.contains(&nn) {
                        let k = ed.cur_pt;
                        ed.points[k] = nn.clone();
                        for a in ed.anims.iter_mut() {
                            for m in a.pts.iter_mut() {
                                if let Some(v) = m.as_object_mut().and_then(|o| o.remove(&n)) {
                                    m[&nn] = v;
                                }
                            }
                        }
                        ed.dirty = true;
                    }
                }
                ui.horizontal(|ui| {
                    if ui.add(theme::btn_small("+ Point")).on_hover_text("Ex. : canon, main… (utilisé par « Créer à un point d’image »)").clicked() {
                        let mut k = ed.points.len() + 1;
                        while ed.points.contains(&format!("Point{}", k)) {
                            k += 1;
                        }
                        ed.points.push(format!("Point{}", k));
                        ed.cur_pt = ed.points.len() - 1;
                        ed.tool = STool::Point;
                        ed.dirty = true;
                    }
                    if !ed.points.is_empty() && ui.add(theme::btn_small("Sur toutes les images")).on_hover_text("Recopie la position du point de cette image sur toutes les images de l’animation").clicked() {
                        ed.push_undo_meta();
                        let n = ed.points[ed.cur_pt].clone();
                        let (ca, cf) = (ed.ca, ed.cf);
                        if let Some(v) = ed.anims[ca].pts[cf].get(&n).cloned() {
                            for m in ed.anims[ca].pts.iter_mut() {
                                if !m.is_object() {
                                    *m = json!({});
                                }
                                m[&n] = v.clone();
                            }
                        }
                    }
                });
                ui.label(egui::RichText::new(format!("Origine ({:.0}, {:.0}) · boîte {:.0},{:.0} {:.0}×{:.0}", ed.ox, ed.oy, ed.cbox[0], ed.cbox[1], ed.cbox[2], ed.cbox[3])).size(11.0).color(C::DIM()));
            });
            });
            ui.separator();
            // ---- centre : toile
            ui.vertical(|ui| {
                canvas(ui, ctx, &mut ed);
                frames_strip(ui, ctx, &mut ed);
            });
        });
    });
    if save {
        app.snapshot();
        let oid = ed.oid.clone();
        if let Some(o) = app.obj_mut(&oid) {
            ed.apply(o);
        }
        ed.dirty = false;
        app.changed();
        app.toast("Sprite enregistré.", false, now);
    }
    if open && !close {
        app.sprite = Some(ed);
    }
}

fn undo(ed: &mut SpriteEd) {
    if let Some((a, w, h)) = ed.undo.pop() {
        ed.redo.push((ed.anims.clone(), ed.w, ed.h));
        ed.anims = a;
        ed.w = w;
        ed.h = h;
        ed.ca = ed.ca.min(ed.anims.len() - 1);
        ed.cf = ed.cf.min(ed.anims[ed.ca].frames.len() - 1);
        ed.touch();
    }
}
fn redo(ed: &mut SpriteEd) {
    if let Some((a, w, h)) = ed.redo.pop() {
        ed.undo.push((ed.anims.clone(), ed.w, ed.h));
        ed.anims = a;
        ed.w = w;
        ed.h = h;
        ed.ca = ed.ca.min(ed.anims.len() - 1);
        ed.cf = ed.cf.min(ed.anims[ed.ca].frames.len() - 1);
        ed.touch();
    }
}

fn canvas(ui: &mut egui::Ui, ctx: &egui::Context, ed: &mut SpriteEd) {
    let z = ed.zoom;
    let size = egui::vec2(ed.w as f32 * z, ed.h as f32 * z);
    // la toile prend la place restante (moins la bande des images en dessous)
    let ch = (ui.available_height() - 190.0).max(140.0);
    let avail = egui::vec2(ui.available_width().max(200.0), ch);
    egui::ScrollArea::both().id_salt("spcv").max_height(ch).max_width(avail.x).show(ui, |ui| {
        let (rect, resp) = ui.allocate_exact_size(size.max(egui::vec2(10.0, 10.0)), egui::Sense::click_and_drag());
        let p = ui.painter_at(rect);
        // damier
        let cs = (8.0f32).max(z);
        let mut y = rect.top();
        let mut row = 0;
        while y < rect.bottom() {
            let mut x = rect.left();
            let mut col = row;
            while x < rect.right() {
                let r = egui::Rect::from_min_max(egui::pos2(x, y), egui::pos2((x + cs).min(rect.right()), (y + cs).min(rect.bottom())));
                p.rect_filled(r, 0.0, if col % 2 == 0 { Color32::from_rgb(0x20, 0x25, 0x2e) } else { Color32::from_rgb(0x1a, 0x1e, 0x26) });
                x += cs;
                col += 1;
            }
            y += cs;
            row += 1;
        }
        // pelure d'oignon
        if ed.onion && ed.cf > 0 {
            let prev = ed.anims[ed.ca].frames[ed.cf - 1].clone();
            let t = tex_of(ctx, &prev, ed.w, ed.h, "onion");
            p.image(t.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::from_white_alpha(70));
        }
        let need = !matches!(&ed.tex, Some((v, a, f, _)) if *v == ed.ver && *a == ed.ca && *f == ed.cf);
        if need {
            let t = tex_of(ctx, &ed.anims[ed.ca].frames[ed.cf], ed.w, ed.h, "spframe");
            ed.tex = Some((ed.ver, ed.ca, ed.cf, t));
        }
        p.image(ed.tex.as_ref().unwrap().3.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        if ed.grid && z >= 6.0 {
            let st = egui::Stroke::new(1.0, Color32::from_white_alpha(18));
            for x in 0..=ed.w {
                p.line_segment([egui::pos2(rect.left() + x as f32 * z, rect.top()), egui::pos2(rect.left() + x as f32 * z, rect.bottom())], st);
            }
            for y in 0..=ed.h {
                p.line_segment([egui::pos2(rect.left(), rect.top() + y as f32 * z), egui::pos2(rect.right(), rect.top() + y as f32 * z)], st);
            }
        }
        // boîte de collision et origine
        if ed.collision == "custom" || ed.tool == STool::BoxCol {
            let b = ed.cbox;
            let r = egui::Rect::from_min_size(rect.min + egui::vec2(b[0] as f32 * z, b[1] as f32 * z), egui::vec2(b[2] as f32 * z, b[3] as f32 * z));
            p.rect_stroke(r, 0.0, egui::Stroke::new(1.5, Color32::from_rgb(0x3d, 0xdc, 0x97)), egui::StrokeKind::Middle);
        }
        let o = rect.min + egui::vec2(ed.ox as f32 * z, ed.oy as f32 * z);
        p.line_segment([o - egui::vec2(7.0, 0.0), o + egui::vec2(7.0, 0.0)], egui::Stroke::new(1.5, C::ACCENT()));
        p.line_segment([o - egui::vec2(0.0, 7.0), o + egui::vec2(0.0, 7.0)], egui::Stroke::new(1.5, C::ACCENT()));
        // polygone de collision et points d'image
        let (ca, cf) = (ed.ca, ed.cf);
        let to_s = |q: [f64; 2]| rect.min + egui::vec2(q[0] as f32 * z, q[1] as f32 * z);
        if ed.collision == "poly" || ed.tool == STool::Poly {
            if let Some(pl) = ed.anims[ca].poly.get(cf).cloned().flatten().or_else(|| ed.anims[ca].poly.first().cloned().flatten()) {
                let pts: Vec<egui::Pos2> = pl.iter().map(|q| to_s(*q)).collect();
                let mut cl = pts.clone();
                if let Some(f) = pts.first() {
                    cl.push(*f);
                }
                p.add(egui::Shape::line(cl, egui::Stroke::new(2.0, Color32::from_rgb(0x3d, 0xdc, 0x97))));
                if ed.tool == STool::Poly {
                    for q in &pts {
                        p.circle(*q, 4.5, Color32::from_rgb(0x3d, 0xdc, 0x97), egui::Stroke::new(1.0, Color32::BLACK));
                    }
                }
            }
        }
        for (k, n) in ed.points.iter().enumerate() {
            let loc = ed.anims[ca].pts.get(cf).and_then(|m| m.get(n)).or_else(|| ed.anims[ca].pts.first().and_then(|m| m.get(n))).map(|v| [v[0].as_f64().unwrap_or(0.0), v[1].as_f64().unwrap_or(0.0)]);
            if let Some(q) = loc {
                let c = to_s(q);
                let col = if k == ed.cur_pt && ed.tool == STool::Point { C::ACCENT() } else { Color32::from_rgb(0x4d, 0xa3, 0xff) };
                p.add(egui::Shape::convex_polygon(vec![c + egui::vec2(0.0, -6.0), c + egui::vec2(6.0, 0.0), c + egui::vec2(0.0, 6.0), c + egui::vec2(-6.0, 0.0)], col, egui::Stroke::new(1.0, Color32::BLACK)));
                p.text(c + egui::vec2(8.0, -8.0), egui::Align2::LEFT_BOTTOM, n, egui::FontId::proportional(11.0), Color32::WHITE);
            }
        }
        if ed.tool == STool::Poly || ed.tool == STool::Point {
            if let Some(pos) = resp.interact_pointer_pos().or(resp.hover_pos()) {
                let g = [(((pos.x - rect.left()) / z).round() as f64).clamp(0.0, ed.w as f64), (((pos.y - rect.top()) / z).round() as f64).clamp(0.0, ed.h as f64)];
                if ed.tool == STool::Point {
                    if let Some(n) = ed.points.get(ed.cur_pt).cloned() {
                        if resp.clicked() || resp.dragged() {
                            if resp.drag_started() || resp.clicked() {
                                ed.push_undo_meta();
                            }
                            if !ed.anims[ca].pts[cf].is_object() {
                                ed.anims[ca].pts[cf] = json!({});
                            }
                            ed.anims[ca].pts[cf][&n] = json!([g[0], g[1]]);
                        }
                    }
                } else {
                    let pl = ed.anims[ca].poly[cf].clone().unwrap_or_default();
                    let near = pl.iter().position(|q| (to_s(*q) - pos).length() < 8.0);
                    if resp.drag_started() {
                        if let Some(k) = near {
                            ed.push_undo_meta();
                            ed.pdrag = Some(k);
                        }
                    }
                    if resp.dragged() {
                        if let Some(k) = ed.pdrag {
                            if let Some(v) = ed.anims[ca].poly[cf].as_mut() {
                                if k < v.len() {
                                    v[k] = g;
                                }
                            }
                        }
                    }
                    if resp.drag_stopped() {
                        ed.pdrag = None;
                    }
                    if resp.clicked() && near.is_none() {
                        ed.push_undo_meta();
                        let mut v = pl.clone();
                        // insère le sommet sur l'arête la plus proche
                        let mut at = v.len();
                        if v.len() >= 3 {
                            let mut bd = f64::INFINITY;
                            for i in 0..v.len() {
                                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                                let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
                                let l2 = (ex * ex + ey * ey).max(1e-9);
                                let t = (((g[0] - a[0]) * ex + (g[1] - a[1]) * ey) / l2).clamp(0.0, 1.0);
                                let d = (a[0] + ex * t - g[0]).hypot(a[1] + ey * t - g[1]);
                                if d < bd {
                                    bd = d;
                                    at = i + 1;
                                }
                            }
                        }
                        v.insert(at, g);
                        ed.anims[ca].poly[cf] = Some(v);
                        ed.collision = "poly".into();
                    }
                    if resp.secondary_clicked() {
                        if let Some(k) = near {
                            ed.push_undo_meta();
                            if let Some(v) = ed.anims[ca].poly[cf].as_mut() {
                                v.remove(k);
                                if v.is_empty() {
                                    ed.anims[ca].poly[cf] = None;
                                }
                            }
                        }
                    }
                }
            }
            return;
        }
        // interactions
        let cell = |pos: egui::Pos2| (((pos.x - rect.left()) / z).floor() as i32, ((pos.y - rect.top()) / z).floor() as i32);
        let (prim, sec) = ctx.input(|i| (i.pointer.primary_down(), i.pointer.secondary_down()));
        let down = (prim || sec) && (resp.is_pointer_button_down_on() || resp.dragged());
        let col = if sec || ed.tool == STool::Eraser { [0, 0, 0, 0] } else { ed.color.to_srgba_unmultiplied() };
        if let Some(pos) = resp.interact_pointer_pos().or(resp.hover_pos()) {
            let (cx, cy) = cell(pos);
            let inside = cx >= 0 && cy >= 0 && cx < ed.w as i32 && cy < ed.h as i32;
            if inside && ed.tool != STool::Origin && ed.tool != STool::BoxCol {
                let r = egui::Rect::from_min_size(rect.min + egui::vec2(cx as f32 * z, cy as f32 * z), egui::vec2(z, z));
                p.rect_stroke(r, 0.0, egui::Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Inside);
            }
            match ed.tool {
                STool::Pencil | STool::Eraser => {
                    if down {
                        if ed.last.is_none() {
                            ed.push_undo();
                            ed.set(cx, cy, col);
                        } else {
                            let l = ed.last.unwrap();
                            ed.line(l, (cx, cy), col);
                        }
                        ed.last = Some((cx, cy));
                        ed.touch();
                    }
                }
                STool::Fill => {
                    if (resp.clicked() || resp.secondary_clicked()) && inside {
                        ed.push_undo();
                        ed.fill(cx, cy, col);
                        ed.touch();
                    }
                }
                STool::Picker => {
                    if resp.clicked() && inside {
                        let c = ed.get(cx, cy);
                        ed.color = Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3].max(1));
                        ed.tool = STool::Pencil;
                    }
                }
                STool::Line | STool::Rect => {
                    if down && ed.start.is_none() {
                        ed.start = Some((cx, cy));
                    }
                    if let Some(s) = ed.start {
                        let a = rect.min + egui::vec2((s.0 as f32 + 0.5) * z, (s.1 as f32 + 0.5) * z);
                        let b = rect.min + egui::vec2((cx as f32 + 0.5) * z, (cy as f32 + 0.5) * z);
                        if ed.tool == STool::Line {
                            p.line_segment([a, b], egui::Stroke::new(z.max(1.0) * ed.brush as f32, ed.color));
                        } else {
                            p.rect_stroke(egui::Rect::from_two_pos(a, b), 0.0, egui::Stroke::new(z.max(1.0) * ed.brush as f32, ed.color), egui::StrokeKind::Middle);
                        }
                        if !prim && !sec {
                            ed.push_undo();
                            if ed.tool == STool::Line {
                                ed.line(s, (cx, cy), col);
                            } else {
                                let (x0, x1) = (s.0.min(cx), s.0.max(cx));
                                let (y0, y1) = (s.1.min(cy), s.1.max(cy));
                                ed.line((x0, y0), (x1, y0), col);
                                ed.line((x1, y0), (x1, y1), col);
                                ed.line((x1, y1), (x0, y1), col);
                                ed.line((x0, y1), (x0, y0), col);
                            }
                            ed.start = None;
                            ed.touch();
                        }
                    }
                }
                STool::Origin => {
                    if down {
                        ed.ox = (((pos.x - rect.left()) / z).round() as f64).clamp(0.0, ed.w as f64);
                        ed.oy = (((pos.y - rect.top()) / z).round() as f64).clamp(0.0, ed.h as f64);
                        ed.dirty = true;
                    }
                }
                STool::Poly | STool::Point => {}
                STool::BoxCol => {
                    let gx = (((pos.x - rect.left()) / z).round() as i32).clamp(0, ed.w as i32);
                    let gy = (((pos.y - rect.top()) / z).round() as i32).clamp(0, ed.h as i32);
                    if down && ed.start.is_none() {
                        ed.start = Some((gx, gy));
                    }
                    if let Some(s) = ed.start {
                        let (x0, x1) = (s.0.min(gx), s.0.max(gx));
                        let (y0, y1) = (s.1.min(gy), s.1.max(gy));
                        ed.cbox = [x0 as f64, y0 as f64, (x1 - x0).max(1) as f64, (y1 - y0).max(1) as f64];
                        ed.collision = "custom".into();
                        ed.dirty = true;
                        if !prim && !sec {
                            ed.start = None;
                        }
                    }
                }
            }
        }
        if !prim && !sec {
            if ed.last.is_some() && !ed.recent.contains(&ed.color) && ed.tool == STool::Pencil && !PAL.iter().any(|c| Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, *c as u8) == ed.color) {
                ed.recent.insert(0, ed.color);
                ed.recent.truncate(8);
            }
            ed.last = None;
        }
    });
}

fn frames_strip(ui: &mut egui::Ui, ctx: &egui::Context, ed: &mut SpriteEd) {
    theme::section(ui, &format!("Images de « {} »", ed.anims[ed.ca].name));
    let n = ed.anims[ed.ca].frames.len();
    egui::ScrollArea::horizontal().id_salt("frs").show(ui, |ui| {
        ui.horizontal(|ui| {
            for k in 0..n {
                let ok = ed.thumbs.iter().position(|t| t.1 == ed.ca && t.2 == k);
                let fresh = ok.map(|i| ed.thumbs[i].0 == ed.ver).unwrap_or(false);
                if !fresh {
                    let t = tex_of(ctx, &ed.anims[ed.ca].frames[k], ed.w, ed.h, "thumb");
                    match ok {
                        Some(i) => ed.thumbs[i] = (ed.ver, ed.ca, k, t),
                        None => ed.thumbs.push((ed.ver, ed.ca, k, t)),
                    }
                }
                let t = ed.thumbs.iter().find(|t| t.1 == ed.ca && t.2 == k).unwrap().3.clone();
                let s = 56.0 / ed.w.max(ed.h) as f32;
                let (r, resp) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::click());
                ui.painter().rect_filled(r, 6.0, C::BG0());
                ui.painter().image(t.id(), egui::Rect::from_center_size(r.center(), egui::vec2(ed.w as f32 * s, ed.h as f32 * s)), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                ui.painter().rect_stroke(r, 6.0, egui::Stroke::new(if k == ed.cf { 2.0 } else { 1.0 }, if k == ed.cf { C::ACCENT() } else { C::LINE2() }), egui::StrokeKind::Inside);
                ui.painter().text(r.left_top() + egui::vec2(4.0, 2.0), egui::Align2::LEFT_TOP, (k + 1).to_string(), egui::FontId::proportional(10.0), C::DIM());
                if resp.clicked() {
                    ed.cf = k;
                }
            }
        });
    });
    ui.horizontal(|ui| {
        let w = ed.w;
        let h = ed.h;
        if ui.add(theme::btn_small("+ Image vide")).clicked() {
            ed.push_undo();
            let ca = ed.ca;
            let k = ed.cf + 1;
            ed.anims[ca].ins(k, vec![0u8; (w * h * 4) as usize], None);
            ed.cf += 1;
            ed.touch();
        }
        if ui.add(theme::btn_small("Dupliquer")).clicked() {
            ed.push_undo();
            let ca = ed.ca;
            let f = ed.anims[ca].frames[ed.cf].clone();
            let k = ed.cf;
            ed.anims[ca].ins(k + 1, f, Some(k));
            ed.cf += 1;
            ed.touch();
        }
        if ui.add_enabled(ed.cf > 0, theme::btn_small("◀ Déplacer")).clicked() {
            ed.push_undo();
            let ca = ed.ca;
            let k = ed.cf;
            ed.anims[ca].swp(k, k - 1);
            ed.cf -= 1;
            ed.touch();
        }
        if ui.add_enabled(ed.cf + 1 < n, theme::btn_small("Déplacer ▶")).clicked() {
            ed.push_undo();
            let ca = ed.ca;
            let k = ed.cf;
            ed.anims[ca].swp(k, k + 1);
            ed.cf += 1;
            ed.touch();
        }
        if ui.add_enabled(n > 1, theme::btn_small("Supprimer")).clicked() {
            ed.push_undo();
            let ca = ed.ca;
            let k = ed.cf;
            ed.anims[ca].rem(k);
            ed.cf = ed.cf.min(ed.anims[ca].frames.len() - 1);
            ed.touch();
        }
        if ui.add(theme::btn_small("Miroir ↔")).clicked() {
            ed.push_undo();
            let (w, h) = (ed.w as usize, ed.h as usize);
            let f = ed.px();
            for y in 0..h {
                for x in 0..w / 2 {
                    for c in 0..4 {
                        f.swap((y * w + x) * 4 + c, (y * w + (w - 1 - x)) * 4 + c);
                    }
                }
            }
            ed.touch();
        }
        if ui.add(theme::btn_small("Effacer l’image")).clicked() {
            ed.push_undo();
            ed.px().fill(0);
            ed.touch();
        }
    });
    // aperçu animé
    let a = &ed.anims[ed.ca];
    if a.frames.len() > 1 && a.speed > 0.0 {
        let t = ctx.input(|i| i.time);
        let k = ((t * a.speed) as usize) % a.frames.len();
        if let Some(th) = ed.thumbs.iter().find(|x| x.1 == ed.ca && x.2 == k) {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Aperçu :").color(C::MUTED()));
                let s = 64.0 / ed.w.max(ed.h) as f32;
                ui.image((th.3.id(), egui::vec2(ed.w as f32 * s, ed.h as f32 * s)));
            });
        }
        ctx.request_repaint();
    }
}
