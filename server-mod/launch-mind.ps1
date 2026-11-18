param([string]$Shm = 'D:\path-tracing-run\server-mind.shm')
$ErrorActionPreference = 'Stop'
$exe = 'D:\path-tracing-target\server-mind\release\mind.exe'
if (-not (Test-Path $exe)) {
  throw "Missing $exe. Build with: cargo build --release --offline -p mind --manifest-path engine\Cargo.toml"
}
& $exe --shm $Shm
