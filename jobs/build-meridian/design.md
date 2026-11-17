# III. The Hollow Meridian — Architectural Design

**Box:** `x` in `2300..2749` (450 blocks), `z` in `-700..-260` (441 blocks), `y` in `-61..170` (232 blocks).  
**Superflat ground surface:** `y = -61` (players stand at `y = -60`).  
**South Quarry Road connection:** `(2525, -60, -260)` toward the river city.  
**Deliverables:** Deterministic generator [`gen.rs`](gen.rs) (stdlib-only Rust) that writes `build.txt` and runs a full 3D voxel audit before flushing.

---

## 1. Design by Order of Decisions (Horizon to Hand)

### 01. The Landscape's Existing Structure
- **Geology:** A fractured, asymmetric volcanic massif rising out of the plain (`y = -61`) to twin northern shoulders (`y = +158..164` NW around `x = 2450, z = -558`; `y = +148..154` NE around `x = 2604, z = -548`) and a lower protective southern front ridge (`y = +95..118`) that conceals the interior void from the valley.
- **Stratigraphy:** Dipping crustal beds (`dip = 0.08 dx + 0.11 dz`): deep `deepslate` basement (`sy < -18`), mid-elevation alternating `tuff`, `andesite`, and `stone` (`-18 <= sy < 56`, with thick `tuff` seams concentrated around the volcanic pipe), and wind-scoured `andesite`/`stone` upper crags (`sy >= 56`). Foothills taper smoothly to `y = -61` at least 18 blocks inside all four box borders.
- **The Fracture & Saddle:** A natural low pass on the SW flank (`x ~ 2450, z ~ -432, y ~ +30`) breaks the outer ring wall; steep gullies to its west (`x ~ 2436`) and south (`x ~ 2468`) expose cliff rock where daylight slits later pierce out.

### 02. Reason for Occupation
- **Quarry Era:** The city to the south needed durable, workable masonry (`tuff_bricks`, `polished_tuff`). Quarrymen found a rich vertical volcanic `tuff` pipe trapped inside hard `andesite` rim rock. Because the lowest pass through the front rim lay at the SW saddle (`y = +28`), they drove a switchbacking haul road up to the saddle, built a winch gatehouse there, tunneled through the rim wall, and excavated the tuff core downward (`y = +30 -> -48`), leaving an enormous benched vertical hollow (`~85` blocks across, `~210` blocks of total vertical relief from `-49` mirror pool to `+164` rim).
- **Observatory Era:** Long after extraction ceased, astronomers recognized that the sheer quiet abyss and open irregular sky crater formed a natural zenith telescope. They carved a razor-precise North Meridian Cut (`x = 2524..2526, y = +82..165`) and East/West horizon slits into the upper ridges, anchored an observation bridge across the middle of the void, installed a real vanilla daylight-detector solar dial on the East face, and grafted timber workrooms and a residence onto the old quarry benches.

