"""MTAALAMU SMART — mlinzi wa mada + lugha (sw default, en switch).

KANUNI: mfumo unajibu maswali ya SOLUTIONS ZA COMPUTER TU (hardware, software,
network, Office, drivers, files, security...). Maswali mengine yanakataliwa kwa
ujumbe wa kiswahili/english — hakuna hallucination nje ya mada.
"""
import re

# Maneno ya mada halali (en + sw) — kila neno lenye \b word-boundary
# ("ip" isiweze kufanana na "recipe"; "app" isiweze "apple" n.k.)
def _w(*words):
    return r"\b(?:" + "|".join(words) + r")\b"

TOPIC_WORDS = re.compile(
    _w("computer","pc","laptop","software","hardware","disk","partition","ssd","hdd","ram","memory","cpu",
       "network","wifi","internet","dns","ip","ethernet","router","printer","driver","windows","linux",
       "ubuntu","macos","office","word","excel","outlook","powerpoint","antivirus","virus","malware",
       "password","user","account","app","apps","install","update","upgrade","boot","startup",
       "taskmanager","device","manager","battery","screen","keyboard","mouse","usb","bluetooth","firewall","backup",
       "file","folder","email","browser","slow","crash","freeze","error","bsod",
       "kompyuta","tarakilishi","simu","mtandao","netiweki","diski","kumbukumbu","prosesi",
       "programu","sakinisha","sakinisha","futa","faili","nenosiri","msimamizi","rekebisha",
       "tatizo","matatizo","suluhisho","haraka","imezima","haiwaki","inabana",
       "breaker","umeme","solar","betri","chaji","charging","skrini","firmware","bios","uefi",
       "format","recover","leseni","okoa","safisha","clean","defrag","scan","monitor","process","service","huduma",
       "onyesha","show","partitions","drivers","updates","files","shortcuts","taskmanager","defragment","antivirus"
    ), re.I,
)

DENY_SW = ("Samahani — MTAALAMU SMART anajibu maswali ya SOLUTIONS ZA COMPUTER TU "
           "(kompyuta, simu, mtandao, programu, hardware, software, Office, drivers, "
           "usalama, files...). Uliza tatizo la computer nakusaidia mara moja! 🖥️")
DENY_EN = ("Sorry — MTAALAMU SMART only answers computer-solution questions "
           "(PC, phone, network, software, hardware, Office, drivers, security, "
           "files...). Ask any computer problem and I'll solve it! 🖥️")


def in_scope(text: str) -> bool:
    return bool(TOPIC_WORDS.search(text or ""))


# ------------------------------------------------------------------ i18n
LANG = "sw"   # default Kiswahili

T = {
    "sw": {
        "greeting": "Karibu MTAALAMU SMART! 🖥️ Andika tatizo la computer — nalisolve mpaka mwisho (HITL: nauiliza kibali kwa hatua hatari).",
        "offline_needed": "⚠️ Tatizo hili linahitaji MTANDAO — tafadhali washa internet kisha endelea.",
        "asking": "❓ Nauliza (HITL):",
        "executing": "▶ Ninatekeleza:",
        "done": "✅ TATIZO LIMEKAMILIKA.",
        "verify": "🔍 Nathibitisha matokeo…",
        "report_saved": "📚 Ripoti imehifadhiwa kwenye Kitabu Kidigitali (knowledge).",
        "not_licensed": "🔒 Lisensi inahitajika — jisajili: mtaalamu register <email> <plan>",
        "no_credits": "🔒 Credits zimeisha (plan yako) — sasisha subscription: mtaalamu upgrade <tier>",
        "denied_topic": DENY_SW,
        "diagnose_start": "🔍 Nachanganua mfumo (agentic vision + system probe)…",
        "found_issues": "Nimegundua matatizo yafuatayo:",
        "hardware_note": "🖥️ HARDWARE: tatizo hili la sehemu (hardware) — nimeandika ripoti na mapendekezo; marekebisho ya sehemu yanahitaji fundi.",
        "language_set": "Lugha: Kiswahili 🇹🇿",
    },
    "en": {
        "greeting": "Welcome to MTAALAMU SMART! 🖥️ Describe any computer problem — I solve it end-to-end (HITL: I ask consent for risky steps).",
        "offline_needed": "⚠️ This fix needs INTERNET — please connect, then continue.",
        "asking": "❓ Question (HITL):",
        "executing": "▶ Executing:",
        "done": "✅ PROBLEM SOLVED.",
        "verify": "🔍 Verifying results…",
        "report_saved": "📚 Report saved to the Digital Book (knowledge).",
        "not_licensed": "🔒 License required — register: mtaalamu register <email> <plan>",
        "no_credits": "🔒 Out of credits for your plan — upgrade: mtaalamu upgrade <tier>",
        "denied_topic": DENY_EN,
        "diagnose_start": "🔍 Scanning the system (agentic vision + system probe)…",
        "found_issues": "I detected these issues:",
        "hardware_note": "🖥️ HARDWARE: this is a physical-part problem — report + recommendations saved; part fixes need a technician.",
        "language_set": "Language: English 🇬🇧",
    },
}


def t(key: str) -> str:
    return T[LANG].get(key, T["sw"].get(key, key))


def set_lang(lang: str) -> None:
    global LANG
    if (lang or "").lower() in ("en", "english", "ingereza"):
        LANG = "en"
    else:
        LANG = "sw"


def detect_lang(text: str) -> str:
    """Mteja akitumia English kwa mwingi, jibu English; vinginevyo Kiswahili (default)."""
    en_words = len(re.findall(r"\b(the|and|please|fix|problem|computer|error|install|help|my)\b", (text or "").lower()))
    sw_words = len(re.findall(r"\b(tafadhali|tatizo|kompyuta|rekebisha|nisaidie|simu|tafuta|napenda|asante)\b", (text or "").lower()))
    return "en" if en_words > sw_words else "sw"
