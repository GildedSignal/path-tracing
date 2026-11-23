# Runs lenses on this PC through omp with the same runner the GitHub workflow uses (lab/actions.sh): GLM by default,
# Mistral with -Model mistral/<id> (the API allowance), or -Model vibe (Mistral's Vibe CLI, on the Vibe allowance). GitHub's runner can't reach Z.ai (5 November: "Connection error" on every call), this PC can.
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-local.ps1 -Lenses foundations-primary -Minutes 180
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-local.ps1 -Lenses <id> -Model mistral/<id from set-key.ps1 -Provider mistral -Test>
# Results land in lab\results\local-<time>-*. The key comes from the encrypted store (scripts\set-key.ps1) and is never printed.
# Both are subscriptions, so no dollar cap.
param([Parameter(Mandatory)][string]$Lenses, [int]$Minutes = 60, [string]$Model = 'zai/glm-5.4')
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$omp = 'D:\path-tracing-deps\omp'
if (-not (Test-Path "$omp\omp.exe")) { throw "omp is missing in $omp." }
$bash = 'C:\Program Files\Git\bin\bash.exe'
$provider, $envName = switch -Regex ($Model) {
  '^zai/' { 'zai', 'ZAI_API_KEY' }
  '^(mistral/|vibe)' { 'mistral', 'MISTRAL_API_KEY' }
  default { throw "Unknown provider in -Model $Model (zai/..., mistral/... or vibe)." }
}
$key = Get-Key $provider
if (-not $key) { throw "No $provider key stored: run scripts\set-key.ps1 -Provider $provider first." }
$temp = 'D:\path-tracing-deps\glm-tmp'
New-Item -ItemType Directory -Force $temp | Out-Null
Set-Item "Env:$envName" $key
$env:LAB_KEY = $key   # what lab/scrub.sh deletes any output file for containing
$env:LAB_MODEL = $Model
$env:LAB_LENSES = $Lenses
$env:LAB_MINUTES = "$Minutes"
$env:GITHUB_RUN_ID = 'local-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + $PID
$env:GITHUB_RUN_ATTEMPT = '1'
$env:RUNNER_TEMP = $temp
# Julien's explicit approval to run here without isolation (S43), and the two files the GitHub runner normally provides.
$env:GLM_LOCAL = 'S43'
$summaryFile = Join-Path $temp "$($env:GITHUB_RUN_ID)-summary.md"
$outputFile = Join-Path $temp "$($env:GITHUB_RUN_ID)-output.txt"
$env:GITHUB_STEP_SUMMARY = $summaryFile
$env:GITHUB_OUTPUT = $outputFile
$env:PATH = "$omp;$env:PATH"
Set-Location $Root
Write-Host "$(Get-Date -Format HH:mm) $Model run $env:GITHUB_RUN_ID started: $Lenses ($Minutes minutes each). Close this window to stop it."
try {
  & $bash lab/actions.sh
  $batchExit = $LASTEXITCODE
  if ($batchExit -ne 0) { throw "Batch exited $batchExit." }
} finally {
  Remove-Item "Env:$envName", Env:LAB_KEY -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $summaryFile, $outputFile -ErrorAction SilentlyContinue
}
Write-Host "$(Get-Date -Format HH:mm) Finished. Results: lab\results\$env:GITHUB_RUN_ID-*"
