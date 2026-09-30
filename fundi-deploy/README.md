# Fundi Deploy — Agent inafanya kazi; msimamizi anasimamia

**P2 IMEKAMILIKA** (backup halisi, multicast, AI OS selection, discovery halisi, images validation)
**P3 IMEKAMILIKA** (cloud multi-tenant + offline-first outbox)

## Wajibu

| Nani | Anafanya nini |
|------|----------------|
| **Agent** | Discover (TCP sweep + ARP), chagua OS (profiles + AI), **backup halisi**, WOL, PXE, **multicast**, job pipeline, ripoti, cloud heartbeat |
| **Msimamizi** | RUHUSU / GHAIRI + fuatilia dashboard `/ui` |

## Pipeline (P2)

`plan (profiles+AI) → HITL → backup (files/image/stub) → WOL → PXE config → [multicast] → install → report → cloud heartbeat`

## API

| Method | Path | Maelezo |
|--------|------|---------|
| GET | `/health` | Health + cloud outbox count |
| GET | `/computers` | Discover LAN (**TCP sweep + arp-scan/arp**, merged) |
| POST | `/deploy` | Deploy + **HITL** (awaiting_approval) |
| POST | `/deploy/auto` | Deploy bila kusubiri (lab) |
| GET | `/jobs` | Orodha |
| POST | `/jobs/:id/approve` | Msimamizi anaruhusu |
| POST | `/jobs/:id/cancel` | Ghairi |
| POST | `/os/select` | `{specs, user_need}` → OS + confidence + reasons |
| GET | `/os/profiles` | Catalog ya OS profiles (data-driven) |
| GET | `/images` | Images + **sha256 + magic bytes (ISO/WIM)** |
| GET | `/images/:name` | Kagua image moja |
| GET | `/backups` | Manifests zote za backup |
| POST | `/api/mc/ack` | Multicast client ACK |
| GET | `/api/mc/status` | Hali ya multicast |
| GET | `/agents` | Agents 10 (agentic) |
| GET | `/supervisor/summary` | Muhtasari |
| GET | `/cloud/status` | Hali ya cloud (hub/edge, outbox) |
| GET/POST | `/cloud/tenants` | Orodha / ongeza tenant |
| POST | `/cloud/heartbeat` | Heartbeats kutoka edge nodes |
| GET | `/ui` | Dashboard ya msimamizi |

## Anzisha

```bash
cd fundi-deploy/server
# hariri dhcp/dnsmasq.conf (subnet yako)
mkdir -p images data tftp backups
docker compose up -d --build
# UI ya msimamizi
open http://SERVER:8080/ui
```

## Backup halisi (P2)

| Mode | Nini kinatokea | Wapi |
|------|----------------|------|
| `files` | Nakili faili halisi + manifest sha256 + **incremental** (inaruka zilizobaki) | `FUNDI_BACKUP_SOURCE` (mount/share) |
| `image` | `dd` disk image halisi | block device, mfano `/dev/sda` |
| `stub` | Maabara tu | default |

```bash
# files mode kwa mteja halisi (SMB/NFS mount kabla):
FUNDI_BACKUP_MODE=files FUNDI_BACKUP_SOURCE=/mnt/client-disk docker compose up -d
```

Manifests: `/data/backups/<mac>/<timestamp>/manifest.json` (kila faili + sha256).
Pia API: `GET /backups`.

## Multicast (P2) — image moja → PCs nyingi

1. Weka image kwenye `server/images/` (ISO/WIM; validation ya magic bytes kwenye `/images`)
2. Anza deploy na `{"multicast": true, "image": "os.iso"}`
3. Clients (PXE live/initramfs) zinaendesha `scripts/fundi_recv.py`:
   - Zinapokea UDP `239.255.0.1:5007`, zinathibitisha **sha256**, kuingiza disk, kutuma **ACK**
4. Server inasubiri ACKs (30s) — SHA mismatch = job fail

```bash
# kwenye client ya test (bila kuandika disk):
FUNDI_MC_TEST=1 FUNDI_MC_TEST_OF=/tmp/out.bin OF=/dev/null \
  MAC=aa:bb:cc:dd:ee:01 FUNDI_API=http://SERVER:8080 python3 fundi_recv.py
```

