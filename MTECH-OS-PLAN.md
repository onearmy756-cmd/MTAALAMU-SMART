# MTECH OS — MPANGO KAMILI WA KIUFUNDI (SEHEMU 1–13 + AGENTIC AI)
### Bila EXE · Rust inabeba kila kitu · Docker kwenye server · Kazi za mbali = WireGuard
### Mbilinyi Tech — Umiliki ni wako, leseni ni yako, faida ni yako

> Iliyosasishwa: 2026-10-06. Inafuata maelezo yote ya mmiliki (SEHEMU 1–13) + Agentic AI
> (ReAct loop + bounded autonomy/HITL). Kila HATUA ina deliverables halisi + file paths.

---

## JUKWAA LA ARCHITECTURE (SEHEMU 1) — kanuni zisizokiukwa

| Sehemu | Engine | Hali |
|---|---|---|
| Core Engine | **Rust** (inabeba mfumo mzima) | ✅ engine-rust + fundi-deploy agent |
| Engine ya R | R (uchanganuzi, ripoti, statistics) | ✅ web-r + analytics-r |
| LLM Engine | llama.cpp → **Qwen 2.5 3B VL** | ✅ Ollama (Qwen 2.5 VL) · llama.cpp server = HATUA 4 |
| UI | **Tauri v2** | HATUA 3 (mtech-os/desktop) |
| Server | Rust + **Docker** | ✅ fundi-deploy/server compose |
| VPN | **WireGuard** | ✅ P4 (vpn.rs) |
| AI Memory | **LanceDB** (vector search) | HATUA 4 (docker service + Rust client) |
| Database | **SQLite** (agents, payments, transactions) | ✅ sqlx |
| Firewall | pfSense | HATUA 4 (config + docs) |
| IDS | Suricata | HATUA 4 (docker + config) |
| SIEM | Wazuh | HATUA 4 (docker + config) |
| Cybersecurity | Kali Linux tools (digital forensics, pentesting) | ✅ mtech-os/kali (ISO) |

**Kanuni za msingi:** HAKUNA EXE · Docker deployment · offline + online (mtu anaamua) ·
API yako ya AI / custom model inaruhusiwa · leseni ya Mbilinyi Tech.

---

## HATUA 1 — UI YA 1: OS AND APP INSTALLATION (Fundi Deploy)
| # | Kazi | Files | Hali |
|---|---|---|---|
| 1.1 | Discovery halisi (IP, username, hostname) | discover.rs | ✅ |
| 1.2 | Majina ya kiotomatiki (hr, hr 1, hr 2… kwa mpangilio wa IP) | namer.rs | ✅ |
| 1.3 | ISO catalog: Kali, Windows 10/11/Server, Ubuntu + zingine | images.rs | ✅ (+magic bytes) |
| 1.4 | Chagua "All Windows / All Ubuntu / All Kali" au per-PC | main.rs deploy | ✅ (os_type) |
| 1.5 | **Bundles 20+ katika CATEGORIES** (office/browsers/dev/comms/utilities/security/graphics/drivers) | data/deploy/app_bundles.json + bundles.rs | 🔄 HATUA 2 ya sasa |
| 1.6 | "Install All Apps" au per-app per-PC | main.rs deploy + UI | 🔄 HATUA 2 |
| 1.7 | Bei kwa kompyuta + ClickPesa/bank/M-Pesa → activate | pricing + clickpesa | ✅ (ClickPesa halisi) · pricing per-PC = HATUA 2 |
| 1.8 | Agents 100+ (jina la user = kitambulisho) | orchestrator.rs | ✅ (concurrency 100) · agent-per-PC naming = HATUA 3 |
| 1.9 | IP addressing: automatic (dnsmasq) au manual per-PC | dhcp/dnsmasq.conf | ✅ (+UI hint) |
| 1.10 | Recovery, Deepscan, Backup | backup.rs + deep | ✅ |

