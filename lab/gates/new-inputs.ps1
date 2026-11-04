param(
  [Parameter(Mandatory = $true)][ValidatePattern('^[A-Za-z0-9_-]+$')][string]$Name,
  [Parameter(Mandatory = $true)][ValidateRange(1, 2147483647)][int]$Ticks,
  [Parameter(Mandatory = $true)][int]$Seed,
  [Parameter(Mandatory = $true)][string[]]$Start
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"

$culture = [Globalization.CultureInfo]::InvariantCulture
$parts = ($Start -join ',').Split(',')
if ($parts.Count -ne 5) { throw 'Start must be x,y,z,yaw,pitch.' }
$origin = @($parts | ForEach-Object {
  $value = [double]::Parse($_, [Globalization.NumberStyles]::Float, $culture)
  if ([double]::IsNaN($value) -or [double]::IsInfinity($value)) { throw 'Start values must be finite.' }
  $value
})
foreach ($angle in $origin[3..4]) {
  if ([single]::IsInfinity([single]$angle)) { throw 'Start angles must fit a float.' }
}

$directory = Join-Path $Rig 'clients\bot\pathtracing\inputs'
$path = Join-Path $directory "$Name.jsonl"
$temporary = "$path.tmp"
Enter-Rig "new-inputs $Name"
try {
  New-Item -ItemType Directory -Force $directory | Out-Null
  $random = New-Object System.Random -ArgumentList $Seed
  $keys = @('forward', 'back', 'left', 'right', 'jump', 'sneak', 'sprint')
  $remaining = @{}
  $held = @{}
  $yaw = [single]$origin[3]
  $yawRemaining = 0
  $writer = New-Object IO.StreamWriter($temporary, $false, (New-Object Text.UTF8Encoding($false)))
  try {
    $writer.NewLine = "`n"
    $writer.WriteLine((@{ start = $origin } | ConvertTo-Json -Compress))
    for ($t = 0; $t -lt $Ticks; $t++) {
      $row = [ordered]@{ t = $t }
      foreach ($key in $keys) {
        if (-not $remaining.ContainsKey($key) -or $remaining[$key] -eq 0) {
          # Alternate pressed/released runs; a pressed run is always 1 through 20 ticks.
          $held[$key] = if ($held.ContainsKey($key)) { -not $held[$key] } else { $random.Next(2) -eq 1 }
          $remaining[$key] = $random.Next(1, 21)
        }
        $row[$key] = $held[$key]
        $remaining[$key]--
      }
      if ($yawRemaining -eq 0) {
        $step = $random.Next(-90, 91)
        if ($step -eq 0) { $step = 1 }
        $yaw = [single]($yaw + $step)
        $yawRemaining = $random.Next(1, 21)
      }
      $yawRemaining--
      $row.yaw = $yaw
      $row.pitch = [single]$origin[4]
      $writer.WriteLine(($row | ConvertTo-Json -Compress))
    }
  } finally { $writer.Dispose() }
  Move-Item -LiteralPath $temporary -Destination $path -Force
  Write-Output "Wrote $Ticks input ticks to $path (seed $Seed)."
} finally { Exit-Rig }
