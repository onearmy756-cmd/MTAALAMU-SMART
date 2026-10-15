//! Decoder ya maandishi ya Qwen2.5-VL (GQA, RMSNorm, SwiGLU, M-RoPE) pamoja na KV cache.

use crate::config::Config;
use crate::rope::MRope;
use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor, D};
use candle_nn::ops;
use crate::norm::RmsNormF;

pub(crate) fn lin(x: &Tensor, w: &Tensor, b: Option<&Tensor>) -> Result<Tensor> {
    // `w` ni (out, in). candle matmul huhitaji rank sawa, kwa hivyo tunabana
    // dims zote za mbele kuwa mstari mmoja kisha tunarudisha umbo la awali.
    let dims = x.dims().to_vec();
    let in_dim = *dims.last().context("lin: tensor haina dims")?;
    let lead: usize = dims[..dims.len() - 1].iter().product();
    let x2 = x.contiguous()?.reshape((lead, in_dim))?;
    let mut y = x2.matmul(&w.t()?.contiguous()?)?;
    if let Some(b) = b {
        y = y.broadcast_add(b)?;
    }
    let mut shape = dims[..dims.len() - 1].to_vec();
    shape.push(w.dim(0)?);
    Ok(y.reshape(shape)?)
}

struct LayerW {
    input_ln: RmsNormF,
    post_ln: RmsNormF,
    q_w: Tensor,
    q_b: Tensor,
    k_w: Tensor,
    k_b: Tensor,
    v_w: Tensor,
    v_b: Tensor,
    o_w: Tensor,
    gate: Tensor,
    up: Tensor,
    down: Tensor,
}

pub struct TextModel {
    embed: Tensor,
    layers: Vec<LayerW>,
    norm: RmsNormF,
    lm_head: Tensor,
    rope: MRope,
    num_heads: usize,
    num_kv_heads: usize,
    head_dim: usize,
    hidden: usize,
    dtype: DType,
    /// KV cache kwa kila layer: (k, v) zenye umbo (1, kv_heads, past, head_dim).
    /// `RefCell` hutuwezesha kubadilisha cache bila `&mut self` (layers hubaki zimekopeshwa).
    cache: std::cell::RefCell<Vec<Option<(Tensor, Tensor)>>>,
    pub rope_delta: i64,
}

pub struct TextWeights {
    pub embed: Tensor,
    pub layers: Vec<LayerWeights>,
    pub norm: Tensor,
    pub lm_head: Tensor,
}

pub struct LayerWeights {
    pub input_ln: Tensor,
    pub post_ln: Tensor,
    pub q_w: Tensor,
    pub q_b: Tensor,
    pub k_w: Tensor,
    pub k_b: Tensor,
    pub v_w: Tensor,
    pub v_b: Tensor,
    pub o_w: Tensor,
    pub gate: Tensor,
    pub up: Tensor,
    pub down: Tensor,
}

/// Huchukua weights zote za decoder kutoka shards (mmap) kwa dtype inayolengwa.
pub fn load_weights(loader: &crate::loader::WeightLoader, cfg: &Config, dtype: DType) -> Result<TextWeights> {
    let embed = loader.get("model.embed_tokens.weight", dtype)?;
    let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
    for i in 0..cfg.num_hidden_layers {
        let p = format!("model.layers.{i}");
        layers.push(LayerWeights {
            input_ln: loader.get(&format!("{p}.input_layernorm.weight"), dtype)?,
            post_ln: loader.get(&format!("{p}.post_attention_layernorm.weight"), dtype)?,
            q_w: loader.get(&format!("{p}.self_attn.q_proj.weight"), dtype)?,
            q_b: loader.get(&format!("{p}.self_attn.q_proj.bias"), dtype)?,
            k_w: loader.get(&format!("{p}.self_attn.k_proj.weight"), dtype)?,
            k_b: loader.get(&format!("{p}.self_attn.k_proj.bias"), dtype)?,
            v_w: loader.get(&format!("{p}.self_attn.v_proj.weight"), dtype)?,
            v_b: loader.get(&format!("{p}.self_attn.v_proj.bias"), dtype)?,
            o_w: loader.get(&format!("{p}.self_attn.o_proj.weight"), dtype)?,
            gate: loader.get(&format!("{p}.mlp.gate_proj.weight"), dtype)?,
            up: loader.get(&format!("{p}.mlp.up_proj.weight"), dtype)?,
            down: loader.get(&format!("{p}.mlp.down_proj.weight"), dtype)?,
        });
    }
    let norm = loader.get("model.norm.weight", dtype)?;
    let lm_head = if loader.has("lm_head.weight") {
        loader.get("lm_head.weight", dtype)?
    } else {
        // tie_word_embeddings = true
        embed.clone()
    };
    Ok(TextWeights {
        embed,
        layers,
        norm,
        lm_head,
    })
}

