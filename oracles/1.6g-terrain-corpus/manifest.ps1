# The existing course order determines the full-corpus seeds.
function Get-TerrainManifest([string]$Mode) {
  if ($Mode -eq 'Full') {
    return @{ Courses = @(Get-FixtureCourses); Runs = 8; Ticks = 400; Seed = 1; Rate = 200 }
  }

  $mixed = [pscustomobject]@{
    Name = 'mixed-seam'
    Start = '6160.5,-60,4108.5,0,0'
    Min = @(6144,-64,4096)
    Max = @(6175,-45,4127)
    Marker = @(6160,-45,4112)
    Commands = @(
      'fill 6144 -64 4096 6175 -45 4127 air'
      'fill 6144 -64 4096 6175 -61 4127 stone'
      'fill 6159 -60 4104 6159 -57 4122 stone'
      'fill 6161 -60 4104 6161 -57 4122 stone'
      'setblock 6160 -60 4122 stone'
      'fill 6160 -61 4106 6160 -61 4111 ice'
      'fill 6160 -60 4112 6160 -60 4113 stone_slab[type=bottom]'
      'fill 6160 -58 4110 6160 -58 4114 stone_slab[type=top]'
      'fill 6160 -61 4114 6160 -61 4115 ice'
      'fill 6160 -61 4116 6160 -61 4121 air'
      'setblock 6160 -61 4121 water[level=0]'
      'setblock 6160 -45 4112 gold_block'
    )
  }
  @{ Courses = @($mixed); Runs = 3; Ticks = 64; Seed = 1; Rate = $(if ($Mode -eq 'Preflight') { 2000 } else { 20 }) }
}

function New-TerrainMixedInputs([string]$Path, [string]$Start, [int]$Run) {
  $null = New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Path)
  $lines = @('{"start":[' + $Start + ']}')
  $lines += for ($tick = 0; $tick -lt 64; $tick++) {
    $reverse = $tick -eq (21 + $Run)
    $forward = (-not $reverse).ToString().ToLowerInvariant()
    $back = $reverse.ToString().ToLowerInvariant()
    $jump = ($tick -ge 28 -and $tick -lt 46).ToString().ToLowerInvariant()
    '{"t":' + $tick + ',"forward":' + $forward + ',"back":' + $back +
      ',"left":false,"right":false,"jump":' + $jump + ',"sneak":false,"sprint":false,"yaw":0,"pitch":0}'
  }
  $lines | Set-Content -LiteralPath $Path -Encoding ASCII
}

function Add-TerrainRestTail([string]$Path, [int]$Ticks) {
  $zero = (Get-Content -LiteralPath $Path | Select-Object -Last 1).Replace(':true', ':false')
  $tail = for ($tick = $Ticks; $tick -lt ($Ticks + 20); $tick++) {
    $zero -replace '"t":\d+', ('"t":' + $tick)
  }
  Add-Content -LiteralPath $Path -Value $tail -Encoding ASCII
}
