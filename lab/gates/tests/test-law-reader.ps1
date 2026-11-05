# The reader lives on wt/law. Test its integration patch in an isolated copy, never that worktree.
$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\..'))
$scratch = Join-Path $repoRoot ('lab\gates\results\law-reader-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Force $scratch
& git -C $repoRoot archive --format=tar --output="$scratch\law.tar" wt/law engine lab/fixtures/README.md
if ($LASTEXITCODE) { throw 'Cannot copy wt/law.' }
& tar -xf "$scratch\law.tar" -C $scratch
if ($LASTEXITCODE) { throw 'Cannot extract wt/law.' }
$patch = Join-Path $repoRoot 'jobs\1.6-l0-fixtures\law-reader.patch'
& git -C $repoRoot apply --unsafe-paths --directory="$scratch" $patch
if ($LASTEXITCODE) { throw 'Cannot apply reader integration patch.' }
& D:\rust\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\cargo.exe test --offline --manifest-path "$scratch\engine\Cargo.toml" --target-dir D:\path-tracing-target\fixture-reader --test replay
if ($LASTEXITCODE) { throw 'Reader integration tests failed.' }
