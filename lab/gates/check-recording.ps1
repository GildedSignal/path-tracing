param([Parameter(Mandatory = $true, Position = 0)][string]$File)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$reader = $null
Enter-Rig 'check-recording'
try {
  $reader = [IO.File]::OpenText((Resolve-Path -LiteralPath $File).Path)
  $header = $reader.ReadLine() | ConvertFrom-Json
  if ($header.type -ne 'recording') { throw 'Missing recording header.' }
  foreach ($key in @('toggleSprint', 'toggleCrouch', 'autoJump')) {
    if ($header.$key -isnot [bool] -or $header.$key) { throw "Invalid hold setting: $key." }
  }
  if ($null -eq $header.sprintWindow -or $header.sprintWindow -ne 0) { throw 'Invalid sprintWindow.' }
  $lineNumber = 1
  $tick = 0
  $previous = $null
  $failed = $false
  while ($null -ne ($line = $reader.ReadLine())) {
    $lineNumber++
    $row = $line | ConvertFrom-Json
    if ($null -eq $row.t -or $row.t -ne $tick) {
      Write-Output "Tick stamp gap at line ${lineNumber}: expected $tick, got $($row.t)."
      $failed = $true
    }
    if ($null -ne $row.PSObject.Properties['anomaly']) {
      Write-Output "Anomaly at line $lineNumber, tick $($row.t): $line"
      $failed = $true
    }
    if ($null -eq $row.tickCount -or $row.tickCount -is [string] -or
        [double]$row.tickCount -ne [Math]::Truncate([double]$row.tickCount)) {
      Write-Output "Missing or invalid tickCount at line $lineNumber, tick $($row.t)."
      $failed = $true
    } elseif ($null -ne $previous -and ([long]$row.tickCount - [long]$previous) -ne 1) {
      Write-Output "tickCount gap at line $lineNumber, tick $($row.t): $previous -> $($row.tickCount)."
      $failed = $true
    }
    $previous = $row.tickCount
    $tick++
  }
  if ($tick -eq 0) { throw 'No recorded ticks.' }
  if ($failed) { exit 1 }
  Write-Output "Recording OK: $tick ticks, no anomalies or tickCount gaps."
} catch {
  Write-Output $_.Exception.Message
  exit 1
} finally {
  if ($null -ne $reader) { $reader.Dispose() }
  Exit-Rig
}
