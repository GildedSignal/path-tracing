# Pass 3 Report — The Last Hearth (Small): Structure and Envelope

## 1. Inspection of Pass 2 Views (Before Any Changes)

Inspected all 19 rendered images in `views/pass-2/`:
- **What reads well:**
  - **Two-period building hierarchy (`iso-NE-day`, `iso-NW-day`, `iso-SW-day`, `elev-N`, `elev-W`):** The stone core with its dominant N–S `deepslate_tiles` gable (`y = -46..-40`) and tall chimney (`y = -37`) clearly commands the composition above the smaller, lower E–W `spruce_planks` wing (`y = -48..-45`) and southern porch canopy (`y = -49`).
  - **Chasm and broken bridge framing (`s3-bridge`, `section-z--6`, `elev-N`, `elev-S`):** The 6-block ravine gap (`x = 426..431`) drops cleanly to `y = -61` between the west (`x = 420..425`) and east (`x = 432..435`) `stone_bricks` abutments. In `s3-bridge`, looking east from the west abutment frames the broken east abutment and abandoned pass between the higher SE cliff (`y = -41`) and stepped-back NE shoulder (`y = -46`).
  - **Approach and grounding (`s1-approach-day`, `s2-arch`, `s4-porch`, `plan-top`):** The climbing western road is completely unburied, curves organically past the SW talus apron up toward the gatehouse bay (`s2-arch`), and meets the stone-brick terrace and west curb cleanly (`s4-porch`).
- **What reads badly / needs Pass 3 work:**
  - **Blocky uncarved masses and plain roofs:** Every roof is solid blocky cubes (`deepslate_tiles` and `spruce_planks`), the gatehouse bay (`x = 417..419, z = -1..2`) is still solid brick blocking the road beneath it (`s2-arch`), and the interiors (`s5-vestibule`, `s6-hearth`, `s7-alcove`) are solid fill (completely black/stone-brick).
  - **Stepped-stone monotony and flat superflat horizon:** The spur, SW crag, and east cliffs are all uniform grey `stone` terraces with zero material differentiation (no `tuff`, `andesite`, `gravel` talus, `moss_block` in damp shade, or `grass_block`/`podzol`/alpine spruces). In `s1-approach-day` and `s4-porch`, the bare superflat grass edge is visible in the left background.
  - **Road climb uses full 1-block steps:** Currently each road step rises a full block (`y = -61` to `-53`), which requires jumping; Pass 3 must make every step climbable without a jump (`0.5`-block slabs / stairs).
  - **Night views (`iso-SW-night`, `s1-approach-night`):** Entirely dark because no windows or lanterns exist yet—Pass 3 must open the warm landmark windows and place the exterior porch lantern (or prepare the envelope openings cleanly so light reads).
  - **East bridge abutment & chasm floor:** In `s3-bridge`, the east abutment (`x = 432..435`) sits as a tidy flat rectangle on a bare grey `stone` chasm floor (`y = -61`), without broken arch ribs, fallen masonry, or the enclosed mountain pool (`y = -60`).
- **What surprised me:**
  - Looking closely at `elev-W` and `elev-N`, the spruce wing's E–W gable ends (`x = 410`) overhang 1 block west of the spruce wing wall (`x = 411`) cleanly, while on the east (`x = 416`) the spruce gable meets the stone core wall (`x = 417`) directly. Keeping `spruce_stairs` eaves at `x = 410..416` with exposed `spruce_log` corner posts and ridge beam will give crisp depth without colliding with the stone wall.
  - In `s4-porch`, the camera (`416 -52 -2.5 yaw 115`) stands right against the east end of the porch looking WNW across the flat grass edge; adding a wind-bent alpine spruce around `x = 404..406, z = -7..-5` and rocky shoulder framing plus shifting/framing the porch door and window will anchor that view.

---
## 2. Render 1 (Initial Envelope, Roofs, Openings, Zero-Jump Road & Pool)

