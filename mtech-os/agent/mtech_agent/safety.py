"""MTECH OS — usalama na HITL (Human-In-The-Loop).

Risky zote (mfumo wa MTAALAMU SMART): hatua hatari zinahitaji kibali cha binadamu.
"""
import re
from typing import Callable, Optional

LOW = "LOW"
MEDIUM = "MEDIUM"
HIGH = "HIGH"

# Amri zisizoruhusu KABISA — hakuna kibali linazipita.
FORBIDDEN = [
    r"\brm\s+(-[a-zA-Z]*[rf][a-zA-Z]*\s+)+/(\s|$)",        # rm -rf /
    r"\bmkfs(\.\w+)?\b",                                     # format filesystem
    r"\bdd\b[^|]*of=/dev/(sd|nvme|hd|mmcblk)",               # andika disk nzima
    r":\(\)\s*\{.*\};\s*:",                                  # fork bomb
    r"\bshutdown\b|\breboot\b|\binit\s+[06]\b|\bpoweroff\b", # zima/restart OS
    r"\busermod\b.*-G\s+sudo|\bpasswd\s+root\b",
    r"\bchmod\s+(-R\s+)?777\s+/(\s|$)",
]

HIGH = [
    r"\brm\s+-[a-zA-Z]*r",                 # rm recursive
    r"\bapt(-get)?\s+(purge|remove|autoremove)\b",
    r"\bkillall\b|\bpkill\b|\bkill\s+-9\b",
    r"\bmv\b\s+/[a-zA-Z]",                 # hamisha root dirs
    r"\btruncate\b|\bshred\b",
    r"\biptables\b|\bufw\b|\bnft\b",       # firewall
    r"\buseradd\b|\buserdel\b|\bgroupadd\b",
    r"\bsystemctl\s+(disable|mask|stop)\b",
    r"\bchown\b.*\s/$",
    r"\bcrontab\b",
    r"\bcurl\b[^|]*\|\s*(ba)?sh\b",        # curl|sh
]

MEDIUM = [
    r"\bapt(-get)?\s+install\b", r"\bpip3?\s+install\b",
    r"\btar\b|\bgzip\b|\bzip\b|\bunzip\b",
    r"\bscp\b|\brsync\b|\bwget\b|\bcurl\b",
    r"\bsystemctl\s+(restart|start|enable)\b",
    r"\bkill\b",
    r"\bgit\s+push\b",
]


def classify(cmd: str) -> str:
    c = cmd.strip().lower()
    for p in FORBIDDEN:
        if re.search(p, c):
            return "FORBIDDEN"
    for p in HIGH:
        if re.search(p, c):
            return HIGH
    for p in MEDIUM:
        if re.search(p, c):
            return MEDIUM
    return LOW


class Gate:
    """HITL gate: callback inapewa (action, risk, detail) inarudisha bool."""

    def __init__(self, auto_approve: bool = False, callback: Optional[Callable[[str, str, str], bool]] = None) -> None:
        self.auto_approve = auto_approve
        self.callback = callback
        self.log: list = []

    def allow(self, action: str, risk: str, detail: str = "") -> bool:
        entry = {"action": action, "risk": risk, "detail": detail, "allowed": False}
        if risk == "FORBIDDEN":
            self.log.append(entry)
            return False
        if risk in (LOW, MEDIUM) or self.auto_approve:
            entry["allowed"] = True
            self.log.append(entry)
            return True
        if self.callback:
            entry["allowed"] = bool(self.callback(action, risk, detail))
        self.log.append(entry)
        return entry["allowed"]
