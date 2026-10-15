# AGENTIC-ENGINE — Injini ya Akili ya MTECH OS

Injini ya modeli ya **Qwen2.5-VL** iliyoandikwa kwa **Rust** (candle), pamoja na
madaraja ya **Go** na **Elixir** kwa mawasiliano ya pembejeo/pembejeo.

Lengo: injini hii ndiyo "akili" ya MTECH OS — inafikiri, inatunza kumbukumbu, na
inaendesha modeli YOTE ndani ya Rust. Go na Elixir hurahisisha mawasiliano
kutoka kwa input hadi kwa modeli na kurudi kwa output.

## Muundo

```
rust/
  akili-core/        maktaba: upakiaji, ViT, decoder, M-RoPE, kumbukumbu
    config.rs        uchambuzi wa config.json (text + vision)
    loader.rs        safetensors kwa mmap (bf16 hukaa kwenye mmap)
    norm.rs          RMSNorm ya mkono (candle 0.10.2 haitumii F16 kwenye CPU)
    rope.rs          M-RoPE ya pande tatu (mrope_section = [16,24,24])
    vision.rs        kichwa cha macho: patch embed 3D, window + full attention
    text.rs          decoder: GQA, SwiGLU, KV cache
    tokenizer.rs     chat template + expand_image_tokens
    sampler.rs       temperature / top-p / repetition penalty
    memory.rs        kumbukumbu (kumbukumbu.jsonl)
  akilid/            binary: run | verify
    src/verify.rs    uthibitisho wa namba dhidi ya rejeleo la transformers
bridge/
  go/                akilibridge — HTTP server inayozunguka akilid
  elixir/            Port + CLI wrappers
scripts/
  ref_dump.py        rejeleo la transformers (bf16) -> /tmp/ref
  ref_vision.py      rejeleo la ViT kwa F32 (RAM ndogo)
  ref_convert.py     .npy -> .bin + manifest.json kwa akilid verify
  fetch_model.sh     kupakua modeli kutoka HuggingFace
  fetch_2nyi.sh      kupakua + kuunganisha 2NYI kutoka GitHub Release
  env.sh             mazingira yanayodumu (/workspace)
  dev_setup.sh       usakinishaji wa zana
```

## Modeli: `models/2NYI`

Modeli ni **Qwen2.5-VL-7B-Instruct**, iliyowekwa chini ya jina `2NYI`.

- Faili **ndogo** (config, tokenizer, chat template, merges, vocab, index,
  `MANIFEST.sha256`) ziko kwenye git chini ya `models/2NYI/`.
- Faili **kubwa** za safetensors (16.6GB jumla) hazimo git — GitHub hukataa
  faili zaidi ya 100MB. Ziko kwenye GitHub Release tag **`2NYI`**, zimegawanywa
  vipande chini ya 2GiB (kikomo cha release asset).

```bash
# Kupakua na kuunganisha (repo ni private -> inahitaji token)
GITHUB_PAT=ghp_... scripts/fetch_2nyi.sh models/2NYI
```

Script hupakua vipande, hiviunganisha kuwa faili tano za safetensors, kisha
huthibitisha kila kimo kwa `MANIFEST.sha256`. Uthibitisho wa duru umefanyika:
faili iliyopakuliwa na kuunganishwa inalingana SHA256 kabisa.

## Matumizi

```bash
source scripts/env.sh                     # CARGO_HOME/RUSTUP_HOME ndani ya /workspace

# Endesha modeli
akilid run --model models/2NYI --dtype f16 \
  --image picha.jpg --prompt "Eleza picha hii"

# Thibitisha namba dhidi ya rejeleo la transformers
python3 scripts/ref_dump.py  <model_dir> /tmp/ref bf16
python3 scripts/ref_convert.py /tmp/ref
akilid verify --model <model_dir> --ref-dir /tmp/ref --dtype f16

# Uthibitisho wa ViT pekee kwa F32 (RAM ~3GB)
python3 scripts/ref_vision.py <model_dir> /tmp/ref
akilid verify --model <model_dir> --ref-dir /tmp/ref --dtype f32 --vision-only
```

## Hali ya uthibitisho (2026-10-15)

Rejeleo la bf16 linatoka kwa PyTorch; kelele ya bf16 yenyewe ni kubwa, kwa hivyo
ulinganifu wa karibu hupimwa kwa F32.

| Kipimo | dtype | max_abs | cosine | Matokeo |
|--------|-------|---------|--------|----------|
| ViT vision_embeds | F32 | 0.00029 | 1.000000 | SAWA |
| ViT vision_embeds | F16 | 1.04 | 0.999201 | SAWA (kelele ya f16) |

Kumbuka: torch-bf16 dhidi ya torch-f32 yenyewe hutoa max_abs ~2.15 kwenye
vision_embeds — hivyo kelele hii si ya Rust.

## Vigezo vya kuthibitisha

| dtype | max_abs | cosine |
|-------|---------|--------|
| F32/F64 | < 0.05 | > 0.999 |
| F16/BF16 | < 1.5 | > 0.999 |

## Mahitaji

- Rust stable (candle-core/candle-nn 0.10.2)
- Go 1.21+, Elixir 1.18+
- Python: torch, transformers, safetensors, numpy, pillow (kwa rejeleo pekee)
