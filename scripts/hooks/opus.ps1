# PreToolUse hook on the Agent tool: a native Opus sub-agent is refused unless its prompt names a job
# (jobs/<id>/prompt.md) whose jobs/<id>/shown.txt exists: Julien has seen the prompt (23 November: "next time
# before lunching an opus please show me the prompt"). An Agent call with no model inherits the integrator's
# Opus, so it counts too, except the small read-only helper types.
try { $in = [Console]::In.ReadToEnd() | ConvertFrom-Json } catch { exit 0 }
$t = $in.tool_input
$model = [string]$t.model
$type = [string]$t.subagent_type
$helpers = 'Explore', 'claude-code-guide', 'web-fetch', 'statusline-setup'
$opus = ($model -match 'opus') -or (-not $model -and $type -notin $helpers)
if (-not $opus) { exit 0 }
$root = Split-Path (Split-Path $PSScriptRoot)
$m = [regex]::Match([string]$t.prompt, 'jobs[\\/]([A-Za-z0-9._-]+)[\\/]prompt\.md')
if ($m.Success -and (Test-Path (Join-Path $root "jobs\$($m.Groups[1].Value)\shown.txt"))) { exit 0 }
[Console]::Error.WriteLine('Refused: an Opus sub-agent needs a job prompt Julien has seen. Write jobs\<id>\prompt.md, show it to him, create jobs\<id>\shown.txt, and name jobs\<id>\prompt.md in the agent prompt. (For a Sonnet or Haiku helper, pass model explicitly.)')
exit 2
