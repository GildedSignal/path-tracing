# Runs a Google job through the Antigravity CLI (AGENTS.md, docs/clan.md). The prompt is the drift guard
# (the goal and philosophy sections of docs/PRINCIPLES.md, verbatim) followed by jobs/<id>/prompt.md, piped
# on stdin because a job prompt is far past the Windows command-line limit.
#   .\scripts\gemini.ps1 -Job corpus-contradictions -Model gemini-4-argon-high
#   .\scripts\gemini.ps1 -Job <id> -Model gemini-4-hydrogen-high -Write D:\path-tracing-wt\<id>
#
# Read-only by default: Google's companions are given the repo to read and report on (clan.md section 3),
# and the report is the CLI's own JSON response, so nothing needs write access. -Write opts a job into
# editing one worktree.
#
# Slugs, from `agy models` on 13 November: gemini-4-argon-{low,medium,high},
# gemini-4-hydrogen-{low,medium,high,xhigh}, gemini-pro-orchestrator-g1-pro,
# claude-opus-5-5-{low,medium,high}, claude-sonnet-5-5-{low,medium,high}.
# The Claude slugs draw on Antigravity's own meter, not Julien's Anthropic plan: that is the point of
# routing judge and red-team work here (clan.md section 4, the meter order).
param(
  [Parameter(Mandatory)][string]$Job,
  [string]$Model = 'gemini-4-argon-high',
  [string]$Write,
  [string[]]$ReadDir = @(),   # extra directories to read, e.g. a worktree under judgment
  [int]$TimeoutMinutes = 60,
  # S129: the most a run may take from its meter (points of "remaining"), and how long its report may go
  # without growing. Over either, the run is killed: an empty Argon run once cost ~10% of its week, and an
  # Opus run the Claude meter's whole week, with nothing to show.
  [int]$Budget = 0,
  [int]$StallMinutes = 20,
  # S210, jobs/agy-commands: let the model run commands, inside agy's terminal sandbox. Needs -Write (the
  # worktree). See the -Commands block below for what the probes of 17 November established.
  [switch]$Commands
)
if ($Commands -and -not $Write) { throw '-Commands needs -Write <worktree>: commands run and write only there.' }
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
if ($Model -match 'opus') { throw 'Opus does not run on Antigravity (S115, S129): it spent the Claude meter''s week with no output. Use a native Opus sub-agent.' }

# The meter rows each model draws on, as `agy /usage` prints them (group, label: N% REMAINING).
$rows = if ($Model -match 'argon') { @('Gemini 4 Argon|Gemini 4 Argon Weekly Limit', 'Gemini Models|Five Hour Limit Remaining') }
        elseif ($Model -match 'hydrogen') { @('Gemini Models|Five Hour Limit Remaining', 'Gemini Models|Weekly Limit Remaining') }
        else { @('Claude and GPT models|Weekly Limit Remaining') }
