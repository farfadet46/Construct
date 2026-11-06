//! Décodage des images (PNG, JPEG, GIF, WebP…) en Pixmap tiny-skia prémultipliée.
use tiny_skia::Pixmap;

pub fn decode(bytes: &[u8]) -> Option<Pixmap> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    let mut pm = Pixmap::new(w.max(1), h.max(1))?;
    let dst = pm.data_mut();
    for (k, px) in img.pixels().enumerate() {
        let a = px[3] as u32;
        let o = k * 4;
        dst[o] = ((px[0] as u32 * a + 127) / 255) as u8;
        dst[o + 1] = ((px[1] as u32 * a + 127) / 255) as u8;
        dst[o + 2] = ((px[2] as u32 * a + 127) / 255) as u8;
        dst[o + 3] = a as u8;
    }
    Some(pm)
}

/// Pixmap prémultipliée → RGBA non prémultiplié (pour l'affichage / l'enregistrement)
pub fn to_rgba(pm: &Pixmap) -> Vec<u8> {
    let src = pm.data();
    let mut out = vec![0u8; src.len()];
    for k in (0..src.len()).step_by(4) {
        let a = src[k + 3] as u32;
        if a == 0 {
            continue;
        }
        out[k] = ((src[k] as u32 * 255 + a / 2) / a).min(255) as u8;
        out[k + 1] = ((src[k + 1] as u32 * 255 + a / 2) / a).min(255) as u8;
        out[k + 2] = ((src[k + 2] as u32 * 255 + a / 2) / a).min(255) as u8;
        out[k + 3] = a as u8;
    }
    out
}

pub fn save_png(pm: &Pixmap, path: &std::path::Path) -> Result<(), String> {
    let rgba = to_rgba(pm);
    image::save_buffer(path, &rgba, pm.width(), pm.height(), image::ColorType::Rgba8).map_err(|e| e.to_string())
}
