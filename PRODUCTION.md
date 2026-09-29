# Production runbook — MTAALAMU SMART

## Requirements

- Rust stable (`rustc`, `cargo`)
- R ≥ 4.2 + packages: `shiny`, `jsonlite`, `htmltools`
- Linux recommended for full deep probe (`/proc`, `/sys`)

## Build

```bash
make build          # release binary
make test           # cargo test
make health         # CLI + data checks
bash scripts/healthcheck.sh
```

Binary: `engine-rust/target/release/mtaalamu` (or `dist/` after `make release`)

## Core commands

```bash
./engine-rust/target/release/mtaalamu deep --top 15
./engine-rust/target/release/mtaalamu sysprobe --top 10
./engine-rust/target/release/mtaalamu vision
./engine-rust/target/release/mtaalamu agentic --msg "Tatizo..." --approve
./engine-rust/target/release/mtaalamu remediate
./engine-rust/target/release/mtaalamu remediate --action report_top_cpu
./engine-rust/target/release/mtaalamu remediate --action clear_user_temp --approve
```

## UI

```bash
make run-ui
# or
Rscript -e "shiny::runApp('web-r', port=3838)"
```

Open http://127.0.0.1:3838 → tab **AGENTIC VISION**

## Logs (audit)

| File | Purpose |
|------|--------|
| `data/learning_log.json` | sessions / issues |
| `data/remediation_log.json` | fix actions |

## Scope (honest)

Production = real OS observability + agentic workflow + safe HITL remediation.  
Not in scope: motherboard X-ray, arbitrary kernel video, AGI.

See also: `docs/PRODUCTION_CAPABILITIES.md`
