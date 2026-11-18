param([string]$Name,[Parameter(ValueFromRemainingArguments=$true)][string[]]$CargoArgs)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$env:PATH='D:\rust\cargo\bin;'+$env:PATH
$env:RUSTUP_HOME='D:\rust\rustup'
$env:CARGO_HOME='D:\rust\cargo'
$env:CARGO_TARGET_DIR='D:\path-tracing-target\u2b'
[Diagnostics.Process]::GetCurrentProcess().PriorityClass='Normal'
$directory='D:\path-tracing-evidence\u2b'
New-Item -ItemType Directory -Force $directory | Out-Null
$files=@(Get-ChildItem (Join-Path $root 'engine') -Recurse -File | Where-Object {$_.Extension -in '.rs','.toml','.lock','.tsv','.bin','.mca','.dat','.txt','.hex'})
$files+=Get-Item (Join-Path $root 'lab/tracks/hardtrack.build.txt')
$files+=Get-ChildItem -LiteralPath $PSScriptRoot -File
$archive=Join-Path $directory 'inputs'
New-Item -ItemType Directory -Force -Path $archive | Out-Null
$inputs=@($files | ForEach-Object {
    $hash=(Get-FileHash -LiteralPath $_.FullName).Hash
    $blob=Join-Path $archive $hash
    if(!(Test-Path -LiteralPath $blob)){Copy-Item -LiteralPath $_.FullName -Destination $blob}
    @{path=$_.FullName.Substring($root.Length+1);sha256=$hash;blob=$blob}
})
Push-Location (Join-Path $root 'engine')
$clock=[Diagnostics.Stopwatch]::StartNew()
$writer=[IO.StreamWriter]::new((Join-Path $directory ($Name+'.log')),$false,[Text.UTF8Encoding]::new($false))
$writer.AutoFlush=$true
try {
 $ErrorActionPreference='Continue'
 $lines=@(& cargo @CargoArgs 2>&1 | ForEach-Object {$writer.WriteLine("$_");"$_"})
 $code=$LASTEXITCODE
} finally {$writer.Dispose();Pop-Location}
$clock.Stop()
@{command='cargo '+($CargoArgs -join ' ');inputs=$inputs;exit=$code;elapsed_ms=$clock.Elapsed.TotalMilliseconds;invariants=@($lines | Where-Object {$_ -match 'U[12]B?_|S203_|RESULT|MARGIN|RED|PASS'});refusals=@($lines | Where-Object {$_ -match 'REFUS|panicked|FAILED|error\['})} | ConvertTo-Json -Depth 7 | Set-Content -Encoding UTF8 (Join-Path $directory ($Name+'.json'))
$lines | Select-Object -Last 65
exit $code

