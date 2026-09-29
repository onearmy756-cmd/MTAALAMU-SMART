# ============================================================
# i18n — LUGHA MBILI: Kiswahili (sw) / English (en)
#
# Vyanzo vya tafsiri:
#   1. data/locales/sw.json  na  data/locales/en.json  (shared, kama ilivyo awali)
#   2. STR hapa chini — misimu ya UI wa dashboard (zilizokuwa zimeandikwa
#      moja kwenye React; sasa zote zina sw + en)
#
# Matumizi:   tr("tabs.live", "sw")   ->  "MFUATILIO HALISI"
#             tl("nav.home", "en")    ->  "Home"  (kutoka locale JSON)
#             bi(list(sw="...", en="..."), "sw") -> lugha ya data ya JSON
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || length(a) == 0) b else a

# --- mistari ya UI (kila kitu na sw + en) --------------------
STR <- list(
  # --- header ---
  "brand.title"       = list(sw = "MTAALAMU SMART", en = "MTAALAMU SMART"),
  "brand.sub"         = list(sw = "MFUATILIO WA MUDA HALISI v2.6.0", en = "AGENT VISION LIVE \u2022 REALTIME MONITOR v2.6.0"),
  "hdr.online"        = list(sw = "MFUMO UMEWASHWA", en = "SYSTEM ONLINE"),
  "hdr.secure"        = list(sw = "USALAMA:<br/>SALAMA", en = "SECURITY:<br/>SECURE"),
  "hdr.meta"          = list(sw = "\u2022 KIPENGO: ALPHA-7 \u2022 Hali: MEWASHWA", en = "\u2022 NODE: ALPHA-7 \u2022 STATUS: ONLINE"),
  "lang.pick"         = list(sw = "LUGHA / LANGUAGE", en = "LANGUAGE / LUGHA"),

  # --- tabs ---
  "tabs.live"         = list(sw = "\u25c9 MFUATILIO HALISI", en = "\u25c9 LIVE MONITOR"),
  "tabs.formula"      = list(sw = "\u25e6 INJINI YA FORMULA", en = "\u25e6 FORMULA ENGINE"),
  "tabs.diag"         = list(sw = "\u25cf UTAMBUZI", en = "\u25cf DIAGNOSIS"),
  "tabs.viz"          = list(sw = "\u25a0 UONESHAJI", en = "\u25a0 VISUALIZATION"),

  # --- LIVE: device map ---
  "live.map.title"    = list(sw = "\u2316 RAMANI YA VIFAA", en = "\u2316 DEVICE MAP"),
  "live.map.meta"     = list(sw = "Vipengele vilivyounganishwa \u2022 Vifaa 4", en = "Connected Nodes \u2022 4 Devices"),
  "live.load"         = list(sw = "Mzigo", en = "Load"),
  "live.online"       = list(sw = "HAPOKAGULIWA", en = "ONLINE"),
  "live.warning"      = list(sw = "TAHADHARI", en = "WARNING"),
  "live.critical"     = list(sw = "HATARI", en = "CRITICAL"),
  "live.latency"      = list(sw = "Ucheleweshaji", en = "Latency"),
  "live.health"       = list(sw = "Afya", en = "Health"),
  "live.optimal"      = list(sw = "NZURI", en = "OPTIMAL"),
  "live.degraded"     = list(sw = "IMEHARIBIKA", en = "DEGRADED"),

  # --- LIVE: processes ---
  "live.proc.title"   = list(sw = "\u2630 MCHAKATO", en = "\u2630 PROCESSES"),
  "live.proc.meta"    = list(sw = "Haziendeshi \u2022 {n} zinaendeshwa", en = "Active \u2022 {n} Running"),
  "live.proc.alert"   = list(sw = "\u26a0 Mwendo wa kutiliwa shaka umeonekana kwenye {n}", en = "\u26a0 Suspicious activity detected on {n}"),
  "live.proc.pid"     = list(sw = "PID", en = "PID"),

  # --- LIVE: topology ---
  "live.net.title"    = list(sw = "\u2611 MUUNO WA MTANDAO", en = "\u2611 NETWORK TOPOLOGY"),
  "live.net.meta"     = list(sw = "Mesh \u2022 Vipengele 4", en = "Mesh \u2022 4 Nodes"),
  "live.net.packets"  = list(sw = "VIPAKETI", en = "PACKETS"),
  "live.net.stable"   = list(sw = "Muunganisho wote waimara", en = "All connections stable"),

  # --- LIVE: issues ---
  "live.iss.title"    = list(sw = "\u26a0 MATATIZO NA TAHADHARI", en = "\u26a0 ISSUES & WARNINGS"),
  "live.iss.meta"     = list(sw = "{n} matatizo \u2022 {c} hatari", en = "{n} Issues \u2022 {c} Critical"),
  "live.sev.crit"     = list(sw = "\u25b2 HATARI", en = "\u25b2 CRITICAL"),
  "live.sev.warn"     = list(sw = "\u25cf TAHADHARI", en = "\u25cf WARNING"),
  "live.sev.info"     = list(sw = "TAARIFA", en = "INFO"),
  "live.iss.i1"       = list(sw = "KIWANGO CHA DISK KINA HATARI", en = "DISK SPACE CRITICAL"),
  "live.iss.i1d"      = list(sw = "Matumizi ya diski yamefikia {v}%", en = "Disk usage at {v}%"),
  "live.iss.i2"       = list(sw = "MATUMIZI YA RAM YA JUU", en = "HIGH RAM USAGE"),
  "live.iss.i2d"      = list(sw = "Mzigo wa RAM {v}% umepita kikomo", en = "RAM load {v}% exceeded threshold"),
  "live.iss.i3"       = list(sw = "MCHAKATO USIOJULIKANA", en = "UNKNOWN PROCESS DETECTED"),
  "live.iss.i3d"      = list(sw = "unknown.exe ina tabia ya kawaida", en = "unknown.exe behavior anomalous"),
  "live.iss.i4"       = list(sw = "MUUNGANISHO USIO THABITI", en = "INTERMITTENT CONNECTION"),
  "live.iss.i4d"      = list(sw = "Ongezeko la ucheleweshaji kwenye kipengele cha PC ({v}ms)", en = "Latency spike on PC node ({v}ms)"),

  # --- FORMULA ---
  "f.title"           = list(sw = "\u25e6 INJINI YA FORMULA", en = "\u25e6 FORMULA ENGINE"),
  "f.meta"            = list(sw = "Inasomwa kutoka JSON \u2022 {n} formulas \u2022 viwanda 7", en = "Data-driven \u2022 {n} formulas \u2022 7 trades"),
  "f.search"          = list(sw = "\u1f50d Tafuta formula...", en = "\u1f50d Search formula..."),
  "f.all"             = list(sw = "ZOTE", en = "ALL"),
  "f.empty"           = list(sw = "Hakuna formula inayolingana na utafutaji wako.", en = "No formula matches your search."),
  "f.none"            = list(sw = "Hakuna formula", en = "No formula"),
  "f.results"         = list(sw = "MATOKEO / RESULTS", en = "RESULTS / MATOKEO"),
  "f.error"           = list(sw = "MAKOSA", en = "ERROR"),
  "f.id"              = list(sw = "ID", en = "ID"),
  "f.loading"         = list(sw = "Inapakia formulas.json \u2026", en = "Loading formulas.json \u2026"),
  "f.hint"            = list(sw = "badilisha vipimo upande wa kushoto", en = "change inputs on the left"),

  # --- DIAGNOSIS ---
  "d.title"           = list(sw = "\u25cf Uchunguzi wa Bayes", en = "\u25cf BAYESIAN DIAGNOSIS"),
  "d.meta"            = list(sw = "{n} models \u2022 P(Cause|Symptoms) \u221d P(S|C) \u00d7 P(C)", en = "{n} models \u2022 P(Cause|Symptoms) \u221d P(S|C) \u00d7 P(C)"),
  "d.model"           = list(sw = "MFUMO", en = "MODEL"),
  "d.symptoms"        = list(sw = "DALILI ZILIZOONEKANA", en = "SYMPTOMS DETECTED"),
  "d.clear"           = list(sw = "\u2715 Ondoa zote", en = "\u2715 Clear all"),
  "d.causes"          = list(sw = "UWEZEKANO WA SABABU", en = "PROBABILITY OF CAUSES"),
  "d.fix"             = list(sw = "SULUHISHO", en = "FIX"),
  "d.cost"            = list(sw = "GHARAMA ~TZS", en = "COST ~TZS"),
  "d.bayes"           = list(sw = "Inatumia Bayes theorem \u2014 hesabu halisi, si makisio ya AI.", en = "Uses true Bayesian math, not AI guesses."),
  "d.loading"         = list(sw = "Inapakia diagnosis.json \u2026", en = "Loading diagnosis.json \u2026"),

  # --- VIZ ---
  "v.kpi.title"       = list(sw = "\u25c6 LEO \u2014 KPI ZA SIKU", en = "\u25c6 TODAY \u2014 DAILY KPI"),
  "v.kpi.meta"        = list(sw = "Dashibodi \u2022 Kila siku inasasishwa", en = "Dashboard \u2022 Updated daily"),
  "v.bar.title"       = list(sw = "\u25a0 MAPATO KWA MKOA", en = "\u25a0 REVENUE BY REGION"),
  "v.bar.meta"        = list(sw = "Bar Chart \u2022 Milioni TZS", en = "Bar Chart \u2022 Millions TZS"),
  "v.pie.title"       = list(sw = "\u25e8 AINA ZA KAZI", en = "\u25e8 WORKLOAD TYPES"),
  "v.pie.meta"        = list(sw = "Pie Chart \u2022 Jumla kazi 20,245", en = "Pie Chart \u2022 Total 20,245 jobs"),
  "v.line.title"      = list(sw = "\u25b2 UKUAJI WA MAPATO", en = "\u25b2 REVENUE GROWTH"),
  "v.line.meta"       = list(sw = "Line Graph \u2022 Miezi 12 (Milioni TZS)", en = "Line Graph \u2022 12 Months (Millions TZS)"),
  "v.top.title"       = list(sw = "\u2605 TOP 5 MAFUNDI", en = "\u2605 TOP 5 TECHNICIANS"),
  "v.top.meta"        = list(sw = "Orodha ya wakuu \u2022 Mapato", en = "Leaderboard \u2022 Revenue"),
  "v.map.title"       = list(sw = "\u25c7 WATAALAMU KWA MIKOA", en = "\u25c7 EXPERTS BY REGION"),
  "v.map.meta"        = list(sw = "Ramani \u2022 Mikoa \u2022 {n} wataalamu", en = "Map \u2022 Regions \u2022 {n} experts"),
  "v.net.title"       = list(sw = "\u25c9 MTANDAO WA SMART TECHNICIAN", en = "\u25c9 SMART TECHNICIAN NETWORK"),
  "v.net.meta"        = list(sw = "Mchoro wa mtandao \u2022 Makuu makuu \u2192 Mikoa \u2192 Wateja", en = "Network Diagram \u2022 HQ \u2192 Regions \u2192 Customers"),
  "v.dev.title"       = list(sw = "\u2699 MFUMO WA KIFAA \u2014 COMPONENTS", en = "\u2699 DEVICE SYSTEM \u2014 COMPONENTS"),
  "v.dev.meta"        = list(sw = "Mfumo wa kifaa \u2022 Miunganisho yote", en = "Device System \u2022 All connections"),
  "v.flow.title"      = list(sw = "\u25c8 WAKALA 5 \u2014 MTIRIRIKO", en = "\u25c8 5 AGENT WORKFLOW"),
  "v.flow.meta"       = list(sw = "Mchoro \u2022 Mteja \u2192 Malipo", en = "Flowchart \u2022 Customer \u2192 Payment"),
  "v.cust.title"      = list(sw = "\u25a1 DASHBOARD YA MTEJA", en = "\u25a1 CUSTOMER DASHBOARD"),
  "v.cust.meta"       = list(sw = "Hatua za kesi \u2022 Maendeleo", en = "Case progress \u2022 Steps"),
  "v.heat.title"      = list(sw = "\u25a5 HEATMAP \u2014 MAHALI PANAPOTAJIKA", en = "\u25a5 HEATMAP \u2014 HOTSPOTS"),
  "v.heat.meta"       = list(sw = "Kiwango cha mtaa \u2022 Msongamano wa mahitaji", en = "Street level \u2022 Demand density"),
  "v.tgt.title"       = list(sw = "\u25ce MALANGO YA MWAKA 1", en = "\u25ce YEAR-1 TARGETS"),
  "v.tgt.meta"        = list(sw = "Malengo \u2022 Maendeleo", en = "Targets \u2022 Progress"),
  "v.tl.title"        = list(sw = "\u25a4 RATIBA YA MWAKA 1", en = "\u25a4 YEAR-1 TIMELINE"),
  "v.tl.meta"         = list(sw = "Ratiba \u2022 Vigoo 12", en = "Timeline \u2022 Milestones 12"),

  # --- VIZ: ndani ---
  "v.kpi.jobs"        = list(sw = "KAZI ZILIZOFANYIKA", en = "JOBS DONE"),
  "v.kpi.jobs.sub"    = list(sw = "kazi za leo", en = "jobs today"),
  "v.kpi.rev"         = list(sw = "MAPATO LEO", en = "REVENUE TODAY"),
  "v.kpi.rev.sub"     = list(sw = "mapato ya leo", en = "today's revenue"),
  "v.kpi.cust"        = list(sw = "WATEJA LEO", en = "CUSTOMERS TODAY"),
  "v.kpi.cust.sub"    = list(sw = "wateja waliopata msaada", en = "customers served"),
  "v.sum.cust"        = list(sw = "Wateja waliosaidika", en = "Customers served"),
  "v.sum.tech"        = list(sw = "Wataalamu wanaotumia", en = "Active experts"),
  "v.sum.reg"         = list(sw = "Mikoa iliyofunikwa", en = "Regions covered"),
  "v.sum.rev"         = list(sw = "Mapato (MWAKA 1)", en = "Revenue (YEAR 1)"),
  "v.sum.target"      = list(sw = "{v} lengo", en = "{v} target"),
  "v.pie.total"       = list(sw = "JUMLA KAZI", en = "TOTAL JOBS"),
  "v.map.tz"          = list(sw = "\u1f1f9\u1f1ff TANZANIA \u2014 WATAALAMU KWA MIKOA", en = "\u1f1f9\u1f1ff TANZANIA \u2014 EXPERTS BY REGION"),
  "v.heat.demand"     = list(sw = "\u25b2 MAHITAJI YA WATEJA \u2014 DAR ES SALAAM", en = "\u25b2 CUSTOMER DEMAND \u2014 DAR ES SALAAM"),
  "v.cust.issue"      = list(sw = "TATIZO LA MTEJA", en = "CUSTOMER CASE"),
  "v.cust.case"       = list(sw = "Kesi \u2022 Maendeleo", en = "Case \u2022 Progress"),
  "v.cust.q"          = list(sw = "\"Simu yangu haifunguki \u2014 imenyezwa moto juu sana\"", en = "\"My phone won't turn on \u2014 it overheats badly\""),
  "v.cust.stage"      = list(sw = "Hatua:", en = "Stage:"),
  "v.cust.diagnose"   = list(sw = "Inachunguza (Diagnose)", en = "Diagnosing"),
  "v.cust.left"       = list(sw = "Muda uliobaki:", en = "Time left:"),
  "v.cust.min"        = list(sw = "dak", en = "min"),
  "v.cust.s1"         = list(sw = "Umetuma tatizo", en = "Problem sent"),
  "v.cust.s2"         = list(sw = "Mwakala anachunguza", en = "Agent is diagnosing"),
  "v.cust.s3"         = list(sw = "Mwakala anatoa suluhisho", en = "Agent provides solution"),
  "v.cust.s4"         = list(sw = "Mteja anafuata", en = "Customer follows"),
  "v.cust.s5"         = list(sw = "Kazi kukamilika", en = "Job completed"),
  "v.tt.jobs"         = list(sw = "Kazi", en = "Jobs"),
  "v.flow.n1"         = list(sw = "Mteja anaandika tatizo", en = "Customer writes problem"),
  "v.flow.n2"         = list(sw = "Husikiliza + kuratibu", en = "Listens + routes"),
  "v.flow.n3"         = list(sw = "Uchunguzi wa Bayes", en = "Bayes inference"),
  "v.flow.n4"         = list(sw = "Sheria/serikali", en = "Gov / Law route"),
  "v.flow.n5"         = list(sw = "Hesabu", en = "Calculations"),
  "v.flow.n6"         = list(sw = "Vipengele vya WhatsApp", en = "WhatsApp steps"),
  "v.flow.n7"         = list(sw = "Thibitisha + kadiria", en = "Confirm + rate"),
  "v.flow.n8"         = list(sw = "Escrow \u2192 Fundi 80% / Wewe 20%", en = "Escrow \u2192 Fundi 80% / You 20%"),
  "v.flow.n9"         = list(sw = "Muafaka \u2014 Kazi imekamilika", en = "Agreed \u2014 Job completed"),
  "v.flow.mteja"      = list(sw = "MTEJA", en = "CUSTOMER"),
  "v.flow.reception"  = list(sw = "A1 MAPOKEZI", en = "A1 RECEPTION"),
  "v.flow.diagnose"   = list(sw = "A2 UCHUNGUZI", en = "A2 DIAGNOSE"),
  "v.flow.sheria"     = list(sw = "A2b SHERIA", en = "A2b LAW"),
  "v.flow.formula"    = list(sw = "A3 FORMULA", en = "A3 FORMULA"),
  "v.flow.action"     = list(sw = "A4 TENDO", en = "A4 ACTION"),
  "v.flow.verify"     = list(sw = "A5 THIBITISHO", en = "A5 VERIFY"),
  "v.flow.pay"        = list(sw = "MALIPO", en = "PAYMENT"),
  "v.flow.rating"     = list(sw = "KADIRIA 5/5 \u2605", en = "RATING 5/5 \u2605"),

  # --- STATUS BAR ---
  "sb.uptime"         = list(sw = "MUDA WA KUENDA:", en = "SYSTEM UPTIME:"),
  "sb.cpu"            = list(sw = "JOTO LA CPU", en = "CPU TEMP"),
  "sb.gpu"            = list(sw = "JOTO LA GPU", en = "GPU TEMP"),
  "sb.bw"             = list(sw = "UPANA WA BENDI", en = "BANDWIDTH"),
  "sb.logging"        = list(sw = "KUANDIKA KUMEWASHWA", en = "LOGGING: ACTIVE"),
  "sb.firewall"       = list(sw = "LEVYA IMEWASHWA", en = "FIREWALL: ENABLED"),

  # --- trade (viwanda) ---
  "trade.all"         = list(sw = "ZOTE", en = "ALL"),
  "trade.umeme"       = list(sw = "UMEME", en = "ELECTRICAL"),
  "trade.solar"       = list(sw = "SOLAR", en = "SOLAR"),
  "trade.ujenzi"      = list(sw = "UJENZI", en = "CONSTRUCTION"),
  "trade.maji"        = list(sw = "MAJI", en = "WATER"),
  "trade.computer"    = list(sw = "COMPUTER", en = "COMPUTER"),
  "trade.telecoms"    = list(sw = "TELECOMS", en = "TELECOMS"),
  "trade.network"     = list(sw = "NETWORK", en = "NETWORK"),

  # --- RAMANI (MAP) — tabs.map ---
  "tabs.map"          = list(sw = "\u25c8 RAMANI", en = "\u25c8 MAP"),
  "map.title"         = list(sw = "\u25c8 RAMANI HALISI — TANZANIA", en = "\u25c8 LIVE MAP — TANZANIA"),
  "map.meta"          = list(sw = "Google Sat \u2022 NASA \u2022 Esri \u2022 OSRM \u2022 mikoa 30",
                             en = "Google Sat \u2022 NASA \u2022 Esri \u2022 OSRM \u2022 30 regions"),
  "map.base"          = list(sw = "PICHA ZA MSINGI (TILES)", en = "BASE IMAGERY (TILES)"),
  "map.overlays"      = list(sw = "TABAKA LA RAMANI", en = "MAP LAYERS"),
  "map.ov.boundaries" = list(sw = "Mipaka ya mikoa", en = "Region boundaries"),
  "map.ov.choropleth" = list(sw = "Choropleth \u2022 kazi kwa mkoa", en = "Choropleth \u2022 jobs by region"),
  "map.ov.techs"      = list(sw = "Wataalamu \u2022 GPS", en = "Technicians \u2022 GPS"),
  "map.ov.customers"  = list(sw = "Wateja \u2022 GPS", en = "Customers \u2022 GPS"),
  "map.ov.jobs"       = list(sw = "Kazi zinazoendelea", en = "Active job sites"),
  "map.ov.corridors"  = list(sw = "Njia kuu za magari (OSRM)", en = "Main driving routes (OSRM)"),
  "map.ov.labels"     = list(sw = "Lebo za mahali (Esri)", en = "Place labels (Esri)"),
  "map.route"         = list(sw = "\u25b6 NJIA YA MAGARI (OSRM)", en = "\u25b6 DRIVING ROUTE (OSRM)"),
  "map.route.from"    = list(sw = "Kutoka", en = "From"),
  "map.route.to"      = list(sw = "Kwenda", en = "To"),
  "map.route.go"      = list(sw = "PATA NJIA", en = "GET ROUTE"),
  "map.route.wait"    = list(sw = "Inahesabu njia\u2026", en = "Calculating route\u2026"),
  "map.route.err"     = list(sw = "Njia haipatikani \u2022 angalia internet/OSRM", en = "Route unavailable \u2022 check internet/OSRM"),
  "map.legend"        = list(sw = "MAELEKEZO", en = "LEGEND"),
  "map.leg.tech"      = list(sw = "Mtaalamu", en = "Technician"),
  "map.leg.cust"      = list(sw = "Mteja", en = "Customer"),
  "map.leg.job"       = list(sw = "Kazi", en = "Job site"),
  "map.leg.scale"     = list(sw = "Kazi nyingi \u2192", en = "More jobs \u2192"),
  "map.lic"           = list(sw = "LESENI YA MATUMIZI & MACHANZO", en = "LICENSE OF USE & SOURCES"),
  "map.lic.src"       = list(sw = "Picha za ramani", en = "Map imagery"),
  "map.lic.route"     = list(sw = "Njia (routing)", en = "Routing"),
  "map.lic.privacy"   = list(sw = "Faragha: taarifa za watu binafsi zinaonekana tu kwa idhini (Data Protection Act 2022).",
                             en = "Privacy: personal data is shown only with consent (Data Protection Act 2022)."),
  "map.hint"          = list(sw = "Kaza picha kubwa, zungusha ramani, bonyeza alama kujua zaidi.",
                             en = "Zoom, pan and click a marker for details."),
  # --- sehemu za popup ---
  "jobs"              = list(sw = "Kazi", en = "Jobs"),
  "techs"             = list(sw = "Wataalamu", en = "Technicians"),
  "customers"         = list(sw = "Wateja", en = "Customers"),
  "place"             = list(sw = "Mahali", en = "Place"),
  "status"            = list(sw = "Hali", en = "Status"),
  "rating"            = list(sw = "Kadirio", en = "Rating"),
  "env"               = list(sw = "Mazingira ya kufanyia kazi", en = "Work environment"),
  "tools"             = list(sw = "Vifaa/mahitaji", en = "Tools/requirements"),
  "route_wait"        = list(sw = "Inahesabu njia\u2026", en = "Calculating route\u2026"),
  "route_err"         = list(sw = "Njia haipatikani", en = "Route unavailable"),
  "st.online"         = list(sw = "MTANDAONI", en = "ONLINE"),
  "st.busy"           = list(sw = "ANA KAZI", en = "BUSY"),
  "st.offline"        = list(sw = "HAYOPO", en = "OFFLINE"),
  "st.open"           = list(sw = "OTEPO LESENI", en = "REQUEST OPEN"),
  "st.urgent"         = list(sw = "DHARURA", en = "URGENT"),
  "st.in_progress"    = list(sw = "INAENDELEA", en = "IN PROGRESS"),
  "st.pending"        = list(sw = "INASUBIRI", en = "PENDING")
)

