param([switch]$ReportOnly)
$ErrorActionPreference = 'Stop'
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskEncoding = [Text.UTF8Encoding]::new($false)
function Write-TaskText($name, $value) { [IO.File]::WriteAllText((Join-Path $PSScriptRoot $name), $value, $taskEncoding) }
function Metric($source, $pattern, $group = 1) {
    $match = [regex]::Match($source, $pattern)
    if (-not $match.Success) { throw "Missing generated metric: $pattern" }
    $match.Groups[$group].Value
}
function Counts($name) {
    $passed = 0; $ignored = 0
    [regex]::Matches((Get-Content (Join-Path $PSScriptRoot $name) -Raw -Encoding UTF8), 'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored') | ForEach-Object {
        $passed += [int]$_.Groups[1].Value; $ignored += [int]$_.Groups[2].Value
    }
    [ordered]@{ passed = $passed; ignored = $ignored }
}
function Input-Hashes {
    $paths = @('engine/Cargo.toml','engine/Cargo.lock','engine/plan/tests/hard_track.rs',
        'lab/tracks/hardtrack.build.txt','lab/tracks/hardtrack.witness.txt','lab/tracks/hardtrack.json',
        'jobs/hard-track-2/old.build.txt','jobs/hard-track-2/old.witness.txt',
        'jobs/hard-track-2/check.ps1','jobs/hard-track-2/report.template.md')
    foreach ($crate in @('law','plan','words','world')) {
        $paths += "engine/$crate/Cargo.toml"
        foreach ($folder in @('src','tests')) {
            $paths += Get-ChildItem (Join-Path $taskRoot "engine/$crate/$folder") -Recurse -File |
                Where-Object { $_.Extension -in '.rs','.bin' } |
                ForEach-Object { $_.FullName.Substring($taskRoot.Length+1).Replace('\','/') }
        }
    }
    $paths += Get-ChildItem (Join-Path $taskRoot 'engine/world/data') -File | ForEach-Object { 'engine/world/data/' + $_.Name }
    $paths += Get-ChildItem (Join-Path $taskRoot 'engine/world/tests/fixtures/world') -Recurse -File -Filter '*.mca' |
        ForEach-Object { $_.FullName.Substring($taskRoot.Length+1).Replace('\','/') }
    foreach ($path in ($paths | Sort-Object -Unique)) {
        (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $taskRoot $path)).Hash + '  ' + $path
    }
}
function Run-TaskCargo([string[]]$Arguments, [string]$Log) {
    Add-Content -LiteralPath (Join-Path $PSScriptRoot 'commands.txt') -Value ('cargo ' + ($Arguments -join ' '))
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $ErrorActionPreference = 'Continue' # Native stderr warnings are evidence in Windows PowerShell 5.1.
    & cargo @Arguments 2>&1 | ForEach-Object { $_.ToString() } | Tee-Object -FilePath (Join-Path $PSScriptRoot $Log)
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    $clock.Stop()
    Add-Content -LiteralPath (Join-Path $PSScriptRoot 'commands.txt') -Value "exit=$code wall_seconds=$($clock.Elapsed.TotalSeconds)"
    if ($code -ne 0) { throw "cargo failed with exit $code; see $Log" }
    [PSCustomObject]@{ exit = $code; seconds = $clock.Elapsed.TotalSeconds }
}
$taskOldEnvironment = @{}
foreach ($name in @('PATH','RUSTUP_HOME','CARGO_HOME','CARGO_TARGET_DIR','HARD_RECORD')) { $taskOldEnvironment[$name] = [Environment]::GetEnvironmentVariable($name) }
try {
    $env:PATH = 'D:\rust\cargo\bin;' + $env:PATH
    $env:RUSTUP_HOME = 'D:\rust\rustup'; $env:CARGO_HOME = 'D:\rust\cargo'
    $env:CARGO_TARGET_DIR = 'D:\path-tracing-target\hard-track-2'; $env:HARD_RECORD = '1'
    if (-not $ReportOnly) {
        $before = @(Input-Hashes)
        Write-TaskText 'inputs.sha256' (($before -join "`n") + "`n")
        Write-TaskText 'commands.txt' "working_directory=engine`nCARGO_TARGET_DIR=$env:CARGO_TARGET_DIR`nHARD_RECORD=1`n"
        Push-Location (Join-Path $taskRoot 'engine')
        try {
            # Capture stdout separately: Run-TaskCargo emits both the log and its final timing object.
            $first = @(Run-TaskCargo @('test','--offline','-p','plan','--test','hard_track','--','--nocapture') 'hard-track.log')
            $first | Where-Object { $_ -is [string] } | Write-Output
            $hard = $first[-1]
            $second = @(Run-TaskCargo @('test','--offline','--workspace') 'workspace.log')
            $second | Where-Object { $_ -is [string] } | Write-Output
            $workspace = $second[-1]
        } finally { Pop-Location }
        $after = @(Input-Hashes)
        if (($before -join "`n") -ne ($after -join "`n")) { throw 'An input changed during the gates; do not publish this run.' }
        $gate = [ordered]@{ commit = (git -C $taskRoot rev-parse HEAD); hard_track = $hard; workspace = $workspace
            hard_track_counts = (Counts 'hard-track.log'); workspace_counts = (Counts 'workspace.log')
            target = $env:CARGO_TARGET_DIR; completed_utc = [DateTime]::UtcNow.ToString('o'); track_refusals = 0 }
        Write-TaskText 'gates.json' ($gate | ConvertTo-Json -Depth 5)
    }
    if ((Get-Content (Join-Path $PSScriptRoot 'inputs.sha256') -Raw -Encoding UTF8).Trim() -ne ((@(Input-Hashes) -join "`n").Trim())) { throw 'Recorded gates do not match current inputs.' }
    $gate = Get-Content (Join-Path $PSScriptRoot 'gates.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($gate.hard_track.exit -ne 0 -or $gate.workspace.exit -ne 0) { throw 'Both required gates must pass.' }
    $new = Get-Content (Join-Path $PSScriptRoot 'new-graph.md') -Raw -Encoding UTF8
    $old = Get-Content (Join-Path $PSScriptRoot 'old-graph.md') -Raw -Encoding UTF8
    $witness = Get-Content (Join-Path $PSScriptRoot 'witness.md') -Raw -Encoding UTF8
    $critical = Get-Content (Join-Path $PSScriptRoot 'critical-exclusions.md') -Raw -Encoding UTF8
    $timing = Get-Content (Join-Path $PSScriptRoot 'timing.md') -Raw -Encoding UTF8
    $track = Get-Content (Join-Path $taskRoot 'lab/tracks/hardtrack.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    $volumes = @()
    foreach ($line in Get-Content (Join-Path $taskRoot 'lab/tracks/hardtrack.build.txt') -Encoding UTF8) {
        $fields = $line.Split('#')[0].Trim() -split '\s+'
        if ($fields[0] -eq '/fill') {
            $volume = [long]1
            for ($axis=0; $axis -lt 3; $axis++) { $volume *= ([int]$fields[$axis+4] - [int]$fields[$axis+1] + 1) }
            $volumes += $volume
        }
    }
    $metrics = @{
        'A' = ($track.A -join ', '); 'B' = ($track.B -join ', ')
        'old_nodes' = (Metric $old 'Nodes: (\d+)'); 'old_edges' = (Metric $old 'edges: (\d+)')
        'nodes' = (Metric $new 'Nodes: (\d+)'); 'edges' = (Metric $new 'edges: (\d+)')
        'speed' = (Metric $new 'Speed bound: (\d+(?:\.\d+)?)'); 'apex' = (Metric $new 'jump apex: (\d+(?:\.\d+)?)')
        'margins' = (Metric $new 'component deficit\): (\[[^\]]+\])')
        'rotation' = (Metric $critical 'every f32 yaw: (\d+(?:\.\d+)?)')
        'step_height' = (Metric $critical 'step-height: (\d+(?:\.\d+)?)')
        'decoy1' = (Metric $new 'Decoy 1: (\d+)'); 'decoy2' = (Metric $new 'Decoy 2: (\d+)'); 'decoy3' = (Metric $new 'Decoy 3: (\d+)')
        'old_excluded' = (Metric $old 'Local excluded edges: (\d+)'); 'excluded' = (Metric $new 'Local excluded edges: (\d+)')
        'ticks' = (Metric $witness 'first B touch: (\d+)'); 'clock' = (Metric $witness 'detector clock: (\d+)')
        'digest' = (Metric $witness 'digest, including the start and every tick: `([^`]+)`')
        'rebound_margin' = (Metric $witness 'vertical margin ([\d.]+)')
        'timing_distribution' = (Metric $timing 'Successful perturbations: ([^\r\n]+)')
        'first_volume' = [string]$volumes[0]; 'later_volume' = [string](($volumes | Select-Object -Skip 1 | Measure-Object -Maximum).Maximum)
        'hard_passed' = [string]$gate.hard_track_counts.passed; 'hard_ignored' = [string]$gate.hard_track_counts.ignored
        'workspace_passed' = [string]$gate.workspace_counts.passed; 'workspace_ignored' = [string]$gate.workspace_counts.ignored
    }
    $report = Get-Content (Join-Path $PSScriptRoot 'report.template.md') -Raw -Encoding UTF8
    foreach ($key in $metrics.Keys) { $report = $report.Replace('{{' + $key + '}}', $metrics[$key]) }
    if ($report.Contains('{{')) { throw 'Unexpanded report metric.' }
    Write-TaskText 'report.md' $report
    $inventory = @('engine/plan/tests/hard_track.rs','lab/tracks/hardtrack.build.txt','lab/tracks/hardtrack.witness.txt','jobs/hard-track-2/files.txt')
    $inventory += Get-ChildItem $PSScriptRoot -File | ForEach-Object { 'jobs/hard-track-2/' + $_.Name }
    Write-TaskText 'files.txt' ((($inventory | Sort-Object -Unique) -join "`n") + "`n")
    $last = @'
# Hard track 2: complete

The rebuilt course passes the ordered whole-box reach certificate, all committed decoy components are closed, and the old wall shortcut is retained as a counterexample only in the old build. See [report.md](report.md), [new-graph.md](new-graph.md), and [critical-exclusions.md](critical-exclusions.md).

The witness is frozen bit for bit and replayed through the existing planner certificate. Timing outcomes and mechanic-removal controls are in [timing.md](timing.md) and [controls.md](controls.md).

Both required offline commands passed with the prescribed target directory. [gates.json](gates.json) records counts, exit codes and wall times; [commands.txt](commands.txt) records commands; [inputs.sha256](inputs.sha256) pins replay inputs; [files.txt](files.txt) lists delivered files. Run `powershell -NoProfile -File jobs/hard-track-2/check.ps1` from the worktree root to regenerate the evidence and report.

No law/planner source, bot code, track metadata, Julien recording, or shortcut record was changed. The pre-existing fixture-region changes were left untouched. No game, server, client or RCON was opened. No launcher, commit or push was used in this continuation. The integrator judges and places the ordered build in the world.

Built-by: Terra
Bought: a whole-box conservative reach proof that retains Julien's old shortcut as a negative control, closes every skipping path and committed decoy exit in the rebuilt course, and supplies a replayable race witness.

STATUS: DONE
'@
    Write-TaskText 'last.md' ($last + "`n")
    Write-Output 'Both offline gates and regenerated hard-track-2 report passed.'
} finally {
    foreach ($name in $taskOldEnvironment.Keys) { [Environment]::SetEnvironmentVariable($name, $taskOldEnvironment[$name]) }
}
