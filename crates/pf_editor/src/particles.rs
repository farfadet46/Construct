//! Éditeur de particules : réglages en clair et aperçu en direct par le vrai moteur.
use crate::theme::{self, C};
use crate::App;
use eframe::egui::{self, Color32};
use pf_core::{json, Json, JsonExt};
use pf_runtime::gfx::Gfx;

const PW: u32 = 420;
const PH: u32 = 300;

pub struct PartEd {
    pub oid: String,
    name: String,
    pt: Json,
    rt: Option<pf_runtime::Runtime>,
    g: Gfx,
    tex: Option<egui::TextureHandle>,
    /// temps écoulé depuis le dernier départ de l'aperçu
    age: f64,
    last: std::time::Instant,
    bg: usize,
    pub dirty: bool,
    /// l'aperçu doit être reconstruit
    stale: bool,
    paused: bool,
}

const BGS: [(&str, &str); 4] = [("Sombre", "#151820"), ("Ciel", "#7ec8f2"), ("Clair", "#f2f2ee"), ("Nuit", "#05060a")];

fn hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}
fn parse(s: &str) -> Color32 {
    let s = s.trim_start_matches('#');
    let v = u32::from_str_radix(if s.len() == 3 { &s[..3] } else { s.get(..6).unwrap_or("ffffff") }, 16).unwrap_or(0xffffff);
    if s.len() == 3 {
        let (r, g, b) = ((v >> 8) & 15, (v >> 4) & 15, v & 15);
        return Color32::from_rgb((r * 17) as u8, (g * 17) as u8, (b * 17) as u8);
    }
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

impl PartEd {
    pub fn open(app: &App, oid: &str) -> Option<PartEd> {
        let o = app.obj(oid)?;
        let pt = o.get("pt").filter(|p| p.is_object()).cloned().unwrap_or_else(|| json!({}));
        Some(PartEd { oid: oid.to_string(), name: o.s("name").to_string(), pt, rt: None, g: Gfx::new(PW, PH), tex: None, age: 0.0, last: std::time::Instant::now(), bg: 0, dirty: false, stale: true, paused: false })
    }

    /// mini-projet : un niveau, l'émetteur au centre (et le sprite utilisé comme image)
    fn build(&mut self, app: &App) {
        let mut ob = app.obj(&self.oid).cloned().unwrap_or_else(|| json!({"id": self.oid, "kind": "particles"}));
        let mut pt = self.pt.clone();
        pt["destroyEnd"] = Json::Bool(false);
        ob["pt"] = pt;
        let mut objs = vec![ob];
        let img = self.pt.s("image").to_string();
        if !img.is_empty() {
            if let Some(s) = app.obj(&img) {
                objs.push(s.clone());
            }
        }
        let up = self.pt.f("gravity", 0.0) < 0.0;
        let y = if self.pt.s("mode") == "burst" { PH as f64 * 0.5 } else if up { PH as f64 * 0.75 } else { PH as f64 * 0.4 };
        let p = json!({
            "app": "platforge", "version": 1, "name": "apercu",
            "settings": {"title": "aperçu", "width": PW, "height": PH, "pixelArt": false, "startLayout": "L"},
            "objects": objs, "sounds": [], "cines": [], "globals": [],
            "sheets": [{"id": "S", "name": "S", "events": []}],
            "layouts": [{"id": "L", "name": "L", "w": PW, "h": PH, "bg": BGS[self.bg].1, "sheet": "S",
                "layers": [{"id": "y", "name": "P", "parallaxX": 100, "parallaxY": 100, "opacity": 100, "visible": true}],
                "instances": [{"id": "i1", "obj": self.oid, "x": PW / 2, "y": y, "w": 32, "h": 32, "angle": 0, "mirrored": false, "opacity": 1, "visible": true}]}]
        });
        self.rt = Some(pf_runtime::Runtime::new(p, pf_runtime::Options::default(), Box::new(pf_runtime::NullHost)));
        self.age = 0.0;
        self.stale = false;
    }

    fn changed(&mut self) {
        self.dirty = true;
        self.stale = true;
    }
}

fn num(ui: &mut egui::Ui, ed: &mut PartEd, k: &str, label: &str, tip: &str, r: std::ops::RangeInclusive<f64>, suffix: &str, def: f64) {
    ui.label(egui::RichText::new(label).color(C::MUTED())).on_hover_text(tip);
    let mut v = ed.pt.f(k, def);
    let step = if *r.end() <= 10.0 { 0.05 } else { 1.0 };
    if ui.add(egui::Slider::new(&mut v, r).step_by(step).suffix(suffix).clamping(egui::SliderClamping::Never)).changed() {
        ed.pt[k] = json!(if step < 1.0 { (v * 100.0).round() / 100.0 } else { v.round() });
        ed.changed();
    }
    ui.end_row();
}

pub fn window(app: &mut App, ctx: &egui::Context, now: f64) {
    let mut ed = match app.part_ed.take() {
        Some(e) => e,
        None => return,
    };
    if app.obj(&ed.oid).is_none() {
        return;
    }
    let mut open = true;
    let (mut save, mut close) = (false, false);
    let (dsz, msz) = theme::fit(ctx, 900.0, 640.0);
    egui::Window::new(format!("Éditeur de particules — {}", ed.name)).id(egui::Id::new("parted")).open(&mut open).default_size(dsz).max_size(msz).min_size([520.0, 320.0]).resizable(true).collapsible(false).constrain(true).show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Préréglages :").color(C::MUTED()));
            if let Some(pr) = pf_core::catalog::catalog()["PTPRESETS"].as_object() {
                for p in pr.values() {
                    if ui.add(theme::btn_small(p.s("n"))).clicked() {
                        let mut q = p.clone();
                        q.as_object_mut().unwrap().remove("n");
                        ed.pt = q;
                        ed.changed();
                    }
                }
            }
            ui.separator();
            if ui.add(theme::btn_accent("Enregistrer")).clicked() {
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
        ui.horizontal_top(|ui| {
            let h = ui.available_height();
            egui::ScrollArea::vertical().id_salt("ptl").max_height(h).auto_shrink([true, false]).show(ui, |ui| { ui.vertical(|ui| {
                ui.set_width(360.0);
                theme::section(ui, "Émission");
                egui::Grid::new("pte").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("Mode").color(C::MUTED()));
                    ui.horizontal(|ui| {
                        let burst = ed.pt.s("mode") == "burst";
                        if ui.selectable_label(burst, "Rafale").on_hover_text("Toutes les particules d’un coup (explosion, impact)").clicked() && !burst {
                            ed.pt["mode"] = json!("burst");
                            ed.changed();
                        }
                        if ui.selectable_label(!burst, "Continu").on_hover_text("Un flux régulier (fumée, feu, pluie)").clicked() && burst {
                            ed.pt["mode"] = json!("stream");
                            ed.changed();
                        }
                    });
                    ui.end_row();
                    if ed.pt.s("mode") == "burst" {
                        num(ui, &mut ed, "burst", "Nombre", "Particules lancées par rafale", 1.0..=300.0, "", 10.0);
                    } else {
                        num(ui, &mut ed, "rate", "Débit", "Particules par seconde", 0.0..=500.0, " /s", 20.0);
                    }
                    num(ui, &mut ed, "width", "Largeur de la source", "Les particules naissent sur une ligne de cette largeur (0 = un point)", 0.0..=400.0, " px", 0.0);
                });
                theme::section(ui, "Direction et vitesse");
                egui::Grid::new("ptd").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    num(ui, &mut ed, "angle", "Direction", "0 = droite, -90 = haut, 90 = bas", -180.0..=180.0, "°", 0.0);
                    num(ui, &mut ed, "spray", "Dispersion", "Ouverture du cône d’émission (360 = toutes les directions)", 0.0..=360.0, "°", 0.0);
                    num(ui, &mut ed, "speed", "Vitesse", "Vitesse de départ", 0.0..=1000.0, " px/s", 0.0);
                    num(ui, &mut ed, "speedRand", "Variation", "Écart aléatoire de vitesse", 0.0..=500.0, " px/s", 0.0);
                });
                theme::section(ui, "Mouvement");
                egui::Grid::new("ptm").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    num(ui, &mut ed, "gravity", "Gravité", "Positive : tombe · négative : monte (fumée)", -1000.0..=1000.0, " px/s²", 0.0);
                    num(ui, &mut ed, "drag", "Freinage", "Ralentissement dans l’air", 0.0..=10.0, "", 0.0);
                    num(ui, &mut ed, "spin", "Rotation", "Vitesse de rotation (carrés et images)", -1000.0..=1000.0, "°/s", 0.0);
                });
                theme::section(ui, "Durée de vie et taille");
                egui::Grid::new("ptl2").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    num(ui, &mut ed, "life", "Durée de vie", "Temps avant disparition", 0.05..=5.0, " s", 1.0);
                    num(ui, &mut ed, "lifeRand", "Variation", "Écart aléatoire de durée", 0.0..=3.0, " s", 0.0);
                    num(ui, &mut ed, "size0", "Taille au début", "", 0.0..=64.0, " px", 4.0);
                    num(ui, &mut ed, "size1", "Taille à la fin", "", 0.0..=64.0, " px", 0.0);
                    num(ui, &mut ed, "sizeRand", "Variation", "Écart aléatoire de taille", 0.0..=32.0, " px", 0.0);
                });
                theme::section(ui, "Apparence");
                egui::Grid::new("pta").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("Forme").color(C::MUTED()));
                    ui.horizontal_wrapped(|ui| {
                        let cur = ed.pt.s("shape").to_string();
                        for (k, n) in [("square", "Carré"), ("circle", "Rond"), ("soft", "Lueur douce"), ("image", "Image")] {
                            if ui.selectable_label(cur == k || (cur.is_empty() && k == "square"), n).clicked() {
                                ed.pt["shape"] = json!(k);
                                ed.changed();
                            }
                        }
                    });
                    ui.end_row();
                    if ed.pt.s("shape") == "image" {
                        ui.label(egui::RichText::new("Sprite").color(C::MUTED()));
                        let cur = ed.pt.s("image").to_string();
                        let sprites: Vec<(String, String)> = app.proj.arr("objects").iter().filter(|o| o.s("kind") == "sprite").map(|o| (o.s("id").to_string(), o.s("name").to_string())).collect();
                        let nm = sprites.iter().find(|s| s.0 == cur).map(|s| s.1.clone()).unwrap_or_else(|| "(choisir)".into());
                        egui::ComboBox::from_id_salt("ptimg").selected_text(nm).show_ui(ui, |ui| {
                            for (id, n) in &sprites {
                                if ui.selectable_label(*id == cur, n).clicked() {
                                    ed.pt["image"] = json!(id);
                                    ed.changed();
                                }
                            }
                        });
                        ui.end_row();
                    }
                    ui.label(egui::RichText::new("Mélange").color(C::MUTED()));
                    ui.horizontal(|ui| {
                        let add = ed.pt.s("blend") == "add";
                        if ui.selectable_label(!add, "Normal").clicked() && add {
                            ed.pt["blend"] = json!("normal");
                            ed.changed();
                        }
                        if ui.selectable_label(add, "Lumineux").on_hover_text("Les couleurs s’additionnent : feu, étincelles, magie").clicked() && !add {
                            ed.pt["blend"] = json!("add");
                            ed.changed();
                        }
                    });
                    ui.end_row();
                    ui.label(egui::RichText::new("Fondu").color(C::MUTED()));
                    let mut f = ed.pt.get("fade") != Some(&Json::Bool(false));
                    if ui.checkbox(&mut f, "s’efface en fin de vie").changed() {
                        ed.pt["fade"] = json!(f);
                        ed.changed();
                    }
                    ui.end_row();
                    ui.label(egui::RichText::new("Couleurs").color(C::MUTED())).on_hover_text("Chaque particule prend une couleur au hasard dans la liste");
                    ui.horizontal_wrapped(|ui| {
                        let mut cols: Vec<String> = ed.pt.arr("colors").iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect();
                        let mut ch = false;
                        let mut del = None;
                        for (k, c) in cols.iter_mut().enumerate() {
                            let mut col = parse(c);
                            let r = ui.color_edit_button_srgba(&mut col);
                            if r.changed() {
                                *c = hex(col);
                                ch = true;
                            }
                            if r.secondary_clicked() {
                                del = Some(k);
                            }
                            r.on_hover_text("Clic droit : retirer");
                        }
                        if let Some(k) = del {
                            cols.remove(k);
                            ch = true;
                        }
                        if ui.add(theme::btn_small("+")).on_hover_text("Ajouter une couleur").clicked() {
                            cols.push(cols.last().cloned().unwrap_or_else(|| "#ffffff".into()));
                            ch = true;
                        }
                        if ch {
                            ed.pt["colors"] = json!(cols);
                            ed.changed();
                        }
                    });
                    ui.end_row();
                    if ed.pt.s("mode") == "burst" {
                        ui.label(egui::RichText::new("À la fin").color(C::MUTED()));
                        let mut d = ed.pt.get("destroyEnd") != Some(&Json::Bool(false));
                        if ui.checkbox(&mut d, "détruire l’émetteur").on_hover_text("Supprime l’objet quand la rafale est terminée").changed() {
                            ed.pt["destroyEnd"] = json!(d);
                            ed.dirty = true;
                        }
                        ui.end_row();
                    }
                });
            }); });
            ui.separator();
            // ---- aperçu
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    theme::section(ui, "Aperçu");
                    for (k, (n, _)) in BGS.iter().enumerate() {
                        if ui.selectable_label(ed.bg == k, *n).clicked() {
                            ed.bg = k;
                            ed.stale = true;
                        }
                    }
                    if ui.add(theme::btn_small("↻ Rejouer")).clicked() {
                        ed.stale = true;
                    }
                    if ui.add(theme::btn_small(if ed.paused { "▶" } else { "⏸" })).clicked() {
                        ed.paused = !ed.paused;
                    }
                });
                if ed.stale {
                    ed.build(app);
                }
                let dt = ed.last.elapsed().as_secs_f64().min(0.1);
                ed.last = std::time::Instant::now();
                if !ed.paused {
                    if let Some(rt) = ed.rt.as_mut() {
                        rt.frame(dt);
                        ed.age += dt;
                    }
                    // rafale : on recommence quand elle est finie
                    let dur = ed.pt.f_or("life", 1.0) + ed.pt.f("lifeRand", 0.0) + 0.5;
                    if ed.pt.s("mode") == "burst" && ed.age > dur {
                        ed.build(app);
                    }
                }
                if let Some(rt) = ed.rt.as_mut() {
                    rt.render(&mut ed.g);
                    let rgba = pf_runtime::img::to_rgba(&ed.g.pm);
                    let im = egui::ColorImage::from_rgba_unmultiplied([PW as usize, PH as usize], &rgba);
                    match ed.tex.as_mut() {
                        Some(t) => t.set(im, egui::TextureOptions::LINEAR),
                        None => ed.tex = Some(ctx.load_texture("ptprev", im, egui::TextureOptions::LINEAR)),
                    }
                }
                if let Some(t) = &ed.tex {
                    let aw = ui.available_width().max(200.0);
                    let s = (aw / PW as f32).min((ui.available_height() - 40.0).max(120.0) / PH as f32).clamp(0.4, 2.0);
                    ui.image((t.id(), egui::vec2(PW as f32 * s, PH as f32 * s)));
                }
                ui.label(egui::RichText::new("L’aperçu utilise le moteur du jeu : c’est exactement le rendu final.").size(11.0).color(C::DIM()));
                if !ed.paused {
                    ctx.request_repaint();
                }
            });
        });
    });
    if save {
        app.snapshot();
        let oid = ed.oid.clone();
        let pt = ed.pt.clone();
        if let Some(o) = app.obj_mut(&oid) {
            o["pt"] = pt;
        }
        ed.dirty = false;
        app.changed();
        app.toast("Particules enregistrées.", false, now);
    }
    if open && !close {
        app.part_ed = Some(ed);
    }
}
