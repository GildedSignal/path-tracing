[CmdletBinding()]
param(
  [Parameter(Mandatory)][ValidatePattern('\A[a-z0-9_-]+\z', Options=0)][string]$Track,
  [ValidateSet('3','5','b','solve')][string]$Mode = '5',
  [string]$Rig,
  [switch]$DryRun
)
$ErrorActionPreference = 'Stop'
$requestedRig = $Rig
. "$PSScriptRoot\common.ps1"
if ($requestedRig) { $Rig = [IO.Path]::GetFullPath($requestedRig) }
$previousRig = $env:PATHTRACING_ROOT
$env:PATHTRACING_ROOT = $Rig
$locked = $false
$reader = $null
$script:pendingReply = ''
$armed = $false
try {
  $directory = Join-Path $Rig 'lab/tracks/plans'
  New-Item -ItemType Directory -Force -Path $directory | Out-Null
  $stamp = [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fffffff')
  $plan = Join-Path $directory "$Track-$stamp.jsonl"
  if (!$DryRun) {
    Enter-Rig "play-track $Track"; $locked = $true
    $log = Join-Path $Rig 'clients/bot/logs/latest.log'
    $stream = [IO.File]::Open($log, 'Open', 'Read', ([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    $null = $stream.Seek(0, [IO.SeekOrigin]::End)
    $reader = [IO.StreamReader]::new($stream)
  }
  function Wait-Reply([string]$Pattern, [int]$Seconds) {
    $deadline = [Diagnostics.Stopwatch]::StartNew()
    while ($deadline.Elapsed.TotalSeconds -lt $Seconds) {
      $script:pendingReply += $reader.ReadToEnd()
      while (($i = $script:pendingReply.IndexOf("`n")) -ge 0) {
        $line = $script:pendingReply.Substring(0, $i).TrimEnd([char]13); $script:pendingReply = $script:pendingReply.Substring($i + 1)
        if ($line.Contains('[pathtracing] ')) {
          if ($line -match 'Track:|Track preparation failed|Track tick failed|Track run discarded|Recording finalization failed') { throw $line }
          if ($line -match $Pattern) { return $line }
        }
      }
      Start-Sleep -Milliseconds 50
    }
    throw "Timed out waiting for $Pattern"
  }
  $clock = [Diagnostics.Stopwatch]::StartNew()
  if ($DryRun) { Write-Output "Would send: track play $Track" }
  else {
    $null = & "$PSScriptRoot\rcon.ps1" "msg PathBot track play $Track"
    $armed = $true
    $null = Wait-Reply ("Armed " + [regex]::Escape($Track) + '; waiting for a plan\.') 60
  }
  Push-Location (Join-Path $Root 'engine')
  try {
    # Call the built binary: `cargo run` spent about 450 ms per click starting cargo and checking freshness,
    # 61% of the measured thinking on stonesimple (14 November). Build it first with cargo build --release.
    $exe = Join-Path $(if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'engine\target' }) 'release\plan-track.exe'
    if (!(Test-Path -LiteralPath $exe)) { throw "Build the planner first: cargo build --offline --release -p plan --bins (missing $exe)" }
    $planner = & $exe $Track --start settled --mode $Mode --out $plan
    $code = $LASTEXITCODE
  } finally { Pop-Location }
  $clock.Stop()
  $result = ($planner | Where-Object { $_ -match '^(SENTENCE|NO SENTENCE) ' } | Select-Object -Last 1)
  if ($code -ne 0 -or !$result -or $result -notmatch '^SENTENCE (\d+) thinking ') {
    Write-Output $(if ($result) { $result } else { 'NO SENTENCE planner failed without a reason' })
    if ($DryRun) { Write-Output 'Would send: track cancel' } else { $null = & "$PSScriptRoot\rcon.ps1" 'msg PathBot track cancel'; $armed = $false }
    return
  }
  $predicted = [int]$Matches[1]
  if (!(Test-Path -LiteralPath $plan -PathType Leaf)) { throw 'Planner reported a sentence but did not write the plan.' }
  Write-Output "Thinking: $($clock.Elapsed.TotalMilliseconds.ToString('F0')) ms"
  Write-Output "Plan: $plan; predicted T: $predicted ticks"
  if ($DryRun) { Write-Output "Would send: track plan $plan"; Write-Output 'Would wait for run row and report clock T, human best and recording; deviation skipped in dry run.'; return }
  $null = & "$PSScriptRoot\rcon.ps1" "msg PathBot track plan $plan"
  $null = Wait-Reply 'Plan accepted:' 30
  $null = Wait-Reply ([regex]::Escape($Track) + ' finished in ') 330
  $armed = $false
  $runs = Join-Path $Rig "lab/tracks/runs/$Track.jsonl"
  $row = Get-Content -LiteralPath $runs | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.plan -ceq $plan } | Select-Object -Last 1
  if (!$row) { throw 'Finished reply did not produce a matching run row.' }
  $definition = Get-Content -LiteralPath (Join-Path $Rig "lab/tracks/$Track.json") -Raw | ConvertFrom-Json
  $human = Get-Content -LiteralPath $runs | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.player -eq $PlayerName -and $_.trackCreated -eq $definition.created } | ForEach-Object { if ($_.clockOrigin -ne 'lastOnA') { $_.T += 1 }; $_ } | Sort-Object T | Select-Object -First 1
  $best = if ($human) { "$($human.T) ticks" } else { 'unavailable' }
  $recording = if ($row.recording) { Join-Path $Rig $row.recording } else { "unavailable: $($row.recordingNote)" }
  Write-Output "Predicted T: $predicted ticks; clock T: $($row.T) ticks; Julien's best: $best; thinking: $($clock.Elapsed.TotalMilliseconds.ToString('F0')) ms; recording: $recording"
  if (Test-Path (Join-Path $Root 'engine/plan/src/bin/deviation.rs')) {
    Push-Location (Join-Path $Root 'engine')
    try { & cargo run --offline --release -q -p plan --bin deviation -- $plan $recording | Select-Object -First 1 } finally { Pop-Location }
  } else { Write-Output 'Deviation: skipped (deviation.rs absent).' }
} finally {
  if ($armed -and !$DryRun) { try { $null = & "$PSScriptRoot\rcon.ps1" 'msg PathBot track cancel' } catch { Write-Warning "Cancel failed: $_" } }
  if ($reader) { $reader.Dispose() }
  if ($locked) { Exit-Rig }
  $env:PATHTRACING_ROOT = $previousRig
}
