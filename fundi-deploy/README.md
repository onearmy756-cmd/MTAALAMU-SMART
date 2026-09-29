# Fundi Deploy — mass OS install (PXE + WOL + AI)

Sehemu ya **MTAALAMU SMART** ecosystem: kuweka OS kwenye computers nyingi kwa network.

## Architecture

```
Dashboard (wewe) → Fundi Cloud / local API → Fundi Server (mteja LAN)
  → DHCP/PXE/TFTP/HTTP/SMB → PCs (WOL + install)
```

## Components

| Sehemu | Path |
|--------|------|
| Site server | `server/` (docker-compose, dnsmasq, nginx, samba, Rust API) |
| Agent API | `server/agent/` — `/deploy`, `/jobs`, `/computers/discover` |
| Boot menu | `server/tftp/pxelinux.cfg/default` |
| Cloud UI (baadaye) | `cloud/ui` |
| Docs | `docs/` |

## Quick start (lab)

```bash
cd fundi-deploy/server
# Edit dhcp/dnsmasq.conf (interface, range, gateway)
./scripts/install.sh
curl http://127.0.0.1:8080/health
```

## Deploy API

```bash
curl -X POST http://SERVER:8080/deploy \
  -H 'Content-Type: application/json' \
  -d '{"os_type":"ubuntu","computers":[{"mac":"aa:bb:cc:dd:ee:ff","name":"PC-001"}]}'
curl http://SERVER:8080/jobs
```

## Safety

- Test on isolated VLAN first.
- Backup data before wipe/install.
- Windows ISO / licenses: operator responsibility.
- Multicast / full imaging: phase 2.

## Link to MTAALAMU SMART

Agentic Vision / knowledge inaweza kutoa **ticket** ya hardware; Fundi Deploy inashughulikia **OS mass deploy**.
