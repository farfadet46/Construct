//! Langage d'expressions de PlatForge (même grammaire, mêmes messages d'erreur
//! et même sémantique que `tokenize`/`parse`/`compile` de runtime.js).

use crate::value::{js_round, parse_float, parse_int, Val};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    N(String),
    S(String),
    I(String),
    O(&'static str),
}

const OPS3: [&str; 5] = ["<>", "<=", ">=", "!=", "=="];
const OPS1: &str = "-+*/%^&|=<>(),.?:";

fn op_static(s: &str) -> &'static str {
    match s {
        "<>" => "<>",
        "<=" => "<=",
        ">=" => ">=",
        "!=" => "!=",
        "==" => "==",
        "-" => "-",
        "+" => "+",
        "*" => "*",
        "/" => "/",
        "%" => "%",
        "^" => "^",
        "&" => "&",
        "|" => "|",
        "=" => "=",
        "<" => "<",
        ">" => ">",
        "(" => "(",
        ")" => ")",
        "," => ",",
        "." => ".",
        "?" => "?",
        ":" => ":",
        _ => "?",
    }
}

fn is_word_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || ('\u{C0}'..='\u{24F}').contains(&c)
}
fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || ('\u{C0}'..='\u{24F}').contains(&c)
}
fn js_ws(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    loop {
        while i < cs.len() && js_ws(cs[i]) {
            i += 1;
        }
        if i >= cs.len() {
            break;
        }
        let c = cs[i];
        // number: \d+(\.\d+)? | \.\d+
        if c.is_ascii_digit() || (c == '.' && i + 1 < cs.len() && cs[i + 1].is_ascii_digit()) {
            let st = i;
            if c == '.' {
                i += 1;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            } else {
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
                if i + 1 < cs.len() && cs[i] == '.' && cs[i + 1].is_ascii_digit() {
                    i += 1;
                    while i < cs.len() && cs[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            out.push(Tok::N(cs[st..i].iter().collect()));
            continue;
        }
        if c == '"' {
            let mut j = i + 1;
            let mut v = String::new();
            let mut closed = false;
            while j < cs.len() {
                if cs[j] == '"' {
                    if j + 1 < cs.len() && cs[j + 1] == '"' {
                        v.push('"');
                        j += 2;
                        continue;
                    }
                    closed = true;
                    j += 1;
                    break;
                }
                v.push(cs[j]);
                j += 1;
            }
            if !closed {
                return Err("Guillemet non fermé".into());
            }
            out.push(Tok::S(v));
            i = j;
            continue;
        }
        if is_word_start(c) {
            let st = i;
            i += 1;
            while i < cs.len() && is_word(cs[i]) {
                i += 1;
            }
            out.push(Tok::I(cs[st..i].iter().collect()));
            continue;
        }
        if i + 1 < cs.len() {
            let two: String = cs[i..i + 2].iter().collect();
            if OPS3.contains(&two.as_str()) {
                out.push(Tok::O(op_static(&two)));
                i += 2;
                continue;
            }
        }
        if OPS1.contains(c) {
            out.push(Tok::O(op_static(&c.to_string())));
            i += 1;
            continue;
        }
        return Err(format!("Caractère inattendu : « {} »", c));
    }
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bin {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Cat,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit(Val),
    Global(String),
    Obj(String, String),
    Meth(String, String, Vec<Expr>),
    Call(String, Vec<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin(Bin, Box<Expr>, Box<Expr>),
    Ter(Box<Expr>, Box<Expr>, Box<Expr>),
}

pub const EXF: [&str; 29] = [
    "random", "floor", "ceil", "round", "abs", "sqrt", "min", "max", "sin", "cos", "clamp", "lerp", "choose", "int", "float", "str", "len",
    "distance", "angle", "zeropad", "upper", "highscore", "highscorename", "highscorecount", "call", "__a", "storage", "loopindex", "lower",
];

struct Parser {
    tk: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn pk(&self) -> Option<&Tok> {
        self.tk.get(self.pos)
    }
    fn is_op(&self, v: &str) -> bool {
        matches!(self.pk(), Some(Tok::O(o)) if *o == v)
    }
    fn is_w(&self, w: &str) -> bool {
        matches!(self.pk(), Some(Tok::I(s)) if s.to_lowercase() == w)
    }
    fn expect(&mut self, v: &str) -> Result<(), String> {
        if !self.is_op(v) {
            return Err(format!("« {} » attendu", v));
        }
        self.pos += 1;
        Ok(())
    }
    fn ter(&mut self) -> Result<Expr, String> {
        let c = self.or()?;
        if self.is_op("?") {
            self.pos += 1;
            let a = self.ter()?;
            self.expect(":")?;
            let b = self.ter()?;
            return Ok(Expr::Ter(Box::new(c), Box::new(a), Box::new(b)));
        }
        Ok(c)
    }
    fn or(&mut self) -> Result<Expr, String> {
        let mut l = self.and()?;
        while self.is_w("ou") || self.is_w("or") || self.is_op("|") {
            self.pos += 1;
            let r = self.and()?;
            l = Expr::Bin(Bin::Or, Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut l = self.comp()?;
        while self.is_w("et") || self.is_w("and") {
            self.pos += 1;
            let r = self.comp()?;
            l = Expr::Bin(Bin::And, Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn comp(&mut self) -> Result<Expr, String> {
        let l = self.cat()?;
        if let Some(Tok::O(o)) = self.pk() {
            let op = match *o {
                "=" | "==" => Some(Bin::Eq),
                "<>" | "!=" => Some(Bin::Ne),
                "<" => Some(Bin::Lt),
                ">" => Some(Bin::Gt),
                "<=" => Some(Bin::Le),
                ">=" => Some(Bin::Ge),
                _ => None,
            };
            if let Some(op) = op {
                self.pos += 1;
                let r = self.cat()?;
                return Ok(Expr::Bin(op, Box::new(l), Box::new(r)));
            }
        }
        Ok(l)
    }
    fn cat(&mut self) -> Result<Expr, String> {
        let mut l = self.add()?;
        while self.is_op("&") {
            self.pos += 1;
            let r = self.add()?;
            l = Expr::Bin(Bin::Cat, Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn add(&mut self) -> Result<Expr, String> {
        let mut l = self.mul()?;
        loop {
            let op = if self.is_op("+") {
                Bin::Add
            } else if self.is_op("-") {
                Bin::Sub
            } else {
                break;
            };
            self.pos += 1;
            let r = self.mul()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn mul(&mut self) -> Result<Expr, String> {
        let mut l = self.un()?;
        loop {
            let op = if self.is_op("*") {
                Bin::Mul
            } else if self.is_op("/") {
                Bin::Div
            } else if self.is_op("%") {
                Bin::Mod
            } else {
                break;
            };
            self.pos += 1;
            let r = self.un()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn un(&mut self) -> Result<Expr, String> {
        if self.is_op("-") {
            self.pos += 1;
            return Ok(Expr::Neg(Box::new(self.un()?)));
        }
        if self.is_op("+") {
            self.pos += 1;
            return self.un();
        }
        if self.is_w("non") || self.is_w("not") {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.un()?)));
        }
        self.pw()
    }
    fn pw(&mut self) -> Result<Expr, String> {
        let b = self.prim()?;
        if self.is_op("^") {
            self.pos += 1;
            let e = self.un()?;
            return Ok(Expr::Bin(Bin::Pow, Box::new(b), Box::new(e)));
        }
        Ok(b)
    }
    fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut a = Vec::new();
        if !self.is_op(")") {
            a.push(self.ter()?);
            while self.is_op(",") {
                self.pos += 1;
                a.push(self.ter()?);
            }
        }
        self.expect(")")?;
        Ok(a)
    }
    fn prim(&mut self) -> Result<Expr, String> {
        let t = match self.tk.get(self.pos).cloned() {
            Some(t) => t,
            None => return Err("Expression incomplète".into()),
        };
        self.pos += 1;
        match t {
            Tok::N(s) => Ok(Expr::Lit(Val::N(num_literal(&s)))),
            Tok::S(s) => Ok(Expr::Lit(Val::S(s))),
            Tok::O("(") => {
                let e = self.ter()?;
                self.expect(")")?;
                Ok(e)
            }
            Tok::I(name) => {
                if self.is_op("(") {
                    self.pos += 1;
                    let a = self.args()?;
                    let f = name.to_lowercase();
                    if !EXF.contains(&f.as_str()) {
                        return Err(format!("Fonction inconnue : {}()", name));
                    }
                    return Ok(Expr::Call(f, a));
                }
                if self.is_op(".") {
                    self.pos += 1;
                    let p = match self.tk.get(self.pos).cloned() {
                        Some(Tok::I(p)) => p,
                        _ => return Err(format!("Propriété attendue après « {}. »", name)),
                    };
                    self.pos += 1;
                    if self.is_op("(") {
                        self.pos += 1;
                        let a = self.args()?;
                        return Ok(Expr::Meth(name, p, a));
                    }
                    return Ok(Expr::Obj(name, p));
                }
                Ok(Expr::Global(name))
            }
            Tok::O(o) => Err(format!("« {} » inattendu", o)),
        }
    }
}

/// littéral numérique tel que l'interpréterait JS (mode non strict : 010 = octal 8)
fn num_literal(s: &str) -> f64 {
    if s.len() > 1 && s.starts_with('0') && !s.contains('.') {
        if s.chars().all(|c| ('0'..='7').contains(&c)) {
            return i64::from_str_radix(&s[1..], 8).unwrap_or(0) as f64;
        }
    }
    s.parse::<f64>().unwrap_or(0.0)
}

fn tok_text(t: &Tok) -> String {
    match t {
        Tok::N(s) | Tok::S(s) | Tok::I(s) => s.clone(),
        Tok::O(o) => o.to_string(),
    }
}

pub fn parse(src: &str) -> Result<Expr, String> {
    let tk = tokenize(src)?;
    let mut p = Parser { tk, pos: 0 };
    let e = p.ter()?;
    if p.pos < p.tk.len() {
        return Err(format!("« {} » inattendu", tok_text(&p.tk[p.pos])));
    }
    Ok(e)
}

/// Ce que le moteur doit fournir pour évaluer une expression.
pub trait Env {
    fn global(&mut self, name: &str) -> Val;
    fn obj(&mut self, name: &str, prop: &str) -> Val;
    fn meth(&mut self, name: &str, meth: &str, args: &[Val]) -> Val;
    /// fonctions remplacées par le moteur (call, storage, highscore…, loopindex) ; None = défaut
    fn func(&mut self, _name: &str, _args: &[Val]) -> Option<Val> {
        None
    }
    fn random(&mut self) -> f64;
}

fn arg(a: &[Val], i: usize) -> Val {
    a.get(i).cloned().unwrap_or(Val::N(f64::NAN))
}

/// fonctions EXF par défaut
pub fn call_default(env: &mut dyn Env, f: &str, a: &[Val]) -> Val {
    let n = |i: usize| a.get(i).map(|v| v.num()).unwrap_or(f64::NAN);
    match f {
        "random" => {
            let x = a.first().map(|v| v.num0()).unwrap_or(0.0);
            if a.len() < 2 {
                Val::N(env.random() * x)
            } else {
                Val::N(x + env.random() * (n(1) - x))
            }
        }
        "floor" => Val::N(n(0).floor()),
        "ceil" => Val::N(n(0).ceil()),
        "round" => Val::N(js_round(n(0))),
        "abs" => Val::N(n(0).abs()),
        "sqrt" => Val::N(n(0).sqrt()),
        "min" => {
            let mut m = f64::INFINITY;
            for v in a {
                let x = v.num();
                if x.is_nan() {
                    return Val::N(f64::NAN);
                }
                if x < m {
                    m = x;
                }
            }
            Val::N(m)
        }
        "max" => {
            let mut m = f64::NEG_INFINITY;
            for v in a {
                let x = v.num();
                if x.is_nan() {
                    return Val::N(f64::NAN);
                }
                if x > m {
                    m = x;
                }
            }
            Val::N(m)
        }
        "sin" => Val::N((n(0) * std::f64::consts::PI / 180.0).sin()),
        "cos" => Val::N((n(0) * std::f64::consts::PI / 180.0).cos()),
        "clamp" => {
            let (v, lo, hi) = (n(0), n(1), n(2));
            let m = if v.is_nan() || hi.is_nan() { f64::NAN } else { hi.min(v) };
            Val::N(if m.is_nan() || lo.is_nan() { f64::NAN } else { lo.max(m) })
        }
        "lerp" => {
            let av = arg(a, 0);
            let d = (n(1) - av.num()) * n(2);
            av.add(&Val::N(d))
        }
        "choose" => {
            if a.is_empty() {
                return Val::N(0.0);
            }
            let k = (env.random() * a.len() as f64) as usize;
            a[k.min(a.len() - 1)].clone()
        }
        "int" => Val::N(parse_int(&arg(a, 0))),
        "float" => Val::N(parse_float(&arg(a, 0))),
        "str" => Val::S(a.first().map(|v| v.str()).unwrap_or_else(|| "undefined".into())),
        "len" => Val::N(a.first().map(|v| v.str()).unwrap_or_else(|| "undefined".into()).encode_utf16().count() as f64),
        "distance" => Val::N((n(2) - n(0)).hypot(n(3) - n(1))),
        "angle" => Val::N((n(3) - n(1)).atan2(n(2) - n(0)) * 180.0 / std::f64::consts::PI),
        "zeropad" => {
            let mut s = Val::N(n(0).floor()).str();
            let d = arg(a, 1);
            while Val::N(s.encode_utf16().count() as f64).lt(&d) {
                s.insert(0, '0');
            }
            Val::S(s)
        }
        "upper" => Val::S(a.first().map(|v| v.str()).unwrap_or_else(|| "undefined".into()).to_uppercase()),
        "lower" => Val::S(a.first().map(|v| v.str()).unwrap_or_else(|| "undefined".into()).to_lowercase()),
        "highscore" | "highscorecount" | "call" | "loopindex" => Val::N(0.0),
        "highscorename" => Val::S(String::new()),
        "storage" => a.get(1).cloned().unwrap_or(Val::N(0.0)),
        _ => Val::N(0.0),
    }
}

pub fn eval(e: &Expr, env: &mut dyn Env) -> Val {
    match e {
        Expr::Lit(v) => v.clone(),
        Expr::Global(n) => env.global(n),
        Expr::Obj(n, p) => env.obj(n, p),
        Expr::Meth(n, p, a) => {
            let av: Vec<Val> = a.iter().map(|x| eval(x, env)).collect();
            env.meth(n, p, &av)
        }
        Expr::Call(f, a) => {
            let av: Vec<Val> = a.iter().map(|x| eval(x, env)).collect();
            if let Some(v) = env.func(f, &av) {
                return v;
            }
            call_default(env, f, &av)
        }
        Expr::Neg(x) => Val::N(-eval(x, env).num()),
        Expr::Not(x) => Val::from(!eval(x, env).truthy()),
        Expr::Ter(c, a, b) => {
            if eval(c, env).truthy() {
                eval(a, env)
            } else {
                eval(b, env)
            }
        }
        Expr::Bin(op, l, r) => {
            match op {
                Bin::And => {
                    let a = eval(l, env);
                    if !a.truthy() {
                        return Val::N(0.0);
                    }
                    return Val::from(eval(r, env).truthy());
                }
                Bin::Or => {
                    let a = eval(l, env);
                    if a.truthy() {
                        return Val::N(1.0);
                    }
                    return Val::from(eval(r, env).truthy());
                }
                _ => {}
            }
            let a = eval(l, env);
            let b = eval(r, env);
            match op {
                Bin::Add => a.add(&b),
                Bin::Sub => Val::N(a.num() - b.num()),
                Bin::Mul => Val::N(a.num() * b.num()),
                Bin::Div => Val::N(a.num() / b.num()),
                Bin::Mod => Val::N(a.num() % b.num()),
                Bin::Pow => Val::N(a.num().powf(b.num())),
                Bin::Cat => {
                    let mut s = a.str();
                    s.push_str(&b.as_str_cow());
                    Val::S(s)
                }
                Bin::Eq => Val::from(a.loose_eq(&b)),
                Bin::Ne => Val::from(!a.loose_eq(&b)),
                Bin::Lt => Val::from(a.lt(&b)),
                Bin::Gt => Val::from(a.gt(&b)),
                Bin::Le => Val::from(a.le(&b)),
                Bin::Ge => Val::from(a.ge(&b)),
                Bin::And | Bin::Or => unreachable!(),
            }
        }
    }
}

/// cache de compilation (équivalent de XC)
#[derive(Default)]
pub struct Cache {
    map: HashMap<String, Rc<Result<Expr, String>>>,
}

impl Cache {
    pub fn get(&mut self, src: &str) -> Rc<Result<Expr, String>> {
        if let Some(e) = self.map.get(src) {
            return e.clone();
        }
        let r = Rc::new(parse(src));
        self.map.insert(src.to_string(), r.clone());
        r
    }
}

/// Visite toutes les références d'une expression (pour la vérification dans l'éditeur).
pub fn walk<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Expr)) {
    f(e);
    match e {
        Expr::Meth(_, _, a) | Expr::Call(_, a) => a.iter().for_each(|x| walk(x, f)),
        Expr::Neg(x) | Expr::Not(x) => walk(x, f),
        Expr::Bin(_, a, b) => {
            walk(a, f);
            walk(b, f)
        }
        Expr::Ter(a, b, c) => {
            walk(a, f);
            walk(b, f);
            walk(c, f)
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct E;
    impl Env for E {
        fn global(&mut self, n: &str) -> Val {
            if n == "Score" {
                Val::N(5.0)
            } else {
                Val::N(0.0)
            }
        }
        fn obj(&mut self, _: &str, p: &str) -> Val {
            if p == "X" {
                Val::N(10.0)
            } else {
                Val::N(0.0)
            }
        }
        fn meth(&mut self, _: &str, _: &str, _: &[Val]) -> Val {
            Val::N(0.0)
        }
        fn random(&mut self) -> f64 {
            0.5
        }
    }
    fn ev(s: &str) -> String {
        eval(&parse(s).unwrap(), &mut E).str()
    }
    #[test]
    fn basics() {
        assert_eq!(ev("1+2*3"), "7");
        assert_eq!(ev("\"Score: \" & Score"), "Score: 5");
        assert_eq!(ev("\"a\" + 1 + 2"), "a12");
        assert_eq!(ev("1 + 2 + \"a\""), "3a");
        assert_eq!(ev("Score > 3 et Score < 10"), "1");
        assert_eq!(ev("non 0"), "1");
        assert_eq!(ev("2^3^2"), "512");
        assert_eq!(ev("-2^2"), "-4");
        assert_eq!(ev("Joueur.X + 1"), "11");
        assert_eq!(ev("Score = 5 ? \"oui\" : \"non\""), "oui");
        assert_eq!(ev("zeropad(7,3)"), "007");
        assert_eq!(ev("round(2.5)"), "3");
        assert_eq!(ev("round(-2.5)"), "-2");
        assert_eq!(ev("\"a\"\"b\""), "a\"b");
        assert_eq!(ev("10/4"), "2.5");
        assert_eq!(ev("1/0"), "Infinity");
        assert_eq!(ev("\"10\" < \"9\""), "1");
        assert_eq!(ev("\"10\" < 9"), "0");
        assert!(parse("foo(").is_err());
        assert_eq!(parse("bar(1)").unwrap_err(), "Fonction inconnue : bar()");
        assert_eq!(parse("1 2").unwrap_err(), "« 2 » inattendu");
        assert_eq!(parse("\"abc").unwrap_err(), "Guillemet non fermé");
        assert_eq!(parse("1 # 2").unwrap_err(), "Caractère inattendu : « # »");
    }
}