## AI OS selection yenye nguvu (P2)

- Catalog: `data/os_profiles.json` — **data-driven** (vigezo, weights, best_for, skip_rules)
- Mtiririko: skip guard (SMART/RAM fail) → rules scoring → Ollama (hiari, lazima ipitie validation) → **reasons + confidence**
- Ollama haiamuzi pekee — inapendekeza; profiles ndizo kanuni (LLM haihesabu kamwe)

```bash
curl -s localhost:8080/os/select -H 'Content-Type: application/json' \
  -d '{"specs":"16gb ram 8 cores 512 ssd","user_need":"developer docker"}' | jq
```

## Cloud multi-tenant (P3)

- Kila fundi server = **tenant** (`FUNDI_TENANT_ID`) yenye token (hashed SHA-256 kwenye DB)
- Edge → Hub: heartbeats za jobs (muhtasari tu — hakuna data nyeti)
- **Offline-first**: hakuna mtandao → outbox `/data/cloud_outbox.jsonl`; sync loop inajaribu kila dakika
- Hub ina `cloud_events` table; dashboard ya wateja wengi: `GET /cloud/tenants` + `/cloud/status`

```bash
# Hub (cloud): FUNDI_CLOUD_URL=disabled (inapokea heartbeats tu)
# Edge (site ya mteja): FUNDI_CLOUD_URL=https://hub.fundiai.co.tz FUNDI_TENANT_ID=fundi-mwanza
# Ongeza tenant kwenye hub:
curl -s localhost:8080/cloud/tenants -H 'Content-Type: application/json' \
  -d '{"id":"fundi-mwanza","name":"Fundi Mwanza","token":"secret123"}'
```

## Discovery halisi ya LAN

TCP sweep /24 (ports 135/445/22/3389) + arp-scan + arp — matokeo yameunganishwa kwa IP, MAC kutoka ARP.
Zima sweep: `FUNDI_SWEEP=0`. Demo hosts: `FUNDI_DEMO_HOSTS=1` (lab), production `=0`.

## Integration na MTAALAMU SMART

```bash
# Rust CLI (engine-rust) —Fundishia FUNDI_DEPLOY_URL kama agent si localhost:
mtaalamu deploy hosts
mtaalamu deploy os --specs "8gb ram i5" --need office
mtaalamu deploy start --macs aa:bb:cc:dd:ee:01 --os auto
mtaalamu deploy approve --id <job_id>

# R/Shiny UI (web-r) — tab "FUNDI DEPLOY"
Rscript -e "shiny::runApp('web-r', port=3838)"
# env: FUNDI_DEPLOY_URL=http://SERVER:8080
```

## Images manager (script)

```bash
python3 scripts/image_manager.py list
python3 scripts/image_manager.py add ubuntu-24.04.iso
python3 scripts/image_manager.py verify          # sha256 + magic bytes
python3 scripts/image_manager.py remove ubuntu-24.04.iso
```

## Env

| Variable | Default | Maelezo |
|----------|---------|---------|
| `FUNDI_DEMO_HOSTS` | `1` | Demo PCs kama LAN tupu (lab). Weka `0` production |
| `FUNDI_SWEEP` | `1` | TCP sweep ya LAN (zima: 0) |
| `FUNDI_AI_MODEL` | `tinyllama` | Ollama model |
| `FUNDI_TFTP` | `/var/lib/tftpboot` | PXE root |
| `FUNDI_IMAGES` | `/images` | Root ya images za OS |
| `FUNDI_BACKUP_MODE` | `stub` | `files` \| `image` \| `stub` |
| `FUNDI_BACKUP_SOURCE` | — | Chanzo cha files mode (mount/share) |
| `FUNDI_BACKUPS` | `/data/backups` | Root ya manifests za backup |
| `FUNDI_TENANT_ID` | `local` | Tenant ya edge node |
| `FUNDI_CLOUD_URL` | `disabled` | URL ya hub; `disabled` = hub mode |
