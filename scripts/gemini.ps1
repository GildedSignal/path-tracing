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
  [int]$TimeoutMinutes = 60
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
# Through `powershell -File` an array arrives as one comma-joined string (launch.ps1 passes it so).
$ReadDir = @($ReadDir | ForEach-Object { $_ -split ',' } | Where-Object { $_ })

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
$contract = "`n`n---`n`n**Write your report as you go** into the file ``$reportFile`` (create it at the start, then keep it current); it is the one file you may write. Your final message should repeat it."
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
  @($slash, ($slash -replace '^[A-Za-z]:','')) | Select-Object -Unique
}
$allow = @(); foreach ($f in (Convert-PermPaths $Root)) { $allow += "read_file($f)" }
foreach ($d in $ReadDir) { foreach ($f in (Convert-PermPaths $d)) { $allow += "read_file($f)" } }
foreach ($f in (Convert-PermPaths $dir)) { $allow += "write_file($f)"; $allow += "read_file($f)" }
if ($Write) {
  foreach ($f in (Convert-PermPaths $Write)) { $allow += "write_file($f)" }
  $allow += @('command(git)', 'command(cargo)')
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
# Start-Process joins arguments with spaces and does not quote them; the repo path has a space in it.
$quoted = $a | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }

$raw = Join-Path $dir 'result.json'
$err = Join-Path $dir 'stderr.txt'
$wd  = if ($Write) { $Write } else { $Root }
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process $agy -ArgumentList $quoted -WorkingDirectory $wd -NoNewWindow -Wait -PassThru `
  -RedirectStandardInput $full -RedirectStandardOutput $raw -RedirectStandardError $err
$sw.Stop()

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
