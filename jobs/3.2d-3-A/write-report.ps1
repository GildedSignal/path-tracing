param([string]$Evidence = 'D:\path-tracing-evidence\3.2d-3-A')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$manifestPath = Join-Path $Evidence 'manifest.json'
$manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
if (!$manifest.passed) { throw 'Cannot write a passed report without the gate manifest.' }
Push-Location $root
try {
    $changed = @(& git diff --name-only) + @(& git ls-files --others --exclude-standard) + @('jobs/3.2d-3-A/last.md')
    $owned = @($changed | Sort-Object -Unique | Where-Object { $_ -notlike 'engine/world/tests/fixtures/world/*' })
    $lines = @(
        '# 3.2d-3 A / R0 - offline result',
        '',
        'Consumer: observed-state capture for P1/P4; resident channel for E/F/K.',
        "Evidence: $Evidence/manifest.json (exact inputs, source pins, commands, raw outputs, refusals).",
        'Baseline: D:/path-tracing-evidence/3.2d-3-A-baseline/manifest.json; inputs.json freezes the original rig definitions/kits and model inputs.',
        '',
        '## Built',
        '',
        '- Handwritten JNI, dependency-free cdylib, QPC counter/frequency, one F worker. World join resets the resident source/table; models parse on the worker.',
        '- Exact state words, known/provenance masks, input phase, session/source version, dimension/body and lossless registry-aware kit context. Missing words refuse before search.',
        '- Client packet-applied section palettes retain canonical fine states; block/unload deltas update the same source. Disk presence grants no loaded lease. No far source is configured: unreceived cells are Unknown, never read from mutable regions.',
        '- goto/come use RequestGoal; Java StraightLine is removed. Results use the explicitly temporary legacy_sentence_adapter in memory. No settle, subprocess, log polling or script export is reachable from resident goal dispatch.',
        '- ObserveEvent retains the actual bits/masks for P1/P4. Cancel/source versions invalidate pending work; exact start/source mismatch refuses dispatch.',
        '- Passive capture records raw issuance/mod/JNI/result stamps, all real packet sequence receipts/application, actual server tick starts and authoritative server resource rows. Observer-only jar contains no candidate controller/planner and is built beside the frozen baseline.',
        '',
        '## Gate commands and outputs',
        '',
        '`./jobs/3.2d-3/verify.ps1 -Case Slice -Slice A -Difficulty Peaceful -Mobs Off` -> OFFLINE PASS.',
        '',
        "- engine/: cargo test --offline -> $($manifest.workspaceTotals.passed) passed, $($manifest.workspaceTotals.ignored) ignored; existing absent-fixture/release-only ignores are preserved.",
        "- jobs/1.2-footprints/verify.ps1 -> all $($manifest.footprintMutants.Count) mutants read dynamically from original call sites CAUGHT; restored tests PASS; randomness audit PASS; all pinned source hashes restored.",
        '- bot/build.ps1 -> built pathtracing.jar.',
        '- bot/tests/test-tracks.ps1 -> PASS (inventory, track mode, compiled injection audit and headless track checks).',
        '- bot/tests/test-native.ps1 -> every field and known/provenance mask Java -> Rust -> Java; every unknown refused; pinned recording retained; QPC monotonic/frequency checks PASS.',
        '- bot/tests/test-resident-path.ps1 -> no old click-path dependency; issuance stamp is first statement after consuming a click and first command statement.',
        '- jobs/3.2d-3/build-observer.ps1 -> observer-only.jar built without installing it.',
        '',
        'Raw logs:',
        ''
    )
    $lines += @($manifest.commands | ForEach-Object { "- $($_.name): $($_.log), exit $($_.exit)" })
    $lines += @('','## Files changed','')
    $lines += @($owned | ForEach-Object { '- ' + $_ })
    $lines += @(
        '',
        'Pre-existing fixture changes preserved: engine/world/tests/fixtures/world/dimensions/minecraft/overworld/region/r.0.-1.mca and r.0.0.mca, plus their .bak-20261114 files. They were supplied before this job; no fixture replacement was made here.',
        '',
        '## Remaining slices / limits',
        '',
        '- E: adopt the proved observed plain arithmetic and eligibility rules after T1-T4; this job does not use plain.rs or clear its failed judge findings.',
        '- F: replace legacy_sentence_adapter with WordPlan, shared event comparator, ownership manifest and exact phase-preserving re-anchor. A refuses an asynchronously stale start; it does not invent a settled replacement.',
        '- K: install only the passive observer beside the frozen original binary, capture the actual Peaceful/mobs-off baseline, supply aligned authoritative resource observations and verify every live hop/receipt/application/restoration. Stock remote clients lack independent server hurt/exhaustion/timer: those required words remain Unknown and current requests refuse rather than fabricate them.',
        '- The live harness, actual source/difficulty/loaded qualification, bracket and T_live remain K. CaptureBaseline/live cases in verify.ps1 refuse absent live evidence; check-hop-clock.ps1 refuses missing origins, real input receipt/application or restoration.',
        '- No client/server/game was launched, no live T or throughput was measured, and no baseline LIVE run or observer-only run is claimed.',
        '- Earlier exploratory worker witness was stopped because it searched a route rather than answering transport; the committed witness checks exact observed entry and in-memory output directly. No production search/gate was weakened.',
        '- First gate wrapper attempt hit PowerShell native-stderr handling; the committed wrapper records child-process exit codes and raw output. No failed run was called passing.',
        '',
        'Worst margin: bit equality, with no tolerance. All expected refusals and replay inputs are in the gate manifest/logs. No unexpected live anomaly was produced.',
        '',
        'STATUS: DONE'
    )
    [IO.File]::WriteAllLines((Join-Path $PSScriptRoot 'last.md'),$lines,[Text.UTF8Encoding]::new($false))
    foreach ($entry in $manifest.files) {
        if ($entry.path -in @('jobs/3.2d-3-A/last.md','jobs/3.2d-3-A/write-report.ps1')) {
            $entry.sha256 = (Get-FileHash -LiteralPath $entry.path).Hash
        }
    }
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
} finally { Pop-Location }
