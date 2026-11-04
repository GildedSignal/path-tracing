# Opens the server in its own console window (type commands there; `stop` to save and quit).
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$server = Join-Path $Rig 'server'
if (Test-Port $ServerHost $ServerPort) { Write-Host 'Server already running.'; return }
Start-Process -FilePath "$Jdk\java.exe" -ArgumentList '-Xms2G', '-Xmx4G', '-jar', 'fabric-server-launch.jar', 'nogui' -WorkingDirectory $server
