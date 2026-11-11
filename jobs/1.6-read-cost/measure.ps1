# Job 1.6's numbers, all offline: the read-cost harness (engine/words/tests/read_cost.rs).
# Counts are exact; times are p50/p90 over interleaved samples, so read them as ratios.
# Writes a run record: inputs, the machine's other load, every printed line, refusals.
param([string]$OutputRoot = 'D:\path-tracing-evidence\read-cost')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$output = Join-Path $OutputRoot ([DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ'))
New-Item -ItemType Directory -Force -Path $output | Out-Null
$utf8 = New-Object Text.UTF8Encoding($false)
$load = Get-Process | Sort-Object CPU -Descending | Select-Object -First 8 Name, CPU | ForEach-Object { "$($_.Name)=$([int]$_.CPU)s" }
[IO.File]::WriteAllText((Join-Path $output 'inputs.json'), ([ordered]@{
    head = (& git -C $root rev-parse HEAD); dirty = @(& git -C $root status --short)
    toolchain = ((& rustc -Vv) -join "`n"); harness = 'engine/words/tests/read_cost.rs'; load = $load
} | ConvertTo-Json -Depth 4), $utf8)
Push-Location (Join-Path $root 'engine')
try {
    foreach ($run in @(
        @{ Name = 'sites'; Arguments = @('test', '--offline', '-p', 'words', '--test', 'read_cost', 'what_the_law_asks', '--', '--nocapture') },
        @{ Name = 'release'; Arguments = @('test', '--release', '--offline', '-p', 'words', '--test', 'read_cost', '--', '--nocapture', '--test-threads', '1') })) {
        $previous = $ErrorActionPreference
        try {
            $ErrorActionPreference = 'Continue'
            $lines = @(& cargo @($run.Arguments) 2>&1 | ForEach-Object { "$_" })
            $code = $LASTEXITCODE
        } finally { $ErrorActionPreference = $previous }
        [IO.File]::WriteAllLines((Join-Path $output "$($run.Name).log"), [string[]]$lines, $utf8)
        if ($code -ne 0) { throw "$($run.Name) failed; see $output" }
        $lines | Where-Object { $_ -match '^\[|refused' } | Write-Output
    }
} finally { Pop-Location }
Write-Output "record: $output"
