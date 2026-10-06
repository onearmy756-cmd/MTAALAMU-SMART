# pfSense — FIREWALL (SEHEMU 12, MTECH OS)

> pfSense haifanyi kazi kwenye Docker (inahitaji kernel ya yake) — inaendeshwa kama
> VM/box tofauti kwenye network, kisha MTECH OS (Rust agent) inaongea nayo kupitia
> API ya pfSense. Config hii ndiyo kanuni za firewall za mfumo.

## Ulinganifu na MTECH OS

| Service | Port | Rule |
|---|---|---|
| WireGuard (wg0) | 51820/udp | Ruhusu WAN → server (peers za mbali pekee) |
| Fundi API | 8080/tcp | Ruhusu VPN subnet (10.66.66.0/24) + LAN ya ndani PEKEE |
| Wazuh | 1514-1515/tcp | Ruhusu kutoka LAN/VPN tu |
| Suricata | — | Inaskua interface ya WAN + wg0 (mirror) |
| PXE/TFTP/DHCP | 67-69/udp | LAN ya ndani tu — KATA WAN |
| Samba (images) | 445/tcp | LAN ya ndani tu |

## Kanuni za msingi (firewall rules)

1. **Default deny** — kila kitu kinachotoka nje kinakatazwa isipokuwa orodha hii
2. **Management (8080, Wazuh UI)** — LAN/VPN tu; WAN IMEKATAZWA
3. **WireGuard** — 51820/udp WAN → API server; peers zote zina IP za 10.66.66.0/24
4. **Outbound ya agents** — kupitia wg0 (kazi zote za mbali = VPN, policy ya vpn.rs)
5. **Logging** — kila rule inalogga → Wazuh (SIEM) + Suricata (IDS alerts)

## Uunganisho wa Rust agent

`fundi-deploy/api` inaweza kupush rules kwa pfSense REST API (env: `PFSENSE_URL`,
`PFSENSE_API_KEY`) — kila peer mpya ya VPN → alias + rule ya kazi zake. (H5: wiring
kamili ya pfSense API kwenye agent.)

## Mfuatano wa usalama
Suricata (IDS) inagundua → Wazuh (SIEM) inaandika + ina-alert → Admin (MTECH OS UI)
anaona → agent (kupitia wg0) inatua kwa idhini (HITL).

## MTECH OS API integration (H5b — imekamilika)

MTECH OS (Rust agent) inaongea na pfSense kupitia **REST API** (pfSense REST API package):

| Kigezo | Thamani |
|---|---|
| Env ya agent | `PFSENSE_API_URL` (mf. `https://10.66.66.1`) + `PFSENSE_API_KEY` (client-auth key) |
| Rules | `GET/POST /api/pfsense/rules` (agent) → pfSense `/api/v1/firewall/rule` |
| Aliases | `GET/POST /api/pfsense/aliases` → pfSense `/api/v1/firewall/alias` |
| Services | `POST /api/pfsense/services` `{"name":"dnsmasq"}` → pfSense `/api/v1/services/restart` |
| Status | `GET /api/pfsense/status` → pfSense `/api/v1/status/system` |

Bila env hizo mbili, endpoints za agent zinarudisha `configured: false` na error ya
configuration — hakuna majibu ya uongo. Thamani hizo zinawekwa kwenye faili ya env ya
server (haziweki kwenye git).
