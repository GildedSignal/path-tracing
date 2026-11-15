param([switch]$Diagnostic, [switch]$ReportOnly)
$ErrorActionPreference = 'Stop'
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskEvidence = Join-Path $taskRoot 'jobs/hard-track'
$taskEncoding = New-Object Text.UTF8Encoding($false)
$oldTarget = $env:CARGO_TARGET_DIR
$oldRecord = $env:HARD_RECORD
$oldTrace = $env:HARD_TRACE
function Write-TaskText($path, $value) { [IO.File]::WriteAllText($path, $value, $taskEncoding) }
function Run-TaskCargo([string[]]$CargoArgs, [string]$LogName) {
    $command = 'cargo ' + ($CargoArgs -join ' ')
    Add-Content -LiteralPath (Join-Path $taskEvidence 'commands.txt') -Value $command
    $ErrorActionPreference = 'Continue' # Native warning stderr is data under Windows PowerShell 5.1.
    & cargo @CargoArgs 2>&1 | ForEach-Object { $_.ToString() } | Tee-Object -FilePath (Join-Path $taskEvidence $LogName)
    $taskExit = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    Add-Content -LiteralPath (Join-Path $taskEvidence 'commands.txt') -Value ('exit=' + $taskExit)
    if ($taskExit -ne 0) { throw "$command failed with exit $taskExit" }
}
try {
    $env:CARGO_TARGET_DIR = 'D:\path-tracing-target\hard-track'
    $env:HARD_RECORD = '1'
    Remove-Item Env:HARD_TRACE -ErrorAction SilentlyContinue
    if (-not $ReportOnly) {
        Write-TaskText (Join-Path $taskEvidence 'commands.txt') "CARGO_TARGET_DIR=D:\path-tracing-target\hard-track`nHARD_RECORD=1`nworking directory: engine/`n"
        Add-Content -LiteralPath (Join-Path $taskEvidence 'commands.txt') -Value 'powershell -NoProfile -File jobs/hard-track/build-facts.ps1 (offline javap and fill-volume audit)'
        & (Join-Path $taskEvidence 'build-facts.ps1')
        Push-Location (Join-Path $taskRoot 'engine')
        try {
            Run-TaskCargo -CargoArgs @('test','--offline') -LogName 'cargo-test.log'
            if ($Diagnostic) { Run-TaskCargo -CargoArgs @('test','--offline','--release','-p','plan','--test','hard_track','current_planner_mode_5','--','--ignored','--nocapture') -LogName 'planner.log' }
        } finally { Pop-Location }
    }
    $witness = Get-Content -LiteralPath (Join-Path $taskEvidence 'witness.md') -Raw -Encoding UTF8
    $bounds = Get-Content -LiteralPath (Join-Path $taskEvidence 'bounds.md') -Raw -Encoding UTF8
    $planner = Get-Content -LiteralPath (Join-Path $taskEvidence 'planner.md') -Raw -Encoding UTF8
    $values = @{
        'witness_ticks' = [regex]::Match($witness,'first B touch: (\d+)').Groups[1].Value
        'race_ticks' = [regex]::Match($witness,'detector clock: (\d+)').Groups[1].Value
        'rebound_margin' = [regex]::Match($witness,'vertical margin ([\d.]+)').Groups[1].Value
        'stone_reach' = [regex]::Match($bounds,'airborne samples: ([\d.]+)').Groups[1].Value
        'jump_apex' = [regex]::Match($bounds,'apex from rest: ([\d.]+)').Groups[1].Value
        'north_apex' = [regex]::Match($bounds,'rebound apex (-?[\d.]+)').Groups[1].Value
        'planner_result' = $(if ($planner -match 'Sentence: None') {'No sentence'} else {'See the recorded sentence'})
    }
    $report = Get-Content -LiteralPath (Join-Path $taskEvidence 'report.template.md') -Raw -Encoding UTF8
    foreach ($key in $values.Keys) {
        if ([string]::IsNullOrEmpty($values[$key])) { throw "missing generated metric: $key" }
        $report = $report.Replace('{{' + $key + '}}', $values[$key])
    }
    if ($report.Contains('{{')) { throw 'unexpanded report metric' }
    Write-TaskText (Join-Path $taskEvidence 'report.md') $report
    $inputs = @('lab/tracks/hardtrack.build.txt','lab/tracks/hardtrack.witness.txt','lab/tracks/hardtrack.json','lab/tracks/hardtrack.tracks.txt','engine/plan/tests/hard_track.rs','jobs/hard-track/check.ps1','jobs/hard-track/build-facts.ps1','jobs/hard-track/report.template.md')
    $inputs += Get-ChildItem -LiteralPath (Join-Path $taskRoot 'engine/law/src') -File | ForEach-Object { 'engine/law/src/' + $_.Name }
    $inputs += Get-ChildItem -LiteralPath (Join-Path $taskRoot 'engine/world/data') -File | ForEach-Object { 'engine/world/data/' + $_.Name }
    $hashes = foreach ($relative in $inputs) { (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $taskRoot $relative)).Hash + '  ' + $relative }
    Write-TaskText (Join-Path $taskEvidence 'inputs.sha256') (($hashes -join "`n") + "`n")
    if ($ReportOnly) { Write-Output 'Hard-track report refreshed from existing run records.' }
    else { Write-Output 'Hard-track offline checks and report complete.' }
} finally {
    $env:CARGO_TARGET_DIR = $oldTarget
    $env:HARD_RECORD = $oldRecord
    $env:HARD_TRACE = $oldTrace
}
