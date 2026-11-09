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
    & $javac -cp $classPath -d $folder.FullName "$PSScriptRoot/FluidFlowOracle.java"
    if ($LASTEXITCODE -ne 0) { throw "Oracle compile exited $LASTEXITCODE." }
    & $java -cp $classPath FluidFlowOracle flows.csv
    if ($LASTEXITCODE -ne 0) { throw "Oracle exited $LASTEXITCODE." }
    $rows = @(Get-Content -LiteralPath 'flows.csv')
    if ($rows.Count -ne 1035) { throw 'Incomplete oracle output.' }
    "Generated $($rows.Count - 1) raw flow vectors in $($folder.FullName)/flows.csv"
} finally {
    Pop-Location
}
