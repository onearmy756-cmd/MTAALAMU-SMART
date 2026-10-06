# MWONGOZO WA KUANZISHA SERVER YA KWANZA (MTECH OS)
*Hii ni hatua za kweli — kila kitu kiko kwenye mfumo tayari.*

## UNACHOHITAJI
- Kompyuta/server (Linux, 8GB+ RAM, diski 100GB+) — au VPS
- Docker + Docker Compose (`docker --version && docker compose version`)
- Mtandao wa LAN (kompyuta za wateja zinaweza kufikia IP ya server)
- (Hiari) VPS ya nje kwa VPN ya nchi-kwa-nchi

## HATUA 1 — PATA CODE NA ANZA MFUMO
```bash
git clone https://github.com/onearmy756-cmd/MTAALAMU-SMART.git
cd MTAALAMU-SMART/hermes-agent/fundi-deploy/server
docker compose up -d --build
```
Huduma zinazoanza: **api** (mfumo mzima + dashboard :8080), dnsmasq (PXE),
nginx (:80), samba (images), **ollama + llamacpp** (AI ya ndani),
suricata (IDS), wazuh (SIEM), kali-tools (zana 34).

## HATUA 2 — THIBITISHA
```bash
curl http://localhost:8080/health          # → {"ok":true,...}
```
Fungua dashboard: `http://IP-YA-SERVER:8080/ui` → **👍 mfumo hai**

## HATUA 3 — ONBOARDING (dakika 2)
Dashboard → **🚀 Onboarding** → chagua aina ya kifaa (server) → *Configure*.
Mfumo unajipanga: bandari, data dir, VPN peer, huduma zinazohitajika.

## HATUA 4 — WIREGUARD VPN (kazi kwa mbali)
Dashboard → **🔒 VPN** → **INIT** → ongeza **PEER** kwa kila kifaa cha mbali →
pakua conf → **UP**. Kazi zote za mbali zinapita hapa (siri, encrypted).

## HATUA 5 — WATEJA WAKO WALIPE (BILI)
1. Dashboard → **💰 BILI** → weka salio la mteja (rejea halisi ya ClickPesa/benki)
   au **Subscribe** (chagua kundi: Basic/Standard/Biashara/Kubwa).
2. Mteja anapata account yake — anaona **ripoti zake pekee**.

## HATUA 6 — WEGENI KOMPYUTA ZA WATEJA (AGENTS)
Kwenye kila kompyuta ya mteja (Windows au Linux):
1. Dashboard → **👥 AGENTS** → ⬇ Pakua `agent-mtech.ps1` / `.sh`
2. Endesha (amri ziko kwenye ukurasa) — agent anajisajili, anapata token,
   anatuma afya kila sekunde 20, na anapata kazi ulizoidhinisha tu.

## HATUA 7 — KAZI YA KWANZA (mfano: shule na PC 20)
1. **MATRIX** → Gundua → chagua PC zote → OS + Bundle → ACTIVATE
2. Tab **Jobs** → **RUHUSU** kila kazi (HITL)
3. Agents wanaendesha kwa wakati mmoja — progress live
4. **📄 RIPOTI** → PDF ya brand (TZS 5,000) → tuma kwa mteja
5. **💰 BILI** → salio limekatwa kiotomatiki (subscription au pay-per-use)

## HATUA 8 — BACKUP NA USALAMA WA SERVER
```bash
# backup ya kila siku ya data (DB, ushahidi, configs)
tar -czf /backup/mtech-$(date +%F).tar.gz data/
```
- Weka server kwenye chumba salama + UPS
- Badilisha passwords (Admin/fundi) mara moja — tab **👤 Admin**
- Zima bandari zisizohitajika kwenye router; dashboard :8080 iwe LAN pekee
  (au weka kwenye reverse-proxy yenye password)

## MATATIZO YA KAWAIDA
| Tatizo | Suluhisho |
|---|---|
| `curl /health` inashindikana | `docker compose logs api` — angalia port 8080 |
| Agents hawaonekani | Hakikisha kifaa kinafika server (ping) + token sahihi |
| VPN haipati | `FUNDI_WG_ENDPOINT` = IP ya umma ya server; firewall UDP 51820 |
| AI haifanyi | Pakua model GGUF kwenye `data/models/` (jina lilipo kwenye compose) |
| Kazi hazianzi | Tab Jobs → RUHUSU (HITL) + salio la BILI la mteja |
