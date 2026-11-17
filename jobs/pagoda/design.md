# Classical 5-Storey Pagoda — Architectural Design

## 1. Overview & Dimensions
- **Box:** `x` in `[20, 52]` (33 blocks), `y` in `[-60, -10]` (51 blocks), `z` in `[-16, 16]` (33 blocks).
- **Pagoda Central Axis (`Shinbashira`):** `(x = 38, z = 0)`, shifted 2 blocks east of box center `(36, 0)` to give an 8-block-deep front Zen courtyard (`x = 20..27`) facing spawn `(0, -60, 0)` while leaving 3–5 blocks of garden margin on the north, south, and east sides.
- **Total Height:** `y = -60` (ground approach & plinth tier 1) to `y = -10` (finial spire tip), spanning all 51 vertical blocks.
- **Walkability:** Zero jumps required from spawn `(0, -60, 0)` through the Torii arch (`x = 20`), over the arched bridge (`x = 23..26`), up the 3-tier stone plinth (`y = -60..-58`), and up the continuous interior spiral staircase (`Storey 1 -> Storey 5`) out onto the wrap-around top balcony (`y = -32`).

### Storey Schedule (Square Tapered Plan)
| Part | Floor `y` | Feet `y` | Wall Outer Size (`r_w`) | Walkable Balcony (`r`) | Roof Eave Outer (`r`) | Headroom |
|---|---:|---:|---|---|---|---:|
| **Plinth (3 tiers)** | `-60..-58` | `-59..-57` | `19x19 -> 17x17 -> 15x15` | `r = 7` open veranda | — | — |
| **Storey 1** | `-58` | `-57` | `13x13` (`r = 6`) | `15x15` stone veranda (`r = 7`) | `21x21` (`r = 10`, flared) | 5 blocks |
| **Storey 2** | `-51` | `-50` | `11x11` (`r = 5`) | `13x13` (`r = 6`, railing `r = 7`) | `17x17` (`r = 8`, flared) | 4 blocks |
| **Storey 3** | `-45` | `-44` | `9x9` (`r = 4`) | `11x11` (`r = 5`, railing `r = 6`) | `15x15` (`r = 7`, flared) | 4 blocks |
| **Storey 4** | `-39` | `-38` | `7x7` (`r = 3`) | `9x9` (`r = 4`, railing `r = 5`) | `13x13` (`r = 6`, flared) | 4 blocks |
| **Storey 5** | `-33` | `-32` | `5x5` (`r = 2`) | `7x7` (`r = 3`, railing `r = 4`) | `11x11` (`r = 5`, flared) + cap | 4 blocks |
| **Sorin Spire** | `-25` base | — | `3x3` roban -> `1x1` shaft | — | `-25..-10` (16 blocks) | — |

---

## 2. Palette & Material Rationale (Daylight Coherence)
Every material is chosen for crisp contrast in bright Minecraft daylight without relying on flat full-cube walls:
- **Foundation & Plinth (`stone_bricks`, `polished_blackstone_bricks`, `chiseled_polished_blackstone`, `gilded_blackstone`):** A cool-slate stone base anchors the warm timber above; blackstone trims and gilded landing inlays mark the cardinal thresholds.
- **Timber Columns & Heart Pillar (`stripped_crimson_hyphae`, `stripped_dark_oak_wood`, `dark_oak_wood`):** Vermilion-lacquered crimson columns (`stripped_crimson_hyphae`) frame every corner and portal bay against deep chocolate-ebony dark oak lintels and brackets (`dark_oak_wood`, `dark_oak_stairs[half=top]`).
- **Plaster Panels & Lattice (`calcite`, `crimson_fence`, `crimson_trapdoor`):** Warm textured cream-white `calcite` infill reads like traditional lime plaster (`shikkui`) rather than harsh flat white concrete; `crimson_fence` forms airy lattice windows and `crimson_trapdoor[open=true]` creates thin carved balcony balustrades that keep the full 1-block balcony walkway unobstructed.
- **Roofs & Upturned Eaves (`deepslate_tiles`, `deepslate_tile_stairs`, `deepslate_tile_slab`, `waxed_cut_copper_slab`):** Deep blue-charcoal slate tiles curve down via bottom stairs (`r_w+1`, `r_w+2`) to thin `top`/`bottom` eave slabs that sweep upward at the flared corners, capped with oxidized-proof `waxed_cut_copper_slab` corner crests and hanging `lantern`s.
- **Finial Spire (`polished_blackstone_bricks`, `gold_block`, `waxed_chiseled_copper`, `waxed_copper_bars`, `waxed_lightning_rod`):** Traditional *roban* base, *fukubachi* gold bowl, *kurin* alternating copper-bar/gold rings, and *hoshu* gold jewel + lightning-rod tip reaching `y = -10`.
- **Courtyard & Zen Garden (`smooth_stone_slab`, `mossy_stone_bricks`, `water`, `sea_pickle`, `cherry_log`, `cherry_leaves[persistent=true]`, `moss_block`, `andesite_wall`, `shroomlight`):** A vermilion Torii gate at `x = 20` frames an arched dark-oak bridge over a sunken glowing pond, flanked by symmetric cherry blossom trees, flowering azaleas, and *toro* stone lanterns.

