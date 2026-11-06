//! Traduction de l'interface (français d'origine → anglais).
//! Les textes du code passent par `tr("…")` ; les modèles avec valeurs par `trf("… {0} …", &[…])`.
//! Le dictionnaire `assets/en.json` associe chaque texte français à sa traduction.
use pf_core::Json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static EN: AtomicBool = AtomicBool::new(false);

fn dict() -> &'static HashMap<&'static str, &'static str> {
    static D: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    D.get_or_init(|| {
        let m: HashMap<String, String> = serde_json::from_str(include_str!("../assets/en.json")).unwrap_or_default();
        m.into_iter().map(|(k, v)| (&*Box::leak(k.into_boxed_str()), &*Box::leak(v.into_boxed_str()))).collect()
    })
}

fn missing(s: &str) {
    if std::env::var_os("PF_I18N_MISSING").is_some() {
        static SEEN: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        let mut v = SEEN.lock().unwrap();
        if !v.iter().any(|x| x == s) {
            eprintln!("[i18n] manquant : {:?}", s);
            v.push(s.to_string());
        }
    }
}

pub fn is_en() -> bool {
    EN.load(Ordering::Relaxed)
}

/// change la langue de l'interface (et des libellés du catalogue)
pub fn set_en(on: bool) {
    EN.store(on, Ordering::Relaxed);
    if on {
        pf_core::catalog::install_ui_en(translate_catalog(pf_core::catalog::catalog()));
    }
    pf_core::catalog::set_ui_en(on);
}

/// langue proposée au premier lancement : celle du système
pub fn system_is_en() -> bool {
    let l = sys_locale::get_locale().unwrap_or_default().to_lowercase();
    !l.is_empty() && !l.starts_with("fr")
}

/// texte fixe du code
pub fn tr(s: &'static str) -> &'static str {
    if !is_en() {
        return s;
    }
    match dict().get(s) {
        Some(t) => t,
        None => {
            missing(s);
            s
        }
    }
}

/// modèle avec valeurs : "Niveau « {0} »" ; `{{` et `}}` donnent des accolades
pub fn trf(key: &'static str, args: &[String]) -> String {
    let t = tr(key);
    let mut out = String::with_capacity(t.len() + 16);
    let b = t.as_bytes();
    let mut i = 0;
    while i < t.len() {
        if t[i..].starts_with("{{") {
            out.push('{');
            i += 2;
        } else if t[i..].starts_with("}}") {
            out.push('}');
            i += 2;
        } else if b[i] == b'{' {
            if let Some(j) = t[i..].find('}') {
                if let Ok(k) = t[i + 1..i + j].parse::<usize>() {
                    out.push_str(args.get(k).map(|s| s.as_str()).unwrap_or(""));
                    i += j + 1;
                    continue;
                }
            }
            out.push('{');
            i += 1;
        } else {
            let c = t[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// copie du catalogue avec les libellés traduits (clés, valeurs et défauts inchangés)
fn translate_catalog(c: &Json) -> Json {
    let mut c = c.clone();
    fn t(v: &mut Json) {
        if let Some(s) = v.as_str() {
            if let Some(x) = dict().get(s) {
                *v = Json::String(x.to_string());
            }
        }
    }
    fn tk(o: &mut Json, keys: &[&str]) {
        for k in keys {
            if let Some(v) = o.get_mut(*k) {
                t(v);
            }
        }
    }
    fn opts(p: &mut Json) {
        if let Some(a) = p.get_mut("o").and_then(|o| o.as_array_mut()) {
            for o in a {
                if let Some(v) = o.get_mut(1) {
                    t(v);
                }
            }
        }
    }
    if let Some(m) = c.get_mut("BEHAVIORS").and_then(|v| v.as_object_mut()) {
        for b in m.values_mut() {
            tk(b, &["n", "d", "hint"]);
            if let Some(p) = b.get_mut("p").and_then(|v| v.as_object_mut()) {
                for q in p.values_mut() {
                    tk(q, &["l", "hint"]);
                    opts(q);
                }
            }
        }
    }
    for tab in ["CONDS", "ACTS"] {
        if let Some(m) = c.get_mut(tab).and_then(|v| v.as_object_mut()) {
            for prov in m.values_mut() {
                if let Some(pm) = prov.as_object_mut() {
                    for a in pm.values_mut() {
                        tk(a, &["n", "t", "cat", "hint"]);
                        if let Some(ps) = a.get_mut("p").and_then(|v| v.as_array_mut()) {
                            for p in ps {
                                tk(p, &["n", "hint"]);
                                opts(p);
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(m) = c.get_mut("PROVNAME").and_then(|v| v.as_object_mut()) {
        for v in m.values_mut() {
            t(v);
        }
    }
    for k in ["KEYS", "SOUNDS", "TILETYPES"] {
        if let Some(a) = c.get_mut(k).and_then(|v| v.as_array_mut()) {
            for x in a {
                if let Some(v) = x.get_mut(1) {
                    t(v);
                }
            }
        }
    }
    if let Some(m) = c.get_mut("ISTYLE").and_then(|v| v.as_object_mut()) {
        for l in m.values_mut() {
            if let Some(a) = l.as_array_mut() {
                for x in a {
                    if let Some(v) = x.get_mut(1) {
                        t(v);
                    }
                }
            }
        }
    }
    if let Some(m) = c.get_mut("PTPRESETS").and_then(|v| v.as_object_mut()) {
        for p in m.values_mut() {
            tk(p, &["n"]);
        }
    }
    c
}
