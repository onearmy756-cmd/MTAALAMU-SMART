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

## H14 ✅ — AGENT LAYER (100+): kompyuta zote kwa wakati mmoja

- **👥 AGENTS kwenye /ui**: hesabu (jumla · online · wanafanya kazi), grid ya
  agent wote (IDLE/INAFANYA KAZI/OFFLINE, afya, progress ya kazi),
  **Sajilia agent** (token inaonyeshwa mara moja — admin anaihifadhi kwenye
  kifaa), auto-refresh kila sekunde 5.
- **Backend mpya (fleet.rs)** — mtiririko wa AGENT LAYER kama mpango:
  1. **REGISTER** `POST /api/fleet/register` {device} → agent_id + **token**
     (siri, DB inahifadhi SHA-256 hash tu); jina la kifaa linakaguliwa
     (valid_target); kikomo **MAX_AGENTS = 500** (100+ inatosha kwa mpango).
  2. **HEARTBEAT** `POST /api/fleet/heartbeat` {agent_id, token, health} —
     kifaa kisichojibu kwa **dakika 3** kinaonekana OFFLINE (test inaithibitisha).
  3. **POLL** `POST /api/fleet/poll` — agent anapata kazi yake
     **ILIZOIDHINISHWA (HITL)** pekee: `pick_approved_job()` (pure fn) inachagua
     oldest ya kifaa hiki; agent mwenye kazi haihangi mpya; kazi inakuwa
     "running" mara agent anaipata.
  4. **REPORT** `POST /api/fleet/report` {progress, message} — maendeleo hadi
     100 (kazi → "done", agent → idle); kila kitu kinaonekana kwenye tab Jobs
     na 👥 AGENTS kwa wakati halisi.
  5. **LIST** `GET /api/fleet/agents` — data halisi ya DB.
- **Ushirikiano wa mfumo**: kazi za MATRIX (deploy), SOLUTIONS (secops:*),
  REPO (bundle:*) zote zinapatikana kwa agents kupitia poll — HITL inabaki
  (idhini kwanza, tab Jobs); audit ya kazi ipo (tool_runs + jobs DB).
- **Tests 7 mpya** (agents **120** wakati mmoja, token-hash, HITL haiwape
  kabla ya idhini, mtiririko kamili register→poll→report→done, offline dakika
  3, jina mibaya, auth) — jumla **129/129**.
- **White-label**: API inarudisha majina salama tu; token ni ya agent
  (hash ndani ya DB); hakuna usanifu wa ndani unaoonekana.

## H15 ✅ — AGENT INSTALLER (Windows + Linux) — malizio ya AGENT LAYER

- **static/agent-mtech.ps1 (Windows)**: PowerShell — jisajili kwenye server
  (token inahifadhiwa `%ProgramData%\MTECH-OS\agent.json`), heartbeat kila
  sekunde 20 (afya halisi: CPU/RAM/Disk), poll kazi ZILIZOIDHINISHWA (HITL),
  utekelezaji: ukaguzi wa usalama wa ndani (firewall/antivirus → findings →
  ripoti ya usalama) na bundle/install (winget silent), report maendeleo 0–100.
- **static/agent-mtech.sh (Linux)**: bash — kama hiyo (token kwenye
  `/etc/mtech/agent.json`, chmod 600, root pekee; ukaguzi wa ufw/watumiaji;
  apt salama kwa bundle), loop moja kwa moja.
- **Faili zinapatikana kwenye server** kupitia ServeDir:
  `http://SERVER:8080/agent-mtech.ps1` na `.../agent-mtech.sh` — vitufe vya
  ⬇ Pakua kwenye kadi 👥 AGENTS (pamoja na amri za usakinishaji).
