# MTAALAMU SMART — DATA SCHEMA CONTRACT (canonical)
Kila data file lazima iwe **UTF-8, JSON halisi** (thibitisha kwa `node -e "JSON.parse(require('fs').readFileSync('FILE','utf8'))"`).
Kanuni: **JSON ndio pekee ya interop**; code haitoi business content (formulas, bei, rules, copy).

## 0. Kanuni za pamoja
- Status vocabulary: `GOOD | WARNING | FAIL` (pia `OK` kwa information-only).
- Grammar ya `check`/`expr` (shared Rust↔JS): `+ - * / ^ %`, mzunguko, functions `sqrt pow ceil floor abs log exp min max if`, `pi`.
  Boolean: `&& || !`, comparisons `< <= > >= == !=`, `true`, `false`.
  Variables = input names + output names (outputs computed before rules; first matching rule wins).
- Kila formula **lazima iwe na `test`** (inputs + expect) kwa ajili ya test runner.
- Maandishi yote ya UI: `{"sw": "...", "en": "..."}` (Kiswahili kwanza).
- Batisha kila file: `node -e "JSON.parse(...)"` kabla ya kuwaambia umekamilika.

## 1. `data/formulas/<trade>.json` (file MOJA kwa kila trade — jumla **132 formulas / 20 trades**)
```json
{
  "trade": "umeme",
  "label": {"sw": "Umeme", "en": "Electrical"},
  "icon": "⚡", "color": "#FF6B00",
  "formulas": [
    {
      "id": "voltage_drop",
      "trade": "umeme",
      "name": {"sw": "Kushuka kwa Voltage", "en": "Voltage Drop"},
      "formula": "Vd = (2 × L × I × ρ) / A",
      "description": {"sw": "...", "en": "..."},
      "inputs": [
        {"name":"L","label":{"sw":"Urefu","en":"Length"},"unit":"m","default":30,"min":0.1,"max":1000},
        {"name":"rho","label":{"sw":"Resistivity","en":"Resistivity"},"unit":"Ω·mm²/m","default":0.0175,
         "select":[{"label":"Copper (0.0175)","value":0.0175},{"label":"Aluminum (0.028)","value":0.028}]}
      ],
      "outputs": [
        {"name":"vd","label":{"sw":"Kushuka","en":"Drop"},"expr":"(2 * L * I * rho) / A","unit":"V","digits":2},
        {"name":"vd_pct","label":{"sw":"Kushuka %","en":"Drop %"},"expr":"((2 * L * I * rho) / A / V) * 100","unit":"%","digits":2}
      ],
      "steps": [
        {"label":"Formula","text":"Vd = (2 × L × I × ρ) / A"},
        {"label":"Weka namba","expr":"(2 * L * I * rho) / A","template":"Vd = {{result}} V"}
      ],
      "rules": [
        {"check":"vd_pct < 3","status":"GOOD","msg":{"sw":"Cable inatosha","en":"Cable is adequate"}},
        {"check":"vd_pct < 5","status":"WARNING","msg":{"sw":"Ongeza cable","en":"Increase cable"}},
        {"check":"true","status":"FAIL","msg":{"sw":"Cable ndogo sana!","en":"Cable too small!"}}
      ],
      "standards": {"sw": ["IEC 60364-5-52", "BS 7671:2018"], "en": ["IEC 60364-5-52", "BS 7671:2018"]},
      "test": {"inputs": {"L":30,"I":16,"A":2.5,"V":230,"rho":0.0175},
               "expect": {"vd": 6.72, "vd_pct": 2.92, "status": "GOOD"}}
    }
  ]
}
```
**Idadi halisi kwa kila trade (lazima zilingane):** umeme 10, solar 7, maji 8, gari 10, hvac 10, ujenzi 10,
useremala 8, welding 8, cctv 5, computer 8, simu 5, electronics 6, pump 5, jenereta 5, gas 4, rangi 5,
gate_motor 4, borehole 5, appliance 4, tailor 5 = **132**.
Vipengele muhimu vinavyolazimika kwa kila formula (kutoka SMART SYSTEM 1): ifadhi (e.g. TANESCO VAT 18% +
TZS 5,000 + EWURA 1%; breaker snap 6/10/16/20/25/32/40/50/63 A; blocks +5% waste; DoD 0.5/0.8; BTU TZS ×700;
VAT/currency TZS). Zote ziwe kwenye `rules` au `standards`, si kwenye code.

