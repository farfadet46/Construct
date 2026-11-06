//! Sons du jeu : effets intégrés, fichiers importés, musique (fondus), étiquettes et moteur de voiture.
//! Implémente `Host` du moteur (avec la sauvegarde persistante sur disque).

use pf_core::{Json, JsonExt, Map};
use pf_runtime::synth::{self, Engine, EngineIn, RATE};
use pf_runtime::{AudioCmd, Host};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct EngSrc {
    eng: Engine,
    inp: Arc<Mutex<(EngineIn, Instant, bool)>>,
    buf: Vec<f32>,
    pos: usize,
}
impl Iterator for EngSrc {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.buf.len() {
            let (mut inp, last, stop) = *self.inp.lock().unwrap();
            if stop {
                return None;
            }
            if last.elapsed() > Duration::from_millis(120) {
                inp.idle = true;
            }
            self.buf.resize(256, 0.0);
            self.eng.render(&inp, &mut self.buf);
            self.pos = 0;
        }
        let v = self.buf[self.pos];
        self.pos += 1;
        Some(v)
    }
}
impl Source for EngSrc {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct AudioHost {
    _stream: Option<OutputStream>,
    h: Option<OutputStreamHandle>,
    master: f32,
    muted: bool,
    sfx_cache: HashMap<String, Arc<Vec<f32>>>,
    sounds: HashMap<String, Arc<Vec<u8>>>,
    voices: Vec<(Sink, String)>,
    music: Option<(String, Sink, f32)>,
    fading: Vec<(Sink, f32, Instant, f32)>,
    engines: HashMap<u32, Arc<Mutex<(EngineIn, Instant, bool)>>>,
    storage_dir: std::path::PathBuf,
}

pub fn data_dir() -> std::path::PathBuf {
    let base = std::env::var_os("APPDATA").map(std::path::PathBuf::from).or_else(|| std::env::var_os("XDG_DATA_HOME").map(std::path::PathBuf::from)).or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))).unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("PlatForge")
}

impl AudioHost {
    pub fn new(project: &Json) -> AudioHost {
        let (stream, h) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };
        let mut sounds = HashMap::new();
        for s in project.arr("sounds") {
            if let Some(b) = pf_core::data_url_bytes(s.s("data")) {
                sounds.insert(s.s("id").to_string(), Arc::new(b));
            }
        }
        AudioHost { _stream: stream, h, master: 1.0, muted: false, sfx_cache: HashMap::new(), sounds, voices: Vec::new(), music: None, fading: Vec::new(), engines: HashMap::new(), storage_dir: data_dir().join("saves") }
    }
    fn gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.master
        }
    }
    fn sink(&self) -> Option<Sink> {
        self.h.as_ref().and_then(|h| Sink::try_new(h).ok())
    }
    fn file_source(&self, id: &str) -> Option<rodio::Decoder<std::io::Cursor<Vec<u8>>>> {
        let b = self.sounds.get(id)?;
        rodio::Decoder::new(std::io::Cursor::new((**b).clone())).ok()
    }
    /// à appeler régulièrement : fondus de musique, nettoyage des voix terminées
    pub fn tick_audio(&mut self) {
        self.voices.retain(|v| !v.0.empty());
        let now = Instant::now();
        self.fading.retain(|(s, from, t0, d)| {
            let k = now.duration_since(*t0).as_secs_f32() / d.max(0.001);
            if k >= 1.0 {
                s.stop();
                return false;
            }
            s.set_volume(from * (1.0 - k));
            true
        });
        let g = self.gain();
        if let Some((_, s, v)) = &self.music {
            if s.volume() < *v * g {
                s.set_volume((s.volume() + 0.02).min(*v * g));
            }
        }
    }
    pub fn stop_all(&mut self) {
        for v in self.voices.drain(..) {
            v.0.stop();
        }
        if let Some((_, s, _)) = self.music.take() {
            s.stop();
        }
        for (_, e) in self.engines.drain() {
            e.lock().unwrap().2 = true;
        }
    }
}

impl Drop for AudioHost {
    fn drop(&mut self) {
        self.stop_all();
    }
}

