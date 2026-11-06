param(
  [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9_-]+$')][string]$Name,
  [Parameter(Mandatory)][ValidateRange(1, 2147483647)][int]$Ticks,
  [Parameter(Mandatory)][string]$Start
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$directory = Join-Path $Rig 'clients\bot\pathtracing\inputs'
$null = New-Item -ItemType Directory -Force $directory
$path = Join-Path $directory "$Name.jsonl"
if ((Test-Path -LiteralPath $path) -or (Test-Path -LiteralPath "$path.tmp")) { throw "Input output exists: $path" }
$writer = [IO.StreamWriter]::new("$path.tmp", $false, [Text.UTF8Encoding]::new($false))
try {
  $writer.NewLine = "`n"
  $writer.WriteLine('{"start":[' + $Start + ']}')
  for ($t = 0; $t -lt $Ticks; $t++) {
    $writer.WriteLine('{"t":' + $t + ',"forward":true,"back":false,"left":true,"right":false,"jump":false,"sneak":false,"sprint":false,"yaw":0,"pitch":0}')
  }
} finally { $writer.Dispose() }
[IO.File]::Move("$path.tmp", $path)
Write-Output "Wrote $Ticks deterministic corner inputs to $path."
