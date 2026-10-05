# MTECH OS

**Akili halisi ya mtaalamu ndani ya OS YOYOTE (Linux, Windows, macOS) — hakuna ISO,
hakuna Docker, hakuna dual-boot.**

> MTECH inasakinishwa moja kwa moja kwenye OS iliyopo (`python3 install.py`) na:
> 1) **inaona full screen yote** (vision ya Qwen 2.5 VL 3B), 2) **inadhibiti kompyuta
> kwa ukamilifu** (mouse + keyboard — mtumiaji akiruhusu `--allow-control`),
> 3) **inaunganisha kernel** (Linux: module halisi `/dev/mtech`; Windows: Sysmon
> Event Log; macOS: Unified Log), 4) zana + drivers za OS husika zinawasili halisi.
> Nayo pia inaweza kujengwa kama **ISO kamili ya Kali** (hiari — angalia chini).

```
╔══════════════════════════════════════════════════════════════╗
║  APPS: MTAALAMU SMART + Kali Tools (desktop launchers)       ║
║   • console ya Qwen  • skills 146  • full screen vision      ║
║   • FULL CONTROL (mouse/keyboard, ALLOW ya mtumiaji)         ║
╠══════════════════════════════════════════════════════════════╣
║  MTECH AGENT (Python, cross-OS)                              ║
║   Qwen 2.5 VL 3B (Ollama) ⇄ tools: shell, files, probe,      ║
║   vision (full screen), control, events, MTAALAMU skills 146 ║
╠══════════════════════════════════════════════════════════════╣
║  OS yoyote: Linux ⇄ Windows ⇄ macOS (HALISI)                 ║
║  screenshot: ImageGrab/scrot/PowerShell — input: pyautogui/  ║
║  xdotool — probe: /proc/CIM/sysctl — events: kernel module/  ║
║  Sysmon/Unified Log — shell: bash/PowerShell/zsh             ║
╚══════════════════════════════════════════════════════════════╝
```

```
╔══════════════════════════════════════════════════════════════╗
║  MTECH SHELL  — GUI ya kipekee (wrapper ya desktop ya Kali)  ║
║   • matukio ya kernel live  • console ya Qwen  • skills 146  ║
║   • HITL approval (kibali cha binadamu kwa hatua hatari)     ║
╠══════════════════════════════════════════════════════════════╣
║  MTECH AGENT (Python) — akili inayofanya kazi                ║
║   Qwen 2.5 VL 3B (Ollama) ⇄ tools: shell, files, probe,      ║
║   vision (macho), control (mikono), kernel events,           ║
║   MTAALAMU SMART (skills 146 + formulas)                     ║
╠══════════════════════════════════════════════════════════════╣
║  KERNEL BRIDGE — mtech_dev.ko → /dev/mtech + /proc/mtech     ║
╠══════════════════════════════════════════════════════════════╣
║  KALI LINUX (live-build rasmi) + LINUX KERNEL (torvalds)     ║
╚══════════════════════════════════════════════════════════════╝
```

## Muonekano = Kali halisi (kila kitu kwenye picha)

| Kwenye picha | Kwenye MTECH OS | File |
|---|---|---|
| Panel ya juu: Applications ▾, Places ▾, tray icons, saa "Tue, 02:45", power | XFCE panel-1 (Whisker + Places + systray + clock `%a, %H:%M` + power) | `kali/variant-mtech/includes.chroot/etc/xdg/xfce4/xfconf/xfce4-panel.xml` |
| Dock ya chini: Files, Terminal, Trash, Wireshark, Burp, Mousepad, Settings, Undercover, ▦ | XFCE panel-2 (launchers 9 + appfinder) | iko panel.xml hii hiyo |
| Desktop icons: kali, File System, Firefox, Burp Suite, Nmap, Metasploit, Terminal | xfdesktop (Home/FS/Trash) + hook inaweka .desktop za Firefox/Burp/Terminal + custom Nmap/Metasploit + MTAALAMU + Kali Tools | `hooks/live/0140-desktop-icons.hook.chroot` + `includes.chroot/etc/xdg/xfce4/xfconf/xfce4-desktop.xml` |
| Terminal: greeting ya dragon (kali-dragon ASCII) + `kali@kali:~$` | `kali-linux-core` inayo na inaonesha hello na default prompt (kali) + `~/.zshrc` ya Kali halisi | (package: kali-linux-core) |
| Dirisha **"Kali Tools"**: Search… + categories 11 (Information Gathering, Vulnerability Analysis, Web Application, Password Attacks, Exploitation, Sniffing & Spoofing, Post Exploitation, Forensics, Reporting, Reverse Engineering, Wireless) | App ya PySide6 yenye search + categories 11 zinazoanzisha zana halisi (nmap, nikto, burpsuite, john, msfconsole, wireshark, autopsy, cherrytree, r2, aircrack-ng) | `desktop/kali-tools-window.py` + `desktop/kali-tools.desktop` |
| CPU: 7% / RAM: 40% (chips chini-kulia) | conky (chips 2, bar ya kijani/bluu) + autostart | `includes.chroot/etc/skel/.conkyrc` + `includes.chroot/etc/xdg/autostart/mtech-conky.desktop` |
| Wallpaper ya dragon | `kali-wallpapers` (package rasmi — dragon chaguomsingi ya Kali) | package-list |

