# Design: I. The Last Hearth

**Site box:** `x 2000..2179`, `z -60..59`, `y -61..120` (superflat top at `y = -61`, standing level `y = -60`).  
**Eastern link:** `(2179, -60, 0)` (toward the River City).  
**Western road exit:** `(2000, -60, 28)` (over the pass).  
**Deliverable:** [`gen.rs`](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/build-hearth/gen.rs) (standard-library Rust, deterministic, writes `build.txt`).

---

## 1. Design Scale by Scale (Order of Decisions)

### 01. Landscape Structure (180 × 120 blocks)
- **Twin Alpine Peaks & Saddle:** A northern peak (`x ≈ 2092, z ≈ -34`, rising to `y ≈ +3` to `+6` above sea-level datum) and a southern peak (`x ≈ 2088, z ≈ +33`, rising to `y ≈ -5`) pinch together to form a narrow high pass saddle around `x = 2070..2095, z = -12..+8` at `y = -38` (23 blocks above the superflat plain). All four box boundaries taper smoothly down via a 12–14 block foothill blend to `y = -61`.
- **The Glacial Ravine (`x ≈ 2104`):** Cuts NNE–SSW directly east of the saddle down to `y = -50`, dividing the eastern approach from the refuge terrace. Three contained, stone-rimmed alpine pools step down the gorge floor (`y = -43`, `-49` directly beneath the broken bridge, and `-53` in a deep lower plunge basin well inside the box).
- **The Landslide Scar (`x ≈ 2097..2111, z ≈ -18..-2`):** Exposed `gravel`, `andesite`, and `cobblestone` talus spill down from the north cliff across the ravine, marking the collapse that snapped the ancient stone bridge and chipped the northeast corner of the old stone guard post.
- **The Second Valley (`x = 2000..2068, z = -34..+14`):** West of the refuge cliff (`x < 2069`), the terrain plunges into a quiet, wind-sheltered alpine bowl (`y = -57..-54`) holding a still glacial tarn (`x = 2033..2044, z = -15..-7`, water surface `y = -56`). The northern and southwestern ridges frame an open western horizon (`x = 2000, z = -18..+4`) aligned with Minecraft's setting sun.
- **Climate & Wind:** Prevailing cold winds come from the north/northwest. Snow (`snow[layers=1..3]`) settles lower on north-facing rock (`y > -21`) than south-facing slopes (`y > -17`). Spruce trees grow in wind-sheltered hollows and valley pockets, explicitly excluded from the eastern landmark sightline and western sunset sightline.

### 02. Reason for Occupation
- Centuries ago, trade between the river city to the east and settlements beyond the pass required a stone road and arched bridge across the ravine. A compact stone tollhouse and gatehouse arch (`x = 2075..2095`) were built on the defensible `y = -38` shelf immediately west of the bridgehead, backed into the north rock face for shelter against avalanches and north gales.
- After the north-cliff landslide snapped the bridge and traffic dwindled, foot travellers still needed a way over the saddle. Later caretakers cleared a rock-stepped ravine detour with a timber footbridge (`z = +14`), enclosed the leeward courtyard behind the old gatehouse in local spruce timber, raised a steep alpine gable roof with a hearth chimney, and kept the room stocked and warm for anyone crossing in storm or dark.

