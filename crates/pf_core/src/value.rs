//! Valeurs dynamiques (nombre ou texte) avec les règles de conversion de JavaScript,
//! pour que les expressions donnent exactement les mêmes résultats que la version HTML.

use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Val {
    N(f64),
    S(String),
}

impl Default for Val {
    fn default() -> Self {
        Val::N(0.0)
    }
}

impl From<f64> for Val {
    fn from(v: f64) -> Self {
        Val::N(v)
    }
}
impl From<i32> for Val {
    fn from(v: i32) -> Self {
        Val::N(v as f64)
    }
}
impl From<usize> for Val {
    fn from(v: usize) -> Self {
        Val::N(v as f64)
    }
}
impl From<bool> for Val {
    fn from(v: bool) -> Self {
        Val::N(if v { 1.0 } else { 0.0 })
    }
}
impl From<String> for Val {
    fn from(v: String) -> Self {
        Val::S(v)
    }
}
impl From<&str> for Val {
    fn from(v: &str) -> Self {
        Val::S(v.to_string())
    }
}

/// `Number(s)` de JavaScript.
pub fn str_to_num(s: &str) -> f64 {
    let t = s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    if t.is_empty() {
        return 0.0;
    }
    let (neg, body) = if let Some(r) = t.strip_prefix('-') {
        (true, r)
    } else if let Some(r) = t.strip_prefix('+') {
        (false, r)
    } else {
        (false, t)
    };
    if body == "Infinity" {
        return if neg { f64::NEG_INFINITY } else { f64::INFINITY };
    }
    if t.len() > 2 && (t.starts_with("0x") || t.starts_with("0X")) {
        return i64::from_str_radix(&t[2..], 16).map(|v| v as f64).unwrap_or(f64::NAN);
    }
    if t.len() > 2 && (t.starts_with("0o") || t.starts_with("0O")) {
        return i64::from_str_radix(&t[2..], 8).map(|v| v as f64).unwrap_or(f64::NAN);
    }
    if t.len() > 2 && (t.starts_with("0b") || t.starts_with("0B")) {
        return i64::from_str_radix(&t[2..], 2).map(|v| v as f64).unwrap_or(f64::NAN);
    }
    // only digits, '.', 'e', sign allowed (rust accepts "inf"/"nan", JS does not)
    let ok = body.chars().all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-');
    if !ok || body.is_empty() {
        return f64::NAN;
    }
    if body.starts_with('e') || body.starts_with('E') || body == "." {
        return f64::NAN;
    }
    match body.parse::<f64>() {
        Ok(v) => {
            if neg {
                -v
            } else {
                v
            }
        }
        Err(_) => f64::NAN,
    }
}

