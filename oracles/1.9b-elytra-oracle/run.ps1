param(
    [Parameter(Mandatory)][string]$Output,
    [string]$Server = 'D:/path-tracing-lab/server',
    [string]$JavaBin = 'D:/path-tracing-deps/data/java/java-runtime-epsilon/bin'
)
$ErrorActionPreference = 'Stop'
$serverRoot = (Resolve-Path -LiteralPath $Server).Path
$jar = Join-Path $serverRoot 'versions/26.2/server-26.2.jar'
if ((Get-FileHash -LiteralPath $jar -Algorithm SHA1).Hash -ne '86765A5899BD9C96461036A628796B4245715058') {
    throw 'Expected the staged Mojang 26.2 server jar.'
}
$java = (Resolve-Path -LiteralPath (Join-Path $JavaBin 'java.exe')).Path
$javac = (Resolve-Path -LiteralPath (Join-Path $JavaBin 'javac.exe')).Path
$folder = New-Item -ItemType Directory -Path $Output
$libraries = Get-ChildItem -LiteralPath (Join-Path $serverRoot 'libraries') -Recurse -File -Filter '*.jar'
$classPath = (@($folder.FullName, $jar) + $libraries.FullName) -join ';'
Push-Location -LiteralPath $folder.FullName
try {
    & $javac -nowarn -cp $classPath -d $folder.FullName "$PSScriptRoot/ElytraOracle.java"
    if ($LASTEXITCODE -ne 0) { throw "Oracle compile exited $LASTEXITCODE." }
    & $java '-XX:-TieredCompilation' '-Xbatch' -cp $classPath ElytraOracle jit.csv
    if ($LASTEXITCODE -ne 0) { throw "Compiled oracle exited $LASTEXITCODE." }
    & $java '-Xint' -cp $classPath ElytraOracle interpreter.csv
    if ($LASTEXITCODE -ne 0) { throw "Interpreter oracle exited $LASTEXITCODE." }
    $jit = @(Get-Content -LiteralPath 'jit.csv')
    $interpreter = @(Get-Content -LiteralPath 'interpreter.csv')
    if ($jit.Count -ne 8205 -or $interpreter.Count -ne $jit.Count) { throw 'Incomplete oracle output.' }
    $differences = for ($index = 1; $index -lt $jit.Count; $index++) {
        if ($jit[$index] -cne $interpreter[$index]) {
            [pscustomobject]@{ Case = $index; Jit = $jit[$index]; Interpreter = $interpreter[$index] }
        }
    }
    "Cases: $($jit.Count - 1). Mode differences: $(@($differences).Count)."
    $differences | Select-Object -First 1 | Format-List
    $folder.FullName
} finally {
    Pop-Location
}
