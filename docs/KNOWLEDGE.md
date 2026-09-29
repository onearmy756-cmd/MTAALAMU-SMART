# Knowledge Hub — muundo kama Electronic Devices

## Modules (zote zinatafutwa sawa)

| Module | Source | Search |
|--------|--------|--------|
| **devices** | `data/devices/*` | `device-solve` / knowledge |
| **problems** | `data/problems.json` (420) | knowledge |
| **diagnosis** | `data/diagnosis.json` (7 models) | knowledge / diagnose |
| **trades** | `data/trades.json` (21) | knowledge |
| **services** | `data/services.json` (105) | knowledge |
| **professions** | `data/professions.json` | knowledge |
| **formulas** | `data/formulas.json` | `calc` / Formula tab |

## CLI

```bash
cargo build --release

# Search ALL modules at once
./target/release/mtaalamu knowledge --msg "kompyuta virus polepole"
./target/release/mtaalamu knowledge --msg "TV haiwaki"
./target/release/mtaalamu knowledge-stats

# Devices only (100% catalog)
./target/release/mtaalamu devices
./target/release/mtaalamu device-solve --msg "Fridge haifanyi baridi"

# Software solve + knowledge + devices
./target/release/mtaalamu solve --msg "slow virus" --approve

# Agentic includes knowledge
./target/release/mtaalamu agentic --msg "umeme breaker" --approve
```

## Policy

- **Software (PC)**: auto after HITL (allowlist)
- **Hardware / devices / trades physical**: guide for human
- Knowledge returns ranked hits with `module`, `score`, `detail_sw`

## Expand

Add JSON under `data/` — `knowledge_hub` already falls back to monolith files (`problems.json`, `services.json`, …). Optional split: `data/knowledge/problems/{trade}.json`.
