param([string]$Output = 'D:\path-tracing-target\server-mod-build')
$ErrorActionPreference = 'Stop'
$env:PATHTRACING_ROOT = 'D:\path-tracing-lab'
$env:PATHTRACING_DEPS = ''
. "$PSScriptRoot\..\scripts\common.ps1"
Add-Type -AssemblyName System.IO.Compression.FileSystem

$clientRoot = Join-Path $PSScriptRoot 'client'
$classes = Join-Path $Output 'client-classes'
$lib = Join-Path $Output 'lib'
Remove-Item $classes -Recurse -Force -ErrorAction SilentlyContinue
$null = New-Item -ItemType Directory -Force $classes, $lib

if (-not (Get-ChildItem "$lib\*.jar" -ErrorAction SilentlyContinue)) {
  $api = Get-ChildItem "$PackMods" -Filter 'fabric-api-*.jar' | Select-Object -First 1
  if (-not $api) { throw "fabric-api jar not found in $PackMods" }
  $zip = [IO.Compression.ZipFile]::OpenRead($api.FullName)
  try {
    foreach ($e in $zip.Entries | Where-Object FullName -like 'META-INF/jars/*.jar') {
      [IO.Compression.ZipFileExtensions]::ExtractToFile($e, (Join-Path $lib $e.Name), $true)
    }
  } finally {
    $zip.Dispose()
  }
}

$deps = @(Get-GameClasspath) + @((Get-ChildItem "$lib\*.jar").FullName)
$sources = @(
  Get-ChildItem "$clientRoot\src" -Recurse -Filter '*.java' |
    ForEach-Object { $_.FullName }
)

$argsFile = Join-Path $Output 'javac-client.args'
(@('--release', '25', '-proc:none', '-encoding', 'UTF-8', '-Xlint:-options', '-cp', ($deps -join ';'), '-d', $classes) + $sources |
  ForEach-Object { '"' + ($_ -replace '\\', '/') + '"' }) -join "`n" | Set-Content $argsFile -Encoding ASCII

& "$Jdk\javac.exe" "@$argsFile"
if ($LASTEXITCODE) { throw 'client-mod compilation failed' }

Copy-Item "$clientRoot\fabric.mod.json", "$clientRoot\pathtracing-swarm-client.mixins.json" $classes
$jar = Join-Path $Output 'pathtracing-swarm-client.jar'
Remove-Item $jar -Force -ErrorAction SilentlyContinue
& "$Jdk\jar.exe" --create --file $jar -C $classes .
if ($LASTEXITCODE) { throw 'client-mod jar packaging failed' }
Write-Output $jar
