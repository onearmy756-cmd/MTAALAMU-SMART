# Fundi Deploy — Agent inafanya kazi; msimamizi anasimamia

## Wajibu

| Nani | Anafanya nini |
|------|----------------|
| **Agent** | Discover, chagua OS (AI/rules), WOL, PXE, job pipeline, progress |
| **Msimamizi** | RUHUSU / GHAIRI + fuatilia dashboard `/ui` |

## API

| Method | Path | Maelezo |
|--------|------|---------|
| GET | `/health` | Health |
| GET | `/computers` | Discover LAN |
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
# hariri dhcp/dnsmasq.conf
docker compose up -d --build
# UI
open http://SERVER:8080/ui
```

## Pipeline ya agent

`plan (AI OS) → HITL → backup_stub → WOL → PXE config → install ticks → report`

Images za OS weka kwenye `server/images/`. Multicast/full backup = phase inayofuata kwenye site yenye switch support.
