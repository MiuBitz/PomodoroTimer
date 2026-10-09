@echo off
setlocal
cd /d "%~dp0"
set "POMODORO_BUILD_DIR=%~dp0target"
if defined CARGO_TARGET_DIR set "POMODORO_BUILD_DIR=%CARGO_TARGET_DIR%"
cargo build --release --locked
if errorlevel 1 exit /b 1
if not exist "dist" mkdir "dist"
copy /y "%POMODORO_BUILD_DIR%\release\pomodoro_timer.exe" "dist\PomodoroTimer.exe" >nul
if errorlevel 1 (
  echo Exit Pomodoro Timer from its tray menu and try again.
  exit /b 1
)
echo Ready: %CD%\dist\PomodoroTimer.exe