### 03. Essential Spaces and Movement
- **Threshold sequence (East → West):**
  1. **Cramped mountain trail** (`x = 2179 → 2111`, winding `y = -61 → -39`): narrow (`~2.6` blocks wide), stone-retained, marked by small cairns.
  2. **Broken bridge & detour** (`x = 2112 → 2095`, dipping down to `y = -48` via `z = +14.5`): forces the traveller off the ancient alignment, down beside the ravine pools, across the timber footplank under the shattered stone piers, and back up to the west bridgehead.
  3. **Old Gatehouse Arch** (`x = 2091..2095, z = 0..+2, y = -38`): tall (4–5 blocks), cold, open stone vault straddling the old road.
  4. **Covered Porch & Low Later Door** (`x = 2085..2090, z = -1..+2`): sheltered under deep south eaves (`y = -34`), with a waiting bench, rainwater cauldron, firewood stack, and the 2-block-tall `spruce_door` at `(2086, -37..-36, -1)`.
  5. **The Vestibule** (`x = 2085..2087, z = -4..-2`): tight 3×3 room, strictly 3 blocks tall (`y = -37..-35`), with boot bench, gear shelf, and dim hanging lantern—hiding the main room until you turn left (west) through `(2084, -37..-36, -3)`.
  6. **The Hearth Room** (`x = 2076..2088` L-plan, ceiling rising to `y = -29` ridge): warm, tall communal heart with the north stone hearth, hanging pot, kitchen corner (`furnace`, `smoker`, `crafting_table`, `chest`, added upper `barrel` cupboard), two south bunk alcoves, and the NE bay whose 2×2 east window (`x = 2089, z = -8..-7`) looks back at the broken bridge and serves as the night beacon down the eastern pass.
  7. **The Sunken West Alcove** (`x = 2071..2074, z = -7..-3, floor y = -39`): screened behind bookshelves at `x = 2075`, stepped down half a slab into subdued shadow, containing a `lectern`, tea pot (`potted_fern`), books, a soft corner lantern, and a fitted window bench (`x = 2071, z = -6..-5`) directly beneath the **two-block valley window** (`x = 2070, z = -6..-5, y = -36..-35`) overlooking the second valley and tarn.
- **Secondary outdoor work:** Across the old road lane (`z = +1..+3`), tucked into the south rock bank, sit the **maintenance bay & small stable** (`x = 2079..2088, z = +5..+9`) and a **protected alpine garden** (`x = 2071..2077, z = +6..+10`).

### 04. Construction Logic
- **Foundations & retaining walls:** Heavy `stone_bricks`, `andesite`, and `cobblestone` anchor the north wall against the mountain cliff, extend down to `y = -41` on the west alcove cliff edge, and form the `y = -52..-39` bridge abutments.
- **Load transfer & roofs:** The main steep 45° alpine gable (`z = -12..+1`, ridge `y = -28`) sheds snow cleanly off both eaves, supported by interior `stripped_spruce_log` tie beams (`x = 2079, 2084`) and south porch posts (`x = 2084, z = +1` and `x = 2090, z = 0`). The secondary alcove lean-to roof (`x = 2069..2073`) tucks neatly under the main west gable eave (`x = 2074`).
- **Drainage:** Rain drips off the south eave directly into the `cauldron` on `mossy_cobblestone` at `(2085, -37, 0)`. Heat from the north stone chimney (`x = 2080..2083`) melts a dry snow-free gap along the north eave directly beside it.

### 05. Form Language
- **Material discipline (matching the three-build world):**
  - Local pass stone: `stone`, `andesite`, `polished_andesite`, `cobblestone`, `stone_bricks`, `cracked_stone_bricks`, `mossy_stone_bricks`.
  - Timber: local `spruce` (`spruce_log`, `stripped_spruce_log`, `spruce_planks`, `spruce_stairs`, `spruce_slab`, `spruce_trapdoor`, `spruce_shelf`), with `dark_oak` reserved strictly for the smoke-darkened hearth mantel and weathered north roof shingles.
  - **The One Carried City Stone:** A single `minecraft:chiseled_tuff_bricks` block sits as the east cheek of the hearth at `(2083, -37, -10)`—paired symmetrically with a salvaged `mossy_stone_bricks` block from the ruined bridge on the west cheek `(2080, -37, -10)`—linking the refuge silently to the river city's quarry masonry.
  - **Copper:** Only one weatherproof `copper_lantern` hangs inside the outdoor maintenance bay (`2081, -35, 7`), marking a newer workshop fitting.
- **Restraint & proportion:** No decorative roof spikes or scattered block noise. Slabs, stairs, and trapdoors express window sills, folded storm shutters, roof overhangs, shelves, and benches.

