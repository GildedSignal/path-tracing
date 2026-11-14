# Job 3.2b's flight, offline: the planner on the 18 tracks (engine/plan/tests/flight.rs).
# Writes a run record: inputs, the machine's load before and after, the report, the log.
param([string]$OutputRoot = 'D:\path-tracing-evidence\planner', [string]$Tracks = '', [string]$Modes = '3,5,b', [string]$Cap = '20', [switch]$Audit)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$output = Join-Path $OutputRoot ([DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ'))
New-Item -ItemType Directory -Force -Path $output | Out-Null
$utf8 = New-Object Text.UTF8Encoding($false)
function Write-Text([string]$Path, [string]$Text) { [IO.File]::WriteAllText($Path, $Text, $utf8) }
function Load { [int](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average }
$before = Load
$env:PLAN_RECORD = $output; $env:PLAN_MODES = $Modes; $env:PLAN_CAP = $Cap
if ($Tracks) { $env:PLAN_TRACKS = $Tracks } else { Remove-Item Env:PLAN_TRACKS -ErrorAction SilentlyContinue }
if ($Audit) { $env:PLAN_AUDIT = '1' } else { Remove-Item Env:PLAN_AUDIT -ErrorAction SilentlyContinue }
Push-Location (Join-Path $root 'engine')
try {
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $lines = @(& cargo test --offline --release -p plan --test flight -- --ignored --nocapture 2>&1 | ForEach-Object { "$_" })
    $code = $LASTEXITCODE
    $ErrorActionPreference = $previous
} finally { Pop-Location }
Write-Text (Join-Path $output 'log.txt') ($lines -join "`n")
Write-Text (Join-Path $output 'run.json') ([ordered]@{
    head = (& git -C $root rev-parse HEAD); dirty = @(& git -C $root status --short)
    toolchain = ((& rustc -Vv) -join "`n"); test = 'engine/plan/tests/flight.rs'; tracks = $Tracks; modes = $Modes; cap = $Cap; audit = [bool]$Audit
    started_utc = (Split-Path $output -Leaf); finished_utc = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
    cpu_load_percent = [ordered]@{ before = $before; after = (Load) }; exit_code = $code
    sentences = @($lines | Where-Object { $_ -match 'SENTENCE certified|all tracks' })
    refusals = @($lines | Where-Object { $_ -match 'NO SENTENCE|HIS T BEATS OURS|ANOMALY|the law refused|mirror failed|FAILED|panicked' })
} | ConvertTo-Json -Depth 4)
$lines | Where-Object { $_ -match 'all tracks|NO SENTENCE|HIS T BEATS|ANOMALY|test result' } | Write-Output
if ($code -ne 0) { throw "the flight failed. Run record $output" }
Write-Output "Run record $output; cpu load before $before"
