//! Sampuli ya tokeni: temperature, top-p (nucleus), repetition penalty.

use crate::GenParams;

/// Huchagua tokeni inayofuata kutoka logits (huhariri logits kwa repetition penalty).
pub fn sample(logits: &mut [f32], p: &GenParams, generated: &[u32]) -> u32 {
    if p.repetition_penalty != 1.0 {
        let penalty = p.repetition_penalty;
        for &t in generated {
            let idx = t as usize;
            if idx < logits.len() {
                if logits[idx] > 0.0 {
                    logits[idx] /= penalty;
                } else {
                    logits[idx] *= penalty;
                }
            }
        }
    }

    let (mut best, mut best_v) = (0u32, f32::NEG_INFINITY);
    for (i, &v) in logits.iter().enumerate() {
        if v > best_v {
            best_v = v;
            best = i as u32;
        }
    }

    if p.temperature <= 1e-4 {
        return best;
    }

    // Softmax
    let max = best_v;
    let mut probs: Vec<f32> = logits.iter().map(|&v| ((v - max) / p.temperature).exp()).collect();
    let sum: f32 = probs.iter().sum();
    if !sum.is_finite() || sum <= 0.0 {
        return best;
    }
    for v in probs.iter_mut() {
        *v /= sum;
    }

    // Top-p
    if p.top_p < 1.0 {
        let mut idx: Vec<usize> = (0..probs.len()).collect();
        idx.sort_by(|a, b| probs[*b].partial_cmp(&probs[*a]).unwrap_or(std::cmp::Ordering::Equal));
        let mut cum = 0f32;
        let mut keep = probs.len();
        for (k, &i) in idx.iter().enumerate() {
            cum += probs[i];
            if cum >= p.top_p {
                keep = k + 1;
                break;
            }
        }
        let kept: std::collections::HashSet<usize> = idx[..keep].iter().copied().collect();
        let mut total = 0f32;
        for (i, v) in probs.iter_mut().enumerate() {
            if !kept.contains(&i) {
                *v = 0.0;
            } else {
                total += *v;
            }
        }
        if total <= 0.0 {
            return best;
        }
        for v in probs.iter_mut() {
            *v /= total;
        }
    }

    // Sampuli kwa RNG rahisi (xorshift inayotokana na seed + urefu)
    let mut state = p.seed ^ (generated.len() as u64).wrapping_mul(0x9E3779B97F4A7C15);
    if state == 0 {
        state = 0x1234_5678_9abc_def0;
    }
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    let r = (state >> 11) as f32 / (1u64 << 53) as f32;
    let mut acc = 0f32;
    for (i, &v) in probs.iter().enumerate() {
        acc += v;
        if r <= acc {
            return i as u32;
        }
    }
    best
}
