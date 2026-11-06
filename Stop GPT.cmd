@echo off
rem Stops every running GPT job. Run the queue again later to resume.
taskkill /F /IM codex.exe /T
pause
