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
[IO.File]::WriteAllText($full, "$guard`n`n---`n`n$body", $utf8)

# In headless mode there is no approval prompt: an action the permissions do not allow is denied and the
# run STILL EXITS 0, with only a note on stderr. So the exit code is not a gate here - the report is.
# On Windows a permission path has its drive letter stripped and its separators flipped, and a directory
# rule needs its trailing slash to be recursive. Get this wrong and the run still "succeeds" with an empty
# report, having been auto-denied - it cost one 197k-token run to learn.
function Convert-PermPath([string]$p) { (($p -replace '\\','/') -replace '^[A-Za-z]:','') + '/' }
$settings = @{ permissions = @{ allow = @("read_file($(Convert-PermPath $Root))") } }
foreach ($d in $ReadDir) { $settings.permissions.allow += "read_file($(Convert-PermPath $d))" }
if ($Write) {
  $settings.permissions.allow += @("write_file($(Convert-PermPath $Write))", 'command(git)', 'command(cargo)')
}
# --settings takes inline JSON or a file path. Inline loses its own quotes when Start-Process builds the
# command line, so always pass a file.
$settingsFile = Join-Path $dir 'settings.json'
[IO.File]::WriteAllText($settingsFile, ($settings | ConvertTo-Json -Depth 6), $utf8)

$a = @('--output-format','json','--model',$Model,'--print-timeout',"$($TimeoutMinutes)m",
       '--add-dir',$Root,'--settings',$settingsFile)
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
  # A soft-denied tool leaves status SUCCESS and exit 0. The report is the gate, and this is the gate.
  if ($j.denied_actions) {
    $status = "DENIED (" + (($j.denied_actions | ForEach-Object { $_.action }) -join ', ') + ")"
    "REFUSED: headless mode auto-denied a tool it could not prompt for. The report is empty or partial and"
    "this run must not be trusted. Fix the permissions.allow rules in $settingsFile and run again."
  }
}
"agy exited $($p.ExitCode) after $([int]$sw.Elapsed.TotalSeconds)s; status $status; $usage"
"report: $(Join-Path $dir 'last.md')"
if ((Get-Item $err -ErrorAction SilentlyContinue).Length -gt 0) {
  "stderr (read it: a soft-denied permission is reported only here, and still exits 0):"
  Get-Content $err -Tail 10
}
