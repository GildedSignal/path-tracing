# Archive exact final replay inputs after all mutation sources are restored.
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence='D:\path-tracing-evidence\u2b'
$manifest=Get-Content -Raw (Join-Path $evidence 'gates-s203.json') | ConvertFrom-Json
if(@($manifest.gates | Where-Object {!$_.recorded}).Count){throw 'A gate is not recorded correctly'}
$files=@(Get-ChildItem (Join-Path $root 'engine') -Recurse -File | Where-Object {$_.Extension -in '.rs','.toml','.lock','.tsv','.bin','.mca','.dat','.txt','.hex'})
$files+=Get-ChildItem -LiteralPath $PSScriptRoot -File
$files+=Get-Item (Join-Path $root 'lab/tracks/hardtrack.build.txt')
$files+=Get-Item 'D:\path-tracing-evidence\u2\kill10-bar.log'
$files+=Get-Item 'C:\Users\PASCAL\Desktop\path tracing\jobs\set-rederive\proposal.md'
$files+=Get-Item 'C:\Users\PASCAL\Desktop\path tracing\jobs\pin-rederive\proposal.md'
$archive=Join-Path $evidence 'inputs'
New-Item -ItemType Directory -Force -Path $archive | Out-Null
$inputs=@($files | ForEach-Object {
    $hash=(Get-FileHash -LiteralPath $_.FullName).Hash
    $blob=Join-Path $archive $hash
    if(!(Test-Path -LiteralPath $blob)){Copy-Item -LiteralPath $_.FullName -Destination $blob}
    @{path=$_.FullName;sha256=$hash;blob=$blob}
})
@{inputs=$inputs;priority='Normal';offline=$true;source_restored=$true;gate_record='gates-s203.json'} | ConvertTo-Json -Depth 7 | Set-Content -Encoding utf8 (Join-Path $evidence 'final-inputs-s203.json')
Write-Output 'U2B_FINAL archived=true source_restored=true'
