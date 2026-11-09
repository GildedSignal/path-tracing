param([string]$Dependencies = 'D:/path-tracing-deps/data')
$ErrorActionPreference = 'Stop'
$engine = Split-Path $PSScriptRoot
$bytecode = Join-Path $engine '.bytecode'
New-Item -ItemType Directory -Force $bytecode | Out-Null
$java = Join-Path $Dependencies 'java/java-runtime-epsilon/bin'
$classpath = ((Get-ChildItem (Join-Path $Dependencies 'libraries') -Filter '*.jar' -File -Recurse).FullName -join ';')
$compileArgs = Join-Path $bytecode 'oracle-compile.args'
$runArgs = Join-Path $bytecode 'oracle-run.args'
$compiler = @('--class-path', ('"'+$classpath.Replace('\','/')+'"'), '-d', ('"'+$bytecode.Replace('\','/')+'"'), ('"'+(Join-Path $PSScriptRoot 'JarOracle.java').Replace('\','/')+'"'))
[IO.File]::WriteAllLines($compileArgs,$compiler)
& (Join-Path $java 'javac.exe') ('@'+$compileArgs)
if ($LASTEXITCODE -ne 0) { throw 'JarOracle compilation failed' }
$runtime = @('--class-path', ('"'+($classpath+';'+$bytecode).Replace('\','/')+'"'), 'JarOracle', ('"'+(Join-Path $engine 'law/src/sin.bin').Replace('\','/')+'"'), ('"'+(Join-Path $engine 'law/tests/data/shapes.tsv').Replace('\','/')+'"'), ('"'+(Join-Path $engine 'law/tests/data/intersections.tsv').Replace('\','/')+'"'))
[IO.File]::WriteAllLines($runArgs,$runtime)
Push-Location $bytecode
try {
    & (Join-Path $java 'java.exe') ('@'+$runArgs)
} finally {
    Pop-Location
}
if ($LASTEXITCODE -ne 0) { throw 'JarOracle execution failed' }