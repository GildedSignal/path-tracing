# Runs a GPT job through the Codex CLI that the Codex app installs (AGENTS.md). The prompt is the drift
# guard (the goal and philosophy sections of docs/PRINCIPLES.md, verbatim) followed by jobs/<id>/prompt.md.
#   .\scripts\gpt.ps1 -Job 1.2-recorder -Worktree D:\path-tracing-wt\recorder
#   .\scripts\gpt.ps1 -Job 1.2-recorder -Worktree D:\path-tracing-wt\recorder -Resume <session id>
param(
  [Parameter(Mandatory)][string]$Job,
  [Parameter(Mandatory)][string]$Worktree,
  [string]$Model = 'gpt-6.1-sol',
  [string]$Resume
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
[IO.File]::WriteAllText($full, "$guard`n`n---`n`n$job", $utf8)

$env:PATHTRACING_ROOT = $Rig
# The sandbox cannot read %LOCALAPPDATA%: builds use the staged copy (scripts\stage-deps.ps1), and the
# staged copy, the Rust homes and the shared cargo target dir are opened to the job.
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
$env:RUSTUP_HOME = 'D:\rust\rustup'; $env:CARGO_HOME = 'D:\rust\cargo'
$roots = "sandbox_workspace_write.writable_roots=['D:/path-tracing-deps','D:/rust','D:/path-tracing-target']"
# A writable root must already exist; the sandbox cannot create it.
New-Item -ItemType Directory -Force 'D:\path-tracing-target' | Out-Null
$codexArgs = if ($Resume) { @('exec', 'resume', $Resume, '-') } else { @('exec', '-m', $Model, '-s', 'workspace-write', '-c', $roots, '-C', $Worktree, '--json', '-o', (Join-Path $dir 'last.md'), '-') }
# Start-Process joins arguments with spaces and does not quote them; the repo path has a space.
$quoted = $codexArgs | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
$p = Start-Process $codex.FullName -ArgumentList $quoted -NoNewWindow -Wait -PassThru `
  -RedirectStandardInput $full -RedirectStandardOutput (Join-Path $dir 'log.jsonl') -RedirectStandardError (Join-Path $dir 'stderr.txt')
"codex exited with $($p.ExitCode); last message: $(Join-Path $dir 'last.md')"