> App ya **MTAALAMU SMART** iko menyu ya Applications + Desktop, na ina kitufe
> **"KALI TOOLS ▸"** kinachofungua dirisha hilo la categories 11.

## ⚡ KUTOKA GITHUB HADI DESKTOP — HATUA 5 (OS yoyote, hakuna ISO/Docker/dual-boot)

### Hatua 1: Pata code
```bash
git clone https://github.com/onearmy756-cmd/MTAALAMU-SMART.git
cd MTAALAMU-SMART/mtech-os
```

### Hatua 2: Sakinisha (kila kitu: venv + Ollama + modeli + apps)
```bash
python3 install.py                     # Linux / macOS / Windows (python install.py)
# chaguo: --no-model  |  --no-gui  |  --kernel (Linux: module ya /dev/mtech)
```

### Hatua 3: Fungua APP ya desktop
- **Linux**: menyu ya Applications → **MTAALAMU SMART** (au `~/.local/share/applications`)
- **Windows**: Desktop → **MTAALAMU SMART.lnk**
- **macOS**: Desktop → **MTAALAMU SMART.command**

### Hatua 4: Ruhusu FULL CONTROL (mtu akiruhusu — inadumu)
```bash
python3 install.py --allow-control     # au baadaye: python -m mtech_agent allow on
# au kwa session moja tu:  python -m mtech_agent ask "..." --allow-control
# au env:                  MTECH_ALLOW_CONTROL=1
```
Agent sasa inaweza kubofya, kuandika, kusogeza kipanya — **kila hatua bado inauliza
kibali (HITL)**, na `mtech allow off` inazima wakati wowote. PyAutoGUI FAILSAFE:
sogeza kipanya kona ya 0,0 kusimamisha kila kitu mara moja.

### Hatua 5: Tumia kama mtaalamu halisi
```bash
mtech ask "kompyuta yangu inaenda polepole, tafuta tatizo na ulimize"   # inafanya kazi yote
mtech skills --q betri          # skills 146 za MTAALAMU
mtech watch                     # matukio ya OS/kernel LIVE
mtech probe                     # hali kamili ya mfumo (OS yoyote)
```

## Kila kitu ni HALISI (OS zote tatu)

| Uwezo | Linux | Windows | macOS |
|---|---|---|---|
| **Full screen vision** | Pillow/scrot | Pillow ImageGrab / PowerShell | Pillow / `screencapture` |
| **Full control** (mouse/kiiboard) | xdotool | pyautogui (native) | pyautogui (native) |
| **Probe ya mfumo** | `/proc` | PowerShell/CIM | `sw_vers`, `/proc`-kama |
| **Matukio ya chini** | **module `/dev/mtech`** (kprobes) | **Sysmon Event Log** | **Unified Log** (`log stream`) |
| **Shell** | bash/zsh | PowerShell | zsh |
| **Drivers/zana** | packages za distro yako | winget/choco | brew |

## (HIARI) ISO kamili ya Kali — kama unataka OS yake