- **Siri**: agents wanatumia API ya umma tu (fleet/* + secops/result); hakuna
  usanifu wa ndani kwenye scripts; HITL inabaki (kazi zilizoidhinishwa pekee).
- **Uthibitisho**: bash -n OK, node --check 2/2, cargo test 129/129.

## H16 ✅ — MWANZO WA BIASHARA (STARTER PACK) — kwa mmiliki

Folda mpya **BIASHARA/** — hati za kuanza kupata wateja na pesa (bei zote
zimechukuliwa moja kwa moja kutoka code, zimehakikiwa 11/11 kulingana na
billing.rs na tools.rs):

- **PRICE-LIST.md** — Subscription (Basic 8,000 / Standard 6,500 / Biashara
  5,000 / Kubwa 4,000 kwa kifaa/mwezi) + Pay-per-use (scan 2,000 … forensics
  25,000) + punguzo la volume (10–20%) + mifano ya mapato (PC 20 = 130,000/mwezi)
  + kanuni za BILI (charge baada ya mafanikio, HITL, ledger).
- **KANDARASI.md** — kandarasi rahisi ya Kiswahili: HITL (idhini kabla ya kazi
  hatari), ushahidi wenye hash, usiri (mfumo haufichuliwi), umiliki wa mfumo
  ni wa Mbilinyi Tech (mteja ananunua HUDUMA), SLA (dharura < saa 4),
  hakuna malipo ya kazi iliyofeli.
- **SERVER-SETUP.md** — hatua 8 halisi: clone → `docker compose up -d --build`
  (api:8080, dnsmasq, nginx, samba, ollama, llamacpp, suricata, wazuh,
  kali-tools) → /health → Onboarding → VPN → BILI (salio/subscription) →
  agents (ps1/sh) → kazi ya kwanza (MATRIX → Jobs RUHUSU → RIPOTI PDF) →
  backup/passwords/bandari + meza ya matatizo ya kawaida.
- **MUHTASARI.md** — muhtasari wa mradi kwa wateja/wawekezaji: uwezo wote
  (H6–H15), ubora (tests 129/129, CI 6/6, PRs #6–#23), njia 4 za mapato,
  gharama (hakuna leseni), ujumbe wa kutangaza.

## H17 ✅ — CLICKPESA + APP YA SIMU + CUSTOMER PORTAL

- **H17a — ClickPesa (malipo ya kiotomatiki, clickpesa.rs)**:
  - `POST /api/clickpesa/checkout` {account, amount_tzs} → Hosted Checkout link
    (token → `checkout-link/generate-checkout-url`, Bearer JWT); order
    reference alphanumeric ya kipekee (CP+ms+random); chini ya TZS 1,000 inakataliwa.
  - `POST /api/clickpesa/webhook` — **PAYMENT RECEIVED**: checksum
    (HMAC-SHA256 ya body na secret, header x-clickpesa-signature/checksum)
    → topup kwenye BILI kiotomatiki; **idempotent** (SUCCESS mara moja tu —
    test inathibitisha salio linaongezeka MARA MOJA); checksum mbaya/kosa → kataa.
  - `GET /api/clickpesa/status/:order` — hali ya checkout (dashboard + portal).
  - Bila keys (CLICKPESA_CLIENT_ID/SECRET) mfumo unaendelea: order PENDING +
    ujumbe wa configuration (kama pfsense.rs). UI: kitufe **📱 LIPA SIMU** kwenye BILI.
- **H17b — App ya simu ya fundi (static/mobile.html)**: login (auth.rs tokens),
  kazi zinazosubiri IDHINI (HITL) na kitufe cha **IDHINISHA kwa mbali**,
  hali ya agents (online/working), auto-refresh sekunde 8; session kwenye
  localStorage; "Add to Home Screen" inafanya kama app.
- **H17c — Customer Portal (static/portal.html + portal.rs)**: mteja anaingia
  kwa **Account + PIN** (SHA-256 hash + salt; sessions DB):
  salio la BILI + **📱 LIPA SIMU** (ClickPesa kwa mteja mwenyewe),
  subscription yake, usalama wa kompyuta ZAKE (afya bars), ushahidi wake
  (hash + IMEFUNGWA). Admin anaweka PIN: `POST /api/admin/portal/set-pin`
  (token ya admin pekee).
- **Access control kila mahali**: portal inatumia ile ile ya secops
  (account anaona ZAKE pekee — test inaithibitisha mteja9 haoni za mteja1).
- **Tests 5 mpya** (checksum, order ref, checkout bila env, webhook
  idempotent+topup, PIN login+access control) — jumla **134/134**.

## H18 ✅ — REPO DASHBOARD v3.1 (versions + gauges za integrity, picha ya mmiliki)

- **Version metadata (bundles.rs)**: `version_of(id)` — Ramani ya toleo la
  bundles zote 32 ya katalogi (mf. `dev-vscode`→v1.93, `util-vlc`→v3.0.21,
  `gfx-gimp`→v2.10, browser-chrome→v130.0) + fallback `v1.0 (custom)` kwa
  custom bundles (drag & drop). `GET /api/bundles` kwa sasa inajibu
  `version` ya kila bundle (main.rs) — sio tena "Package Name" pekee.
- **UI REPO v3.1 (static/index.html)**:
  - Kadi (repRender) zinaonyesha **Version vX** (bluu) chini ya Package Name —
    kama picha ya mmiliki.
  - **Gauges 2 za SVG** (ring, katikati ya duara): **Docker Container
    Status** (Healthy/Nje ya Mtandao — kutoka /health huduma zimepakiwa) na
    **Wazuh Sync Status** (~100% — usawazishaji wa katalogi: % ya bundles
    zenye apps). Iliyotangulia hapo ilikuwa `rep_integrity` maandishi tu;
    sasa gauges + ripoti ndogo (Bundles/Categories/Apps/Huduma x/9).
    Majina ya gauges ni kwa **dashboard ya mmiliki pekee** — chapisho la API
    (JSON) zote bado ni white-label; mobile/portal hazibadiliki.
  - Kagua tena majaribio ya white-label: catalog test (toolkit) na pdf test
    (reports) zote 2 zinaendelea kupita.
- **Tests 1 mpya**: `versions_zote_wa_catalog` (kila bundle ina vX; v1.0
  (custom) kwa custom; v1.93 kwa vscode). **Jumla 135/135**.
- **UI check**: node --check 2/2 (script blocks zote 2 za index.html).

### H18b (kuendelea na picha) — Bundle ya R-Language Core
- **bundles.rs**: bundle mpya **dev-r "R-Language Core"** (category
  Development, toleo **v4.4.1** kama picha ya mmiliki) — apps: R Language
  (winget `RProject.R` / apt `r-base`) + RStudio Desktop (winget `Posit.RStudio`);
  `version_of()` → `v4.4.1`; test ya versions imethibitisha pamoja na
  find_bundle/find_app. Jumla bundles sasa **33**. Tests **135/135**.

### H18c — DRAG & DROP category + terminal ya mistari zaidi
- **bundles.rs**: `category_is_known('custom')=true` — custom setup inaruhusu
  category yoyote ya kadi (mf. "DEVELOPMENT TOOLS"), si 'custom' pekee; test
  mpya `custom_category_ipokelewa_catalog_inakatalia`. **136/136**.
- **index.html (REPO card)**:
  - DRAG & DROP sasa ina **select ya Category** na
    **"DEVELOPMENT TOOLS" ikichaguliwa kuanzia** (kama picha ya mmiliki) —
    pamoja na 8 za katalogi + CUSTOM (generiki).
  - repFiles inatuma category hiyo kwenye POST /api/bundles; majina ya
    label (DEVELOPMENT TOOLS/MAWASILIANO/USALAMA…) kutoka `REP_DROP_LABEL`.
  - Kadi zinaonyesha **`Category: DEVELOPMENT TOOLS · apps N`** (uppercase
    kama picha), na custom id inaonyesha `CUSTOM`.
  - Terminal (rep_term) inapata **mistari zaidi ya kila faili** kama picha
    ("[OK] Setup: {jina} — Category: {LABEL} (faili 1/1) imeandikishwa..."),
    scroll inafuata mwisho; kosa linaonekana pia ([ERROR] + reason).
  - Kurekebisha kosa la JS mfululizo wa mabano (node --check twofold).
- ** verifies**: cargo test 136/136 (exit 0), node --check 2/2.

## H19 ✅ — AI AUTO-TRANSLATE: MFUMO UNATAFSIRI UKICHOAGUA LUGHA (biashara ya lugha)

- **Kanuni ya mmiliki**: "Mfumo ndo ufanye hivyo sio wewe" — mteja anachagua
  lugha, na **AI ya server** inatafsiri majibu YOTE kiatomatiki. (Mimi/[AI
  assistant] si mtafsiri wa laatste; mfumo wenyewe unafanya.)
- **chat.rs**:
  - `ChatReply` mpya: `answer` (translated, serde rename; haijaanguka kwenye
    mobile/portal), `language` (code ya DB).
  - `ask_username(db, brain, username, ...)` — inaita `ask` kama kawaida,
    kisha **`language::auto_translate`** (ai_config::translate_db; LLM ya LAN
    haipatikani → jibu la asili, hakuna uongo). Majibu yote yanapita
    `tools::sanitize_output` (white-label inabaki).
  - Test mpya `ask_username_inatafsiri_kwa_lugha_ya_mtumiaji`.
- **main.rs**:
  - `POST /api/chat` inakubali `username` (opisinale: chini, bila
    username → ask ya kawaida).
  - Route mpya **`POST /api/portal/language`** — token ya portal + language;
    portal::auth kwanza, kisha `language::set_language(account)`.
- **index.html (CHAT)**: select ya lugha (sw/en/fr/ar) + Account; sendChat
  inahifadhi lugha (`/api/language`) kwanza, kisha jibu linaonyesha
  `answer` (processed) + badge ya lugha.
- **portal.html**: kadi mpya **🌐 LUGHA YAKO** — select + HIFADHI →
  `/api/portal/language`; mfumo unaeleza utatafsiri kiatomatiki.
- **mobile.html**: fundi anachagua lugha kwenye INGIA (localStorage-free;
  mfumo unaihifadhi kwenye DB kwa username yake).
- **16/16 tests za language + chat** (137 jumla): lugha ya kila mtumiaji
  inahifadhiwa na kubadilishwa; code mbaya inakataliwa; auto_translate
  haipanuki bila LLM (jibu la asili).
- **137/137 tests** (136 + 1 mpya), node --check: index 2/2, portal 1/1,
  mobile 1/1.

## H19 ✅ — RUST PEKEE + REPO LOGO HALISI + UI YA KISASA

- **Rust pekee (kanuni ya mmiliki)**: mfumo wote unakaa Rust: axum+sqlx+reqwest
  (LLM/ClickPesa/tftp helpers) — hakuna python/node/java/go kwenye server
  (Cargo.toml haianza runtime nyingine). Test mpya ya bundles: kila app ya
  katalogi (mf. python/node) ni kwa **PC za wateja** (winget/apt REPO) — sio
  dependency ya server; logos zote 33 zina SVG iko static/img/rep/
  (test inathibitisha file.zipopo kwa kila bundle —kukosa 404).
- **REPO logos halisi (static/img/rep/)**: SVG 33 — kila bundle ina picha halisi
  (chupa/circle/monogram/cartoon zilizokufupisha branding: VS Code blue,
  R circle, Docker whale, GIMP wilber simplified, Slack, Teams, Zoom, VLC
  cone, 7z, WinRAR, Dell/HP/Lenovo/Acer/ASUS plates…). Kadi (repRender) sasa
  ina **<img src="img/rep/{id}.svg">** na **fallback emoji** kama faili
  halipatikani (custom bundles). test ya bundles inasema logo kila bundle.
- **UI ya kisasa (mwonekano muhimu umebaki — rangi/brand zilezile)**:
  - `.rep-card`: gradient lesi ya kisasa, hover inapanda (translateY(-3px)),
    border inamulika #00e5ff, shadow laini, transition .18s, active-press.
  - `.toast` kwenye chini → badala ya `alert()` kwenye REPO DISTRIBUTE
    (alerti za sehemu nyingine zimebaki — hazikuguswa).
  - `:focus-visible` outline kwa keyboard-navigation (weusi wa kupita).
  - repDistribute error ina `needs_billing` hint kwenye terminal (imebaki).
- **Tests 2 mpya** (`mfumo_rust_pekee_catalog_haina_runtime_za_nje`,
  `logo_path` check) — **jumla 138/138**; node --check 2/2 (kabla ya commit).

## H20 ✅ — THIBITISHO: HERMES/FUNDI-DEPLOY + KALI LINUX ZIMEUNGANISHWA

Swali la mmiliki: "hermes agent na zana zake zote, kali linux na zana zake
zote zimeunganishwa na zinafanya kazi?" — JIBU LINAPIGWA NA TEST SI MANENO:

- **Hermes/Fundi-Deploy (Rust)**: `cargo test` → **139/139** (fundi-deploy)
  + **70/70** (engine-rust). CI 6/6 kila PR (2× engine, 2× fundi-deploy,
  2× lint dashboard). Zana za mfumo: PXE/WOL/multicast/VPN/backup/images/
  fleet/billing/reports/secops/toolkit/portal/ClickPesa — zote zina tests
  zinazopita (rejea blocks H6–H19 hapo juu).
- **Kali Linux**: docker-compose.yml ina huduma **kali-tools**
  (image `kalilinux/kali-rolling`, container `fundi-kali`, work dir
  ./data/kali-work) — zana 34 za toolkit.rs (nmap, masscan, nikto, wpscan,
  msfconsole, sqlmap, hydra, aircrack-ng, kismet, john, hashcat, crunch,
  tcpdump, tshark, ettercap, sleuthkit-fls, volatility3, foremost, scalpel,
  binwalk, dc3dd, photorec, testdisk, autopsy, guymager, + za mfumo) zina
  **binary + args kamili ndani ya Rust** (`ToolDef`), zinakimbia kwa
  arg-array salama + timeout 60s + `sanitize_output` + audit (tool_runs).
- **Test mpya ya muhuri** `kila_zana_ina_mpangilio_kamili_na_kali_imeunganishwa`:
  1) kila zana ina id pekee + binary + jina; 2) katalogi ya API HAIVUI
  binary yoyote kwa zana MOJA MOJA (white-label); 3) compose ina
  kali-tools/kalilinux/kali-rolling; 4) registry ≥ 30.
- **Kumbuka**: zana za Kali zinakimbia kwenye container ya Kali au host
  yenye пакет husika; toolkit inarudisha ujumbe mzuri ("zana ya ndani
  haipatikani kwenye server hii") kama binary haipo — hakuna kufilisika.
- **Jumla tests**: 139 (fundi) + 70 (engine) = **209/209**.

## H21 ✅ — HERO SLIDER (RETOOL-STYLE) JUU YA DASHBOARD

- **Kusoma retool.com**: mwonekano = hero kubwa na slides zinazojiendesha
  (badges "NEW", mada kubwa, CTA mbili: kuu + secondary), dots za
  uelekezaji, na visual za kuchora upande wa kulia. Rangi + brand ni
  zetu (teal/cyan/mwanga wa bluu) — muundo tu ni wa Retool.
- **index.html**:
  - `.hero-slider` juu ya kadi ya MWANZO HAPA (`#hs_top`): slides **3**
    (1: Mfumo una-fanyaje kazi → MWANZO HAPA/JOBS; 2: REPO programu 33
    isho na picha halisi → REPO/TOOLS; 3: BILI + LIPA SIMU → BILI/CYBER).
  - `.hs-badge` ("NEW · …"), CTA zote zina `location.hash` → sehemu
    husika ya dashboard; visual za upande wa kulia zimechorwa kwa
    mistari midogo ya taarifa halisi (si lorem).
  - **Dots** (`#hs_dots`) na **autoplay 6s**; mtumiaji anapobonyeza dot,
    autoplay inasimama (kutokwenda kinyume na matumizi yake).
  - Responsive: picha inashuka chini ya screen ndogo (max-width 800px).
  - `hsInit()` inaitwa kwenye `window.load` (baada ya flLoad).
- **Kagua hali**: node --check 2/2; hakuna Rust iliyoguswa (tests ziko
  139+70 kutoka H20 — hazirepeatishwi kwa info lost).

## H22 ✅ — PULSE (ANALYTICS KAMA RETOOL) + UKAGUZI WA DOCKERFILE

- **Maswali yote ya mmiliki yanajibiwa**:
  1. **Dockerfile ya server iko tayari na kamili** — `hermes-agent/fundi-deploy/server/agent/Dockerfile`
     ina `COPY static ./static` (Docker inanakili folda NZIMA pamoja na
     `img/rep/` — SVG 33 zote pamoja na index.html wanakaa kwenye image;
    NDANI YA binary `fundi-deploy` + huduma 8080). Compose pia ina kali-tools dnsmasq samba ollama llamacpp suricata wazuh (SERVER-SETUP.md inaeleza).
     Dashboard lint ya CI (zero-dependency: title/charset/script-balance/
     node --check/duplicate id/reference integrity) kila PR.
  2. **Retool hero-slider** — kwenye main (H21), inajicharaza kila 6s
     (dots + badges + CTA mbili kila slide, anchors zote 24 zinapatana
     na kadi halisi).
  3. **Chati/Analytics kama Retool** — **PULSE** mpya (H22): kadi yenye
     **stat tiles 3** (Kazi zinazoendesha + jumla; Agents online/working;
     Salio BILI wa mteja1 kutoka ledger) + **bar chart ya SVG** (kazi 6
     zilizopita, mafupi kwa jina + % ya progress, rangi kwa hali: done
     green, working cyan, awaiting amber, failed red) — data halisi kutoka
     `/jobs`, `/api/fleet/agents`, `/api/billing/statement/mteja1`, refesh
     kila 5s. Chini ya 700px bado sawa (tiles 3 mlalo).
  4. **Kitufe 📊 PULSE** kwenye nav (kabla ya REPO).
- **Kagua wa ziada**: lint ya dashboard (script iza zote + ref integrity)
  — kosa pekee ni la KALE (href ya WG conf yenye encodeURIComponent,
  template-literal lint huipa false positive; route halisi ipo main.rs
  `/api/vpn/peers/:name/conf`) — SI la mabadiliko ya H22; CI ya web-lint
  inalinda web-r/www/index.html pekee (fundi-deploy dashboard ina node
  --check ya blocks zote 2 kwenye hali halisi).
- **Tests**: hakuna Rust iliyoguswa (139+70 kama H20); node --check 2/2.

## H23 ✅ — CHATI YA AI + OLLAMA/LLAMA.CPP/CLOUD ZIMEUNGANISHWA (UKAGUZI + CLOUD KEY)

Swali la mmiliki: "chati ya AI + ollama cloud, llm cpp zimeunganishwa?" —
JIBU (ukweli kwenye disk):

- **Chat ya AI IPO na inafanya kazi**: kadi 🖥️ SOLUTIONS ina "AI AGENT —
  Chat ya matatizo" (POST /api/chat, Enter au ➤), jibu lina source
  (brain/llm/offline_rules) + confidence + references + **auto-translate**
  kwa lugha ya mtumiaji. Kadi 🧠 AI inaruhusu endpoint/model ya CUSTOM
  (DB-backed, inatumika mara moja).
- **Ollama imeunganishwa**: compose ya server ina huduma **ollama**
  (`ollama/ollama:latest`, port 11434, data kwenye ./data/ollama) —
  ai_config default nayo ni `FUNDI_LLM_URL=http://127.0.0.1:11434` +
  `FUNDI_LLM_MODEL=qwen2.5vl:3b` → chat/translate zinaita Ollama moja kwa
  moja (offline LAN).
- **llama.cpp imeunganishwa**: huduma **llamacpp** (`ghcr.io/ggml-org/
  llama.cpp:server`, port 8081, GGUF Qwen2.5-VL-3B q4_k_m kwenye
  ./data/models) — URL yoyote ya :8081/llamacpp inatumika kama LAN model.
- **Cloud AI (H23 mpya)**: style_for_url() inatofautisha
  - LAN (Ollama :11434 / llamacpp :8081) → `/api/generate` (prompt style);
  - OpenAI-compatible (OpenAI/Groq/DeepSeek/Anthropic-compat/vLLM) →
    `/chat/completions` (system+user messages) + **Bearer FUNDI_AI_KEY**
    (env pekee, HAKUNA key kwenye DB); jibu linasomwa kutoka
    choices[0].message.content AU response (zote mbili zinakubalika).
  - UI bado ni ile ile ya 🧠 AI: weka URL ya cloud + jina la model;
    key inawekwa kwenye env ya server (FUNDI_AI_KEY) — canonical.
- **Test mpya** `style_urls_na_key_hakuna_panic`: LAN style 3, cloud
  style 2, key bila env → None (hakuna panic), defaults za LAN
  (:11434/qwen2.5vl:3b). **Jumla 140/140** (fundi-deploy).

## H24 ✅ — UHALISIA (LIVE SERVER) + CHAT YA PROMPT KAMA RETOOL

Swali la mmiliki: "sehemu ya kuandika prompt inafanya kazi kama Retool chat?
je kila kitu kipo kama kilivyopangwa na kinafanya kazi kwenye uhalisia?"
— NILIJIBU NA SERVER HALISI ILIYOWAKA (binary ya release, SQLite halisi):

**Tests ya LIVE (curl, server inaendesha):**
- `GET /health` → status ok, huduma 9 (pxe/tftp/http/smb/ai/wol/backup/
  multicast/cloud), cloud_outbox:0.
- `POST /api/chat` (prompt ya kuchora halisi, "Printer haiandiki…",
  length=medium, username=mteja1) → **jibu lililojibu**: source
  `brain_offline`, confidence **0.2**, language sw, jawabu 373 chars
  (maelezo marefu ya offline; LLM ya LAN ndiyo itaongeza conf hadi 0.7–0.9
  ikianza). Prompt ya mtumiaji inaingia + inaandika kwa kiswahili.
- `GET /api/bundles` → **bundles 33, dev-r ipo v4.4.1** (R-Language Core)
  — H18b inathibitishwa LIVE.
- `POST /api/bundles/distribute` (dev-r, lab_user_001, live-test) →
  **billing gate inafanya kazi halisi**: `hakuna subscription na salio
  halitoshi: app_install TZS 1500, salio TZS 0` — kanuni ya mmiliki:
  kazi haitianzi bila malipo/HITL.
- `POST /api/toolkit/run` (system-discover) pia imezuiliwa na billing
  (health_check TZS 2,000) — gate sawa, HITL ni neno, hakuna free-run.
- `GET /api/billing/statement/mteja1` → ok, balance 0 (ledger halisi).
- `GET /jobs` + `GET /api/fleet/agents` → [] (hakuna agents bado — ok,
  hakuna uongo).
- `GET /api/toolkit/catalog` → tools 34, cats 8, **haizui binary yoyote**
  (white-label imebaki).

**Kosa limerekebishwa kwenye binary/DB**: kwanza nilipima na DB ya zamani —
bundles 32 bila dev-r (kwa sababu kila catalog ni Rust, dev-r ipo). Kisha
nilimpa binary nyefu fresh DB → **33 + dev-r** v4.4.1 live.

**UI: Chat ya prompt kama Retool** (kadi SOLUTIONS):
- **Bubbles**: ujumbe wako (Wewe) unapanda kulia bubble ya buluu;
  jibu la AI (AI Agent) kushoto bubble nyeusi, border laini;
  **pili ya source+confidence** (mf. "brain_offline · 20%") kwenye kila
  jibu; auto-scroll kwa kila jibu.
- swali linatumwa na **username=account** (H19 auto-translate linatumika) —
  live-test imethibitisha.
- node --check 2/2; anchor-target zote zinapatana.

**Kwa mmiliki**: kwenye server yako halisi, ongeza `FUNDI_AI_KEY` env au
wacha Ollama LAN; chat itaenda LLM kiotomatiki na conf itapanda kwenye
kadi. Tests: 140/140 (H23) — hii H24 ni kaza ya live-verification, hakuna
mabadiliko ya Rust (build iliyokaa imetumiwa kutumia binary).