## HATUA 2 — UI YA 2: COMPUTER SOLUTIONS
| # | Kazi | Files | Hali |
|---|---|---|---|
| 2.1 | Scan matatizo kwa kompyuta ZOTE kwa wakati mmoja (jina + tatizo → server kuu) | orchestrator + report.rs | ✅ (parallel safe-steps) |
| 2.2 | HITL: ruhusa → solve kwa wakati mmoja → ripoti per-PC | pipeline.rs HITL | ✅ |
| 2.3 | Chat ya maswali/matatizo → agent inajibu (Qwen 2.5 3B VL) | chat endpoint + ui | HATUA 3 |
| 2.4 | Auto-daily scan (offline) + ripoti kwa admin + subiri ruhusa | cron + report | HATUA 3 |

## HATUA 3 — TAURI v2 UI (desktop bila EXE ya wageni)
| # | Kazi | Files |
|---|---|---|
| 3.1 | Tauri v2 workspace (cargo tauri) kwenye mtech-os/desktop | src-tauri/ |
| 3.2 | UI 1: OS AND APP INSTALLATION · UI 2: COMPUTER SOLUTIONS (webviews) | src/ |
| 3.3 | Icons halisi za bundles, charts, 3D map (MapLibre) | www/ |
| 3.4 | Admin: ku-remote PC zote, chat, kuongeza bundles (hata wa mbali — notify) | src/ + api |

## HATUA 4 — SERVER KAMILI (Docker)
| Service | Kazi | Hali |
|---|---|---|
| api (Rust) | Core engine | ✅ |
| ollama / llama.cpp | Qwen 2.5 3B VL (GGUF) | ✅ Ollama · llama.cpp = nchi hii |
| lancedb | AI Memory (vector search) | HATUA 4 |
| sqlite | Agents, payments, transactions | ✅ |
| wireguard | VPN (wg0, port 51820/udp) | ✅ |
| suricata | IDS (docker + config) | HATUA 4 |
| wazuh | SIEM (docker + config) | HATUA 4 |
| pfSense | Firewall (VM/nje ya docker — config + docs) | HATUA 4 |
| kali-tools | Cybersecurity/forensics container | HATUA 4 |
| nginx/dnsmasq/samba | HTTP / DHCP-PXE-TFTP / images | ✅ |

## HATUA 5 — GITHUB + UPDATES + LESENI
| # | Kazi | Hali |
|---|---|---|
| 5.1 | CI: cargo test + build kwenye kila push | 🔄 .github/workflows |
| 5.2 | Release artifacts (tar/deb — HAKUNA exe ya wageni) | HATUA 5 |
| 5.3 | Update notification kwa wateja (version check) | HATUA 5 |
| 5.4 | Leseni MST-XXXX-XXXX tiers + branding Mbilinyi Tech | ✅ data + docs |

## AGENTIC AI (ReAct + Bounded Autonomy) — inavuka UI zote
| # | Kazi | Hali |
|---|---|---|
| A.1 | ReAct loop: THINK → ACT (ping/cmd) → OBSERVE → THINK… | engine-rust agentic ✅ (msingi) · per-PC loop = HATUA 3 |
| A.2 | Bounded autonomy: low-risk auto (cache, scan) · high-risk HITL (reboot, wipe, install) | ✅ HITL pipeline |
| A.3 | Shared memory ya agents (LanceDB): kila agent inajifunza kutoka wengine | HATUA 4 |
| A.4 | Vision-driven (screenshots → LLM → action) kwa remote GUI | HATUA 3 (Qwen VL) |

## CUSTOMER SUPPORT (SEHEMU 8.4) — kwenye UI zote
Simu **079675645** · Calls · SMS · WhatsApp · IG **@mbilinyitech** · **mbilinyitech@gmail.com** · **mbilinyitech.co.tz**

## OFFLINE/ONLINE (SEHEMU 7) + PAYMENTS (8.2–8.3)
- Offline: scan/solve/ripoti za ndani · Online (mtu anaruhusu): VPN work, AI, updates
- Malipo: ClickPesa ✅ · Bank · M-Pesa — per-PC pricing + packages (pay-per-use / subscription) = HATUA 2

## HALI YA KUZUIA (uwazi)
WinPE halisi (license ya Microsoft) · pfSense = VM nje ya docker · LanceDB+llama.cpp containers = HATUA 4.
