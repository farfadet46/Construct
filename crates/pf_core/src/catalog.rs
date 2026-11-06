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
