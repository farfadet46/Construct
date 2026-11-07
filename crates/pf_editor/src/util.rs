pub fn slug(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        let c = match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            o.push(c.to_ascii_lowercase());
        } else if !o.ends_with('-') && !o.is_empty() {
            o.push('-');
        }
    }
    let o = o.trim_end_matches('-').to_string();
    if o.is_empty() {
        "jeu".into()
    } else {
        o
    }
}

/// vrai seulement si l'utilisateur tape dans un champ de texte
/// (un bouton cliqué garde le focus clavier dans egui : il ne doit pas bloquer les raccourcis)
pub fn typing(ctx: &eframe::egui::Context) -> bool {
    match ctx.memory(|m| m.focused()) {
        Some(id) => eframe::egui::text_edit::TextEditState::load(ctx, id).is_some(),
        None => false,
    }
}

/// Largeur d'un panneau latéral avant son affichage.
pub fn panel_width(ctx: &eframe::egui::Context, id: &str) -> Option<f32> {
    eframe::egui::containers::panel::PanelState::load(ctx, eframe::egui::Id::new(id)).map(|s| s.rect.width())
}

/// Un panneau latéral ne doit changer de largeur que si l'utilisateur tire sur son bord.
/// egui l'élargit dès qu'un contenu déborde, ne serait-ce qu'une image (par exemple quand la
/// colonne des libellés d'une grille change de largeur au changement de langue) et ne le
/// rétrécit jamais : on remet donc la largeur d'avant.
pub fn keep_panel_width(ctx: &eframe::egui::Context, id: &str, before: Option<f32>, right_side: bool) {
    use eframe::egui::{containers::panel::PanelState, Id};
    let Some(w0) = before else { return };
    let pid = Id::new(id);
    // le bord est tiré par l'utilisateur (y compris l'image où il relâche la souris)
    let dragging = ctx.read_response(pid.with("__resize")).map(|r| r.dragged() || r.drag_stopped() || r.is_pointer_button_down_on()).unwrap_or(false);
    if dragging {
        return;
    }
    if let Some(mut st) = PanelState::load(ctx, pid) {
        if st.rect.width() > w0 + 0.5 {
            if right_side {
                st.rect.min.x = st.rect.max.x - w0;
            } else {
                st.rect.max.x = st.rect.min.x + w0;
            }
            ctx.data_mut(|d| d.insert_persisted(pid, st));
            ctx.request_repaint();
        }
    }
}
