# Movement bits unchanged: the retained recorder pairs replay identically on main and on this branch.
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$output = 'D:\path-tracing-deps\footprints-verification'
$baseline = 'D:\path-tracing-deps\footprints-main'
if (Test-Path -LiteralPath $baseline) { Remove-Item -LiteralPath $baseline -Recurse -Force }
New-Item -ItemType Directory -Force -Path $baseline, $output | Out-Null
Push-Location $root
try {
    & git archive --format=tar "--output=$output\main.tar" main engine lab/fixtures oracles
    if ($LASTEXITCODE -ne 0) { throw 'Cannot archive main' }
    & tar -xf "$output\main.tar" -C $baseline
    if ($LASTEXITCODE -ne 0) { throw 'Cannot unpack main' }
    $results = @()
    foreach ($source in @($baseline, $root)) {
        $staged = [IO.Path]::GetFullPath((Join-Path $source 'lab/fixtures/footprints-verification'))
        if (Test-Path -LiteralPath $staged) { throw "Staging already exists at $staged" }
        New-Item -ItemType Directory -Path $staged | Out-Null
        try {
            foreach ($case in @('corner', 'full-blocks', 'slime')) {
                Copy-Item -LiteralPath "D:\path-tracing-evidence\l0-original-pairs\fixtures\$case" -Destination $staged -Recurse
            }
            Push-Location (Join-Path $source 'engine')
            try {
                $ErrorActionPreference = 'Continue'
                $lines = & cargo test --offline -p law --test replay replay -- --exact --nocapture 2>&1
            } finally { $ErrorActionPreference = 'Stop'; Pop-Location }
            $name = if ($source -eq $baseline) { 'main' } else { 'branch' }
            $lines | Set-Content -Encoding UTF8 -LiteralPath "$output\replay-$name.log"
            # Keep only the replay's own verdict lines, not build paths or timings.
            $results += (($lines | Where-Object { $_ -match 'left:|right:|failed:|server resources:|test replay' }) -join "`n")
        } finally { Remove-Item -LiteralPath $staged -Recurse -Force }
    }
    if ($results[0] -ne $results[1]) { throw "Replay differs between main and branch. See $output" }
    Write-Output "IDENTICAL on main and branch:`n$($results[1])"
} finally { Pop-Location }
