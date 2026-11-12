# Job 3.2c's measure, offline: the class of vocabulary.md version 2 against version 1's on
# the 18 tracks (engine/words/tests/vocabulary.rs). Writes a run record: inputs, the
# machine's load before and after, the log, the measured lines, the witnesses.
param([string]$OutputRoot = 'D:\path-tracing-evidence\vocabulary-v2', [string]$Tracks = '')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$output = Join-Path $OutputRoot ([DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ'))
New-Item -ItemType Directory -Force -Path $output | Out-Null
$utf8 = New-Object Text.UTF8Encoding($false)
function Write-Text([string]$Path, [string]$Text) { [IO.File]::WriteAllText($Path, $Text, $utf8) }
function Load { [int](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average }
$before = Load
$env:VOCABULARY_RECORD = $output
if ($Tracks) { $env:VOCABULARY_TRACKS = $Tracks } else { Remove-Item Env:VOCABULARY_TRACKS -ErrorAction SilentlyContinue }
Push-Location (Join-Path $root 'engine')
try {
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $lines = @(& cargo test --offline --release -p words --test vocabulary -- --ignored --nocapture 2>&1 | ForEach-Object { "$_" })
    $code = $LASTEXITCODE
    $ErrorActionPreference = $previous
} finally { Pop-Location }
Write-Text (Join-Path $output 'log.txt') ($lines -join "`n")
Write-Text (Join-Path $output 'run.json') ([ordered]@{
    head = (& git -C $root rev-parse HEAD); dirty = @(& git -C $root status --short)
    toolchain = ((& rustc -Vv) -join "`n"); test = 'engine/words/tests/vocabulary.rs'; tracks = $Tracks
    started_utc = (Split-Path $output -Leaf); finished_utc = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
    cpu_load_percent = [ordered]@{ before = $before; after = (Load) }; exit_code = $code
    measured = @($lines | Where-Object { $_ -match '^hit rate |^\[' })
    refusals = @($lines | Where-Object { $_ -match 'refused|FAILED|panicked' })
} | ConvertTo-Json -Depth 4)
$lines | Where-Object { $_ -match 'all tracks|refused|test result' } | Write-Output
if ($code -ne 0) { throw "the measure failed. Run record $output" }
Write-Output "Run record $output"
