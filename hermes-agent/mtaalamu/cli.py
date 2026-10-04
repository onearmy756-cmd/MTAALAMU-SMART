"""MTAALAMU SMART — CLI moja kwa kila kitu.

    mtaalamu register me@mail.com INDIVIDUAL GOLD     # jisajili (license key)
    mtaalamu license                                  # onyesha leseni/usage
    mtaalamu ask "kompyuta inaenda polepole"          # chat/plan (HITL)
    mtaalamu solve "cleanup" --yes                    # solve kwa auto-confirm
    mtaalamu scan                                     # gundua matatizo sasa
    mtaalamu catalog                                  # ops zote
    mtaalamu shortcuts [run WIN F12]                  # shortcuts zote
    mtaalamu lang en                                  # en/sw
    mtaalamu model local tinyllama | api URL KEY_ENV  # chagua model/API
    mtaalamu wallet                                   # salio la wallet (TZS)
    mtaalamu pay 15000 --phone 255712345678           # lipa kwa ClickPesa (USSD push)
    mtaalamu pay --op disk.cleanup --phone 2557...    # lipa kiasi cha zana mahsusi
    mtaalamu admin unlock [--key XXX]                 # ⚑ ADMIN (owner ama MTECH_ADMIN_KEY)
    mtaalamu admin lock                               # zima admin
    mtaalamu admin price disk.cleanup 20000           # badilisha bei (0 = BURE)
    mtaalamu admin tier disk.partition.create GOLD    # huduma kwa GOLD+ tu
    mtaalamu admin dashboard                          # FULL SYSTEM (OS nzima)
    mtaalamu doctor                                   # ukaguzi
    mtaalamu serve                                    # HTTP API kwa UI ya hermes
"""
import argparse
import json
import sys

from . import admin, billing, clickpesa, ipc, models, scope, unified
from .catalog import CATALOG
from .shortcuts import ALL_SHORTCUTS, run_shortcut


def _p(x):
    print(json.dumps(x, ensure_ascii=False, indent=1, default=str))


