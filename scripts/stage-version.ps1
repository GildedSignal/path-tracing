# Stages one game version's client runtime from Mojang's metadata, every file checked by its SHA-1:
# the libraries the launcher lacks go to D:\path-tracing-deps\versions\<v>\libraries (read by
# Get-GameClasspath), the asset index and objects to the launcher's shared, content-addressed store.
#   .\stage-version.ps1 -McVersion 26.3
param([Parameter(Mandatory)][string]$McVersion)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1" -McVersion $McVersion
if (-not (Test-Path -LiteralPath $MojangMetadata)) { throw "Stage Mojang's $McVersion.json at $MojangMetadata first." }
$meta = Get-Content -LiteralPath $MojangMetadata -Raw | ConvertFrom-Json

function Save-Checked([string]$Url, [string]$Path, [string]$Sha1) {
  if ((Test-Path -LiteralPath $Path) -and (Get-FileHash -LiteralPath $Path -Algorithm SHA1).Hash -eq $Sha1) { return $false }
  New-Item -ItemType Directory -Force (Split-Path -Parent $Path) | Out-Null
  $tmp = "$Path.part"
  Invoke-WebRequest -UseBasicParsing $Url -OutFile $tmp
  if ((Get-FileHash -LiteralPath $tmp -Algorithm SHA1).Hash -ne $Sha1) { Remove-Item $tmp; throw "SHA-1 mismatch: $Url" }
  Move-Item -Force $tmp $Path
  $true
}

$libs = 0
foreach ($lib in $meta.libraries) {
  $a = $lib.downloads.artifact
  if (-not $a -or -not (Test-Rules $lib) -or $lib.name -match 'natives-windows-(x86|arm64)') { continue }
  if (Test-Path -LiteralPath (Join-Path $Libraries $a.path)) { continue }
  if (Save-Checked $a.url (Join-Path "$McStage/libraries" $a.path) $a.sha1) { $libs++ }
}

$index = Join-Path $Assets "indexes\$($meta.assetIndex.id).json"
$null = Save-Checked $meta.assetIndex.url $index $meta.assetIndex.sha1
$objects = 0
foreach ($o in (Get-Content -LiteralPath $index -Raw | ConvertFrom-Json).objects.PSObject.Properties.Value) {
  $h = $o.hash; $p = Join-Path $Assets "objects\$($h.Substring(0,2))\$h"
  if (Test-Path -LiteralPath $p) { continue }
  if (Save-Checked "https://resources.download.minecraft.net/$($h.Substring(0,2))/$h" $p $h) { $objects++ }
}
Write-Host "Minecraft $McVersion staged: $libs libraries and $objects asset objects downloaded, all SHA-1 checked."
