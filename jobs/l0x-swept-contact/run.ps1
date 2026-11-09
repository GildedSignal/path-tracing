param([string]$ReferenceServer = 'D:/path-tracing-lab/server')
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
. "$root/scripts/common.ps1"
$env:CARGO_HOME = 'D:/rust/cargo'
$env:RUSTUP_HOME = 'D:/rust/rustup'
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$result = Join-Path $root "lab/gates/results/$stamp-swept-contact"
$null = New-Item -ItemType Directory -Path $result
$serverJar = Join-Path $ReferenceServer "versions/$McVersion/server-$McVersion.jar"
$cp = (@($serverJar) + @((Get-ChildItem "$ReferenceServer/libraries" -Recurse -Filter '*.jar').FullName)) -join ';'
$cases = Join-Path $result 'oracle.tsv'
& "$Jdk/java.exe" --class-path $cp "$PSScriptRoot/SweptContactOracle.java" | Set-Content -LiteralPath $cases -Encoding ASCII
if ($LASTEXITCODE -ne 0) { throw 'Vanilla oracle failed' }

function Invoke-ContactComparison([string]$Manifest, [string]$Name) {
  $target = Join-Path $result $Name
  & "$env:CARGO_HOME/bin/cargo.exe" build --offline --release --lib --manifest-path $Manifest --target-dir $target
  if ($LASTEXITCODE -ne 0) { throw "$Name build failed" }
  $binary = Join-Path $target 'compare.exe'
  & "$env:CARGO_HOME/bin/rustc.exe" --edition 2024 --extern "law=$target/release/liblaw.rlib" "$PSScriptRoot/compare.rs" -o $binary
  if ($LASTEXITCODE -ne 0) { throw "$Name verifier build failed" }
  & $binary $cases | Set-Content -LiteralPath "$result/$Name.log" -Encoding ASCII
  $code = $LASTEXITCODE
  Get-Content -LiteralPath "$result/$Name.log" | Write-Host
  $code
}

$normal = Invoke-ContactComparison "$root/engine/Cargo.toml" 'original'
$mutant = Join-Path $result 'mutant'
$null = New-Item -ItemType Directory -Path $mutant
Copy-Item -LiteralPath "$root/engine/Cargo.toml", "$root/engine/Cargo.lock", "$root/engine/law" -Destination $mutant -Recurse
$law = Join-Path $mutant 'law/src/shapes.rs'
$source = [IO.File]::ReadAllText($law)
$inset = '.minus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7))'
if ([regex]::Matches($source, [regex]::Escape($inset)).Count -ne 1) { throw 'Expected exactly one swept inset' }
[IO.File]::WriteAllText($law, $source.Replace($inset, ''), [Text.UTF8Encoding]::new($false))
$control = Invoke-ContactComparison "$mutant/Cargo.toml" 'no-inset'
"Vanilla jar SHA-256 $((Get-FileHash -LiteralPath $serverJar).Hash)"
"Original exit $normal, planted inset omission exit $control"
"Evidence $result"
if ($normal -ne 0 -or $control -ne 1) { exit 1 }
