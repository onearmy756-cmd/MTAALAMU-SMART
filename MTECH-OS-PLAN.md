# MTECH OS — MPANGO KAMILI WA KIUFUNDI (SEHEMU 1–13 + AGENTIC AI)
### Bila EXE · Rust inabeba kila kitu · Docker kwenye server · Kazi za mbali = WireGuard
### Mbilinyi Tech — Umiliki ni wako, leseni ni yako, faida ni yako

> Iliyosasishwa: 2026-10-06.
> **KANUNI MPYA YA MMILIKI:** Logic iko kwenye **RUST** (structs, enums, functions, match arms) —
> SI kila kitu data-driven. JSON inabaki tu kwa config nyepesi (subnet VPN, endpoint, mtu wa malipo).
> Kila SEHEMU ya maelezo ya mmiliki ina HATUA zake, kila HATUA ina msimbo wa Rust + files halisi.

---

## SEHEMU 1 — ARCHITECTURE (Kanuni + Engines)
Kanuni: HAKUNA EXE · Rust core inabeba kila kitu · Docker deployment · offline+online ·
custom AI API inaruhusiwa · leseni ya Mbilinyi Tech.
| Engine | Lugha | Hali |
|---|---|---|
| Core Engine | Rust | ✅ fundi-deploy agent (axum, tokio, sqlx) |
| Engine ya R | R | ✅ web-r (dashboard), analytics-r |
| LLM Engine | llama.cpp → Qwen 2.5 3B VL | ✅ Ollama sasa · llama.cpp service = H4 |
| UI | Tauri v2 | H3 (desktop shell, webview) |
| VPN | WireGuard | ✅ vpn.rs (P4) |
| AI Memory | LanceDB | H4 (Rust `lancedb` crate) |
| Database | SQLite | ✅ sqlx |
| Firewall/IDS/SIEM | pfSense / Suricata / Wazuh | H4 (containers + Rust wiring) |
| Cybersecurity | Kali tools | ✅ mtech-os/kali ISO · forensics agent = H4 |

## SEHEMU 2 — UI 1: OS AND APP INSTALLATION (Rust modules za kila hatua)
| # | Mahitaji ya mmiliki | Rust module (SI JSON) | Hali |
|---|---|---|---|
| 2.1 | Discovery: IP + username + hostname (surge-ping/pnet/nmap) | `discover.rs` (TCP+ARP halisi) | ✅ · ping sweep = H3 |
| 2.2 | Majina yanayofanana: user 1,2,3… kwa mpangilio wa PC | `namer.rs` (assign_names) | ✅ |
| 2.3 | OS catalog: Kali, Windows 10/11/Server, Ubuntu + zingine; All Windows/All Ubuntu/All Kali | `catalog.rs` (enum Os::…) | H2 |
| 2.4 | Bundles 20+ katika CATEGORIES (office, browsers, dev, comms, utilities, security, graphics, drivers) | `bundles.rs` (structs + `fn catalog()` — Rust structs, si JSON) | **H2 — INAFANYIKA SASA** |
| 2.5 | Install All Apps / per-app / per-PC (PC mbali + apps zinazofanana) | `bundles.rs::DeploymentPlan` | **H2** |
| 2.6 | Bei per-PC + ClickPesa/Bank/M-Pesa → activate kabla ya kazi | `pricing.rs` (fn price_for_pc) + clickpesa ✅ | **H2** |
| 2.7 | Agents 100+; jina la user = kitambulisho cha agent per-PC | `orchestrator.rs` (100) + naming | ✅/H3 |
| 2.8 | IP addressing automatic (dnsmasq) au manual per-PC | dhcp config + API field | ✅/H3 |
| 2.9 | Recovery, Deepscan, Backup + zote za Fundi Deploy | backup.rs ✅ · deepscan.rs | ✅/H3 |

