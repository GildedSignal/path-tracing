# Job 1.2's numbers: the workspace suite, each planted mutant caught, the randomness audit.
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$engine = Join-Path $root 'engine'
# One log folder per checkout: parallel jobs shared one and were denied each other's workspace.log (S104).
$output = Join-Path 'D:\path-tracing-deps\footprints-verification' (Split-Path -Leaf $root)
New-Item -ItemType Directory -Force -Path $output | Out-Null

function Cargo-Check([string]$Name, [string[]]$Arguments, [switch]$MustFail) {
    $log = Join-Path $output "$Name.log"
    $previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $lines = & cargo @Arguments 2>&1
        $code = $LASTEXITCODE
    } finally { $ErrorActionPreference = $previous }
    $lines | Set-Content -Encoding UTF8 -LiteralPath $log
    if ($MustFail) {
        if ($code -eq 0 -or !(($lines -join "`n") -match 'test result: FAILED')) { throw "$Name was not caught by a running test. See $log" }
        Write-Output "$Name CAUGHT"
        return
    }
    if ($code -ne 0) { throw "$Name failed. See $log" }
    $lines | Where-Object { $_ -match 'outside edits|kept their|in scope and exact|closed under|Frame {' } | Write-Output
    $passed = 0; $ignored = 0
    foreach ($m in [regex]::Matches(($lines -join "`n"), 'test result: ok\. (\d+) passed; \d+ failed; (\d+) ignored')) {
        $passed += [int]$m.Groups[1].Value; $ignored += [int]$m.Groups[2].Value
    }
    Write-Output "$Name PASS: passed=$passed ignored=$ignored"
}

# The source is restored in finally; a killed run can still leave a mutant, so check git diff after one.
function Mutant([string]$Name, [string]$File, [string]$Before, [string]$After, [string[]]$Arguments) {
    $path = Join-Path $engine $File
    $original = [IO.File]::ReadAllBytes($path)
    $source = [IO.File]::ReadAllText($path)
    if (!$source.Contains($Before)) { throw "Missing mutant site $Name" }
    try {
        [IO.File]::WriteAllText($path, $source.Replace($Before, $After))
        Cargo-Check $Name $Arguments -MustFail
    } finally { [IO.File]::WriteAllBytes($path, $original) }
}

Push-Location $engine
try {
    Cargo-Check 'workspace' @('test', '--offline', '--workspace', '--', '--nocapture')
    $query = @('test', '--offline', '-p', 'law', '--test', 'speed_differential', 'random_box_queries_equal_full_scan')
    Mutant 'collision-halo' 'law/src/world.rs' 'include(min,max,!self.blocks.unbounded)' 'include(min,max,true)' $query
    Mutant 'no-collision' 'law/src/world.rs' 'self.visit_collisions(bounds, context, |_| false)' 'true' $query
    Mutant 'suffocation' 'law/src/world.rs' '!self.visit_collision_cells(bounds,lo.map(|v|v-1),hi.map(|v|v+1), |p| {' 'false && !self.visit_collision_cells(bounds,lo.map(|v|v-1),hi.map(|v|v+1), |p| {' $query
    $hooks = @('test', '--offline', '-p', 'law', '--test', 'speed_differential', 'tick_equals_forced_fluid_and_inside_hooks')
    Mutant 'fluid-flag' 'law/src/world.rs' 'self.blocks.fluids || self.regions.iter().any(|(_,_,b)|b.fluid.kind!=FluidKind::Empty)' 'false' $hooks
    Mutant 'inside-flag' 'law/src/world.rs' 'self.blocks.inside_effects || self.regions.iter().any(|(_,_,b)|b.has_inside_effects())' 'false' $hooks
    Mutant 'single-box' 'law/src/shapes.rs' 'if self.shape.single {' 'if self.shape.single { return distance; } if self.shape.single {' @('test', '--offline', '-p', 'law', '--lib', 'single_box_collide_equals_general_path')
    Mutant 'missing-footprint' 'law/src/footprint.rs' '*slot |= bits;' 'let _ = (slot, bits);' @('test', '--offline', '-p', 'law', '--test', 'footprints', 'outside_footprint_changes_cannot_alter_tick')
    # The key: a facing that does not turn must lose the found rotations, on test blocks and on Mojang's stairs.
    Mutant 'unturned-facing' 'law/src/pattern.rs' 'fn facing(self, facing: u8) -> u8 {' 'fn facing(self, facing: u8) -> u8 { return facing;' @('test', '--offline', '--workspace', '--test', 'pattern')
    # A cell must be keyed by every answer its queries read: drop the fluid, then the sneak shape.
    $answers = @('test', '--offline', '-p', 'law', '--test', 'footprints', 'edits_that_keep_a_cells_answers_cannot_alter_tick')
    Mutant 'fluid-unkeyed' 'law/src/pattern.rs' 'groups & (FLUID | HOOKS) != 0' 'groups & HOOKS != 0' $answers
    Mutant 'sneak-unkeyed' 'law/src/pattern.rs' 'for shape in [shape, above, below, sneak, inside_cell] {' 'let _ = sneak; for shape in [shape, above, below, inside_cell] {' $answers
    Mutant 'inside-cell-unkeyed' 'law/src/pattern.rs' 'for shape in [shape, above, below, sneak, inside_cell] {' 'let _ = inside_cell; for shape in [shape, above, below, sneak] {' @('test', '--offline', '-p', 'world', '--test', 'blocks', 'inside_cell_shape_is_keyed_in_recorded_footprint')
    # A shift that leaves a binade must not be called exact.
    Mutant 'scope-unbounded' 'law/src/pattern.rs' '(0..3).all(|a| shift[a] == 0 ||' '(0..3).all(|a| true ||' @('test', '--offline', '-p', 'law', '--test', 'pattern', 'shifts_inside_the_binades_replay_bit_exact')
    # The fluid check's loaded reads at absolute chunk corners: a shifted key no longer sees them.
    Mutant 'chunk-corners' 'law/src/movement.rs' 'if !world.loaded([x, lo[1], z]) {' 'if !world.loaded([x>>4<<4, lo[1], z>>4<<4]) {' @('test', '--offline', '-p', 'law', '--test', 'pattern', 'a_shift_across_a_chunk_edge_cannot_hit_when_the_law_sees_it')
    Cargo-Check 'restored' @('test', '--offline', '--workspace', '--test', 'pattern', '--test', 'footprints', '--test', 'speed_differential')
    $hidden = Get-ChildItem (Join-Path $engine 'law/src') -File -Filter '*.rs' |
        Select-String -Pattern 'SystemTime|Instant::|std::time|thread_rng|rand::|\.next_(float|double|int)\('
    if ($hidden) { throw "Review new randomness or clock sites: $hidden" }
    Write-Output 'Randomness and wall-clock audit PASS: no draws or clock reads in the law library.'
} finally { Pop-Location }
