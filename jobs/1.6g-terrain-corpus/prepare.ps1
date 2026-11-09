param(
  [Parameter(Mandatory)][string]$Target,
  [Parameter(Mandatory)][string]$ReferenceServer
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
if (Test-Path -LiteralPath $Target) { throw 'Choose a fresh verifier and build directory.' }
$null = New-Item -ItemType Directory -Path $Target
$Target = (Resolve-Path -LiteralPath $Target).Path
$reference = (Resolve-Path -LiteralPath $ReferenceServer).Path
$client = "$Target/client-source"
$null = New-Item -ItemType Directory -Path "$client/bot", "$client/scripts"
Copy-Item -LiteralPath "$root/bot/src" -Destination "$client/bot" -Recurse
Copy-Item -LiteralPath "$root/bot/build.ps1", "$root/bot/fabric.mod.json", "$root/bot/pathtracing.mixins.json" -Destination "$client/bot"
Copy-Item -LiteralPath "$root/scripts/common.ps1" -Destination "$client/scripts"
foreach ($path in @('bot/src/pathtracing/ClockAudit.java', 'bot/src/pathtracing/mixin/MinecraftClockMixin.java', 'scripts/launch-client.ps1')) {
  $source = git -C $root show "388a1bc:$path"
  if ($LASTEXITCODE) { throw "Cannot read the frozen lab clock source $path." }
  $source | Set-Content -LiteralPath "$client/$path" -Encoding ASCII
}
$mixins = Get-Content -LiteralPath "$client/bot/pathtracing.mixins.json" -Raw | ConvertFrom-Json
$mixins.client += 'MinecraftClockMixin'
$mixins | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath "$client/bot/pathtracing.mixins.json" -Encoding ASCII
& "$client/bot/build.ps1"
$null = & "$root/lab/gates/probes/server-phase/build.ps1" -Server $reference -Output "$Target/probe" -ClientLib "$client/bot/lib"
& "$PSScriptRoot/build-verifier.ps1" -Target $Target

# Record the exact comparison and generator inputs before the first live command.
$sources = @(Get-ChildItem -LiteralPath $PSScriptRoot -File | Where-Object { $_.Name -ne 'last.md' })
$sources += @(Get-Item -LiteralPath "$root/lab/gates/record-fixtures.ps1", "$root/lab/gates/courses.ps1",
  "$root/lab/gates/fixture-rig.ps1", "$root/lab/gates/check-recording.ps1",
  "$root/lab/gates/new-inputs.ps1", "$root/lab/gates/corner-inputs.ps1", "$root/lab/gates/check-l0-controls.ps1",
  "$root/jobs/1.6f-landing-motion/verify.rs", "$root/jobs/1.6f-landing-motion/WireCodec.java",
  "$root/engine/law/Cargo.toml", "$root/engine/law/tests/replay.rs",
  "$root/engine/law/tests/support/json.rs", "$root/engine/law/tests/support/server_resources.rs",
  "$client/bot/build/pathtracing.jar", "$client/scripts/launch-client.ps1", "$Target/probe/phase-probe.jar",
  "$Target/terrain-verify.exe", "$reference/versions/26.2/server-26.2.jar")
$sources += @(Get-ChildItem -LiteralPath "$root/engine/law/src" -File -Recurse)
$sources.FullName | Get-FileHash | ConvertTo-Json | Set-Content -LiteralPath "$Target/frozen-sha256.json" -Encoding ASCII
$reference | Set-Content -LiteralPath "$Target/reference-server.txt" -Encoding UTF8
"Prepared $Target"
