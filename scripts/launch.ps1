# One way to start any companion on a job, detached, so it outlives the session that started it (S100).
#   .\scripts\launch.ps1 <job-id> terra  [-Worktree D:\path-tracing-wt\<name>] [-Base <commit>]
#   .\scripts\launch.ps1 <job-id> astra  -Worktree ...
# Terra, Astra and Sol go through gpt.ps1 (Codex). Opus and Sonnet run natively through the integrator's Agent tool.
# A GPT job's worktree is created from -Base (default main) on branch wt/<job-id> if it does not exist.
# Every job leaves jobs/<id>/last.md; `.\scripts\status.ps1` shows what runs and what waits to be read.
param(
  [Parameter(Mandatory, Position = 0)][string]$Job,
  [Parameter(Mandatory, Position = 1)][ValidateSet('terra','astra','sol')][string]$Who,
  [string]$Worktree,
  [string]$Base = 'main',
  # Check slots and print what would start, without starting it. Test changes to this script this way:
  # on 14 November a "test" of the slot fix launched a real duplicate job.
  [switch]$DryRun
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
if (-not (Test-Path (Join-Path $Root "jobs\$Job\prompt.md"))) { throw "No spec: jobs\$Job\prompt.md" }

if ($Who -in 'terra','astra','sol') {
  if (-not $Worktree) { throw 'A GPT job needs -Worktree' }
  # AGENTS: one GPT job beside Astra, four Terra at most, fresh first prompts only. Sol 6.2 (S215, S217)
  # counts as a big model like Astra until its cost per run is measured.
  # Count jobs, not codex processes: between a job's resume runs no codex.exe exists for a moment, and
  # 1.3d launched into such a gap beside Astra and 3.3h (14 November, 15:22). A follow-up runs gpt.ps1
  # inside follow-up.ps1 and is Astra unless it names another model.
  $running = @(Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match '(gpt|follow-up)\.ps1' })
  $astra = @($running | Where-Object { $_.CommandLine -match 'astra|sol' -or ($_.CommandLine -match 'follow-up\.ps1' -and $_.CommandLine -notmatch 'terra') }).Count
  if ($running.Count -ge 4 -or ($astra -and $running.Count -ge 2) -or ($Who -in 'astra','sol' -and $running.Count -ge 2)) {
    throw "GPT lane full: $($running.Count) running ($astra Astra). Wait, or queue it in jobs\queue.txt."
  }
  if (-not (Test-Path $Worktree) -and -not $DryRun) {
    # git reports progress on stderr, which PS 5.1 turns into a terminating error under 'Stop'.
    $ErrorActionPreference = 'Continue'; git -C $Root worktree add -b "wt/$Job" $Worktree $Base 2>&1 | Out-Null; $ErrorActionPreference = 'Stop'
    if (-not (Test-Path $Worktree)) { throw "worktree failed: $Worktree from $Base" }
  }
  # The fixture world is git-ignored, so a new worktree lacks it and its track tests fail (14 November,
  # three times). Copy main's working fixture every launch, not only when the worktree has none: its
  # region files are tracked but refreshed in main's working tree, so a checkout gets stale ones and the
  # plain-path judge found a missing chunk (15 November). robocopy copies only the files that differ.
  # Never in a dry run: on 15 November the dry run's copy created the worktree folder, so the real
  # launch skipped `git worktree add` and Terra started in a folder that was not a checkout.
  $fixture = 'engine\world\tests\fixtures\world'
  if (-not $DryRun -and (Test-Path (Join-Path $Root $fixture))) {
    robocopy (Join-Path $Root $fixture) (Join-Path $Worktree $fixture) /E /NFL /NDL /NJH /NJS /NP | Out-Null
  }
  $model = @{ terra = 'gpt-6.1-terra'; astra = 'gpt-6.2-astra'; sol = 'gpt-6.2-sol' }[$Who]
  $a = @('-NoProfile','-ExecutionPolicy','Bypass','-File',"$PSScriptRoot\gpt.ps1",'-Job',$Job,'-Worktree',$Worktree,'-Model',$model)
}
# Start-Process does not quote its arguments and this repo's path has a space (clan.md, trap 1).
$q = $a | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
if ($DryRun) { "dry run: slots allow $Job on $Who ($model); would run: powershell $($q -join ' ')"; return }
$p = Start-Process powershell -ArgumentList $q -WorkingDirectory $Root -WindowStyle Hidden -PassThru
# Lanes run at Normal priority (S158, "no need for low priority"). Before a live gate, lower them to Idle:
# at Normal, L0 Preflight missed the lab clock's rate floor (986 against 1,000 ticks per second; S106).
Add-Content -Encoding utf8 (Join-Path $Root 'jobs\launch.log') "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $Job $Who $model pid $($p.Id) $Worktree"
"launched $Job on $Who ($model), pid $($p.Id)"