if (-not $Budget) { $Budget = if ($Model -match 'argon') { 15 } elseif ($Model -match 'hydrogen') { 15 } else { 10 } }
function Read-Remaining {
  $agyExe = Join-Path $env:LOCALAPPDATA 'agy\bin\agy.exe'
  $in = Join-Path $env:TEMP "agy-usage-in-$PID.txt"; $out = Join-Path $env:TEMP "agy-usage-out-$PID.txt"
  [IO.File]::WriteAllText($in, "/usage`n")
  Start-Process $agyExe -RedirectStandardInput $in -RedirectStandardOutput $out -RedirectStandardError "$out.err" -NoNewWindow -Wait
  $m = @{}
  foreach ($l in (Get-Content $out -ErrorAction SilentlyContinue)) {
    $f = $l -split "`t"; if ($f.Count -ge 3 -and $f[2] -match '^(\d+)%$') { $m["$($f[0])|$($f[1])"] = [int]$Matches[1] }
  }
  $m
}
# Through `powershell -File` an array arrives as one comma-joined string (launch.ps1 passes it so).
$ReadDir = @($ReadDir | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
# Every run may read the project's own data roots on D: (all worktrees, the evidence, the lab's jars and
# decompiles): specs cite them, and one ungranted path ends a headless run with nothing (15 November:
# spec-premises died on the plain-path judge's worktree, which its -ReadDir did not name).
$ReadDir += @('D:\path-tracing-wt', 'D:\path-tracing-evidence', 'D:\path-tracing-lab') | Where-Object { Test-Path $_ }

$agy = Join-Path $env:LOCALAPPDATA 'agy\bin\agy.exe'
if (-not (Test-Path $agy)) { throw "agy.exe not found at $agy (install: irm https://antigravity.google/cli/install.ps1 | iex)" }

$dir = Join-Path $Root "jobs\$Job"
if (-not (Test-Path (Join-Path $dir 'prompt.md'))) { throw "No prompt at $dir\prompt.md" }
$utf8 = [Text.UTF8Encoding]::new($false)   # a BOM made settings.json be rejected until 1.5.8; never write one

$principles = [IO.File]::ReadAllText((Join-Path $Root 'docs\PRINCIPLES.md'), $utf8)
$guard = [regex]::Match($principles, '(?s)## The goal.*?(?=\n## From Emergent Space)').Value
if (-not $guard) { throw 'Could not find the goal and philosophy sections in docs/PRINCIPLES.md' }
$body = [IO.File]::ReadAllText((Join-Path $dir 'prompt.md'), $utf8)
$full = Join-Path $dir 'full-prompt.md'
# S113-S115: a refused read_file has twice ended a run with its whole report lost (Argon on 14 November:
# 352k in, 16k out, empty). So the job may write into its own folder only, and must write its report there
# as it goes; if the final response comes back empty, that file becomes last.md.
$reportFile = Join-Path $dir 'report.md'
# 14 November: a run that wrote only "Drafting in progress..." lost 208k tokens of reading to one denied read.
# S129, measured by scripts\test-antigravity.ps1: absolute drive-letter paths are read; a relative path or
# a file:/// URL is denied, and one denial ends the run with nothing.
$contract = "`n`n---`n`n**Read files only by absolute Windows paths with the drive letter** (``C:\...`` or ``D:\...``). A relative path or a ``file:///`` link is denied and ends your run at once with nothing kept: convert any link or relative path to an absolute drive-letter path before you read it.`n`n**Write your report as you go** into the file ``$reportFile``: it is the one file you may write. Create it at the start, and **append what you have found after every three or four files you read**, never only at the end: a denied read ends the run at once, and only what is already in that file survives. Your final message should repeat it."
if ($Commands) {
  $contract += "`n`n**You can run commands, inside a sandbox.** They may write only in ``$Write`` (your worktree), ``$dir`` (your job folder), ``D:\path-tracing-target\agy-$Job`` (build output) and ``D:\rust`` (the toolchain); any other write is refused. Your working directory may not be the worktree: start each command with ``cd $Write;`` or pass absolute paths, and build with ``--target-dir D:\path-tracing-target\agy-$Job``. Edit files with your file tools or with commands, inside the worktree only."
}
[IO.File]::WriteAllText($full, "$guard`n`n---`n`n$body$contract", $utf8)

# In headless mode there is no approval prompt: an action the permissions do not allow is denied and the
# run STILL EXITS 0, with only a note on stderr. So the exit code is not a gate here - the report is.
# On Windows a permission path has its drive letter stripped and its separators flipped, and a directory
# rule needs its trailing slash to be recursive. Get this wrong and the run still "succeeds" with an empty
# report, having been auto-denied - it cost one 197k-token run to learn.
# The docs say a Windows permission path has its drive letter stripped, and it does - but the agent's own
# first read often uses the drive-lettered form, is denied, and only sometimes retries a form that matches.
# When it does not retry, the run returns an EMPTY report having spent its tokens (measured twice: 9,481
# output tokens for nothing). So grant BOTH forms; the extra rule costs nothing and removes the race.
function Convert-PermPaths([string]$p) {
  $slash = ($p -replace '\\','/').TrimEnd('/') + '/'
  # The agent links files as file:///C:/Users/.../path%20tracing/..., and may read them in that form.
  # S129: the probe showed a file:/// URL read denied, ending the run; so every form an agent may use is
  # granted: drive and no drive, lower-case drive, a leading slash before the drive, file:/// URLs, %20.
  $drive = $slash; $lower = if ($slash -match '^[A-Za-z]:') { $slash.Substring(0,1).ToLower() + $slash.Substring(1) } else { $slash }
  $forms = @($drive, $lower, ($slash -replace '^[A-Za-z]:',''), "/$drive", "file:///$drive", "file:///$lower", "file://$drive")
  $forms + @($forms | ForEach-Object { $_ -replace ' ','%20' }) | Select-Object -Unique
}
$allow = @()
if ($Commands) {
  # Measured on 17 November (jobs/_probe-cmd, scripts/probe-agy-commands.ps1):
  # - Without --sandbox a command runs "unsandboxed" and needs a rule matching its WHOLE command line:
  #   command(cargo) does not allow `cargo --version`, command(cargo run) does not allow `cargo run --quiet`.
  # - With --sandbox, agy builds read-only mounts from the read rules and writable mounts from the write
  #   rules. Any form without a drive letter (/C:/..., file:///...) is rejected as "non-absolute file path"
  #   and NO command runs: that was the builds' failure (S210). So only drive-letter forms here.
  # - A writable mount comes only from a write rule WITHOUT the trailing slash; `D:/x/` stays read-only
  #   for commands. Writes outside the writable mounts are refused ("Acces refuse").
  # So: --sandbox, command(*), drive-letter read forms, and writable mounts for the worktree, the job
  # folder, the build target folder and the Rust toolchain (the same writable roots Codex gets).
  function Convert-DrivePaths([string]$p) { $s = ($p -replace '\\','/').TrimEnd('/'); @("$s/", ($s.Substring(0,1).ToLower() + $s.Substring(1) + '/')) }
  foreach ($d in @($Root) + $ReadDir) { foreach ($f in (Convert-DrivePaths $d)) { $allow += "read_file($f)" } }
  $target = "D:\path-tracing-target\agy-$Job"
  # Cargo makes a missing target folder through a temporary sibling in its parent, which is not writable:
  # create it here (measured: "Acces refuse ... at path D:\path-tracing-target\agy-<job>cFMi3n").
  New-Item -ItemType Directory -Force $target | Out-Null
  foreach ($w in @($Write, $dir, $target, 'D:\rust')) {
    $s = ($w -replace '\\','/').TrimEnd('/')
    $allow += @("write_file($s)", "write_file($s/)", "read_file($s/)")
  }
  $allow += 'command(*)'
} else {
  foreach ($f in (Convert-PermPaths $Root)) { $allow += "read_file($f)" }
  foreach ($d in $ReadDir) { foreach ($f in (Convert-PermPaths $d)) { $allow += "read_file($f)" } }
  foreach ($f in (Convert-PermPaths $dir)) { $allow += "write_file($f)"; $allow += "read_file($f)" }
  if ($Write) {
    foreach ($f in (Convert-PermPaths $Write)) { $allow += "write_file($f)" }
    $allow += @('command(git)', 'command(cargo)')
  }
}
$settings = @{ permissions = @{ allow = $allow } }
# --settings takes inline JSON or a file path. Inline loses its own quotes when Start-Process builds the
# command line, so always pass a file.
$settingsFile = Join-Path $dir 'settings.json'
[IO.File]::WriteAllText($settingsFile, ($settings | ConvertTo-Json -Depth 6), $utf8)

$a = @('--output-format','json','--model',$Model,'--print-timeout',"$($TimeoutMinutes)m",
       '--add-dir',$Root,'--settings',$settingsFile,'--log-file',(Join-Path $dir 'agy.log'))
foreach ($d in $ReadDir) { $a += @('--add-dir',$d) }
if ($Write) { $a += @('--add-dir',$Write,'--mode','accept-edits') }
if ($Commands) { $a += '--sandbox' }
# Start-Process joins arguments with spaces and does not quote them; the repo path has a space in it.
$quoted = $a | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }

