# Patch keys — source baada ya i18n.R au ungeze kwenye STR
# (app.R inaongeza keys hizi baada ya source i18n)

AV_STR <- list(
  "tabs.agentic"      = list(sw = "◉ AGENTIC VISION", en = "◉ AGENTIC VISION"),
  "av.qa.title"       = list(sw = "◌ ULIZA / AUTOMATIC WORK", en = "◌ ASK / AUTOMATIC WORK"),
  "av.qa.meta"        = list(sw = "Andika tatizo · Agent inaanza ndani ya sekunde", en = "Describe problem · Agent starts in seconds"),
  "av.qa.label"       = list(sw = "TATIZO LAKO", en = "YOUR PROBLEM"),
  "av.qa.placeholder" = list(sw = "Mfano: Kompyuta inaenda polepole na ina joto kali...", en = "Example: Computer is slow and overheating..."),
  "av.btn.start"      = list(sw = "▶ ANZA UTATUZI", en = "▶ START SOLVE"),
  "av.btn.approve"    = list(sw = "✓ RUHUSU (HITL)", en = "✓ APPROVE (HITL)"),
  "av.btn.scan"       = list(sw = "⟲ SCAN UPYA", en = "⟲ RESCAN"),
  "av.btn.voice"      = list(sw = "🔊 SAUTI", en = "🔊 VOICE"),
  "av.agents.title"   = list(sw = "MULTI-AGENT 10", en = "MULTI-AGENT 10"),
  "av.agents.meta"    = list(sw = "Orchestrator · pipeline · HITL gates", en = "Orchestrator · pipeline · HITL gates"),
  "av.pipe.title"     = list(sw = "PIPELINE PIITVD", en = "PIPELINE PIITVD"),
  "av.pipe.meta"      = list(sw = "Panga → Tambua → Tekeleza → Jaribu → Thibitisha → Andika", en = "Plan → Identify → Implement → Test → Verify → Document"),
  "av.map.title"      = list(sw = "RAMANI YA KIFAA", en = "DEVICE MAP"),
  "av.map.meta"       = list(sw = "Sehemu zote labelled · status real-time", en = "All parts labelled · real-time status"),
  "av.proc.title"     = list(sw = "MICHAKATO (LIVE)", en = "PROCESSES (LIVE)"),
  "av.proc.meta"      = list(sw = "CPU · RAM · hali", en = "CPU · RAM · status"),
  "av.topo.title"     = list(sw = "MIUNGANISHO / TOPOLOGY", en = "CONNECTIONS / TOPOLOGY"),
  "av.topo.meta"      = list(sw = "Input → Output · nodes live", en = "Input → Output · live nodes"),
  "av.iss.title"      = list(sw = "MATATIZO YALIYOGUNDULIWA", en = "DETECTED ISSUES"),
  "av.iss.meta"       = list(sw = "Automatic work · maelezo + kitendo", en = "Automatic work · detail + action"),
  "av.bus.title"      = list(sw = "SYSTEM BUS", en = "SYSTEM BUS"),
  "av.bus.meta"       = list(sw = "Njia za data · power · memory · storage", en = "Data paths · power · memory · storage"),
  "av.report.title"   = list(sw = "RIPOTI / KITABU KIDIGITALI", en = "REPORT / DIGITAL BOOK"),
  "av.report.meta"    = list(sw = "Tatizo → njia → suluhisho → tarehe", en = "Problem → path → solution → date"),
  "av.no_proc"        = list(sw = "Hakuna michakato", en = "No processes"),
  "av.no_issues"      = list(sw = "Hakuna matatizo yaliyorekodiwa sasa", en = "No issues recorded right now"),
  "av.session.ready"  = list(sw = "Tayari kupokea tatizo", en = "Ready for a problem"),
  "av.session.running"= list(sw = "Inafanya kazi...", en = "Working..."),
  "av.session.hitl"   = list(sw = "Inasubiri ruhusa yako (HITL)", en = "Waiting for your approval (HITL)"),
  "av.session.done"   = list(sw = "Imemaliza — angalia ripoti", en = "Done — see report")
)

# merge into STR if exists
if (exists("STR")) {
  for (nm in names(AV_STR)) STR[[nm]] <<- AV_STR[[nm]]
}
