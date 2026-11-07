param(
  [Parameter(Mandatory)][string]$Capture20,
  [Parameter(Mandatory)][string]$Capture200,
  [Parameter(Mandatory)][string]$Edges20,
  [Parameter(Mandatory)][string]$Edges200,
  [Parameter(Mandatory)][string]$Teleport,
  [Parameter(Mandatory)][string]$ReferenceServer,
  [string]$Target = 'D:/path-tracing-target/1.6f'
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
. "$root/scripts/common.ps1"
$env:CARGO_HOME = 'D:/rust/cargo'
$env:RUSTUP_HOME = 'D:/rust/rustup'
$env:CARGO_MANIFEST_DIR = (Resolve-Path "$root/engine/law").Path
$env:L0_EDGES_20 = (Resolve-Path -LiteralPath $Edges20).Path
$env:L0_LANDING_20 = (Resolve-Path -LiteralPath $Capture20).Path
$env:L0_LANDING_200 = (Resolve-Path -LiteralPath $Capture200).Path
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$log = Join-Path $env:L0_LANDING_20 "1.6f-$stamp-verification.log"
$javaLog = Join-Path $env:L0_LANDING_20 "1.6f-$stamp-codec.log"
& "$PSScriptRoot/../1.6e-landing-motion/check-refusals.ps1" -Edges20 $Edges20 -Edges200 $Edges200 -Teleport $Teleport |
  Tee-Object -FilePath $log
& "$env:CARGO_HOME/bin/cargo.exe" build --offline --release --lib --manifest-path "$root/engine/Cargo.toml" --target-dir $Target
if ($LASTEXITCODE) { throw 'Cannot build the Rust law.' }
& "$env:CARGO_HOME/bin/rustc.exe" --edition 2024 --test --extern "law=$Target/release/liblaw.rlib" "$PSScriptRoot/verify.rs" -o "$Target/landing-verify.exe"
if ($LASTEXITCODE) { throw 'Cannot build the landing verifier.' }
# Retain the first native failure in the case report directory.
$ErrorActionPreference = 'Continue'
& "$Target/landing-verify.exe" --exact live_landing_motion --nocapture 2>&1 | Tee-Object -FilePath $log -Append
if ($LASTEXITCODE) { throw "1.6f verification failed. See $log." }
$cp = @(
  "$ReferenceServer/versions/$McVersion/server-$McVersion.jar"
  (Get-ChildItem "$ReferenceServer/libraries" -Recurse -Filter '*.jar').FullName
) -join ';'
$wire = (Get-ChildItem -LiteralPath $env:L0_LANDING_20, $env:L0_LANDING_200 -Filter '*.server-motion.csv').FullName
& "$Jdk/java.exe" --class-path $cp "$PSScriptRoot/WireCodec.java" @wire 2>&1 | Tee-Object -FilePath $javaLog
if ($LASTEXITCODE) { throw "1.6f codec verification failed. See $javaLog." }
"1.6f PASS. Evidence: $log, $javaLog"
