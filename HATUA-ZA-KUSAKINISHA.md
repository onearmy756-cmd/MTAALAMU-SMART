# 📥 HATUA ZA KUSAKINISHA — MTECH OS + MTAALAMU SMART

> Njia zote 4 zinafanya kazi kwenye **Linux · Windows · macOS** (na simu za
> ARM64 kupitia Docker/ISO). Chagua MOJA kulingana na unachohitaji.

---

## NJIA 1 — Computer yako ya sasa (rahisi zaidi, hakuna ISO/Docker)

### 1. Pata repo
```bash
git clone --depth 1 -b slim https://github.com/onearmy756-cmd/MTAALAMU-SMART.git
cd MTAALAMU-SMART            # slim = 21MB (folders kubwa zimeondolewa)
# kwa KILA KITU (upstream, openmrs n.k.): -b main
```

### 2. ONE-CLICK install (inasakinisha kila kitu yenyewe)
```bash
# Windows:  double-click  install.bat   (au setup.cmd — ladder ya binary)
# Linux / macOS:
cd hermes-agent
python3 setup_all.py
```

Inasakinisha kwa mpangilio (hakuna configuration):
1. gcc/make (C)
2. **Rust (cargo)** — rustup rasmi
3. **R (Rscript)**
4. **LibreOffice** (Swahili locale)
5. Python venv + deps (Pillow, pyautogui)
6. **Ollama + qwen2.5vl:3b**
7. KIOTOMATIKI: account (DIAMOND + ADMIN) → server (127.0.0.1:8795) → dashboard inafunguka

Chaguo: `--check` (ukaguzi tu) · `--skip-rust` · `--skip-r` · `--skip-office` ·
`--no-model` · `--email wewe@mail.com`

### 3. Kama kitu kilikosekana baadaye — ndani ya OS:
```bash
python3 -m mtaalamu env install all    # ollama, qwen, rust, engine, r, r-packages, node, libreoffice
```

### 4. Anza mfumo
```bash
cd hermes-agent
python3 -m mtaalamu serve        # API: http://127.0.0.1:8795
# fungua web-html/mtaalamu-unified.html kwenye browser (tabs 11)
```

---

## NJIA 2 — Bundle kamili (HAKUNA kusakinisha Python/R/Rust)

```bash
# kwenye machine moja (yenye python tu):
python3 hermes-agent/mtaalamu/setup_builder.py --bundle-runtime
# → hermes-agent/mtaalamu/dist/MTAALAMU-bundle-<os>-<arch>.zip
#    bin/mtaalamu-linux|MTAALAMU-Setup.exe  (Python NDANI — PyInstaller)
#    bin/mtaalamu-engine                    (engine ya Rust NDANI)
```

Kwenye computer nyingine:
```bash
unzip MTAALAMU-bundle-*.zip && cd MTAALAMU-bundle-*
chmod +x mtaalamu.sh && ./mtaalamu.sh serve     # Windows: mtaalamu.cmd serve
```

---

## NJIA 3 — Docker (amd64 + arm64 — simu na PC)

```bash
git clone --depth 1 https://github.com/onearmy756-cmd/MTAALAMU-SMART.git
cd MTAALAMU-SMART
./mtech-os/kali/docker-run.sh            # inajenga image yenyewe kwa mara ya kwanza
./mtech-os/kali/docker-run.sh serve      # API tu (port 8795)

# multi-arch (amd64 + arm64 kwa pamoja):
docker buildx build --platform linux/amd64,linux/arm64 -t mtech-os .
```
Ndani: Kali halisi + Rust + R + LibreOffice + apps za mfumo zote.

---

## NJIA 4 — ISO KAMILI ya Kali (MTECH OS halisi — USB boot)

```bash
git clone --depth 1 -b main https://github.com/onearmy756-cmd/MTAALAMU-SMART.git
cd MTAALAMU-SMART
./mtech-os/kali/build-iso.sh --check            # ukaguzi (packages 60/60 ✔)
sudo ./mtech-os/kali/build-iso.sh               # amd64
sudo ./mtech-os/kali/build-iso.sh --arch arm64  # simu/PC za 64-bit ARM
sudo ./mtech-os/kali/build-iso.sh --everything  # zana ZOTE za Kali (~1800)
sudo ./mtech-os/kali/build-iso.sh --container   # jenga ndani ya docker
```

Flash `mtech-os/build/kali-work/images/*.iso` kwenye USB (balenaEtcher / `dd`)
→ boot → **Live** au **Install**.

Ndani ya ISO tayari: zana+drivers+terminals za Kali · Rust · R · LibreOffice ·
apps za mfumo (dirisha la **System Apps**) · firstboot inavuta **qwen2.5vl:3b**
yenyewe · huduma za systemd (mtech-agent, mtech-ollama).

---

## BAADA YA KUINGIA NDANI (OS yoyote — vitu vya kila siku)

```bash
mtaalamu apps                        # apps za mfumo (hali halisi)
mtaalamu apps launch web-r           # UI halisi ya Shiny → http://localhost:3838
mtaalamu apps launch openmrs         # UI halisi ya OpenMRS → :8080/openmrs (docker)
mtaalamu apps launch home-assistant  # UI halisi ya HA → :8123 (docker)
mtaalamu apps launch desktop         # Hermes Desktop (Electron)
mtaalamu apps install web-r          # sakinisha/pakua mahitaji ya app (npm/R/docker)

mtaalamu kali                        # zana+terminals+drivers+partitions za Kali
mtaalamu kali tool nmap              # fungua zana kwenye terminal HALISI
mtaalamu term                        # fungua command prompt mpya

mtaalamu fs mkdir ~/MTECH/ripoti     # tengeneza folder/directory
mtaalamu fs find '*.pdf' ~           # kupata vitu
mtaalamu fs download https://…       # pakua app/file → ~/Downloads
mtaalamu fs memory                   # memory access halisi
mtaalamu fs partitions               # orodha ya partitions
mtaalamu fs install htop             # apt/brew/winget/pip:/npm:

mtaalamu env                         # hali ya mazingira (cargo/engine/R/Ollama/Qwen)
mtaalamu env install qwen            # Ollama + qwen2.5vl:3b ndani ya OS
mtaalamu admin unlock                # ADMIN (owner) — partition-create n.k.

python3 -m mtaalamu boot             # UKIWAKA (checks halisi za OS)
python3 -m mtaalamu doctor           # ukaguzi wa mashine
```

**Keys hiari** (Settings → Environment / .env): `CLICKPESA_CLIENT_ID` +
`CLICKPESA_API_KEY` (malipo ya TZS) · `MTECH_ADMIN_KEY` (usalama zaidi).

**API kamili**: angalia `hermes-agent/mtaalamu/README.md` na
`mtech-os/README.md` (Docker · ARM · kernel · huduma).