## SEHEMU 3 — UI 2: COMPUTER SOLUTIONS (Rust)
| # | Mahitaji | Rust module | Hali |
|---|---|---|---|
| 3.1 | Scan matatizo ya PC ZOTE kwa wakati mmoja (jina+tatizo → server kuu, ripoti per-PC) | orchestrator parallel + report.rs | ✅ |
| 3.2 | Ruhusa (HITL) → solve kwa wakati mmoja → ripoti kwa admin | pipeline.rs HITL ✅ | ✅ |
| 3.3 | Chat: andika tatizo/swali → agent inatua + inajibu | `chat.rs` (ReAct + LLM) | **H2 — INAFANYIKA SASA** |
| 3.4 | Auto-daily scan offline → ripoti kwa admin → subiri ruhusa → solve | `auto_daily.rs` (tokio cron) | H3 |

## SEHEMU 4 — DASHBOARD YA ADMIN (Rust API + UI)
| # | Mahitaji | Rust module | Hali |
|---|---|---|---|
| 4.1 | Health ya kila PC, maendeleo ya OS, matatizo, predictive maintenance, brands, system info, disk | hardware.rs ✅ + `admin.rs` | H3 |
| 4.2 | Remote PC zote (kupitia wg0) + chat + **kuongeza bundles kwa admin (hata wa mbali, notify)** | `bundles.rs` admin API | **H2** |
| 4.3 | Admin mkuu anagawa kazi kwa wasaidizi | `admin.rs` (roles enum: Mkuu/Msaidizi) | H3 |
| 4.4 | Dashboard ya kuongeza bundles za setup mbambali | UI + `POST /api/bundles` | **H2** |

## SEHEMU 5 — REMOTE + WIREGUARD VPN
| # | Mahitaji | Hali |
|---|---|---|
| 5.1 | Install OS/apps/drivers kwa wakati mmoja kwa mbali, nchi kwa nchi | ✅ vpn.rs + pipeline |
| 5.2 | Notify wateja wa mbali; kazi zote kupitia wg0 | ✅ · notify = H3 |
| 5.3 | Kali tools kwa cybersecurity/forensics kwa mbali kupitia agent+VPN | H4 (forensics agent) |

## SEHEMU 6 — BONUS SERVICES
3D Map ✅ (ramani-3d.html) · Navigation ✅ · Maps ✅ · Visualization ✅ · Real Monitoring ✅ (sysprobe). Icons halisi za bundles = H2 (UI).

## SEHEMU 7 — OFFLINE NA ONLINE
Offline: scan/solve/ripoti za ndani (✅) · Online kwa ruhusa ya mtu: VPN/AI/updates (✅ policy kwenye vpn.rs). Toggle ya mtumiaji = H3.

## SEHEMU 8 — LESENI, MALIPO, BIASHARA
8.1 Leseni ya Mbilinyi Tech (MST-XXXX tiers ✅, leseni nzuri = H3 UI) · 8.2 Malipo: ClickPesa ✅ + Bank + M-Pesa (H2 pricing) · 8.3 Packages: pay-per-use + subscription kila moja na bei yake → **H2 `pricing.rs` (enum Plan)** · 8.4 Support: 079675645, WhatsApp, IG @mbilinyitech, mbilinyitech@gmail.com, mbilinyitech.co.tz → kwenye UI zote (H2 footer).

## SEHEMU 9 — UI, DASHBOARD, VISUALIZATION
UI nzuri + visualization + icons halisi + charts + 3D + real monitoring ✅ msingi · Login kwa default credentials + manage section = H3 · Admin dashboard (PC zote + maendeleo + matatizo) = H3 · Dashboard ya kuongeza bundles = **H2**.

## SEHEMU 10 — UPDATES NA GITHUB
CI (cargo test/build) kwenye kila push = **H2 (workflow)** · Release artifacts (tar/deb, HAKUNA exe) = H5 · Update notification kwa wateja (version check kwenye agent) = H5.