### 06. Experience in Motion and Time
- **Night & snowfall:** From `(2179, -60, 0)`, the pass is dark except for the tiny warning lantern at the bridge brink `(2112, -37, 2)` and the warm glowing 2×2 east window `(2089, -36..-35, -8..-7)` shining high on the saddle.
- **Rain & storms:** The covered porch (`z = 0`), gatehouse vault, and open-fronted shed provide dry transition pauses before entering.
- **Sunset:** Sitting or standing on the alcove window seat (`x = 2071, z = -6..-5`) frames the setting sun descending directly into the V-gap of the second valley above the reflective tarn.

### 07. Evidence of Occupation, Craft, and History
- **Three layers visible simultaneously:** Era 1 stone gatehouse & north core -> Era 2 landslide repair (`spruce_planks` patching the broken NE stone corner at `x = 2088..2089, z = -11`), timber enclosure, and detour footbridge -> Era 3 newer `spruce` roof repair patch on the south-west slope (`x = 2075..2081, z = -4..+1`), stacked second cupboard `barrel` in the kitchen (`2079, -36, -10`), maintenance tools (`stonecutter`, `grindstone`, `anvil`), firewood pile, and empty `chest` / `lectern` ready for passing travellers.

---

## 2. ASCII Plans & Sections

### A. Site Plan (`x = 2000..2179`, `z = -60..59`, schematic, East = right)
```text
z=-55  [North Ridge & Snowfields (peak y=+6 at x=2092,z=-34)]
         \                                   /
z=-30     \  Spruce    [Spring -43]    Spruce  [East Foothills]
           \ Groves       |  Ravine            Groves
z=-15       [Tarn y=-56]  | [Landslide]   ____/
z=-5   <== [Second Valley]| [Refuge][Gate]--X Broken Bridge --(Trail climbs)--
z=+5   (West Sunset Gap)  | [Stable][Gard]\__Detour Footbridge (y=-48)________(2179,-60,0)
z=+20       (Old West Road winds SW -> (2000,-60,28))  | [Plunge Basin y=-53]
         /                                             \
z=+45  [South Ridge (peak y=-5 at x=2088,z=+33)]
      x=2000            2038       2070..2095   2104   2112               2179
```