tr <- function(key, lang) {
  x <- STR[[key]]
  if (is.null(x)) return(key)
  x[[lang]] %||% x[["en"]] %||% key
}

# --- lokali JSON (data/locales/*.json) -----------------------
PACKS <- new.env(parent = emptyenv())

load_packs <- function() {
  if (!is.null(PACKS$loaded)) return(invisible(TRUE))
  for (lg in c("sw", "en")) {
    p <- find_path("data", "locales", paste0(lg, ".json"))
    PACKS[[lg]] <- if (is.null(p)) NULL else jsonlite::fromJSON(p, simplifyVector = FALSE)
  }
  PACKS$loaded <- TRUE
  invisible(TRUE)
}

# lookup kwenye JSON: tl("nav.home", "en")
tl <- function(path, lang) {
  load_packs()
  root <- PACKS[[lang]]
  if (is.null(root)) return(NULL)
  keys <- strsplit(path, ".", fixed = TRUE)[[1]]
  dig <- function(node, ks) {
    for (k in ks) {
      if (is.null(node)) return(NULL)
      node <- node[[k]]
    }
    if (is.list(node)) NULL else node
  }
  v <- dig(root, keys)
  if (is.null(v) && !is.null(root$translations)) v <- dig(root$translations, keys)
  v
}

# tafsiri ya data ya JSON yenye uwanja sw/en
bi <- function(x, lang) {
  if (is.null(x)) return("")
  if (is.list(x)) {
    v <- x[[lang]]
    if (is.null(v)) v <- x[["en"]] %||% x[["sw"]]
    return(v)
  }
  x
}

# jimiza {n} ndani ya tafsiri
fill <- function(x, ...) {
  for (nm in names(list(...))) x <- gsub(paste0("{", nm, "}"), list(...)[[nm]], x, fixed = TRUE)
  x
}

find_path <- function(...) {
  rel <- file.path(...)
  roots <- c(".", if (exists("APP_DIR", inherits = TRUE)) get("APP_DIR", inherits = TRUE))
  cands <- character(0)
  for (r in roots) {
    cands <- c(cands, file.path(r, rel), file.path(r, "..", rel), file.path(r, "..", "..", rel))
  }
  cands <- unique(c(rel, cands))
  for (p in cands) if (file.exists(p)) return(normalizePath(p))
  NULL
}
