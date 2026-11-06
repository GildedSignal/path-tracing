@echo off
rem Runs the GPT jobs in jobs\queue.txt one after another. Safe to run again after a shutdown.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scriptsun-queue.ps1"
pause
