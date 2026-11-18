param([string]$Output = 'D:\path-tracing-evidence\3.2d-3-A-baseline')
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/../../scripts/common.ps1"
$manifestPath = Join-Path $Output 'inputs.json'
if (Test-Path -LiteralPath $manifestPath) { throw 'Inputs already frozen; preserve original bytes.' }
$inputs = @(Get-ChildItem (Join-Path $Rig 'lab/tracks') -File -Filter '*.json')
$inputs += @(Get-ChildItem "$Root/engine/world/data" -File)
$inputs += @(Get-Item "$Root/engine/world/tests/fixtures/tracks.txt")
$entries = @()
New-Item -ItemType Directory -Force (Join-Path $Output 'inputs') | Out-Null
foreach ($file in $inputs) {
    $category = if ($file.DirectoryName -eq (Join-Path $Rig 'lab/tracks')) { 'track' } else { 'model' }
    $target = Join-Path $Output ('inputs/' + $category + '-' + $file.Name)
    Copy-Item -LiteralPath $file.FullName -Destination $target
    $entries += @{source=$file.FullName;file=$target;sha256=(Get-FileHash -LiteralPath $target).Hash}
}
@{ files=$entries; note='Frozen rig definitions/kits and model inputs; no live control/capture or world-file snapshot.' } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
Write-Output "Frozen original inputs: $manifestPath"
