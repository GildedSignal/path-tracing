# One atomic entity NBT snapshot, bracketed by game-time queries for alignment evidence.
param([Parameter(Mandatory, Position=0)][ValidatePattern('^[A-Za-z0-9_]{1,16}$')][string]$PlayerName)
$ErrorActionPreference = 'Stop'

function Read-Number([string]$Text) {
  if ($Text -notmatch '^(-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)[bBsSlLfFdD]?$') {
    throw "Expected an NBT number, got '$Text'"
  }
  [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
}
function Split-Compound([string]$Text) {
  # Split only at top-level commas. Nested items and quoted strings can contain field names.
  $depth = 0; $quote = [char]0; $escape = $false; $begin = 0
  for ($i = 0; $i -lt $Text.Length; $i++) {
    $c = $Text[$i]
    if ($quote -ne [char]0) {
      if ($escape) { $escape = $false }
      elseif ($c -eq '\') { $escape = $true }
      elseif ($c -eq $quote) { $quote = [char]0 }
    } elseif ($c -eq '"' -or $c -eq "'") { $quote = $c }
    elseif ($c -eq '{' -or $c -eq '[') { $depth++ }
    elseif ($c -eq '}' -or $c -eq ']') { $depth-- }
    elseif ($c -eq ',' -and $depth -eq 0) { $Text.Substring($begin, $i-$begin).Trim(); $begin = $i+1 }
    if ($depth -lt 0) { throw 'Malformed entity NBT.' }
  }
  if ($depth -ne 0 -or $quote -ne [char]0) { throw 'Truncated entity NBT.' }
  $Text.Substring($begin).Trim()
}
$replies = @(& "$PSScriptRoot\rcon.ps1" -Commands @('time query gametime', "data get entity $PlayerName", 'time query gametime'))
if ($replies.Count -ne 3) { throw "Expected three RCON replies, got $($replies.Count)" }
$times = @()
foreach ($index in @(0, 2)) {
  if ($replies[$index] -cnotmatch '^The game time is (\d+) tick\(s\)\s*$') { throw "Unexpected game-time reply: $($replies[$index])" }
  $times += [long]$Matches[1]
}
if ($replies[1] -notmatch '(?s):\s*\{(.*)\}\s*$') { throw "Unexpected entity NBT reply: $($replies[1])" }
$fields = @{}
foreach ($entry in (Split-Compound $Matches[1])) {
  if ($entry -notmatch '^([A-Za-z0-9_]+|"[A-Za-z0-9_]+"|''[A-Za-z0-9_]+'')\s*:\s*(.*)$') { throw "Unexpected NBT entry: $entry" }
  $key = $Matches[1].Trim([char]34, [char]39)
  if ($fields.ContainsKey($key)) { throw "Duplicate NBT field $key" }
  $fields[$key] = $Matches[2]
}
$state = [ordered]@{ gameTimeBefore = $times[0]; gameTimeAfter = $times[1] }
foreach ($field in @('Health', 'foodLevel', 'foodSaturationLevel', 'foodExhaustionLevel', 'foodTickTimer', 'playerGameType', 'Pos', 'Motion', 'FallDistance', 'TicksFrozen')) {
  # Vanilla omits TicksFrozen when zero.
  if (-not $fields.ContainsKey($field)) {
    if ($field -eq 'TicksFrozen') { $state[$field] = 0; continue }
    throw "Entity snapshot missing $field"
  }
  if ($field -in @('Pos', 'Motion')) {
    if ($fields[$field] -notmatch '^\[(.*)\]$') { throw "Expected an NBT vector: $field" }
    $parts = $Matches[1].Split(',')
    if ($parts.Count -ne 3) { throw "Expected three vector components: $field" }
    $state[$field] = @($parts | ForEach-Object { Read-Number $_.Trim() })
  } else { $state[$field] = Read-Number $fields[$field] }
}
$state | ConvertTo-Json -Compress
