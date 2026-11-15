param([string]$Output = 'D:\path-tracing-evidence\3.2d-3-A-baseline')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
. "$root/scripts/common.ps1"
if (Test-Path -LiteralPath (Join-Path $Output 'manifest.json')) { throw 'Baseline already frozen; preserve its original bytes.' }
New-Item -ItemType Directory -Force -Path $Output | Out-Null
$files = @('bot/src/pathtracing/Bot.java','bot/src/pathtracing/Recorder.java','bot/src/pathtracing/Tracks.java','scripts/play-track.ps1','engine/plan/src/bin/plan-track.rs')
$entries = @()
foreach ($file in $files) {
    $text = & git -C $root show "HEAD:$file" 2>$null
    if ($LASTEXITCODE) { continue }
    $target = Join-Path $Output ($file.Replace('/','_'))
    [IO.File]::WriteAllText($target, ($text -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
    $entries += @{ source = "HEAD:$file"; file = $target; sha256 = (Get-FileHash -LiteralPath $target).Hash }
}
foreach ($file in @((Join-Path $Rig 'clients/bot/mods/pathtracing.jar'), (Join-Path $Rig 'engine/target/release/plan-track.exe'), (Join-Path $Rig 'tracks.json'))) {
    if (!(Test-Path -LiteralPath $file)) { continue }
    $target = Join-Path $Output ([IO.Path]::GetFileName($file))
    Copy-Item -LiteralPath $file -Destination $target
    $entries += @{ source = $file; file = $target; sha256 = (Get-FileHash -LiteralPath $target).Hash }
}
@{ commit = (& git -C $root rev-parse HEAD); files = $entries; liveCaptured = $false; observerOnly = $true; note = 'Offline freeze only. CaptureBaseline live repetitions remain K; no candidate control has run.' } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Output 'manifest.json') -Encoding UTF8
Write-Output "Frozen baseline: $Output"