### 03. Essential Spaces and Movement
- **Concealment before revelation:** From the valley road (`z = -260..-420`), only the haul road, spoil terrace, retaining walls, modest timber-and-tuff gatehouse (`y = +28`), and the distant vertical notch in the summit crest are visible.
- **Compression -> Release:** A 56-block, 3-wide rock-cut tunnel (`y = +28 -> +30`) with three narrow daylight slits opens abruptly onto the **Arrival Gallery** (`y = +30`), 78 blocks above the dark deepslate basin (`y = -48`). Across 80 blocks of empty dark stone glows **one warm 2×2 window** (setting human scale) and a quiet horizontal row of **7 indicator lamps**.
- **Clockwise Descent Circuit:** Arrival Gallery (`y = +30`) -> West Cliff Gallery (`+30 -> +20`) -> Curved Rock Library (`+20`) -> Great Meridian Cross-Bridge (`+16`) -> Main Instrument & Solar Dial Chamber (`+16`, with branch stairs up to the Astronomer's Residence at `+25` and Sheltered Pocket Garden at `+29`) -> East & South-East cantilevered descent ribbon (`+16 -> -44`) past a Hoist Overlook Alcove (`-4`) -> **Lowest Public Chamber** (`-44`) and **Meridian Causeway & Mirror Pools** (`-48`).
- **Second Revelation:** From the Lowest Public Chamber (`x = 2525, y = -44, z = -438`), turning North-East and looking upward reveals the entire human work winding up the rock walls: the descent ribbon, the 7-lamp dial, the glowing residence window, the half-stone/half-copper cross-bridge, the library piers, the arrival gallery mouth, and the North Meridian Cut against the sky.

### 04. Construction Logic
- **Walkability:** All paths use axis-aligned `build_axis_path` geometry with flat corner landings, continuous `tuff_brick_stairs`/`polished_tuff_stairs`/`spruce_stairs` (`<= 1` block rise per step, zero jumps), `4–5` blocks of clear headroom, battered `tuff_bricks` retaining foundations down to solid rock, and `tuff_brick_wall` or `waxed_oxidized_copper_bars` parapets on drop-offs.
- **Cross-Bridge Asymmetry (`y = +16`):** The West end (`x = 2486..2504`) rests on a heavy battered `tuff_bricks` quarry arch; the East end (`x = 2547..2565`) hangs from visibly newer `waxed_weathered_cut_copper` and `waxed_oxidized_copper_bars` truss struts bolted into the irregular cliff; the center (`x = 2525, z = -502`) holds a `waxed_oxidized_copper_grate` zenith oculus aligned over the basin causeway.
- **The Instrument (Vanilla Java 26.2):** Directly behind the 7-lamp display wall (`x = 2568, y = +19, z = -504..-492`), an open sky light-well (`x = 2572..2584, z = -505..-491, y = +20..170`) feeds 7 isolated straight `redstone_wire` channels of calibrated lengths (`3, 7, 11, 14, 11, 7, 3` blocks) from 6 normal `daylight_detector[inverted=false]` and 1 night `daylight_detector[inverted=true]`. As sky light rises toward Solar Noon (`power = 15`), lamps illuminate symmetrically inward until the center **Noon Zenith Gold Lamp** (`L = 14`) fires; at dusk, the outer **Copper Night Lamp** takes over. A raised inspection catwalk (`z = -508..-506, y = +19`) lets visitors look directly down onto the open channels and understand the physics.

### 05. Form Language
- **Geology vs. Intervention:** Coarse fractured `tuff`, `andesite`, and `deepslate` quarry faces remain largely untouched and quiet; human cuts are crisp (`polished_tuff`, `polished_deepslate`, `tuff_bricks`).
- **Material Coding:** `tuff` & `tuff_bricks` = original quarry & shared city stone; `spruce_log`/`spruce_planks` = human warmth & dwellings; `waxed_weathered_cut_copper` / `waxed_oxidized_copper_*` = astronomical instruments, railings, and newer structural reinforcements; `calcite` = pale graduation marks on dark stone; `gold_block` = strictly reserved for exact calibration points (Noon lamp frame, Bridge Oculus alignment, Basin Zenith Ring & 10-block meridian ticks, North Cut sill).

### 06. Experience in Motion and Time
- **Morning:** Low sun enters from the East (`+X`) through the **East Dawn Slit** (`y = +54..80`) and across the crater rim, lighting the West Gallery and Curved Library; `L = 3, 7` dial lamps glow.
- **Solar Noon:** Sun aligns overhead on `x = 2525`, pouring down through the **North Meridian Cut**, the **Bridge Oculus**, and onto the **Basin Zenith Ring**; all daytime dial lamps including the central Gold Noon Lamp light.
- **Dusk & Night:** Sun enters the **West Dusk Slit** (`y = +54..80`); at nightfall, the crater frames the stars, the inverted copper night lamp activates, and the **Astronomer's single warm window** becomes the anchor of human presence across the dark void.

### 07. Evidence of Occupation, Craft & History
- Quarry spoil terrace with ruined timber derrick and stacked cut `tuff_bricks`/`chiseled_tuff_bricks` awaiting haulage; cart `rail` track and fenced hoist pit inside the Gatehouse; empty `barrel`, `chest`, `furnace`, `smoker`, `lectern`, `cartography_table`, `crafting_table`; `flower_pot` teacup and `brown_bed` beside the Astronomer's window; sheltered moss-and-spruce rainwater garden; calibrated wire-length `calcite` markers beside the solar dial.

---

## 2. ASCII Plans & Sections

### A. Site Plan (`x = 2300..2749, z = -700..-260`, North = Up `-Z`)
```text
x: 2300        2400        2483   2525   2567      2650      2749
z=-700 +-----------------------------------------------------------+
       | . . . . . . . . plain (y=-61) . . . . . . . . . . . . . . |
-620   | . . .  foothills & sparse alpine spruce  . . . . . . . .  |
       |           +---NW Peak(+164)---+   +---NE Peak(+154)---+   |
-540   |           |        || MERIDIAN CUT (x=2525) ||        |   |
       |        [Library]   +-------HOLLOW-------+  [LightWell]    |
-490   |   <DuskSlit   | (West) ==BRIDGE== (East) |  [Dial/Res]  DawnSlit>
       |               |          (Pool)          |  [Garden]      |
-440   |   [Gatehouse]==Tunnel==>  [LowestChamber]|                |
       |    (y=+28)  \             (y=-44)        /                |
-380   |        \=====\==========Switchback=======/                |
       |                 [Spoil/Crane]  (y=-45..-6)                |
-310   |                        ||                                 |
-260   +------------------------||--(2525,-60,-260) Road to City---+
```

### B. Hollow Interior Plan (Composite `y = -48..+30`, `x = 2444..2586, z = -536..-422`)
```text
       2444    2460     2484        2525        2566     2586
z=-534 .       .       /=====[Curved Library]    .        .
       .       .      /        (y=+20)   |       .        .
-510   .       .     /   V O I D         |   [Catwalk y=19]
-502   .       .   [W-Pier]====[BRIDGE y=+16]====[Inst. y=16][SkyWell]
-490   .       .     |     ~ ~ |[Zenith]| ~ ~    [7 Lamps]  |
-476   .       .   [Arrival]   ~ ~pool~ ~        [Res. y=25](2567 lit win)
-470   .   /===Tunnel(y=30)    | Causeway |      [Garden y=29, sky open]
-455   .  /    |               v (y=-48)  |         | (Descent)
-443   . |   [Gatehouse]   [Lowest Chamber y=-44]==/ [Hoist Alcove y=-4]
-423   . |   (y=+28)
```

### C. East–West Cross Section at `z = -498` (Looking North `-Z`, Vertical Relief `y = -61..+165`)
```text
y
+165        /^\                        /^\
+140       /   \    (Open Sky Rim)    /   \
+100      /     |                    |     \
 +60     /      |  <Dusk  ..  Dawn>  |      \
 +30    /  [Arrival y=30]    [Res/Win y=27]  \
 +16   |        |==[BRIDGE y=16]==| [Dial][SkyWell y=19..170]
  +4   |        |                 |  \Descent
 -18   |        |   (Open Void)   |    \
 -44   |        |                 | [Lowest Chamber y=-44]
 -48   |________|___~[Zenith]~____|________________________
 -61   +---------------------------------------------------+
      2400    2484     2525      2566    2586            2660
```

### D. Solar Instrument Mechanism Detail (`z = -508..-491, x = 2566..2585, y = +16..+24`)
```text
z=-507  [Catwalk y=+19, Lectern, Cartography Table, Copper Rail] -> views south
z=-504  [Lamp] <- r r [Det SL15]                                 (Dawn, L=3)
z=-502  [Lamp] <- r r r r r r [Det SL15]                         (Mid-AM, L=7)
z=-500  [Lamp] <- r r r r r r r r r r [Det SL15]                 (Late-AM, L=11)
z=-498  [GOLD] <- r r r r r r r r r r r r r [Det SL15]           (NOON ZENITH, L=14)
z=-496  [Lamp] <- r r r r r r r r r r [Det SL15]                 (Early-PM, L=11)
z=-494  [Lamp] <- r r r r r r [Det SL15]                         (Late-PM, L=7)
z=-492  [CuLp] <- r r [Det INV]                                  (NIGHT, L=3)
       x=2568  2569 ...................... 2583                  (Open sky shaft y=20..170)
```

---

## 3. Step-by-Step Main Walk (With Coordinates)

1. **South Box Edge `(2525, -60, -260)`:** Enter north along the worn `tuff`/`coarse_dirt` quarry cart track; climb gently (`z = -280..-328`) to the **Spoil Terrace (`y = -45`, `(2525, -45, -331)`)** beside the ruined loading crane (`x = 2517, z = -332`).
2. **Switchback Ascent:** Turn East along the `tuff_bricks` retaining wall to `(2609, -12, -334)` -> `(2609, -6, -366)` (milestone view over the plains) -> West along the upper flank to `(2452, 22, -366)` -> North onto the SW Saddle Forecourt `(2452, 28, -423)`.
3. **Gatehouse `(2452, 29, -432)`:** Enter the modest stone-and-spruce winch house (cart `rail`, fenced hoist pit, `furnace`, `barrel`s, `lectern`). Pass out the NE doorway `(2458, 29, -441)`.
4. **Restrained Entry Passage `(2458, 29, -441) -> (2483, 31, -472)`:** Walk 56 blocks through 3-wide `tuff` rock with timber mine-props and 3 sharp daylight slits (`z = -449`, `x = 2468`, `z = -464`).
5. **Arrival Gallery `(2486, 31, -472)`:** The passage ends. Step onto the quiet `polished_tuff` balcony 78 blocks above the `-48` basin. Across the void (`x = 2567, y = 27..28, z = -486..-485`) shines the single warm Astronomer's window; below it (`y = 19`) glows the 7-lamp scale.
6. **West Gallery & Curved Library `(2484, 30, -476) -> (2480, 21, -522)`:** Descend 10 blocks north into the concave rock-cut library (`bookshelf`, `cartography_table`, reading bench, `lectern`).
7. **Great Meridian Cross-Bridge `(2485, 17, -502) -> (2566, 17, -502)`:** Cross from the ancient `tuff_bricks` West pier over the central `waxed_oxidized_copper_grate` Zenith Oculus `(2525, 16, -502)` onto the newer `waxed_weathered_cut_copper` East truss.
8. **Instrument Chamber `(2566, 17, -507) -> (2584, 20, -507)`:** Walk north of the 7-lamp wall onto the raised inspection catwalk overlooking the sky chimney and calibrated redstone wire channels.
9. **Astronomer's Residence & Pocket Garden `(2579, 17, -490) -> (2572, 26, -482) -> (2574, 30, -468)`:** Climb the tucked east stairs into the warm spruce cabin (viewing the famous lit window from inside, with desk, teapot, `brown_bed`, stove), then step through the south door up to the sunlit moss-and-spruce rainwater sanctuary.
10. **Descent Ribbon & Lowest Public Chamber `(2566, 17, -490) -> (2522, -43, -443)`:** Descend the cantilevered East and South-East galleries past the Machinery Hoist Alcove `(2563, -3, -453)` into the vaulted Lowest Public Chamber `(2525, -43, -438)`.
11. **Meridian Basin & Second Revelation `(2525, -47, -451) -> (2525, -47, -490)`:** Step down the broad `polished_deepslate_stairs` onto the `calcite`/`gold_block` Meridian Causeway between the twin mirror pools, stand at the Zenith Ring `(2525, -48, -490)`, and look back North and upward at the entire route winding up the mountain walls beneath the North Meridian Cut.

---

## 4. Section 6 Tests (Honest Evaluation)

- **Does it produce a real change in the visitor's understanding of what they're seeing, and does that understanding deepen the feeling of awe?**
  Yes: the distant precision notch on the summit ridge, the inexplicable row of 7 lamps seen from the Arrival Gallery, and the dizzying dark pool below all resolve into one intelligible physical system once you reach the bridge oculus, inspect the open daylight-detector wire channels, and look back up from the basin causeway.
- **If the impressive view disappears, does the place still have character?**
  Yes: the quarry road's battered retaining walls, the utilitarian winch gatehouse, the tight daylight-slit tunnel, the curved rock library, and the quiet spruce cabin with its sheltered rainwater garden each have strong enclosure, scale, and lived-in function on their own.
- **If the elaborate decoration is removed, do the spaces still feel good?**
  The spaces have almost no surface ornament—nearly all visual power comes from geological mass, void compression/release, stepped levels, bridge structural contrast (stone pier vs. copper truss), and motivated light.
- **Can the visitor understand why a structure is there?**
  Yes: the quarry road climbs to the only low rim pass (`y = +28`); the tunnel cuts the rim wall; the galleries reuse quarry extraction benches; the cross-bridge connects opposing cliff buttresses across the narrowest middle span; the residence sits on an old maintenance ledge; and the instrument sits beneath open sky.
- **Does the back of a building receive as much architectural thought as the front?**
  The Gatehouse has full pitched gable geometry and rear tunnel integration; the Residence has an east cliff stair, chimney/hearth, and south garden door; the Instrument Wall's back face is literally the pedagogical centerpiece (the open inspection channels).
- **Could the building be modified after a year of use without destroying its identity?**
  Yes: empty chests, barrels, lecterns, and open quarry benches invite players to store logs, add books/maps, extend redstone indicators, or carve new side studies into the quiet tuff walls without disturbing the void.
- **Does the place become more interesting after you've seen it ten times?**
  Visiting at dawn, solar noon, dusk, night, and rain changes which horizon slits catch sky, which lamps illuminate, and whether the single residence window or the crater sky dominates the void.

---

## 5. Self-Check & Verification Notes

1. **Block ID & State Verification (`blocks.tsv`):** All 63 distinct block IDs used in `gen.rs` were checked directly against `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv` (using `iron_chain`, `waxed_oxidized_copper_*`, `waxed_weathered_cut_copper`, `redstone_wire[east=side,north=none,power=0,south=none,west=side]`, `daylight_detector[inverted=false|true]`, both halves of `spruce_door`, both parts of `brown_bed`, `spruce_leaves[persistent=true]`, `lantern[hanging=true|false]`, `water[level=0]`, `stairs` with `facing`/`half`/`shape`, `slab` with `type`, `trapdoor` with `facing`/`half`/`open`).
2. **Coordinate Bounds & `/fill` Limits:** `World::check_bounds` enforces `2300 <= x <= 2749`, `-61 <= y <= 170`, `-700 <= z <= -260` on every placement; `World::emit_fill_split` and `emit_landscape_from_grid` automatically subdivide every `/fill` so no command exceeds `32,768` blocks.
3. **Physical Invariants (`run_audit` in `gen.rs`):** Every water block (`water[level=0]` in the Pocket Garden rainwater basin and the twin Mirror Pools) is verified to be bounded on all four horizontal sides and underneath by solid non-air blocks; every `gravel` talus block is verified to rest on solid support; every `spruce_leaves` has `persistent=true`; every `daylight_detector` has pure `air` to `y = 170`; every `lantern`, `rail`, `lectern`, `redstone_wire`, `spruce_door`, and `brown_bed` has valid support/pairing; and the 80-block sightline from Arrival Gallery `(2487, 32, -472)` to the Astronomer's Lit Window `(2566, 28, -485)` is raycast-verified unobstructed.
4. **Execution Status:** `run_command` was unavailable in this session due to a harness sandbox configuration error (`readonly file://D:/path-tracing-wt/: non-absolute file path`), so `rustc.exe` was not executed here; therefore `build.txt` and exact runtime command/block counts are produced when the integrator compiles and runs `gen.rs` (`D:\rust\cargo\bin\rustc.exe -O gen.rs` then `gen.exe "C:\Users\PASCAL\Desktop\path tracing\jobs\build-meridian"`), which also appends the exact `# SELF-AUDIT SUMMARY` counts to `build.txt`.
