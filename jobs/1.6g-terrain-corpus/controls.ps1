param(
  [Parameter(Mandatory)][string]$EvidenceRoot,
  [Parameter(Mandatory)][string]$ClockEvidence,
  [Parameter(Mandatory)][string]$ReferenceServer,
  [Parameter(Mandatory)][string]$Target
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
. "$root/scripts/common.ps1"
$paths = @{}
foreach ($line in Get-Content -LiteralPath "$PSScriptRoot/movement-controls.tsv") {
  $name, $path, $recording, $world = $line -split "`t"
  $base = Join-Path $EvidenceRoot $path
  if ((Get-FileHash -LiteralPath "$base.jsonl").Hash -cne $recording -or
      (Get-FileHash -LiteralPath "$base.world.jsonl").Hash -cne $world) {
    throw "Archived movement control changed: $base."
  }
  $paths[$name] = "$base.jsonl"
}
# The archived controls are adjudicated against the same law as the corpus.
git -C $root diff --quiet HEAD -- engine/law/src engine/law/Cargo.toml
if ($LASTEXITCODE) { throw 'The movement law must be in the selected revision before running the frozen mutant gate.' }
& "$root/lab/gates/check-l0-controls.ps1" @paths

$edges20 = Join-Path $ClockEvidence '20261106T1257229131311Z-fixtures/l0_20261106T1257229131311Z_edges_1.jsonl'
$edges200 = Join-Path $ClockEvidence '20261106T1249512984107Z-fixtures/l0_20261106T1249512984107Z_edges_1.jsonl'
$teleport = Join-Path $EvidenceRoot '20261105T0838027621258Z-gate-1.2/g12_20261105T0838027621258Z_negative.jsonl'
& "$root/jobs/1.6e-landing-motion/check-refusals.ps1" -Edges20 $edges20 -Edges200 $edges200 -Teleport $teleport

# These saved 1.6f captures are positive controls. They keep the v1 witness.
$saved = Join-Path $EvidenceRoot '1.6f-20261107T0920057086048Z/integration/lab/gates/results'
$env:L0_EDGES_20 = $edges20
$env:L0_LANDING_20 = Join-Path $saved '20261107T0922271381133Z-fixtures'
$env:L0_LANDING_200 = Join-Path $saved '20261107T0924458090763Z-fixtures'
$ErrorActionPreference = 'Continue'
& "$Target/terrain-verify.exe" --exact live_landing_motion --nocapture 2>&1 | ForEach-Object { "$_" }
if ($LASTEXITCODE) { throw 'Archived 1.6f movement or provenance control failed.' }
$cp = @(
  "$ReferenceServer/versions/$McVersion/server-$McVersion.jar"
  (Get-ChildItem "$ReferenceServer/libraries" -Recurse -Filter '*.jar').FullName
) -join ';'
$wire = (Get-ChildItem -LiteralPath $env:L0_LANDING_20, $env:L0_LANDING_200 -Filter '*.server-motion.csv').FullName
& "$Jdk/java.exe" --class-path $cp "$root/jobs/1.6f-landing-motion/WireCodec.java" @wire 2>&1 | ForEach-Object { "$_" }
if ($LASTEXITCODE) { throw 'Archived 1.6f codec control failed.' }
