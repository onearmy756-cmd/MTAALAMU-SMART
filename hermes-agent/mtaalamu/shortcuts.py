"""MTAALAMU SMART — SHORTCUTS ZOTE za Windows/macOS/Ubuntu/Linux.

Kila shortcut ina: OS, key-combo, kazi halisi (op ya catalog / console),
maelezo sw/en. Mtumiaji anabonyeza (F1/F2/F12, Ctrl+Alt+T...) → mfumo
unafungua/unatekeleza; agent inasolve offline.
"""
from . import unified

ALL_SHORTCUTS = [
    # ---------- WINDOWS ----------
    {"os": "windows", "key": "F1", "sw": "Msaada wa MTAALAMU (offline diagnose)", "en": "MTAALAMU help (offline diagnose)", "action": "help"},
    {"os": "windows", "key": "F2", "sw": "Scan ya antivirus (Defender)", "en": "Antivirus scan (Defender)", "action": "op:av.scan"},
    {"os": "windows", "key": "F12", "sw": "Boot menu / diagnostics ya mfumo", "en": "Boot menu / system diagnostics", "action": "op:hw.scan"},
    {"os": "windows", "key": "Ctrl+Shift+Esc", "sw": "Task Manager (processes)", "en": "Task Manager (processes)", "action": "op:sys.taskmanager"},
    {"os": "windows", "key": "Win+X", "sw": "Menyu ya admin (Device/Disk Manager)", "en": "Admin menu (Device/Disk Manager)", "action": "op:sys.devmgr"},
    {"os": "windows", "key": "Win+R", "sw": "Run (amri yoyote)", "en": "Run (any command)", "action": "run:cmd"},
    {"os": "windows", "key": "Win+I", "sw": "Settings", "en": "Settings", "action": "run:ms-settings:"},
    # ---------- macOS ----------
    {"os": "macos", "key": "Cmd+Space", "sw": "Spotlight (tafuta kila kitu)", "en": "Spotlight (search everything)", "action": "run:spotlight"},
    {"os": "macos", "key": "Cmd+Opt+Esc", "sw": "Force quit apps (task manager)", "en": "Force quit apps (task manager)", "action": "op:sys.taskmanager"},
    {"os": "macos", "key": "Cmd+Opt+R", "sw": "Recovery (internet recovery)", "en": "Recovery (internet recovery)", "action": "help"},
    {"os": "macos", "key": "F12", "sw": "Diagnostics ya Apple", "en": "Apple Diagnostics", "action": "op:hw.scan"},
    # ---------- UBUNTU / LINUX ----------
    {"os": "linux", "key": "F1", "sw": "Msaada wa MTAALAMU", "en": "MTAALAMU help", "action": "help"},
    {"os": "linux", "key": "F2", "sw": "Scan ya usalama", "en": "Security scan", "action": "op:av.scan"},
    {"os": "linux", "key": "F12", "sw": "Boot menu / firmware", "en": "Boot menu / firmware", "action": "op:hw.scan"},
    {"os": "linux", "key": "Ctrl+Alt+T", "sw": "Terminal", "en": "Terminal", "action": "run:terminal"},
    {"os": "linux", "key": "Ctrl+Alt+Del", "sw": "Task manager / logout", "en": "Task manager / logout", "action": "op:sys.taskmanager"},
    {"os": "linux", "key": "Super+A", "sw": "Apps zote", "en": "All apps", "action": "run:appgrid"},
]

# consoles halisi za kila OS (command prompt zote)
CONSOLES = {
    "windows": ["cmd.exe", "powershell", "wt"],
    "linux": ["xfce4-terminal", "gnome-terminal", "konsole", "xterm"],
    "macos": ["Terminal", "iTerm"],
}


def run_shortcut(os_name: str, key: str) -> dict:
    """Tekeleza shortcut HALISI: op ya catalog au fungua console/run."""
    import subprocess
    for s in ALL_SHORTCUTS:
        if s["os"] == os_name and s["key"].lower() == (key or "").lower():
            action = s["action"]
            if action.startswith("op:"):
                from .catalog import CATALOG_BY_ID
                op = CATALOG_BY_ID.get(action[3:])
                if op:
                    plan = unified.solve(op.name_en)
                    return {"ok": True, "shortcut": key, "plan": plan}
            if action == "run:cmd":
                subprocess.Popen(["cmd.exe"], start_new_session=True)
                return {"ok": True, "opened": "cmd.exe"}
            if action == "run:terminal":
                for term in CONSOLES["linux"]:
                    try:
                        subprocess.Popen([term], start_new_session=True)
                        return {"ok": True, "opened": term}
                    except OSError:
                        continue
            if action == "run:spotlight":
                subprocess.Popen(["osascript", "-e", 'tell application "System Events" to keystroke " " using command down'])
                return {"ok": True, "opened": "Spotlight"}
            if action == "run:ms-settings:":
                subprocess.Popen(["start", "ms-settings:"], shell=True)
                return {"ok": True, "opened": "Settings"}
            if action == "run:appgrid":
                subprocess.Popen(["gtk-launch", "org.gnome.Software"], start_new_session=True)
                return {"ok": True, "opened": "appgrid"}
            # help / diagnostics
            plan = unified.solve("diagnose mfumo mzima hardware na software")
            return {"ok": True, "shortcut": key, "plan": plan}
    return {"ok": False, "error": f"shortcut haipo: {os_name} {key}"}
