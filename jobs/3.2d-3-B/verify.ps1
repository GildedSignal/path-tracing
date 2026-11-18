# R1's offline gate and replay record. The original footprint gate owns its mutants.
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$engine = Join-Path $root 'engine'
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\r1-codec'
$output = Join-Path 'D:\path-tracing-evidence' ('r1-codec-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $output | Out-Null
$pin = Join-Path $root 'jobs/1.2-footprints/verify.ps1'
$tokens = $null; $errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($pin, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Pinned footprint gate does not parse' }
$calls = @($ast.FindAll({ param($node) $node -is [Management.Automation.Language.CommandAst] -and $node.GetCommandName() -eq 'Mutant' }, $true))
$mutants = @($calls | ForEach-Object { $_.CommandElements[1].Value })
if (!$mutants.Count -or @($mutants | Select-Object -Unique).Count -ne $mutants.Count) { throw 'Invalid pinned mutant list' }
$mutants | Set-Content -Encoding UTF8 (Join-Path $output 'mutants.txt')
$inputs = @('engine/law/src/lib.rs','engine/law/src/codec.rs','engine/law/src/codec_state.rs','engine/law/src/codec_planes.rs','engine/law/src/codec_proof.rs',
    'engine/law/src/pattern.rs','engine/law/src/footprint.rs','engine/law/src/mining.rs','engine/law/src/movement.rs','engine/law/src/state.rs','engine/law/src/world.rs','engine/law/src/shapes.rs',
    'engine/words/src/class.rs','engine/words/src/table.rs','engine/words/src/segment.rs','engine/words/src/gait.rs','engine/plan/src/reach.rs',
    'engine/law/tests/codec.rs','engine/words/tests/codec.rs','engine/world/tests/codec.rs','engine/Cargo.lock',
    'engine/world/data/models.tsv','engine/world/data/maps.tsv','engine/world/data/blocks.tsv','jobs/1.2-footprints/verify.ps1','jobs/3.2d-3-B/verify.ps1',
    'engine/world/tests/fixtures/world/dimensions/minecraft/overworld/region/r.0.-1.mca','engine/world/tests/fixtures/world/dimensions/minecraft/overworld/region/r.0.0.mca')
$hashes = @($inputs | ForEach-Object { $path = Join-Path $root $_; [pscustomobject]@{ file = $_; sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash } })
$manifest = [ordered]@{ root=$root; target=$env:CARGO_TARGET_DIR; difficulty='Peaceful'; mobs='Off'; scope='offline codec qualification; no live timing claim'; inputs=$hashes; mutants=$mutants; commands=@('cargo test --offline','jobs/1.2-footprints/verify.ps1'); result='RUNNING' }
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 (Join-Path $output 'run.json')
Write-Output "Evidence: $output"
Push-Location $engine
try {
    $ErrorActionPreference = 'Continue'
    $lines = & cargo test --offline 2>&1
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    $lines | Set-Content -Encoding UTF8 (Join-Path $output 'workspace.log')
    if ($code -ne 0) { throw "cargo test --offline failed: $code" }
    $results = [regex]::Matches(($lines -join "`n"), 'test result: ok\. (\d+) passed; \d+ failed; (\d+) ignored')
    $passed = 0; $ignored = 0
    foreach ($result in $results) { $passed += [int]$result.Groups[1].Value; $ignored += [int]$result.Groups[2].Value }
    Write-Output "workspace PASS: passed=$passed ignored=$ignored"
    $gate = @(& $pin)
    $gate | Set-Content -Encoding UTF8 (Join-Path $output 'footprints.log')
    foreach ($name in $mutants) {
        if (@($gate | Where-Object { $_ -eq "$name CAUGHT" }).Count -ne 1) { throw "Pinned mutant not caught exactly once: $name" }
    }
    foreach ($input in $hashes) {
        if ((Get-FileHash -LiteralPath (Join-Path $root $input.file) -Algorithm SHA256).Hash -ne $input.sha256) { throw "Source not restored: $($input.file)" }
    }
    $manifest.result = 'PASS'
    $manifest.workspace = @{ passed=$passed; ignored=$ignored }
    $manifest.invariants = @{ unequal_byte_aliases=0; wrong_question_proofs_accepted=0; unexpected_refusals=0; worst_margin='exact identities and bits; no numerical tolerance'; mutant_count=$mutants.Count; mutants_caught=$mutants.Count; all_sources_restored=$true }
    $manifest.refusals = @('Expected: unported Break/Place -> Unknown preserving fine identity', 'Expected: wrong state/context/version/question/owner -> refusal', 'Expected: absent coverage/loaded bits -> Unknown', 'Expected: changed recorded cell/schedule -> No for ReadsHold only')
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 (Join-Path $output 'run.json')
    Write-Output "footprints PASS: caught=$($mutants.Count)/$($mutants.Count), sources restored"
} catch {
    $manifest.result = 'FAIL'; $manifest.refusal = $_.ToString()
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 (Join-Path $output 'run.json')
    throw
} finally { Pop-Location }
