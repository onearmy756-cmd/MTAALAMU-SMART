# Knowledge pack — 100%

## Problems: 420 → 21 trades

`data/problems.json` tayari iko kwenye repo (**420** matatizo).

Tengeneza sehemu zote 21:

```bash
python3 scripts/split_problems_to_knowledge.py
```

Inaandika:

```
data/knowledge/problems/
  appliance.json   (20)
  borehole.json    (20)
  cctv.json        (20)
  computer.json    (20)
  gari.json        (20)
  gas.json         (20)
  gate_motor.json  (20)
  hvac.json        (20)
  jenereta.json    (20)
  maji.json        (20)
  pikipiki.json    (20)
  pump.json        (20)
  rangi.json       (20)
  roofing.json     (20)
  simu.json        (20)
  solar.json       (20)
  tailor.json      (20)
  ujenzi.json      (20)
  umeme.json       (20)
  useremala.json   (20)
  welding.json     (20)
  manifest.json
```

## CLI (baada ya expand / split)

```bash
cd engine-rust && cargo build --release
./target/release/mtaalamu knowledge --msg "kompyuta virus"
./target/release/mtaalamu knowledge --msg "umeme breaker"
./target/release/mtaalamu knowledge-stats
```

> **Kumbuka:** `knowledge_hub` inasoma `data/problems.json` moja kwa moja kama fallback — **420 hits zinapatikana hata bila split**. Split ni kwa muundo wa sehemu kama devices.

## Modules zingine (kama devices)

| Module | Path | Count |
|--------|------|-------|
| devices | `data/devices/` | 59 / 507 |
| problems | `data/problems.json` | 420 |
| diagnosis | `data/diagnosis.json` | 7 |
| trades | `data/trades.json` | 21 |
| services | `data/services.json` | 105 |
| professions | `data/professions.json` | categories |
