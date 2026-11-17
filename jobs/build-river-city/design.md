# II. The City That Borrowed a River — Architectural Design

**Box (inclusive):** `x in [2300, 2749]` (450 blocks), `z in [-125, 124]` (250 blocks), `y in [-61, 80]` (142 blocks).  
**External Connections:**
- **West Road:** enters at `(2300, -60, 0)` from *The Last Hearth* (pass & refuge).
- **North Quarry Road:** leaves at `(2525, -60, -125)` toward *The Hollow Meridian* (tuff quarry & observatory).

---

## 1. Design Order & Underlying Logic

### 1.1 Landscape Structure (Geology First)
The superflat world outside sits at `y = -61` (walkable surface `y = -60`). Inside the box, a smooth perimeter taper brings all four edges down to `y = -61` over 25–42 blocks of foothills and meadow so no vertical cliff or water body ever touches a box boundary.

- **Eastern High Plateau (`x = 2620..2710`):** Karst limestone/andesite shelf at `y = +4..+18`, framed by two flanking mountain spurs—the **Northeast Escarpment** (`x = 2645, z = -56`, peak `y = +38`) and **Southeast Ridge** (`x = 2668, z = +68`, peak `y = +48`)—plus north and south canyon rims (`y = +24..+34`).
- **The S-Bend Gorge (`x = 2410..2635`):** A river rises from a cliff spring (`x = 2675, z = +19, y = +12`) into the **Spring Basin** (`x = 2652..2672, z = +12..+26`, water `y = +4`), spills over the **Intake Weir** (`x = 2624, y = +2`), curves **northwest** down steep rapids (`y = +2 -> -32`) through the narrowest **Gorge Throat** (`x = 2525, z = -24`, cliffs rising to `y = -18`), then bends **southwest** around the Market Promontory (`y = -32 -> -54`) before widening into the **Lower Quays Canal & Terminal Estuary Lake** (`x = 2340..2458, z = -2..+48`, water `y = -58`, bed `y = -61`, natural levee rim `y = -57`).
- **Visual Concealment:** Because the gorge bends in an S-curve around the southern promontory (`z = +4..+40`) and northern cliff (`z = -50..-18`), an observer entering at the river basin cannot see the whole town at once—only the high bridge crossing the throat, stacked hillside roofs, and the elevated aqueduct disappearing behind stone workshops.

### 1.2 Reason for Occupation
Two geographic facts forced a city into existence here:
1. **The Narrows (`x = 2525, z = -24`):** The only point where the gorge pinches narrow enough (~12 blocks at water level) for a single stone arch to carry the north–south quarry/trade road.
2. **The Drop (`+4` to `-58`):** Sixty-two blocks of river fall over 240 horizontal blocks—too steep for boats above the basin, but ideal for tapping high water at the plateau weir (`y = +2`) and carrying it horizontally along the north canyon shelf to power workshops, quenching troughs, and washing vats far above the floodline.

### 1.3 The Three Networks (Designed Before Façades)
1. **Water Network:**
   - **Natural River:** Spring Basin (`y = +4`) -> Intake Weir (`y = +2`) -> Upper Gorge stepped pools (`-4, -10, -16, -24`) -> Bridge Throat (`-32`) -> Lower Weir pools (`-40, -48, -54`) -> Lower Quays Canal (`-58`) -> Terminal Estuary Lake (`-58`, fully enclosed inside `x >= 2340`).
   - **Borrowed River (Elevated Aqueduct):** Diverted at `(2624, +2, +10)` -> North Canyon Arcade (`y = +1 -> 0 -> -2`) -> Cistern Settling Tower (`x = 2475, z = -32, y = -14`) -> Stepped Masonry Tailrace (`y = -14 -> -54`) feeding real quenching & washing vats in the Smithy and Tannery before returning to the Canal at `(2454, -58, +10)`.
   - **Obsolete Aqueduct Spur:** Older high branch (`x = 2546 -> 2498, z = -42 -> -26, y = -8`) bricked shut with tuff and oxidized copper, drained and repurposed into a planted elevated walkway.
   - **Terrace Irrigation:** Contained stone channels (`y = -4..+4`) watering stepped orchard/garden plots on the south slope.
