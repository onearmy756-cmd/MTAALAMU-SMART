# 🚀 MTAALAMU × HERMES — Kuweka Server (Kiswahili, hatua kwa hatua)

Mwongozo huu unakuwezesha kuendesha **UI HALISI tatu** kwenye server yako, zote
kutoka source iliyoko **NDANI ya hermes-agent/** (hakuna iframe, hakuna URL za nje):

| Kitu | Anwani kwenye server yako | Source |
|---|---|---|
| Hermes Agent dashboard | `http://SERVER:18080` | `hermes-agent/web/` |
| Home Assistant **HALISI** chini ya Hermes | `http://SERVER:18080/ha/` | `hermes-agent/home-assistant/` |
| OpenMRS **HALISI** chini ya Hermes | `http://SERVER:18080/openmrs/` | `hermes-agent/openmrs/` |
| Home Assistant moja kwa moja | `http://SERVER:8123` | `hermes-agent/home-assistant/` |
| OpenMRS moja kwa moja | `http://SERVER:8081/openmrs` | `hermes-agent/openmrs/` |

**Mahitaji:** Ubuntu 22.04/24.04, RAM 8GB+ (OpenMRS ni nzito), disk 40GB+, Docker.

---

## HATUA 1 — Andaa server (dakika 5)

```bash
sudo apt update && sudo apt -y upgrade
sudo apt -y install git curl unzip

# Docker rasmi:
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER
newgrp docker
docker --version   # thibitisha
```

## HATUA 2 — Vuta mradi + source kamili (dakika 10)

```bash
cd ~
git clone https://github.com/onearmy756-cmd/MTAALAMU-SMART.git
cd MTAALAMU-SMART

# Vuta commits HALISI za Home Assistant + OpenMRS (integrations.lock.json):
node hermes-agent/scripts/mtaalamu/fetch_integrations.mjs
```

> Kama `node` haipo: `curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && sudo apt -y install nodejs`

Script hii inafanya:
- `hermes-agent/home-assistant/` ← clone ya home-assistant/core @ commit iliyopimwa
- `hermes-agent/openmrs/` ← clone ya openmrs-core @ commit iliyopimwa

## HATUA 3 — Siri kupitia Infisical (dakika 10)

```bash
curl -fsSL https://infisical.com/cli/install.sh | bash   # au: sudo apt install infisical
infisical login
cd ~/MTAALAMU-SMART
infisical secrets set HASS_TOKEN="LONG-LIVED-TOKEN-YAKO-YA-HA"
infisical secrets set OPENMRS_USER="admin"
infisical secrets set OPENMRS_PASS="Admin123"   # badilisha baadaye!
```

**HASS_TOKEN unapatikana wapi:** Home Assistant → bonyeza jina lako (Profile) →
Security → **Long-Lived Access Tokens** → Create Token.

Zingine (OLLAMA_API_KEY, FUNDI_SMS_URL, FUNDI_SMTP_*) — ona `.env.example`.

## HATUA 4 — Anzisha stack KAMILI (dakika 20–40 kwa build ya kwanza)

```bash
cd ~/MTAALAMU-SMART
infisical run -- docker compose -f hermes-agent/deploy/docker-compose.mtaalamu.yml up -d --build
```

Docker itajenga kutoka source halisi:
- `hermes-agent/home-assistant/` → picha ya HA core
- `hermes-agent/openmrs/` → picha ya OpenMRS core (Maven build — inachukua muda)
- `hermes-agent/` → picha ya Hermes Agent

Fuatilia:
```bash
docker compose -f hermes-agent/deploy/docker-compose.mtaalamu.yml logs -f
```

## HATUA 5 — Fungua UI HALISI (tayari!)

1. **Hermes**: `http://SERVER_IP:18080`
   - Sidebar upande wa kushoto chini ya **MTAALAMU Systems**:
     - 🏠 **Home Assistant** → `/ha/` (Lovelace HALISI, ukurasa kamili)
     - 🩺 **OpenMRS** → `/openmrs/` (o3 HALISI, ukurasa kamili)
     - 🤖 **Hermes UI** → `/hermes-ui/`
   - Pia tab ya **MTAALAMU** kwenye dashboard ina kitufe
     "FUNGUA UI HALISI (/ha/ · /openmrs/)".
2. **OpenMRS mara ya kwanza**: `http://SERVER:8081/openmrs` → installation wizard →
   schema `openmrs`, user `openmrs`/`openmrs` (MySQL iko compose). Chukua muda.
3. **Home Assistant**: `http://SERVER:8123` → onboarding ya kwanza (unda admin).

## HATUA 6 — HERMES kama msimamizi (tools za agent)

Ndani ya chat ya Hermes (`http://SERVER:18080`) unaweza kusema/kutuma:

```
/mtaalamu_ha states
/mtaalamu_ha turn_on light.living_room     ← itaomba HITL kwanza
/mtaalamu_openmrs find Test
/mtaalamu_openmrs create --given Amina --family Juma --gender F --age 30
```

Amri za kubadilisha hali (turn_on/turn_off/create) zina **HITL** — HERMES
itaomba idhini kwanza, na role `user` haiwezi kuidhinisha (admin/specialist tu).

## Hitilafu za kawaida

| Tatizo | Suluhisho |
|---|---|
| `/ha/` inarudisha 503 | `HASS_URL`/`HASS_TOKEN` hazijaingia — rudia HATUA 3, kisha `docker compose restart hermes` |
| `/openmrs/` inarudisha 502 | OpenMRS bado inaamka (inachukua ~2–5 min) — subiri, kisha jaribu tena |
| HA inaomba onboarding | Fungua `:8123` moja kwa moja mara ya kwanza, maliza onboarding |
| Build ya OpenMRS imekufa | RAM: ongeza swap → `sudo fallocate -l 4G /swapfile && sudo chmod 600 /swapfile && sudo mkswap /swapfile && sudo swapon /swapfile` |
| Nembo ya HA/OpenMRS haionekani ndani ya proxy | Hakikisha ulivuta commits za lock (HATUA 2) — source ni pamoja na frontend assets |

## Usalama (muhimu!)

- Usiweke `HASS_TOKEN` wala passwords kwenye files — Infisical pekee.
- Funga port 8123/8081 kutoka nje kwa firewall (Hermes proxy inatosha):
  ```bash
  sudo ufw allow 18080/tcp && sudo ufw enable
  ```
- Badilisha `OPENMRS_PASS` mara moja baada ya kuanza.
- Weka HTTPS mbele (Caddy/Nginx + Let's Encrypt) kabla ya kutumia mtandaoni.
