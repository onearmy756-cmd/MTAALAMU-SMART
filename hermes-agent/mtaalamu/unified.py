"""MTAALAMU SMART — Unified Solver engine.

Mzunguko KAMILI (moja kwa moja, HITL kila hatua hatari):
  1. detect   — system probe (vision/proc) + ulinganifu na catalog
  2. plan     — op inayofaa + maswali ya chaguo (mteja anachagua kwenye fomu)
  3. gate     — billing (license + credits) + offline/online check
  4. execute  — amri HALISI za OS husika
  5. verify   — thibitisha matokeo (probe upya)
  6. learn    — Kitabu Kidigitali (ripoti + maarifa, inajifunza)
"""
import json
import os
import platform
import re
import subprocess
import time
from pathlib import Path

from . import admin, billing, clickpesa, models, scope
from .catalog import CATALOG, CATALOG_BY_ID, build_command, ops_for_os

import sys as _sys
FAMILY = "windows" if os.name == "nt" else ("macos" if _sys.platform == "darwin" else "linux")
BOOK = Path(os.environ.get("MTECH_BOOK", Path.home() / ".mtaalamu" / "digital_book.jsonl"))
KNOW = Path(os.environ.get("MTECH_KNOW", Path.home() / ".mtaalamu" / "knowledge.jsonl"))


# ------------------------------------------------------------------ probe
def probe_system() -> dict:
    """Uchunguzi HALISI wa OS (processes, disks, memory, network) — offline."""
    info = {"os": platform.system(), "release": platform.release(), "machine": platform.machine(),
            "node": platform.node()}
    cmds = {
        "linux": "uname -r; df -h / | tail -1; free -m | head -2; ip -br addr 2>/dev/null | head -5",
        "windows": ("powershell -NoProfile -Command "
                    "\"Get-PSDrive C | Select Used,Free | Format-Table; "
                    "Get-CimInstance Win32_OperatingSystem | Select TotalVisibleMemorySize,FreePhysicalMemory | Format-Table\""),
        "macos": "df -h / | tail -1; vm_stat | head -3; ifconfig en0 | grep 'inet '",
    }
    try:
        p = subprocess.run(cmds[FAMILY], shell=True, capture_output=True, text=True, timeout=25)
        info["snapshot"] = (p.stdout or "").strip()[:1200]
    except (OSError, subprocess.TimeoutExpired) as e:
        info["snapshot"] = f"(probe: {e})"
    return info


_GENERIC = {"tengeneza", "fanya", "rekebisha", "naomba", "tafadhali", "nataka", "mpya", "new",
            "make", "fix", "help", "please", "sasa", "kwa", "jina", "name", "disk", "list",
            "zangu", "yangu", "hii", "the", "and", "for", "my", "nisaidie"}


def detect_issues(text: str = "") -> list:
    """Match maneno ya mteja + probe → ops za catalog.
    Verbs/jina za jumla hazipewi uzito — nouns maalum (partition, wifi, user...) ndizo
    zinazoamua; kama neno la op id limetajwa wazi, lina bonus kubwa."""
    blob = (text or "").lower()
    hits = []
    for op in ops_for_os(FAMILY):
        hay = f"{op.name_sw} {op.name_en} {op.id} {op.group}".lower()
        score = sum(2 for w in re.findall(r"[a-z]{3,}", blob) if w in hay and w not in _GENERIC)
        # bonus: action verbs maalum zinaonyesha op (create≠list, update≠show)
        pair = {
            "disk.partition.create": ("create", "tengeneza"),
            "disk.partition.list": ("onyesha", "show", "list", "orodhesha"),
            "driver.update": ("update", "updating", "sasi"),
            "user.create": ("create", "tengeneza"),
            "app.install": ("install", "sakinisha"),
            "disk.cleanup": ("cleanup", "safisha", "clean", "safisha", "punguza"),
        }
        verbs = pair.get(op.id, ())
        if any(v in blob for v in verbs):
            score += 6
        # punguza ops za jumla kama verbs za op nyingine zimetajwa wazi
        if op.id.endswith(".list") and any(v in blob for vs in pair.values() for v in vs if v not in ("onyesha", "show", "list", "orodhesha")):
            score -= 3
        # bonus kama op id yenyewe imetajwa wazi
        if op.id in blob:
            score += 10
        # PARTITION HEURISTIC: "partition" + verb ya create/list inaamua op kikamilifu
        if "partition" in blob:
            is_create = any(v in blob for v in ("create", "tengeneza"))
            is_list = any(v in blob for v in ("onyesha", "show", "orodhesha"))
            if op.id == "disk.partition.create" and is_create:
                score += 8
            if op.id == "disk.partition.list" and is_list and not is_create:
                score += 8
        # OFFICE HEURISTIC: "office" (au word/excel/outlook) → office.diagnose
        if op.id == "office.diagnose" and any(w in blob for w in ("office", "word", "excel", "outlook", "powerpoint")):
            score += 8
        if score:
            hits.append((score, op))
    hits.sort(key=lambda x: -x[0])
    return [op for _, op in hits[:6]]


