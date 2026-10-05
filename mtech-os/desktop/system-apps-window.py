#!/usr/bin/env python3
"""MTECH OS — System Apps (dirisha la apps za mfumo, UI HALISI za waundaji).

Apps hizi ndizo zilizoundwa na mfumo — zinaonekana watu wanaposakinisha
apps nyingine, na kila moja inazindua UI yake HALISI:

  • OpenMRS         — EHR (docker compose up → http://localhost:8080/openmrs)
  • Home Assistant  — nyumba mahiri (docker run → http://localhost:8123)
  • Web R           — dashboard ya Shiny (Rscript app.R → http://localhost:8089)
  • Hermes Desktop  — app ya Electron (npm start → dirisha la desktop)
  • MTAALAMU Website— Docusaurus (npm start → http://localhost:3000)
  • Engine Rust     — mtaalamu reason '...' (CLI halisi ya Rust)
  • Analytics R     — Rscript analytics.R (riporti + chati)

Mtiririko HALISI: py mtenganishaji (kali.py/system_apps.py kutoka hermes-agent)
→ HTTP API 127.0.0.1:8795 — UI ile ile ya tabs (mtaalamu-unified.html).
"""
import json
import subprocess
import sys
import urllib.request

from PySide6 import QtCore, QtGui, QtWidgets

API = "http://127.0.0.1:8795"
FALLBACK_MSG = ("API ya MTAALAMU haipatikani — endesha: mtaalamu serve\n"
                "(au python3 -m mtaalamu serve)")

BG = "#0d0d0d"
ROW = "#151515"
ROW_HOVER = "#1f1f1f"
TXT = "#d0d0d0"
GREEN = "#2ecc71"
RED = "#e05555"
BLUE = "#367bf0"


def _get(path: str):
    try:
        with urllib.request.urlopen(API + path, timeout=10) as r:
            return json.loads(r.read().decode())
    except Exception:  # noqa: BLE001
        return None


def _post(path: str, body: dict):
    req = urllib.request.Request(API + path, data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return json.loads(r.read().decode())
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "error": f"{type(e).__name__}: {e}"}


QSS = f"""
QWidget {{ background: {BG}; color: {TXT}; font-size: 13px; }}
#title {{ font-size: 15px; font-weight: 600; color: {BLUE}; }}
#row {{ background: {ROW}; border: 1px solid #232323; border-radius: 8px; }}
#row:hover {{ border-color: {BLUE}; }}
#appname {{ font-weight: 600; }}
#status-running {{ color: {GREEN}; font-weight: 600; }}
#status-installed {{ color: {BLUE}; }}
#status-missing {{ color: {RED}; }}
QPushButton {{ background: #1c2a3a; border: 1px solid #2a3a4a; border-radius: 6px; padding: 5px 12px; }}
QPushButton:hover {{ border-color: {BLUE}; color: {BLUE}; }}
#hint {{ color: #8a8a8a; font-size: 11px; }}
"""