**Files:** `pass-3.txt` (joined with `pass-2.txt` -> `so-far.txt`), rendered to `views/pass-3/` (3.188 s, `Plain-cube fallbacks: []`, `Missing textures: []`).

### What Reads Well
- **Interiors & spatial compression/release (`s5-vestibule`, `s6-hearth`, `section-z--6`, `section-x-420`):** The 3-block-high timber vestibule (`y = -52..-50`) with dark ceiling joists and warm lantern glow leads cleanly through the thick oak-silled stone threshold (`x = 417`) into the tall 9-block Hearth Room (`y = -52..-41`) with exposed collar ties and the 2-block East Alcove opening.
- **Zero-jump road & Gatehouse Arch (`s1-approach-day`, `s2-arch`):** The alternating full-block / bottom-slab climbing steps (`+0.5` block per `x`) climb smoothly into the vaulted 3-wide stone Toll Arch (`x = 417..419, z = -1..1`), while the porch window & lantern warm the left side of `s2-arch`.
- **Ravine pool & night landmark (`iso-NW-day`, `iso-SW-night`, `section-z--6`):** The closed water basin (`y = -60`) sits naturally at the bottom of the dark `tuff`/`cobbled_deepslate` chasm with fallen bridge voussoirs, and at night (`iso-SW-night`) the porch lantern casts a restrained warm pool of light under the timber canopy.

### What Reads Badly / Surprised Me
1. **Incomplete storm-repair roof patch (`elev-W`, `iso-NW-day`):** On the SW roof slope (`x = 418, y = -44, z = -3`), `deepslate_tiles` remained beneath `spruce_stairs` while `z = -4` was switched to `spruce_planks`, creating a notched bite in the repair patch.
2. **Stark dark-under-light cliff banding & horizontal `podzol` stripes (`iso-NE-day`, `iso-NW-day`, `elev-N`, `elev-S`):** Horizontal `podzol` ledges along cliff sides draw bright brown stripes across grey `stone`, and on the NE corner (`x = 425, z = -11..-1`) the light-grey `y = -55..-53` stone terrace overhangs dark `tuff`/`cobbled_deepslate` and the undercut chasm face (`z = 0..1`), making the terrace look like a cantilevered concrete slab.
3. **Boxy lollipop trees & bare flat west spur nose (`elev-W`, `s1-approach-day`):** The 4 alpine spruces use uniform `3×3` flat leaf layers (`elev-W`), and in `s1-approach-day` the flat grey `stone` step at `x = 402..405, z = -1..2` juts straight into the superflat meadow.
4. **East bridge abutment shows raw `stone` behind undercut (`s3-bridge`):** Carving `x = 432, z = 0..1, y = -60..-56` to `air` exposed raw `stone` at `x = 433` behind the `stone_bricks` arch ribs, and `s3-bridge` (`pitch 12` from `x = 422`) sits too far back to see down into the ravine water pool.
5. **Porch view shows superflat grass horizon & recessed door (`s4-porch`):** Looking WNW from `416 -52 -2.5` still shows bare superflat grass left of the west terrace curb (`Tree 1` at `z = -8` was too far north to frame it), and `spruce_door[facing=south]` sits recessed on the inner edge of `z = -4` out of view.
6. **Alcove window stares at a blank grey cliff wall (`s7-alcove`):** Looking straight east (`yaw 270`) from `422.5 -52 -6.5` hits plain `stone` steps (`x = 432..437, z = -10..-4`) without framing the mossy ledges, alpine spruce, chasm drop, or far pass.
7. **Vestibule lantern hangs in the doorway sightline & `s1-approach-night` window hidden (`s5-vestibule`, `s1-approach-night`):** The vestibule lantern `(415, -50, -6)` hangs right across the upper stone threshold, and at `s1-approach-night` the west landmark window (`411, -51, -6`) is partially screened by the `y = -52` curb at `x = 409, z = -6`.

