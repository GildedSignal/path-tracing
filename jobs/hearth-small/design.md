# The Last Hearth (Small) — Pass 1 Architectural Design

**Box:** `x` in `[400, 440]` (41), `y` in `[-61, -16]` (46), `z` in `[-20, 20]` (41)  
**Target World:** Minecraft Java 26.2 (Superflat overworld, ground top `y = -61`, standing `y = -60`)  
**Scale Target:** Pagoda-sized (~800–2,000 hand-written `/fill` and `/setblock` commands across Passes 2–5, zero generators)

---

## 1. The Site's Story

High on a narrow rocky spur above a steep ravine, an ancient dressed-stone road once carried travellers and pack animals eastward across a stone-arch bridge toward a distant valley. At the western bridgehead stood a compact stone toll post with a vaulted gatehouse arch spanning the carriageway.

Over time, two forces reshaped the crossing. First, the stone bridge collapsed into the chasm, leaving only its western and eastern masonry abutments staring across a six-block void above a shaded mountain pool. Second, a rockslide broke off the south-west crag and spilled coarse boulders and gravel across the outer edge of the approach road, forcing passers-by to wear a narrow bypass around the talus.

After the toll ceased to matter, the site still mattered as shelter before the drop. Someone enclosed the lee side of the stone tollhouse with a lower spruce-timber wing, tucked a covered porch out of the wind, opened a low wooden door away from the exposed bridgehead, and converted the old guardroom into a warm hearth room with a cooking corner, dry bunks, and a quiet two-block window seat looking east into the ravine. Later hands repaired storm damage on the main slate roof with newer oak and spruce timbers, stacked firewood under the north eaves, and placed an empty lectern and map table by the fire for whoever arrives next. Nobody is named, and nobody waits for thanks.

---

## 2. Reason for Each Mass

Every built volume and landform in the `41 × 46 × 41` box answers a physical cause:

1. **The Rocky Spur & Ravine (`x = 400..425` west spur; `x = 426..431` chasm; `x = 432..440` east cliff):**  
   Because superflat cannot be excavated below `-63`, the ravine floor sits at `y = -61..-60` (an enclosed stone basin holding cold mountain water at `y = -60`), while the spur climbs eastward from the western entry (`y = -61` top, `-60` feet at `x = 400`) to a flat headland terrace at `y = -53` top (`-52` feet at `x = 416..425`), dropping 7–8 blocks sheer into the chasm. Across the gap, the eastern cliff rises to `y = -43..-41` so the ravine reads as a true mountain gorge rather than a notch in a flat floor.
2. **The Old Road & Landslide Talus (`x = 400..425`, climbing from `z = +4` to `z = 0`; crag `z = +6..+16`):**  
   Retaining stonework holds the road against the south flank of the spur. A steep rocky crag on the south-west (`x = 408..417, z = +7..+16`, peaking at `y = -46`) sheds an apron of andesite and gravel debris across `x = 409..414, z = +1..+5`, explaining the narrowed detour path.
3. **The Surviving Stone Tollhouse Core & Gatehouse Arch (`x = 417..424, z = -10..-2`, arch bay `x = 417..419, z = -1..+2`):**  
   The oldest mass, anchored directly behind the West Bridge Abutment (`x = 420..425, z = -1..+2`) where a single toll-keeper could watch the crossing. Its heavy stone walls (`y = -52..-46`) carry the **dominant steep gable roof** (ridge running N–S along `x = 420..421`, peaking at `y = -40`), the tall **soot-darkened stone chimney** (`x = 419..421, z = -10`, rising to `y = -37`), and the projecting **East Alcove** (`x = 425, z = -8..-5`) cantilevered toward the chasm.
