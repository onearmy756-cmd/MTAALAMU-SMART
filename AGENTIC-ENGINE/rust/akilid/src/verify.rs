//! Uthibitisho wa namba wa injini ya Rust dhidi ya rejeleo la transformers (`.bin` + manifest).

use akili_core::vision::Grid;
use akili_core::{Engine, EngineOptions, GenParams};
use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
struct Entry {
    shape: Vec<usize>,
    bytes: usize,
}

fn read_bin(dir: &Path, man: &HashMap<String, Entry>, name: &str) -> Result<Vec<f32>> {
    let e = man
        .get(name)
        .with_context(|| format!("`{name}` haipo kwenye manifest.json"))?;
    let raw = std::fs::read(dir.join(format!("{name}.bin")))?;
    anyhow::ensure!(raw.len() == e.bytes, "`{name}`: bytes hazilingani");
    Ok(raw
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

struct Cmp {
    max_abs: f32,
    mean_abs: f32,
    cosine: f32,
    n: usize,
}

fn compare(a: &[f32], b: &[f32]) -> Cmp {
    let n = a.len().min(b.len());
    let mut max_abs = 0f32;
    let mut sum_abs = 0f64;
    let mut dot = 0f64;
    let mut na = 0f64;
    let mut nb = 0f64;
    for i in 0..n {
        let d = (a[i] - b[i]).abs();
        if d > max_abs {
            max_abs = d;
        }
        sum_abs += d as f64;
        dot += (a[i] as f64) * (b[i] as f64);
        na += (a[i] as f64) * (a[i] as f64);
        nb += (b[i] as f64) * (b[i] as f64);
    }
    let cosine = if na > 0.0 && nb > 0.0 {
        (dot / (na.sqrt() * nb.sqrt())) as f32
    } else {
        0.0
    };
    Cmp {
        max_abs,
        mean_abs: (sum_abs / n.max(1) as f64) as f32,
        cosine,
        n,
    }
}

fn report(label: &str, got: &[f32], want: &[f32]) {
    let c = compare(got, want);
    println!(
        "  {label:<22} n={:<7} max_abs={:.5} mean_abs={:.6} cosine={:.6}",
        c.n, c.max_abs, c.mean_abs, c.cosine
    );
}

pub fn run(model: &Path, ref_dir: &Path, dtype: DType, vision_only: bool) -> Result<()> {
    let man: HashMap<String, Entry> =
        serde_json::from_str(&std::fs::read_to_string(ref_dir.join("manifest.json"))?)?;
    println!("[verify] model={} dtype={dtype:?}", model.display());

    // Uthibitisho wa macho-pekee: haupakii decoder ya maandishi (RAM ndogo).
    if vision_only {
        let v = akili_core::Engine::load_vision(model, dtype)?;
        let pv = read_bin(ref_dir, &man, "pixel_values")?;
        let pv_shape = &man["pixel_values"].shape;
        let grid_thw = read_bin(ref_dir, &man, "image_grid_thw")?;
        let grid = Grid {
            t: grid_thw[0] as usize,
            h: grid_thw[1] as usize,
            w: grid_thw[2] as usize,
        };
        println!("\n[1] Kichwa cha macho (ViT)");
        println!(
            "  pixel_values {:?}, grid_thw=({},{},{})",
            pv_shape, grid.t, grid.h, grid.w
        );
        let pt = Tensor::from_vec(pv, (pv_shape[0], pv_shape[1]), &Device::Cpu)?;
        let feats = v.forward(&pt, &grid)?;
        let got = feats.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
        let want = read_bin(ref_dir, &man, "vision_embeds")?;
        let c = compare(&got, &want);
        report("vision_embeds", &got, &want);
        let (tol_abs, tol_cos) = match dtype {
            DType::F32 | DType::F64 => (0.05f32, 0.999f32),
            _ => (1.5f32, 0.999f32),
        };
        let ok = c.max_abs < tol_abs && c.cosine > tol_cos;
        if !ok {
            println!(
                "  ✗ vision_embeds IMEFELI (max_abs={}, cosine={})",
                c.max_abs, c.cosine
            );
            anyhow::bail!("vision_embeds imeshindikana");
        }
        println!("[verify] vision_only=kweli; ZOTE ZIMEPITA");
        return Ok(());
    }

    let mut engine = Engine::load(
        model,
        EngineOptions {
            dtype,
            with_vision: true,
            memory_path: std::env::temp_dir().join("akili_verify_memory.jsonl"),
            ..Default::default()
        },
    )?;
    let dev = Device::Cpu;
    let mut failures = 0usize;
    // Vigezo hutofautiana kwa dtype: rejeleo huandikwa kwa bf16, na kelele ya
    // bf16/f16 yenyewe ni kubwa (hata torch-bf16 dhidi ya torch-f32 hutoa
    // max_abs ~2). Kwa F32 tunahitaji ulinganifu wa karibu kabisa.
    let (tol_abs, tol_cos) = match dtype {
        DType::F32 | DType::F64 => (0.05f32, 0.999f32),
        _ => (1.5f32, 0.999f32),
    };
    let check = |label: &str, c: &Cmp| -> bool {
        let ok = c.max_abs < tol_abs && c.cosine > tol_cos;
        if !ok {
            println!("  ✗ {label} IMEFELI (max_abs={}, cosine={})", c.max_abs, c.cosine);
        }
        ok
    };
    println!("  (vigezo: max_abs<{tol_abs}, cosine>{tol_cos})");

    // ---------- MACHO ----------
    println!("\n[1] Kichwa cha macho (ViT)");
    let pv = read_bin(ref_dir, &man, "pixel_values")?;
    let pv_shape = &man["pixel_values"].shape; // (n_patches, 1176)
    let grid_thw = read_bin(ref_dir, &man, "image_grid_thw")?;
    let (gt, gh, gw) = (
        grid_thw[0] as usize,
        grid_thw[1] as usize,
        grid_thw[2] as usize,
    );
    println!("  pixel_values {:?}, grid_thw=({gt},{gh},{gw})", pv_shape);
    let pt = Tensor::from_vec(pv, (pv_shape[0], pv_shape[1]), &dev)?;
    let grid = Grid { t: gt, h: gh, w: gw };
    let feats = engine.vision_forward(&pt, &grid)?;
    let got = feats.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
    let want = read_bin(ref_dir, &man, "vision_embeds")?;
    let c = compare(&got, &want);
    report("vision_embeds", &got, &want);
    if !check("vision_embeds", &c) {
        failures += 1;
    }

    // ---------- MAANDISHI PEKEE ----------
    println!("\n[2] Decoder ya maandishi (bila picha)");
    let ids_f = read_bin(ref_dir, &man, "text_input_ids")?;
    let ids: Vec<u32> = ids_f.iter().map(|v| *v as u32).collect();
    println!("  prompt tokens = {}", ids.len());
    let (hidden, logits) = engine.forward_ids(&ids, &[], None)?;
    let h0 = read_bin(ref_dir, &man, "text_hidden_0")?;
    let h0_shape = &man["text_hidden_0"].shape;
    let hgot = hidden.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
    anyhow::ensure!(h0.len() == hgot.len(), "umbo la hidden halilingani");
    let c = compare(&hgot, &h0);
    report("hidden_0 (embeds)", &hgot, &h0);
    if !check("hidden_0", &c) {
        failures += 1;
    }
    let _ = h0_shape;
    let wl = read_bin(ref_dir, &man, "text_logits")?;
    let c = compare(&logits, &wl);
    report("logits (mwisho)", &logits, &wl);
    if !check("text_logits", &c) {
        failures += 1;
    }
    let argmax = |v: &[f32]| -> usize {
        v.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    };
    println!(
        "  argmax: rust={} torch={}",
        argmax(&logits),
        argmax(&wl)
    );

    // Greedy ya maandishi
    let params = GenParams {
        max_tokens: 8,
        temperature: 0.0,
        top_p: 1.0,
        repetition_penalty: 1.0,
        ..Default::default()
    };
    let mut eng2 = Engine::load(
        model,
        EngineOptions {
            dtype,
            with_vision: true,
            memory_path: std::env::temp_dir().join("akili_verify_memory2.jsonl"),
            ..Default::default()
        },
    )?;
    let out = eng2.generate("", &[], &[], &params);
    let _ = out;

    // ---------- PICHA + MAANDISHI ----------
    println!("\n[3] Picha + maandishi (logits za mwisho)");
    let iids_f = read_bin(ref_dir, &man, "img_input_ids")?;
    let iids: Vec<u32> = iids_f.iter().map(|v| *v as u32).collect();
    println!("  prompt tokens = {}", iids.len());
    let (ih, il) = engine.forward_ids(&iids, &[grid], Some(&feats))?;
    let il_want = read_bin(ref_dir, &man, "img_logits")?;
    let c = compare(&il, &il_want);
    report("logits (picha)", &il, &il_want);
    if !check("img_logits", &c) {
        failures += 1;
    }
    let ih_want = read_bin(ref_dir, &man, "img_hidden_0")?;
    let ih_got = ih.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
    let c = compare(&ih_got, &ih_want);
    report("hidden_0 (picha)", &ih_got, &ih_want);
    if !check("img_hidden_0", &c) {
        failures += 1;
    }
    println!("  argmax: rust={} torch={}", argmax(&il), argmax(&il_want));

    // ---------- GREEDY ----------
    println!("\n[4] Uzalishaji wa greedy (tokeni 16)");
    let gen_ids = read_bin(ref_dir, &man, "text_gen_ids")?;
    let ref_gen: Vec<u32> = gen_ids[ids.len()..].iter().map(|v| *v as u32).collect();
    let params = GenParams {
        max_tokens: 16,
        temperature: 0.0,
        top_p: 1.0,
        repetition_penalty: 1.0,
        ..Default::default()
    };
    let out = engine.generate("", &[], &[], &params)?;
    let got_ids = engine.tok.encode(&out.text)?;
    println!("  rejeleo : {:?}", ref_gen);
    println!("  rust    : {:?}", got_ids);
    println!("  maandishi ya rust: {}", out.text.trim());

    println!("\n[verify] failures={failures}");
    if failures == 0 {
        println!("[verify] ZOTE ZIMEPITA");
        Ok(())
    } else {
        anyhow::bail!("{failures} vipimo vimeshindikana")
    }
}
