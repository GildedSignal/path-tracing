# Structural checks on the actual fill/setblock data, without connecting to Minecraft.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\courses.ps1"
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
$courses = @(Get-FixtureCourses)
$expected = @('full-blocks', 'slabs-stairs', 'step-ups', 'ceilings', 'ladders-vines', 'water',
  'ice', 'slime', 'soul-sand', 'honey', 'cobweb', 'powder-snow', 'edges')
Assert (($courses.Name -join ',') -ceq ($expected -join ',')) 'L0 course classes/order changed.'
$previousMax = $null
foreach ($course in $courses) {
  $blocks = @{}
  foreach ($command in $course.Commands) {
    $parts = $command.Split(' ')
    if ($parts[0] -eq 'fill') {
      $min = @([int]$parts[1], [int]$parts[2], [int]$parts[3])
      $max = @([int]$parts[4], [int]$parts[5], [int]$parts[6])
      $block = $parts[7]
    } elseif ($parts[0] -eq 'setblock') {
      $min = @([int]$parts[1], [int]$parts[2], [int]$parts[3]); $max = $min
      $block = $parts[4]
    } else { throw "Unknown course construction command $command" }
    for ($axis = 0; $axis -lt 3; $axis++) {
      Assert ($min[$axis] -ge $course.Min[$axis] -and $max[$axis] -le $course.Max[$axis] -and $min[$axis] -le $max[$axis]) "$($course.Name) out-of-box command $command"
    }
    $volume = ($max[0]-$min[0]+1)*($max[1]-$min[1]+1)*($max[2]-$min[2]+1)
    Assert ($volume -le 32768) "Oversized fill $command"
    for ($x = $min[0]; $x -le $max[0]; $x++) {
      for ($y = $min[1]; $y -le $max[1]; $y++) {
        for ($z = $min[2]; $z -le $max[2]; $z++) { $blocks["$x,$y,$z"] = $block }
      }
    }
  }
  $start = @($course.Start.Split(',') | ForEach-Object { [double]::Parse($_, [Globalization.CultureInfo]::InvariantCulture) })
  $sx = [int][Math]::Floor($start[0]); $sy = [int]$start[1]; $sz = [int][Math]::Floor($start[2])
  Assert ($blocks["$sx,$sy,$sz"] -eq 'air' -and $blocks["$sx,$($sy+1),$sz"] -eq 'air') "$($course.Name) start is obstructed."
  Assert ($blocks["$sx,$($sy-1),$sz"] -eq 'stone') "$($course.Name) start cannot settle at its exact specified height."
  foreach ($x in $course.Min[0]..$course.Max[0]) {
    foreach ($z in $course.Min[2]..$course.Max[2]) {
      Assert ($blocks["$x,-64,$z"] -ne 'air' -and $blocks["$x,-45,$z"] -ne 'air') "$($course.Name) open floor or roof."
      if ($x -in @($course.Min[0], $course.Max[0]) -or $z -in @($course.Min[2], $course.Max[2])) {
        foreach ($y in -63..-46) { Assert ($blocks["$x,$y,$z"] -eq 'stone') "$($course.Name) opening in enclosure at $x,$y,$z." }
      }
    }
  }
  Assert ($blocks[($course.Marker -join ',')] -eq 'gold_block') "$($course.Name) missing last-block marker."
  if ($null -ne $previousMax) { Assert ($course.Min[0] -gt $previousMax+64) 'Course patches are too close.' }
  $previousMax = $course.Max[0]
  Write-Output "PASS $($course.Name) - $($course.Commands.Count) commands; enclosed; settled plain start."
}
Write-Output "Course structure checks passed ($($courses.Count) courses)."