class AppCard(QtWidgets.QFrame):
    """Kadi moja ya app — status HALISI + vitufe vya LAUNCH/STOP/UI."""

    def __init__(self, app: dict, window: "SystemApps"):
        super().__init__()
        self.app = app
        self.win = window
        self.setObjectName("row")
        lay = QtWidgets.QGridLayout(self)
        lay.setContentsMargins(12, 9, 12, 9)

        name = QtWidgets.QLabel(f"{app['name']}")
        name.setObjectName("appname")

        desc = QtWidgets.QLabel(app.get("desc", ""))
        desc.setObjectName("hint")

        st = QtWidgets.QLabel()
        if app.get("running"):
            st.setText("▶ RUNNING")
            st.setObjectName("status-running")
        elif app.get("installed"):
            st.setText("✔ IPO")
            st.setObjectName("status-installed")
        else:
            st.setText("✗ HAIPO")
            st.setObjectName("status-missing")
            st.setToolTip(str(app.get("source") or "folda ya code haipo"))

        ui_lbl = QtWidgets.QLabel(f"UI: {app.get('ui', '—')}")
        ui_lbl.setObjectName("hint")

        btn_open = QtWidgets.QLabel()
        btn_ui = QtWidgets.QPushButton("FUNGUA UI")
        btn_ui.setVisible(str(app.get("ui", "")).startswith("http"))
        btn_ui.clicked.connect(self.open_ui)

        btn_launch = QtWidgets.QPushButton("ANZISHA")
        btn_launch.clicked.connect(self.launch)

        btn_stop = QtWidgets.QPushButton("SIMAMISHA")
        btn_stop.clicked.connect(self.stop)
        btn_stop.setVisible(bool(app.get("running")))

        lay.addWidget(name, 0, 0)
        lay.addWidget(st, 0, 1)
        lay.addWidget(btn_launch, 0, 2)
        lay.addWidget(btn_stop, 0, 3)
        lay.addWidget(btn_ui, 0, 4)
        lay.addWidget(desc, 1, 0, 1, 3)
        lay.addWidget(ui_lbl, 1, 3, 1, 3)

    def launch(self):
        self.win.status(f"▸ Inaanzisha {self.app['id']}…")
        r = _post("/api/apps/launch", {"app": self.app["id"]})
        if r.get("ok"):
            self.win.status(f"✔ {self.app['id']}: {r.get('note', 'imeanzishwa')}")
        else:
            self.win.status(f"✖ {r.get('error', 'imeshindikana')}")
        self.win.refresh()

    def stop(self):
        r = _post("/api/apps/stop", {"app": self.app["id"]})
        self.win.status(("✔ " if r.get("ok") else "✖ ") + (r.get("note") or r.get("error") or ""))
        self.win.refresh()

    def open_ui(self):
        url = self.app.get("ui", "")
        if url.startswith("http"):
            QtGui.QDesktopServices.openUrl(QtCore.QUrl(url))


class SystemApps(QtWidgets.QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("MTECH OS — System Apps")
        self.resize(860, 640)
        self.statusBar().hide()

        central = QtWidgets.QWidget()
        self.setCentralWidget(central)
        root = QtWidgets.QVBoxLayout(central)
        root.setContentsMargins(14, 12, 14, 12)
        root.setSpacing(10)

        title = QtWidgets.QLabel("◤ SYSTEM APPS — UI HALISI ZA WAUNDAJI")
        title.setObjectName("title")
        root.addWidget(title)

        self.hint = QtWidgets.QLabel("Apps hizi zimeundwa na mfumo — zinaonekana pamoja na apps zingine ulizosakinisha.")
        self.hint.setObjectName("hint")
        root.addWidget(self.hint)

        self.list_area = QtWidgets.QScrollArea()
        self.list_area.setWidgetResizable(True)
        self.list_area.setFrameShape(QtWidgets.QFrame.NoFrame)
        self.cards_holder = QtWidgets.QWidget()
        self.cards_lay = QtWidgets.QVBoxLayout(self.cards_holder)
        self.cards_lay.setContentsMargins(0, 0, 0, 0)
        self.cards_lay.setSpacing(8)
        self.list_area.setWidget(self.cards_holder)
        root.addWidget(self.list_area, 1)

        self.msg = QtWidgets.QLabel("")
        self.msg.setObjectName("hint")
        root.addWidget(self.msg)

        self.timer = QtCore.QTimer(self)
        self.timer.timeout.connect(self.refresh)
        self.timer.start(15000)  # refresh kila 15s
        self.refresh()

    def status(self, text: str):
        self.msg.setText(text)

    def refresh(self):
        data = _get("/api/apps")
        while self.cards_lay.count():
            item = self.cards_lay.takeAt(0)
            if w := item.widget():
                w.deleteLater()
        if not data:
            self.msg.setText(FALLBACK_MSG)
            return
        runtimes = data.get("runtimes", {})
        rt = "  ·  ".join(f"{k}:{'✔' if v else '✗'}" for k, v in runtimes.items())
        self.hint.setText(f"Runtimes: {rt}  —  bofya ANZISHA kwa UI halisi ya app")
        for app in data.get("apps", []):
            self.cards_lay.addWidget(AppCard(app, self))
        self.cards_lay.addStretch(1)


def main():
    app = QtWidgets.QApplication(sys.argv)
    app.setStyleSheet(QSS)
    win = SystemApps()
    win.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
