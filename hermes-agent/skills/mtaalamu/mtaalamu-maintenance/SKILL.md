---
name: mtaalamu-maintenance
description: "Predictive maintenance: scan systems, forecast risk, prevent failures."
version: 1.0.0
author: Zawadi Stephano (onearmy756-cmd), Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [MTAALAMU, Maintenance, Scanner, Reliability]
    category: mtaalamu
    related_skills: [mtaalamu-triage]
---

# MTAALAMU Maintenance Skill

Keep customer systems healthy proactively: run the background scanner, forecast
failure risk from real probes plus rules, and fix what it finds — with the
customer's consent — before it breaks.

## When to Use

- Scheduled checkups (cron / remote job).
- Customer asks "ni nini kinaweza kuvunjika hivi karibuni?"
- The scanner flagged an anomaly and you need to act on it.

## Prerequisites

- MTAALAMU plugin enabled (`mtaalamu_maintenance`, `mtaalamu_scanner`,
  `mtaalamu_scan`, `mtaalamu_remediate`).
- For scheduled runs, a Hermes cron or the dashboard plugin's Remote Jobs panel.

## How to Run

1. Baseline: `mtaalamu_scanner` (status) then `mtaalamu_scan deep=true` for a
   full reading.
2. Forecast: `mtaalamu_maintenance horizon=30 top=10` — the engine's rules turn
   real probe data into wear/failure risks ranked by severity.
3. Narrate the top risks to the customer in their language with plain causes
   ("dishi umekaribia kujaa", "RAM ina matumizi ya juu kila wakati").
4. Prevention plan: for each risk, find the matching `mtaalamu_remediate`
   action; present the plan and get explicit consent (HITL) before running.
5. Fix, then re-scan to show the improvement; record the before/after delta.
6. Offer scheduling: a daily/weekly scanner pass that reports only when a risk
   crosses the customer's chosen threshold.

## Quick Reference

| Tool | Purpose |
|---|---|
| `mtaalamu_scan deep=true` | full real probe (baseline/verify) |
| `mtaalamu_maintenance horizon=N` | ranked failure-risk forecast |
| `mtaalamu_scanner mode=run_once` | one supervised auto-work cycle (HITL) |
| `mtaalamu_remediate action=<id> approved=true` | consented fix |

## Pitfalls

- Forecasts are rules over real probes — never fabricate numbers the engine
  did not return.
- Auto-fixing without consent is forbidden (HITL), even for "obvious" risks.
- A passing scan is not immortality: state the residual risk honestly.

## Verification

- Post-fix `mtaalamu_scan` shows the flagged metric inside the safe band.
- `mtaalamu_scanner` history records the resolved finding.
