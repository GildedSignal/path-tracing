param([Parameter(Mandatory = $true, Position = 0)][string]$File)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$reader = $null
try {
  $reader = [IO.File]::OpenText((Resolve-Path -LiteralPath $File).Path)
  $header = $reader.ReadLine() | ConvertFrom-Json
  if ($header.type -ne 'dump' -or $null -eq $header.mainHand -or $null -eq $header.offHand) {
    throw 'Missing dump header or held items.'
  }
  $expected = @{
    'minecraft:ice' = @{ friction = [single]0.98 }
    'minecraft:blue_ice' = @{ friction = [single]0.989 }
    'minecraft:slime_block' = @{ friction = [single]0.8 }
    'minecraft:soul_sand' = @{ speedFactor = [single]0.4 }
    'minecraft:honey_block' = @{ speedFactor = [single]0.4; jumpFactor = [single]0.5 }
  }
  $seen = @{}
  $count = 0
  $summary = $null
  while ($null -ne ($line = $reader.ReadLine())) {
    if ($null -ne $summary) { throw 'Summary must be the final line.' }
    $row = $line | ConvertFrom-Json
    if ($row.type -eq 'summary') { $summary = $row; continue }
    if (-not $row.block) { throw "Missing block id after $count states." }
    $count++
    if ($expected.ContainsKey($row.block)) {
      $seen[$row.block] = $true
      foreach ($key in $expected[$row.block].Keys) {
        # Mojang stores these constants as floats; compare after the same conversion.
        if ($null -eq $row.$key -or [single]$row.$key -ne $expected[$row.block][$key]) {
          throw "Wrong $key on $($row.block): $($row.$key)."
        }
      }
    }
  }
  if ($null -eq $summary -or $null -eq $summary.writtenCount -or $null -eq $summary.registryCount) {
    throw 'Missing summary counts.'
  }
  if ($count -ne $summary.writtenCount -or $count -ne $summary.registryCount) {
    throw "Count mismatch: state lines $count, written $($summary.writtenCount), registry $($summary.registryCount)."
  }
  foreach ($block in $expected.Keys) { if (-not $seen.ContainsKey($block)) { throw "Missing $block." } }
  Write-Output "Dump OK: state lines $count, written $($summary.writtenCount), registry $($summary.registryCount); movement constants match."
} catch {
  Write-Output $_.Exception.Message
  exit 1
} finally {
  if ($null -ne $reader) { $reader.Dispose() }
}