```bash
sudo ./mtech-os/kali/build-iso.sh            # Kali halisi (kali-linux-large)
sudo ./mtech-os/kali/build-iso.sh --everything  # zana ZOTE (~1800)
sudo ./mtech-os/kali/build-iso.sh --arch arm64  # ARM64 (simu/PC za 64-bit ARM, Raspberry Pi)
sudo ./mtech-os/kali/build-iso.sh --arch armhf  # ARM32
./mtech-os/kali/build-iso.sh --container     # kutoka OS yoyote yenye Docker/Podman
./mtech-os/kali/build-iso.sh --check         # hakiki packages dhidi ya repo halisi ya Kali
```
Kisha flash kwenye USB (balenaEtcher / `dd`) → boot → **Live** au **Install**.

Ndani ya ISO (zote zimeunganiwa moja kwa moja):
- **Kali kamili** — zana zote, drivers halisi (firmware zote), partitions tools, terminals zote
- **Rust + R + LibreOffice** — compiler, analytics/chati, ofisi kamili (packages za Kali)
- **MTAALAMU SMART + apps za mfumo** — OpenMRS, Home Assistant, Web R, Desktop, Website
  (dirisha la "System Apps" + tab ya APPS kwenye dashboard)

## DOCKER — MTECH OS kama container (bila ISO, bila kusakinisha)

```bash
docker build -t mtech-os -f mtech-os/kali/Dockerfile.mtech .
./mtech-os/kali/docker-run.sh              # shell ya MTECH OS + API 8795
./mtech-os/kali/docker-run.sh serve        # API tu

# Multi-arch (amd64 + arm64 — simu zote na PC):
docker buildx build --platform linux/amd64,linux/arm64 -t mtech-os --push .
```
Ndani ya container: Kali halisi + Rust + R + LibreOffice + apps za mfumo zote
(OpenMRS/Home Assistant/Web R/Desktop/Website) — `mtaalamu apps`, `mtaalamu kali`,
`mtaalamu serve`.

## Vyanzo (vilivyopakuliwa kwenye `upstream/`)

| Chanzo | Kazi | Mahali |
|---|---|---|
| **torvalds/linux** (GitHub) | Kernel halisi ya Linux (7.3) — msingi wa MTECH kernel + module | `upstream/linux` |
| **kali-live** (GitLab rasmi ya Kali) | Live-build config rasmi — ndiyo inayotengeneza ISO za Kali | `upstream/kali-live` |
| **hermes-agent/data + engine-rust** | Akili ya MTAALAMU SMART: skills **146**, formulas, diagnosis, engine ya Rust | `hermes-agent/` |
| **Ollama + qwen2.5vl:3b** | Mwangaza (LLM + vision) — inavutwa na firstboot | inasakinishwa |

## Muundo wa folda

```
mtech-os/
├── README.md            ← hii (nyaraka kuu)
├── Makefile             → make skills|ask|watch|serve|app|module|kernel|iso|iso-everything|install|smoke
├── install.py           ← INSTALLER MOJA ya OS zote 3 (hakuna ISO/Docker/dual-boot)
├── install.sh           ← (Linux) kama install.py + systemd services
├── agent/               ← AKILI: Qwen 2.5 VL 3B + tools + skills bridge + HITL
│   ├── mtech_agent/     (config, llm, tools, skills_bridge, kernel_events, vision, safety,
│   │                     platform ← adapters za OS zote: screenshot/input/probe/events/shell)
│   └── requirements.txt
├── kernel/              ← KIUNGANISHI CHA KERNEL
│   ├── mtech_dev.c      → moduli ya kernel: /dev/mtech (kprobes: exec/exit)
│   ├── mtech.config     → config fragment (BPF, kprobes, seccomp…)
│   ├── build-kernel.sh  → build kernel kutoka upstream/linux + module
│   └── README.md
├── gui/                 ← APP YA MTAALAMU SMART
│   └── mtech_shell.py   → app ya desktop (PySide6/Qt6): console ya Qwen + skills + events + zana za Kali
├── desktop/             ← APP YA DESKTOP (kama Nmap/Firefox)
│   ├── mtaalamu.desktop → menyu ya Applications + icon ya Desktop
│   ├── mtaalamu-app     → launcher
│   └── mtaalamu.svg     → icon (ubongo + circuit)
├── kali/                ← ISO YA KALI + MTECH
│   ├── build-iso.sh     → jenga ISO (live-build rasmi)
│   └── variant-mtech/   → package list + hook ya chroot
└── services/            ← systemd: mtech-agent, mtech-ollama, mtech-firstboot
```

## Njia 1 — OS yoyote (NALIYO MSINGI — hakuna ISO/Docker/dual-boot)

