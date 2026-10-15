//! Usindikaji wa picha: smart_resize, bicubic resize (yenye antialias), normalize,
//! na patchify ya Qwen2.5-VL.

use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};

pub const CLIP_MEAN: [f32; 3] = [0.48145466, 0.4578275, 0.40821073];
pub const CLIP_STD: [f32; 3] = [0.26862954, 0.26130258, 0.27577711];

/// Picha ya RGB yenye data planar (3, h, w) f32 kwenye [0,1].
pub struct PlanarImage {
    pub data: Vec<f32>,
    pub w: usize,
    pub h: usize,
}

pub fn smart_resize(
    height: usize,
    width: usize,
    factor: usize,
    min_pixels: u64,
    max_pixels: u64,
) -> Result<(usize, usize)> {
    anyhow::ensure!(height > 0 && width > 0, "picha haina ukubwa halali");
    let ratio = height.max(width) as f64 / height.min(width) as f64;
    anyhow::ensure!(ratio <= 200.0, "uwiano wa picha ni mkubwa mno ({ratio:.1})");
    let round_to = |v: usize| -> usize { ((v as f64 / factor as f64).round() as usize) * factor };
    let mut h_bar = round_to(height);
    let mut w_bar = round_to(width);
    let (min_pixels, max_pixels) = (min_pixels as f64, max_pixels as f64);
    if ((h_bar * w_bar) as f64) > max_pixels {
        let beta = ((height * width) as f64 / max_pixels).sqrt();
        h_bar = factor.max((((height as f64) / beta / factor as f64).floor() as usize) * factor);
        w_bar = factor.max((((width as f64) / beta / factor as f64).floor() as usize) * factor);
    } else if ((h_bar * w_bar) as f64) < min_pixels {
        let beta = (min_pixels / (height * width) as f64).sqrt();
        h_bar = (((height as f64) * beta / factor as f64).ceil() as usize) * factor;
        w_bar = (((width as f64) * beta / factor as f64).ceil() as usize) * factor;
    }
    Ok((h_bar, w_bar))
}

/// Hupakia faili la picha (JPEG/PNG/WEBP/BMP) kama RGB planar yenye [0,1].
pub fn load_image(path: &std::path::Path) -> Result<PlanarImage> {
    let img = image::open(path).with_context(|| format!("picha `{}` haipatikani", path.display()))?;
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    let raw = rgb.into_raw();
    let mut data = vec![0f32; 3 * w * h];
    for i in 0..(w * h) {
        for c in 0..3 {
            data[c * w * h + i] = raw[i * 3 + c] as f32 / 255.0;
        }
    }
    Ok(PlanarImage { data, w, h })
}

/// Hupakia picha kutoka bytes (kwa HTTP upload).
pub fn load_image_bytes(bytes: &[u8]) -> Result<PlanarImage> {
    let img = image::load_from_memory(bytes).context("bytes za picha si sahihi")?;
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    let raw = rgb.into_raw();
    let mut data = vec![0f32; 3 * w * h];
    for i in 0..(w * h) {
        for c in 0..3 {
            data[c * w * h + i] = raw[i * 3 + c] as f32 / 255.0;
        }
    }
    Ok(PlanarImage { data, w, h })
}

fn cubic(x: f32, a: f32) -> f32 {
    let x = x.abs();
    if x <= 1.0 {
        (a + 2.0) * x * x * x - (a + 3.0) * x * x + 1.0
    } else if x < 2.0 {
        a * x * x * x - 5.0 * a * x * x + 8.0 * a * x - 4.0 * a
    } else {
        0.0
    }
}