## SEHEMU 11 — STORAGE NA DATABASES
ISOs/Apps/Drivers/Models → disk ✅ · Deployments/Problems/Computers → LanceDB AI memory (H4) · Agents/Payments/Transactions → SQLite ✅ · Logs → disk ✅.

## SEHEMU 12 — SERVER (DOCKER) — compose kamili
api (Rust ✅) · R engine (H4) · llama.cpp+Qwen 2.5 3B VL (H4) · LanceDB (H4) · SQLite ✅ · WireGuard ✅ · Suricata (H4) · Wazuh (H4) · pfSense (VM, H4 docs) · Kali tools (H4) · nginx/dnsmasq/samba ✅.

## SEHEMU 13 — AGENTIC AI (ReAct + Bounded Autonomy) — Rust loop
| # | Mahitaji | Rust module | Hali |
|---|---|---|---|
| 13.1 | ReAct: THINK → ACT (ping/cmd kwenye PC nyingi) → OBSERVE → THINK… | `agent/reacon.rs` (enum Step::Think/Act/Observe) | **H2 — INAFANYIKA SASA** |
| 13.2 | Bounded autonomy: low-risk auto (scan/cache) · high-risk HITL (reboot/wipe/install) | risk enum + HITL ✅ | **H2** |
| 13.3 | Shared memory: kila agent inajifunza kutoka wengine (LanceDB) | H4 | ⏳ |
| 13.4 | Vision-driven (screenshots → Qwen VL → action) kwa remote GUI | H3 | ⏳ |

---

## MPANGO WA UTEKELEZAJI — HATUA (code kwa RUST)
- **H1 ✅** P4: vpn.rs + namer.rs + bug fixes (PR #6, tests 19/19)
- **H2 ✅** bundles.rs (Rust structs, bundles 22 / categories 8) + pricing.rs (TZS, plans) +
  reacon.rs (ReAct + bounded autonomy + confidence_score) + API + UI (📦 🤖) + CI (PR #6, tests 34/34)
- **H2b ✅** brain.rs — Neuralis Brain (shared memory ya agents) + confidence_score + fleet exec (PR #7, tests 40/40)
- **H3 ✅** auth.rs (login defaults admin/fundi + sessions + change_password) · admin.rs (admin mkuu anagawa
  kazi kwa wasaidizi, role gate) · license.rs (MST- keys, Personal/Business/Enterprise, issue/validate/revoke) ·
  mode.rs (offline/online toggle — mtu anaamua) · chat.rs (agent inajibu: brain offline + Ollama/llama.cpp online,
  Qwen 2.5 3B VL) · daily.rs (scan ya kila siku → ripoti → RUHUSA → solve) · updates.rs (check + notification) ·
  UI kamili (/ui: 🔐 Admin, 📋 Kazi, 🔓 Leseni, 🔌 Mode, 🔄 Updates, 📅 Kila Siku, 💬 Chat) · Tauri v2 workspace
  (mtech-os/desktop/src-tauri — UI mbili, targets deb/AppImage/dmg, HAKUNA exe) — (PR #8, tests 58/58)
- **H4 ✅** docker-compose: llamacpp (Qwen2.5-VL-3B GGUF, port 8081) + suricata (IDS, config kwenye
  security/suricata/) + wazuh-manager (SIEM) + kali-tools (pentesting/forensics container) + PFSENSE.md
  (firewall rules, SEHEMU 12). LanceDB vector search — interface ya brain.rs tayari (H4b: swap ndani ya module)
- **H5 ✅** release.yml (tag v* → tests → build → tar.gz artifacts → GitHub Release; notification
  kupitia updates.rs — wateja wanaona version mpya) · pfSense API wiring kwenye agent = H5b (opsional)

## UTHIBITISHO (kila hatua)
`cargo test` (tests mpya kwa kila module ya Rust) · `cargo build --release` · API smoke (curl) · UI review (/ui).
