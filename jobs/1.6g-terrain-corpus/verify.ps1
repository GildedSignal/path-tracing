param(
  [Parameter(Mandatory)][ValidateSet('Preflight','Audit','Full')][string]$Mode,
  [Parameter(Mandatory)][string]$Capture,
  [Parameter(Mandatory)][string]$Target,
  [Parameter(Mandatory)][string]$ReferenceServer,
  [string]$Course,
  [int]$Run,
  [int]$Prefix
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
. "$root/scripts/common.ps1"
. "$root/lab/gates/courses.ps1"
. "$PSScriptRoot/manifest.ps1"
$Capture = (Resolve-Path -LiteralPath $Capture).Path
$Target = (Resolve-Path -LiteralPath $Target).Path
$manifest = Get-TerrainManifest $Mode
$ticks = $manifest.Ticks + 20
$reduce = $PSBoundParameters.ContainsKey('Prefix')
if ($reduce -and (-not $Course -or $Run -lt 1 -or $Prefix -lt 0 -or $Prefix -ge $ticks)) {
  throw 'A reduction needs a single case and a proper replay prefix.'
}
if ([bool]$Course -ne ($Run -gt 0) -or $Run -gt $manifest.Runs -or
    ($Course -and $Course -cnotin $manifest.Courses.Name)) { throw 'Case is outside the frozen manifest.' }
if (-not $reduce) { $Prefix = $ticks }
$cp = @(
  "$ReferenceServer/versions/$McVersion/server-$McVersion.jar"
  (Get-ChildItem "$ReferenceServer/libraries" -Recurse -Filter '*.jar').FullName
) -join ';'
$ps51 = (Get-Command powershell.exe).Source

function Verify-Case($Definition, [int]$Index, [int]$Run) {
  $name = $Definition.Name
  $seed = $manifest.Seed + $Index * $manifest.Runs + $Run - 1
  $pattern = "^l0_(?<stamp>[0-9]{8}T[0-9]{13}Z)_$name" + "_$Run\.jsonl$"
  $files = @(Get-ChildItem -LiteralPath $Capture -File -Filter "*_${name}_$Run.jsonl" | Where-Object { $_.Name -cmatch $pattern })
  if ($files.Count -ne 1) { throw "$name run ${Run}: expected one new raw recording in $Capture, found $($files.Count)." }
  $file = $files[0]
  $stamp = [regex]::Match($file.Name, $pattern).Groups['stamp'].Value
  if ((Split-Path $Capture -Leaf) -cne "$stamp-fixtures") { throw "Recording belongs to a different capture: $($file.Name)." }
  $base = Join-Path $Capture $file.BaseName
  "Verifying $base (seed $seed; $ticks ticks)"
  foreach ($suffix in @('inputs.jsonl','world.jsonl','resources.csv','client-motion.csv','server-motion.csv')) {
    if (-not (Test-Path -LiteralPath "$base.$suffix" -PathType Leaf)) { throw "Incomplete new capture: missing $base.$suffix." }
  }

  # Regenerate from the frozen manifest, separately from the driver and its saved inputs.
  $previousRig = $env:PATHTRACING_ROOT
  $inputName = 'verify_' + [Guid]::NewGuid().ToString('N')
  $expected = Join-Path $Target "clients/bot/pathtracing/inputs/$inputName.jsonl"
  try {
    $env:PATHTRACING_ROOT = $Target
    if ($name -ceq 'mixed-seam') {
      New-TerrainMixedInputs $expected $Definition.Start $Run
    } elseif ($name -ceq 'corner') {
      & $ps51 -NoProfile -ExecutionPolicy Bypass -File "$root/lab/gates/corner-inputs.ps1" -Name $inputName -Ticks $manifest.Ticks -Start $Definition.Start
      if ($LASTEXITCODE) { throw "Could not regenerate $name controls." }
    } else {
      & $ps51 -NoProfile -ExecutionPolicy Bypass -File "$root/lab/gates/new-inputs.ps1" -Name $inputName -Ticks $manifest.Ticks `
        -Seed $seed -Start $Definition.Start -SneakPercent $Definition.SneakPercent
      if ($LASTEXITCODE) { throw "Could not regenerate $name controls." }
    }
    Add-TerrainRestTail $expected $manifest.Ticks
    if ((Get-FileHash -LiteralPath $expected).Hash -cne (Get-FileHash -LiteralPath "$base.inputs.jsonl").Hash) {
      throw "Inputs differ from the frozen manifest. Expected $expected; captured $base.inputs.jsonl."
    }
  } finally { $env:PATHTRACING_ROOT = $previousRig }
  Remove-Item -LiteralPath $expected

  $min = @($Definition.Min[0] - 3; $Definition.Min[1]; $Definition.Min[2] - 3)
  $max = @($Definition.Max[0] + 3; $Definition.Max[1] + 3; $Definition.Max[2] + 3)
  $env:L0_TERRAIN_RECORDING = $file.FullName
  $env:L0_TERRAIN_RATE = [string]$manifest.Rate
  $env:L0_TERRAIN_TICKS = [string]$ticks
  $env:L0_TERRAIN_PREFIX = [string]$Prefix
  $env:L0_TERRAIN_PRE_WORLD = Join-Path $Capture "l0_${stamp}_${name}_ready.world.jsonl"
  $env:L0_TERRAIN_ENVELOPE = "[$($min -join ',')];[$($max -join ',')]"
  $env:L0_TERRAIN_MIXED = ($name -ceq 'mixed-seam').ToString().ToLowerInvariant()
  $ErrorActionPreference = 'Continue'
  $replay = @(& "$Target/terrain-verify.exe" --exact terrain::authoritative_terrain_case --nocapture 2>&1 | ForEach-Object { "$_" })
  $status = $LASTEXITCODE
  $replay
  if ($status) {
    $first = [regex]::Match(($replay -join "`n"), '(?:failed: tick |first unsupported movement boundary at tick )([0-9]+)')
    if ($first.Success) {
      $tick = [int]$first.Groups[1].Value
      "Movement boundary at tick $tick. Compare prefixes of $tick and $($tick + 1) rows."
    }
    throw "Terrain replay refused $base. Retain this case and its first diagnostic."
  }
  & "$Jdk/java.exe" --class-path $cp "$PSScriptRoot/../1.6f-landing-motion/WireCodec.java" "$base.server-motion.csv" 2>&1 | ForEach-Object { "$_" }
  if ($LASTEXITCODE) { throw "Staged 26.2 codec refused $base.server-motion.csv." }
}

for ($i = 0; $i -lt $manifest.Courses.Count; $i++) {
  $definition = $manifest.Courses[$i]
  if ($Course -and $definition.Name -cne $Course) { continue }
  if ($Course) { Verify-Case $definition $i $Run }
  else {
    for ($r = 1; $r -le $manifest.Runs; $r++) { Verify-Case $definition $i $r }
  }
}
