# Pass 1 Report — The Last Hearth (Small)

## Status
Starting Pass 1: reading the private architecture brief, the taste profile, the pagoda reference (`design.md` and `build.txt`), and verifying 26.2 block IDs/states in `engine\world\data\blocks.tsv`.

## Findings from Reference Reading
- **Pagoda benchmark (`jobs\pagoda\design.md`, `build.txt`):**
  - 797 commands total (`319` `/fill`, `478` `/setblock`), ~6,603 non-air placements inside a 33x51x33 box.
  - Worked well because every structural layer, stair, slab, overhang, and interior connection was explicitly planned in ASCII elevations/plans and written by hand with absolute coordinates and exact blockstates.
  - Zero-jump walkability from spawn through entrance, threshold, stairs, and upper levels.
- **Architectural direction for The Last Hearth (Small):**
  - Box: `x = 400..440` (41 wide), `z = -20..20` (41 deep), `y = -61..-16` (46 tall; superflat top surface at `y = -61`, standing at `y = -60`).
  - Road enters from the west at `x = 400`. Since superflat cannot be dug below `-63` and top surface is `-61`, we build up the rocky spur, plateau, and ravine cliffs from `y = -61` so the ravine floor sits around `-61..-59` while the spur/road/bridge deck rise up into `y = -54..-48` (leaving plenty of height below `-16` for the two-tier refuge, dominant roof, chimney, and tall pines).
  - Core narrative & spatial rhythm: an old mountain road along a rocky spur meets a collapsed ravine bridge at the surviving west abutment; travellers detour around the broken crossing to a sheltered lower timber doorway beside the heavy stone tollhouse core; inside, a tight low vestibule opens into a warmer, taller hearth room with a window glancing back at the broken bridge, leading onward to a quiet kitchen/workspace corner, sleeping alcoves, and finally a secluded two-block-wide alcove window seat looking eastward across the ravine toward the far cliffside.
  - Emotional register: quiet continuity and unthanked stewardship over generations—restraint, shadow and warm lamplight, directional weathering (moss on shaded damp rock, soot at the chimney, newer spruce repairs against aged stone).

## Pass 1 Decisions & Outputs
- **Checked 26.2 block IDs against `engine\world\data\blocks.tsv`:**
  - Confirmed `campfire` and `soul_campfire` are absent from `blocks.tsv`; designed the hearth firebox around fire-free warm light (`ochre_froglight`, `magma_block`, `lantern[hanging=false]`) recessed behind `iron_bars` and soot-stained `polished_basalt`/`cobbled_deepslate`.
  - Confirmed `tuff_bricks`, `polished_tuff`, `exposed_copper_lantern`, `leaf_litter`, `potted_fern`, `cartography_table`, `lectern`, `grindstone`, `smoker`, `cauldron`, `tripwire_hook` exist in `blocks.tsv`.
  - All water blocks (`water[level=0]`) will sit strictly inside stone basins closed on all four sides and below (the ravine pool at `y = -60` and the rainwater basin on the north terrace).
- **Written files in `C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\`:**
  - [design.md](C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\design.md): site story, rationale for each mass, 7-station walk with zero-jump continuity (`y = -60` road entry climbing via 0.5-block slab steps to `y = -52` plateau/floor), ASCII site/main/roof plans, two ASCII sections (`x = 420` and `z = -6`), verified 26.2 palette, and render view list.
  - [views.txt](C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\views.txt): 3 isometric views, 2 plans (`top`, `-51`), 2 elevations (`N`, `W`), 2 sections (`x 420`, `z -6`), and 8 station eye views (covering Stations 1–7 by day and night).
- **Ready for Pass 2:** landform, spur, ravine, road, broken bridge abutments, and plain-massing volumes in <=4 materials.
