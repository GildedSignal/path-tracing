@echo off
rem Stops every running GPT job (codex.exe). Resume one with: codex exec resume <session id>.
taskkill /F /IM codex.exe /T
pause
