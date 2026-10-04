"""MTECH OS — registry ya zana (tools) za agent.

Zana hizi ndizo 'mikono' ya Qwen 2.5 VL: shell, files, probe ya kernel,
vision (macho), control (mikono juu ya GUI), kernel events, na akili ya
MTAALAMU SMART (skills 146).
"""
import ast
import json
import os
import platform
import shutil
import subprocess
import time
from pathlib import Path

from .config import CONFIG, SYSTEM_PROMPT
from .safety import Gate, classify

MAX_OUT = 6000


def _clip(text: str) -> str:
    text = str(text)
    return text if len(text) <= MAX_OUT else text[:MAX_OUT] + f"\n...[imekatwa {len(text) - MAX_OUT} chars]"


# ---------------------------------------------------------------- shell/files
def shell_exec(args: dict, gate: Gate) -> dict:
    cmd = args.get("cmd", "")
    risk = classify(cmd)
    if not gate.allow(f"shell: {cmd}", risk):
        return {"ok": False, "error": f"HITL: kibali hakipatikani (risk={risk})"}
    timeout = min(int(args.get("timeout", 60)), 300)
    p = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
    return {"ok": p.returncode == 0, "code": p.returncode, "stdout": _clip(p.stdout), "stderr": _clip(p.stderr)}


def file_read(args: dict, gate: Gate) -> dict:
    path = Path(args.get("path", ""))
    if not gate.allow(f"read {path}", "LOW"):
        return {"ok": False, "error": "HITL imekataa"}
    try:
        return {"ok": True, "content": _clip(path.read_text(errors="replace"))}
    except OSError as e:
        return {"ok": False, "error": str(e)}


def file_write(args: dict, gate: Gate) -> dict:
    path = Path(args.get("path", ""))
    content = args.get("content", "")
    if not gate.allow(f"write {path} ({len(content)} bytes)", "MEDIUM"):
        return {"ok": False, "error": "HITL imekataa"}
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        return {"ok": True}
    except OSError as e:
        return {"ok": False, "error": str(e)}


def file_list(args: dict, gate: Gate) -> dict:
    path = Path(args.get("path", "."))
    if not gate.allow(f"list {path}", "LOW"):
        return {"ok": False, "error": "HITL imekataa"}
    try:
        items = sorted(os.listdir(path))[:200]
        return {"ok": True, "path": str(path), "items": items}
    except OSError as e:
        return {"ok": False, "error": str(e)}


# ---------------------------------------------------------------- kernel probe
def sys_probe(args: dict, gate: Gate) -> dict:
    out = {"kernel": platform.release(), "arch": platform.machine(), "hostname": platform.node()}
    try:
        load1, load5, load15 = os.getloadavg()
        out["load"] = [round(load1, 2), round(load5, 2), round(load15, 2)]
    except OSError:
        pass
    mem = {}
    try:
        for line in Path("/proc/meminfo").read_text().splitlines()[:5]:
            k, v = line.split(":", 1)
            mem[k.strip()] = v.strip()
        out["mem"] = mem
    except OSError:
        pass
    try:
        out["uptime_s"] = int(float(Path("/proc/uptime").read_text().split()[0]))
    except (OSError, ValueError, IndexError):
        pass
    disks = []
    try:
        for p in Path("/proc/mounts").read_text().splitlines():
            parts = p.split()
            if len(parts) >= 4 and not parts[1].startswith(("/snap", "/boot/efi")):
                disks.append({"dev": parts[0], "mnt": parts[1], "type": parts[2]})
        out["mounts"] = disks[:12]
    except OSError:
        pass
    procs = []
    for pid_dir in list(Path("/proc").glob("[0-9]*"))[:400]:
        try:
            comm = (pid_dir / "comm").read_text().strip()
            with (pid_dir / "stat").open() as f:
                parts = f.read().rsplit(") ", 1)[1].split()
                state = parts[0]
                procs.append({"pid": int(pid_dir.name), "comm": comm, "state": state})
        except (OSError, IndexError, ValueError):
            continue
    out["processes"] = procs[:60]
    out["process_count"] = len(procs)
    return {"ok": True, "system": out}


