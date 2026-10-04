@echo off
setlocal EnableDelayedExpansion
REM ============================================================
REM  MTAALAMU SMART - SETUP (kama hermes: binary au Python)
REM    - dist\MTAALAMU-Setup.exe ipo?  → inatumika MOJA (hakuna Python)
REM    - runtime\python.exe ipo?       → inatumika (embedded Python)
REM    - py / python ipo?              → inatumika
REM    - HAKUNA chochote?              → winget/python.org (auto)
REM  Kila njia inaishia setup_all.py: kila kitu inajimaliza yenyewe
REM  (C, Rust, R, venv, Ollama+Qwen, account+ADMIN, server+dashboard)
REM ============================================================
title MTAALAMU SMART - Setup
cd /d "%~dp0"
set "PYCMD="

rem --- A) BINARY TAYARI (hakuna Python inayohitajika kabisa)
if exist "mtaalamu\dist\MTAALAMU-Setup.exe" (
  echo  [A] Binary tayari — inatumika moja kwa moja...
  mtaalamu\dist\MTAALAMU-Setup.exe admin unlock
  mtaalamu\dist\MTAALAMU-Setup.exe serve
  echo  Dashboard: web-html\mtaalamu-unified.html  ^(browser^)
  pause
  exit /b 0
)

rem --- B) EMBEDDED Python (runtime\python.exe)
if exist "runtime\python.exe" set "PYCMD=runtime\python.exe"

rem --- C) py launcher / python
if not defined PYCMD (
  py -3 -c "import sys; sys.exit(0 if sys.version_info>=(3,10) else 1)" >nul 2>nul && set "PYCMD=py -3"
)
if not defined PYCMD (
  python -c "import sys; sys.exit(0 if sys.version_info>=(3,10) else 1)" >nul 2>nul && set "PYCMD=python"
)

rem --- D) PATH-less Python
if not defined PYCMD (
  for /d %%D in ("%LOCALAPPDATA%\Programs\Python\Python3*") do (
    if exist "%%D\python.exe" set "PYCMD=%%D\python.exe"
  )
)

rem --- E) HAKUNA → winget auto-install
if not defined PYCMD (
  echo  Python haipo — inasakinishwa kiotomatiki (winget)...
  winget --version >nul 2>nul
  if !errorlevel!==0 (
    winget install -e --id Python.Python.3.12 --accept-source-agreements --accept-package-agreements --silent
    for /d %%D in ("%LOCALAPPDATA%\Programs\Python\Python3*") do (
      if exist "%%D\python.exe" set "PYCMD=%%D\python.exe"
    )
  )
)

rem --- F) Bado hakuna → python.org
if not defined PYCMD (
  echo.
  echo  [X] Python haikuweza kupatikana. Nimekufungulia python.org:
  echo      1. Pakua "Windows installer (64-bit)"
  echo      2. WEKA TICK "Add python.exe to PATH"
  echo      3. cmd MPYA → endesha SETUP tena
  start "" https://www.python.org/downloads/windows/
  pause
  exit /b 1
)

echo  Python: !PYCMD!
echo.
!PYCMD! setup_all.py %*
echo.
pause