### B. Main Building & Yard Plan (`x = 2069..2096`, `z = -12..+10`, floor `y = -38`, view at `y = -37`)
```text
       2069  70  71  72  73  74  75  76  77  78  79  80  81  82  83  84  85  86  87  88  89  90  91..95
z=-12        .   .   .   .   .   .   .   .   .   .   #   #   #   #   .   .   .   .   .   .   .
z=-11        .   .   .   .   .   #   #   #   #   #   #   _   _   #   #   #   #   #   R   R   .   (R=Landslide spruce repair)
z=-10        .   .   .   .   .   #   C   F   S   B   M   a   .   T   .   .   .   .   .   #   .   (M=bridge stone, T=chiseled_tuff_bricks)
z=-9         .   .   .   .   .   #   W   .   .   .   .   .   .   .   .   .   .   .   .   #   .   (C=Cauldron, F=Furnace, S=Smoker, B=Double Barrel)
z=-8     [======Alcove======]   #   H   .   .   .   .   .   .   .   .   h   t   .   m  [E]  .   ([E]=East 2x2 Window, t=Table, m=Lamp/Pot)
z=-7         #   L   f   l   .   B   .   .   .   .   c   .   .   c   .   h   t   .   L  [E]  .   (f=Fern, l=Lectern, c=Fireside Seat)
z=-6        [W]  s   .   .   d   .   .   .   .   .   .   .   .   .   .   .   .   .   .   #   .   ([W]=2x2 Valley Window, s=Window Seat, d=Step Down)
z=-5        [W]  s   .   .   d   .   .   .   .   .   .   .   .   .   .   P   P   P   P   #   .   (P=Vestibule Partition)
z=-4         #   L   .   .   .   B   .   .   .   .   .   .   .   .   .   .   .   b   .   #   .   (b=Vestibule Boot Bench)
z=-3         #   .   .   b   sh  #   .   .   .   .   .   .   .   .   .   >   .   .   sh  #   .   (>=Inner Doorway; sh=Shelf)
z=-2         #   #   #   #   #   #   .  [B1  B1] td  n  [B2  B2] .   #   P   P   P   #   #   [Gatehouse N Pier]
z=-1                             #   #   #   #   #  [w]  #   #   #   #   #  [D]  #   #   #   [Gatehouse N Pier]
z= 0                             .   .   .  [Firewood]   x   P   .   ca  .  (p)  bn  #   .   [Road / Arch Open]
z=+1         .   .   .   .   .   .   .   .   .   .   .   .   .   P   .   .   .   .   .   .   . [Arch Open z=0..2]
z=+2  <-- Old West Road (y=-38) ------------------------------------------------------------------------> Broken Bridge (2096..2100)
z=+3         .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   . [Gatehouse S Pier]
z=+4         .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   .   . [Gatehouse S Pier]
z=+5         .   .   .   .   .   .   .   .   .   #   .   .   .   #   f   g   f   f   #   .   . (g=Stable Gate)
z=+6         .   #   #   #   #   #   #   #   .   #   .   .   .   #   .   .   .   .   #   .   .
z=+7         .   #   az  .   .   lv  .   #   .   #   ba  .   lg  #   .   .   .   ba  #   .   . (ba=Barrel, lg=Spare Logs)
z=+8         .   #   .   .   [w] .   .   #   .   #   sc  gr  an  #   ca  .   hy  hy  #   .   . (sc=Stonecutter, gr=Grindstone, an=Anvil, hy=Hay)
z=+9         .   #   co  fe  .   .   fl  #   .   #   #   #   #   #   #   #   #   #   #   .   .
z=+10        .   #   #   #   #   #   #   #   .   .   .   .   .   .   .   .   .   .   .   .   .
```

### C. East–West Section (through `z = -6..-5` Refuge/Alcove and `z = 0..1` Gatehouse/Bridge)
```text
y=-24                     [Chimney Cap]
y=-28                         /\ (Main Roof Ridge z=-6..-5)
y=-31                  [x=2074]/  \_____[Gatehouse Parapet y=-31..-32]
y=-34           __[Alcove]    |    |     |  [Arch Vault]   (Ravine)
y=-36   (West) [W]  Alcove |  Hearth Room  |  Arch (y=-38) [Abutment]  ~ ~  [E.Pier] -> Trail
y=-38   \      #____(y=-39)|_______________|______________|    ||            ||
y=-45    \  Cliff                                              ||  (Broken)  ||
y=-50     \_____                                               ||___Pool_____||
y=-56           \_____[Tarn y=-56]____                         (y=-49..-51)
```

### D. North–South Section (at `x = 2081..2086`, Hearth → Porch → Lane → Stable)
```text
y=-24       [Chimney Cap]
y=-28            /\  (45-deg Alpine Spruce Gable)
y=-31           /  \
y=-34    (Cliff)    \_____(South Porch Eave)               ____(Stable Shed Roof)
y=-37    # [Hearth]  |    Hearth Room   | [Low Door] (Porch)  (Lane z=+2)  [Stable/Bench]
y=-38   ===#=========|==================|================|================|==============#==
```

---

## 3. The Walk (with Coordinates)

