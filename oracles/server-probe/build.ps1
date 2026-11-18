param([string]$Output = 'D:\path-tracing-target\server-probe-build')
$ErrorActionPreference = 'Stop'
$env:PATHTRACING_ROOT = 'D:\path-tracing-lab'
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
. "$PSScriptRoot\..\..\scripts\common.ps1"
Add-Type -AssemblyName System.IO.Compression.FileSystem

$classes = Join-Path $Output 'classes'
$lib = Join-Path $Output 'lib'
Remove-Item $classes -Recurse -Force -ErrorAction SilentlyContinue
$null = New-Item -ItemType Directory -Force $classes, $lib

if (-not (Get-ChildItem "$lib\*.jar" -ErrorAction SilentlyContinue)) {
  $api = Get-ChildItem "$Rig\clients\bot\mods" -Filter 'fabric-api-*.jar' | Select-Object -First 1
  if (-not $api) { throw "fabric-api jar not found in $Rig\clients\bot\mods" }
  $zip = [IO.Compression.ZipFile]::OpenRead($api.FullName)
  try {
    foreach ($e in $zip.Entries | Where-Object FullName -like 'META-INF/jars/*.jar') {
      [IO.Compression.ZipFileExtensions]::ExtractToFile($e, (Join-Path $lib $e.Name), $true)
    }
  } finally {
    $zip.Dispose()
  }
}

$deps = @(
  "$Rig\server\versions\$McVersion\server-$McVersion.jar"
) + @((Get-ChildItem "$Rig\server\libraries" -Recurse -Filter '*.jar').FullName) +
    @((Get-ChildItem "$Libraries\net\fabricmc" -Recurse -Filter '*.jar').FullName) +
    @((Get-ChildItem $lib -Filter '*.jar').FullName)

$sources = @(
  Get-ChildItem $PSScriptRoot -Recurse -Filter '*.java' |
    Where-Object { $_.Name -ne 'Profile.java' } |
    ForEach-Object { $_.FullName }
)

$argsFile = Join-Path $Output 'javac.args'
(@('--release', '25', '-proc:none', '-encoding', 'UTF-8', '-Xlint:-options', '-cp', ($deps -join ';'), '-d', $classes) + $sources |
  ForEach-Object { '"' + ($_ -replace '\\', '/') + '"' }) -join "`n" | Set-Content $argsFile -Encoding ASCII

& "$Jdk\javac.exe" "@$argsFile"
if ($LASTEXITCODE) { throw 'server-probe compilation failed' }

Copy-Item "$PSScriptRoot\fabric.mod.json", "$PSScriptRoot\server-probe.mixins.json" $classes
$jar = Join-Path $Output 'server-probe.jar'
& "$Jdk\jar.exe" --create --file $jar -C $classes .
if ($LASTEXITCODE) { throw 'server-probe packaging failed' }
Write-Output $jar
