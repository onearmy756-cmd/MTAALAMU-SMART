//! akilid — daemon ya MBILINYI AKILI (Qwen2.5-VL kwa Rust).
//!
//! Hutumia JSON-lines kwenye stdin/stdout: kila mstari ni ombi moja la JSON.
//! Elixir/Go huzungumza nayo kwa njia hii (au kwa kuiendesha kama process).

use akili_core::image::PlanarImage;
use akili_core::tokenizer::Turn;
use akili_core::{Engine, EngineOptions, GenParams};
use anyhow::{Context, Result};
use base64::Engine as _;
use candle_core::DType;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "akilid", version, about = "MBILINYI AKILI — Qwen2.5-VL engine (Rust)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Endesha daemon (JSON-lines kwenye stdin/stdout).
    Serve {
        #[arg(long, env = "AKILI_MODEL")]
        model: PathBuf,
        #[arg(long, default_value = "f16")]
        dtype: String,
        #[arg(long, default_value_t = false)]
        no_vision: bool,
        #[arg(long, env = "AKILI_MEMORY", default_value = "data/kumbukumbu.jsonl")]
        memory: PathBuf,
    },
    /// Jaribio la haraka la maandishi.
    Ask {
        #[arg(long, env = "AKILI_MODEL")]
        model: PathBuf,
        prompt: String,
        #[arg(long, default_value_t = 128)]
        max_tokens: usize,
        #[arg(long, default_value_t = 0.2)]
        temperature: f32,
        #[arg(long, default_value = "f16")]
        dtype: String,
    },
    /// Uthibitisho wa namba dhidi ya rejeleo la transformers.
    Verify {
        #[arg(long, env = "AKILI_MODEL")]
        model: PathBuf,
        #[arg(long, default_value = "/tmp/ref")]
        ref_dir: PathBuf,
        #[arg(long, default_value = "f16")]
        dtype: String,
        #[arg(long, default_value_t = false)]
        vision_only: bool,
    },
}

fn dtype_of(s: &str) -> Result<DType> {
    Ok(match s.to_lowercase().as_str() {
        "f16" | "fp16" | "half" => DType::F16,
        "f32" | "fp32" | "float" => DType::F32,
        "bf16" => DType::BF16,
        other => anyhow::bail!("dtype `{other}` haijulikani (tumia f16|f32)"),
    })
}

fn load(model: &Path, dtype: &str, vision: bool, memory: &Path) -> Result<Engine> {
    let opts = EngineOptions {
        dtype: dtype_of(dtype)?,
        with_vision: vision,
        memory_path: memory.to_path_buf(),
        ..Default::default()
    };
    Engine::load(model, opts)
}

#[derive(Deserialize)]
struct Msg {
    role: String,
    content: String,
    #[serde(default)]
    images: Vec<String>,
}

#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    id: Value,
    op: String,
    #[serde(default)]
    system: Option<String>,
    #[serde(default)]
    messages: Vec<Msg>,
    #[serde(default)]
    images: Vec<String>,
    #[serde(default)]
    max_tokens: Option<usize>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    repetition_penalty: Option<f32>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    salience: Option<f32>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    remember: Option<bool>,
}

#[derive(Serialize)]
struct Response {
    id: Value,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generated_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stopped: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memories: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<Value>,
}

fn load_images(paths: &[String]) -> Result<Vec<PlanarImage>> {
    let mut out = Vec::with_capacity(paths.len());
    for p in paths {
        if let Some(b64) = p.strip_prefix("base64:") {
            let raw = base64::engine::general_purpose::STANDARD
                .decode(b64.trim())
                .context("base64 ya picha si sahihi")?;
            out.push(akili_core::image::load_image_bytes(&raw)?);
        } else {
            out.push(akili_core::image::load_image(Path::new(p))?);
        }
    }
    Ok(out)
}

