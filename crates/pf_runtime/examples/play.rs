//! Lance un projet sans fenêtre : `play <projet.json> <ticks> [script] [sortie.png]`
//! script : "10:+ArrowRight,40:-ArrowRight,50:+Space,52:-Space,60:click:100:200"
use pf_runtime::{gfx::Gfx, Options, Runtime, NullHost};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let p: pf_core::Json = serde_json::from_str(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let ticks: i64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let script = a.get(3).cloned().unwrap_or_default();
    let mut ev: Vec<(i64, String)> = script.split(',').filter(|s| !s.is_empty()).filter_map(|s| { let (t, r) = s.split_once(':')?; Some((t.parse().ok()?, r.to_string())) }).collect();
    ev.sort_by_key(|e| e.0);
    let mut rt = Runtime::new(p, Options { seed: Some(42), lcg: if std::env::var("PF_LCG").is_ok() { Some(42) } else { None }, ..Default::default() }, Box::new(NullHost));
    let mut g0 = Gfx::new(rt.w as u32, rt.h as u32);
    if std::env::var("PF_RENDER").is_ok() { rt.render(&mut g0); }
    let t0 = std::time::Instant::now();
    for t in 0..ticks {
        for (_, e) in ev.iter().filter(|e| e.0 == t) {
            if let Some(k) = e.strip_prefix('+') { rt.key_down(k, k, false); }
            else if let Some(k) = e.strip_prefix('-') { rt.key_up(k); }
            else if let Some(r) = e.strip_prefix("click:") { let (x, y) = r.split_once(':').unwrap(); let (x, y) = (x.parse().unwrap(), y.parse().unwrap()); rt.mouse_down("left", x, y); rt.step(pf_runtime::DT); rt.mouse_up("left"); continue; }
        }
        rt.step(pf_runtime::DT);
        if std::env::var("PF_RENDER").is_ok() { rt.render(&mut g0); }
    }
    let el = t0.elapsed();
    println!("layout={} tick={} count={} errors={:?} ({:.2} ms/step)", rt.layout_name(), rt.tick, rt.all.len(), rt.errors, el.as_secs_f64() * 1000.0 / ticks.max(1) as f64);
    for (k, v) in &rt.globals { print!("{}={} ", k, v); }
    println!();
    for name in a.get(5).map(|s| s.split('|').collect::<Vec<_>>()).unwrap_or_default() {
        for i in rt.find(name) { println!("{} x={:.2} y={:.2}", name, i.x, i.y); }
    }
    if let Some(out) = a.get(4).filter(|s| !s.is_empty()) {
        let mut g = Gfx::new(rt.w as u32, rt.h as u32);
        let t1 = std::time::Instant::now();
        rt.render(&mut g);
        println!("render {:.1} ms", t1.elapsed().as_secs_f64() * 1000.0);
        pf_runtime::img::save_png(&g.pm, std::path::Path::new(out)).unwrap();
    }
}
