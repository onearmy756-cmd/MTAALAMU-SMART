# Knowledge pack

## Problems (420 → 21 trades)

```bash
# Option A — expand packed files (all 21 trades)
python3 scripts/expand_knowledge_problems.py

# Option B — regenerate from data/problems.json
python3 scripts/split_problems_to_knowledge.py
```

Creates `data/knowledge/problems/{trade}.json` for every trade.

Rust `knowledge_hub` and CLI `mtaalamu knowledge` read these parts first, then fall back to `data/problems.json`.

## Other modules

| Module | Path |
|--------|------|
| devices | `data/devices/` |
| diagnosis | `data/diagnosis.json` |
| trades | `data/trades.json` |
| services | `data/services.json` |
| professions | `data/professions.json` |
