//! Clavier : touches egui → codes KeyboardEvent.code utilisés par les jeux.
use crate::Play;
use eframe::egui::{self, Key};

pub fn code(k: Key) -> Option<String> {
    let s = match k {
        Key::ArrowLeft => "ArrowLeft",
        Key::ArrowRight => "ArrowRight",
        Key::ArrowUp => "ArrowUp",
        Key::ArrowDown => "ArrowDown",
        Key::Space => "Space",
        Key::Enter => "Enter",
        Key::Escape => "Escape",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Delete => "Delete",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::Minus => "Minus",
        Key::Period => "Period",
        Key::Comma => "Comma",
        _ => {
            let n = k.name();
            if n.len() == 1 {
                let c = n.chars().next().unwrap();
                if c.is_ascii_alphabetic() {
                    return Some(format!("Key{}", c.to_ascii_uppercase()));
                }
                if c.is_ascii_digit() {
                    return Some(format!("Digit{}", c));
                }
            }
            if let Some(f) = n.strip_prefix('F') {
                if f.parse::<u8>().is_ok() {
                    return Some(n.to_string());
                }
            }
            return None;
        }
    };
    Some(s.to_string())
}

/// transmet les touches et la saisie de texte au jeu
pub fn feed(ctx: &egui::Context, p: &mut Play) {
    let focused = ctx.input(|i| i.focused);
    if !focused && p.focused {
        p.rt.blur();
    }
    p.focused = focused;
    let evs = ctx.input(|i| i.events.clone());
    let typing = p.rt.hs_in.is_some();
    for e in evs {
        match e {
            egui::Event::Key { key, physical_key, pressed, repeat, modifiers, .. } => {
                // modificateurs gauche (Maj, Ctrl) comme touches de jeu
                if modifiers.shift != p.shift {
                    p.shift = modifiers.shift;
                    if p.shift { p.rt.key_down("ShiftLeft", "Shift", false) } else { p.rt.key_up("ShiftLeft") }
                }
                if modifiers.ctrl != p.ctrl {
                    p.ctrl = modifiers.ctrl;
                    if p.ctrl { p.rt.key_down("ControlLeft", "Control", false) } else { p.rt.key_up("ControlLeft") }
                }
                let k = physical_key.unwrap_or(key);
                let c = match code(k) {
                    Some(c) => c,
                    None => continue,
                };
                if pressed {
                    if repeat {
                        // la version HTML ignore les répétitions, sauf dans la saisie du nom
                        if typing && matches!(key, Key::Backspace | Key::ArrowLeft | Key::ArrowRight) {
                            p.rt.key_down(&c, &c, false);
                        }
                        continue;
                    }
                    // pendant la saisie du nom, les lettres arrivent par l'événement Text
                    if typing && c.starts_with("Key") || typing && c.starts_with("Digit") || typing && c == "Space" {
                        continue;
                    }
                    let keyname = match c.as_str() {
                        "Space" => " ".to_string(),
                        c => c.to_string(),
                    };
                    p.rt.key_down(&c, &keyname, modifiers.ctrl || modifiers.command);
                } else {
                    p.rt.key_up(&c);
                }
            }
            egui::Event::Text(t) if typing => {
                for ch in t.chars() {
                    p.rt.key_down("", &ch.to_string(), false);
                }
            }
            _ => {}
        }
    }
}
