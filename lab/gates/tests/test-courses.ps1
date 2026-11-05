# Structural checks on the actual fill/setblock data, without connecting to Minecraft.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\courses.ps1"
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
$courses = @(Get-FixtureCourses)
$expected = @('full-blocks', 'slabs-stairs', 'step-ups', 'ceilings', 'ladders-vines', 'water',
  'ice', 'slime', 'soul-sand', 'honey', 'cobweb', 'powder-snow', 'edges', 'corner')
Assert (($courses.Name -join ',') -ceq ($expected -join ',')) 'L0 course classes/order changed.'

function Trace-Water($Blocks) {
  # Vanilla water on these courses meets only air, water and full stone cubes.
  # Downward flow has priority; horizontal range is seven with a four-cell slope search.
  $directions = @(@(1,0), @(-1,0), @(0,1), @(0,-1))
  function Open([int]$X, [int]$Y, [int]$Z) { $Blocks["$X,$Y,$Z"] -eq 'air' }
  function Drop([int]$X, [int]$Y, [int]$Z) { Open $X ($Y-1) $Z }
  function Slope([int]$X, [int]$Y, [int]$Z, [int]$Depth, [int]$BackX, [int]$BackZ) {
    $best = 1000
    foreach ($d in $directions) {
      $nx=$X+$d[0]; $nz=$Z+$d[1]
      if (($nx -eq $BackX -and $nz -eq $BackZ) -or -not (Open $nx $Y $nz)) { continue }
      if (Drop $nx $Y $nz) { return $Depth }
      if ($Depth -lt 4) { $best = [Math]::Min($best, (Slope $nx $Y $nz ($Depth+1) $X $Z)) }
    }
    $best
  }
  $levels = @{}; $queue = [Collections.Generic.Queue[object]]::new()
  foreach ($entry in $Blocks.GetEnumerator()) {
    if ($entry.Value -match '^water(?:\[level=0\])?$') {
      $p = @($entry.Key.Split(',') | ForEach-Object { [int]$_ })
      $levels[$entry.Key] = 0; $queue.Enqueue(@($p[0],$p[1],$p[2],0))
    }
  }
  function Add-Flow([int]$X, [int]$Y, [int]$Z, [int]$Level) {
    $key = "$X,$Y,$Z"
    if ((Open $X $Y $Z) -and (-not $levels.ContainsKey($key) -or $Level -lt $levels[$key])) {
      $levels[$key]=$Level; $queue.Enqueue(@($X,$Y,$Z,$Level))
    }
  }
  while ($queue.Count -gt 0) {
    $p=$queue.Dequeue(); $x=$p[0]; $y=$p[1]; $z=$p[2]; $level=$p[3]
    if (Drop $x $y $z) {
      Add-Flow $x ($y-1) $z 8
      if ($level -ne 0) { continue }
    }
    $next = if ($level -eq 8) { 1 } else { $level+1 }
    if ($next -gt 7) { continue }
    $choices=@(); $best=1000
    foreach ($d in $directions) {
      $nx=$x+$d[0]; $nz=$z+$d[1]
      if (-not (Open $nx $y $nz)) { continue }
      $slope = if (Drop $nx $y $nz) { 0 } else { Slope $nx $y $nz 1 $x $z }
      $best=[Math]::Min($best,$slope)
      $choices += ,@($nx,$nz,$slope)
    }
    foreach ($choice in $choices) { if ($choice[2] -eq $best) { Add-Flow $choice[0] $y $choice[1] $next } }
  }
  $levels
}
function Assert-DryStart($Blocks, $Start, [string]$Name) {
  $wet = Trace-Water $Blocks
  foreach ($key in $wet.Keys) {
    $p=@($key.Split(',') | ForEach-Object { [int]$_ })
    $distance2=0.0
    for ($a=0; $a -lt 3; $a++) {
      $gap=[Math]::Max(0, [Math]::Max($p[$a]-$Start[$a], $Start[$a]-($p[$a]+1)))
      $distance2 += $gap*$gap
    }
    Assert ($distance2 -gt 9) "$Name fluid reaches within 3 blocks of start at $key."
  }
  $wet
}
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
  $wet = Assert-DryStart $blocks $start $course.Name
  if ($course.Name -eq 'water') {
    foreach ($key in $wet.Keys) { Assert ([int]$key.Split(',')[2] -ge $course.Min[2]+7) 'Water reached the plain strip.' }
    # Negative control: removing the start-facing upper rim must flood the start vicinity.
    $mutant = $blocks.Clone()
    for ($x=$course.Min[0]+1; $x -le $course.Max[0]-1; $x++) { $mutant["$x,-60,$($course.Min[2]+7)"]='air' }
    $failed=$false
    try { $null = Assert-DryStart $mutant $start 'missing-rim' } catch {
      if ($_.Exception.Message -notlike '*fluid reaches within 3 blocks of start*') { throw }
      $failed=$true
    }
    Assert $failed 'Fluid tracing did not reject the missing-rim mutant.'
  }
  if ($course.Name -eq 'edges') {
    Assert ($course.SneakPercent -eq 90) 'Edges do not use the course-specific sneak setting.'
    foreach ($step in @(@(1,-61),@(2,-60),@(3,-59),@(4,-58))) {
      $key="$sx,$($step[1]),$($course.Min[2]+$step[0])"
      Assert ($blocks[$key] -eq 'stone') 'Catch floor lacks the return staircase.'
      Assert ($blocks["$sx,$($step[1]+1),$($course.Min[2]+$step[0])"] -eq 'air') 'Return staircase is obstructed.'
    }
  }
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
