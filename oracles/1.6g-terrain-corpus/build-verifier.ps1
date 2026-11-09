param([Parameter(Mandatory)][string]$Target)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$env:CARGO_HOME = 'D:/rust/cargo'
$env:RUSTUP_HOME = 'D:/rust/rustup'
$env:CARGO_MANIFEST_DIR = "$root/engine/law"
& "$env:CARGO_HOME/bin/cargo.exe" build --offline --release --lib --manifest-path "$root/engine/Cargo.toml" --target-dir $Target
if ($LASTEXITCODE) { throw 'Cannot build the Rust law.' }
& "$env:CARGO_HOME/bin/rustc.exe" --edition 2024 --test --extern "law=$Target/release/liblaw.rlib" `
  "$PSScriptRoot/../1.6f-landing-motion/verify.rs" -o "$Target/terrain-verify.exe"
if ($LASTEXITCODE) { throw 'Cannot build the terrain verifier.' }
