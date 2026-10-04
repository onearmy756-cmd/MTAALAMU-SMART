# MTAALAMU SMART — Unified Solver 🖥️🇹🇿

**Mfumo mmoja unaotatua matatizo YOTE ya computer** — umeunganisha
**hermes-agent** (CLI, tools, skills, UI) na **MTAALAMU SMART** (catalog, HITL,
kernel vision) kuwa solver mmoja halisi. Mtumiaji anaandika tatizo — mfumo
unauona (probe), unapanga, unauliza (HITL fomu ya chaguo), unatekeleza amri
**HALISI za OS husika**, unathibitisha, na unahifadhi kwenye **Kitabu
Kidigitali** (inajifunza). Kiswahili default; English inabadilishwa au
inagundulika kiotomatiki.

## Uwezo (kila kitu ulichoorodhesha)
| Uwezo | Hali |
|---|---|
| Partition (tengeneza/orodhesha), disk cleanup, defrag/TRIM, SMART health | ✅ catalog 31 ops, amri halisi kwa OS zote |
| Driver update/install + orodha | ✅ Windows Update + apt firmware |
| Network: diagnose, DNS reset, WiFi reconnect, stack reset | ✅ + **online-gate**: bila net inasema "washa mtandao" |
| Install/uninstall/update apps (apt/winget/brew halisi) | ✅ |
| Users: tengeneza account, badilisha password, orodha | ✅ |
| Task Manager / Device Manager / Disk Manager / Startup apps | ✅ kwa OS zote |
| Microsoft Office: diagnose, repair (Quick/Online), leseni | ✅ |
| Antivirus HALISI: Defender scan (Win), ClamAV (Linux/mac) + status | ✅ |
| Hardware scan + battery report + thermal | ✅ inaonyesha + inamjulisha mteja (hardware = ripoti + maelekezo) |
| Files: tengeneza folder, futa (kwa kibali) — kama binadamu | ✅ |
| **Shortcuts zote** Win (F1/F2/F12, Win+X, Win+R…), macOS (Cmd+Space, Cmd+Opt+Esc…), Ubuntu/Linux (Ctrl+Alt+T, F12…) | ✅ 17 shortcuts zinazotekeleza ops halisi |
| Command prompts zote: cmd/PowerShell/wt, xfce4/gnome-terminal/konsole, Terminal/iTerm | ✅ zinafunguliwa na shortcuts |
| **HITL fomu**: maswali + options (mteja anachagua → mfumo unaendelea mpaka mwisho) | ✅ |
| Kugundua matatizo mapema + kumjulisha mteja | ✅ scan + probe + knowledge |
| **Kitabu Kidigitali** (ripoti + kujifunza; tatizo kama hili linaliwa kwanza) | ✅ knowledge.jsonl |
| Offline/online: offline-first; kinachohitaji net → inaambia mteja | ✅ ping gate halisi |
| Models: **TinyLlama** (offline default: kufikiri + kutafsiri), **Qwen 2.5 VL 3B** (vision/kazi), **API yoyote** (Ollama cloud/OpenAI-compatible) — mteja anachagua | ✅ `mtaalamu model` |
| Hakuna hallucination: amri zinatoka catalog; LLM inaueleza tu; mada ni computer-solutions TU | ✅ scope guard |
| **Billing**: email halisi + INDIVIDUAL (1 seat) / CORPORATION (10) / ORGANIZATION (N) × BASIC/BRONZE/GOLD/PLATINUM/DIAMOND; kila zana ina credits (metering) | ✅ license HMAC-signed, haitengenezwi upya |
| **ClickPesa PSP** — MALIPO KABLA YA ZANA (pay-before-use): USSD push (M-Pesa, Tigo Pesa, Airtel Money, HaloPesa) → wallet inaongezeka TU kwa SUCCESS iliyothibitishwa | ✅ `mtaalamu/clickpesa.py` + wallet |
| **ADMIN** — FULL SYSTEM dashboard (OS nzima), zana ZOTE BURE, kubadilisha bei ya zana yoyote (0 = BURE), kuweka TIER kwa huduma yoyote | ✅ `mtaalamu/admin.py` + UI panel |
| **Setup binaries** (download & use): Windows/Linux/macOS onefile | ✅ `setup_builder.py` (PyInstaller) |
| IPC (unix socket) + **R bridge** (uchambuzi halisi wa usage kwa Rscript) | ✅ |
| Storage kidogo: JSONL ndogo + catalog ndogo; hakuna DB kubwa | ✅ |
| UI ya hermes: chat + HITL fomu + catalog + leseni + charts | ✅ `web-html/mtaalamu-unified.html` (offline single-file) |

## Sakinisha / Tumia

### A) Binafsi (computer yako — OS yoyote)
```bash
# 1) Pata repo, hakuna dependencies nzito (stdlib tu; hiari: Pillow, PySide6)
cd hermes-agent
python3 -m mtaalamu register jina@gmail.com INDIVIDUAL GOLD
python3 -m mtaalamu doctor            # ukaguzi wa mashine
python3 -m mtaalamu serve             # API http://127.0.0.1:8795 (IPC: /tmp/mtaalamu.sock)
# UI: fungua web-html/mtaalamu-unified.html (browser) — chat, fomu, catalog, leseni
```

### B) CLI kwa kila kitu
```bash
mtaalamu ask "kompyuta inaenda polepole"      # plan + maswali (HITL)
mtaalamu solve "cleanup" --answers '{"op":"disk.cleanup","confirm":"ndio"}' --yes
mtaalamu scan | catalog | shortcuts | usage | license | doctor
mtaalamu shortcuts run linux "Ctrl+Alt+T"     # shortcut halisi
mtaalamu model local tinyllama                # au: api https://... OPENAI_KEY
mtaalamu lang en                              # sw/en (default sw)
```