/// `String(n)` de JavaScript.
pub fn num_to_str(n: f64) -> String {
    if n.is_nan() {
        return "NaN".into();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if n == 0.0 {
        return "0".into();
    }
    let a = n.abs();
    if a >= 1e21 || a < 1e-6 {
        let s = format!("{:e}", n);
        // rust: 1e21 / 1.5e-7 ; js: 1e+21 / 1.5e-7
        if let Some(p) = s.find('e') {
            let (m, e) = s.split_at(p);
            let e = &e[1..];
            if e.starts_with('-') {
                return format!("{}e{}", m, e);
            }
            return format!("{}e+{}", m, e);
        }
        return s;
    }
    if n.fract() == 0.0 && a < 9.007_199_254_740_992e15 {
        return format!("{}", n as i64);
    }
    format!("{}", n)
}

impl Val {
    pub fn num(&self) -> f64 {
        match self {
            Val::N(n) => *n,
            Val::S(s) => str_to_num(s),
        }
    }
    /// `+v||0`
    pub fn num0(&self) -> f64 {
        let n = self.num();
        if n.is_nan() {
            0.0
        } else {
            n
        }
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Val::S(_))
    }
    pub fn truthy(&self) -> bool {
        match self {
            Val::N(n) => *n != 0.0 && !n.is_nan(),
            Val::S(s) => !s.is_empty(),
        }
    }
    pub fn str(&self) -> String {
        match self {
            Val::N(n) => num_to_str(*n),
            Val::S(s) => s.clone(),
        }
    }
    pub fn as_str_cow(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Val::N(n) => std::borrow::Cow::Owned(num_to_str(*n)),
            Val::S(s) => std::borrow::Cow::Borrowed(s.as_str()),
        }
    }
    /// JS `a + b`
    pub fn add(&self, o: &Val) -> Val {
        match (self, o) {
            (Val::N(a), Val::N(b)) => Val::N(a + b),
            _ => {
                let mut s = self.str();
                s.push_str(&o.as_str_cow());
                Val::S(s)
            }
        }
    }
    /// JS `a == b`
    pub fn loose_eq(&self, o: &Val) -> bool {
        match (self, o) {
            (Val::S(a), Val::S(b)) => a == b,
            _ => self.num() == o.num(),
        }
    }
    /// JS relational comparison; None when undefined (NaN)
    pub fn js_cmp(&self, o: &Val) -> Option<Ordering> {
        match (self, o) {
            (Val::S(a), Val::S(b)) => {
                let x: Vec<u16> = a.encode_utf16().collect();
                let y: Vec<u16> = b.encode_utf16().collect();
                Some(x.cmp(&y))
            }
            _ => self.num().partial_cmp(&o.num()),
        }
    }
    pub fn lt(&self, o: &Val) -> bool {
        self.js_cmp(o) == Some(Ordering::Less)
    }
    pub fn gt(&self, o: &Val) -> bool {
        self.js_cmp(o) == Some(Ordering::Greater)
    }
    pub fn le(&self, o: &Val) -> bool {
        matches!(self.js_cmp(o), Some(Ordering::Less | Ordering::Equal))
    }
    pub fn ge(&self, o: &Val) -> bool {
        matches!(self.js_cmp(o), Some(Ordering::Greater | Ordering::Equal))
    }
    /// typed(old,v): keeps the type of the previous value
    pub fn typed_like(old: &Val, v: Val) -> Val {
        if old.is_str() {
            Val::S(v.str())
        } else {
            Val::N(v.num0())
        }
    }
    pub fn from_json(v: &serde_json::Value) -> Val {
        match v {
            serde_json::Value::Number(n) => Val::N(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Val::S(s.clone()),
            serde_json::Value::Bool(b) => Val::N(if *b { 1.0 } else { 0.0 }),
            serde_json::Value::Null => Val::N(0.0),
            other => Val::S(other.to_string()),
        }
    }
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Val::N(n) => {
                if n.is_finite() {
                    serde_json::json!(*n)
                } else {
                    serde_json::Value::Null
                }
            }
            Val::S(s) => serde_json::Value::String(s.clone()),
        }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str_cow())
    }
}

/// cmp(a,op,b) de la version HTML (comparaisons lâches de JS)
pub fn cmp_op(a: &Val, op: &str, b: &Val) -> bool {
    match op {
        "=" => a.loose_eq(b),
        "<>" => !a.loose_eq(b),
        "<" => a.lt(b),
        "<=" => a.le(b),
        ">" => a.gt(b),
        ">=" => a.ge(b),
        _ => false,
    }
}

/// JS `Math.round`
pub fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// JS `parseInt(String(v),10)||0`
pub fn parse_int(v: &Val) -> f64 {
    let s = v.str();
    let t = s.trim_start();
    let mut out = String::new();
    let mut chars = t.chars().peekable();
    if let Some(&c) = chars.peek() {
        if c == '-' || c == '+' {
            out.push(c);
            chars.next();
        }
    }
    for c in chars {
        if c.is_ascii_digit() {
            out.push(c);
        } else {
            break;
        }
    }
    out.parse::<f64>().unwrap_or(0.0)
}

/// JS `parseFloat(String(v))||0`
pub fn parse_float(v: &Val) -> f64 {
    let s = v.str();
    let t = s.trim_start();
    if t.starts_with("Infinity") || t.starts_with("+Infinity") {
        return f64::INFINITY;
    }
    if t.starts_with("-Infinity") {
        return f64::NEG_INFINITY;
    }
    let b = t.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == ds || (i == ds + 1 && b[ds] == b'.') {
        return 0.0;
    }
    let mant_end = i;
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let es = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > es {
            i = j;
        } else {
            i = mant_end;
        }
    }
    let v = t[..i].parse::<f64>().unwrap_or(0.0);
    if v.is_nan() {
        0.0
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conv() {
        assert_eq!(num_to_str(3.0), "3");
        assert_eq!(num_to_str(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(num_to_str(1e21), "1e+21");
        assert_eq!(num_to_str(-0.0), "0");
        assert_eq!(str_to_num(" 12 "), 12.0);
        assert!(str_to_num("12px").is_nan());
        assert!(str_to_num("inf").is_nan());
        assert_eq!(str_to_num(""), 0.0);
        assert_eq!(Val::N(1.0).add(&Val::from("a")).str(), "1a");
        assert!(Val::from("2").loose_eq(&Val::N(2.0)));
        assert_eq!(parse_int(&Val::from("12abc")), 12.0);
        assert_eq!(parse_float(&Val::from("3.5e2x")), 350.0);
    }
}