# ------------------------------------------------------------------ HITL questions
def questions_for(op) -> list:
    """Fomu ya maswali (mteja anajaza + anachagua options)."""
    qs = {
        "disk.partition.create": [
            {"k": "disk", "q_sw": "Disk ipi? (mf: 0 au /dev/sda)", "q_en": "Which disk? (e.g. 0 or /dev/sda)", "options": None},
            {"k": "size", "q_sw": "Size ya partition? (mf: 50GB)", "q_en": "Partition size? (e.g. 50GB)", "options": ["10GB", "50GB", "100GB", "200GB"]},
        ],
        "app.install": [{"k": "app", "q_sw": "Jina la app?", "q_en": "App name?", "options": ["vlc", "gimp", "libreoffice", "7zip", "spotify"]}],
        "app.uninstall": [{"k": "app", "q_sw": "App ya kuondoa?", "q_en": "App to remove?", "options": None}],
        "user.create": [
            {"k": "user", "q_sw": "Jina la user?", "q_en": "Username?", "options": None},
            {"k": "admin", "q_sw": "Admin?", "q_en": "Admin?", "options": ["ndio", "hapana"]},
        ],
        "user.password.change": [{"k": "user", "q_sw": "User gani?", "q_en": "Which user?", "options": None}],
        "sys.process.kill": [{"k": "target", "q_sw": "PID au jina la process?", "q_en": "PID or process name?", "options": None}],
        "file.create.folder": [{"k": "path", "q_sw": "Njia ya folder?", "q_en": "Folder path?", "options": None}],
        "file.delete": [{"k": "path", "q_sw": "Njia ya kufuta?", "q_en": "Path to delete?", "options": None}],
        "office.repair": [{"k": "mode", "q_sw": "Aina ya repair?", "q_en": "Repair type?", "options": ["Quick", "Online"]}],
        "av.scan": [{"k": "scope", "q_sw": "Scan gani?", "q_en": "Which scan?", "options": ["quick", "full"]}],
    }
    base = list(qs.get(op.id, []))
    if op.ask:
        base.insert(0, {"k": "confirm", "q_sw": op.ask[0], "q_en": op.ask[1], "options": ["ndio", "hapana"]})
    return base


# ------------------------------------------------------------------ execute
def _run(cmds: list) -> list:
    out = []
    for c in cmds:
        try:
            p = subprocess.run(c, shell=True, capture_output=True, text=True, timeout=180)
            out.append({"cmd": c[:160], "code": p.returncode,
                        "out": (p.stdout or "")[:800], "err": (p.stderr or "")[:300]})
        except (OSError, subprocess.TimeoutExpired) as e:
            out.append({"cmd": c[:160], "code": -1, "out": "", "err": str(e)[:200]})
    return out


def execute(op, answers: dict) -> dict:
    """Tekeleza op HALISI (baada ya HITL + billing + MALIPO KABLA YA ZANA).

    PAY-BEFORE-USE (kulinda kutoibiwa): salio la wallet LINAKATWA KABLA
    ya amri yoyote kuendeshwa. Wallet inaongezeka tu kwa malipo halisi ya
    ClickPesa (USSD push + webhook iliyothibitishwa).
    """
    if not billing.is_active():
        return {"ok": False, "stage": "gate", "msg": scope.t("not_licensed")}
    if not billing.charge(op.id, op.credits):
        return {"ok": False, "stage": "gate", "msg": scope.t("no_credits")}
    tier = (billing.current_license() or {}).get("tier")
    is_admin = admin.is_admin()
    # TIER GATE: huduma yenye min-tier (admin anaweka) — admin anapita
    required = billing.tool_tier(op.id)
    if not is_admin and not billing.tier_allows(required, tier):
        return {"ok": False, "stage": "tier", "op": op.id, "min_tier": required,
                "your_tier": tier,
                "msg": f"🔒 Zana hii inahitaji tier {required}+ (wako: {tier or 'hakuna'}) — panda tier au muulize admin"}
    # MALIPO YA KILA ZANA (TZS 5,000 → milioni) — KABLA YA EXECUTE; ADMIN = BURE
    price = billing.price_tzs(op.id, op.risk, tier)
    if is_admin:
        billing.record_admin_use(op.id)
        pay_note = f"👑 ADMIN — {op.id} ni BURE (hakuna malipo; audit imehifadhiwa)"
    else:
        try:
            billing.wallet_spend(op.id, price, tier)
        except ValueError as e:
            return {"ok": False, "stage": "payment", "payment_required": True,
                    "op": op.id, "price_tzs": price, "price": billing.fmt_tzs(price),
                    "wallet_tzs": billing.wallet_balance(), "msg": f"💳 {e}",
                    "how": ("Lipa:  mtaalamu pay " + str(price) + " --phone 255712345678"
                            + "  (au POST /api/pay)  — USSD prompt itafika kwenye simu yako")}
        pay_note = f"💳 Malipo: {billing.fmt_tzs(price)} (zana: {op.id}) — wallet: {billing.fmt_tzs(billing.wallet_balance())}"
    if not op.offline:
        # online check halisi (ping 1.1.1.1 — bila kubadilisha mfumo)
        online = subprocess.run("ping -c 1 -W 2 1.1.1.1" if FAMILY != "windows" else "ping -n 1 -w 2000 1.1.1.1",
                                shell=True, capture_output=True).returncode == 0
        if not online:
            return {"ok": False, "stage": "network", "msg": scope.t("offline_needed")}

    scope.t("executing")
    cmds = build_command(op, FAMILY, answers)
    results = _run(cmds)

    # verify: probe upya (tofauti = tatizo limebadilika)
    after = probe_system()
    ok = all(r["code"] == 0 for r in results) if results else False
    return {"ok": ok, "stage": "done", "op": op.id, "results": results, "after": after,
            "payment": pay_note, "price_tzs": price}