---

## 3. ASCII Side Elevation (Looking East from Spawn, Cross-Section at `z = 0` / Silhouette)

```text
y
-10                     |          <- waxed_lightning_rod[facing=up]
-11                     G          <- gold_block (hoshu jewel)
-12                     |          <- waxed_lightning_rod
-13                     #          <- waxed_copper_bars
-14                     |          <- waxed_lightning_rod
-15..23                 #G#C#G#C   <- kurin alternating copper bars / gold / chiseled copper
-24                    cGc         <- fukubachi gold bowl & copper slabs
-25                    sBs         <- roban blackstone base
-26                   /###\        <- Storey 5 cap (r=1) + copper corner crests
-27                  /#####\       <- r=2
-28        c        /#######\        c   <- r=3 + upturned wingtip crests (r=5)
-29        T=======/=========\=======T   <- r=4 stairs + upturned eaves & corner lanterns
-30        L    d[  |  *  |  ]d    L     <- brackets + Storey 5 lintel + hanging corner lanterns
-31             |[  |     |  ]|          <- crimson_fence windows + East balcony door
-32             |[  |  S  |  ]|          <- Storey 5 top balcony (r=3) & trapdoor railing (r=4)
-33       c=====/=============\=====c    <- Floor 5 (-33) + Storey 4 roof & upturned tips (r=6)
-34       T====/===============\====T    <- brackets + Storey 4 roof Ring 2 (r=5)
-35       L  d[ |   |  *  |   | ]d  L    <- Storey 4 lintel & corner lanterns
-36           | |   |     |   | |        <- Storey 4 windows & West/N/S doors
-37           | |   |  S  |   | |
-38           |_[___|_____|___]_|        <- Storey 4 balcony (r=4) & railing (r=5)
-39      c=====/=================\=====c   <- Floor 4 (-39) + Storey 3 roof & upturned tips (r=7)
-40      T====/===================\====T   <- Storey 3 roof Ring 2 (r=6)
-41      L  d[ |   |   *   |   | ]d  L     <- Storey 3 frieze & corner lanterns
-42          | |   |       |   | |
-43          | |   |   S   |   | |         <- Storey 3 windows & doors
-44          |_[___|_______|___]_|         <- Storey 3 balcony (r=5) & railing (r=6)
-45     c=====/=====================\=====c  <- Floor 3 (-45) + Storey 2 roof & tips (r=8)
-46     T====/=======================\====T  <- Storey 2 roof Ring 2 (r=7)
-47     L  d[ |    |   *   |    | ]d  L      <- Storey 2 frieze & corner lanterns
-48         | |    |       |    | |
-49         | |    |   S   |    | |          <- Storey 2 windows & doors
-50         |_[____|_______|____]_|          <- Storey 2 balcony (r=6) & railing (r=7)
-51    c=====/=========================\=====c  <- Floor 2 (-51) + Storey 1 roof & tips (r=10)
-52    T====/===========================\====T  <- Storey 1 roof Ring 2 (r=8) & Ring 3 (r=9)
-53    L  dd[ |    |   *   |    | ]dd  L        <- Storey 1 frieze, double brackets & lanterns
-54   (Torii) |    |       |    | |             <- Torii cap (-54), Storey 1 lintel (-54)
-55    ====   |    |       |    | |             <- Torii beam (-55), Storey 1 windows (-55..-56)
-56    |  |   |    |   S   |    | |             <- Storey 1 3-wide ceremonial portals (-55..-57)
-57    |  |   |____|_______|____|_|  (Toro)     <- Storey 1 floor (-58, feet -57)
-58    |  |  /=====================\  L         <- Plinth Tier 3 (-58) + Bridge railings (-59)
-59    |  | /=======================\           <- Plinth Tier 2 (-59) + Bridge deck (-59)
-60   _[__]/=========================\_         <- Approach (-60), Pond (-60), Plinth Tier 1 (-60)
     x=20  23..26  29..31 32..44 45..47 48..52
```
*Legend: `*` = shinbashira core, `S` = spiral stairwell, `[ ]` = balcony railings, `T/=` = deepslate roof, `c` = copper wingtip crest, `L` = hanging lantern, `d` = dark oak under-eave brackets.*

