@echo off
rem Saves the world and stops the server cleanly over RCON. Double-click.
powershell -NoProfile -ExecutionPolicy Bypass -Command ". '%~dp0scripts\common.ps1'; if (-not (Test-Port $ServerHost 25575)) { Write-Host 'The server is not running.'; exit 0 }; & '%~dp0scripts\rcon.ps1' 'save-all flush' 'stop'; $t = Get-Date; while ((Test-Port $ServerHost $ServerPort) -and ((Get-Date) - $t).TotalSeconds -lt 60) { Start-Sleep 1 }; if (Test-Port $ServerHost $ServerPort) { Write-Host 'Still running after 60 s.'; exit 1 } else { Write-Host 'Server saved and stopped.' }"
if errorlevel 1 pause
