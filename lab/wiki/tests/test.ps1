$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$compiler = Join-Path $root 'scripts\lab-wiki.ps1'
$out = Join-Path $env:TEMP ('lab-wiki-test-' + [guid]::NewGuid().ToString('N'))
$previous = $env:ZAI_API_KEY
$env:ZAI_API_KEY = 'fixture-secret-value'
try {
  & $compiler -Results (Join-Path $PSScriptRoot 'results') -Output $out
  $first = Get-Content (Join-Path $out 'fixture.md') -Raw -Encoding UTF8
  if ($first -notmatch 'Grade: reproduced' -or $first -notmatch 'Grade: rate-limited') { throw 'Grade classification failed.' }
  if ($first.IndexOf('1653') -gt $first.IndexOf('1554')) { throw 'Runs are not newest first.' }
  $before = @{}; Get-ChildItem $out -File | ForEach-Object { $before[$_.Name] = (Get-FileHash $_.FullName -Algorithm SHA256).Hash }
  & $compiler -Results (Join-Path $PSScriptRoot 'results') -Output $out
  foreach ($file in Get-ChildItem $out -File) { if ($before[$file.Name] -ne (Get-FileHash $file.FullName -Algorithm SHA256).Hash) { throw 'Output changed on the second run.' } }
  Write-Host 'PASS: grades and deterministic output'
} finally { $env:ZAI_API_KEY = $previous; if (Test-Path $out) { Remove-Item -LiteralPath $out -Recurse -Force } }
