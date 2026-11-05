# Server-authoritative endpoints for a recording. The client never receives exhaustion.
param([Parameter(Mandatory, Position=0)][ValidatePattern('^[A-Za-z0-9_]{1,16}$')][string]$PlayerName)
$ErrorActionPreference = 'Stop'

function Read-Number([string]$Text) {
  if ($Text -notmatch '^(-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)[bBsSlLfFdD]?$') {
    throw "Expected an NBT number, got '$Text'"
  }
  [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
}

$fields = @('Health', 'foodLevel', 'foodSaturationLevel', 'foodExhaustionLevel', 'Pos', 'Motion', 'FallDistance')
$commands = @($fields | ForEach-Object { "data get entity $PlayerName $_" })
$replies = @(& "$PSScriptRoot\rcon.ps1" -Commands $commands)
if ($replies.Count -ne $fields.Count) { throw "Expected $($fields.Count) RCON replies, got $($replies.Count)" }
$state = [ordered]@{}
for ($i = 0; $i -lt $fields.Count; $i++) {
  if ($replies[$i] -notmatch ': (.+)$') { throw "Unexpected RCON reply: $($replies[$i])" }
  $value = $Matches[1].Trim()
  if ($fields[$i] -in @('Pos', 'Motion')) {
    if ($value -notmatch '^\[(.*)\]$') { throw "Expected an NBT vector, got '$value'" }
    $parts = $Matches[1].Split(',')
    if ($parts.Count -ne 3) { throw "Expected 3 vector components, got '$value'" }
    $state[$fields[$i]] = @($parts | ForEach-Object { Read-Number $_.Trim() })
  } else {
    $number = Read-Number $value
    $state[$fields[$i]] = if ($fields[$i] -eq 'foodLevel') { [int]$number } else { $number }
  }
}
$state | ConvertTo-Json -Compress
