# Fundi Deploy — Agent inafanya kazi; msimamizi anasimamia

## Wajibu

| Nani | Anafanya nini |
|------|----------------|
| **Agent** | Discover, chagua OS (AI/rules), WOL, PXE, job pipeline, progress, ripoti |
| **Msimamizi** | RUHUSU / GHAIRI + fuatilia dashboard `/ui` |

## API

| Method | Path | Maelezo |
|--------|------|---------|
| GET | `/health` | Health |
| GET | `/computers` | Discover LAN (+ demo hosts lab) |
| POST | `/deploy` | Deploy + **HITL** (awaiting_approval) |
| POST | `/deploy/auto` | Deploy bila kusubiri (lab) |
| GET | `/jobs` | Orodha |
| POST | `/jobs/:id/approve` | Msimamizi anaruhusu |
| POST | `/jobs/:id/cancel` | Ghairi |
| GET | `/supervisor/summary` | Muhtasari |
| GET | `/ui` | Dashboard ya msimamizi |

## Anzisha

```bash
cd fundi-deploy/server
# hariri dhcp/dnsmasq.conf (subnet yako)
mkdir -p images data tftp
docker compose up -d --build
# UI ya msimamizi
open http://SERVER:8080/ui
```

## Pipeline ya agent

`plan (AI OS) → HITL → backup_stub → WOL → PXE config → install ticks → report`

Images za OS weka kwenye `server/images/`. Multicast/full backup = phase inayofuata.

## Env

| Variable | Default | Maelezo |
|----------|---------|--------|
| `FUNDI_DEMO_HOSTS` | `1` | Demo PCs kama LAN tupu (lab). Weka `0` production |
| `FUNDI_AI_MODEL` | `tinyllama` | Ollama model |
| `FUNDI_TFTP` | `/var/lib/tftpboot` | PXE root |