fn handle(engine: &mut Engine, req: &Request) -> Result<Response> {
    let id = req.id.clone();
    match req.op.as_str() {
        "health" => Ok(Response {
            id,
            ok: true,
            error: None,
            text: None,
            prompt_tokens: None,
            generated_tokens: None,
            stopped: None,
            ms: None,
            memories: None,
            memory_id: None,
            info: Some(json!({
                "model": engine.dir.display().to_string(),
                "dtype": format!("{:?}", engine.dtype),
                "vision": engine.vision_enabled(),
                "layers": engine.cfg.num_hidden_layers,
                "vocab": engine.cfg.vocab_size,
                "memories": engine.memory.count(),
            })),
        }),
        "chat" => {
            let mut turns: Vec<Turn> = req
                .messages
                .iter()
                .map(|m| {
                    let mut t = if m.role == "assistant" {
                        Turn::assistant(m.content.clone())
                    } else {
                        Turn::user(m.content.clone())
                    };
                    t.images = m.images.len();
                    t
                })
                .collect();
            let mut paths: Vec<String> = Vec::new();
            for m in &req.messages {
                paths.extend(m.images.iter().cloned());
            }
            paths.extend(req.images.iter().cloned());
            if !req.images.is_empty() {
                if let Some(t) = turns.iter_mut().rev().find(|t| t.role == "user") {
                    t.images += req.images.len();
                }
            }
            if turns.is_empty() {
                turns.push(Turn::user(String::new()));
            }
            let images = load_images(&paths)?;
            let params = GenParams {
                max_tokens: req.max_tokens.unwrap_or(256),
                temperature: req.temperature.unwrap_or(0.2),
                top_p: req.top_p.unwrap_or(0.9),
                repetition_penalty: req.repetition_penalty.unwrap_or(1.05),
                ..Default::default()
            };
            let system = match req.system.clone() {
                Some(s) if !s.is_empty() => s,
                _ => default_system(engine),
            };
            let out = engine.generate(&system, &turns, &images, &params)?;
            if req.remember.unwrap_or(false) && !out.text.trim().is_empty() {
                let mut tags = vec!["mazungumzo".to_string()];
                if !images.is_empty() {
                    tags.push("picha".to_string());
                }
                engine.memory.remember("majibu", out.text.trim(), tags, 1.0)?;
            }
            Ok(Response {
                id,
                ok: true,
                error: None,
                text: Some(out.text),
                prompt_tokens: Some(out.prompt_tokens),
                generated_tokens: Some(out.generated_tokens),
                stopped: Some(out.stopped),
                ms: Some(out.ms),
                memories: None,
                memory_id: None,
                info: None,
            })
        }
        "remember" => {
            let text = req.text.clone().unwrap_or_default();
            anyhow::ensure!(!text.trim().is_empty(), "`text` ni tupu");
            let m = engine.memory.remember(
                req.kind.as_deref().unwrap_or("ukweli"),
                text.trim(),
                req.tags.clone(),
                req.salience.unwrap_or(1.0),
            )?;
            Ok(Response {
                id,
                ok: true,
                error: None,
                text: None,
                prompt_tokens: None,
                generated_tokens: None,
                stopped: None,
                ms: None,
                memories: None,
                memory_id: Some(m.id),
                info: Some(json!({ "count": engine.memory.count() })),
            })
        }
        "recall" => {
            let query = req.query.clone().unwrap_or_default();
            let limit = req.limit.unwrap_or(8);
            let found = engine.memory.recall(&query, limit);
            Ok(Response {
                id,
                ok: true,
                error: None,
                text: None,
                prompt_tokens: None,
                generated_tokens: None,
                stopped: None,
                ms: None,
                memories: Some(serde_json::to_value(&found)?),
                memory_id: None,
                info: Some(json!({ "total": engine.memory.count() })),
            })
        }
        "digest" => {
            let limit = req.limit.unwrap_or(2000);
            Ok(Response {
                id,
                ok: true,
                error: None,
                text: Some(engine.memory.digest(limit)),
                prompt_tokens: None,
                generated_tokens: None,
                stopped: None,
                ms: None,
                memories: None,
                memory_id: None,
                info: None,
            })
        }
        other => anyhow::bail!("op `{other}` haijulikani"),
    }
}

fn default_system(engine: &mut Engine) -> String {
    let digest = engine.memory.digest(1200);
    let mut s = String::from(
        "Wewe ni MTAALAMU, akili kuu ya MTECH OS. Jibu kwa Kiswahili kifupi na kwa uwazi. \
         Tumia kumbukumbu zifuatazo ukiwa na uhakika nazo.",
    );
    if !digest.is_empty() {
        s.push_str("\n\nKUMBUKUMBU:\n");
        s.push_str(&digest);
    }
    s
}

fn serve(model: &Path, dtype: &str, vision: bool, memory: &Path) -> Result<()> {
    let mut engine = load(model, dtype, vision, memory)?;
    eprintln!(
        "[akilid] model={} dtype={:?} vision={} kumbukumbu={}",
        engine.dir.display(),
        engine.dtype,
        engine.vision_enabled(),
        engine.memory.count()
    );
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<Request>(&line) {
            Ok(req) => match handle(&mut engine, &req) {
                Ok(r) => r,
                Err(e) => Response {
                    id: req.id.clone(),
                    ok: false,
                    error: Some(format!("{e:#}")),
                    text: None,
                    prompt_tokens: None,
                    generated_tokens: None,
                    stopped: None,
                    ms: None,
                    memories: None,
                    memory_id: None,
                    info: None,
                },
            },
            Err(e) => Response {
                id: Value::Null,
                ok: false,
                error: Some(format!("JSON si sahihi: {e}")),
                text: None,
                prompt_tokens: None,
                generated_tokens: None,
                stopped: None,
                ms: None,
                memories: None,
                memory_id: None,
                info: None,
            },
        };
        serde_json::to_writer(&mut stdout, &resp)?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
    }
    Ok(())
}

fn ask(model: &Path, prompt: &str, max_tokens: usize, temperature: f32, dtype: &str) -> Result<()> {
    let mut engine = load(model, dtype, false, Path::new("data/kumbukumbu.jsonl"))?;
    let params = GenParams {
        max_tokens,
        temperature,
        ..Default::default()
    };
    let out = engine.generate("", &[Turn::user(prompt)], &[], &params)?;
    println!("{}", out.text);
    eprintln!(
        "[akilid] prompt_tokens={} generated={} ms={}",
        out.prompt_tokens, out.generated_tokens, out.ms
    );
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Serve {
            model,
            dtype,
            no_vision,
            memory,
        } => serve(&model, &dtype, !no_vision, &memory),
        Cmd::Ask {
            model,
            prompt,
            max_tokens,
            temperature,
            dtype,
        } => ask(&model, &prompt, max_tokens, temperature, &dtype),
        Cmd::Verify {
            model,
            ref_dir,
            dtype,
            vision_only,
        } => akilid::verify::run(&model, &ref_dir, dtype_of(&dtype)?, vision_only),
    }
}
