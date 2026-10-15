//! M-RoPE ya Qwen2.5-VL: cos/sin za pande tatu (temporal, height, width) zilizounganishwa
//! kwa `mrope_section = [16, 24, 24]`.

use anyhow::Result;
use candle_core::{DType, Device, Tensor};

pub struct MRope {
    inv_freq: Tensor,
    pub section: [usize; 3],
}

impl MRope {
    pub fn new(head_dim: usize, rope_theta: f64, section: [usize; 3], dev: &Device) -> Result<Self> {
        let inv: Vec<f32> = (0..head_dim)
            .step_by(2)
            .map(|i| 1f32 / (rope_theta.powf(i as f64 / head_dim as f64) as f32))
            .collect();
        let n = inv.len();
        Ok(Self {
            inv_freq: Tensor::from_vec(inv, (n,), dev)?,
            section,
        })
    }

    /// `positions`: (3, seq) f32/f64 (temporal, height, width).
    /// Hurudisha (cos, sin) zenye umbo (seq, head_dim) f16.
    pub fn cos_sin(&self, positions: &Tensor, dtype: DType) -> Result<(Tensor, Tensor)> {
        let (three, seq) = positions.dims2()?;
        anyhow::ensure!(three == 3, "positions lazima ziwe (3, seq)");
        // (3, seq, n) = positions[..., None] * inv_freq
        let inv = self.inv_freq.unsqueeze(0)?.unsqueeze(0)?; // (1,1,n)
        let freqs = positions
            .to_dtype(DType::F32)?
            .unsqueeze(2)?
            .broadcast_mul(&inv)?; // (3, seq, n)
        let cos = freqs.cos()?;
        let sin = freqs.sin()?;

        let [s0, s1, s2] = self.section;
        let mut cos_parts: Vec<Tensor> = Vec::with_capacity(3);
        let mut sin_parts: Vec<Tensor> = Vec::with_capacity(3);
        // freq.split(section, -1) -> parts za mrope_section; m[i % 3] huchukuliwa
        let bounds = [0usize, s0, s0 + s1, s0 + s1 + s2];
        for i in 0..3usize {
            let width = bounds[i + 1] - bounds[i];
            if width == 0 {
                continue;
            }
            let src = i % 3;
            let c = cos.narrow(0, src, 1)?.narrow(2, bounds[i], width)?;
            let s = sin.narrow(0, src, 1)?.narrow(2, bounds[i], width)?;
            cos_parts.push(c.squeeze(0)?);
            sin_parts.push(s.squeeze(0)?);
        }
        let c = Tensor::cat(&cos_parts, 1)?; // (seq, n)
        let s = Tensor::cat(&sin_parts, 1)?;
        // concat(c, c) -> (seq, head_dim)
        let c = Tensor::cat(&[&c, &c], 1)?.to_dtype(dtype)?;
        let s = Tensor::cat(&[&s, &s], 1)?.to_dtype(dtype)?;
        anyhow::ensure!(c.dims() == [seq, 2 * self.inv_freq.dims1()?]);
        Ok((c, s))
    }
}

/// Rope ya kawaida ya 1D (maandishi pekee).
pub fn text_positions(seq_len: usize, start: usize, dev: &Device) -> Result<Tensor> {
    let v: Vec<f32> = (start..start + seq_len).map(|i| i as f32).collect();
    let t = Tensor::from_vec(v, (1, seq_len), dev)?;
    Ok(Tensor::cat(&[&t, &t, &t], 0)?)
}
