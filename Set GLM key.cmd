@echo off
rem Asks for your Z.ai key in a hidden Windows dialog, stores it encrypted for your account,
rem copies it to the GitHub Actions secret, then checks once that Z.ai accepts it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\set-glm-key.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\set-glm-key.ps1" -Test
pause