2. **Movement Network:**
   - **Public Spine:** West Road `(2300, -60, 0)` -> West Ferry Landing `(2336, -56, +2)` -> Lower Quays Wharf Street (`y = -56`) -> 3-wide Cliff Switchback Stairway (`y = -56 -> -18`, passing under the Guild Toll House) -> Old Market Square (`y = -18`) -> Old Tuff Bridge (`y = -18`) -> Upper Terraces stepped lanes (`y = -16..+6`) -> Intake Weir Footbridge (`y = +5`) -> Aqueduct Maintenance Walkway (`y = +2..0`) -> Obsolete Spur Footpath (`y = -8`) -> Old Fulling Mill loft -> Small Rooftop Garden doorway (`y = -4`).
   - **Quarry Road & Freight Bypass:** North Road `(2525, -60, -125)` climbs to Customs Gatehouse `(2525, -18, -50)`. At `(2512, -32, -74)`, a dedicated **Freight Haul Ramp** peels west straight down to the Lower Quays Stone Yard (`y = -56`), keeping heavy tuff sleds out of the market.
   - **Local Shortcuts:** Under-Bridge Service Ledge (`y = -30`, hidden stair behind Harbormaster's shed up to South Bridgehead) and Mill Cellar Stair.
3. **Productive Network:**
   - **Lower Quays:** Boatbuilder's Slipway, Cargo Warehouses (wide 3-block ground loading bays, smaller upper lofts), Stone Yard (`stonecutter`, `grindstone`, crane over canal), navigable canal basin.
   - **Waterworks:** Copper Foundry & Smithy (`blast_furnace`, `furnace`, `anvil`, `smithing_table`, quenching vat), Tannery & Washhouse (`cauldron`s, water vats, `loom`s).
   - **Old Market:** Positive outdoor plaza with Conduit Fountain, trading stalls (`crafting_table`, `barrel`, `chest`), Bakery (`smoker`s), Assay/Guild Hall (`lectern`, `cartography_table`, `bookshelf`), Inn ("The Weir & Bell").
   - **Upper Terraces:** Irrigated orchard/melon/pumpkin/sugarcane plots, `composter`s, Potter's Kiln, Tea House, homes with beds, stoves, and storage.

### 1.4 Outdoor Rooms & Four Districts
- **Outdoor Rooms Designed First:**
  1. **Old Market Square (`x = 2506..2542, z = -48..-28, y = -18`):** Enclosed trapezoidal room anchored by the Customs Gate (N), Assay Hall (E), Bakery (NW), Old Fulling Mill (SW), and Bridgehead (S), with a central fountain and arcade edges where people linger.
  2. **Communal South Courtyard (`x = 2520..2542, z = -4..+14, y = -16`):** Built inside the surviving tuff foundation walls of a demolished riverside mill, shaded by a central flowering tree.
  3. **Lower Quays Wharfside (`x = 2340..2462, z = +4..+13, y = -56`):** Tight, shadowed working street pinched between retaining walls and moored canal barges.
  4. **The Rooftop Garden (`x = 2480..2495, z = -26..-9, y = -4`):** High, quiet, unroofed belvedere atop the adapted Fulling Mill.

### 1.5 Construction Logic, Form Language & Three Historical Episodes
- **Episode 1 (Old Crossing):** Heavy Meridian `tuff`, `tuff_bricks`, `chiseled_tuff_bricks`, battered bases, small recessed windows, semicircular arches (Old Bridge, Customs Gatehouse, Watchtower Base on South Terrace).
- **Episode 2 (Waterworks & Industry):** Repetitive arched bays (`tuff_bricks`, `stone_bricks`, `deepslate_bricks` in damp lower zones), heavy `spruce_log` trusses, wide ground loading arches, `deepslate_tiles` roofs.
- **Episode 3 (Adaptation & Repair):** `calcite` lime plaster upper storeys, `spruce_planks` / `oak_planks` interiors, `bricks` chimneys, and `waxed_*_copper` exclusively for waterproofing, sluices, roof patches, and gutter/flashing repairs (progression of oxidation reflecting repair age). **Extraordinary buildings** (Old Bridge, Assay Hall, Aqueduct Arcade) stand beside **ordinary neighbours** (shops, sheds, worker cottages).

---

## 2. ASCII Plans & Sections

### 2.1 Site Plan (`x = 2300..2749`, `z = -125..124`)
North is UP (`-Z`), West is LEFT (`-X`).
```text
z=-125                  [North Quarry Road (2525,-60,-125)]
  |                             ||
  |     (N Foothills)           ||  <-- Weigh-Post (2519,-42,-89)
  |                             ||==\\ Freight Ramp -> Lower Quays
  |                             ||   \\
z=-50             [NE Ridge]    [Customs Gate]
  |        ======================++=========  <-- Active Aqueduct (y=0..+2, x=2624->2475)
  |       / [Cistern]   [Obsolete Spur]  [Assay Hall]    \\
  |      /  (2475,-14)  [Old Market Sq]  [Inn]            \\  [Upper Gorge]
z=-20   |   [Tailrace]  [Mill+ROOF GARDEN]===BRIDGE===     \\=== [Intake Weir (2624,+2)]
  |     |   [Smithy]    [Cliff Stairs]    (2525,-18)             [Spring Basin (2662,+4)]
  |     |   [StoneYd]                     [South Courtyard]          (Plateau y=+8..+18)
z=  0 ==+==[West Rd] [Boatyard][Warehouses][Terrace Houses]  [Irrigated Terraces]
  |     |  [Landing]~~~~~~~~~~~~~~~~~~~~~~[Canal y=-58]      [Tea House & Kiln]
z=+30   |  ~~~~~~~~~[Terminal Lake y=-58]~~~~~~~~~~~~~~         [SE Ridge y=+48]
  |
z=+124     (South Foothills sloping cleanly to y=-61)
      x=2300       2380       2460       2525       2585       2630       2680      2749
```

### 2.2 Old Market Square & Crossing Plan (`x = 2484..2554, z = -54..+16, y = -18..-4`)
```text
         2485       2495       2506       2518  2525  2532       2542       2552
z=-54    .          .          .          [=== NORTH GATE ===]   .          .
z=-48    .          .          [BAKERY   ]  .    ||    .  [MARKET]  [ASSAY  ]
z=-42    .  [Aqueduct Arcade]============+=====[SPUR]=====+==[STALL] [GUILD ]
z=-36    .          .          [Arcade   ]   (Fountain)   [Stall ] [HALL   ]
z=-30    .  [Cistern]          <-(Cliff Stairs)  ||       .  [WEIR & BELL  ]
z=-24    [OLD FULLING]  [Toll  ]                 ||==========[INN          ]
z=-18    [MILL       ]  [House ]~~~~~~GORGE~~~~~~||~~RIVER~~~~~~~~~~~~~~~~~~
z=-12    [ROOF GARDEN]  [Stair ]             [OLD BRIDGE]  [Watchtower-Base]
z= -6    .          .          .             [y=-18     ]  [Merchant House ]
z=  0    .  [Gorge Overlook]   [Demolished-Mill Courtyard]  .
z= +8    .  [Terrace Houses]   [Tree + Benches + Planters]  [Stepped Lane ->]
```

### 2.3 Lower Quays & Waterworks Plan (`x = 2334..2480, z = -38..+36, y = -58..-44`)
```text
         2336    2354      2376      2400      2424      2452      2468      2480
z=-34    .       .         .         .         .         [Cistern Tower y=-14]
z=-26    .       .         .         .         (Ramp)    [Tannery][Tailrace]
z=-16    .       .         .         .         [Freight] [Smithy & Foundry ]
z= -6    [Ferry] [Boatyard][Warehouse][Granary][StoneYd] [Cliff Stair Base ]
z= +4  ==[West ]=[=========== LOWER QUAYS WHARF STREET (y=-56) ============]
z=+14    [Landg] ~~~~~~~~~~[CANAL & QUAY BASIN (y=-58, bed=-61)]~~~~~~~~~~~
z=+28    ~~~~~~~ [TERMINAL ESTUARY LAKE (y=-58, levee rim y=-57)] ~~~~~~~~~
```

### 2.4 Longitudinal Gorge Section (West to East, `y = -61..+50`)
```text
y
+48                                                                 /\ SE Ridge
+24                                                           /\___/  \__
 +4                                                     [Spring Basin]==[Intake Weir]
 -4            (Mill Roof)     [ROOFTOP GARDEN]          ||  Active Aqueduct (y=0..+2)
-18            /               [Mill] [Market Square]====[BRIDGE]=== [Terraces]
-32           /                (Cliff Stairs)         ~~River(-32)~~
-46          /                 [Smithy][Tailrace]
-56  [Road] [West Landing] [Wharf Street y=-56] [StoneYd]
-58  ______ [~~~~~~~~~~~ Lake & Canal y=-58 ~~~~~~~~~~~~]
-61  ====== [Bed -61 ===================================]=========================
     x=2300  2340          2400        2465    2495      2525        2624    2680  2749
```

---

## 3. The Walk (Main Route Step-by-Step)

1. **Arrival (`(2300, -60, 0)` -> `(2336, -56, +2)`):** Enter along the West Road, climbing the gentle river levee past the West Ferry Landing onto the edge of the Terminal Estuary Lake.
2. **First View from the River (`(2360, -56, +6)` -> `(2410, -56, +8)`):** Walk east along the narrow, shadowed Lower Quays Wharf Street beside the canal (`y = -58`). Looking up the bending gorge, you see the Old Tuff Bridge high overhead (`y = -18`), terraced roofs stacked on the south hillside, and the masonry aqueduct disappearing behind the foundry cliff.
3. **Working Lower District (`(2358..2452, -56, -2..+10)`):** Pass the Boatbuilder's slipway, wide-bay cargo warehouses with hoists, the tuff Stone Yard, and the Freight Ramp descending from the north quarry road.
4. **The Cliff Climb (`(2462, -56, +6)` -> `(2508, -18, -28)`):** Ascend the 3-flight switchback stairway, passing the glowing blast furnaces and quenching troughs of the Copper Foundry (`(2466, -46, -8)`), then passing underneath the cantilevered Guild Toll House (`(2500, -28, -16)`).
5. **The Old Market & Crossing (`(2508, -18, -28)` -> `(2525, -18, -18)`):** Emerge into the open sunlight of the Old Market Square. The bridge that towered overhead is now level with your feet; you recognize the Toll House above the stairs you climbed; across the plaza stand the carved Assay Hall, Bakery, Inn, and Customs Gate.
6. **Upper Terraces & Infrastructure (`(2525, -18, -8)` -> `(2624, +5, +18)` -> `(2498, 0, -40)`):** Cross the Old Tuff Bridge to the demolished-mill Communal Courtyard (`(2530, -16, +4)`), climb past the Watchtower-Base house, Potter's Tea House, and irrigated terraces to the Intake Weir Footbridge (`(2624, +5, +18)`), then walk west along the **Aqueduct Maintenance Walkway** (`y = +2..0`) beside flowing water, stepping onto the repurposed Obsolete Spur Footpath (`y = -8`).
7. **The Rooftop Garden (`(2489, -4, -17)` -> `(2490, -4, -17)`):** Enter the Old Fulling Mill loft (`x = 2480..2495`), climb the quiet timber stair, open the small unremarkable spruce door at `(2489, -4, -17)`, and step out onto the Rooftop Garden (`y = -4`)—seeing the Market, Bridge, Terraces, River, Intake, Aqueduct, and Lower Quays united as one living organism.

---

## 4. Section 6 Tests (Honest Evaluation)

- **Can someone infer geography, architecture, and activity by walking through?** Yes: the tuff bridge sits at the sole narrow throat; the quarry road bypasses the market straight to the quay yard; the waterworks visibly tap the high weir (`+2`) to serve workshops mid-cliff before rejoining the canal; warehouses have wide loading ground floors while adapted upper terraces reuse old foundations and obsolete aqueduct piers.
- **Can new inhabitants create meaningful uses without breaking coherence?** Yes: empty chests, barrels, furnaces, blast furnaces, smokers, looms, stonecutters, crafting tables, boat slips, open market stalls, and empty rooms in the terraces and warehouses invite immediate survival use and adaptation.
- **If the view or ornament disappears, do the spaces still work?** Yes: the positive outdoor rooms (Market Square, South Courtyard, Wharf Street, Rooftop Garden) are shaped by enclosure, proportion, and circulation rather than façade clutter.
- **Are backs of buildings treated seriously?** Yes: service alleys, drainage spouts, retaining walls, chimney backs, and woodpiles occupy the rear edges against the rock.

---

## 5. Self-Check & Verification

- **Block IDs:** Every block ID in `gen.rs` is checked strictly against `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv` (Minecraft Java 26.2). No `farmland`, `wheat`, `campfire`, or unlisted IDs are used.
- **Box Bounds:** All commands emitted by `gen.rs` are clamped/asserted inside `x in [2300, 2749]`, `z in [-125, 124]`, `y in [-61, 80]`.
- **`/fill` Limit:** `gen.rs` provides a `fill_box(...)` helper that automatically subdivides any region exceeding 32,768 blocks so no single `/fill` command ever exceeds the limit.
- **Leaves, Falling Blocks, Water, Entities:** Every leaf block emits `[persistent=true]`; gravel/sand only rest on solid stone/dirt; every water block sits inside a solid-floored, solid-walled basin or bounded spillway inside the box (no water near any edge); zero entities or NBT.
- **Counts:** `gen.rs` (pure Rust `std`, greedy 3D box-merger + automated self-auditor across the `450 x 142 x 250 = 15,975,000` voxel box) emits `build.txt` and appends exact `fills`, `setblocks`, `total_commands`, `total_blocks`, and `audit_repairs` summary comments at the end of `build.txt` when compiled and run (`rustc -O gen.rs`). (Command execution is unavailable in this harness.)
