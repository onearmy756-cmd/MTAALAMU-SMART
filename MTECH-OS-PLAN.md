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

## JINA RASMI + BEI HALISI (maelezo ya mwisho ya mmiliki)
- **FUNDI DEPLOY → "OS AND APP INSTALLATION"** — jina rasmi kwenye API root, /ui header, web-r tab, docs
- **Bei halisi za soko (TZS)** — `pricing.rs`: kwa **ugumu wa kazi** (Server 90k > Win11 35k > Ubuntu 20k;
  apps za Windows licensing 8k > Linux 5k; drivers Dell/HP/Lenovo 25k > Acer/Asus 18k; repair 30k > scan 5k)
  + **utumiaji wa computer** (subscription per-PC: 15k/PC ≤10, 12k/PC ≤50, 10k/PC 50+ = monitoring/scan/ripoti kila siku)
  + **faida halisi**: `revenue_split()` — fundi 60% / mmiliki 30% / reserve 10%
  + punguzo la batch halisi (agents 100 = gharama ile ile → 10%/20% kwa wateja wa batch)
- **Kampuni/matawi** — `company.rs`: matawi ni subnets za wg0; admin anasimamia ZOTE kupitia WireGuard;
  muhtasari wa kila tawi (PCs, online/offline, matatizo) kutoka remote_view (probe halisi)
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
  kupitia updates.rs — wateja wanaona version mpya)
