# Electronic Devices Solver — Agentic

## Data

- `data/devices_catalog.json` — searchable catalog (vifaa + matatizo + suluhisho)
- Full dump (optional): paste `electronic_devices_solver` into `data/electronic_devices_solver.json`

## Policy

| Domain | After HITL |
|--------|------------|
| PC software (`problems.json` + remediate) | Auto allowlist actions |
| Physical devices (TV, Fridge, Router…) | **Human guide only** — show steps from catalog |

## CLI

```bash
cd engine-rust && cargo build --release
./target/release/mtaalamu devices
./target/release/mtaalamu device-solve --msg "TV haiwaki"
./target/release/mtaalamu solve --msg "Router ina wifi issues" --approve
```

## UI

1. Source `web-r/R/devices_bridge.R` in app (with `data_bridge.R`)
2. Agentic: type device problem → ANZA → see matches → RUHUSU shows hardware checklist for human

## Expand catalog

Replace/merge `data/devices_catalog.json` with more devices from your `smart_computer_final` JSON. Rust + R load the same file (no code change).