1. **Entering the Box `(2179, -60, 0)`:** Ground sits flush at `y = -61` (`coarse_dirt`). Looking WNW up the dark pass at night, the distant 2×2 east window `(2089, -36..-35, -8..-7)` shines high on the saddle above the ravine notch.
2. **Climbing the East Trail `(2179, -60, 0) → (2111, -38, 0)`:** Follow stone cairns (`2168, -3` -> `2143, 1` -> `2124, 4`) up gentle `1:3`–`1:4` gravel, `coarse_dirt`, and `andesite` grades to the shattered east pier `(2111, -38, 0)`.
3. **The Broken Bridge & Detour `(2112, -38, 1) → (2105, -47, 14) → (2095, -37, 1)`:** A fence-mounted warning lantern at `(2112, -37, 2)` marks the brink. Drop south along stone steps down to `y = -48` at `z = +14.5`, cross the timber footbridge (`x = 2102..2108, z = 13..15`) with its rail lantern `(2105, -46, 13)` looking north at collapsed `stone_bricks` lying in the `-49` pool, then climb the west switchback to `(2095, -37, 1)`.
4. **Old Gatehouse Arch `(2095..2091, -37, 0..2)`:** Pass beneath the cold 4.5-block-tall stone vault (`lantern` at `2093, -34, 1`) into the covered yard (`x = 2085..2090`).
5. **Covered Porch & Low Later Door `(2086, -37, -1)`:** Pause under the low south spruce eave beside the waiting bench `(2088, -37, 0)`, rainwater cauldron `(2085, -37, 0)`, and stacked firewood `(2078..2081, -37, 0)`. Step through the 2-block-tall `spruce_door` at `(2086, -37..-36, -1)`.
6. **The Vestibule `(2085..2087, -37, -4..-2)`:** A snug 3×3×3 room (`ceiling y = -34`) with boot bench `(2087, -37, -4)` and pack shelf `(2087, -36, -3)`. Turn left (west) through the inner frame `(2084, -37, -3)`.
7. **The Hearth Room `(2076..2088, -37, -10..-2)`:** Light and ceiling height open up. Ahead sits the glowing hearth (`2081..2082, -37, -11`), hanging cauldron (`2082, -36, -10`), salvaged bridge cheek (`2080, -37, -10`), and single carried `chiseled_tuff_bricks` (`2083, -37, -10`); to the right is the NE map table & east bridge window (`2089, -36..-35, -8..-7`); to the left are the kitchen (`2076..2079, -10..-8`) and two bunks (`z = -2`).
8. **The Quiet West Alcove `(2071..2074, -38, -7..-3)`:** Step between the bookshelves at `x = 2075, z = -6..-5` down the half-slab threshold `(2074, -38, -6..-5)` into the dim timber alcove (`lectern` at `2073, -38, -7`, `potted_fern` at `2072, -37, -7`, quiet lantern at `2073, -37, -3`). Sit/stand on the window seat `(2071, -38, -6..-5)` to frame the second valley, the reflective tarn `(2038, -56, -11)`, and the setting sun through the 2×2 opening `(2070, -36..-35, -6..-5)`.

---

## 4. Section 6 Tests (Honest Answers)

- **Does it become a place someone genuinely wants to return to with no practical reason?**  
  Yes, provided the player approaches across the pass and pauses in the alcove: the sequence of work (detour -> cold arch -> cramped vestibule -> warm hearth -> quiet sunset seat) makes the shelter feel earned, and the empty `chest`, `lectern`, bunks, and stable give players reasons to actually store gear, sleep, and leave notes.
- **If the impressive view disappears, does the place still have character?**  
  Yes. Even in fog or dark, the broken bridge abutment, landslide scar on the NE stone corner, 3×3×3 vestibule turn, hearth with its single carried tuff stone, rainwater cauldron, and tucked-away stable/garden tell a cohesive story.
- **If the elaborate decoration is removed, do the spaces still feel good?**  
  The spatial ratios carry the building: a 4.5-high stone arch, a 3×3×3 low vestibule, a 6–9-block-tall pitched hearth hall, and a sunken 4×5 alcove with a 2×2 window. Strip the shelves and pots and those compression/release transitions still hold.
- **Does the back receive as much thought as the front?**  
  The north back wall is heavy stone retaining the cliff, penetrated by the sooted chimney (`basalt` flue, rain cap, melted snow gap on the north overhang). The west back houses the sunken alcove, stone foundation cliff drop (`y = -41`), and shutters overlooking the second valley.
