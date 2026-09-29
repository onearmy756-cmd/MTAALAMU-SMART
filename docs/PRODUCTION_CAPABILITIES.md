# MTAALAMU SMART — Production Capabilities (Ukweli wa Uhandisi)

## Senior engineer position

Software **haiwezi** kuona PCB ya motherboard kama kamera, wala “video ya kernel” bila drivers maalum za OEM, debugger hardware, au hypervisor. Dai kinyume ni udanganyifu.

Tunatoa **production stack ya kweli** inayofanya kazi:

| Layer | Implementation | Status |
|-------|----------------|--------|
| OS metrics | `sysinfo` (CPU, RAM, disk, net, processes) | Production |
| Deep probe | `/proc`, `/sys/class/thermal`, process tree (ppid) | Production (Linux full; Windows via sysinfo) |
| Live Vision UI | SVG + data halisi inayosasishwa | Production |
| Multi-agent 10 + HITL + PIITVD | Rust orchestrator | Production |
| Safe remediation | catalog + HITL + audit log | Production |
| Reports + learning log | JSON/HTML/MD | Production |
| R/Shiny dashboard | live probe + agentic | Production |
| Motherboard X-ray / silicon bus video | — | **Impossible without hardware instrumentation** |
| AGI / solve any hardware by click | — | **Not available** |
| Studio human voice | browser TTS or external TTS API | Partial (browser TTS shipped) |

## CLI (production)

```bash
cd engine-rust && cargo build --release

./target/release/mtaalamu deep --top 15
./target/release/mtaalamu vision
./target/release/mtaalamu agentic --msg "Kompyuta inaenda polepole" --approve
./target/release/mtaalamu remediate
./target/release/mtaalamu remediate --action report_top_cpu
./target/release/mtaalamu remediate --action clear_user_temp --approve
```

## What `deep` returns (real)

- hostname, OS, kernel
- CPU % / brand / cores, RAM %, load average, uptime
- processes with **pid + ppid** (tree)
- disks + network interfaces (bytes)
- Linux: thermal zones °C, sample TCP table from `/proc/net/tcp`
- `capabilities[]` — what this host actually supports
- `issues[]` from thresholds — not fictional malware names

## Remediation policy

- **low risk**: list temp, report top CPU, `sync` — no HITL
- **medium**: clear user temp files — **requires `--approve` / HITL**
- **never**: arbitrary kill of system processes, firmware flash, disk wipe without explicit separate tooling and legal consent

## UI

```r
shiny::runApp("web-r", port = 3838)
```

Agentic tab: OS PROBE + live map from `build_live_agent_data()` + HITL + book download.

## Verdict

Everything **that is software-observable on a normal OS** is implemented for production.  
Everything that requires **hardware-level vision of the silicon** is outside the product boundary by physics and OS security — not by lack of effort.
