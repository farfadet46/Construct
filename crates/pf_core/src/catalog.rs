//! Catalogue des comportements, conditions, actions et expressions.
//! Extrait tel quel de runtime.js (`catalog.json`) pour garder les mêmes libellés et paramètres.
use serde_json::Value;
use std::sync::OnceLock;

static CAT: OnceLock<Value> = OnceLock::new();

pub fn catalog() -> &'static Value {
    CAT.get_or_init(|| serde_json::from_str(include_str!("catalog.json")).expect("catalog.json"))
}
pub fn behaviors() -> &'static serde_json::Map<String, Value> {
    catalog()["BEHAVIORS"].as_object().unwrap()
}
pub fn conds() -> &'static serde_json::Map<String, Value> {
    catalog()["CONDS"].as_object().unwrap()
}
pub fn acts() -> &'static serde_json::Map<String, Value> {
    catalog()["ACTS"].as_object().unwrap()
}
/// paramètres par défaut d'un comportement (bdefaults)
pub fn bdefaults(t: &str) -> serde_json::Map<String, Value> {
    let mut o = serde_json::Map::new();
    if let Some(d) = behaviors().get(t) {
        if let Some(p) = d["p"].as_object() {
            for (k, v) in p {
                o.insert(k.clone(), v.get("d").cloned().unwrap_or(Value::Null));
            }
        }
    }
    o
}
/// définition d'une condition / action à partir de sa clé ("system.gset", "b:platform.floor")
pub fn ace_def(acts_not_conds: bool, key: &str) -> Option<&'static Value> {
    let tab = if acts_not_conds { acts() } else { conds() };
    let p = key.rfind('.')?;
    tab.get(&key[..p]).and_then(|m| m.get(&key[p + 1..]))
}
pub fn sysexp() -> Vec<&'static str> {
    catalog()["SYSEXP"].as_array().unwrap().iter().filter_map(|v| v.as_str()).collect()
}
pub fn objexp() -> Vec<&'static str> {
    catalog()["OBJEXP"].as_array().unwrap().iter().filter_map(|v| v.as_str()).collect()
}
pub fn keys() -> Vec<(&'static str, &'static str)> {
    catalog()["KEYS"].as_array().unwrap().iter().map(|k| (k[0].as_str().unwrap(), k[1].as_str().unwrap())).collect()
}
pub fn provname(k: &str) -> &'static str {
    catalog()["PROVNAME"].get(k).and_then(|v| v.as_str()).unwrap_or("")
}

// ---- catalogue d'affichage de l'éditeur (français ou traduit) ; le moteur garde `catalog()`
static UI_EN: OnceLock<Value> = OnceLock::new();
static EN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// installe la version traduite (mêmes clés et valeurs, libellés traduits)
pub fn install_ui_en(v: Value) {
    let _ = UI_EN.set(v);
}
pub fn set_ui_en(on: bool) {
    EN.store(on, std::sync::atomic::Ordering::Relaxed);
}
pub fn ui_catalog() -> &'static Value {
    if EN.load(std::sync::atomic::Ordering::Relaxed) {
        if let Some(v) = UI_EN.get() {
            return v;
        }
    }
    catalog()
}
pub fn ui_behaviors() -> &'static serde_json::Map<String, Value> {
    ui_catalog()["BEHAVIORS"].as_object().unwrap()
}
pub fn ui_conds() -> &'static serde_json::Map<String, Value> {
    ui_catalog()["CONDS"].as_object().unwrap()
}
pub fn ui_acts() -> &'static serde_json::Map<String, Value> {
    ui_catalog()["ACTS"].as_object().unwrap()
}
pub fn ui_ace_def(acts_not_conds: bool, key: &str) -> Option<&'static Value> {
    let tab = if acts_not_conds { ui_acts() } else { ui_conds() };
    let p = key.rfind('.')?;
    tab.get(&key[..p]).and_then(|m| m.get(&key[p + 1..]))
}
pub fn ui_keys() -> Vec<(&'static str, &'static str)> {
    ui_catalog()["KEYS"].as_array().unwrap().iter().map(|k| (k[0].as_str().unwrap(), k[1].as_str().unwrap())).collect()
}
pub fn ui_provname(k: &str) -> &'static str {
    ui_catalog()["PROVNAME"].get(k).and_then(|v| v.as_str()).unwrap_or("")
}
