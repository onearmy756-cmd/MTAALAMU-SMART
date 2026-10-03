# Fundi Deploy — Phases

| Phase | Kazi | Hali |
|-------|------|------|
| **P1** | API + WOL + PXE menu + HITL + dashboard ya msimamizi | ✅ (agent Rust, docker-compose, /ui, approve/cancel) |
| **P2** | AI OS selection yenye nguvu, **full backup**, **multicast** | ✅ (ona chini) |
| **P3** | **Cloud multi-tenant** + offline-first outbox | ✅ (ona chini) |
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

## Kubaki (zaidi ya P3, si lazima kwa sasa)

- WinPE/Windows deployment halisi (WIM apply kwenye clients) — inahitaji WinPE boot images
- Config management baada ya install (Ansible/playbooks)
- Cloud dashboard ya wateja wengi (web UI tofauti na /ui) — API tayari ipo