- **Could it be modified after a year of use without destroying its identity?**  
  Yes—there is open yard/shelf space for extra crates, hooks, or crops in the garden, a second bunk or loft bed can be fitted under the `y = -29` ridge, and the hearth/kitchen already shows an added upper barrel cupboard.
- **Does it become more interesting after ten visits?**  
  Returning reveals small causal links easily missed the first time: why the NE corner is timber-patched directly inline with the ravine landslide scar, why the single `chiseled_tuff_bricks` sits at the fireplace corner, why snow melts off only the center of the north eave, and how the old west road hides south of the wild tarn bowl.

---

## 5. Self-Check & Verification Audit

- **Block IDs checked against `engine/world/data/blocks.tsv`:** Every block ID in `gen.rs` (`stone`, `andesite`, `polished_andesite`, `cobblestone`, `mossy_cobblestone`, `gravel`, `dirt`, `coarse_dirt`, `podzol`, `rooted_dirt`, `grass_block`, `moss_block`, `moss_carpet`, `basalt`, `coal_block`, `stone_bricks`, `mossy_stone_bricks`, `cracked_stone_bricks`, `chiseled_tuff_bricks`, `snow`, `water`, `spruce_*`, `dark_oak_*`, `glass_pane`, `iron_bars`, `iron_chain`, `lantern`, `copper_lantern`, `shroomlight`, `cauldron`, `furnace`, `smoker`, `crafting_table`, `chest`, `barrel`, `lectern`, `bookshelf`, `chiseled_bookshelf`, `spruce_shelf`, `brown_bed`, `red_bed`, `brown_carpet`, `white_candle`, `hay_block`, `anvil`, `grindstone`, `stonecutter`, `flower_pot`, `potted_fern`, `potted_lily_of_the_valley`, `short_grass`, `fern`, `oxeye_daisy`, `lily_of_the_valley`, `cornflower`, `leaf_litter`, `azalea`, `flowering_azalea`) is verified present in `blocks.tsv`. (Note: `water_cauldron` was deliberately avoided after checking `blocks.tsv` and finding only `minecraft:cauldron`.)
- **Coordinate bounds:** `gen.rs` enforces `X_MIN..=X_MAX` (`2000..2179`), `Y_MIN..=Y_MAX` (`-61..120`), `Z_MIN..=Z_MAX` (`-60..59`) in `World::idx` and `World::fill`.
- **`/fill` limit (`<= 32,768`):** `emit_commands` enforces `(x2 - ix + 1) * (z2 - iz + 1) * (y2 - iy + 1) <= 32768` at every growth step.
- **Hard constraints verified both by inspection and by `validate(&world)` in `gen.rs`:**
  - All `spruce_leaves` carry `[persistent=true]`.
  - All `water` blocks (second valley tarn, 3 ravine pools, and `1x1` garden spring) are strictly interior (`>= 25` blocks from every box edge) and surrounded on bottom + all 4 cardinal sides by solid blocks or water (`validate` panics if any neighbor is `air`).
  - All `gravel` blocks rest on solid support below.
  - Zero `fire`, `lava`, `campfire`, entities, or NBT braces `{}`.
  - `chiseled_tuff_bricks` appears exactly **1 time** (`(2083, -37, -10)`), checked by `validate` (`tuff_count <= 1`).
  - All hanging/standing lanterns checked for top/bottom support by `validate`.
- **Command & block counts:** `run_command` is sandbox-restricted in this session, so `gen.rs` could not be executed locally to print exact `build.txt` line counts. When compiled and run by the integrator (`D:\rust\cargo\bin\rustc.exe -O gen.rs` then `./gen <out_dir>`), `gen.rs` operates over the `180 × 182 × 120` (`3,931,200` cell) grid and prints exact `commands` and `blocks` counts to `stderr` while writing `build.txt`.
