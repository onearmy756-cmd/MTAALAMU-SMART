@echo off
REM ============================================================
REM  MTAALAMU SMART - SETU (Windows: bonyeza au endesha cmd)
REM    install.bat                    = kila kitu (auto, hakuna config)
REM    install.bat --check            = ukaguzi tu
REM    install.bat --email me@mail.com --no-model --skip-rust --skip-r
REM ============================================================
title MTAALAMU SMART - Setup
cd /d "%~dp0"

where py >nul 2>nul
if %errorlevel%==0 (
  py -3 setup_all.py %*
) else (
  where python >nul 2>nul
  if %errorlevel%==0 (
    python setup_all.py %*
  ) else (
    echo.
    echo  [X] Python 3.10+ haipo. Sakinisha kutoka https://python.org
    echo      (angalia "Add python.exe to PATH" wakati wa kusakinisha)
    echo.
  )
)
echo.
pause
