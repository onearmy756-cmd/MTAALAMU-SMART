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

### C) Setup binaries (download & use)
```bash
python3 setup_builder.py            # inajenga binary ya OS hii (PyInstaller onefile)
# dist/mtaalamu-linux | MTAALAMU-Setup.exe | MTAALAMU-macos
```

## Usalama na Ukweli
- **HITL**: kila op yenye risk HIGH/MEDIUM inauliza; mteja akikata — hairudi.
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
