param(
  [Parameter(Mandatory)][string]$Rig20,
  [Parameter(Mandatory)][string]$Rig200,
  [Parameter(Mandatory)][int]$Port20,
  [Parameter(Mandatory)][int]$Port200,
  [Parameter(Mandatory)][string]$ReferenceServer,
  [Parameter(Mandatory)][string]$Edges20,
  [Parameter(Mandatory)][string]$Edges200,
  [Parameter(Mandatory)][string]$Teleport
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$Rig20 = (Resolve-Path -LiteralPath $Rig20).Path
$Rig200 = (Resolve-Path -LiteralPath $Rig200).Path
$ReferenceServer = (Resolve-Path -LiteralPath $ReferenceServer).Path
$overlap = @($Port20, ($Port20 + 10)) | Where-Object { $_ -in @($Port200, ($Port200 + 10)) }
if ($Rig20 -eq $Rig200 -or $overlap) {
  throw 'Use two distinct disposable rigs and two disjoint port pairs.'
}
$launcher = (Get-Command "$root/scripts/launch-client.ps1").Parameters
if (-not $launcher.ContainsKey('FollowTickRate') -or -not $launcher.ContainsKey('FullTickDemand')) {
  throw 'Integrate the wt/clock client and launcher changes before this run. See gate.md.'
}
foreach ($rig in @($Rig20, $Rig200)) {
  foreach ($marker in @('landing-motion.disposable','resource-pair.disposable')) {
    if (-not (Test-Path -LiteralPath (Join-Path $rig $marker) -PathType Leaf)) {
      throw "Missing $marker in disposable rig $rig."
    }
  }
}
& "$root/bot/build.ps1"
$jar = Join-Path $root 'bot/build/pathtracing.jar'
$probeOutput = Join-Path 'D:/path-tracing-target/1.6f' ('probe-' + [Guid]::NewGuid().ToString('N'))
$probe = & "$root/lab/gates/probes/server-phase/build.ps1" -Server $ReferenceServer -Output $probeOutput
$ps = (Get-Command powershell.exe).Source
$driver = Join-Path $root 'lab/gates/record-fixtures.ps1'
$oldRoot = $env:PATHTRACING_ROOT
$oldPort = $env:PATHTRACING_SERVER_PORT
function Capture([string]$Rig, [int]$Port, [int]$Rate) {
  $env:PATHTRACING_ROOT = $Rig
  $env:PATHTRACING_SERVER_PORT = $Port
  $output = @(& $ps -NoProfile -ExecutionPolicy Bypass -File $driver -Courses 'edges,full-blocks,ice' -Runs 1 -Ticks 400 -Seed 1 -Jar $jar -ResourceProbeJar $probe -LandingRate $Rate)
  $code = $LASTEXITCODE
  $output | ForEach-Object { Write-Host $_ }
  if ($code -ne 0) { throw "Raw $Rate TPS capture failed. See its report." }
  $report = $output | Where-Object { $_ -like '*report.md' } | Select-Object -Last 1
  if (-not $report) { throw "Missing $Rate TPS capture report." }
  Split-Path -Parent $report
}
try {
  $capture20 = Capture $Rig20 $Port20 20
  $capture200 = Capture $Rig200 $Port200 200
} finally {
  $env:PATHTRACING_ROOT = $oldRoot
  $env:PATHTRACING_SERVER_PORT = $oldPort
}
& "$PSScriptRoot/verify.ps1" -Capture20 $capture20 -Capture200 $capture200 -Edges20 $Edges20 -Edges200 $Edges200 -Teleport $Teleport -ReferenceServer $ReferenceServer