---
## 3. Render 2 (Material & Cliff Fixes, Flush Door, Alcove & Bridge View Refinements)

**Changes made:**
- Completed the SW storm-repair roof patch (`418 -44 -3` -> `spruce_planks`).
- Removed horizontal `podzol` cliff stripes, blended vertical `tuff`/`andesite` transitions on the NE headland, and backfilled solid `stone_bricks`/`tuff_bricks` behind the bridge pier undercuts.
- Resculpted all 4 alpine spruces with asymmetric `+`-shaped tiers; moved Tree 1 to `(405, -6)` and Tree 4 to `(436, -8)` facing the East Alcove across a mossy rock ledge.
- Set `spruce_door` to `facing=north` (flush on the porch wall face), shifted the vestibule lantern to `(414, -50, -7)` out of the threshold sightline, and added a hearth lantern at `(420, -46, -5)`.
- Adjusted `s3-bridge`, `s4-porch`, and `s7-alcove` angles in `views.txt`.

**Files:** `pass-3.txt`, `views.txt`, rendered to `views/pass-3/` (3.858 s, `Plain-cube fallbacks: []`, `Missing textures: []`).

### What Reads Well
- **Repaired roof patch & NE corner (`elev-W`, `iso-NE-day`):** The `2×3` spruce/oak repair patch on the slate roof is solid and watertight, and the NE corner now reads as grounded alpine rock rather than a cantilevered slab.
- **Vestibule, door & alcove view (`s5-vestibule`, `s4-porch`, `s7-alcove`):** Moving the vestibule lantern north opens the threshold sightline cleanly; `s4-porch` clearly shows the vertical planks and iron hardware of the low `spruce_door` beside the copper lantern and glazed window; and `s7-alcove` now frames a mossy fern ledge, alpine spruce, and the east pass instead of a grey wall.

### What Reads Badly / Surprised Me
1. **Floating `andesite` boulder on the west approach (`s1-approach-day`):** Filling `407..408, -56..-54, -4..-2` left `407, -57, -4..-2` as air above the `-58` slope, creating a visible hanging rock block, and `404, -60, 1` left a 1-block pit in the foreground grass bench.
2. **Floating Hearth Room lantern (`section-x-420`):** Placing the hearth lantern at `(420, -46, -5)` put it 1 block south of the `(420, -45, -4)` collar tie, leaving it hanging in mid-air.
3. **Notched pier base on east bridge abutment (`s3-bridge`):** Replacing `432, -55, 2` with an upside-down stair above dark `tuff` (`-56`) created a notch in the vertical pier face, and `pitch 20` at `x = 423.5` still hides most of the chasm water pool.
4. **Superflat green horizon still dominates left half of `s4-porch`:** At `yaw 118`, the left half of the FOV looks past `(409, -2)` out to the flat green superflat plain; Tree 1 at `(405, -6)` sits too far north to screen it.

---
## 4. Render 3 (Grounded West Slope, Solid Bridge Piers, Anchored Lantern & Framed Porch)

**Changes made:**
- Grounded the west approach boulders (`406..408, -60..-54, -4..-2`) solidly down to `y = -60` and cleaned the foreground meadow transition (`402..405`) so there is no hanging rock or missing pit.
- Kept both outer bridge piers (`x = 425` and `x = 432` at `z = -1, +2`) solid `stone_bricks`/`tuff_bricks` down into the pool (`y = -60`), moving the curved arch springing stairs strictly to the center bay (`z = 0..1, y = -55`) and projecting corbels (`x = 426, 431`).
- Anchored the Hearth Room lantern at `(420, -44, -4)` directly under the full `(420, -43, -4)` `stripped_spruce_log` collar tie, and added a provisional south-window lantern at `(421, -52, -3)` for night landmark checks.
- Shifted Tree 1 to `(405, -4)` and adjusted `s3-bridge` (`424 -52 0.2 270 28 78`) and `s4-porch` (`415.5 -52 -2.2 132 -2 75`) in `views.txt`.

