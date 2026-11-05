@echo off
rem Starts the server, the bot on your second monitor, and your client. Double-click.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0play.ps1"
if errorlevel 1 pause
