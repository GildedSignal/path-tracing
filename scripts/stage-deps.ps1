# Copies what builds need (the JDK, the game's classpath, its metadata, Fabric API) from the launcher's
# data folder into a staging folder with the same layout. GPT jobs run in a sandbox that cannot read
# %LOCALAPPDATA%; gpt.ps1 points them here through PATHTRACING_DEPS.
#   .\scripts\stage-deps.ps1 [-Destination D:\path-tracing-deps\data]
param([string]$Destination = 'D:\path-tracing-deps\data')
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"

function Copy-Relative([string]$Path) {
  $target = Join-Path $Destination $Path.Substring($PrismData.Length + 1)
  New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
  Copy-Item -LiteralPath $Path $target -Force
}

$jdkHome = Split-Path $Jdk
$jdkTarget = Join-Path $Destination $jdkHome.Substring($PrismData.Length + 1)
New-Item -ItemType Directory -Force $jdkTarget | Out-Null
Copy-Item "$jdkHome\*" $jdkTarget -Recurse -Force

Get-GameClasspath | ForEach-Object { Copy-Relative $_ }
$mcMeta = Get-Content "$Meta\net.minecraft\$McVersion.json" -Raw | ConvertFrom-Json
foreach ($m in "net.minecraft\$McVersion.json", "net.fabricmc.fabric-loader\$LoaderVersion.json",
               "net.fabricmc.intermediary\$McVersion.json", "org.lwjgl3\$($mcMeta.requires[0].suggests).json") {
  Copy-Relative (Join-Path $Meta $m)
}
Get-ChildItem $PackMods -Filter 'fabric-api-*.jar' | ForEach-Object { Copy-Relative $_.FullName }
"Staged into $Destination"