## 2. `data/formulas_network/<kundiXX>_<name>.json` (jumla **285 formulas / kundi 20**)
Schemа kama §1, isipokuwa kila formula ina `"group": "signal"` badala ya trade, na kila file:
```json
{"group":{"id":"signal","code":"KUNDI 1","label":{"sw":"Signal & Power","en":"Signal & Power"}},
 "formulas":[{"id":"db_from_power","group":"signal", ...}]}
```
Idadi kwa kundi (N-101…N-120): signal 15, antenna 20, propagation 15, link_budget 15, modulation 15,
traffic 15, ran 15, fiber 20, satellite 15, nr5g 15, subnetting 20, bandwidth 15, latency 10, routing 15,
qos 10, wireless 15, security 10, monitoring 10, virtualization 10, cloud 10 = **285**.

## 3. `data/trades.json`
`{"trades":[{"id","name_sw","name_en","icon","color","skills":[{"name","formula","difficulty":1-5,"tools":[]}],"common_problems":[]}]}`
— trades **21** (Computer, Umeme, Mabomba, Mechanic, AC, Electronics, Useremala, Ujenzi, Welding, CCTV,
Solar, Pikipiki, Jenereta, Rangi, Roofing, Gas, Water Pump, Tailor, Appliance, Borehole, Gate Motor).

## 4. `data/problems.json` (seed ya production — ongeza kwa JSON tu)
`{"problems":[{"id","trade","description","symptoms":[],"causes":{"overload":0.3,...},"solution",
"formula","time_min","cost_tzs","success_rate","severity","cases"}]}` — jumla ≥ **400** problems
zilizogawanywa kwa trades zote; causes = probabilities (jumla ~1.0).

## 5. `data/diagnosis.json` (Bayesian — thibitisha thamani halisi)
```json
{"models": {
  "electrical": {"priors": {"overload":0.30,"short_circuit":0.20,"earth_leak":0.15,"loose_wire":0.15,"bad_breaker":0.10,"bad_switch":0.10},
    "symptoms": ["breaker_trips","sparks","shock","no_power","flicker","sound"],
    "likelihood": {"breaker_trips":[0.90,0.85,0.20,0.30,0.95,0.10], "sparks":[0.10,0.90,0.30,0.60,0.20,0.70],
                   "shock":[0,0.10,0.95,0.60,0,0.20], "no_power":[0.40,0.80,0.30,0.70,0.90,0.60],
                   "flicker":[0.30,0.20,0.10,0.80,0.40,0.85], "sound":[0.20,0.30,0.10,0.40,0.70,0.50]},
    "causes": ["overload","short_circuit","earth_leak","loose_wire","bad_breaker","bad_switch"],
    "fixes": {"overload": {"sw":"...","en":"...","cost_tzs":35000,"time_min":20}, ...}},
  "mechanic": {"symptoms":["haianzi","moshi_mweusi","moshi_mweupe","moshi_bluu","overheating","sauti","check_engine"],
               "causes":["battery","starter","fuel_pump","injectors","air_filter","head_gasket","piston_rings","radiator","thermostat"],
               "likelihood": {7×9 probabilities zote}, "priors": {...}},
  "computer": {...}
}}
```
**Lazima itumike thamani halisi kutoka SMART SYSTEM 1** (mechanic 7×9 = probabilities 63).

## 6. `data/decision_trees/car_tree.json` na `phone_tree.json`
`{"nodes":[{"id":0,"question":"Gari inawaka?","yes":1,"no":2},{"id":...,"result":{"diagnosis","solution","confidence","actions":["..."],"cost_tzs"}}]}`
— car tree: nodes **13** (R-214, maswali yote ya Kiswahili yaliotajwa); phone tree: nodes **7** (N-159, result iwe na bei TZS 80K–150K / 100K–300K).

## 7. `data/rules/rules_electrical.json`
`{"rules":[{"id","name_sw","priority","conditions":[{"field","operator":"gt|gte|lt|lte|eq","value"}],
"action":{"action_type":"recommend|warn|reject","message","solution","confidence"}}]}` — VOLTAGE_DROP_HIGH (>5 reject), WARNING (>3 && ≤5 warn), GOOD (≤3 recommend), BREAKER_UNDERSIZED.

