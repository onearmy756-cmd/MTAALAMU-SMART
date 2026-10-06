# MTECH OS — ORODHA YA BEI (TZS)
**Mbilinyi Tech · Umiliki ni wako, leseni ni yako, faida ni yako**
*Bei hizi ziko ndani ya mfumo (billing.rs) — dashboard na API zinazalisha kiotomatiki.*

---

## 1. SUBSCRIPTION (kwa mwezi, kwa kifaa) — huduma ZOTE bila malipo ya ziada

| Kundi | Vifaa | Bei/kifaa/mwezi |
|---|---|---|
| **BASIC** | 1 – 10 | **TZS 8,000** |
| **STANDARD** | 11 – 50 | **TZS 6,500** |
| **BIASHARA** | 51 – 200 | **TZS 5,000** |
| **KAMPANI KUBWA** | 200+ | **TZS 4,000** |

Subscription inashughulikia: uchunguzi wa kila siku, afya ya kompyuta, AI chat,
kurekebisha (baada ya idhini yako), forensics, ripoti za mtandaoni, agents,
sambaza programu — kila kitu kwenye dashboards.

**Mifano ya mapato (mapato ya kudumu kila mwezi):**
- Shule/kampuni na PC 20 → 20 × 6,500 = **TZS 130,000/mwezi**
- Biashara na PC 60 → 60 × 5,000 = **TZS 300,000/mwezi**
- Kampuni kubwa PC 250 → 250 × 4,000 = **TZS 1,000,000/mwezi**

## 2. PAY-PER-USE (bila subscription) — kila kazi ina bei yake

| Huduma | Bei |
|---|---|
| Uchunguzi wa Afya / Scan | TZS 2,000 |
| Kichanganuzi cha Mtandao | TZS 2,000 |
| Uchanganuzi wa Viviruski | TZS 2,000 |
| Usasishaji wa Drivers | TZS 2,000 |
| Usakinishaji wa Programu (bundle) | TZS 1,500 |
| Usimamizi wa Vifaa vya Mtandao | TZS 4,000 |
| Ripoti Rasmi (PDF) | TZS 5,000 |
| Usakinishaji wa Mfumo (OS) | TZS 5,000 |
| Kurekebisha Tatizo | TZS 15,000 |
| Uchunguzi wa Kidijitali (Forensics) | TZS 25,000 |

## 3. PUNGUZO LA VOLUME (pay-per-use)
- Kazi **10+**: **−10%**
- Kazi **50+**: **−20%**

## 4. KANUNI ZA BILI (zalishwa ndani ya mfumo)
- Subscription hai = **hakuna malipo ya ziada** kwa huduma zote.
- Pay-per-use: **malipo yanakatwa BAADA ya kazi kufanikiwa tu**.
- Kazi kubwa (repair, forensics, OS install) zinaomba **IDHINI YAKO (HITL)** kwanza.
- Kila shilingi inaandikwa kwenye **ledger** — unaweza kuonyesha mteja alipacho.
- Salio linaingizwa kupitia ClickPesa/benki (rejea halisi LAZIMA).

## 5. WAPI BEI ZINAONEKANA
- Dashboard → tab **💰 BILI** (bei zote live kutoka `/api/billing/prices`)
- Ripoti ya BILI (PDF) kwa mteja — tab **📄 RIPOTI**