---

## 4. ASCII Plans (North = Up `-Z`, West = Left `-X` toward Spawn)

### Courtyard & Ground Level (`y = -60..-58`, `x = 20..52, z = -16..16`)
```text
z
-16 . m m m m m m m m m g g g g g g g g g g g g g g g g g g m m m m m
-13 . m m C C C m m m . g g g g L g g g g T g g g g g g g g m B B B m   C = NW/SW Cherry Tree (24,±13)
-10 . m m m m m m m m m g g g g g g g g g g g g g g g g g g m m m m m   B = NE/SE Bonsai (49,±13)
 -9 . . M M M M M M . . [========= Plinth Tier 1 (r=9) =========] m m   M = Mossy pond rim
 -6 . . M ~ ~ ~ ~ M T . [=== Tier 2 ===][=== Tier 3 ===] . . . T m m   ~ = Zen Pond water
 -2 . H M w w w w M . . [=][========= Storey 1 =========][=] . . m m   H = Torii gate (x=20)
  0 . = = = B B B = = > > >       carpet -> *  Altar     < < < = = m   w = Bridge stringer / B = deck
 +2 . H M w w w w M . . [=][========= Storey 1 =========][=] . . m m   T = Toro stone lantern
 +6 . . M ~ ~ ~ ~ M T . [=== Tier 2 ===][=== Tier 3 ===] . . . T m m
 +9 . . M M M M M M . . [========= Plinth Tier 1 (r=9) =========] m m
+13 . m m C C C m m m . g g g g L g g g g T g g g g g g g g m B B B m
+16 . m m m m m m m m m g g g g g g g g g g g g g g g g g g m m m m m
     20 22  24  27 28 29 31 32        38        44 45 47 48 49   52  x
```

### Storey 1 Plan (`y = -57`, walls `13x13` `r = 6`, veranda `15x15` `r = 7`)
```text
      31 32 33 34 35 36 37 38 39 40 41 42 43 44 45  x
z=-7   V  V  V  V  V  V  S  S  S  V  V  V  V  V  V   V = open stone veranda
z=-6   V  C  #  W  #  C  .  .  .  C  #  W  #  C  V   C = stripped_crimson_hyphae column
z=-5   V  #  B  .  .  .  .  .  .  .  .  .  B  #  V   # = calcite wall / W = crimson_fence window
z=-4   V  W  .  .  .  .  .  .  .  .  .  .  .  W  V   B = bookshelf
z=-3   V  #  .  .  .  .  .  .  .  .  .  .  .  #  V   S = 3-wide entrance portal
z=-2   V  C  .  .  .  .  .  .  .  .  .  .  .  C  V
z=-1   V  .  .  .  .  .  .  .  .  u  .  p  .  .  V   u = Stair 1 start (40,-57,-1)
z= 0   V  .  r  r  r  r  .  *  .  ^  .  A  .  .  V   r = red_carpet, * = shinbashira, A = bell altar
z=+1   V  .  .  .  .  .  ^  .  .  ^  .  p  .  .  V   p = potted bamboo / cherry sapling
z=+2   V  C  .  .  .  .  L  <  <  L  .  .  .  C  V   L = planks landing, < = stair west
z=+3..5V  #..W..# (symmetric)                #  V
z=+6   V  C  #  W  #  C  .  .  .  C  #  W  #  C  V
z=+7   V  V  V  V  V  V  S  S  S  V  V  V  V  V  V
```