/// Resize ya channel moja kwa bicubic (a = -0.5) yenye antialias kwa kupunguza ukubwa.
fn resize_channel(src: &[f32], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<f32> {
    let a = -0.5f32;
    let mut tmp = vec![0f32; dw * sh];
    // Mhimili wa X
    let scale_x = sw as f32 / dw as f32;
    let radius_x = if scale_x > 1.0 { 2.0 * scale_x } else { 2.0 };
    for y in 0..sh {
        for x in 0..dw {
            let center = (x as f32 + 0.5) * scale_x - 0.5;
            let lo = (center - radius_x).ceil() as i64;
            let hi = (center + radius_x).floor() as i64;
            let mut acc = 0f32;
            let mut wsum = 0f32;
            for i in lo..=hi {
                let mut w = cubic((i as f32 - center) / scale_x.max(1.0), a);
                if scale_x > 1.0 {
                    w /= scale_x;
                }
                let xi = i.clamp(0, sw as i64 - 1) as usize;
                acc += w * src[y * sw + xi];
                wsum += w;
            }
            tmp[y * dw + x] = if wsum != 0.0 { acc / wsum } else { 0.0 };
        }
    }
    // Mhimili wa Y
    let scale_y = sh as f32 / dh as f32;
    let radius_y = if scale_y > 1.0 { 2.0 * scale_y } else { 2.0 };
    let mut out = vec![0f32; dw * dh];
    for y in 0..dh {
        let center = (y as f32 + 0.5) * scale_y - 0.5;
        let lo = (center - radius_y).ceil() as i64;
        let hi = (center + radius_y).floor() as i64;
        for x in 0..dw {
            let mut acc = 0f32;
            let mut wsum = 0f32;
            for i in lo..=hi {
                let mut w = cubic((i as f32 - center) / scale_y.max(1.0), a);
                if scale_y > 1.0 {
                    w /= scale_y;
                }
                let yi = i.clamp(0, sh as i64 - 1) as usize;
                acc += w * tmp[yi * dw + x];
                wsum += w;
            }
            out[y * dw + x] = if wsum != 0.0 { acc / wsum } else { 0.0 };
        }
    }
    out
}

pub struct Patchified {
    /// (n_tokens, channel * temporal * ps * ps) f32
    pub patches: Vec<f32>,
    pub grid_t: usize,
    pub grid_h: usize,
    pub grid_w: usize,
    pub patch_dim: usize,
}

/// Huandaa picha moja: smart_resize -> bicubic -> normalize -> patchify.
pub fn preprocess(
    img: &PlanarImage,
    patch_size: usize,
    temporal: usize,
    merge: usize,
    min_pixels: u64,
    max_pixels: u64,
) -> Result<Patchified> {
    let factor = patch_size * merge;
    let (th, tw) = smart_resize(img.h, img.w, factor, min_pixels, max_pixels)?;
    let gh = th / patch_size;
    let gw = tw / patch_size;

    // Resize + normalize kwa kila channel
    let mut norm = vec![0f32; 3 * th * tw];
    for c in 0..3 {
        let src: Vec<f32> = img.data[c * img.w * img.h..(c + 1) * img.w * img.h].to_vec();
        let r = resize_channel(&src, img.w, img.h, tw, th);
        for i in 0..(th * tw) {
            norm[c * th * tw + i] = (r[i] - CLIP_MEAN[c]) / CLIP_STD[c];
        }
    }

    // Patchify: token order = (gh/merge, gw/merge, merge_h, merge_w)
    let n_tokens = gh * gw;
    let patch_dim = 3 * temporal * patch_size * patch_size;
    let mut patches = vec![0f32; n_tokens * patch_dim];
    let bh = gh / merge;
    let bw = gw / merge;
    for bi in 0..bh {
        for bj in 0..bw {
            for k in 0..merge {
                for l in 0..merge {
                    let token = (bi * bw + bj) * (merge * merge) + k * merge + l;
                    let h0 = (bi * merge + k) * patch_size;
                    let w0 = (bj * merge + l) * patch_size;
                    let dst = &mut patches[token * patch_dim..(token + 1) * patch_dim];
                    let mut p = 0usize;
                    for c in 0..3 {
                        for _t in 0..temporal {
                            for ph in 0..patch_size {
                                for pw in 0..patch_size {
                                    dst[p] = norm[c * th * tw + (h0 + ph) * tw + (w0 + pw)];
                                    p += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(Patchified {
        patches,
        grid_t: 1,
        grid_h: gh,
        grid_w: gw,
        patch_dim,
    })
}

pub fn to_tensor(p: &Patchified, dev: &Device) -> Result<Tensor> {
    Ok(Tensor::from_vec(p.patches.clone(), (p.grid_h * p.grid_w, p.patch_dim), dev)?.to_dtype(DType::F32)?)
}
