# UserPromptSubmit hook (the overhaul's C1): his message goes verbatim to archive/turns/inbox.md, and the
# heuristics card is printed, so it is always the most recent instruction in context.
$utf8 = [Text.UTF8Encoding]::new($false)
$root = Split-Path (Split-Path $PSScriptRoot)
try {
  $in = [Console]::In.ReadToEnd() | ConvertFrom-Json
  if ($in.prompt) {
    $line = "`n### $(Get-Date -Format 'yyyy-MM-dd HH:mm')`n`n$($in.prompt)`n"
    [IO.File]::AppendAllText((Join-Path $root 'archive\turns\inbox.md'), $line, $utf8)
  }
} catch {}
[Console]::OutputEncoding = $utf8
[Console]::Out.Write([IO.File]::ReadAllText((Join-Path $PSScriptRoot 'card.md'), $utf8))
