# Runs a GPT job through the Codex CLI that the Codex app installs (AGENTS.md). The prompt is the drift
# guard (the goal and philosophy sections of docs/PRINCIPLES.md, verbatim) followed by jobs/<id>/prompt.md.
#   .\scripts\gpt.ps1 -Job 1.2-recorder -Worktree D:\path-tracing-wt\recorder
#   .\scripts\gpt.ps1 -Job 1.2-recorder -Worktree D:\path-tracing-wt\recorder -Resume <session id>
param(
  [Parameter(Mandatory)][string]$Job,
  [Parameter(Mandatory)][string]$Worktree,
  # Terra by default (S99 retired Sol; S103: Sol is twice Terra's price for little gain). Astra only for
  # what Terra stalls on.
  [string]$Model = 'gpt-6.1-terra',
  [string]$Resume,
  # Pinned (clan.md 4b): Terra's factuality error is 16.7% at low effort against 4.3% at max, so an
  # unpinned default is unusable for anything load-bearing.
  [string]$Effort = 'high',
  [int]$MaxResumes = 4
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"

# The app keeps the CLI under a hash folder that changes with updates: take the newest.
$codex = Get-ChildItem "$env:LOCALAPPDATA\OpenAI\Codex\bin\*\codex.exe" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $codex) { throw 'codex.exe not found under %LOCALAPPDATA%\OpenAI\Codex\bin (is the Codex app installed?)' }

$dir = Join-Path $Root "jobs\$Job"
$utf8 = [Text.UTF8Encoding]::new($false)
$principles = [IO.File]::ReadAllText((Join-Path $Root 'docs\PRINCIPLES.md'), $utf8)
$guard = [regex]::Match($principles, '(?s)## The goal.*?(?=\n## From Emergent Space)').Value
if (-not $guard) { throw 'Could not find the goal and philosophy sections in docs/PRINCIPLES.md' }
$job = [IO.File]::ReadAllText((Join-Path $dir 'prompt.md'), $utf8)
$full = Join-Path $dir 'full-prompt.md'
# S104: Terra ends its turn after a few minutes and ~20 lines, reporting "incomplete", whatever the spec
# says. So every prompt ends with a status contract, and a run that does not end in DONE or BLOCKED is
# resumed in the same session (context kept) up to $MaxResumes times.
$contract = @'

---

**Your final message must end with exactly one of these lines:** `STATUS: DONE` when every item of "Done means" holds, or `STATUS: BLOCKED: <the one thing you cannot do, and why>` when something outside your control stops you. Anything else is read as unfinished, and you will be resumed in this same session to continue. Partial progress is not a stopping point: keep going until DONE or BLOCKED.
'@
[IO.File]::WriteAllText($full, "$guard`n`n---`n`n$job$contract", $utf8)

$env:PATHTRACING_ROOT = $Rig
# The sandbox cannot read %LOCALAPPDATA%: builds use the staged copy (scripts\stage-deps.ps1), and the
# staged copy, the Rust homes, the shared cargo target dir, the reference rig (its lock and disposable worlds) and the evidence folder are opened to the job. Julien's own world (server in this checkout) is not.
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
$env:RUSTUP_HOME = 'D:\rust\rustup'; $env:CARGO_HOME = 'D:\rust\cargo'
$roots = "sandbox_workspace_write.writable_roots=['D:/path-tracing-deps','D:/rust','D:/path-tracing-target','D:/path-tracing-lab','D:/path-tracing-evidence']"
$effortArg = "model_reasoning_effort=`"$Effort`""
# A writable root must already exist; the sandbox cannot create it.
New-Item -ItemType Directory -Force 'D:\path-tracing-target' | Out-Null
# A resumed session must keep its worktree, writable roots and report file, or it loses write access (job 1.1).
$codexArgs = if ($Resume) { @('exec', 'resume', '-c', $roots, '-c', $effortArg, '-c', 'sandbox_mode=workspace-write', '--json', '-o', (Join-Path $dir 'last.md'), $Resume, '-') } else { @('exec', '-m', $Model, '-s', 'workspace-write', '-c', $roots, '-c', $effortArg, '-C', $Worktree, '--json', '-o', (Join-Path $dir 'last.md'), '-') }
function Invoke-Codex($arguments, $stdin, $n) {
  # Start-Process joins arguments with spaces and does not quote them; the repo path has a space.
  $quoted = $arguments | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
  $log = if ($n) { "log-$n.jsonl" } else { 'log.jsonl' }
  $p = Start-Process $codex.FullName -ArgumentList $quoted -WorkingDirectory $Worktree -NoNewWindow -Wait -PassThru `
    -RedirectStandardInput $stdin -RedirectStandardOutput (Join-Path $dir $log) -RedirectStandardError (Join-Path $dir 'stderr.txt')
  "codex exited with $($p.ExitCode) (run $n)"
}
Invoke-Codex $codexArgs $full 0
$report = Join-Path $dir 'last.md'
$session = if ($Resume) { $Resume } else {
  $line = Get-Content (Join-Path $dir 'log.jsonl') -TotalCount 1 -ErrorAction SilentlyContinue
  if ($line -match '"thread_id":"([^"]+)"') { $Matches[1] } }
$more = Join-Path $dir 'continue.md'
[IO.File]::WriteAllText($more, "Continue the same job from where you stopped. Re-read your last message's list of what remains, do it, and keep going until every item of `"Done means`" holds. End with ``STATUS: DONE`` or ``STATUS: BLOCKED: <why>``.", $utf8)
for ($n = 1; $n -le $MaxResumes -and $session; $n++) {
  $tail = (Get-Content $report -Raw -ErrorAction SilentlyContinue)
  if ($tail -match 'STATUS: (DONE|BLOCKED)') { break }
  Copy-Item $report (Join-Path $dir "last-$($n-1).md") -Force -ErrorAction SilentlyContinue
  $resumeArgs = @('exec', 'resume', '-c', $roots, '-c', $effortArg, '-c', 'sandbox_mode=workspace-write', '--json', '-o', $report, $session, '-')
  Invoke-Codex $resumeArgs $more $n
}
"last message: $report"
