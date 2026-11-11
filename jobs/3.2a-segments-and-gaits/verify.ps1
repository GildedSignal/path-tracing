# Job 3.2a's numbers and gates, all offline: the words tests in release, the workspace,
# the footprint mutants, and the retained recorder pairs replayed on main and here.
# Writes a run record: inputs, the measured lines, refusals.
param([string]$OutputRoot = 'D:\path-tracing-evidence\segments', [switch]$SkipGates)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$output = Join-Path $OutputRoot ([DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ'))
New-Item -ItemType Directory -Force -Path $output | Out-Null
$utf8 = New-Object Text.UTF8Encoding($false)
function Write-Text([string]$Path, [string]$Text) { [IO.File]::WriteAllText($Path, $Text, $utf8) }
Write-Text (Join-Path $output 'inputs.json') ([ordered]@{
    head = (& git -C $root rev-parse HEAD); dirty = @(& git -C $root status --short)
    toolchain = ((& rustc -Vv) -join "`n"); tests = 'engine/words/tests/compiler.rs'
} | ConvertTo-Json -Depth 4)
$measured = 'gaits ground=|us/segment|second column|hit rate \[|state fields read|width of|CAUGHT|PASS|IDENTICAL|^test result:'
function Run([string]$Name, [string]$Program, [string[]]$Arguments) {
    $previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $lines = @(& $Program @Arguments 2>&1 | ForEach-Object { "$_" })
        $code = $LASTEXITCODE
    } finally { $ErrorActionPreference = $previous }
    Write-Text (Join-Path $output "$Name.log") ($lines -join "`n")
    Write-Text (Join-Path $output "$Name.json") ([ordered]@{
        program = $Program; arguments = $Arguments; exit_code = $code
        measured = @($lines | Where-Object { $_ -match $measured })
        refusals = @($lines | Where-Object { $_ -match 'refusals=[1-9]|unresolved=\[[0-9]|FAILED|panicked|ignored' })
    } | ConvertTo-Json -Depth 4)
    $lines | Where-Object { $_ -match $measured } | Write-Output
    if ($code -ne 0) { throw "$Name failed. Run record $output" }
}
Push-Location (Join-Path $root 'engine')
try {
    Run 'words-release' 'cargo' @('test', '--offline', '--release', '-p', 'words', '--', '--nocapture', '--test-threads=1')
    if (!$SkipGates) {
        $shell = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File')
        Run 'footprint-mutants' 'powershell.exe' ($shell + (Join-Path $root 'jobs/1.2-footprints/verify.ps1'))
        Run 'retained-replay' 'powershell.exe' ($shell + (Join-Path $root 'jobs/1.2-footprints/retained-replay.ps1'))
    }
} finally { Pop-Location }
Write-Output "Run record $output"
