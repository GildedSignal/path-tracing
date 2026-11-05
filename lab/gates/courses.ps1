# Data for L0 on the lab superflat. All coordinates are absolute and all fill volumes are <= 32768.
# Patches are 128 blocks apart. A solid floor, 15-block walls and roof enclose each 32x32 patch,
# including ladders and slime jumps, so any length walk stays inside its own course.
function Get-FixtureCourses {
  $names = @('full-blocks', 'slabs-stairs', 'step-ups', 'ceilings', 'ladders-vines', 'water',
    'ice', 'slime', 'soul-sand', 'honey', 'cobweb', 'powder-snow', 'edges')
  for ($index = 0; $index -lt $names.Count; $index++) {
    $name = $names[$index]
    $x = 4096 + 128 * $index
    $z = 4096
    $commands = [Collections.Generic.List[string]]::new()
    # Local construction helpers keep the actual course definitions readable.
    function Fill([int]$X0, [int]$Y0, [int]$Z0, [int]$X1, [int]$Y1, [int]$Z1, [string]$Block) {
      $commands.Add(('fill {0} {1} {2} {3} {4} {5} {6}' -f ($x+$X0), $Y0, ($z+$Z0), ($x+$X1), $Y1, ($z+$Z1), $Block))
    }
    Fill 0 -64 0 31 -45 31 air
    Fill 0 -64 0 31 -61 31 stone
    Fill 0 -60 0 0 -46 31 stone
    Fill 31 -60 0 31 -46 31 stone
    Fill 1 -60 0 30 -46 0 stone
    Fill 1 -60 31 30 -46 31 stone
    Fill 0 -45 0 31 -45 31 stone
    $startY = -60
    switch ($name) {
      'full-blocks' {
        Fill 1 -61 6 30 -61 30 grass_block
        Fill 8 -61 6 14 -61 30 dirt
        Fill 20 -61 6 26 -61 30 cobblestone
      }
      'slabs-stairs' {
        for ($row = 6; $row -le 27; $row += 4) {
          Fill 1 -60 $row 30 -60 ($row+1) 'stone_slab[type=bottom]'
          Fill 1 -60 ($row+2) 30 -60 ($row+2) 'stone_stairs[facing=south,half=bottom]'
        }
      }
      'step-ups' {
        for ($column = 2; $column -le 26; $column += 6) {
          Fill $column -60 6 ($column+1) -60 30 'stone_slab[type=bottom]'
          Fill ($column+2) -60 6 ($column+3) -60 30 stone
        }
      }
      'ceilings' { Fill 1 -58 6 30 -58 30 stone }
      'ladders-vines' {
        for ($row = 6; $row -le 26; $row += 5) {
          Fill 1 -60 $row 30 -51 $row stone
          Fill 1 -60 ($row-1) 15 -51 ($row-1) 'ladder[facing=north]'
          Fill 16 -60 ($row-1) 30 -51 ($row-1) 'vine[south=true]'
        }
      }
      'water' {
        # Two-deep source pool, a dry divider, and a one-deep source lip flowing south.
        Fill 1 -62 6 14 -62 30 stone
        Fill 1 -61 6 14 -60 30 water
        Fill 15 -60 6 16 -60 30 stone
        Fill 17 -60 6 30 -60 6 'water[level=0]'
      }
      'ice' {
        Fill 1 -61 6 15 -61 30 ice
        Fill 16 -61 6 30 -61 30 blue_ice
      }
      'slime' { Fill 1 -61 6 30 -61 30 slime_block }
      'soul-sand' { Fill 1 -61 6 30 -61 30 soul_sand }
      'honey' {
        Fill 1 -61 6 30 -61 30 honey_block
        for ($column = 6; $column -le 24; $column += 6) {
          Fill $column -60 8 ($column+1) -57 28 honey_block
          Fill ($column-1) -60 8 ($column-1) -60 28 'stone_slab[type=bottom]'
        }
      }
      'cobweb' { Fill 1 -60 6 30 -59 30 cobweb }
      'powder-snow' { Fill 1 -60 6 30 -60 30 powder_snow }
      'edges' {
        # Safe catch floor 3 blocks below a raised grid. Sneaking encounters edges in both axes.
        $startY = -57
        Fill 12 -58 1 20 -58 5 stone
        for ($column = 2; $column -le 29; $column += 3) { Fill $column -58 6 $column -58 30 stone }
        for ($row = 6; $row -le 30; $row += 3) { Fill 1 -58 $row 30 -58 $row stone }
      }
    }
    # Last block is a receipt marker, inspected in the client's readiness export.
    $commands.Add(('setblock {0} -45 {1} gold_block' -f ($x+16), ($z+16)))
    [pscustomobject]@{
      Name = $name
      Start = '{0},{1},{2},0,0' -f ($x+16.5).ToString([Globalization.CultureInfo]::InvariantCulture), $startY,
        ($z+4.5).ToString([Globalization.CultureInfo]::InvariantCulture)
      Min = @($x, -64, $z)
      Max = @(($x+31), -45, ($z+31))
      Marker = @(($x+16), -45, ($z+16))
      Commands = $commands.ToArray()
    }
  }
}