### Storey 2 Plan (`y = -50`, walls `11x11` `r = 5`, balcony `r = 6`, railing `r = 7`)
```text
      31 32 33 34 35 36 37 38 39 40 41 42 43 44 45  x
z=-7   .  [  [  [  [  [  [  [  [  [  [  [  [  [  .   [ = crimson_trapdoor railing (r=7)
z=-6   [  b  b  b  b  b  b  b  b  b  b  b  b  b  [   b = walkable dark_oak_planks balcony (r=6)
z=-5   [  b  C  #  W  C  #  .  #  C  W  #  C  b  [   . at cardinals = open balcony doorway
z=-4   [  b  #  .  .  .  .  .  .  .  .  .  #  b  [
z=-2   [  b  C  .  .  .  .  .  .  .  .  .  C  b  [
z=-1   [  b  #  .  .  .  a  .  .  u  .  .  #  b  [   a = arrival (37,-51,-1), u = stair 2 start
z= 0   [  b  .  .  .  .  s  *  .  ^  .  .  .  b  [   s = top stair from S1 (37,-51,0)
z=+1   [  b  #  .  .  f  O  f  f  ^  .  .  #  b  [   O = stairwell opening, f = dark_oak_fence guard
z=+2   [  b  C  .  .  f  O  O  O  L  .  .  C  b  [
z=+5   [  b  C  #  W  C  #  .  #  C  W  #  C  b  [
z=+6   [  b  b  b  b  b  b  b  b  b  b  b  b  b  [
z=+7   .  [  [  [  [  [  [  [  [  [  [  [  [  [  .
```

### Storey 3 Plan (`y = -44`, walls `9x9` `r = 4`, balcony `r = 5`, railing `r = 6`)
```text
      32 33 34 35 36 37 38 39 40 41 42 43 44  x
z=-6   .  [  [  [  [  [  [  [  [  [  [  [  .
z=-5   [  b  b  b  b  b  b  b  b  b  b  b  [
z=-4   [  b  C  W  C  #  .  #  C  W  C  b  [
z=-2   [  b  C  .  .  .  .  .  .  .  C  b  [
z=-1   [  b  #  .  .  .  .  .  u  .  #  b  [   u = stair 3 start (40,-44,-1)
z= 0   [  b  .  .  .  a  *  .  ^  .  .  b  [   a = arrival (37,-45,0)
z=+1   [  b  #  .  .  s  f  f  O  .  #  b  [   s = top stair from S2 (37,-45,1), O = hole
z=+2   [  b  C  .  f  O  O  O  O  .  C  b  [
z=+4   [  b  C  W  C  #  .  #  C  W  C  b  [
z=+5   [  b  b  b  b  b  b  b  b  b  b  b  [
z=+6   .  [  [  [  [  [  [  [  [  [  [  [  .
```

### Storey 4 Plan (`y = -38`, walls `7x7` `r = 3`, balcony `r = 4`, railing `r = 5`)
```text
      33 34 35 36 37 38 39 40 41 42 43  x
z=-5   .  [  [  [  [  [  [  [  [  [  .
z=-4   [  b  b  b  b  b  b  b  b  b  [
z=-3   [  b  C  #  W  .  W  #  C  b  [   N/S/W open doors; E is lattice window
z=-2   [  b  #  .  .  .  .  .  #  b  [
z=-1   [  b  W  .  .  .  .  u  W  b  [   u = stair 4 start (40,-38,-1)
z= 0   [  b  .  .  a  *  .  ^  W  b  [   a = arrival (37,-39,0)
z=+1   [  b  W  .  s  f  f  L  W  b  [   s = top stair from S3 (37,-39,1)
z=+2   [  b  #  f  O  O  O  O  #  b  [
z=+3   [  b  C  #  W  .  W  #  C  b  [
z=+4   [  b  b  b  b  b  b  b  b  b  [
z=+5   .  [  [  [  [  [  [  [  [  [  .
```

