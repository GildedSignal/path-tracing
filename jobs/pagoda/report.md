# Pagoda Design Job — Progress Report

Starting design of a classical multi-tiered pagoda in `C:\Users\PASCAL\Desktop\path tracing\jobs\pagoda\`.
Site bounds: x in [20, 52], z in [-16, 16], y in [-60, -10]. Ground top surface y = -61, player stands at y = -60. Spawn at (0, -60, 0).

## Log
- Created `report.md`. Next: inspect `blocks.tsv` format and verify all candidate block IDs and block states for Minecraft Java 26.2.

### `blocks.tsv` Verification
- Read all 1,140 lines of `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv`.
- Confirmed 26.2 block IDs available for our pagoda palette:
  - Stone base & paths: `minecraft:stone_bricks`, `minecraft:chiseled_stone_bricks`, `minecraft:mossy_stone_bricks`, `minecraft:stone_brick_stairs`, `minecraft:stone_brick_slab`, `minecraft:stone_brick_wall`, `minecraft:polished_blackstone`, `minecraft:polished_blackstone_bricks`, `minecraft:chiseled_polished_blackstone`, `minecraft:polished_blackstone_brick_stairs`, `minecraft:polished_blackstone_brick_slab`, `minecraft:polished_blackstone_brick_wall`, `minecraft:polished_blackstone_stairs`, `minecraft:polished_blackstone_slab`, `minecraft:gilded_blackstone`, `minecraft:smooth_stone`, `minecraft:smooth_stone_slab`, `minecraft:cobblestone`, `minecraft:mossy_cobblestone`, `minecraft:andesite`, `minecraft:polished_andesite`.
  - Columns & timber framing: `minecraft:dark_oak_wood`, `minecraft:stripped_dark_oak_wood`, `minecraft:dark_oak_planks`, `minecraft:dark_oak_stairs`, `minecraft:dark_oak_slab`, `minecraft:dark_oak_fence`, `minecraft:dark_oak_trapdoor`, `minecraft:dark_oak_door`, `minecraft:spruce_planks`, `minecraft:spruce_stairs`, `minecraft:spruce_slab`, `minecraft:spruce_fence`, `minecraft:spruce_trapdoor`, `minecraft:stripped_crimson_hyphae`, `minecraft:crimson_hyphae`, `minecraft:crimson_planks`, `minecraft:crimson_stairs`, `minecraft:crimson_slab`, `minecraft:crimson_fence`, `minecraft:crimson_trapdoor`, `minecraft:red_terracotta`, `minecraft:red_nether_bricks`.
  - Roofs & upturned eaves: `minecraft:deepslate_tiles`, `minecraft:deepslate_tile_stairs`, `minecraft:deepslate_tile_slab`, `minecraft:deepslate_tile_wall`, `minecraft:deepslate_bricks`, `minecraft:deepslate_brick_stairs`, `minecraft:deepslate_brick_slab`, `minecraft:polished_deepslate`, `minecraft:polished_deepslate_stairs`, `minecraft:polished_deepslate_slab`, `minecraft:copper_lantern` / `minecraft:lantern` / `minecraft:soul_lantern`, `minecraft:waxed_cut_copper_stairs`, `minecraft:waxed_cut_copper_slab`, `minecraft:gold_block`, `minecraft:lightning_rod`.
  - Interior & lighting: `minecraft:ladder`, `minecraft:oak_planks`, `minecraft:cherry_planks`, `minecraft:white_concrete`, `minecraft:calcite`, `minecraft:bookshelf`, `minecraft:chiseled_bookshelf`, `minecraft:bell`, `minecraft:red_carpet`, `minecraft:sea_lantern`, `minecraft:glowstone`, `minecraft:shroomlight`, `minecraft:lantern`, `minecraft:flower_pot`, `minecraft:potted_bamboo`, `minecraft:potted_cherry_sapling`, `minecraft:potted_azalea_bush`.
  - Courtyard & garden: `minecraft:grass_block`, `minecraft:moss_block`, `minecraft:moss_carpet`, `minecraft:dirt_path`, `minecraft:water`, `minecraft:cherry_log`, `minecraft:cherry_leaves`, `minecraft:pink_petals`, `minecraft:flowering_azalea_leaves`, `minecraft:azalea_leaves`, `minecraft:bamboo_sapling` / `minecraft:sugar_cane`, `minecraft:lily_of_the_valley`, `minecraft:poppy`, `minecraft:allium`, `minecraft:peony`, `minecraft:short_grass`, `minecraft:fern`, `minecraft:sea_pickle`, `minecraft:seagrass`, `minecraft:smooth_quartz_slab`.

## Completed Design & Deliverables
- Wrote [build.txt](C:\Users\PASCAL\Desktop\path tracing\jobs\pagoda\build.txt) (963 lines total: `319` `/fill` commands + `478` `/setblock` commands = `797` commands; `7,951` block placements: `6,603` non-air + `1,348` `minecraft:air` carves).
- Wrote [design.md](C:\Users\PASCAL\Desktop\path tracing\jobs\pagoda\design.md) containing dimensions, palette rationale, ASCII plans for the courtyard and all 5 storeys, an ASCII side elevation, self-check results, and exact command/block counts.

### Architecture Highlights
- **Classical 5-Storey Pagoda** centered on `(38, -60..-10, 0)`, leaving an 8-block front Zen courtyard (`x = 20..27`) facing spawn `(0, -60, 0)`.
- **Approach & Courtyard:** Walkable jump-free entry from `x = 20, z = 0` under a vermilion Torii gate, across a dark-oak arched bridge over a glowing `sea_pickle` pond (`x = 23..26, z = -8..8`), flanked by symmetric cherry blossom trees, flowering azalea bonsai, flora, and 6 *toro* stone lanterns.
- **Tapered Storeys & Sweeping Upturned Eaves:** `13x13 -> 11x11 -> 9x9 -> 7x7 -> 5x5` walls with `stripped_crimson_hyphae` columns, `calcite` plaster panels, `crimson_fence` lattice windows, walkable 1-wide wrap-around balconies (`crimson_trapdoor[open=true]` railings), `dark_oak_stairs[half=top]` under-eave brackets, and layered `deepslate_tiles` roofs flaring into `waxed_cut_copper_slab` corner crests with hanging lanterns.
- **Crowning Sorin Finial (`y = -25..-10`):** `polished_blackstone_bricks` roban pedestal, `gold_block` fukubachi bowl, alternating `waxed_chiseled_copper` / `gold_block` / `waxed_copper_bars` kurin shaft, and `waxed_lightning_rod[facing=up]` tip at `y = -10`.
- **Walkable Lit Interior:** Continuous spiral staircase around the central `stripped_dark_oak_wood` *shinbashira* from Storey 1 (`y = -57`) to Storey 5 (`y = -32`) and out onto the top balcony, lit on every storey by `lantern[hanging=true]` and recessed `shroomlight`s, with a Storey 1 red-carpet aisle, bookshelves, and gilded bell shrine.

### Verification & Audit Fixes
- Verified all 53 distinct block IDs against `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv`.
- Verified all coordinates inside `x in [20, 52]`, `y in [-60, -10]`, `z in [-16, 16]` and max `/fill` volume `605 <= 32,768`.
- Audit catches fixed in `build.txt`: removed duplicate Torii column `/fill`, preserved Storey 5 stairwell arrival landing `(39, -33, -1)` while clearing `(38, -33, 1)` for >= 3-block headroom, and replaced a floating Storey 2 guard fence `(37, -50, 1)` with solid-supported fences `(36, -50, 1..2)`.
