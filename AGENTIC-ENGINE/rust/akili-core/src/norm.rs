//! RMSNorm ya mkono.
//!
//! `candle_nn::RmsNorm` (0.10.x) hutupa `unsupported dtype for rmsnorm F16`
//! kwenye CPU, kwa hivyo tunakokotoa kwa F32 kisha kurudisha dtype ya pembejeo.

use anyhow::Result;
use candle_core::{DType, Tensor, D};

pub struct RmsNormF {
    weight: Tensor,
    eps: f64,
}

impl RmsNormF {
    pub fn new(weight: Tensor, eps: f64) -> Result<Self> {
        Ok(Self {
            weight: weight.to_dtype(DType::F32)?.contiguous()?,
            eps,
        })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let dtype = x.dtype();
        let x = x.to_dtype(DType::F32)?;
        let var = x.sqr()?.mean_keepdim(D::Minus1)?;
        let normed = x.broadcast_div(&(var + self.eps)?.sqrt()?)?;
        Ok(normed.broadcast_mul(&self.weight)?.to_dtype(dtype)?)
    }
}