### Storey 5 Plan (`y = -32`, walls `5x5` `r = 2`, top balcony `r = 3`, railing `r = 4`)
```text
      34 35 36 37 38 39 40 41 42  x
z=-4   .  [  [  [  [  [  [  [  .   [ = crimson_trapdoor railing (r=4)
z=-3   [  b  b  b  b  b  b  b  [   b = walkable top balcony (r=3)
z=-2   [  b  C  W  W  W  C  b  [   W = crimson_fence window over calcite
z=-1   [  b  W  O  s  a  W  b  [   s = top stair from S4 (38,-33,-1), a = arrival (39,-33,-1)
z= 0   [  b  W  O  *  F  .  b  [   F = solid floor (39,-33,0) -> East door (40,-32,0)
z=+1   [  b  W  O  O  O  W  b  [   O = stairwell opening
z=+2   [  b  C  W  W  W  C  b  [
z=+3   [  b  b  b  b  b  b  b  [
z=+4   .  [  [  [  [  [  [  [  .
```

---

## 5. Self-Check & Audit
1. **Block ID & State Verification against `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv`:**
   - All 53 distinct block IDs used in `build.txt` were checked directly against `blocks.tsv`:
     `air`, `allium`, `andesite_wall`, `azalea`, `bell`, `bookshelf`, `calcite`, `cherry_leaves`, `cherry_log`, `chiseled_polished_blackstone`, `crimson_fence`, `crimson_trapdoor`, `dark_oak_fence`, `dark_oak_planks`, `dark_oak_slab`, `dark_oak_stairs`, `dark_oak_wood`, `deepslate_tile_slab`, `deepslate_tile_stairs`, `deepslate_tiles`, `fern`, `flowering_azalea`, `gilded_blackstone`, `gold_block`, `grass_block`, `lantern`, `lily_of_the_valley`, `moss_block`, `moss_carpet`, `mossy_stone_bricks`, `polished_andesite`, `polished_blackstone_brick_slab`, `polished_blackstone_brick_stairs`, `polished_blackstone_bricks`, `poppy`, `potted_bamboo`, `potted_cherry_sapling`, `red_carpet`, `sea_pickle`, `short_grass`, `shroomlight`, `smooth_stone_slab`, `spruce_planks`, `stone_brick_slab`, `stone_brick_stairs`, `stone_bricks`, `stripped_crimson_hyphae`, `stripped_dark_oak_wood`, `water`, `waxed_chiseled_copper`, `waxed_copper_bars`, `waxed_cut_copper_slab`, `waxed_lightning_rod`.
   - Explicit valid states used for all directional/height/attachment blocks: `stairs` (`facing`, `half`, `shape`), `slabs` (`type`), `trapdoors` (`facing`, `half`, `open`), `lanterns` (`hanging`), `leaves` (`persistent=true`), `sea_pickle` (`pickles`, `waterlogged`), `bell` (`attachment`, `facing`), `waxed_lightning_rod` (`facing=up`), `water` (`level=0`).
2. **Coordinate Bounds:** Every `/fill` and `/setblock` coordinate satisfies `20 <= x <= 52`, `-60 <= y <= -10`, `-16 <= z <= 16`.
3. **`/fill` Limit:** Maximum single `/fill` volume is `11 x 5 x 11 = 605` blocks (Storey 1 interior air carve) and `19 x 1 x 19 = 361` blocks (Plinth Tier 1), well below the `32,768` command limit.
4. **Elevation Audit Fixes Applied:**
   - Removed duplicate Torii column `/fill` command.
   - Fixed Storey 4 -> Storey 5 stairwell opening so landing `(39, -33, -1)` stays solid `spruce_planks` while `(38, -33, 1)` is cleared for >= 3 blocks of headroom.
   - Replaced a floating fence post `(37, -50, 1)` over the Storey 2 stairwell opening with solid-supported guard fences `(36, -50, 1)` and `(36, -50, 2)`.
   - Verified every hanging lantern has a solid block directly above it, every standing lantern/fence has a solid block below it, and every balcony has a full 1-block walkable deck plus >= 4 blocks of headroom.

---

## 6. Command & Block Counts
- **`/fill` commands:** `319` (`7,473` block placements)
- **`/setblock` commands:** `478` (`478` block placements)
- **Total commands:** `797`
- **Total block placements executed:** `7,951` (`6,603` non-air block placements + `1,348` `minecraft:air` carve placements inside shells, portals, stairwells, and tree corners)
