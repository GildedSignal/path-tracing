# Offline metadata/layout adapter for plan-track. Never connects to the server.
param(
  [Parameter(Mandatory)][string]$World,
  [Parameter(Mandatory)][string]$TrackFile,
  [Parameter(Mandatory)][string]$View
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$jar = Join-Path $Root 'bot\build\pathtracing.jar'
if (-not (Test-Path -LiteralPath $jar)) { throw 'Build the bot first with bot\build.ps1.' }
$classpath = @($jar) + @(Get-GameClasspath)
& "$Jdk\java.exe" -cp ($classpath -join ';') pathtracing.TrackWorldView $World $TrackFile $View
if ($LASTEXITCODE) { throw 'Track world view could not be prepared.' }
