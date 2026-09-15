@echo off
chcp 65001 > nul
echo ================================================================================
echo    LIVA BANKING HARNESS - LIVE DEMO INNOSTART 2026
echo ================================================================================
powershell -ExecutionPolicy Bypass -File "%~dp0run_live_demo.ps1"
pause
