//! Synthèse des sons intégrés (mêmes recettes que la version HTML) et du moteur de voiture.

pub const RATE: u32 = 44100;

/// [forme, f0, f1, durée, volume, délai]
fn recipe(n: &str) -> &'static [(&'static str, f32, f32, f32, f32, f32)] {
    match n {
        "jump" => &[("square", 220.0, 560.0, 0.12, 0.05, 0.0)],
        "coin" => &[("square", 988.0, 988.0, 0.06, 0.05, 0.0), ("square", 1319.0, 1319.0, 0.18, 0.05, 0.06)],
        "hit" => &[("triangle", 420.0, 80.0, 0.16, 0.14, 0.0)],
        "hurt" => &[("sawtooth", 460.0, 55.0, 0.4, 0.07, 0.0), ("square", 220.0, 40.0, 0.35, 0.04, 0.05)],
        "powerup" => &[("square", 392.0, 392.0, 0.08, 0.05, 0.0), ("square", 523.0, 523.0, 0.08, 0.05, 0.08), ("square", 659.0, 659.0, 0.08, 0.05, 0.16), ("square", 784.0, 784.0, 0.2, 0.05, 0.24)],
        "spring" => &[("sine", 160.0, 980.0, 0.26, 0.13, 0.0)],
        "laser" => &[("square", 1400.0, 200.0, 0.18, 0.05, 0.0)],
        "explosion" => &[("noise", 0.0, 0.0, 0.5, 0.25, 0.0)],
        "blip" => &[("square", 660.0, 660.0, 0.06, 0.05, 0.0)],
        "win" => &[("square", 523.0, 523.0, 0.1, 0.045, 0.0), ("square", 659.0, 659.0, 0.1, 0.045, 0.1), ("square", 784.0, 784.0, 0.1, 0.045, 0.2), ("square", 1047.0, 1047.0, 0.35, 0.045, 0.3)],
        "lose" => &[("triangle", 392.0, 392.0, 0.18, 0.1, 0.0), ("triangle", 330.0, 330.0, 0.18, 0.1, 0.18), ("triangle", 262.0, 190.0, 0.5, 0.1, 0.36)],
        _ => &[],
    }
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// exponentialRampToValueAtTime
fn exp_ramp(a: f32, b: f32, t: f32) -> f32 {
    a * (b / a).powf(t.clamp(0.0, 1.0))
}

/// échantillons mono (44,1 kHz) d'un son intégré
pub fn sfx(n: &str) -> Vec<f32> {
    let parts = recipe(n);
    if parts.is_empty() {
        return Vec::new();
    }
    let total = parts.iter().map(|p| p.5 + p.3 + 0.03).fold(0.0f32, f32::max);
    let len = (total * RATE as f32) as usize + 1;
    let mut out = vec![0f32; len];
    let mut rng = Rng(0x1234_5678);
    let noise: Vec<f32> = {
        let nl = (RATE as f32 * 0.6) as usize;
        (0..nl).map(|k| rng.next() * (1.0 - k as f32 / nl as f32)).collect()
    };
    for &(ty, f0, f1, d, vol, delay) in parts {
        let s0 = (delay * RATE as f32) as usize;
        let n = ((d + 0.03) * RATE as f32) as usize;
        let mut ph = 0f32;
        for k in 0..n {
            let t = k as f32 / RATE as f32;
            let g = if t < 0.01 { exp_ramp(0.0001, vol, t / 0.01) } else if t < d { exp_ramp(vol, 0.0001, (t - 0.01) / (d - 0.01).max(0.001)) } else { 0.0001 };
            let v = if ty == "noise" {
                noise.get(k).copied().unwrap_or(0.0)
            } else {
                let f = exp_ramp(f0.max(1.0), f1.max(1.0), t / d);
                ph = (ph + f / RATE as f32) % 1.0;
                match ty {
                    "square" => {
                        if ph < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    "triangle" => 1.0 - 4.0 * (ph - 0.5).abs(),
                    "sawtooth" => 2.0 * ph - 1.0,
                    _ => (ph * std::f32::consts::TAU).sin(),
                }
            };
            if let Some(o) = out.get_mut(s0 + k) {
                *o += v * g;
            }
        }
    }
    out
}

// ---------------------------------------------------------------- filtres

#[derive(Clone, Copy, Default)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}
impl Biquad {
    pub fn set(&mut self, kind: u8, f: f32, q: f32) {
        let w = std::f32::consts::TAU * (f / RATE as f32).clamp(0.0001, 0.49);
        let (sn, cs) = w.sin_cos();
        let alpha = sn / (2.0 * q.max(0.0001));
        let (b0, b1, b2, a0, a1, a2) = if kind == 0 {
            ((1.0 - cs) / 2.0, 1.0 - cs, (1.0 - cs) / 2.0, 1.0 + alpha, -2.0 * cs, 1.0 - alpha)
        } else {
            // passe-bande (gain de crête = Q, comme WebAudio)
            (alpha * q, 0.0, -alpha * q, 1.0 + alpha, -2.0 * cs, 1.0 - alpha)
        };
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
    pub fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// réglages envoyés par le jeu (engSet)
#[derive(Clone, Copy, Debug, Default)]
pub struct EngineIn {
    pub v: f32,
    pub load: f32,
    pub screech: f32,
    pub rumble: f32,
    /// le jeu ne met plus le moteur à jour : on coupe en douceur
    pub idle: bool,
}

/// moteur synthétisé : onde douce + sous-harmonique, filtre passe-bas, « chug » ; crissement et grondement en bruit filtré
pub struct Engine {
    rpm: f32,
    hz: f32,
    ph1: f32,
    ph2: f32,
    phl: f32,
    vol: f32,
    fcut: f32,
    sg: f32,
    rg: f32,
    bpf: f32,
    lp1: Biquad,
    lp2: Biquad,
    bp: Biquad,
    bp2: Biquad,
    lpn: Biquad,
    rng: Rng,
    wave: Vec<(f32, f32)>,
    k: u32,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Engine {
        let wave = (1..24).map(|k| {
            let amp = 1.0 / (k as f32).powf(1.35) * if k % 2 == 1 { 1.0 } else { 0.6 };
            (amp, k as f32 * 1.7)
        }).collect();
        let mut e = Engine { rpm: 0.2, hz: 32.0, ph1: 0.0, ph2: 0.0, phl: 0.0, vol: 0.0, fcut: 320.0, sg: 0.0, rg: 0.0, bpf: 2300.0, lp1: Biquad::default(), lp2: Biquad::default(), bp: Biquad::default(), bp2: Biquad::default(), lpn: Biquad::default(), rng: Rng(0x9e37_79b9), wave, k: 0 };
        e.lp2.set(0, 1600.0, 0.3);
        e.bp2.set(1, 3100.0, 11.0);
        e.lpn.set(0, 180.0, 0.5);
        e
    }
    /// remplit `out` (mono) ; `inp` est lu une fois par bloc
    pub fn render(&mut self, inp: &EngineIn, out: &mut [f32]) {
        let n = out.len() as f32 / RATE as f32;
        // engSet : rpm suit la vitesse sur 5 rapports
        let gears = 5.0;
        let v = inp.v.clamp(0.0, 1.5);
        let gi = (v * gears).floor().min(gears - 1.0);
        let w = v * gears - gi;
        let rpm = if v < 0.02 { 0.12 } else if gi > 0.0 { 0.38 + 0.62 * w } else { 0.15 + 0.85 * w };
        self.rpm += (rpm - self.rpm) * if rpm < self.rpm { 0.06 } else { 0.12 } * (n * 60.0).min(4.0);
        let tgt_hz = 26.0 + self.rpm * 92.0;
        let tgt_f = 200.0 + self.rpm * 520.0 + inp.load * 380.0;
        let tgt_vol = if inp.idle { 0.0 } else { 0.05 + 0.035 * self.rpm + 0.03 * inp.load };
        let tgt_sg = if inp.idle { 0.0 } else { inp.screech.clamp(0.0, 1.0) * 0.028 };
        let tgt_rg = if inp.idle { 0.0 } else { inp.rumble.clamp(0.0, 1.0) * 0.12 };
        let tgt_bp = 2000.0 + inp.screech.min(1.0) * 700.0;
        let kf = |tau: f32| 1.0 - (-1.0 / (RATE as f32 * tau)).exp();
        let (k06, k08, k05, k10) = (kf(0.06), kf(0.08), kf(0.05), kf(0.1));
        for o in out.iter_mut() {
            self.hz += (tgt_hz - self.hz) * k06;
            self.fcut += (tgt_f - self.fcut) * k08;
            let kv = if inp.idle { k05 } else { k08 };
            self.vol += (tgt_vol - self.vol) * kv;
            self.sg += (tgt_sg - self.sg) * kv;
            self.rg += (tgt_rg - self.rg) * kv;
            self.bpf += (tgt_bp - self.bpf) * k10;
            if self.k % 64 == 0 {
                self.lp1.set(0, self.fcut, 0.6);
                self.bp.set(1, self.bpf, 9.0);
            }
            self.k = self.k.wrapping_add(1);
            self.ph1 = (self.ph1 + self.hz / RATE as f32) % 1.0;
            self.ph2 = (self.ph2 + self.hz * 0.5 / RATE as f32) % 1.0;
            self.phl = (self.phl + self.hz * 0.25 / RATE as f32) % 1.0;
            let mut s1 = 0.0;
            let a = self.ph1 * std::f32::consts::TAU;
            for (k, (amp, phs)) in self.wave.iter().enumerate() {
                let kk = (k + 1) as f32;
                // re*cos + im*sin = amp*cos(k*a - phs)
                s1 += amp * (kk * a - phs).cos();
            }
            let s2 = (self.ph2 * std::f32::consts::TAU).sin() * 0.55;
            let mixg = 0.78 + 0.22 * (self.phl * std::f32::consts::TAU).sin();
            let x = (s1 * 0.5 + s2) * mixg;
            let y = self.lp2.run(self.lp1.run(x)) * self.vol;
            let nz = self.rng.next();
            let scr = (self.bp.run(nz) + self.bp2.run(nz)) * self.sg;
            let rum = self.lpn.run(nz) * self.rg;
            *o = y + scr + rum;
        }
    }
    pub fn silent(&self) -> bool {
        self.vol < 0.0005 && self.sg < 0.0005 && self.rg < 0.0005
    }
}