impl TextModel {
    pub fn new(cfg: &Config, w: TextWeights, dev: &Device, dtype: DType) -> Result<Self> {
        let rms = |t: Tensor| -> Result<RmsNormF> { RmsNormF::new(t, cfg.rms_norm_eps) };
        let layers = w
            .layers
            .into_iter()
            .map(|l| {
                Ok(LayerW {
                    input_ln: rms(l.input_ln)?,
                    post_ln: rms(l.post_ln)?,
                    q_w: l.q_w,
                    q_b: l.q_b,
                    k_w: l.k_w,
                    k_b: l.k_b,
                    v_w: l.v_w,
                    v_b: l.v_b,
                    o_w: l.o_w,
                    gate: l.gate,
                    up: l.up,
                    down: l.down,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let n = cfg.num_hidden_layers;
        Ok(Self {
            embed: w.embed,
            layers,
            norm: rms(w.norm)?,
            lm_head: w.lm_head,
            rope: MRope::new(cfg.head_dim, cfg.rope_theta, cfg.mrope_section, dev)?,
            num_heads: cfg.num_attention_heads,
            num_kv_heads: cfg.num_key_value_heads,
            head_dim: cfg.head_dim,
            hidden: cfg.hidden_size,
            dtype,
            cache: std::cell::RefCell::new(vec![None; n]),
            rope_delta: 0,
        })
    }

    pub fn reset_cache(&mut self) {
        for c in self.cache.borrow_mut().iter_mut() {
            *c = None;
        }
    }

    pub fn cache_len(&self) -> usize {
        self.cache
            .borrow()
            .iter()
            .find_map(|c| c.as_ref().map(|(k, _)| k.dim(2).unwrap_or(0)))
            .unwrap_or(0)
    }

    /// Hupanga embeddings kutoka ids (hutumia `embed` matrix).
    pub fn embed_tokens(&self, ids: &Tensor) -> Result<Tensor> {
        let (_b, seq) = ids.dims2()?;
        let flat = ids.flatten_all()?.to_dtype(DType::U32)?;
        let e = self.embed.index_select(&flat, 0)?;
        Ok(e.reshape((1, seq, self.hidden))?)
    }

    /// Forward ya block moja. `embeds`: (1, seq, hidden), `positions`: (3, seq).
    /// Hurudisha hidden states za mwisho (1, seq, hidden).
    pub fn forward(&mut self, embeds: &Tensor, positions: &Tensor) -> Result<Tensor> {
        let (cos, sin) = self.rope.cos_sin(positions, self.dtype)?;
        let seq = embeds.dim(1)?;
        let past = self.cache_len();
        let total = past + seq;
        let mask = self.causal_mask(seq, past, total, embeds.device())?;
        let mut x = embeds.clone();
        for (i, l) in self.layers.iter().enumerate() {
            x = self.block(i, l, &x, &cos, &sin, mask.as_ref())?;
        }
        Ok(self.norm.forward(&x)?)
    }

    fn causal_mask(&self, q: usize, past: usize, total: usize, dev: &Device) -> Result<Option<Tensor>> {
        if q == 1 && past > 0 {
            return Ok(None);
        }
        let mut data = vec![0f32; q * total];
        for i in 0..q {
            let limit = past + i;
            for j in 0..total {
                if j > limit {
                    data[i * total + j] = -1e30;
                }
            }
        }
        let m = Tensor::from_vec(data, (1, 1, q, total), dev)?.to_dtype(self.dtype)?;
        Ok(Some(m))
    }

    fn block(
        &self,
        idx: usize,
        l: &LayerW,
        x: &Tensor,
        cos: &Tensor,
        sin: &Tensor,
        mask: Option<&Tensor>,
    ) -> Result<Tensor> {
        let (b, seq, _h) = x.dims3()?;
        let residual = x;
        let h = l.input_ln.forward(&x)?;

        let q = lin(&h, &l.q_w, Some(&l.q_b))?;
        let k = lin(&h, &l.k_w, Some(&l.k_b))?;
        let v = lin(&h, &l.v_w, Some(&l.v_b))?;
        let q = q.reshape((b, seq, self.num_heads, self.head_dim))?.transpose(1, 2)?;
        let k = k.reshape((b, seq, self.num_kv_heads, self.head_dim))?.transpose(1, 2)?;
        let v = v.reshape((b, seq, self.num_kv_heads, self.head_dim))?.transpose(1, 2)?;

        // Rope (rotate-half)
        let cos = cos.unsqueeze(0)?.unsqueeze(0)?; // (1,1,seq,d)
        let sin = sin.unsqueeze(0)?.unsqueeze(0)?;
        let q = rope_apply(&q, &cos, &sin)?;
        let k = rope_apply(&k, &cos, &sin)?;

        // KV cache
        let (k, v) = match self.cache.borrow_mut()[idx].take() {
            Some((pk, pv)) => (Tensor::cat(&[&pk, &k], 2)?, Tensor::cat(&[&pv, &v], 2)?),
            None => (k, v),
        };
        self.cache.borrow_mut()[idx] = Some((k.clone(), v.clone()));

        let n_rep = self.num_heads / self.num_kv_heads;
        let k = repeat_kv(&k, n_rep)?;
        let v = repeat_kv(&v, n_rep)?;

        let scale = 1.0 / (self.head_dim as f64).sqrt();
        let mut attn = (q.contiguous()?.matmul(&k.transpose(2, 3)?.contiguous()?)? * scale)?;
        if let Some(m) = mask {
            attn = attn.broadcast_add(m)?;
        }
        let attn = ops::softmax(&attn, D::Minus1)?;
        let out = attn.matmul(&v.contiguous()?)?; // (b, heads, seq, d)
        let out = out.transpose(1, 2)?.reshape((b, seq, self.hidden))?;
        let out = lin(&out, &l.o_w, None)?;
        let x = (residual + out)?;

        let residual = x.clone();
        let h = l.post_ln.forward(&x)?;
        let gate = lin(&h, &l.gate, None)?.silu()?;
        let up = lin(&h, &l.up, None)?;
        let mlp = lin(&(gate * up)?, &l.down, None)?;
        Ok((residual + mlp)?)
    }

    /// Logits za tokeni ya mwisho tu: (vocab,) f32.
    pub fn logits_last(&self, hidden: &Tensor) -> Result<Vec<f32>> {
        let seq = hidden.dim(1)?;
        let last = hidden.narrow(1, seq - 1, 1)?;
        let logits = lin(&last, &self.lm_head, None)?;
        Ok(logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?.to_vec1::<f32>()?)
    }

    /// Logits kwa nafasi zote: (seq, vocab) f32.
    pub fn logits_all(&self, hidden: &Tensor) -> Result<Tensor> {
        let logits = lin(hidden, &self.lm_head, None)?;
        Ok(logits.squeeze(0)?.to_dtype(DType::F32)?)
    }
}

pub(crate) fn rope_apply(x: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let dt = x.dtype();
    let x = x.to_dtype(DType::F32)?;
    let cos = cos.to_dtype(DType::F32)?;
    let sin = sin.to_dtype(DType::F32)?;
    let d = x.dim(D::Minus1)?;
    let x1 = x.narrow(D::Minus1, 0, d / 2)?;
    let x2 = x.narrow(D::Minus1, d / 2, d / 2)?;
    let rot = Tensor::cat(&[&x2.neg()?, &x1], D::Minus1)?;
    let out = (x.broadcast_mul(&cos)? + rot.broadcast_mul(&sin)?)?;
    Ok(out.to_dtype(dt)?)
}

pub(crate) fn repeat_kv(x: &Tensor, n_rep: usize) -> Result<Tensor> {
    if n_rep == 1 {
        return Ok(x.clone());
    }
    let (b, kv, s, d) = x.dims4()?;
    Ok(x.unsqueeze(2)?
        .broadcast_as((b, kv, n_rep, s, d))?
        .reshape((b, kv * n_rep, s, d))?)
}
