# MTECH OS — Desktop (Tauri v2)

Shell ya desktop yenye **UI mbili** (SEHEMU 2 + 3):

1. **OS AND APP INSTALLATION** — discovery, majina ya kiotomatiki, ISO catalog,
   bundles (categories 8), bei per-PC + malipo, VPN, Agentic AI
2. **COMPUTER SOLUTIONS** — scan matatizo kwa wakati mmoja, chat ya agent,
   kila siku (auto-daily), dashboards za admin

## Anza (dev)

```bash
# 1. Server ya agent (Rust) — server yako au ya ndani:
cd ../../hermes-agent/fundi-deploy/server && docker compose up -d --build

# 2. Desktop (Tauri v2):
cd src-tauri
cargo install tauri-cli --version "^2"
cargo tauri dev          # dev
cargo tauri build        # production → .deb / .AppImage / .dmg (HAKUNA exe ya wageni)
```

## Kwa nini Tauri v2?
- UI nzuri (webview) + core ya **Rust** (kanuni ya mmiliki: Rust inabeba kila kitu)
- Bundle ndogo sana (si Electron) · targets: Linux `.deb`/`.AppImage`, macOS `.dmg`
- `MTECH_API` env inaelekeza shell kwenye server ya mbali (mfano `http://SERVER:8080`)
  — hii ndiyo remote kwa mbali kupitia WireGuard.

## Umiliki
Licensed by **Mbilinyi Tech** — mbilinyitech.co.tz · mbilinyitech@gmail.com · 079675645