def main() -> None:
    ap = argparse.ArgumentParser(prog="mtaalamu", description="MTAALAMU SMART — Unified Solver")
    sub = ap.add_subparsers(dest="cmd")

    r = sub.add_parser("register"); r.add_argument("email"); r.add_argument("plan", choices=billing.PLANS); r.add_argument("tier", choices=billing.TIERS); r.add_argument("--seats", type=int, default=None)
    sub.add_parser("license")
    a = sub.add_parser("ask"); a.add_argument("text"); a.add_argument("--lang", default=None)
    s = sub.add_parser("solve"); s.add_argument("text"); s.add_argument("--answers", default="{}"); s.add_argument("--yes", action="store_true")
    sub.add_parser("scan")
    sub.add_parser("catalog")
    sh = sub.add_parser("shortcuts"); sh.add_argument("run", nargs="?", default=None); sh.add_argument("os_", nargs="?"); sh.add_argument("key", nargs="?")
    l = sub.add_parser("lang"); l.add_argument("lang", choices=("sw", "en"))
    m = sub.add_parser("model"); m.add_argument("mode", choices=("local", "api")); m.add_argument("model"); m.add_argument("api_url", nargs="?", default=""); m.add_argument("api_key_env", nargs="?", default="")
    pay = sub.add_parser("pay"); pay.add_argument("amount", nargs="?", type=int, default=0); pay.add_argument("--op", default=""); pay.add_argument("--phone", required=True); pay.add_argument("--wait", type=int, default=90)
    ad = sub.add_parser("admin"); ad.add_argument("action", choices=("unlock", "lock", "price", "tier", "dashboard")); ad.add_argument("arg1", nargs="?", default=None); ad.add_argument("arg2", nargs="?", default=None); ad.add_argument("--key", default=None)
    sub.add_parser("wallet")
    sub.add_parser("usage"); sub.add_parser("doctor"); sub.add_parser("serve")
    args = ap.parse_args()

    if args.cmd == "register":
        try:
            lic = billing.register(args.email, args.plan, args.tier, args.seats)
            print("✅ IMEFANIKIKA! Leseni yako:")
            _p(lic)
            print("\nIweke kwenye kompyuta zote za plan yako (seats).")
        except ValueError as e:
            print("✖", e); sys.exit(2)
    elif args.cmd == "license":
        _p(billing.usage_summary())
    elif args.cmd == "ask":
        if args.lang: scope.set_lang(args.lang)
        _p(unified.solve(args.text))
    elif args.cmd == "solve":
        _p(unified.solve(args.text, answers=json.loads(args.answers), auto_confirm=args.yes))
    elif args.cmd == "scan":
        probe = unified.probe_system()
        issues = unified.detect_issues("scan diagnose matatizo yote")
        _p({"probe": probe, "suspected_ops": [i.id for i in issues]})
    elif args.cmd == "catalog":
        tier = (billing.current_license() or {}).get("tier")
        for o in CATALOG:
            price = billing.fmt_tzs(billing.price_tzs(o.id, o.risk, tier))
            print(f"  {o.id:<28} [{o.group:<9}] {o.name_sw}  ({'offline' if o.offline else 'online'}, {o.credits}cr, {o.risk}, {price})")
    elif args.cmd == "shortcuts":
        if args.run == "run" and args.os_ and args.key:
            _p(run_shortcut(args.os_, args.key))
        else:
            for s in ALL_SHORTCUTS:
                print(f"  {s['os']:<8} {s['key']:<16} {s['sw']}")
    elif args.cmd == "lang":
        scope.set_lang(args.lang); print(scope.t("language_set"))
    elif args.cmd == "model":
        models.save_choice(args.mode, args.model, args.api_url, args.api_key_env)
        _p({"ok": True, "choice": models.load_choice()})
    elif args.cmd == "wallet":
        _p({"wallet_tzs": billing.wallet_balance(), "gateway": clickpesa.status_mode()})
    elif args.cmd == "admin":
        try:
            if args.action == "unlock":
                info = admin.unlock(args.key)
                print("👑 ADMIN UMEFUNGULIWA (mode:", info["mode"] + ")")
                if info["mode"] == "owner":
                    print("   ⚠ Kwa usalama zaidi, weka MTECH_ADMIN_KEY kwenye Settings → Environment.")
            elif args.action == "lock":
                admin.lock(); print("🔒 ADMIN UMEZIMWA")
            elif args.action == "price":
                if not args.arg1 or args.arg2 is None:
                    print("✖ Tumia: mtaalamu admin price <op_id> <amount_tzs>  (0 = BURE)"); sys.exit(2)
                _p(admin.set_price(args.arg1, int(args.arg2)))
            elif args.action == "tier":
                if not args.arg1 or not args.arg2:
                    print("✖ Tumia: mtaalamu admin tier <op_id> <ANY|BASIC|BRONZE|GOLD|PLATINUM|DIAMOND>"); sys.exit(2)
                _p(admin.set_tool_tier(args.arg1, args.arg2))
            elif args.action == "dashboard":
                d = admin.dashboard(probe=unified.probe_system())
                print(f"════ FULL SYSTEM — MTAALAMU SMART (ADMIN: {d['admin'].get('mode', '?')}) ════")
                o = d["os"]; print(f"OS: {o.get('os') or o.get('family')} {o.get('release','')} ({o.get('machine','')}) @ {o.get('node','')}")
                l = d["license"]; print(f"Leseni: {l.get('email','—')} · {l.get('plan','—')} {l.get('tier','—')} · seats {l.get('seats','—')}")
                print(f"Wallet: {billing.fmt_tzs(d['wallet_tzs'])} · Malipo yote: {billing.fmt_tzs(d['payments_total_tzs'])} · Gateway: {d['gateway']}")
                print(f"Catalog: {d['catalog_ops']} ops · BURE {d['free_ops']} · zenye bei {d['priced_ops']}")
                if d["price_overrides"]:
                    print("Bei maalum (admin):", ", ".join(f"{k}={v['price']}" for k, v in d["price_overrides"].items()))
                if d["tier_overrides"]:
                    print("Tier za huduma:", ", ".join(f"{k}→{v}+" for k, v in d["tier_overrides"].items()))
                print("— Snapshot ya OS —"); print(o.get("snapshot", "")[:400])
                if args.arg1 == "--json":
                    _p(d)
        except (PermissionError, ValueError) as e:
            print("✖", e); sys.exit(2)
    elif args.cmd == "pay":
        amount = args.amount
        if args.op and not amount:
            op = __import__(".catalog", fromlist=["CATALOG_BY_ID"]).CATALOG_BY_ID.get(args.op)
            if not op:
                print(f"✖ op haijulikani: {args.op}"); sys.exit(2)
            amount = billing.price_tzs(op.id, op.risk, (billing.current_license() or {}).get("tier"))
        if amount <= 0:
            print("✖ Kiasi: mtaalamu pay 15000 --phone 255712345678"); sys.exit(2)
        try:
            print(f"📲 Tuma USSD push: {billing.fmt_tzs(amount)} → {clickpesa.normalize_phone(args.phone)} ({clickpesa.status_mode()})...")
            out = clickpesa.collect(amount, args.phone, timeout_s=args.wait)
        except (RuntimeError, ValueError) as e:
            print(f"✖ {e}"); sys.exit(2)
        if out.get("ok"):
            w = billing.wallet_topup(out["amount_tzs"], source="clickpesa", ref=out.get("ref", ""),
                                     tx=out.get("tx", ""), channel=out.get("channel", ""))
            print(f"✅ MALIPO YAMEKAMILIKA ({out.get('channel')} — ref {out.get('ref')})")
            print(f"   Wallet: {billing.fmt_tzs(w['balance_tzs'])}")
        else:
            print(f"✖ Malipo hayakukamilika: {out.get('status')} (ref {out.get('ref')})"); sys.exit(1)
    elif args.cmd == "usage":
        _p(billing.usage_summary())
    elif args.cmd == "doctor":
        from .doctor_lite import run_checks
        _p(run_checks())
    elif args.cmd == "serve":
        from .server import main as serve_main
        serve_main()
    else:
        ap.print_help()


if __name__ == "__main__":
    main()
