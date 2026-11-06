//! PlatForge — socle commun : valeurs, expressions, catalogue, accès au projet (JSON).
pub mod catalog;
pub mod expr;
pub mod value;

pub use serde_json::{json, Map, Value as Json};
pub use value::Val;

/// Petites aides de lecture d'un projet JSON (identiques aux `||` de la version HTML).
pub trait JsonExt {
    fn s(&self, k: &str) -> &str;
    fn f(&self, k: &str, d: f64) -> f64;
    /// `o.k||d` : 0 ou absent → d
    fn f_or(&self, k: &str, d: f64) -> f64;
    fn b(&self, k: &str, d: bool) -> bool;
    fn arr(&self, k: &str) -> &[Json];
    fn has(&self, k: &str) -> bool;
}
static EMPTY: Vec<Json> = Vec::new();
impl JsonExt for Json {
    fn s(&self, k: &str) -> &str {
        self.get(k).and_then(|v| v.as_str()).unwrap_or("")
    }
    fn f(&self, k: &str, d: f64) -> f64 {
        match self.get(k) {
            Some(Json::Number(n)) => n.as_f64().unwrap_or(d),
            Some(Json::String(s)) => {
                let v = value::str_to_num(s);
                if v.is_nan() {
                    d
                } else {
                    v
                }
            }
            Some(Json::Bool(b)) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => d,
        }
    }
    fn f_or(&self, k: &str, d: f64) -> f64 {
        let v = self.f(k, 0.0);
        if v == 0.0 || v.is_nan() {
            d
        } else {
            v
        }
    }
    fn b(&self, k: &str, d: bool) -> bool {
        match self.get(k) {
            Some(Json::Bool(b)) => *b,
            Some(Json::Null) | None => d,
            Some(Json::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
            Some(Json::String(s)) => !s.is_empty(),
            Some(_) => true,
        }
    }
    fn arr(&self, k: &str) -> &[Json] {
        self.get(k).and_then(|v| v.as_array()).map(|v| v.as_slice()).unwrap_or(&EMPTY)
    }
    fn has(&self, k: &str) -> bool {
        matches!(self.get(k), Some(v) if !v.is_null())
    }
}

/// décode une URL `data:...;base64,` en octets
pub fn data_url_bytes(d: &str) -> Option<Vec<u8>> {
    let p = d.find(',')?;
    let head = &d[..p];
    let body = &d[p + 1..];
    if head.ends_with(";base64") {
        b64_decode(body)
    } else {
        Some(pct_decode(body))
    }
}
fn pct_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut o = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                o.push(v);
                i += 3;
                continue;
            }
        }
        o.push(b[i]);
        i += 1;
    }
    o
}
pub fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut n = 0;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        } as u32;
        buf = (buf << 6) | v;
        n += 6;
        if n >= 8 {
            n -= 8;
            out.push((buf >> n) as u8);
            buf &= (1 << n) - 1;
        }
    }
    Some(out)
}
pub fn b64_encode(b: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::with_capacity(b.len() * 4 / 3 + 4);
    for ch in b.chunks(3) {
        let v = (ch[0] as u32) << 16 | (*ch.get(1).unwrap_or(&0) as u32) << 8 | *ch.get(2).unwrap_or(&0) as u32;
        o.push(T[(v >> 18 & 63) as usize] as char);
        o.push(T[(v >> 12 & 63) as usize] as char);
        o.push(if ch.len() > 1 { T[(v >> 6 & 63) as usize] as char } else { '=' });
        o.push(if ch.len() > 2 { T[(v & 63) as usize] as char } else { '=' });
    }
    o
}

/// identifiant aléatoire au format de l'éditeur ("o_xxxxxxxx")
pub fn uid(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static C: AtomicU64 = AtomicU64::new(0);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
    let mut x = t ^ C.fetch_add(0x9E3779B97F4A7C15, Ordering::Relaxed);
    let mut s = String::from(prefix);
    s.push('_');
    for _ in 0..8 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let d = (x % 36) as u8;
        s.push(if d < 10 { (b'0' + d) as char } else { (b'a' + d - 10) as char });
    }
    s
}