**Files:** `pass-3.txt`, `views.txt`, rendered to `views/pass-3/` (3.410 s, `Plain-cube fallbacks: []`, `Missing textures: []`).

### What Reads Well
- **Approach & porch grounding (`s1-approach-day`, `s4-porch`):** The hanging boulder is gone, the climbing half-slab road leads smoothly past the mossy curb into the Gatehouse Arch, and `s4-porch` now frames the `spruce_door`, porch window, weathered copper lantern, slender post, west curb, and Tree 1's alpine foliage cleanly.
- **Broken bridge & chasm drop (`s3-bridge`):** Both east abutment piers now stand solidly in the dark pool with fallen bridge voussoirs visible below, curved arch ribs above, and the abandoned pass rising behind.
- **Night landmark & interior clarity (`iso-SW-night`, `s5-vestibule`, `s6-hearth`, `section-x-420`):** Both the sheltered porch and the south bridge-glance window glow warmly at night (`iso-SW-night`), the Hearth Room lantern hangs solidly from the collar tie (`section-x-420`), and all three interior stations (`s5`, `s6`, `s7`) are open, watertight, and legible.

### What Reads Badly / Surprised Me
- **Minor foreground stone slab (`s1-approach-day`):** A single `andesite_slab` at `(403, -60, -2)` sits isolated on flat meadow grass left of the road; Pass 4 or 5 can swap it for a tuft of `fern` or tuck it against the spur base.
- **Provisional Hearth light blocks (`s6-hearth`, `s1-approach-night`):** The two bare `ochre_froglight` blocks on the Hearth Room floor `(420..421, -52, -9)` serve Pass 3's structural night checks and await Pass 4's proper stone hearth hood, iron bars, and fireplace fender; from `s1-approach-night`, the west window `(411, -51, -6)` remains screened by the spur/Tree 1 while the porch canopy and south bridge window carry the night beacon.

---

## 5. Pass 3 Summary

### Command Count
- **`pass-3.txt`:** `414` commands (`160` `/fill`, `254` `/setblock`), all absolute coordinates inside `x = 400..440, y = -61..-16, z = -20..20`.
- **Combined so far (`pass-2.txt` + `pass-3.txt`):** `514` commands (`100` + `414`).

### Pass 2 Blocks Replaced by Pass 3 (and Why)
- **Ravine floor, cliffs, spur slopes & talus (`stone`):** Replaced exposed uniform `stone` with `cobbled_deepslate` and `tuff` in the wet chasm floor/walls, `moss_block`/`fern`/`moss_carpet` in damp shade, `andesite`/`andesite_slab`/`andesite_stairs` on crag ridges, `gravel`/`coarse_dirt` on the landslide talus apron, and `grass_block`/`podzol` at meadow edges and tree bases.
- **Climbing road (`stone_bricks`):** Replaced full-step blocks at alternating `x` coordinates with `stone_brick_slab[type=bottom]` / `andesite_slab[type=bottom]` (`+0.5` block per step, zero jumps), cleared a 2-wide bypass at `x = 410..411, z = 0..1`, and dressed retaining curbs with `stone_brick_wall` / `mossy_stone_brick_wall`.
- **Bridge abutments & chasm (`stone_bricks`, `air`):** Undercut the center chasm faces (`z = 0..1, y = -60..-56`) to carve broken arch springings while keeping solid `stone_bricks`/`tuff_bricks` pier walls at `z = -1, +2`, added protruding `stone_brick_stairs` arch ribs and broken `stone_brick_wall` parapets, and placed the enclosed `water[level=0]` pool (`y = -60`, bounded by `tuff`/`cobbled_deepslate` dams at `z = -14` and `z = +12`) with fallen voussoirs.
- **Gatehouse Bay, Stone Core, East Alcove & Spruce Wing (`stone_bricks`, `spruce_planks`, `deepslate_tiles`):** Hollowed the vaulted Gatehouse Arch (`y = -52..-49`), Hearth Room (`y = -52..-41`), East Alcove recess (`x = 424..425, z = -7..-6`), and 3-block-high Vestibule (`y = -52..-50`); replaced blocky roofs with pitched `deepslate_tile_stairs`/`slab`, `spruce_stairs`/`slab`, exposed interior collar ties, a `2×3` spruce/oak storm-repair patch, and a soot-stained chimney crown; installed the flush `spruce_door` (both halves), glazed `light_gray_stained_glass_pane` windows, porch `spruce_fence` posts, and 4 asymmetric alpine spruces (`spruce_leaves[persistent=true]`).