```bash
python3 mtech-os/install.py              # Linux/Windows/macOS — kila kitu
python3 mtech-os/install.py --allow-control   # + full control (mouse/keyboard)
python3 mtech-os/install.py --kernel          # (Linux) + module ya /dev/mtech
```

Angalia **"KUTOKA GITHUB HADI DESKTOP — HATUA 5"** hapo juu kwa maelezo kamili.

## Njia 2 — Weka kwenye Linux na systemd (huduma zote)

```bash
sudo ./mtech-os/install.sh                # MTAALAMU SMART kama app + Qwen 2.5 VL 3B
sudo ./mtech-os/install.sh --with-kernel  # + kernel ya MTECH (Linux 7.3 + /dev/mtech)
sudo ./mtech-os/install.sh --kali-look    # + panel/dock/conky kama Kali halisi
```

Desktop yako haibadilishwi — **MTAALAMU SMART inaonekana kama app tu**:
menyu ya Applications + icon ya Desktop. Baada ya kuipa access (root/systemd),
app ina udhibiti wa kompyuta yote:

| Amri | Inafanya |
|---|---|
| `mtech probe` | uchunguzi wa mfumo (kernel, processes, mem, mounts) |
| `mtech skills` | skills zote 146 za MTAALAMU (na `--q neno` kutafuta) |
| `mtech skill --id … --inputs …` | tekeleza skill (formula halisi) |
| `mtech ask "kompyuta inaenda polepole"` | agent loop kamili: Qwen + zana + HITL |
| `mtech ask "..." --allow-control` | + full control (mouse/keyboard) kwa session |
| `mtech allow on/off` | fungua/funga full control (dumufuli) |
| `mtech watch` | matukio ya kernel/OS live (kila OS) |
| `mtech serve` | API ya HTTP kwa MTAALAMU app (port 8790) |

## Kernel — "inaona kila kitu kupitia kernel"

```bash
cd mtech-os/kernel && make                      # moduli dhidi ya kernel inayoendesha
sudo insmod mtech_dev.ko
cat /proc/mtech_status                          # hali
cat /dev/mtech                                  # kila exec/exit ya process — LIVE
mtech watch                                     # agent akisoma pia
```

`/dev/mtech` inatoka kwenye kprobes za kernel (`do_execve`, `do_exit`) —
**hakuna programu inayoweza kujificha**: mfululizo wa matukio unatoka kwenye kernel
yenyewe, kabla ya kila log ya userland. Kernel kamili ya MTECH (Linux 7.3 kutoka
torvalds + config ya MTECH) inajengwa na `kernel/build-kernel.sh`.

## Usalama (HITL) — kanuni za MTAALAMU SMART

- **HITL**: hatua za hatari kubwa (kill, firewall, rm recursive, systemd disable…)
  zinahitaji kibali — `--approve` kwenye CLI, vitufe kwenye MTECH Shell.
- **FORBIDDEN**: `rm -rf /`, `mkfs`, `dd of=/dev/sd…`, fork bomb, shutdown — zimezuiwa
  kabisa, hata na kibali (`agent/mtech_agent/safety.py`).
- **LLM haihesabu kamwe** (R-1): hesabu = formula engine (evaluator salama ya `ast`
  au engine ya Rust ya MTAALAMU). Qwen inaongea, inaona, inaamua — si kuhesabu.
- Moduli ya kernel ni **read-only** — inaona, haibadilishi kernel.

## Muungano na MTAALAMU SMART (mradi wa "smart")

- Skills **146** (`hermes-agent/data/skills/skills.json`) — trades 21 zote.
- Formulas + diagnosis + trades data zote → zimefungamenwa kwenye ISO na install.
- Engine ya Rust (`mtaalamu`) ikiwepo, `mtech_agent` inaitumia; la, evaluator
  salama ya formula inafanya kazi offline.
- MTECH Shell + agent ni UI mpya juu ya akili ile ile — data-driven (KANUNI 2/5).

## Nini kijacho (roadmap)

- [ ] `mtech_mon.bpf.c` — eBPF: `openat`/`connect` (kila file na connection)
- [ ] HITL kupitia MTECH Shell (vitufe vyenye action halisi ya kuidhinisha)
- [ ] Wayland support ya control_input (kwa sasa X11/xdotool)
- [ ] `arm64` ISO variant (Raspberry Pi n.k.)
