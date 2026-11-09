# Runs GLM lenses on this PC with the same runner the GitHub workflow uses (lab/actions.sh).
# GitHub's runner can't reach Z.ai (5 November: "Connection error" on every call), this PC can.
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-local.ps1 -Lenses foundations-primary -Minutes 180
# Results land in lab\results\local-<time>-*. The key comes from the encrypted store (Set GLM key.cmd) and is never printed.
param([Parameter(Mandatory)][string]$Lenses, [int]$Minutes = 60)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$omp = 'D:\path-tracing-deps\omp'
if (-not (Test-Path "$omp\omp.exe")) { throw "omp is missing in $omp." }
$bash = 'C:\Program Files\Git\bin\bash.exe'
$key = Get-GlmKey
if (-not $key) { throw 'No GLM key stored: double-click Set GLM key.cmd first.' }
$temp = 'D:\path-tracing-deps\glm-tmp'
New-Item -ItemType Directory -Force $temp | Out-Null
$env:ZAI_API_KEY = $key
$env:LAB_LENSES = $Lenses
$env:LAB_MINUTES = "$Minutes"
$env:GITHUB_RUN_ID = 'local-' + (Get-Date -Format 'yyyyMMdd-HHmm')
$env:GITHUB_RUN_ATTEMPT = '1'
$env:RUNNER_TEMP = $temp
# Julien's explicit approval to run here without isolation (S43), and the two files the GitHub runner normally provides.
$env:GLM_LOCAL = 'S43'
$env:GITHUB_STEP_SUMMARY = Join-Path $temp 'summary.md'
$env:GITHUB_OUTPUT = Join-Path $temp 'output.txt'
$env:PATH = "$omp;$env:PATH"
Set-Location $Root
Write-Host "$(Get-Date -Format HH:mm) GLM run $env:GITHUB_RUN_ID started: $Lenses ($Minutes minutes each). Close this window to stop it."
try {
  & $bash lab/actions.sh
  $batchExit = $LASTEXITCODE
  & "$PSScriptRoot\lab-wiki.ps1" -Results (Join-Path $Root 'lab\results')
  if ($batchExit -ne 0) { throw "GLM batch exited $batchExit; available results were compiled." }
} finally { Remove-Item Env:ZAI_API_KEY -ErrorAction SilentlyContinue }
Write-Host "$(Get-Date -Format HH:mm) Finished. Results: lab\results\$env:GITHUB_RUN_ID-*"