$raw = Join-Path $dir 'result.json'
$err = Join-Path $dir 'stderr.txt'
$wd  = if ($Write) { $Write } else { $Root }
# Before: a run its meter cannot afford dies at the limit half-way, with its tokens spent. Refuse it.
$before = Read-Remaining
foreach ($r in $rows) {
  if (-not $before.ContainsKey($r)) { throw "agy /usage has no row '$r': cannot guard this run, so it does not start." }
  if ($before[$r] -lt $Budget) { throw "Meter '$r' has $($before[$r])% left, under this run's budget of ${Budget}: not started (S129)." }
}
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process $agy -ArgumentList $quoted -WorkingDirectory $wd -NoNewWindow -PassThru `
  -RedirectStandardInput $full -RedirectStandardOutput $raw -RedirectStandardError $err
# During: every two minutes, kill the run if it has taken more than its budget from any of its meters, or
# if its report has not grown for StallMinutes (it is reading without writing: what the empty runs did).
$killed = ''; $grown = Get-Date; $size = 0
while (-not $p.WaitForExit(120000)) {
  $now = (Get-Item $reportFile -ErrorAction SilentlyContinue).Length
  if ($now -gt $size) { $size = $now; $grown = Get-Date }
  $left = Read-Remaining
  foreach ($r in $rows) { if ($left.ContainsKey($r) -and $before[$r] - $left[$r] -gt $Budget) { $killed = "OVER BUDGET: '$r' fell from $($before[$r])% to $($left[$r])% (budget $Budget)" } }
  if (-not $killed -and ((Get-Date) - $grown).TotalMinutes -gt $StallMinutes) { $killed = "STALLED: report.md has not grown for $StallMinutes minutes" }
  if ($killed) {
    Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -eq $p.Id } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    Add-Content -Encoding utf8 (Join-Path $dir 'killed.txt') "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $killed"
    "KILLED: $killed"; break
  }
}
$sw.Stop()
$after = Read-Remaining
if ($killed -and (Get-Item $reportFile -ErrorAction SilentlyContinue).Length -gt 0) { Copy-Item $reportFile (Join-Path $dir 'last.md') -Force; "the report written before the kill is kept as last.md" }
"meter cost: " + (($rows | ForEach-Object { "$_ $($before[$_] - $after[$_]) points" }) -join '; ')

# The response becomes last.md, so a Google job leaves the same artefact a GPT job does.
$status = 'NO OUTPUT'; $usage = ''
if ((Get-Item $raw -ErrorAction SilentlyContinue).Length -gt 0) {
  $j = Get-Content $raw -Raw | ConvertFrom-Json
  $status = $j.status
  [IO.File]::WriteAllText((Join-Path $dir 'last.md'), [string]$j.response, $utf8)
  $u = $j.usage
  $usage = "in $($u.input_tokens) (cached $($u.cache_read_tokens)) out $($u.output_tokens) think $($u.thinking_tokens)"
  if ($j.error) { "error: $($j.error)" }
  # A soft-denied tool leaves status SUCCESS and exit 0, so neither is a gate. But `denied_actions` is not
  # one either: the agent often retries a different path form and recovers, logging the denial anyway
  # (measured: a run that read the file and returned the right answer still reported one). The honest
  # signal is an EMPTY response - that is the run that spent its tokens and produced nothing.
  if (-not [string]::IsNullOrWhiteSpace([string]$j.response)) {
    if ($j.denied_actions) { "note: $(@($j.denied_actions).Count) denied action(s), recovered - the report is non-empty" }
  } else {
    $status = 'EMPTY'
    # A detached run's console is seen by no one, so mark the folder: a partial report must never pass for
    # a finished one (15 November: spec-premises ended at batch 3 on a denial and looked done).
    [IO.File]::WriteAllText((Join-Path $dir 'incomplete.txt'), "$(Get-Date -Format 'yyyy-MM-dd HH:mm') empty final response; denied: $(@($j.denied_actions | ForEach-Object { $_.action }) -join ', ')`n", $utf8)
    if ((Get-Item $reportFile -ErrorAction SilentlyContinue).Length -gt 0) {
      Copy-Item $reportFile (Join-Path $dir 'last.md') -Force
      $status = 'EMPTY RESPONSE, REPORT KEPT'
      "the final response was empty; the report written as it went is kept as last.md"
    }
    "REFUSED: the run produced no report. If stderr mentions a permission, fix permissions.allow in"
    "$settingsFile; otherwise just run it again - an empty first turn has been seen and is not reproducible."
  }
}
"agy exited $($p.ExitCode) after $([int]$sw.Elapsed.TotalSeconds)s; status $status; $usage"
"report: $(Join-Path $dir 'last.md')"
if ((Get-Item $err -ErrorAction SilentlyContinue).Length -gt 0) {
  "stderr (read it: a soft-denied permission is reported only here, and still exits 0):"
  Get-Content $err -Tail 10
}
