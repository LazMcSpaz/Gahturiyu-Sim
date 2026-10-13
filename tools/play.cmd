@echo off
rem Gahturiyu launcher (Laz's PC): pull the latest, build, play.
rem
rem   1  main         your game       C:\Users\Laz PC\Repos\Gahturiyu-Sim
rem   2  the preview  next/items      C:\Users\Laz PC\Repos\Gahturiyu-Sim-next
rem
rem A copy of this file sits on the desktop as "Play Gahturiyu.cmd". It
rem pulls (fast-forward only, so it never touches local changes), builds
rem with half the CPU, and starts the game from the clone's own folder.
rem If the pull fails (offline, or the branch has moved on in a way that
rem isn't a fast-forward) it says so and plays what's already here.
setlocal
title Gahturiyu
set "ROOT=C:\Users\Laz PC\Repos"
set "CARGO_BUILD_JOBS=8"
set "CARGO_TARGET_DIR="
rem (cargo and git, wherever this is run from.)
set "PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\Git\cmd;%PATH%"

echo.
echo   Gahturiyu
echo.
echo   1  Play main (your game): pull, build, play
echo   2  Play the preview (next/items): pull, build, play
echo   3  Play main as it is (no pull, no build)
echo   4  Play the preview as it is (no pull, no build)
echo.
choice /c 1234 /n /m "  Pick 1-4: "
set "PICK=%errorlevel%"

if "%PICK%"=="1" ( set "DIR=%ROOT%\Gahturiyu-Sim" & set "FRESH=1" & set "WHAT=main" )
if "%PICK%"=="2" ( set "DIR=%ROOT%\Gahturiyu-Sim-next" & set "FRESH=1" & set "WHAT=the preview" )
if "%PICK%"=="3" ( set "DIR=%ROOT%\Gahturiyu-Sim" & set "FRESH=" & set "WHAT=main" )
if "%PICK%"=="4" ( set "DIR=%ROOT%\Gahturiyu-Sim-next" & set "FRESH=" & set "WHAT=the preview" )

if not exist "%DIR%\Cargo.toml" (
  echo.
  echo   Can't find the game at "%DIR%".
  pause
  exit /b 1
)
cd /d "%DIR%"

if defined FRESH (
  echo.
  echo   Pulling %WHAT%...
  git pull --ff-only
  if errorlevel 1 (
    echo.
    echo   Couldn't pull ^(offline, or the branch moved in a way that isn't a fast-forward^).
    echo   Playing what's already here.
    echo.
  )
  echo.
  echo   Building %WHAT% ^(about a minute after a change; the first time takes much longer^)...
  cargo build --release
  if errorlevel 1 (
    echo.
    echo   The build failed. If the game is still open, close it and try again:
    echo   Windows locks the exe while it runs.
    pause
    exit /b 1
  )
)

if not exist "%DIR%\target\release\gahturiyu.exe" (
  echo.
  echo   There's no built game at "%DIR%\target\release\gahturiyu.exe". Pick 1 or 2 to build it.
  pause
  exit /b 1
)

echo.
echo   Starting %WHAT%...
start "" /d "%DIR%" "%DIR%\target\release\gahturiyu.exe"
endlocal
exit /b 0
