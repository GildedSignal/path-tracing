# Compiles the mod straight against the 26.2 game jar (26.x ships unobfuscated, so no Gradle/Loom remapping)
# and packs build\pathtracing.jar. lib\ holds Fabric API's module jars, extracted from the pack's fabric-api jar.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\scripts\common.ps1"
Add-Type -AssemblyName System.IO.Compression.FileSystem

$lib = Join-Path $PSScriptRoot 'lib'
if (-not (Get-ChildItem "$lib\*.jar" -ErrorAction SilentlyContinue)) {
  New-Item -ItemType Directory -Force $lib | Out-Null
  $fabricApi = Get-ChildItem $PackMods -Filter 'fabric-api-*.jar' | Select-Object -First 1
  $zip = [IO.Compression.ZipFile]::OpenRead($fabricApi.FullName)
  try {
    foreach ($e in $zip.Entries | Where-Object FullName -like 'META-INF/jars/*.jar') {
      [IO.Compression.ZipFileExtensions]::ExtractToFile($e, (Join-Path $lib $e.Name), $true)
    }
  } finally { $zip.Dispose() }
}

$build = Join-Path $PSScriptRoot 'build'
Remove-Item $build -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$build\classes" | Out-Null

$classpath = @(Get-GameClasspath) + @((Get-ChildItem "$lib\*.jar").FullName)
$sources = (Get-ChildItem "$PSScriptRoot\src" -Recurse -Filter *.java).FullName
$argFile = Join-Path $build 'javac.args'
(@('--release', '25', '-proc:none', '-encoding', 'UTF-8', '-Xlint:-options', '-cp', ($classpath -join ';'), '-d', "$build\classes") + $sources |
  ForEach-Object { '"' + ($_ -replace '\\', '/') + '"' }) -join "`n" | Set-Content $argFile -Encoding ascii
& "$Jdk\javac.exe" "@$argFile"
if ($LASTEXITCODE) { throw 'javac failed' }

Copy-Item "$PSScriptRoot\fabric.mod.json" "$build\classes\"
Copy-Item "$PSScriptRoot\pathtracing.mixins.json" "$build\classes\"
& "$Jdk\jar.exe" --create --file "$build\pathtracing.jar" -C "$build\classes" .
if ($LASTEXITCODE) { throw 'jar failed' }
Write-Host "Built $build\pathtracing.jar"
