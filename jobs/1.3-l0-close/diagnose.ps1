param([string]$Target = 'D:/path-tracing-target/l0-close-20261109-v5')
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$env:CARGO_HOME = 'D:/rust/cargo'
$env:RUSTUP_HOME = 'D:/rust/rustup'
$env:CARGO_MANIFEST_DIR = "$root/engine/law"
$binary = "$Target/sprint-diagnostic.exe"
& "$env:CARGO_HOME/bin/rustc.exe" --edition 2024 --test -C opt-level=3 -C lto=thin -C codegen-units=1 `
  --extern "law=$Target/release/liblaw.rlib" "$PSScriptRoot/diagnostic.rs" -o $binary
if ($LASTEXITCODE) { throw 'Cannot build the sprint diagnostic.' }
try {
  $env:L0_DIAGNOSTIC_DERIVE_MODIFIER = '1'
  $ErrorActionPreference = 'Continue'
  $old = @(& $binary --exact recorded_sprint_diagnostic --nocapture 2>&1 | ForEach-Object { "$_" })
  $code = $LASTEXITCODE
  $ErrorActionPreference = 'Stop'
  $old | Set-Content -LiteralPath "$Target/sprint-old.log" -Encoding UTF8
  if ($code -ne 101 -or ($old -join "`n") -notmatch 'tick 204: logged speedBits') {
    throw 'The old derivation did not fail on the original first divergent field.'
  }
} finally { Remove-Item Env:L0_DIAGNOSTIC_DERIVE_MODIFIER -ErrorAction SilentlyContinue }
& $binary --exact recorded_sprint_diagnostic --nocapture | Tee-Object -FilePath "$Target/sprint-fixed.log"
if ($LASTEXITCODE) { throw 'Corrected law failed the diagnostic recording.' }
& "$PSScriptRoot/sprint-witness.ps1" -Output "$Target/sprint-bytecode.txt" | Out-Null
'Original tick-204 speed refusal reproduced; corrected law matches all 420 diagnostic rows.'