### What Pass 4 Must Do (Rooms, Light and Traces)
1. **Build the real Hearth & Chimney Hood (`x = 419..422, z = -10..-9`):** Replace the two provisional `ochre_froglight` blocks at `(420..421, -52, -9)` with a proper stone hearth surround (`tuff_bricks`, `stone_bricks`, `iron_bars`, hidden/recessed `ochre_froglight` + `magma_block` ember bed, one mossy dressed stone salvaged from the broken bridge, and a projecting stone/dark-oak mantel hood).
2. **Furnish the Kitchen Corner, Workspace, Sleeping Alcoves, Vestibule & Window Seat:**
   - **Porch & Vestibule:** Porch bench beside the low door (`x = 413..414, z = -3`), vestibule gear bench (`x = 412..413, z = -5`), pack hook (`tripwire_hook`), dry boot shelf/chest (`barrel`/`chest`, empty), and dry bunks + bookshelf divider in the north half of the spruce wing (`z = -8..-7`).
   - **Hearth Room:** Kitchen corner (`smoker`, `cauldron`, prep counter, empty `barrel` on `x = 418, z = -9..-8`), communal hearth table (`x = 420..421, z = -7..-6`), maintenance tool bench (`crafting_table`, `grindstone`, `cartography_table`), and **empty `lectern`** (`x = 420, z = -4`) for travellers' messages.
   - **East Alcove:** Fitted `spruce_slab`/`spruce_stairs` cushioned window seat (`x = 424, z = -7..-6`) beneath the 2-block window.
   - **North Eaves Exterior (`z = -12..-11`):** Stacked firewood (`spruce_log` / `stripped_spruce_log`) and closed rainwater trough (`stone_brick_stairs`/`slab` basin holding `water[level=0]` closed on every side and below).
3. **Replace provisional Pass 3 lights & add lived-in traces:** Replace the floor lantern `(421, -52, -3)` with a proper shelf/table lantern, refine interior lighting (subdued *In Praise of Shadows* warmth—hearth glow, vestibule lantern, alcove reading candle/lantern, porch copper lantern), and add directional moss/soot/worn threshold details.

### Changes Recommended for the Plan (`design.md` & `views.txt`)
- **`design.md` East Alcove cheek (`z = -8`):** Update the alcove projection span in `design.md` from `z = -7..-5` to `z = -8..-5` so both ends of the two-block window (`z = -7..-6`) have solid stone piers (`z = -8` and `z = -5`) to anchor the glass panes.
- **`design.md` Spruce Wing interior wall (`x = 416`):** Note that omitting a duplicate plank wall at `x = 416` exposes the original exterior stone core wall (`x = 417`) inside the timber vestibule, giving both more room (`x = 412..416`) and visible construction history.
- **`views.txt`:** Keep the refined `s3-bridge` (`424 -52 0.2 270 28 78`), `s4-porch` (`415.5 -52 -2.2 132 -2 75`), and `s7-alcove` (`422.8 -52 -6.5 262 5 72`) angles; in Pass 4, consider adding a north-eaves view or interior detail view if needed to inspect the woodpile, rainwater basin, and sleeping bunks.
