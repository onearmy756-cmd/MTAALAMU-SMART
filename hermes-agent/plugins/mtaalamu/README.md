# MTAALAMU plugin kwa Hermes Agent

Plugin rasmi ya kuunganisha **MTAALAMU SMART** ndani ya **Hermes Agent**
(Nous Research) — backend + skills + dashboard tab, kwa njia zote rasmi za
Hermes (hakuna kubadilisha core yake).

## Kilichomo

| Sehemu | Njia rasmi ya Hermes inayotumika |
|---|---|
| Zana 11 za agent (solve, agentic, scan, remediate, deploy, WhatsApp/SMS/email, maintenance, scanner, remote job) | `plugins/mtaalamu/` — general plugin, `register(ctx)` + `ctx.register_tool` |
| Skills 4 (triage, deploy, channels, maintenance) | `skills/mtaalamu/<name>/SKILL.md` — frontmatter + section order ya Hermes |
| Tab ya dashboard (Tatua / Deploy / Mobile / Channels / Maintenance / Jobs + RBAC) | `plugins/mtaalamu/dashboard/manifest.json` — dashboard plugin (SDK v1.1.0) |
| Backend API ya tab | `plugins/mtaalamu/dashboard/plugin_api.py` — FastAPI router, mounted `/api/plugins/mtaalamu/` |

## Kanuni zilizohifadhiwa

- **R-1**: LLM hauhesabu kamwe — hesabu/utambuzi ni engine ya Rust (`mtaalamu` CLI); LLM anaeleza.
- **HITL**: kila hatua inayobadilisha mfumo (deploy start, remediate, agentic approve, scanner run) inamwuliza mteja; `user` role hawezi kuidhinisha (RBAC: admin/specialist/user).
- **Hakuna data bandia**: engine ikiwa haipo au service imezimwa, kosa halisi linarudishwa na `hint` ya kuiweka.
- **Ollama Cloud**: Hermes ina provider `ollama-cloud` ndani yake (`OLLAMA_API_KEY` → `https://ollama.com/v1`); weka key Infisical/env, LLM wa Hermes anatumia cloud.

## Kuiwasha

1. Engine: `cd engine-rust && cargo build --release` (au weka `MTAALAMU_ROOT` env ikielekeza repo root).
2. Hermes: anzisha kama kawaida — plugin (backend + tab) inagundulwa automatically
   (bundled source). Skills ziko `skills/mtaalamu/`.
3. Channels: sanidi `channels.whatsapp` kwenye `cli-config.yaml`; SMS/email
   credentials (`FUNDI_SMS_URL`, `FUNDI_SMTP_*`) iwe Infisical.
