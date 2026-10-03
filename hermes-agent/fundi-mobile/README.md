# 🩺 FUNDI MOBILE — Daktari wa Simu

Bidhaa ya ecosystem ya MTAALAMU: **agentic mobile repair** — Android · iPhone · **simu za button**.
Rust backend, data-driven (`data/mobile/*.json`), Kiswahili fasaha, **HITL daima**.

## ⚠️ SHERIA (KANUNI ISIZOKIUKA)

> Kufuta password ya simu ya mtu mwingine **bila ruhusa = kosa la jinai**
> (Sheria ya Makosa ya Jinai Tanzania, **Kifungu 267** — faini TZS 500K+ au jela miaka 2+).

- Kila kazi inayoharibu inahitaji **consent iliyosajiliwa** kwenye `data/mobile/consents.json`
- **Agent haitaendelea** bila consent — inasimamisha job kwenye `awaiting_consent` (HITL)
- Fomu ya kusaini: `fundi-mobile consent form` (print → sahihi → piga picha)

## Uwezo

| Kundi | Zana |
|-------|------|
| Android | Reset ADB (data inabaki) · Recovery wipe · Flash (Odin/Mi Flash/SP Flash/QFIL) · FRP · Update · Backup · Diagnostics |
| iPhone | iTunes/Finder restore · Recovery · DFU · iCloud (kisheria) |
| **Button** | **Master reset codes** (Nokia `*#7370#`, Itel, Tecno, Alcatel, Symphony, generic MTK/SPD) · hard-reset combos · flash (CM2/Miracle/SPD) |
| Agentic | Agents 10 · PIITVD auto-work · kitabu kidigitali · learning loop · dashboard |

## Install

```bash
cd fundi-mobile
cargo build --release
# Tools za nje (Android): choco install adb fastboot  (au platform-tools za Google)
# Drivers: Samsung/Xiaomi/Huawei/MTK USB drivers kwa brand unazofanya
./target/release/fundi-mobile check
```

## Amri

```bash
fundi-mobile catalog                     # huduma zote + bei (TZS)
fundi-mobile brands samsung              # recovery combo + flash tool + firmware site
fundi-mobile button nokia                # master reset codes za button
fundi-mobile button-codes                # codes zote (kwa duka)
fundi-mobile devices                     # adb devices + iPhones
fundi-mobile diagnostics                 # battery/storage/props (HALISI kwa adb)
fundi-mobile consent form                # fomu ya kusaini
fundi-mobile consent add --name "Fatuma" --phone 0712345678 --imei 354123456789012 \
    --brand samsung --model A12 --service reset_password_recovery --destroys --price 30000
fundi-mobile agentic run --customer Fatuma --brand samsung --model A12 \
    --imei 354123456789012 --service reset_password_adb \
    --problem "nimesahau password" --tech "Fundi wa Kwanza"
fundi-mobile agentic jobs
fundi-mobile dashboard
fundi-mobile book <job_id> book.html     # kitabu kidigitali
fundi-mobile menu                        # interactive
```

## Mtiririko wa Agentic (kila kazi)

```
P  receptionist — pokea tatizo, ainisha kifaa
P  vision       — scan halisi (adb props/battery/storage) au mwongozo wa brand
I  diagnoser    — tatizo → njia (kutoka services.json + brands.json)
H  hitl         — HAKUNA consent? Job inasimama: awaiting_consent ❗
I  solver       — ADB halisi (reset_adb) au mwongozo wa brand (recovery/flash/button)
T  tester       — pima upya (halisi kama adb ipo)
V  verifier     — thibitisha kwa mteja
D  scribe       — narration Kiswahili + kitabu kidigitali (HTML)
D  learner      — learning_log.json (maarifa ya kweli)
```

## Data (source of truth)

| Faili | Maudhui |
|-------|---------|
| `data/mobile/services.json` | huduma + bei TZS + risk + `requires_consent` |
| `data/mobile/brands.json` | recovery/download combos, flash tools, **button master codes** |
| `data/mobile/consents.json` | consent log (HITL) — kazi bila hii IMEZUIWA |
| `data/mobile/jobs.json` | jobs zote + frames za agents |
| `data/mobile/learning_log.json` | maarifa yanayojifunzwa |

## Bei (TZS) — kutoka services.json

| Huduma | Bei | Muda |
|--------|-----|------|
| Reset Password (ADB) | 30,000 | 10 min |
| Reset Password (Recovery) | 30,000 | 15 min |
| **Reset (Button phone)** | **20,000** | 10 min |
| Bypass FRP 5-10 / 11+ | 50,000 / 100,000 | 20 / 40 min |
| Flash Firmware | 80,000 | 45 min |
| iPhone Reset / DFU | 50,000 / 80,000 | 30 / 45 min |
| Update OS | 40,000 | 30 min |
| Backup Data | 25,000 | 30 min |
| Diagnostics | 15,000 | 10 min |

## Tests

```bash
cargo test   # consent store roundtrip (kazi bila consent imezuiliwa)
```
