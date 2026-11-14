# The first flight's scoreboard, regenerated from a live play log (AGENTS: every number in a doc comes from a script).
#   .\scripts\flight-scoreboard.ps1 [-Log D:\path-tracing-evidence\flight-20261114\live.txt]
param([string]$Log = 'D:\path-tracing-evidence\flight-20261114\live.txt')
Get-Content $Log | ForEach-Object {
  if ($_ -match '^=== (\S+)') { $script:t = $Matches[1] }
  elseif ($_ -match 'Predicted T: (\d+) ticks; clock T: (\S+) ticks; Julien.s best: (\d+)[^;]*; thinking: (\d+) ms') {
    '{0,-22} SENTENCE  predicted {1,4}  clock {2,4}  his {3,4}  gap {4,5}  thinking {5,5} ms' -f $script:t, $Matches[1], $Matches[2], $Matches[3], ([int]$Matches[2] - [int]$Matches[3]), $Matches[4] }
  elseif ($_ -match '^NO SENTENCE') { '{0,-22} NO SENTENCE' -f $script:t }
  elseif ($_ -match '"corrections":(\d+),"pushes":(\d+)') { '{0,-22}   corrections {1}, pushes {2}' -f '', $Matches[1], $Matches[2] }
}