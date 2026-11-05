# Builds the L0 test courses (x = 4096 + 128*i, z = 4096) in the superflat world and places command blocks:
#   spawn:       a button that takes you onto the roof of the first course;
#   each roof:   "back to spawn" and "next course" buttons.
# The courses are closed boxes, and nothing is placed inside them, so the recordings stay what the law expects.
# To look inside a course, switch to spectator (F3+N) and fly through the roof; F3+N again to press buttons.
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\place-teleports.ps1
# Needs the server running. Safe to run again: it rebuilds the courses and the panels.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
if (-not (Test-Port $ServerHost $RconPort)) { throw 'The server is not running (RCON closed). Start it first.' }

$coursesFile = Join-Path $Root 'lab\gates\courses.ps1'
if (-not (Test-Path $coursesFile)) { $coursesFile = 'D:\path-tracing-wt\fixtures\lab\gates\courses.ps1' }
. $coursesFile
$courses = @(Get-FixtureCourses)

function Rcon([string[]]$Commands) { & "$PSScriptRoot\rcon.ps1" @Commands }
function Wait-Loaded([int]$X, [int]$Z) {
  for ($i = 0; $i -lt 100; $i++) {
    $reply = Rcon "execute if loaded $X -60 $Z"
    if ("$reply" -match 'passed') { return }
    Start-Sleep -Milliseconds 100
  }
  throw "Chunk at $X $Z did not load."
}
function Panel([int]$X, [int]$Y, [int]$Z, [string]$Label, [string]$Command) {
  # A command block with a button on its south face and a sign on top, readable from the south.
  $c = $Command.Replace('"', '\"')
  Rcon "setblock $X $Y $Z command_block[facing=up]{Command:`"$c`"} replace",
       "setblock $X $Y $($Z + 1) stone_button[face=wall,facing=south] replace",
       "setblock $X $($Y + 1) $Z oak_sign[rotation=0]{front_text:{messages:[`"$Label`",`"`",`"`",`"`"]}} replace"
}

Rcon 'gamerule command_blocks_work true'

for ($i = 0; $i -lt $courses.Count; $i++) {
  $course = $courses[$i]
  $x = 4096 + 128 * $i; $z = 4096
  Rcon "forceload add $x $z $($x + 31) $($z + 31)"
  Wait-Loaded ($x + 16) ($z + 16)
  $replies = Rcon $course.Commands
  $bad = @($replies | Where-Object { "$_" -match 'error|unknown|incorrect|not loaded|Expected' })
  if ($bad) { throw "Course $($course.Name): $($bad -join ' | ')" }
  # Roof panel: the roof is at y = -45, so the panel stands on it at y = -44.
  $next = $courses[($i + 1) % $courses.Count]
  $nx = 4096 + 128 * (($i + 1) % $courses.Count)
  Panel ($x + 14) -44 ($z + 13) 'Spawn' 'tp @p 0.5 -60 2.5 180 0'
  Panel ($x + 18) -44 ($z + 13) ('Next: ' + $next.Name) "tp @p $($nx + 16) -44 $($z + 17) 180 0"
  Rcon "setblock $($x + 16) -43 $($z + 13) oak_sign[rotation=0]{front_text:{messages:[`"$($course.Name)`",`"`",`"`",`"`"]}} replace"
  Rcon "forceload remove $x $z $($x + 31) $($z + 31)"
  Write-Host "Built $($course.Name) at x=$x."
}

# Spawn panel, 4 blocks north of spawn, facing it.
Rcon 'forceload add 0 -4'
Wait-Loaded 0 -4
Panel 0 -60 -4 'Courses' "tp @p 4112 -44 4113 180 0"
Rcon 'forceload remove 0 -4'
Write-Host 'Done: the Courses button is 4 blocks north of spawn.'
