@echo off
rem Starts the server and your client. Double-click.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0play.ps1" -PlayerOnly
if errorlevel 1 pause
