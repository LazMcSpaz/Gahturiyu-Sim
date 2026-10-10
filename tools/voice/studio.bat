@echo off
rem Opens the accent workbench: a page of knobs and sliders, on this computer only.
rem The first run sets itself up (a few minutes, about 350 MB of voice models).
setlocal
cd /d "%~dp0"

set PY=
where py >nul 2>nul && set PY=py -3
if not defined PY where python >nul 2>nul && set PY=python
if not defined PY (
  echo Python isn't installed. Get Python 3.11 or newer from https://www.python.org/downloads/
  echo ^(tick "Add python.exe to PATH" in its installer^), then run this again.
  pause
  exit /b 1
)
%PY% -c "import sys; sys.exit(0 if sys.version_info >= (3, 11) else 1)"
if errorlevel 1 (
  echo This needs Python 3.11 or newer. Get it from https://www.python.org/downloads/ then run this again.
  pause
  exit /b 1
)

if not exist ".venv\Scripts\python.exe" (
  echo Setting up, first run only...
  %PY% -m venv .venv || goto fail
)
".venv\Scripts\python.exe" -m pip install -q --disable-pip-version-check -r requirements.txt || goto fail
".venv\Scripts\python.exe" voice.py setup --studio || goto fail
".venv\Scripts\python.exe" voice.py studio
exit /b 0

:fail
echo.
echo Something above went wrong. Copy this window's text and give it to Claude.
pause
exit /b 1
