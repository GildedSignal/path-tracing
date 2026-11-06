@echo off
rem Runs the GLM deep research (lens foundations-primary) on this PC, up to 3 hours. Close the window to stop it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\glm-local.ps1" -Lenses foundations-primary -Minutes 180
pause