# ---------------------------------------------------------------- GUI control
def _xdotool(*xargs: str) -> dict:
    if not shutil.which("xdotool"):
        return {"ok": False, "error": "xdotool haipo (saki: apt install xdotool)"}
    p = subprocess.run(["xdotool", *xargs], capture_output=True, text=True, timeout=30)
    return {"ok": p.returncode == 0, "out": _clip(p.stdout), "err": _clip(p.stderr)}


def control_input(args: dict, gate: Gate) -> dict:
    """Mikono ya agent juu ya GUI: mouse + keyboard kupitia X11."""
    action = args.get("action", "")
    if not gate.allow(f"control:{action}", "HIGH"):
        return {"ok": False, "error": "HITL: udhibiti wa GUI unahitaji kibali"}
    if action == "move":
        return _xdotool("mousemove", str(args.get("x", 0)), str(args.get("y", 0)))
    if action == "click":
        return _xdotool("click", str(args.get("button", 1)))
    if action == "type":
        return _xdotool("type", "--delay", "40", str(args.get("text", "")))
    if action == "key":
        return _xdotool("key", str(args.get("key", "Return")))
    if action == "focus":
        return _xdotool("windowactivate", str(args.get("window", "")))
    return {"ok": False, "error": f"action haijulikani: {action}"}


def process_kill(args: dict, gate: Gate) -> dict:
    target = args.get("pid") or args.get("name", "")
    if not gate.allow(f"kill {target}", "HIGH"):
        return {"ok": False, "error": "HITL imekataa"}
    p = subprocess.run(["kill", str(target)], capture_output=True, text=True)
    return {"ok": p.returncode == 0, "err": p.stderr}


# ---------------------------------------------------------------- MTAALAMU skills
def mtaalamu_skills(args: dict, gate: Gate) -> dict:
    from .skills_bridge import search_skills, plan_skills

    q = args.get("q") or args.get("msg") or ""
    if not q:
        return {"ok": True, "skills": search_skills("")[:40]}
    hits = search_skills(q)
    if not hits:
        hits = plan_skills(q)
    return {"ok": True, "query": q, "count": len(hits), "skills": hits[:15]}


def mtaalamu_skill_run(args: dict, gate: Gate) -> dict:
    from .skills_bridge import run_skill

    if not gate.allow(f"skill:{args.get('id')}", "MEDIUM"):
        return {"ok": False, "error": "HITL imekataa"}
    return run_skill(args.get("id", ""), args.get("inputs") or {})


# ---------------------------------------------------------------- registry
TOOLS = {
    "shell_exec": shell_exec,
    "file_read": file_read,
    "file_write": file_write,
    "file_write_alias": file_write,
    "file_list": file_list,
    "sys_probe": sys_probe,
    "see_screen": None,     # huunganishwa na llm (vision) — angalia loop
    "control_input": control_input,
    "process_kill": process_kill,
    "kernel_events": None,  # huunganishwa na kernel_events.EventFeed
    "mtaalamu_skills": mtaalamu_skills,
    "mtaalamu_skill_run": mtaalamu_skill_run,
}

TOOL_HELP = {
    "shell_exec": {"cmd": "amri ya shell", "timeout": "sekunde"},
    "file_read": {"path": "njia"},
    "file_write": {"path": "njia", "content": "maandishi"},
    "file_list": {"path": "saraka"},
    "sys_probe": {},
    "see_screen": {"question": "swali kuhusu screen"},
    "control_input": {"action": "move|click|type|key", "x": 0, "y": 0, "text": "", "key": "Return"},
    "process_kill": {"pid": 123},
    "kernel_events": {"limit": 20},
    "mtaalamu_skills": {"q": "neno la utafutaji"},
    "mtaalamu_skill_run": {"id": "skill.id", "inputs": {"k": "v"}},
}
