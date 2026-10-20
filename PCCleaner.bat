@echo off
title PC Cleaner
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0PCCleaner.ps1" %*
if errorlevel 1 (
    echo.
    echo PC Cleaner could not start. / Nie udalo sie uruchomic PC Cleanera.
    pause
)
