# Fundi Deploy — Phases

| Phase | Kazi | Hali |
|-------|------|------|
| **P1** | API + WOL + PXE menu + HITL + dashboard ya msimamizi | ✅ (agent Rust, docker-compose, /ui, approve/cancel) |
| **P2** | AI OS selection yenye nguvu, **full backup**, **multicast** | ✅ (ona chini) |
| **P3** | **Cloud multi-tenant** + offline-first outbox | ✅ (ona chini) |
| **P4** | **WIREGUARD VPN** + **MAJINA YA KIOTOMATIKI** | ✅ (ona chini) |
| **Zaidi** | Discovery halisi ya LAN, images validation, integration (engine-rust CLI + Shiny tab) | ✅ |

## P2 — iliyotekelezwa

1. **OS selection yenye nguvu** (`agent/src/osselect.rs` + `data/os_profiles.json`)
   - Skip guard (SMART/RAM/disk fail → `skip`, msimamizi arekebishe hardware kwanza)
   - Rules scoring data-driven: weights kwa kila OS (kutoka JSON, si hard-coded)
   - Ollama ni **mpendekezi** tu — lazima ipitie validation ya profiles (LLM haiamuzi)
   - Output: `os + confidence + method + reasons[]` — kila chaguo kina sababu (auditable)
   - Tests: skip guard, developer→linux, office→windows, reasons zisitupwe
2. **Full backup** (`agent/src/backup.rs`)
   - `files`: nakili halisi + manifest sha256 + **incremental** (baseline = manifest ya mwisho)
   - `image`: `dd` halisi ya block device
   - `stub`: maabara tu (default kwa usalama)
   - API: `GET /backups` (manifests zote)
3. **Multicast** (`agent/src/multicast.rs` + `scripts/fundi_recv.py`)
   - UDP 239.255.0.1:5007, manifest→chunks→END; client inathibitisha sha256, inaandika disk, inatuma ACK
   - Pipeline: inasubiri ACKs (30s); mismatch = job fail
   - UI: badge ya MC stage kwenye jobs table

## P3 — iliyotekelezwa

1. **Tenants** (`agent/src/cloud.rs`): `tenants` table (token hashed SHA-256), `POST /cloud/tenants`
2. **Heartbeats**: edge → hub (`POST /cloud/heartbeat`), hub inahifadhi `cloud_events`
3. **Offline-first outbox**: `/data/cloud_outbox.jsonl` + sync loop (kila dakika, retry)
4. **Pipeline** inatuma heartbeat kila job inapokamilika

## Zaidi ya phases

1. **Discovery halisi**: TCP sweep /24 (135/445/22/3389) + arp-scan + arp → merge kwa IP; `FUNDI_SWEEP=0` kwa kuzima
2. **Images**: `GET /images` na sha256 (≤ limit) + magic bytes (ISO9660 `CD001`, WIM `MSWIM`); `image_manager.py` list/add/verify/remove
3. **Integration engine-rust**: `mtaalamu deploy hosts|jobs|images|summary|cloud|os|start|approve|cancel`
4. **Integration Shiny**: tab **FUNDI DEPLOY** kwenye web-r (discovery, deploy HITL, jobs, images, OS select, agents, cloud)

## P4 — WIREGUARD VPN + MAJINA (2026-10)

1. **VPN ya WireGuard** (`agent/src/vpn.rs` + `data/deploy/wireguard.json`)
   - Keys halisi za Curve25519: `wg genkey` kama ipo, vinginevyo x25519-dalek (pure Rust)
   - Server keys zinahifadhiwa `/data/wireguard/server.key` (0600) — zinaendelea kwenye volume
   - Peers kwenye SQLite (`wg_peers`): kila computer/site ya mbali ina IP yake (10.66.66.2+)
   - Client conf inapakuliwa kwa kila peer: `GET /api/vpn/peers/<name>/conf`
   - `wg-quick up/down` HALISI (inahitaji NET_ADMIN + /dev/net/tun — compose tayari inaweka)
   - Status halisi: `wg show wg0 dump` (handshake, rx, tx) → `/api/vpn/status`
   - **Policy: kazi ZOTE za mbali (discovery, OS, apps, drivers, ripoti) zinaenda kupitia wg0**
   - Scan ya subnet ya VPN: `GET /api/vpn/scan` (TCP halisi juu ya tunnel)
   - Tests: keypair (44-char b64), unique, server/client conf, subnet base
2. **Majina ya kiotomatiki** (`agent/src/namer.rs`)
   - Computer zenye jina LILE LILE (mf "hr") zinapewa: `hr`, `hr 1`, `hr 2`, `hr 3`…
   - Mpangilio unafuata IP ascending (mpangilio wa computer kwenye mtandao)
   - `/computers` sasa inarudisha `display_name` (dedupe) + `renamed` + `index`
   - Tests: dedupe 3× "hr", majina tofauti hazibadilishwi, jina tupu → "pc"

### API mpya (P4)

| Method | Path | Maelezo |
|--------|------|---------|
| POST | `/api/vpn/init` | Keys za server + wg0.conf (peers zote) |
| GET | `/api/vpn/status` | Tunnel + peers + policy (wg show halisi) |
| GET/POST | `/api/vpn/peers` | Orodha / ongeza peer (keypair mpya + IP ya bure) |
| GET | `/api/vpn/peers/:name/conf` | Pakua client conf (wg-quick ready) |
| DELETE | `/api/vpn/peers/:name` | Ondoa peer + regen conf |
| POST | `/api/vpn/up` / `down` | Washa / zima tunnel (wg-quick halisi) |
| GET | `/api/vpn/scan` | Scan subnet ya VPN (hosts + services) |
| GET | `/api/vpn/server-conf` | Pakua wg0.conf ya server |

### Anzisha VPN (dakika 3)

```bash
# 1. Endpoint ya public kwenye docker-compose (IP au domain ya server):
#    FUNDI_WG_ENDPOINT=196.x.x.x docker compose up -d --build api
# 2. UI: http://SERVER:8080/ui → tab 🔒 VPN → INIT → ongeza PEER ("hr") → pakua conf
# 3. Computer ya mbali: wg-quick up wg-hr.conf
# 4. Sasa discovery/install/ripoti za mbali zinaenda kupitia tunnel (wg0)
```

## Kubaki (zaidi ya P4, si lazima kwa sasa)

- WinPE/Windows deployment halisi (WIM apply kwenye clients) — inahitaji WinPE boot images
- Config management baada ya install (Ansible/playbooks)
- Cloud dashboard ya wateja wengi (web UI tofauti na /ui) — API tayari ipo
