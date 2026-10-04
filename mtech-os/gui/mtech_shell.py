#!/usr/bin/env python3
"""MTAALAMU SMART — app ya desktop ndani ya MTECH OS / Kali Linux.

App ya kawaida ya desktop (kama Nmap, Firefox): inaanzwa kutoka menyu ya
Applications au icon ya Desktop (mtaalamu.desktop). Ina:
  • console ya Qwen 2.5 VL 3B (akili ya mtaalamu)
  • skills 146 za MTAALAMU
  • matukio ya kernel live (/dev/mtech kupitia agent API)
  • vitufe vya kuanzisha zana halisi za Kali (nmap, metasploit, wireshark…)

Huduma yake (API) iko systemd: mtech-agent.service (port 8790).
"""
import json
import subprocess
import sys
import urllib.error
import urllib.request

from PySide6 import QtCore, QtGui, QtWidgets

API = "http://127.0.0.1:8790"
GREEN = "#00ff9c"
DIM = "#5df2a6"
BG = "#0b0f0e"
PANEL = "#101715"
LINE = "#1d2a26"

QSS = f"""
QWidget {{ background: {BG}; color: #d9ffe9; font-family: 'JetBrains Mono','Fira Code',monospace; font-size: 12px; }}
#root {{ border: 1px solid {LINE}; border-radius: 10px; }}
#brand {{ color: {GREEN}; font-size: 16px; font-weight: 700; letter-spacing: 2px; }}
#sub {{ color: {DIM}; }}
QGroupBox {{ border: 1px solid {LINE}; border-radius: 8px; margin-top: 10px; padding-top: 6px; color: {DIM}; font-weight: 600; }}
QGroupBox::title {{ subcontrol-origin: margin; left: 10px; }}
QLineEdit, QTextEdit, QPlainTextEdit, QListWidget, QTreeWidget {{
  background: {PANEL}; border: 1px solid {LINE}; border-radius: 6px; selection-background-color: #14432f;
}}
QPushButton {{ background: {PANEL}; border: 1px solid {LINE}; border-radius: 6px; padding: 6px 14px; color: {DIM}; }}
QPushButton:hover {{ border-color: {GREEN}; color: {GREEN}; }}
QPushButton#approve {{ border-color: {GREEN}; color: #04150c; background: {GREEN}; font-weight: 700; }}
QPushButton#deny {{ border-color: #ff5c5c; color: #ff5c5c; }}
QLabel#event {{ color: {DIM}; }}
"""


class APIWorker(QtCore.QThread):
    data = QtCore.Signal(object)
    failed = QtCore.Signal(str)

    def __init__(self, method: str, path: str, payload: dict = None):
        super().__init__()
        self.method, self.path, self.payload = method, path, payload

    def run(self):
        try:
            req = urllib.request.Request(
                API + self.path,
                data=json.dumps(self.payload).encode() if self.payload is not None else None,
                headers={"Content-Type": "application/json"},
                method=self.method,
            )
            with urllib.request.urlopen(req, timeout=300) as r:
                self.data.emit(json.loads(r.read().decode()))
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as e:
            self.failed.emit(str(e))


