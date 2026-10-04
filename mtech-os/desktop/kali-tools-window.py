#!/usr/bin/env python3
"""MTECH OS — Kali Tools (dirisha kama picha ya Kali).

Replication ya dirisha "Kali Tools" kutoka picha:
  • Title: "Kali Tools" + minimize/maximize/close
  • Utafutaji (Search…) juu
  • Categories 11 (columns 2) zenye icons za rangi:
      Information Gathering, Vulnerability Analysis, Web Application,
      Password Attacks, Exploitation Tools, Sniffing & Spoofing,
      Post Exploitation, Forensics, Reporting Tools, Reverse Engineering,
      Wireless Attacks
  • Kila category inaanzisha zana halisi za Kali (kali-menu categories):
      Information Gathering → xfce4-terminal -x nmap
      Vulnerability Analysis → nikto (terminal)
      Web Application → burpsuite
      Password Attacks → john (terminal)
      Exploitation Tools → msfconsole (terminal)
      Sniffing & Spoofing → wireshark
      Post Exploitation → xfce4-terminal (msfvenom etc.)
      Forensics → autopsy
      Reporting Tools → cherrytree
      Reverse Engineering → ghidra / r2 (terminal)
      Wireless Attacks → aircrack-ng (terminal)
"""
import subprocess
import sys

from PySide6 import QtCore, QtGui, QtWidgets

BG = "#0d0d0d"
ROW = "#151515"
ROW_HOVER = "#1f1f1f"
TXT = "#d0d0d0"
ACCENT = "#367bf0"

# (jina, icon-emoji/rangi, amri halisi ya Kali)
CATEGORIES = [
    ("Information Gathering", "#3aa0f0", "xfce4-terminal -x nmap &"),
    ("Vulnerability Analysis", "#c0c0c0", "xfce4-terminal -x nikto &"),
    ("Web Application", "#e05555", "burpsuite &"),
    ("Password Attacks", "#f0a030", "xfce4-terminal -x john &"),
    ("Exploitation Tools", "#3a3a3a", "xfce4-terminal -x msfconsole &"),
    ("Sniffing & Spoofing", "#3a8ff0", "wireshark &"),
    ("Post Exploitation", "#5a8ff0", "xfce4-terminal &"),
    ("Forensics", "#e0e0e0", "xfce4-terminal -x autopsy &"),
    ("Reporting Tools", "#d0d0d0", "cherrytree &"),
    ("Reverse Engineering", "#8a8a8a", "xfce4-terminal -x r2 &"),
    ("Wireless Attacks", "#e06060", "xfce4-terminal -x aircrack-ng &"),
]

QSS = f"""
QWidget {{ background: {BG}; color: {TXT}; font-size: 13px; }}
#title {{ font-weight: 600; }}
#search {{
  background: {ROW}; border: 1px solid #2a2a2a; border-radius: 6px; padding: 7px 12px;
}}
#row {{
  background: {ROW}; border-radius: 8px;
}}
#row:hover {{ background: {ROW_HOVER}; }}
#icon {{ font-size: 22px; }}
#catname {{ font-size: 13px; color: {TXT}; }}
"""


class CategoryRow(QtWidgets.QFrame):
    def __init__(self, name: str, color: str, command: str):
        super().__init__()
        self.setObjectName("row")
        self.command = command
        lay = QtWidgets.QHBoxLayout(self)
        lay.setContentsMargins(10, 8, 10, 8)

        icon = QtWidgets.QLabel("\U0001f4e6")  # fallback emoji; icons halisi chini
        icon.setObjectName("icon")

        label = QtWidgets.QLabel(name)
        label.setObjectName("catname")

        lay.addWidget(icon)
        lay.addWidget(label)
        lay.addStretch(1)
        self.setCursor(QtCore.Qt.PointingHandCursor)

    def mousePressEvent(self, e):
        subprocess.Popen(self.command, shell=True, start_new_session=True)
        self.window().statusBar().showMessage(f"Imeanzisha: {self.command.split('&')[0].strip()}", 4000)


class KaliTools(QtWidgets.QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Kali Tools")
        self.resize(780, 620)
        self.statusBar().hide()

        central = QtWidgets.QWidget()
        self.setCentralWidget(central)
        root = QtWidgets.QVBoxLayout(central)
        root.setContentsMargins(12, 12, 12, 12)
        root.setSpacing(10)

        self.search = QtWidgets.QLineEdit()
        self.search.setPlaceholderText("Search…")
        self.search.setObjectName("search")
        self.search.textChanged.connect(self.filter_rows)
        root.addWidget(self.search)

        # columns 2 kama picha
        grid = QtWidgets.QGridLayout()
        grid.setHorizontalSpacing(10)
        grid.setVerticalSpacing(10)
        self.rows: list[CategoryRow] = []
        for i, (name, color, cmd) in enumerate(CATEGORIES):
            row = CategoryRow(name, color, cmd)
            row.setToolTip(cmd)
            self.rows.append(row)
            grid.addWidget(row, i // 2, i % 2)
        grid.setRowStretch(len(CATEGORIES) // 2 + 1, 1)
        root.addLayout(grid)

    def filter_rows(self, text: str):
        t = text.lower().strip()
        for row in self.rows:
            name = row.findChild(QtWidgets.QLabel, "catname").text().lower()
            row.setVisible(t in name)


def main():
    app = QtWidgets.QApplication(sys.argv)
    app.setStyleSheet(QSS)
    win = KaliTools()
    win.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