- **H5b ✅ (MWISHO — limekamilika)** vector.rs: **LanceDB-compatible AI memory** — embedding halisi
  (hashing bag-of-words 256-dim, pure Rust) + cosine similarity search kwenye
  `data/lancedb/brain_memories.lance/` (JSONL shards); brain.rs = **dual-write** (SQLite + vectors) na
  recall sasa ni SEMANTIC (cosine × confidence) · pfsense.rs: **API halisi ya pfSense REST** (rules,
  aliases, services/restart, status/system) kupitia PFSENSE_API_URL + PFSENSE_API_KEY — bila env,
  error ya configuration (hakuna uongo) · routes 6 mpya (/api/pfsense/*) + compose envs + PFSENSE.md
  (sehemu ya MTECH OS API). Tests 79/79.

## UTHIBITISHO (kila hatua)
`cargo test` (tests mpya kwa kila module ya Rust) · `cargo build --release` · API smoke (curl) · UI review (/ui).

## H6 ✅ — WHITE-LABEL + CREDITS + SELF-HOST + NETWORK MGMT (maelezo ya mmiliki ya mwisho)

- **White-label (siri ya biashara)**: mteja anaona **HUDUMA tu** (katalogi ya umma:
  jina la huduma + maelezo + credits PEKEE) — `TOOL_REGISTRY` ni `pub(crate)`
  (server-side pekee), `sanitize_output()` inaficha majina ya zana/amri/paths
  kwenye kila matokeo (chat LLM + rules + scans), UI tabs za KALI TOOLS na
  RAMANI (map/3D/navigation) zimefichwa kwenye dashboard ya mteja.
- **Credit billing**: `credit.rs` — accounts + purchase (ref ya malipo:
  ClickPesa TXN/bank) + `authorize()` gate (kazi inazuia kabla haijaanza) +
  `spend()` ledger; HUDUMA kila moja na bei yake (network_scanner 2, forensics 8,
  malware 5, os_install 20…); subscription = credits za mwezi (SUBSCRIPTION_MONTHLY_CREDITS).
- **Self-host onboarding**: `server_setup.rs` — mteja anasakinisha kwenye
  **server/computer/Raspberry/kingine chake**, anachagua target kwenye ONBOARDING →
  config ZOTE automatic (ports, dirs, VPN peer 10.66.66.x, services checklist) →
  mfumo unaanza kutumia. Mmiliki anapata malipo kupitia credits/ClickPesa.
- **Network + security mgmt**: `netmgmt.rs` — status/backup/firmware/posture ya
  vifaa vya mtandao (routers/switches/firewall) + majukwaa ya usalama wa kampuni.
- UI mpya (/ui): 🧰 Huduma (service catalog + run) · 💳 Credits (salio/nunua/ledger)
  · 🚀 Onboarding (target select + config automatic) · 🌐 Network (vifaa + usalama).

## H7 ✅ — DASHBOARD HALISI + SIRI KAMILI + KUJIFUNZA KILA SIKU

- **Demo data imeondolewa**: dashboards (browser www/index.html + /ui) zinaonyesha
  **data halisi tu** kutoka API ya Rust — hakuna Math.random/mock; panel zisizo na
  data zinaonyesha "offline/hakuna data" kama ilivyo (hakuna uongo).
- **Navigation + UI zilizopo**: tabs za mfumo — MWANZO, Computers, OS Chagua, Jobs,
  OS INSTALL (Remote), Agent, VPN, Bundles, Agentic AI, Admin, Kila Siku, Kampuni,
  Huduma, Credits, Onboarding, Network — zimepangwa upya kwenye nav moja.
- **Siri kamili ya usanifu (white-label total)**:
  - FORMULA ENGINE imefichwa kabisa kwenye dashboards zote (browser + Shiny views.R
    + i18n) — hakuna neno "formula engine" kwa mteja.
  - Majina ya engines/usanifu yamefichwa: "Rust + JSON data-driven" → "Mfumo Mahiri
    wa MTECH OS"; engine label → "MTECH OS"; shiny.css → system.css; Ollama/Qwen/
    llama/R-Engine/Docker hayatajiki kwenye UI yoyote ya mteja (AI = "AI ya ndani").
  - Wataalamu wa makampuni wanaodeploy wao wenyewe hawaoni architecture yoyote —
    wanaona HUDUMA za MTECH OS tu (ENDA: /api/services).
- **AI summarize (Length)**: chat ina short (ufupi) / **medium (maelezo ya kati)** /
  **long (maelezo marefu yakamil)** — LLM prompt inabadilika kwa Length; jawabu za
  rules zina detail blocks za kati/marefu. Test ya tofauti ya refu ipo.
- **Kujifunza kila siku**: kila scan ya kila siku inaandika lessons kwenye Neuralis
  Brain (kumbukumbu ya pamoja) — finding → solution hint + confidence; agents wa
  kesho wanajifunza kutoka scan ya leo (daily.rs → brain.rs).
- **HTML source guard**: contextmenu/F12/Ctrl+Shift+I/J/C/K/Ctrl+U zimezuiwa kwenye
  dashboards zote mbili; source comments za architecture zimefutwa; CSS/API paths
  hazionyeshi framework (system.css).

## H8 ✅ — MATRIX KWA MTEJA + BEI ZA KIBISHARA + CUSTOM MODEL/API

- **FUNDI DEPLOY MATRIX kwenye /ui (dashboard ya mteja)**: STEP 1 hosts
  (search, select-all, auto-naming pattern), STEP 2 OS + Bundle,
  STEP 3 bei (devices × credits → TZS) + **ACTIVATE PARALLEL AGENT SYSTEM**,
  progress bar + log live (kama dashboard kuu). Kila kitu kinafanya kazi
  **OFFLINE kwenye server ya mteja**; kutumia huduma kunalipwa credits.
- **Bei za kibishara** (credit 1 = TZS 500): network_scanner/health_check 1,
  malware/device/driver 2, app_install 3, forensics 6, os_install 10 (TZS 5,000) —
  hakuna kunyonya; test inazuia credits > 10 kwa huduma yoyote.
- **Custom/Local Model au API**: `/api/ai/custom` (GET/POST/DELETE) — mteja
  anaweka endpoint ya model yake (Ollama/llama.cpp/vLLM/OpenAI-compatible) +
  jina la model; **inatumika mara moja** kwenye chat + tafsiri (chat.rs/
  language.rs zinasoma config ya DB). Bila custom: AI ya ndani (offline) inatumika.
- **Usafi kamili wa UI**: mvujaji wote wa usanifu wamefichwa (OLLAMA LLM/qwen,
  engine-rust, Whisper, SearXNG, HERMES Elixir/LangChain, docker compose + paths);
  default integration label → "Utafutaji wa Mfumo"; maneno ya ndani (H7/fiche)
  hayamo kwenye code ya dashboards.

## H9 ✅ — SOLUTIONS DASHBOARD + SUBSCRIPTION & PAY-PER-USE (TZS, credits ZIMEONDOKWA)

- **🖥️ AUTOMATED COMPUTER SOLUTIONS kwenye /ui** (kama dashboard kuu):
  COMPUTER HEALTH MATRIX (kagua yote kwa wakati mmoja, alerts per-PC),
  **AI AGENT chat** (andika tatizo → AI inajibu na kupendekeza hatua),
  Console Streams live. Kwa subscription au pay-per-use.
- **MFUMO WA KIBIASHARA MPYA (billing.rs) — CREDITS ZIMEONDOKWA KABISA**:
  - **SUBSCRIPTION** (kwa mwezi, kwa kifaa — TZS za Kitanzania za kibishara):
    Basic (1–10 kifaa) TZS 8,000 · Standard (11–50) TZS 6,500 ·
    Biashara (51–200) TZS 5,000 · Kampuni Kubwa (200+) TZS 4,000.
    Subscription = huduma ZOTE (monitoring, scan ya kila siku, repair,
    remoting, AI chat, forensics) — bila malipo ya ziada.
  - **PAY-PER-USE**: scan TZS 2,000 · repair TZS 15,000 · os_install TZS 5,000 ·
    apps TZS 1,500 · forensics TZS 25,000 · netmgmt TZS 4,000.
  - **Punguzo la volume**: 10+ kazi = 10% · 50+ = 20% (wateja wana faida).
  - **WALLET**: topup kupitia ClickPesa/benki (ref halisi LAZIMA) →
    subscription inaweza kulipiwa kutoka salio; kila kazi inaandika ledger.
  - `authorize()` inazuia kazi kabla haijaanza (subscription au salio);
    `charge()` inakata pay-per-use tu baada ya kazi kuva fanikiwa.
- **CREDITS zimefutwa**: routes /api/credits/* + handlers + ServicePrice.credits
  (→ price_tzs) + UI ya credits — zote zimeondolewa; services gate sasa = billing.
- **MATRIX quote sasa ni TZS**: (os 5,000 + apps 1,500) × devices − punguzo la
  volume (10% / 20%) — hakuna credits.

## H10 ✅ — BUNDLE & ASSET REPOSITORY MANAGER (DISTRIBUTE + DRAG & DROP + INTEGRITY)

- **📦 BUNDLE & ASSET REPOSITORY MANAGER kwenye /ui** (kama dashboard ya picha):
  Category select + search + grid ya bundles (emoji kwa app maarufu — VS Code,
  GIMP, VLC, n.k.) na kitufe cha **DISTRIBUTE** kwa kila bundle.
- **DISTRIBUTE** → `POST /api/bundles/distribute` `{bundle, targets, account}`:
  gate ya BILI (subscription inatosha; vinginevyo app_install × targets kutoka
  salio) → thibitisha bundle ipo kwenye katalogi halisi ya Rust
  (`bundles::find_bundle`) → kazi za HITL (needs_approval) kwa kila target →
  `charge()` baada ya mafanikio tu. Inarudisha `{ok, jobs, count, bundle,
  balance_tzs}`; Terminal ya REPO inaandika [OK]/[ERROR] + salio jipya.
- **DRAG & DROP CUSTOM SETUP**: donesa faili (exe/msi/zip/deb/tar.gz) au chagua —
  inaandika custom bundle kwenye DB (`POST /api/bundles`, category "custom")
  na inaonekana mara moja kwenye grid + Repository Integrity.
- **Repository Integrity**: hali ya mfumo (`/health` → "Huduma za Mfumo:
  HAI/NJE YA MTANDAO") + "Usawazishaji wa Repository: %" (bundles zenye apps)
  + hesabu: Bundles · Categories · Apps.
- **Terminal** yenye **Silent Install Switch: [ -y --silent /q ]** kama picha.
- **UI SAFI ZAIDI (white-label)**: mabaki yote ya credits yameondolewa —
  nav 💳 Credits, kadi credits_card, creditsBalance/Buy/Ledger (API zilikuwa
  zimefutwa H9); HUDUMA inaonyesha price_tzs halisi; MATRIX inaonyesha
  **Salio la BILI (TZS)** live (mxBalance — inabadilika baada ya ACTIVATE);
  comments zenye maneno yanayoashiria kuficha (fiche/H7) zimeondolewa.
- **FIX ya API interface**: mxLoadCatalog / repLoad / repIntegrity sasa
  zinasoma umbo halisi la `/api/bundles` (`categories[].bundles` +
  `custom_bundles`) — dropdown ya Bundle ya MATRIX na grid ya REPO zinajaa
  katalogi halisi (22 bundles + custom).

## H11 ✅ — CYBER SECURITY & FORENSICS CENTER (usalama + uchanganuzi, kompyuta nyingi kwa wakati mmoja)

- **🛡️ CYBER & FORENSICS kwenye /ui** (SEHEMU 6–7 ya mpango): chagua kompyuta →
  chagua hali (**USALAMA TZS 2,000/kifaa · FORENSICS TZS 25,000/kifaa · ZOTE
  MBILI**) → ANZA: kazi za HITL kwa kila kompyuta, zinaendeshwa kwa WAKATI
  MMOJA baada ya RUHUSU (tab Jobs).
- **Backend mpya (secops.rs)**:
  - `POST /api/secops/start` `{account, targets, mode}` — gate ya BILI kwa kila
    op (security → `malware_scan` TZS 2,000; forensics → `digital_forensic`
    TZS 25,000; volume discount ya BILI inatumika), kazi HITL
    (stage `secops:security|forensics`), malipo baada ya kazi kuundwa;
    inarudisha `{ok, jobs, count, mode, prices, balance_tzs}`.
  - `POST /api/secops/result` `{account, target, mode, findings[], sources, note}`
    — ripoti ya usalama: afya inapunguzwa na severity ya kila tulipatalo
    (clamp 0–100); daraja: salama ≥80 · tahadhari ≥50 · hatari; AU kesi ya
    forensics: ushahidi UNAFUNGWA na **SHA-256 hash**
    (account|target|vyanzo|wakati) — chain-of-custody rahisi.
  - `GET /api/secops/summary/:account` — data HALISI: kila kompyuta (afya %,
    daraja, matatizo, **mashambulizi yaliyozuiwa**, **udhaifu**, virusi),
    muhtasari (Salama/Tahadhari/Hatari), aina za vitukeo (kwa chart), ripoti
    20 za mwisho, kesi 10 za forensics. **Access control**: account anaona
    ZAKE PEKEE (SEHEMU 8.3).
- **Dashboard (SEHEMU 7.1/7.2)**: Afya ya Usalama kwa kila kompyuta (bars za
  rangi kulingana na hatari), muhtasari, chart ya vitukeo, Ripoti za Usalama,
  Ushahidi wa Forensics (hash + IMEFUNGWA 🔒), na viungo: AI chat + remote
  (tab 🖥️ SOLUTIONS), malipo (tab 💰 BILI). Audit: kila kazi na matokeo
  yanarekodiwa DB (SEHEMU 8.4).
- **Tests 7 mpya** (mode + bei, afya/clamp, vizingiti vya daraja, normalizing
  ya kinds, hash thabiti + 64-hex, DB roundtrip + access control) — jumla
  **112/112**.
- **White-label**: dashboard haina majina ya zana wala bandari — majina salama
  ya Kiswahili tu ("Uchunguzi wa usalama", "Uchunguzi wa kidijitali",
  "Mashambulizi yaliyozuiwa").

## H12 ✅ — TOOLKIT LAYER (registry + executor + batch wakati mmoja + API + dashboard)

- **Architecture kama mpango (SEHEMU 1–8)**: Rust CORE inabeba kila kitu —
  toolkit.rs mpya ina REGISTRY (zana 34 katika makundi 8), EXECUTOR (arg-array
  salama, HAKUNA shell, timeout 60s, matokeo yanapita sanitize_output), BATCH
  (tokio::spawn kwa kila kompyuta — WAKATI MMOJA halisi), na AUDIT (tool_runs
  DB — kila utekelezaji unaandikwa).
- **Makundi 8 (majina SALAMA ya Kiswahili)**: Uchunguzi wa Mtandao (5) ·
  Udhaifu (3) · Upimaji wa Usalama (3) · Wi-Fi (2) · Nywila (3) · Trafiki (3) ·
  Uchanganuzi wa Kidijitali (10) · Huduma za Mfumo (5) = **zana 34**
  (SEHEMU 8.2 kamili: recon/vuln/exploit/wireless/password/sniff/forensics).
- **KANUNI YA SIRI (mmiliki)**: katalogi ya umma (`/api/toolkit/catalog`)
  inarudisha id + jina la Kiswahili + kundi PEKEE. **Test inathibitisha:
  binaries 33+ na args za siri (nmap, msfconsole, hashcat, ...) HAZIPATIKANI
  kwenye katalogi wala matokeo.** Binaries/args ni `&'static` ndani ya Rust.
- **API**: `GET /api/toolkit/catalog` · `POST /api/toolkit/run` (kompyuta 1) ·
  `POST /api/toolkit/batch` (kompyuta NYYINGI kwa wakati mmoja) ·
  `GET /api/toolkit/runs` (audit 30 za mwisho). BILI gate: system/* =
  health_check, forensic/* = digital_forensic TZS 25,000, nyingine =
  malware_scan TZS 2,000; charge kwa mafanikio tu (batch: idadi ya mafanikio).
- **UI (🧰 TOOLS)**: makundi + grid ya kazi (CHAGUA) + RUN (1) + RUN BATCH
  (wakati mmoja, matokeo kila kompyuta) + terminal + audit ya kazi
  zilizotangulia.
- **Usalama wa utekelezaji**: kikomo cha kasi (`--max-rate=1000`) kwa zana za
  kasi kubwa; valid_target kwenye kila target (hakuna injection); root
  inaendesha kwa sudo tu kwenye server ya mmiliki.
- **Tests 6 mpya** (registry 34/8, catalog BILA vuja, rate caps, parse, audit
  DB, batch 3 kompyuta) — jumla **118/118**.

## H13 ✅ — RIPOTI RASMI (PDF/CSV) ZENYE BRAND — MALIZIO YA MRADI

- **📄 RIPOTI kwenye /ui**: chagua aina → Hakiki (preview bure) → ⬇ CSV (bure)
  au ⬇ **PDF (TZS 5,000)** — inashuka mara moja na brand ya **MTECH OS ·
  Mbilinyi Tech** ("Umiliki ni wako, leseni ni yako, faida ni yako").
- **Aina 4** (kila account inaona ZAKE PEKEE — access control, test inaithibitisha):
  🛡️ USALAMA (afya per-PC + Salama/Tahadhari/Hatari) · 🔬 FORENSICS (kesi +
  hash ya ushahidi IMEFUNGWA) · 💰 BILI (ledger: hela iliyoingizwa vs matumizi)
  · 🧰 KAZI (audit ya kazi — majina salama tu).
- **Backend (reports.rs)**: CSV escape sahihi (RFC 4180), **PDF 1.4 halisi
  bila dependencies** (kichwa brand, muhtasari, jedwali, footer — hakuna
  "hakuna data" kama rows tupu). BILI gate: PDF → `authorize("report")`
  (TZS 5,000; subscription inatosha) + `charge()` kwa mafanikio; 402 wakati
  hakuna salio; CSV bure.
- **API**: `GET /api/reports/preview/:kind/:account` ·
  `GET /api/reports/:kind/csv/:account` (Content-Disposition: mtech-<kind>-
  <account>.csv) · `GET /api/reports/:kind/pdf/:account` (application/pdf).
- **WHITE-LABEL FIX muhimu**: PDF ya deploy ya kale ilikuwa na brand ya ndani
  ("FUNDI DEPLOY" kwenye Title/Producer/kichwa) — imebadilishwa kuwa MTECH OS
  kila mahali; test mpya inazuiwa kurudi kwa brand ya kale.
- **Tests 4 mpya** (CSV escape, CSV brand, PDF brand bila FUNDI, collect +
  access control) — jumla **122/122**.

- **H13b (nondo):** doc-headers za report.rs zilikuwa na marejeo ya ndani ya
  awali ("FUNDI/fundi-mobile") — yamesafishwa; comments ni za white-label pia.
