@echo off
setlocal EnableDelayedExpansion
REM ============================================================
REM  MTAALAMU SMART - SETU (Windows: bonyeza au endesha cmd)
REM    install.bat                          = kila kitu (auto, hakuna config)
REM    install.bat --check                  = ukaguzi tu
REM    install.bat --email me@mail.com --no-model --skip-rust --skip-r
REM
REM  Python haipo? Inasakinishwa KIOTOMATIKI (winget) au inakufungulia
REM  ukurasa wa python.org — hakuna configuration ya mkono.
REM ============================================================
title MTAALAMU SMART - Setup
cd /d "%~dp0"

set "PYCMD="

rem --- 1) py launcher
py -3 -c "import sys; sys.exit(0 if sys.version_info>=(3,10) else 1)" >nul 2>nul && set "PYCMD=py -3"

rem --- 2) python (3.10+ tu)
if not defined PYCMD (
  python -c "import sys; sys.exit(0 if sys.version_info>=(3,10) else 1)" >nul 2>nul && set "PYCMD=python"
)

rem --- 3) python aliyesakinishwa lakini haipo PATH (%LOCALAPPDATA%\Programs\Python)
if not defined PYCMD (
  for /d %%D in ("%LOCALAPPDATA%\Programs\Python\Python3*") do (
    if exist "%%D\python.exe" (
      "%%D\python.exe" -c "import sys; sys.exit(0 if sys.version_info>=(3,10) else 1)" >nul 2>nul && set "PYCMD=%%D\python.exe"
    )
  )
)

rem --- 4) hakuna Python kwa kabisa → winget inasakinisha yenyewe
if not defined PYCMD (
  echo  Python 3.10+ haipo — inasakinishwa KIOTOMATIKI kwa winget (subiri dakika 1-2)...
  winget --version >nul 2>nul
  if !errorlevel!==0 (
    winget install -e --id Python.Python.3.12 --accept-source-agreements --accept-package-agreements --silent
    for /d %%D in ("%LOCALAPPDATA%\Programs\Python\Python3*") do (
      if exist "%%D\python.exe" set "PYCMD=%%D\python.exe"
    )
  )
)

rem --- 5) bado hakuna → fungua python.org na maelezo wazi
if not defined PYCMD (
  echo.
  echo  [X] Python haikuweza kusakinishwa kiotomatiki (winget haipo).
  echo  Nimekufungulia ukurasa wa kupakua — fanya hivi:
  echo     1. Pakua "Windows installer (64-bit)"
  echo     2. Usakinishe — WEKA TICK kwenye "Add python.exe to PATH"
  echo     3. Fungua cmd MPYA kisha endesha install.bat tena
  echo.
  start "" https://www.python.org/downloads/windows/
  pause
  exit /b 1
)

echo  Python: !PYCMD!
echo.
!PYCMD! setup_all.py %*
echo.
pause
