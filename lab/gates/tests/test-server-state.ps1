# Exercise the real snapshot reader with offline RCON replies, including nested/quoted SNBT.
$ErrorActionPreference = 'Stop'
$scratch = Join-Path $PSScriptRoot ('..\results\server-state-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Force $scratch
Copy-Item -LiteralPath "$PSScriptRoot\..\..\..\scripts\server-state.ps1" -Destination $scratch
$stub = @'
param([string[]]$Commands)
if (($Commands -join '|') -cne 'time query gametime|data get entity PathBot|time query gametime') { throw 'Snapshot was not one entity read.' }
if ($env:SNAPSHOT_TEST_CASE -eq 'malformed-time') { 'Unknown query 123' } else { 'The game time is 123 tick(s)' }
$nbt = '{Inventory:[{Health:99f,tag:{text:"a, b: {x}, \"quoted\""}}],Health:19.5f,foodLevel:18,foodSaturationLevel:2.5f,foodExhaustionLevel:1.25e-2f,foodTickTimer:37,playerGameType:2,Pos:[1.5d,-60d,4.5d],Motion:[0d,-0.0784d,0d],fall_distance:0.125d}'
switch ($env:SNAPSHOT_TEST_CASE) {
  'missing' { $nbt = $nbt.Replace('foodTickTimer:37,','') }
  'truncated' { $nbt = $nbt.Substring(0,$nbt.Length-5) }
  'frozen' { $nbt = $nbt.Replace('Health:19.5f','TicksFrozen:24,Health:19.5f') }
}
"PathBot has the following entity data: $nbt"
if ($env:SNAPSHOT_TEST_CASE -eq 'straddle') { 'The game time is 124 tick(s)' } else { 'The game time is 123 tick(s)' }
'@
Set-Content -LiteralPath (Join-Path $scratch 'rcon.ps1') -Value $stub -Encoding ASCII
$previous = $env:SNAPSHOT_TEST_CASE
try {
  foreach ($case in @('normal','frozen','straddle','missing','truncated','malformed-time')) {
    $env:SNAPSHOT_TEST_CASE = $case
    $reply = @(& "$PSHOME\powershell.exe" -NoProfile -Command "try { & '$scratch\server-state.ps1' PathBot } catch { Write-Output `$_; exit 1 }" 2>&1)
    if ($case -in @('missing','truncated','malformed-time')) {
      if ($LASTEXITCODE -eq 0) { throw "Accepted $case RCON reply" }
      if ($case -eq 'malformed-time' -and ($reply -join "`n") -notmatch 'Unexpected game-time reply') {
        throw 'Malformed game time was rejected for the wrong reason.'
      }
    } else {
      if ($LASTEXITCODE) { throw ($reply -join "`n") }
      $state = ($reply -join "`n") | ConvertFrom-Json
      if ($state.Health -ne 19.5 -or $state.foodTickTimer -ne 37 -or $state.foodExhaustionLevel -ne 0.0125 -or $state.playerGameType -ne 2) { throw 'Wrong resource snapshot.' }
      $expectedFrozen = if ($case -eq 'frozen') { 24 } else { 0 }
      $expectedAfter = if ($case -eq 'straddle') { 124 } else { 123 }
      if ($state.TicksFrozen -ne $expectedFrozen -or $state.gameTimeBefore -ne 123 -or $state.gameTimeAfter -ne $expectedAfter) { throw 'Wrong timing/frozen state.' }
      if ($state.Pos[1] -ne -60 -or $state.Motion[1] -ne -0.0784 -or $state.fall_distance -ne 0.125) { throw 'Wrong motion snapshot.' }
    }
    Write-Output "PASS server-state $case"
  }
} finally { $env:SNAPSHOT_TEST_CASE = $previous }
