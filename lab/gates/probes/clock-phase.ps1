# Diagnose archived clock samples without the gate classifier or movement comparator.
param(
  [Parameter(Mandatory)][string]$Samples,
  [Parameter(Mandatory)][string]$Recording
)
$ErrorActionPreference = 'Stop'
$brackets = Get-Content -LiteralPath $Samples -Raw | ConvertFrom-Json
if ($brackets.Count -lt 2) { throw 'Need at least two clock brackets.' }
$largestLower = $null
$smallestUpper = $null
$previous = $null
for ($i = 0; $i -lt $brackets.Count; $i++) {
  $sample = $brackets[$i]
  if ($sample.ClientAfter -lt $sample.ClientBefore -or
      ($null -ne $previous -and ($sample.Server -le $previous.Server -or
      $sample.ClientBefore -lt $previous.ClientAfter))) { throw "Invalid clock bracket at sample $i." }
  $sample | Add-Member -NotePropertyName Index -NotePropertyValue $i
  $sample | Add-Member -NotePropertyName Lower -NotePropertyValue ($sample.ClientBefore - $sample.Server)
  $sample | Add-Member -NotePropertyName Upper -NotePropertyValue ($sample.ClientAfter - $sample.Server)
  if ($null -eq $largestLower -or $sample.Lower -gt $largestLower.Lower) { $largestLower = $sample }
  if ($null -eq $smallestUpper -or $sample.Upper -lt $smallestUpper.Upper) { $smallestUpper = $sample }
  $previous = $sample
}

# A positive separation proves net phase change even if neither sample's exact offset is known.
$gap = $largestLower.Lower - $smallestUpper.Upper
$direction = if ($gap -le 0) { 'unresolved' } elseif ($largestLower.Index -lt $smallestUpper.Index) {
  'client lost phase'
} else { 'client gained phase' }
$steps = [Collections.Generic.Dictionary[long,int]]::new()
$prior = $null
$rows = 0
foreach ($line in [IO.File]::ReadLines((Resolve-Path -LiteralPath $Recording).Path)) {
  if ($line -cnotmatch '^\{"t":(\d+),"tickCount":(\d+),"gameTime":(\d+),') { continue }
  $clock = [long]$Matches[3]
  if ([int]$Matches[1] -ne $rows) { throw "Missing recorder row at $rows." }
  $rows++
  if ($null -ne $prior) {
    $step = $clock - $prior
    if (-not $steps.ContainsKey($step)) { $steps.Add($step, 0) }
    $steps[$step]++
  }
  $prior = $clock
}
[pscustomobject]@{
  Samples = $brackets.Count
  LargestLower = $largestLower
  SmallestUpper = $smallestUpper
  GuaranteedPhaseChangeTicks = [Math]::Max(0, $gap)
  Direction = $direction
  RecordingRows = $rows
  # ClientLevel gameTime reflects local extrapolation plus time-packet re-anchors.
  # These steps are a separate observation and do not estimate authoritative server TPS.
  ClientGameTimeSteps = @($steps.GetEnumerator() | Sort-Object Key | Select-Object Key, Value)
} | ConvertTo-Json -Depth 5