impl Host for AudioHost {
    fn audio(&mut self, c: AudioCmd) {
        let g = self.gain();
        match c {
            AudioCmd::Sfx { name, vol } => {
                if self.muted {
                    return;
                }
                let buf = self.sfx_cache.entry(name.clone()).or_insert_with(|| Arc::new(synth::sfx(&name))).clone();
                if buf.is_empty() {
                    return;
                }
                if let Some(s) = self.sink() {
                    s.set_volume(vol as f32 * g);
                    s.append(rodio::buffer::SamplesBuffer::new(1, RATE, (*buf).clone()));
                    self.voices.push((s, String::new()));
                }
            }
            AudioCmd::PlayFile { id, vol, looping, tag } => {
                if let (Some(src), Some(s)) = (self.file_source(&id), self.sink()) {
                    s.set_volume(vol.max(0.0) as f32 * g);
                    if looping {
                        s.append(src.repeat_infinite());
                    } else {
                        s.append(src);
                    }
                    self.voices.push((s, tag));
                }
            }
            AudioCmd::Music { id, vol, fade } => {
                if let Some((mid, s, v)) = &mut self.music {
                    if *mid == id {
                        *v = vol as f32;
                        s.set_volume(vol as f32 * g);
                        return;
                    }
                }
                if let Some((_, s, v)) = self.music.take() {
                    self.fading.push((s, v * g, Instant::now(), fade as f32));
                }
                if let (Some(src), Some(s)) = (self.file_source(&id), self.sink()) {
                    s.set_volume(if fade > 0.0 { 0.0 } else { vol as f32 * g });
                    s.append(src.repeat_infinite());
                    self.music = Some((id, s, vol as f32));
                }
            }
            AudioCmd::StopMusic { fade } => {
                if let Some((_, s, v)) = self.music.take() {
                    if fade > 0.0 {
                        self.fading.push((s, v * g, Instant::now(), fade as f32));
                    } else {
                        s.stop();
                    }
                }
            }
            AudioCmd::StopTag(t) => {
                for v in &self.voices {
                    if v.1 == t {
                        v.0.stop();
                    }
                }
            }
            AudioCmd::StopAll => {
                for v in self.voices.drain(..) {
                    v.0.stop();
                }
                if let Some((_, s, _)) = self.music.take() {
                    s.stop();
                }
            }
            AudioCmd::Volume(v) => {
                self.master = v as f32;
                let g = self.gain();
                if let Some((_, s, mv)) = &self.music {
                    s.set_volume(*mv * g);
                }
            }
            AudioCmd::Mute(m) => {
                self.muted = m;
                let g = self.gain();
                if let Some((_, s, mv)) = &self.music {
                    s.set_volume(*mv * g);
                }
                for v in &self.voices {
                    if m {
                        v.0.set_volume(0.0);
                    }
                }
            }
            AudioCmd::Engine { uid, v, load, screech, rumble } => {
                if self.muted {
                    return;
                }
                let inp = EngineIn { v: v as f32, load: load as f32, screech: screech as f32, rumble: rumble as f32, idle: false };
                if let Some(e) = self.engines.get(&uid) {
                    let mut l = e.lock().unwrap();
                    l.0 = inp;
                    l.1 = Instant::now();
                    return;
                }
                let st = Arc::new(Mutex::new((inp, Instant::now(), false)));
                if let Some(s) = self.sink() {
                    s.set_volume(g);
                    s.append(EngSrc { eng: Engine::new(), inp: st.clone(), buf: Vec::new(), pos: 0 });
                    s.detach();
                    self.engines.insert(uid, st);
                }
            }
            AudioCmd::EngineStopAll => {
                for (_, e) in self.engines.drain() {
                    e.lock().unwrap().2 = true;
                }
            }
        }
    }
    fn tick(&mut self) {
        self.tick_audio();
    }
    fn tag_playing(&self, tag: &str) -> bool {
        self.voices.iter().any(|v| v.1 == tag && !v.0.empty())
    }
    fn storage_load(&mut self, ns: &str) -> Map<String, Json> {
        let p = self.storage_dir.join(format!("{}.json", sanitize(ns)));
        std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str::<Json>(&s).ok()).and_then(|v| v.as_object().cloned()).unwrap_or_default()
    }
    fn storage_save(&mut self, ns: &str, m: &Map<String, Json>) {
        let _ = std::fs::create_dir_all(&self.storage_dir);
        let p = self.storage_dir.join(format!("{}.json", sanitize(ns)));
        let _ = std::fs::write(p, serde_json::to_string(&Json::Object(m.clone())).unwrap_or_default());
    }
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}
