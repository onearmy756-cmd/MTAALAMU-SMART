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
    mtaalamu doctor                                   # ukaguzi
    mtaalamu serve                                    # HTTP API kwa UI ya hermes
"""
import argparse
import json
import sys

from . import billing, ipc, models, scope, unified
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
