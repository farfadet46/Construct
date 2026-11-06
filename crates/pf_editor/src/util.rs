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