# ------------------------------------------------------------------ digital book / learning
def record_case(problem: str, op_id: str, answers: dict, results: dict, diagnosis: str = "") -> dict:
    """Kitabu Kidigitali: kila tatizo + suluhisho → maarifa (inajifunza)."""
    case = {"ts": int(time.time()), "os": FAMILY, "problem": problem[:500],
            "op": op_id, "answers": answers, "ok": results.get("ok"),
            "diagnosis": diagnosis[:500],
            "results": [{k: r[k] for k in ("cmd", "code")} for r in results.get("results", [])]}
    for p in (BOOK, KNOW):
        p.parent.mkdir(parents=True, exist_ok=True)
        with p.open("a") as f:
            f.write(json.dumps(case, ensure_ascii=False) + "\n")
    return case


def learn_lookup(problem: str) -> str | None:
    """Kumbukumbu: kama tatizo kama hili limesululiwa awali, rudisha suluhisho."""
    if not KNOW.exists():
        return None
    words = set(re.findall(r"[a-z]{4,}", problem.lower()))
    best, best_score = None, 0
    for line in KNOW.read_text().splitlines()[-300:]:
        try:
            case = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not case.get("ok"):
            continue
        hay = set(re.findall(r"[a-z]{4,}", (case.get("problem", "")).lower()))
        score = len(words & hay)
        if score > best_score:
            best, best_score = case, score
    if best and best_score >= 2:
        return f"👉 [kumbukumbu] Tatizo kama hili lilisululiwa awali kwa '{best['op']}' — {best.get('diagnosis','')[:200]}"
    return None


# ------------------------------------------------------------------ main solve loop
def solve(text: str, answers: dict | None = None, auto_confirm: bool = False) -> dict:
    """Mzunguko kamili: scope → detect → plan → gate → execute → learn."""
    # 0) lugha (auto-detect; default sw)
    lang = scope.detect_lang(text)
    scope.set_lang(lang)

    # 1) scope guard (computer-solutions tu)
    if not scope.in_scope(text):
        return {"stage": "denied", "msg": scope.t("denied_topic")}

    # 2) memory-first (kitabu kidigitali)
    memo = learn_lookup(text)

    # 3) detect (probe + catalog)
    issues = detect_issues(text)
    probe = probe_system()

    if answers is None:
        # awamu ya 1: mpango + maswali (UI inaonyesha fomu)
        op = issues[0] if issues else None
        tier = (billing.current_license() or {}).get("tier")
        price = billing.price_tzs(op.id, op.risk, tier) if op else 0
        is_adm = admin.is_admin()
        need = bool(op) and not is_adm and billing.wallet_balance() < price
        plan = {
            "stage": "plan", "lang": scope.LANG, "op": op.id if op else None,
            "op_name": (op.name_sw if op else None), "credits": op.credits if op else 0,
            "price_tzs": price, "price": billing.fmt_tzs(price) if op else "TZS 0",
            "payment_required": need,
            "admin_free": is_adm, "min_tier": billing.tool_tier(op.id) if op else None,
            "wallet_tzs": billing.wallet_balance(),
            "gateway": clickpesa.status_mode(),
            "offline": op.offline if op else True,
            "questions": questions_for(op) if op else [],
            "alternatives": [i.id for i in issues[1:4]],
            "probe": probe, "memory": memo,
            "thinking": models.thinking_summary(text) if op else "",
        }
        return plan

    # awamu ya 2: mteja amejaza/chagua → execute
    op = CATALOG_BY_ID.get(answers.get("op") or (issues[0].id if issues else ""))
    if not op:
        return {"stage": "denied", "msg": scope.t("denied_topic")}
    confirm = str(answers.get("confirm", "ndio")).lower() in ("ndio", "yes", "y", "1")
    if not auto_confirm and not confirm:
        return {"stage": "cancelled", "msg": "IMEKATALIWA na mteja (HITL)."}

    res = execute(op, answers)
    case = record_case(text, op.id, answers, res)
    return {"stage": "done", "lang": scope.LANG, "op": op.id, "result": res,
            "case": {"ts": case["ts"]}, "memory": memo,
            "message": scope.t("done") if res.get("ok") else "⚠️ Angalia matokeo hapa chini."}