4. **The Later Spruce Extension & Covered Porch (`x = 410..416, z = -10..-4`; porch `z = -3..-2`):**  
   Grafted onto the sheltered west flank of the stone core, out of the chasm updraft. It carries a **smaller, lower cross-gable roof** (ridge running E–W along `z = -7`, peaking at `y = -44` against the stone core's west wall), creating a clear two-era silhouette. Its southern eaves project over a dry porch (`z = -3..-2`) with a bench and low timber door, while its north eaves shelter the stacked firewood and closed rainwater trough (`z = -12..-11`).
5. **Wind-Bent Alpine Spruces (`x = 404, z = -7`; `x = 414, z = -15`; `x = 437, z = -13`; `x = 437, z = +13`):**  
   Placed only in wind-sheltered pockets on the spur and high east ledge—never scattered evenly—partially screening the refuge on initial approach so the building is discovered gradually.

---

## 3. The Walk (7 Stations)

Zero jumps are required anywhere along the exterior road, detour, porch, or interior rooms (`0.5`-block slab steps on the approach climb; continuous `y = -52` feet across the terrace, bridgehead, and all interior rooms).

| # | Station & Coordinates (`x y z`, feet) | Aim (`yaw`, `pitch`, time) | What the Visitor Sees and Feels |
|---|---|---|---|
| **1** | **West Trailhead** (`400 -60 4`) | `yaw 250`, `pitch -8` (day & night) | Stepping off the flat plain onto worn road slabs beside a mossy retaining wall. Ahead, landslide boulders pinch the road; past a wind-bent spruce rises the dark slate gable of the stone core and, at night, one warm glowing window and porch lantern signalling shelter without revealing the east cliff. |
| **2** | **The Tollhouse Gatehouse Arch** (`415 -53 0`) | `yaw 270`, `pitch -4` (day) | Passing beneath the cold, three-block-wide stone archway (`y = -52..-49`) of the original road. Compressed stone walls frame the bright gap ahead—and the first clear sign that the bridge deck beyond stops short. |
| **3** | **The Broken Bridge Abutment** (`422 -52 0`) | `yaw 270`, `pitch 12` (day) | Standing on the surviving masonry abutment at the chasm lip (`x = 425`). Broken arch stones drop into empty air above the cold pool (`y = -60`); across the six-block gorge stand the matching eastern abutment (`x = 432..435`) and the abandoned trail climbing the far cliff. The crossing is dead; you must turn back. |
| **4** | **The Detour & Sheltered Porch** (`414 -52 -2`) | `yaw 195`, `pitch -4` (day) | Turning north off the road just before the arch onto dry terrace flagstones under projecting spruce eaves. A timber bench, hanging lantern, and **low two-block wooden door** (`x = 412, z = -4`) offer quiet human shelter in contrast to the cold toll arch beside it. |
| **5** | **The Low Vestibule** (`412 -52 -5.5`) | `yaw 265`, `pitch 0` (night) | Compression: a tight, three-block-high (`y = -52..-50`) timber vestibule (`x = 411..413, z = -6..-5`) with a gear bench, pack hook, and dry storage. Straight ahead through the thick stone threshold (`x = 417`), warm hearth light spills out from a much taller room. |
| **6** | **The Hearth Room & Kitchen Corner** (`419 -52 -4`) | `yaw 205`, `pitch -6` (night) | Release: a `6 × 7` hall rising into dark spruce rafters (`y = -43`). The stone hearth (`z = -10`) glows behind iron bars, incorporating one mossy dressed stone salvaged from the fallen bridge; beside it sit the smoker, cauldron, prep counter, map table, empty lectern, and a south window glancing back toward the broken bridge. |
| **7** | **The East Alcove & Window Seat** (`422.5 -52 -6.5`) | `yaw 270`, `pitch 4` (day) | The quiet reward held back until last: tucked into the thick east stone wall (`z = -7..-6`), a cushioned spruce window seat sits beneath a two-block-wide glazed casement (`x = 425, y = -51..-50`). Looking east, the chasm pool, the broken eastern abutment, and the sheer pine-crowned cliff open up in stillness. |

---

## 4. ASCII Plans (North = Up `-Z`, West = Left `-X`)

### A. Site Plan (`x = 400..440`, `z = -20..+20`, overview of terrain, road, ravine & masses)
```text
z
-20 . . . . . . . . . . . . . . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ c c c c c c c c c
-16 . . . . . . . . . . . . T . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ c c c c T c c c c
-12 . . . . . . . . . w B . . . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ c c c c c c c c c
-10 . . . . . . . . . +---------+ +-------+ . . . . . . ~ ~ ~ ~ ~ ~ c c c c c c c c c
 -8 . . . T . . . . . |         | |       | . . . . . . ~ ~ ~ ~ ~ ~ c c c r r r c c c
 -6 . . . . . . . . . | SPRUCE  |#| STONE |A] . . . . . ~ ~ ~ ~ ~ ~ c c r r c c c c c
 -4 . . . . . . . . . |  WING   | | CORE  | . . . . . . ~ ~ ~ ~ ~ ~ c r r c c c c c c
 -2 . . . . . . . . . +--D--+---+ +---W---+ . . . . . . ~ ~ ~ ~ ~ ~ c c c c c c c c c
  0 . . . . . . . . . . p p p p > > [===] = = = X . . . ~ ~ ~ ~ ~ ~ [===] r r r c c c
 +2 . . . . . . . > > > > > * * * * [===] = = = X . . . ~ ~ ~ ~ ~ ~ [===] c c c c c c
 +4 . > > > > > > . . * * * * . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ c c c c c c c c c
 +8 . . . . . . . . C C C * * . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ c c c c c c c c c
+12 . . . . . . . C C C C C . . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ c c c T c c c c c
+16 . . . . . . . . C C C . . . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ c c c c c c c c c
+20 . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . c c c c c c c c c
    400       405       410   413   417   421 424 426   428       432   435       440 x
```
*Legend: `>` = climbing old road, `*` = landslide talus from south crag `C`, `[===]` = stone toll arch & broken bridge abutments (`X` = sheer drop), `p` = sheltered porch/detour, `D` = low spruce door, `#` = stone threshold, `A]` = 2-block east alcove projection, `W` = south bridge-glance window, `w B` = firewood stack & closed rainwater basin, `~` = ravine floor & enclosed cold pool (`x=426..431`), `c` = east cliffside (`x=432..440`), `r` = abandoned east road, `T` = alpine spruce.*

### B. Main Level & Courtyard Plan (`y = -52` feet / `y = -51` eye slice, `x = 409..436`, `z = -13..+3`)
```text
      409 410 411 412 413 414 415 416 417 418 419 420 421 422 423 424 425 | 426..431 | 432 433 434 435
z=-12  .   w   w   .   B   B   B   .   .   .   .   .   .   .   .   .   .  |  ravine  |  c   c   c   c
z=-11  .   w   .   .   B   ~   B   .   .   .   S   C   S   .   .   .   .  |  below   |  c   c   c   c
z=-10  .   P   S   S   S   S   S   S   S   S   S   H   S   S   S   S   .  | (-61/-60)|  c   c   c   c
z= -9  .   S   b   b   d   b   b   S   S   K   K   f   C   .   .   S   .  |          |  c   c   c   c
z= -8  .   S   .   .   .   .   .   S   S   k   .   .   .   .   .   S   .  |          |  c   c   c   c
z= -7  .   S   P   P   P   .   .   S   S   .   .   t   t   s   a   A   .  |          |  c   c   c   c
z= -6  .   W   h   .   .   .   .   .   .   .   .   t   t   s   a   A   .  |          |  c   c   c   c
z= -5  .   S   n   .   .   .   .   .   .   .   .   .   .   .   .   S   .  |          |  c   c   c   c
z= -4  .   P   S   D   S   W   S   P   S   M   m   L   .   .   .   S   .  |          |  c   c   c   c
z= -3  .   .   .   .   n   n   .   .   S   .   .   .   .   .   .   S   .  |          |  c   c   c   c
z= -2  .   P   .   .   .   .   .   P   S   S   S   W   W   S   S   S   .  |          |  c   c   c   c
z= -1  .   .   .   .   .   .   .   .   P   .   .   P   =   =   =   =   .  |  chasm   |  =   =   =   c
z=  0  .   .   >   >   >   >   >   >   .   .   .   .   =   =   =   =   .  |  gap     |  =   =   =   r
z= +1  .   .   .   .   .   .   .   .   P   .   .   P   =   =   =   =   .  |          |  =   =   =   c
z= +2  .   .   .   .   .   .   .   .   S   S   S   S   .   .   .   .   .  |          |  c   c   c   c
```
*Legend: `P` = corner log post / stone pier, `S` = solid wall (`spruce_planks`/`logs` in wing `410..416`, stone masonry in core `417..424`), `D` = low spruce door `(412,-52,-4)`, `W` = glazed window (`410,-51,-6` & `414,-51,-4` warm landmark windows; `420..421,-51,-2` bridge-glance window), `h` = gear hook, `n` = bench, `b` = bunks, `d` = bookshelf divider, `H/f` = hearth firebox & stone fender (`420,-52,-10`), `K/k` = smoker/cauldron/counter/barrel kitchen corner, `t` = hearth table, `M/m` = tool bench & grindstone, `L` = empty lectern & map table, `s` = alcove step/entry, `a`/`A` = 2-block window seat `(424,-52,-7..-6)` & glazed casement `(425,-51,-7..-6)`, `=` = bridge abutment deck, `w` = woodpile, `B/~` = closed rainwater basin.*

### C. Upper / Roof Plan (`y = -46..-40`, `x = 409..426`, `z = -11..+3`)
```text
      409 410 411 412 413 414 415 416 417 418 419 420 421 422 423 424 425 426
z=-11  .   .   .   .   .   .   .   .   e   s   s   C   s   s   s   e   e   .   C = Chimney stack (y=-46..-37)
z=-10  e   e   e   e   e   e   e   .   s   /   /   C   \   \   \   s   e   .   / \ = Dominant N-S gable
z= -9  e   /   /   /   /   /   \   .   s   /   /   R   \   \   \   s   e   .   R = Main ridge (y=-40, x=420..421)
z= -8  e   /   /   /   /   /   \   .   s   /   /   R   \   \   \   s   e   .
z= -7  e   r   r   r   r   r   r   >   s   /   /   R   \   \   \   a   a   .   r = Lower E-W wing ridge (y=-44)
z= -6  e   \   \   \   \   \   /   .   s   /   /   R   \   \   \   a   a   .   a = Alcove shed dormer roof
z= -5  e   \   \   \   \   \   /   .   s   /   /   R   \   \   \   s   e   .
z= -4  e   e   e   e   e   e   e   .   s   /   /   R   \   \   \   s   e   .
z= -3  e   p   p   p   p   p   p   .   s   /   /   R   \   \   \   s   e   .   p = Porch lean-to roof (y=-48)
z= -2  e   e   e   e   e   e   e   .   e   e   e   e   e   e   e   e   e   .
z= -1  .   .   .   .   .   .   .   .   A   A   A   A   .   .   .   .   .   .   A = Lower stone parapet over
z= +1  .   .   .   .   .   .   .   .   A   A   A   A   .   .   .   .   .   .       Toll Arch (y=-48..-47)
```

---

## 5. Two ASCII Sections

### Section 1: East–West Cut at `z = -6` (Looking North, `x = 400..440`, `y = -61..-36`)
```text
y
-37                              C          <- Stone chimney top + soot cap (x=420, z=-10 proj.)
-40                           /=====\       <- Main N-S deepslate ridge (y=-40)
-42                         /         \     <- Exposed interior timber rafters (y=-43)
-44             r r r r r r/           \    <- Lower Spruce Wing ridge (y=-44) meets stone wall
-46           e/===========|           |    <- Eaves & upper stone wall (x=417..424)
-48          |   spruce    |  HEARTH   |d   <- Alcove dormer roof (d) at x=425
-50          |  [=======]  #   ROOM    [A]  <- Vestibule low ceiling (-50), stone threshold (#), 2-wide East Alcove window [A]
-52          |n__vestibule_|___table__s[a]  <- Floor (-53, feet -52); seat [a] at x=424  |         c c c c c
-54     spruce wing & spur plateau          |                                           |       c c c c c c
-56       rocky spur terrain                |        RAVINE CHASM (6 wide)              |     c c c c c c c
-58                                         |                                           |   c c c c c c c c
-60  [road starts y=-60]                    |_____________~ ~ ~ ~ ~ ~___________________| c c c c c c c c c
-61  ======================================================================================================
    400       406         412         417   420       424 426       431                 432     436     440 x
```

### Section 2: North–South Cut at `x = 420` (Looking West, `z = -18..+18`, `y = -61..-36`)
```text
y
-37                C                        <- Chimney flue & soot-stained stone cap (z=-10)
-40                R R R R R R R R R        <- Dominant roof ridge (x=420..421, z=-10..-2)
-43               /|               |\       <- High open rafters over Hearth Room
-46              | |   HEARTH      | |      <- Heavy stone gable walls
-48              | |    ROOM       | |[===] <- Gatehouse Toll Arch vaulted soffit (z=-1..+1)
-50              |H|     t         |W||   | <- Firebox (H), hearth table (t), South Bridge-Glance Window (W), cold Toll Arch
-52         w B  |_|_____t_________|_||___| <- North woodpile/basin (w B), Stone Core floor (-52), Toll Road under arch (-52)
-54        ========================================\          * * *         /==============================
-56       /   north rocky spur slope                \       * * talus *    /   south crag flank
-58      /                                           \____________________/
-61  ======================================================================================================
    -18    -14  -11       -6        -2 -1 +1 +2       +6       +10       +14       +18 z
```

---

## 6. Palette (Every Block Verified Against `engine/world/data/blocks.tsv`)

| Role | Verified 26.2 Block IDs & States | Architectural Reason |
|---|---|---|
| **Spur, Ravine & Crag** | `stone`, `andesite`, `tuff`, `cobbled_deepslate`, `andesite_stairs`, `andesite_slab`, `andesite_wall`, `tuff_stairs`, `tuff_slab`, `tuff_wall`, `gravel`, `coarse_dirt`, `grass_block`, `podzol`, `moss_block`, `moss_carpet`, `leaf_litter` | Tuff and cobbled deepslate anchor wet shaded ravine walls; stone and andesite form exposed dry spur rock; gravel and coarse dirt mark the landslide talus; moss collects only in damp shade. |
| **Alpine Flora & Water** | `spruce_log` (`axis`), `spruce_wood`, `spruce_leaves[persistent=true]`, `fern`, `short_grass`, `short_dry_grass`, `azalea`, `vine`, `water[level=0]` | Wind-swept spruces in sheltered pockets; ferns and vines on damp ravine ledges; water strictly in closed stone basins (ravine pool & north rainwater trough). |
| **Tollhouse Stone Core, Road & Bridge** | `stone_bricks`, `cracked_stone_bricks`, `mossy_stone_bricks`, `chiseled_stone_bricks`, `tuff_bricks`, `polished_tuff`, `cobblestone`, `mossy_cobblestone`, `stone_brick_stairs`, `stone_brick_slab`, `stone_brick_wall`, `mossy_stone_brick_stairs`, `mossy_stone_brick_slab`, `mossy_stone_brick_wall`, `stone_stairs`, `stone_slab`, `cobblestone_wall` | Dressed stone bricks distinguish old engineered masonry (toll core, gatehouse arch, bridge abutments, retaining walls) from cut hillside tuff; cracked/mossy variants appear only at fracture points and damp shaded bases. |
| **Later Spruce Extension & Carpentry** | `spruce_log`, `stripped_spruce_log`, `stripped_spruce_wood`, `spruce_planks`, `spruce_stairs`, `spruce_slab`, `spruce_fence`, `spruce_trapdoor`, `spruce_door` (both halves), `oak_planks`, `oak_stairs`, `oak_slab`, `dark_oak_planks`, `dark_oak_stairs`, `dark_oak_slab`, `dark_oak_trapdoor` | Local spruce forms the newer enclosed wing, porch posts, low door, bunks, and alcove lining; worn oak slabs mark repaired thresholds and tread edges; dark oak forms heavy interior beams and hearth furniture. |
| **Roofs & Chimney Soot** | `deepslate_tiles`, `deepslate_tile_stairs`, `deepslate_tile_slab`, `polished_basalt`, `cobbled_deepslate_wall` | Deep slate tiles give the dominant gable crisp shadow lines against alpine stone; spruce stairs/slabs mark the smaller wing roof and a visibly newer patch repair on the main slope; basalt and cobbled deepslate show soot near the chimney crown. |
| **Windows, Hearth & Traces** | `light_gray_stained_glass_pane`, `glass_pane`, `lantern[hanging=true/false]`, `exposed_copper_lantern[hanging=true/false]`, `ochre_froglight`, `magma_block`, `iron_bars`, `iron_chain`, `smoker`, `crafting_table`, `cartography_table`, `grindstone`, `cauldron`, `barrel`, `chest`, `lectern`, `bookshelf`, `brown_bed`, `white_bed`, `tripwire_hook`, `flower_pot`, `potted_fern`, `potted_spruce_sapling`, `brown_carpet`, `gray_carpet` | Fire-free hearth glow (`ochre_froglight` + `magma_block` + low `lantern` behind `iron_bars`); weathered copper lantern on the exterior porch; empty containers and lectern for travellers' traces; map table and tools for upkeep. |

---

## 7. Views to Render (`views.txt`)

- **Overviews & Cuts:**
  - `iso NE day`, `iso SW day`, `iso SW night`
  - `plan top`, `plan -51` (eye-height cut slicing doors, windows, vestibule, hearth, alcove, and toll arch)
  - `elev N`, `elev W`
  - `section x 420` (N–S through chimney, hearth, south bridge window, and toll arch)
  - `section z -6` (E–W through spruce wing, vestibule, stone threshold, hearth, east alcove window seat, chasm, and far cliff)
- **Station Eye Views (`eye <x> <y> <z> <yaw> <pitch> <fov> <day|night> <name>`):**
  - `eye 400 -60 4 250 -8 75 day s1-approach-day`
  - `eye 400 -60 4 250 -8 75 night s1-approach-night`
  - `eye 415 -53 0 270 -4 75 day s2-arch`
  - `eye 422 -52 0 270 12 75 day s3-bridge`
  - `eye 414 -52 -2 195 -4 75 day s4-porch`
  - `eye 412 -52 -5.5 265 0 80 night s5-vestibule`
  - `eye 419 -52 -4 205 -6 80 night s6-hearth`
  - `eye 422.5 -52 -6.5 270 4 70 day s7-alcove`