### C) Malipo: ClickPesa (pay-before-use — kulinda kutoibiwa)

**Jinsi inavyofanya kazi (HALISI):**
```
1. Mteja anataka kutumia zana (mf. disk.cleanup — TZS 12,000)
2. Mfumo unakata kama wallet haitoshi →  stage:"payment"
3. mtaalamu pay 12000 --phone 255712345678     (au POST /api/pay)
   → ClickPesa INATUMA USSD PROMPT kwenye simu ya mteja (PIN)
4. Mteja anaingiza PIN (M-Pesa / Tigo Pesa / Airtel Money / HaloPesa)
5. Mfumo unapoll status mpaka SUCCESS → wallet inaongezwa (audit)
6. SASA zana inatekelezwa — salio linakatwa KABLA ya amri zote
```

**Ulinzi dhidi ya wizi:**
- Wallet inaongezeka TU kwa malipo yaliyothibitishwa na ClickPesa:
  webhook yenye HMAC sahihi (`X-ClickPesa-Signature`) AMA poll ya status SUCCESS.
- Webhook iliyoharibiwa/tampered → 401, hakuna credit (imetesteriwa).
- FAILED / TIMEOUT → HAKUNA topup, zana haifunguki.
- Salio linakatwa KABLA ya execute — hakuna zana isiyolipwa.

**Keys (Settings → Environment):**
| Key | Kazi |
|---|---|
| `CLICKPESA_CLIENT_ID` | (lazima) Client ID ya app yako ClickPesa |
| `CLICKPESA_API_KEY` | (lazima) API key |
| `CLICKPESA_CHECKSUM_KEY` | (hiari) inawasha checksum + webhook HMAC verify |
| `CLICKPESA_SANDBOX=1` | (hiari) api-sandbox.clickpesa.com kwa majaribio |

**Amri:**
```bash
mtaalamu wallet                                  # salio + hali ya gateway
mtaalamu pay 15000 --phone 255712345678          # lipa kiasi (USSD push + poll)
mtaalamu pay --op disk.cleanup --phone 2557...   # lipa bei ya zana mahsusi
# API: POST /api/pay {"op_id":"disk.cleanup","phone":"255..."}
#      GET  /api/pay/status?ref=MST...  |  GET /api/wallet
#      POST /api/webhook/clickpesa  (ClickPesa webhook → HMAC verify → credit)
```

### D) ADMIN — FULL SYSTEM (mmiliki wa mfumo)
```bash
mtaalamu admin unlock                # owner-mode (kompyuta yako)
mtaalamu admin unlock --key XXX      # kama MTECH_ADMIN_KEY imeweka Environment
mtaalamu admin dashboard             # OS nzima + leseni + wallet + catalog (bei/tier)
mtaalamu admin price disk.cleanup 20000   # badilisha bei (0 = BURE kwa wote)
mtaalamu admin price av.scan 0            # huduma kuwa BURE
mtaalamu admin tier disk.partition.create GOLD   # huduma kwa GOLD+ tu
mtaalamu admin lock                  # zima admin
```
- **Zana BURE kwa admin** — hakuna wallet/malipo (audit inarekodi `ADMIN`).
- **Bei** (prices.json) inaweka nguvu juu ya defaults na tier discounts; `0` = BURE.
- **Tier za huduma** (tool_tiers.json): mf. `GOLD` → wateja wa GOLD/PLATINUM/DIAMOND tu.
- UI: panel ya **⚑ FULL SYSTEM** kwenye `web-html/mtaalamu-unified.html` (unlock, dashboard, bei, tier).
- Usalama: kama `MTECH_ADMIN_KEY` ipo Environment → unlock inahitaji key; kama haipo → owner-mode (mmiliki wa kompyuta).

### E) Setup binaries (download & use)
```bash
python3 setup_builder.py            # inajenga binary ya OS hii (PyInstaller onefile)
# dist/mtaalamu-linux | MTAALAMU-Setup.exe | MTAALAMU-macos
```

## Usalama na Ukweli
- **HITL**: kila op yenye risk HIGH/MEDIUM inauliza; mteja akikata — hairudi.
- **Malipo kabla ya zana**: `wallet_spend` inatoa ValueError kama salio halitoshi — `execute()` hairudi mbele; audit (`payments.jsonl`) ina kila malipo.
- **Hakuna hallucination ya amri**: catalog ndiyo chanzo; model inachagua/kueleza.
- **Scope**: maswali yasiyo ya computer yanakataliwa kwa ujumbe wa lugha ya mteja.
- **Leseni**: HMAC-signed; key ya kuibiwa haitumiki kwenye data tofauti; tamper → imeshikwa.
- **Corporation = seats 10 cap**; Organization = N (mteja anachagua).
- Modeli **isiyopo offline** → inarudi TinyLlama; kama hata hiyo haipo → catalog
  bado inaendesha (LLM ni msaada, si msingi).

## Muungano na hermes-agent
- Server ya mtaalamu inaunganishwa na gateway ya hermes (HTTP 8795 + IPC socket).
- UI ya hermes (web/) au web-html/ inaonyesha chat + fomu za HITL + catalog + leseni.
- Skills zote za MTAALAMU SMART (146) + tools za hermes zinapatikana kupitia
  catalog na agent loop ya mtech-os (tazama `mtech-os/`).

## Nyaraka za kina
- Catalog kamili: `mtaalamu/catalog.py` (kila op + amri halisi + risk + credits)
- Engine: `mtaalamu/unified.py` (detect→plan→gate→execute→verify→learn)
- Billing: `mtaalamu/billing.py` · Models: `mtaalamu/models.py` · Shortcuts: `mtaalamu/shortcuts.py`