## 8. `data/professions.json` — **104 professions / kundi 10**
`{"categories":[{"code","name_sw","icon","color","professions":[{"code","name_sw","name_en","icon","education","license","services":[],"avg_fee_tzs","consultation_time_min","per_hour"}]}]}`
Kundi: afya 12, sheria 10, fedha 9, uhandisi 11, elimu 10, teknolojia 12, biashara 10, sanaa 12, mali 6, nyingine 12 = **104**.
Bei halisi (mf. daktari 50,000/30min, mwanahabari... — N-170).

## 9. `data/services.json` — catalog ya huduma zote
`{"services":[{"code","name":{"sw","en"},"category":"computer|phone|remote|government|recovery|training|platform",
"price_tzs","duration_min","remote_pct","description":{"sw","en"}}]}`
Lazima ujumuisha: **30 computer** + **25 phone** + huduma **7 mpya** (email, government, hacked, data recovery,
backup, security, training) + **remote 12** + **pay-as-you-use 8** (password_reset 30000, email_setup 25000,
virus_removal 50000, windows_install 120000, data_recovery 80000, frp_bypass 60000, network_setup 70000,
security_setup 45000).

## 10. `data/pricing.json` — PRICING BIBLE (chanzi pekee ya bei zote)
```json
{"currency":"TZS","vat_rate":0.18,"service_charge_tzs":5000,"ewura_levy_rate":0.01,
 "commission":{"job_rate":0.10,"range":[0.10,0.15]},
 "escrow":{"fee_rate":0.08,"legacy":[0.05,0.08],"fundi_share":0.92},
 "subscriptions":[{"code":"solo","price_tzs":0,...},{"code":"personal","price_tzs":20000,"days":30},
                  {"code":"pro","price_tzs":20000,"months":1},{"code":"premium","price_tzs":50000},
                  {"code":"business","price_tzs":200000},{"code":"enterprise","price_tzs":1200000}],
 "materials":{"cement_bag":30000,"sand_m3":50000,"gravel_m3":60000,"steel_kg":3000},
 "hardware":{...FUNDI BOX BOM 710000, Pi 250000, USB 150000...},
 "products":{...fundi deploy/rescue/box/map...},
 "payment_methods":["M-Pesa","Tigo Pesa","Airtel Money","Halopesa","Bank"]}
```

## 11. `data/locales/sw.json` na `en.json` (LanguagePack)
`{"language":"sw","name":"Kiswahili","flag":"🇹🇿","translations":{"app":{},"nav":{},"buttons":{},"status":{},
"hardware":{},"services":{},"dashboard":{},"license":{}}}`
— vikundi vyote vinahitajika kwa lugha zote mbili (J-71); keys ni pamoja na dashboard copy, errors, HITL, wallet.

## 12. `data/config.json`
`{"system":{"name":"MTAALAMU SMART","version":"1.0.0","language":"sw","offline_mode":true},
"ai":{"model":"qwen2.5:3b","learning_rate":0.05,"bayesian_enabled":true,"fuzzy_enabled":true},
"database":{"type":"sqlite","path":"./data/smart_technician.db"},
"payment":{"method":"M-Pesa","commission":0.10,"escrow_enabled":true},
"notifications":{"sms":true,"whatsapp":true,"email":false}}` (R-231)

## 13. `data/agent_data.json` (Agent Vision — V-67…V-70)
`{"components":[{"id","name","icon","status":"good|warning|critical","x","y"}],
"processes":[{"name","cpu","ram","status"}],
"topology":{"nodes":[{"id","label","x","y","status":"online|warning|offline"}],"edges":[{"from","to"}]},
"issues":[{"title","desc","action"}]}`

## 14. `data/schema/*.sql` (SQLite)
`trades.sql` (trades, skills, problems, solutions, cases, learning — R-222) · `hitl.sql`
(hitl_requests + indexes 3 — N-148…N-150) · `escrow.sql` (clients, technicians, jobs, escrow,
transactions, disputes — D-118…D-119) · `payasyouuse.sql` (customers, wallets, topups,
service_usage, usage_log — N-278…N-282).