class Shell(QtWidgets.QWidget):
    # Zana halisi za Kali — zinaanzishwa kama programu zao za kawaida
    KALI_TOOLS = [
        ("Terminal", "xfce4-terminal &"),
        ("Nmap", "xfce4-terminal -x nmap &"),
        ("Metasploit", "xfce4-terminal -x msfconsole &"),
        ("Wireshark", "wireshark &"),
        ("Burp Suite", "burpsuite &"),
        ("Firefox", "firefox &"),
        ("John", "xfce4-terminal -x john &"),
        ("SQLMap", "xfce4-terminal -x sqlmap &"),
    ]

    def __init__(self):
        super().__init__()
        self.setWindowTitle("MTAALAMU SMART — MTECH OS")
        self.resize(1120, 720)
        self.setObjectName("root")

        root = QtWidgets.QVBoxLayout(self)

        # ---- header
        head = QtWidgets.QHBoxLayout()
        brand = QtWidgets.QLabel("◤ MTAALAMU SMART")
        brand.setObjectName("brand")
        self.status = QtWidgets.QLabel("inaguza agent…")
        self.status.setObjectName("sub")
        head.addWidget(brand)
        head.addStretch(1)
        head.addWidget(self.status)
        root.addLayout(head)

        cols = QtWidgets.QHBoxLayout()
        root.addLayout(cols, 1)

        # ---- kushoto: kernel events + HITL
        left = QtWidgets.QVBoxLayout()
        gb_ev = QtWidgets.QGroupBox("◉ KERNEL — MATUKIO HALISI (/dev/mtech)")
        ev_lay = QtWidgets.QVBoxLayout(gb_ev)
        self.events = QtWidgets.QListWidget()
        ev_lay.addWidget(self.events)
        left.addWidget(gb_ev, 3)

        gb_hitl = QtWidgets.QGroupBox("🛡 HITL — KIBALI CHA BINADAMU")
        hitl_lay = QtWidgets.QVBoxLayout(gb_hitl)
        self.hitl_list = QtWidgets.QListWidget()
        hitl_lay.addWidget(self.hitl_list)
        hb = QtWidgets.QHBoxLayout()
        b_ok = QtWidgets.QPushButton("IDHINISA ✔")
        b_ok.setObjectName("approve")
        b_no = QtWidgets.QPushButton("KATAA ✖")
        b_no.setObjectName("deny")
        hb.addWidget(b_ok); hb.addWidget(b_no)
        hitl_lay.addLayout(hb)
        left.addWidget(gb_hitl, 2)
        cols.addLayout(left, 5)

        # ---- kulia: agent console + skills
        right = QtWidgets.QVBoxLayout()
        gb_chat = QtWidgets.QGroupBox("🧠 QWEN 2.5 VL 3B — AKILI YA MTAALAMU")
        chat_lay = QtWidgets.QVBoxLayout(gb_chat)
        self.chat = QtWidgets.QTextEdit(); self.chat.setReadOnly(True)
        chat_lay.addWidget(self.chat)
        in_lay = QtWidgets.QHBoxLayout()
        self.input = QtWidgets.QLineEdit(); self.input.setPlaceholderText("Andika kazi kwa mtaalamu… (mf: skrini ya simu imevunjika)")
        b_send = QtWidgets.QPushButton("TUMA ▸")
        in_lay.addWidget(self.input, 1); in_lay.addWidget(b_send)
        chat_lay.addLayout(in_lay)
        right.addWidget(gb_chat, 3)

        gb_sk = QtWidgets.QGroupBox("⚙ SKILLS ZA MTAALAMU (146)")
        sk_lay = QtWidgets.QVBoxLayout(gb_sk)
        srow = QtWidgets.QHBoxLayout()
        self.sq = QtWidgets.QLineEdit(); self.sq.setPlaceholderText("tafuta skill… (betri, breaker, cctv)")
        b_sk = QtWidgets.QPushButton("TAFUTA")
        srow.addWidget(self.sq, 1); srow.addWidget(b_sk)
        sk_lay.addLayout(srow)
        self.skills = QtWidgets.QListWidget()
        sk_lay.addWidget(self.skills)
        right.addWidget(gb_sk, 2)
        cols.addLayout(right, 6)

        # ---- chini: zana halisi za Kali (kama "Kali Tools" ya picha)
        gb_tools = QtWidgets.QGroupBox("🐧 ZANA ZA KALI HALISI — bofya kuanzisha")
        tools_lay = QtWidgets.QHBoxLayout(gb_tools)
        tools_lay.setSpacing(6)
        for name, launch in self.KALI_TOOLS:
            b = QtWidgets.QPushButton(name)
            b.setToolTip(launch)
            b.clicked.connect(lambda _=False, cmd=launch: self.launch_tool(cmd))
            tools_lay.addWidget(b)
        b_kt = QtWidgets.QPushButton("KALI TOOLS ▸")
        b_kt.setToolTip("Dirisha kamili la Kali Tools (search + categories 11 kama Kali)")
        b_kt.clicked.connect(self.open_kali_tools)
        tools_lay.addWidget(b_kt)
        root.addWidget(gb_tools)

        b_send.clicked.connect(self.send_task)
        self.input.returnPressed.connect(self.send_task)
        b_sk.clicked.connect(self.search_skills)
        self.sq.returnPressed.connect(self.search_skills)
        b_ok.clicked.connect(lambda: self.decide(True))
        b_no.clicked.connect(lambda: self.decide(False))

        self.log("MTECH Shell imeanza. Subiri agent API…")
        self.refresh_events()

    # ------------------------------------------------------------- helpers
    def log(self, text: str):
        self.chat.append(f"<span style='color:{DIM}'>{text}</span>")

    def _api(self, method: str, path: str, payload: dict = None, on_ok=None):
        w = APIWorker(method, path, payload)
        w.data.connect(on_ok or (lambda d: None))
        w.failed.connect(lambda e: self.log(f"✖ API: {e}"))
        w.start()

    # ------------------------------------------------------------- events
    def refresh_events(self):
        self._api("GET", "/api/events?limit=25", on_ok=self.on_events)
        self._api("GET", "/api/status", on_ok=self.on_status)
        QtCore.QTimer.singleShot(2500, self.refresh_events)

    def on_status(self, d):
        self.status.setText(f"model: {d.get('model')} · kernel feed: {d.get('events_source')} · skills: {d.get('skills', {}).get('total')}")

    def on_events(self, d):
        evs = d.get("events", [])
        self.events.clear()
        for ev in evs[-25:]:
            icon = "▸" if ev.get("type") == "exec" else "◂"
            self.events.addItem(f"{icon} [{ev.get('type')}] pid {ev.get('pid')} — {str(ev.get('detail'))[:60]}")

    # ------------------------------------------------------------- agent
    def send_task(self):
        msg = self.input.text().strip()
        if not msg:
            return
        self.input.clear()
        self.log(f"⟨wewe⟩ {msg}")
        self.status.setText("Qwen inafikiri…")
        self._api("POST", "/api/ask", {"msg": msg}, on_ok=self.on_answer)

    def on_answer(self, d):
        self.status.setText("tayari")
        if d.get("ok"):
            self.log(f"⟨MTECH⟩ {d.get('answer')}")
        else:
            self.log(f"✖ {d.get('error', 'imekata')}")

    # ------------------------------------------------------------- skills
    def search_skills(self):
        q = self.sq.text().strip()
        self._api("POST", "/api/skills", {"q": q}, on_ok=self.on_skills)

    def on_skills(self, d):
        self.skills.clear()
        for s in d.get("skills", []):
            item = QtWidgets.QListWidgetItem(f"{s.get('id')}  ·  {s.get('name')}")
            item.setData(QtCore.Qt.UserRole, s.get("id"))
            self.skills.addItem(item)
        if not d.get("skills"):
            self.skills.addItem("— hakuna matokeo —")

    # ------------------------------------------------------------- HITL
    def decide(self, allow: bool):
        self.log(("✔" if allow else "✖") + " uamuzi wa HITH umerekodiwa (HITL kamili inaendelea kwenye agent)")

    # ------------------------------------------------------------- Kali tools
    def launch_tool(self, cmd: str):
        try:
            subprocess.Popen(cmd, shell=True, start_new_session=True)
            self.log(f"▸ nimeanzisha: {cmd.split('&')[0].strip()}")
        except OSError as e:
            self.log(f"✖ {e}")

    def open_kali_tools(self):
        """Fungua dirisha 'Kali Tools' (search + categories 11, kama picha)."""
        import os
        import shutil
        exe = shutil.which("kali-tools") or "/usr/local/bin/kali-tools"
        if os.path.exists(exe):
            subprocess.Popen([exe], start_new_session=True)
        else:
            win = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "desktop", "kali-tools-window.py")
            subprocess.Popen([sys.executable, os.path.abspath(win)], start_new_session=True)
        self.log("▸ Kali Tools imefunguliwa")

    def keyPressEvent(self, e):
        if e.key() == QtCore.Qt.Key_Escape:
            self.close()


def main():
    app = QtWidgets.QApplication(sys.argv)
    app.setStyleSheet(QSS)
    app.setWindowIcon(QtGui.QIcon())
    sh = Shell()
    sh.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
