param([string]$SourceWorld = 'C:/Users/PASCAL/Desktop/path tracing/server/world', [switch]$TrimOnly)
$ErrorActionPreference = 'Stop'
$fixtures = Join-Path $PSScriptRoot '../tests/fixtures'
$destination = "$fixtures/world"
$regions = "$destination/dimensions/minecraft/overworld/region"
# The chunks the tests read: the superflat and cache tests in r.0.0, the four courses in r.8.8.
$keep = @{
    'r.0.0.mca' = @(@(0,0),@(1,0),@(2,0),@(1,1))
    'r.8.8.mca' = foreach ($x in 256,257,264,265,272,273,280,281) { foreach ($z in 256,257) { ,@($x,$z) } }
}
if (-not $TrimOnly) {
    New-Item -ItemType Directory -Force $regions,"$destination/data/minecraft" | Out-Null
    # Source files are only read. Neither this script nor the Rust tests open them for writing.
    foreach ($relative in @('level.dat','data/minecraft/game_rules.dat','data/minecraft/world_gen_settings.dat',
        'dimensions/minecraft/overworld/region/r.0.0.mca','dimensions/minecraft/overworld/region/r.8.8.mca')) {
        Copy-Item -LiteralPath (Join-Path $SourceWorld $relative) -Destination (Join-Path $destination $relative)
    }
    # Run main's course definitions unmodified; course-source.txt names the blob.
    $courses = Join-Path ([IO.Path]::GetTempPath()) 'world-fixture-courses.ps1'
    & git show main:lab/gates/courses.ps1 | Set-Content $courses -Encoding UTF8
    if ($LASTEXITCODE -ne 0) { throw 'Could not read main course definitions' }
    . $courses
    Get-FixtureCourses | Where-Object { $_.Name -in @('full-blocks','slabs-stairs','step-ups','ceilings') } | ForEach-Object {
        @('# '+$_.Name) + $_.Commands
    } | Set-Content "$fixtures/commands.txt" -Encoding ASCII
    & git rev-parse main:lab/gates/courses.ps1 | Set-Content "$fixtures/course-source.txt" -Encoding ASCII
}
# Keep only the listed chunks: their location and timestamp entries and their sectors,
# byte for byte, repacked from sector 2. Every other slot reads as not generated.
foreach ($name in $keep.Keys) {
    $path = Join-Path $regions $name
    $in = [IO.File]::ReadAllBytes($path)
    $out = New-Object System.Collections.Generic.List[byte]
    $out.AddRange([byte[]]::new(8192))
    foreach ($c in $keep[$name]) {
        $slot = 4 * (($c[0] -band 31) + 32 * ($c[1] -band 31))
        $sector = ([int]$in[$slot] -shl 16) -bor ([int]$in[$slot+1] -shl 8) -bor $in[$slot+2]
        $count = [int]$in[$slot+3]
        if ($sector -lt 2 -or $count -eq 0) { throw "$name has no chunk $($c -join ',')" }
        $at = $out.Count / 4096
        $out[$slot] = [byte](($at -shr 16) -band 255); $out[$slot+1] = [byte](($at -shr 8) -band 255)
        $out[$slot+2] = [byte]($at -band 255); $out[$slot+3] = [byte]$count
        for ($i = 0; $i -lt 4; $i++) { $out[4096+$slot+$i] = $in[4096+$slot+$i] }
        $out.AddRange([byte[]]$in[($sector*4096)..(($sector+$count)*4096-1)])
    }
    [IO.File]::WriteAllBytes($path, $out.ToArray())
}
