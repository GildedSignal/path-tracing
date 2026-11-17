# Read-only real-rig audit. The corrected registry is delivered inside the worktree.
[CmdletBinding()]
param([string]$Output, [string]$Record)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
if (Test-Port $ServerHost $ServerPort) { throw "Refusing spawn audit: server port $ServerPort is open." }
if (-not $Output) { $Output = Join-Path $Root 'jobs\track-ux\maps.json' }
if (-not $Record) { $Record = Join-Path $Root 'jobs\track-ux\spawns.json' }
$registryHash = (Get-FileHash -LiteralPath (Join-Path $Rig 'maps.json')).Hash
$classpath = @((Join-Path $Root 'bot\build\pathtracing.jar')) + @(Get-GameClasspath)
& "$Jdk\java.exe" -cp ($classpath -join ';') pathtracing.MapSpawnAudit $Rig $Output $Record
if ($LASTEXITCODE) { throw 'Spawn audit failed.' }
if (Test-Port $ServerHost $ServerPort) { throw 'Server started during the audit; refusing the result.' }
if ((Get-FileHash -LiteralPath (Join-Path $Rig 'maps.json')).Hash -ne $registryHash) { throw 'Registry changed during the audit; refusing the result.' }
@{command='scripts/audit-map-spawns.ps1'; rig=$Rig; port=$ServerPort; portClosedBeforeAndAfter=$true;
  inputRegistrySha256=$registryHash; outputSha256=(Get-FileHash -LiteralPath $Output).Hash;
  rowsSha256=(Get-FileHash -LiteralPath $Record).Hash; result='PASS'} |
  ConvertTo-Json | Set-Content -LiteralPath "$Record.run.json" -Encoding ASCII
