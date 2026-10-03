# MTAALAMU × HERMES — Miradi iliyounganishwa (source halisi NDANI ya hermes-agent)

Source code **KAMILI** ya miradi hii ipo ndani ya hermes-agent moja kwa moja — hakuna submodule,
hakuna "upstream" nje, hakuna iframe, hakuna URL za nje:

| Miradi | Source (ndani ya hermes-agent/) | UI HALISI | Route chini ya Hermes |
|---|---|---|---|
| Home Assistant (`Apache-2.0`) | `hermes-agent/home-assistant/` | Lovelace — kama ilivyo | `/ha/` |
| OpenMRS (`MPL-2.0`) | `hermes-agent/openmrs/` | OpenMRS 3 (o3) — kama ilivyo | `/openmrs/` |
| Hermes Agent (`MIT`) | `hermes-agent/web/` | Hermes dashboard — kama ilivyo | `/hermes-ui/` |

Commits zilizopimwa: `hermes-agent/integrations.lock.json`.

## Kwenye server yako

```bash
# 1) source kamili + history (commits za lock):
node hermes-agent/scripts/mtaalamu/fetch_integrations.mjs

# 2) endesha UI halisi zote tatu kutoka source:
docker compose -f hermes-agent/deploy/docker-compose.mtaalamu.yml up -d --build

# 3) fungua dashibodi:
#    http://SERVER:18080           → Hermes (UI halisi)
#    http://SERVER:18080/ha/       → Home Assistant HALISI (Lovelace) chini ya Hermes
#    http://SERVER:18080/openmrs/  → OpenMRS HALISI (o3) chini ya Hermes
```

## Jinsi UI halisi zinavyoonekana chini ya Hermes (SI iframe, SI URL za nje)

`hermes-agent/plugins/mtaalamu/realui.py` ina **reverse-proxy ya kina** inayoandikishwa kwenye
web server ya Hermes (`hermes_cli/web_server_dashboard.py`) kabla ya SPA catch-all:

- `GET /ha/...` → Hermes inapitisha ombi lako kwa Home Assistant iliyokimbia kutoka
  `hermes-agent/home-assistant/` (server yako) na kurudisha **jibu lake HALISI**
  (HTML, assets, cookies za session) chini ya asili ya Hermes.
- `GET /openmrs/...` → vivyo hivyo kwa OpenMRS iliyokimbia kutoka `hermes-agent/openmrs/`.
- `GET /hermes-ui/...` → UI ya Hermes yenyewe (`hermes-agent/web/`).

`X-Frame-Options` / `frame-ancestors` za upstream **hazipitishwi** — ni headers za Hermes
mwenyewe zinatumika, hivyo kila UI ni ukurasa kamili chini ya Hermes (bila iframe).

## Tools za HERMES (agent anazitumia moja kwa moja)

- `mtaalamu_ha` — states / turn_on / turn_off / toggle (REST halisi ya HA, HITL + RBAC)
- `mtaalamu_openmrs` — find / create patient (REST halisi ya OpenMRS, HITL + RBAC)

Siri: `HASS_URL`, `HASS_TOKEN`, `OPENMRS_URL`, `OPENMRS_USER`, `OPENMRS_PASS` — Infisical pekee
(ona `.env.example`).
