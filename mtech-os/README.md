# MTECH OS

**Linux + Akili (Qwen 2.5 VL 3B + MTAALAMU SMART) + Kernel Bridge + GUI ya kipekee.**

> Mfumo unaojenga OS kamili: kernel ya Linux (torvalds), base ya Kali Linux (live-build
> rasmi), akili ya mtaalamu (Qwen 2.5 VL 3B + skills 146 za MTAALAMU SMART), na GUI
> wrapper (**MTECH Shell**) inayoona kila kitu kupitia kernel (`/dev/mtech`).

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
├── Makefile             → make skills|ask|watch|serve|shell|module|kernel|iso|install|smoke
├── install.sh           ← weka kwenye OS iliyopo (wrapper mode)
├── agent/               ← AKILI: Qwen 2.5 VL 3B + tools + skills bridge + HITL
│   ├── mtech_agent/     (config, llm, tools, skills_bridge, kernel_events, vision, safety)
│   └── requirements.txt
├── kernel/              ← KIUNGANISHI CHA KERNEL
│   ├── mtech_dev.c      → moduli ya kernel: /dev/mtech (kprobes: exec/exit)
│   ├── mtech.config     → config fragment (BPF, kprobes, seccomp…)
│   ├── build-kernel.sh  → build kernel kutoka upstream/linux + module
│   └── README.md
├── gui/                 ← GUI YA KIPEEKEE
│   └── mtech_shell.py   → MTECH Shell (PySide6/Qt6): events live + console + skills + HITL
├── kali/                ← ISO YA KALI + MTECH
│   ├── build-iso.sh     → jenga ISO (live-build rasmi)
│   └── variant-mtech/   → package list + hook ya chroot
└── services/            ← systemd: mtech-agent, mtech-ollama, mtech-firstboot
```

## Njia 1 — Jenga ISO kamili ya MTECH OS (Kali)

```bash
sudo apt install -y live-build rsync
make -C mtech-os iso            # ISO → mtech-os/build/kali-work/images/
```

ISO ina: Kali XFCE + zana za mtaalamu + agent + MTECH Shell + kernel headers +
module ya MTECH + Ollama. **Mara ya kwanza inapoanza**, `mtech-firstboot`
inavuta `qwen2.5vl:3b` (GB ~2.3 — mtandao unahitajika mara moja).

## Njia 2 — Weka kwenye OS yako (wrapper mode)

```bash
sudo ./mtech-os/install.sh                # kila kitu + Qwen 2.5 VL 3B
sudo ./mtech-os/install.sh --with-kernel  # + kernel ya MTECH (Linux 7.3 + /dev/mtech)
```

Baada ya kuupa access (root/systemd), mfumo una udhibiti wa kompyuta yote:

| Amri | Inafanya |
|---|---|
| `mtech probe` | uchunguzi wa mfumo (kernel, processes, mem, mounts) |
| `mtech skills` | skills zote 146 za MTAALAMU (na `--q neno` kutafuta) |
| `mtech skill --id … --inputs …` | tekeleza skill (formula halisi) |
| `mtech ask "kompyuta inaenda polepole"` | agent loop kamili: Qwen + zana + HITL |
| `mtech watch` | matukio ya kernel live (`/dev/mtech` au fallback ya `/proc`) |
| `mtech serve` | API ya HTTP kwa MTECH Shell (port 8790) |

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
