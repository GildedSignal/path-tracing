# One way to start any companion on a job, detached, so it outlives the session that started it (S100).
#   .\scripts\launch.ps1 <job-id> terra  [-Worktree D:\path-tracing-wt\<name>] [-Base <commit>]
#   .\scripts\launch.ps1 <job-id> astra  -Worktree ...
#   .\scripts\launch.ps1 <job-id> sonnet|opus|argon|hydrogen  [-ReadDir D:\path-tracing-wt\<name>]
# Terra and Astra go through gpt.ps1 (Codex); the others through gemini.ps1 (Antigravity: the Claude slugs
# there draw on Antigravity's meter and spare the native Anthropic plan, clan.md section 4).
# A GPT job's worktree is created from -Base (default main) on branch wt/<job-id> if it does not exist.
# Every job leaves jobs/<id>/last.md; `.\scripts\status.ps1` shows what runs and what waits to be read.
param(
  [Parameter(Mandatory, Position = 0)][string]$Job,
  [Parameter(Mandatory, Position = 1)][ValidateSet('terra','astra','sonnet','opus','argon','hydrogen')][string]$Who,
  [string]$Worktree,
  [string]$Base = 'main',
  [string[]]$ReadDir = @()
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
if (-not (Test-Path (Join-Path $Root "jobs\$Job\prompt.md"))) { throw "No spec: jobs\$Job\prompt.md" }

if ($Who -in 'terra','astra') {
  if (-not $Worktree) { throw 'A GPT job needs -Worktree' }
  # AGENTS: one GPT job beside Astra, four Terra at most, fresh first prompts only.
  $running = @(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' })
  $astra = @($running | Where-Object { $_.CommandLine -match 'astra' }).Count
  if ($running.Count -ge 4 -or ($astra -and $running.Count -ge 2) -or ($Who -eq 'astra' -and $running.Count -ge 2)) {
    throw "GPT lane full: $($running.Count) running ($astra Astra). Wait, or queue it in jobs\queue.txt."
  }
  if (-not (Test-Path $Worktree)) {
    # git reports progress on stderr, which PS 5.1 turns into a terminating error under 'Stop'.
    $ErrorActionPreference = 'Continue'; git -C $Root worktree add -b "wt/$Job" $Worktree $Base 2>&1 | Out-Null; $ErrorActionPreference = 'Stop'
    if (-not (Test-Path $Worktree)) { throw "worktree failed: $Worktree from $Base" }
  }
  $model = @{ terra = 'gpt-6.1-terra'; astra = 'gpt-6-astra' }[$Who]
  $a = @('-NoProfile','-ExecutionPolicy','Bypass','-File',"$PSScriptRoot\gpt.ps1",'-Job',$Job,'-Worktree',$Worktree,'-Model',$model)
} else {
  # S105: Antigravity's two meters each get a slot budget, counted in units: Gemini meter, one Argon (2)
  # or two Hydrogen (1 each); Claude meter, one Opus (2) or two Sonnet (1 each). Adjust only on a
  # measured drain (a 429 naming the reset), and write the change here and in clan.md.
  $units = @{ argon = 2; hydrogen = 1; opus = 2; sonnet = 1 }
  $meter = if ($Who -in 'opus','sonnet') { 'claude' } else { 'gemini' }
  $used = 0
  Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'gemini\.ps1' } | ForEach-Object {
    $m = [regex]::Match($_.CommandLine, '-Model (\S+)').Groups[1].Value
    $w = if ($m -match 'opus') { 'opus' } elseif ($m -match 'sonnet') { 'sonnet' } elseif ($m -match 'argon') { 'argon' } else { 'hydrogen' }
    $wm = if ($w -in 'opus','sonnet') { 'claude' } else { 'gemini' }
    if ($wm -eq $meter) { $used += $units[$w] }
  }
  if ($used + $units[$Who] -gt 2) { throw "Antigravity $meter meter full ($used of 2 units, S105). Wait for the running job." }
  $model = @{ sonnet = 'claude-sonnet-5-5-high'; opus = 'claude-opus-5-5-high'
              argon = 'gemini-4-argon-high'; hydrogen = 'gemini-4-hydrogen-high' }[$Who]
  $a = @('-NoProfile','-ExecutionPolicy','Bypass','-File',"$PSScriptRoot\gemini.ps1",'-Job',$Job,'-Model',$model)
  if ($ReadDir) { $a += '-ReadDir'; $a += ($ReadDir -join ',') }
}
# Start-Process does not quote its arguments and this repo's path has a space (clan.md, trap 1).
$q = $a | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
$p = Start-Process powershell -ArgumentList $q -WorkingDirectory $Root -WindowStyle Hidden -PassThru
Add-Content -Encoding utf8 (Join-Path $Root 'jobs\launch.log') "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $Job $Who $model pid $($p.Id) $Worktree"
"launched $Job on $Who ($model), pid $($p.Id)"
