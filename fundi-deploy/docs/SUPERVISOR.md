# Msimamizi — wewe unasimamia tu

**Agent inafanya kazi zote.** Wewe unaruhusu au kughairi pekee.

## Hatua 5

1. Fungua `http://SERVER:8080/ui`
2. **Gundua computers** (agent inatumia arp-scan/arp; lab ina demo hosts)
3. Chagua targets → **Anza deploy (HITL)**
4. Jobs zilizo `awaiting_approval` → **RUHUSU** au **GHAIRI**
5. Fuatilia progress — usiguse PC mwenyewe isipokuwa hardware fail

## Agent anafanya nini (bila wewe)

| Hatua | Agent |
|-------|--------|
| Discover | LAN scan / demo hosts |
| Plan | AI (Ollama) au rules → chagua OS |
| HITL | Inasubiri **RUHUSU** yako |
| Backup | Stub (P2 full backup) |
| WOL | Magic packet |
| PXE | Andika `pxelinux.cfg/01-mac` |
| Install | Progress + report |

## Lab bila HITL

**Deploy auto** — agent inaendelea bila idhini (lab/VLAN pekee).

## Production

- VLAN ya majaribio kwanza
- Backup policy kabla ya wipe
- Images za OS kwenye `server/images/`
- Weka `FUNDI_DEMO_HOSTS=0` kwenye production (hakuna fake PCs)
