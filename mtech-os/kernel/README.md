# MTECH OS — Kernel Bridge

Hapa ndipo **Qwen 2.5 VL 3B inaungana na kernel**. Mfumo una tabaka mbili:

```
┌──────────────────────────────────────────────┐
│  Qwen 2.5 VL 3B (akili) + MTAALAMU skills   │
├──────────────────────────────────────────────┤
│  agent (Python)  ← /dev/mtech + /proc,/sys   │
├──────────────────────────────────────────────┤
│  mtech_dev.ko  (moduli hii — kprobes)        │
├──────────────────────────────────────────────┤
│  LINUX KERNEL 7.3 (upstream/linux, torvalds) │
└──────────────────────────────────────────────┘
```

## Files

| File | Kazi |
|---|---|
| `mtech_dev.c` | Moduli ya kernel → `/dev/mtech` (ring buffer ya matukio: kila process inayoanza/kufa, kupitia kprobes za `do_execve`/`do_exit`) + `/proc/mtech_status` |
| `mtech.config` | Config fragment — BPF, kprobes, tracing, seccomp (zinazoitajika na MTECH) |
| `build-kernel.sh` | Build kernel kamili kutoka `upstream/linux` (torvalds) na module ya MTECH |
| `Makefile` | Build module pekee dhidi ya kernel inayoendesha |

## Kwa nini kernel?

Agent anapoomba "iona kila kitu kinachotokea kwenye kompyuta":

1. **`/dev/mtech`** — moduli inasikiliza kernel yenyewe (kprobes). Hakuna app inayoweza kujificha: kila `exec` (programu iliyozinduliwa) na `exit` inaonekana mara moja, kabla hata ya logging yoyote ya userland.
2. **`/proc`, `/sys`** — uchunguzi wa hali: processes, kumbukumbu, disk, joto.
3. **eBPF (hiari, ya juu)** — `mtech_mon.bpf.c` (inakuja) inaongeza `openat`/`connect` — kila file na connection.

## Build (kwenye Kali/Debian)

```bash
# a) Kernel kamili ya MTECH (Linux 7.3 + module) — dakika 10-40
./mtech-os/kernel/build-kernel.sh --install

# b) Module tu kwenye kernel iliyopo (haraka, ya majaribio)
./mtech-os/kernel/build-kernel.sh --module-only
sudo insmod mtech_dev.ko

# Hakiki:
cat /proc/mtech_status
cat /dev/mtech          # matukio live
./mtech-os/agent/... mtech watch   # agent akisoma pia
```

## Ukweli wa production

- Moduli hii ina **kusoma tu** — haibadilishi tabia ya kernel; inaona na kuripoti.
- Amri hatari za agent zinapita kwenye **HITL gate** (`agent/mtech_agent/safety.py`) kabla ya kufika kwa shell.
- Bila module, agent ina fallback ya `/proc` (`kernel_events.py`) — mfumo unaendelea kufanya kazi.
