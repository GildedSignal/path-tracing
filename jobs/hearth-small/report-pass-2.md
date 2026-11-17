# Pass 2 Report — The Last Hearth (Small): Landform and Massing

## Preparation
- Re-read `architecture-brief.md` (Section I + Sections 5–7), `taste-profile.md`, `design.md`, `views.txt`, and `report-pass-1.md`.
- Pass 1 produced the spatial plan and `views.txt`; no commands or views were rendered in Pass 1.
- Pass 2 goal: build the spur, ravine, road, broken bridge abutments, and building masses as plain volumes in at most 4 materials (no openings or details), render, inspect every view, and iterate at least twice until proportions, silhouette, ravine depth, and the 7-station walk read strongly.

## Render 1 (Initial Massing from Pass 1 Coordinates)
**Files:** `pass-2.txt` (61 commands: 60 `/fill`, 1 `/setblock`), rendered to `views/pass-2/` (3.84 s, 0 plain-cube fallbacks, 0 missing textures).

### What Reads Well
- **Ravine chasm at the crossing (`iso-SW-day`, `s3-bridge`, `section-z--6`):** The 6-block drop (`x = 426..431`) between the stone-brick west and east bridge abutments (`y = -53` down to `-61`) immediately establishes the broken crossing and the verticality of the chasm.
- **Two-part building silhouette (`iso-SW-day`, `elev-N`):** The contrast between the dark `deepslate_tiles` N–S gable + tall chimney and the lower `spruce_planks` E–W wing is legible as two distinct construction periods.

### What Reads Badly / Surprised Me
1. **Floating spruce wing (`s1-approach-day`, `section-z--6`):** The spur stepped down too early west of `x = 415`, leaving the western half of the spruce wing (`x = 410..414`) hanging over air at `y = -52`.
2. **East cliff is a monolithic slab wall (`iso-NE-day`, `iso-SW-day`, `section-z--6`):** Filling `x = 432..440` to `y = -47..-41` across all `z = -20..20` turned the east bank into a boxy grey fortress wall that blocks `iso-NE-day`. Worse, carving the abandoned trail inside it created an unintended rock tunnel/pillar (`section-z--6`).
3. **Spruce wing competes too strongly with the dominant roof (`elev-W`, `iso-SW-day`):** With walls to `y = -47` and ridge to `-43`, the spruce wing sits nearly as high and bulky as the stone core (`y = -40` ridge). It should be visibly lower and tighter so the main slate roof clearly dominates.
4. **Stone gable faces poke through the slate roof (`iso-NE-day`, `iso-SW-day`, `plan-top`):** Setting `stone_bricks` at `z = -10` and `z = -2` on the same `x` width as the stepped `deepslate_tiles` created grey vertical stripes cutting out through the roof slope.
5. **Spur & road step too steeply at the trailhead (`s1-approach-day`, `plan-top`):** The spur jumped in 2-block cliffs beside the trailhead, hiding the road climb behind a blank stone wall, and left stray green grass patches inside the south spur (`plan-top`, `plan--51`).
6. **Camera framing at Stations 2 and 4 (`s2-arch`, `s4-porch`):** `s2-arch` (`415 -53 0`) and `s4-porch` (`414 -52 -2`) sit 1.5 blocks from uncarved solid mass walls in Pass 2, filling the frame with flat brick/plank texture. (`s5-vestibule`, `s6-hearth`, `s7-alcove` are also inside solid uncarved masses as expected for Pass 2.)

## Render 2 (Grounded Foundation, Organic East Pass, Dominant Roof Ratio)
**Changes made:**
- Extended the `stone` headland foundation cleanly under the spruce wing (`x = 410..425, z = -11..2`), eliminating the floating wing and stray grass gaps.
- Replaced the monolithic east cliff with twin crag shoulders flanking a V-shaped mountain-pass saddle behind the east bridge abutment (`z = -3..3`).
- Scaled the spruce wing lower (`y = -52..-49` walls, `-45` ridge) beneath the dominant stone core (`y = -52..-46` walls, `-40` ridge) and tucked the stone gable infills strictly inside the slate roof slope.
- Updated `views.txt`: added `iso NW day` and `elev S`; adjusted `s1`, `s2`, `s4` sightlines.

### What Reads Well
- **Building hierarchy & silhouette (`iso-NE-day`, `iso-NW-day`, `iso-SW-day`, `elev-N`):** The stone core + steep deepslate roof + centered chimney now clearly dominate the lower spruce wing, and zero stone stripes poke through the roof slope.
- **No floating volumes (`section-z--6`, `elev-S`):** The spruce wing rests solidly on the stone terrace.
- **Saddle pass at the broken bridge (`s3-bridge`, `iso-SW-day`):** Looking east from the west abutment now reveals the east abutment framed by twin crags opening onto a pass to the sky.
- **Porch canopy & depth (`s4-porch`):** Standing at the east end of the porch clearly shows the stone-brick terrace, timber wall, and projecting spruce canopy.

### What Reads Badly / Surprised Me
1. **East cliff shoulders look like twin ziggurats (`iso-NE-day`, `iso-NW-day`, `iso-SW-day`):** The two rectangular stepped boxes (`x = 433..440, z = -14..-4` and `z = 4..14`) look overly symmetrical and geometric, like two matching pyramids flanking the pass. The south-west crag (`x = 407..417, z = 6..16`) also reads a bit pyramid-like in `iso-NW-day` and `iso-SW-day`.
2. **`s2-arch` (`412 -54 1`) is inside a stone block!** Because the road at `x = 412..413` is at `y = -55` top (`-54` feet), WAIT—no! The landslide talus or terrace at `x = 412, z = 1` had `stone`? Let's check: at `x = 412, y = -54 + 1.62 = -52.38` -> `y = -53`! Look at `pass-2.txt`: `/fill 410 -60 -11 425 -53 2 minecraft:stone` filled `x = 410..425, z = -11..2` all the way up to `y = -53`! So the road at `x = 412..415` was BURIED inside the `-53` headland stone plateau! That's why `s2-arch` was solid grey inside a `stone` block and why the road disappeared in `plan-top` between `x = 410` and `416`!
3. **West spur retaining wall blocks `s1-approach-day` (`s1-approach-day`, `elev-S`):** At `x = 408..409`, the spur steps up straight across `z = -10..+4` right next to the trailhead, so `s1-approach-day` still stares mostly at a grey stone cliff in the foreground while the road climbs invisibly behind/beside it.
4. **Spruce wing south-west corner / gatehouse junction (`elev-S`, `elev-W`):** In `elev-S` (`x = 410..425`), look how well the gatehouse arch bay projects in front of the stone core, while the spruce wing roof eave (`y = -48`) sits comfortably below the main roof eave (`y = -46`). However, on the north side (`elev-N`), the spruce wing's east end meets the stone core cleanly, while on the west (`x = 410`) the terrace drops abruptly at `x = 409`.
5. **Porch view (`s4-porch`) sees flat superflat grass horizon on the left:** Because the west spur drops away north-west of the porch (`z = -3`), looking WNW from `416 -52 -2.5` shows empty green flatland right beside the porch. A low stone terrace parapet or stepped rocky shoulder on the west end of the terrace (`x = 409..410`) plus a slightly angled view (`yaw 125`) will keep the mountain spur grounded in frame.

## Render 3 (Unburied Climbing Road, Asymmetric East Crags, West Shoulder)
**Changes made:**
- Restricted the `y = -53` headland plateau to `z = -11..-1` (plus `x = 416..425, z = 0..2`) and carved clean headroom above the climbing road (`x = 400..415`) so the entire stone-brick roadbed is exposed.
- Kept the NW spur slope below road height along the trailhead sightline (`s1-approach-day`) and added a rocky stone shoulder (`x = 408..409, z = -11..-3`) + low terrace curb (`x = 409, z = -3..-2`) at the west end of the spruce wing.
- Replaced the twin symmetrical east-cliff pyramids with an asymmetric higher SE ridge (`y = -41`) and stepped-back NE shoulder (`y = -46`), and shaped the SW crag as a diagonal ridge (`y = -46`).

### What Reads Well
- **Approach sightline (`s1-approach-day`):** The climbing stone-brick road, the rocky spur, the lower spruce wing, and the taller slate-roofed stone core now all read together from Station 1.
- **Road to gatehouse (`s2-arch`):** Station 2 now stands cleanly on the climbing road looking up toward the stone gatehouse mass and spruce wing eaves.
- **Porch grounding (`s4-porch`):** The low stone-brick curb at the west edge of the terrace frames the porch floor instead of ending abruptly over open grass.
- **Overall silhouette (`iso-NE-day`, `iso-NW-day`, `iso-SW-day`, `elev-N`, `elev-S`, `elev-W`):** Asymmetric terrain frames the ravine and pass naturally; `iso-NE-day` now sees directly down into the chasm and onto the East Alcove mass.

### What Reads Badly / Surprised Me
1. **Floating rock shelf in the east pass (`s3-bridge`, `iso-NE-day`, `elev-W`):** `/fill 438 -49 1 440 -49 3` extended to `z = 3`, while the layer underneath (`/fill 438 -51 -4 440 -50 2`) stopped at `z = 2`, leaving a 1-block floating stone overhang with daylight underneath in `s3-bridge`.
2. **Minor curb/spur step irregularity near trailhead (`s1-approach-day`, `plan-top`):** At `x = 404..405, z = 5`, the south retaining curb (`z = 6`) sits detached by 1 block from the road (`z = 2..4`), and a shallow 1-block notch sits on the west spur face (`x = 408`).

## Render 4 (East-Pass Support Fix & Road Curb Polish)
**Changes made:**
- Supported the `z = 3` east-saddle stone ledge down to `y = -51..-50` (`x = 438..440`), removing the floating overhang and daylight gap visible in `s3-bridge` and `elev-W`.
- Tightened the south road retaining curb (`x = 402..409`) to hug the road curve without leaving a detached gap at `x = 404..405`.

### What Reads Well
- Every view (`iso-NE-day`, `iso-NW-day`, `iso-SW-day`, `plan-top`, `plan--51`, `elev-N`, `elev-S`, `elev-W`, `section-x-420`, `section-z--6`, and exterior stations `s1-approach-day`, `s2-arch`, `s3-bridge`, `s4-porch`) is grounded, watertight, and cleanly legible.
- The dominant `deepslate_tiles` N–S gable (`y = -46..-40`) and chimney (`y = -37`) clearly rule the composition above the lower `spruce_planks` wing (`y = -48..-45`), anchored between the SW crag (`y = -46`) and the asymmetric east ravine cliffs (`y = -41` SE, `-46` NE).
- Zero plain-cube fallbacks and zero missing textures in `views/pass-2/notes.txt`.

---

## Pass 2 Summary

- **Command count (`pass-2.txt`):** `100` commands total (`92` `/fill`, `8` `/setblock`), all absolute coordinates inside `x = 400..440, y = -61..-16, z = -20..20`.
- **Materials used (4 solid materials + `air`):**
  1. `minecraft:stone` — rocky spur, crag, landslide talus, ravine bed, and asymmetric east cliffside.
  2. `minecraft:stone_bricks` — climbing roadbed, retaining curbs, terrace paving, west/east bridge abutments, stone tollhouse core, gatehouse mass, chimney, and east alcove projection.
  3. `minecraft:spruce_planks` — later spruce wing walls, lower cross-gable roof, and southern porch canopy.
  4. `minecraft:deepslate_tiles` — dominant N–S gable roof over the stone core.
  (`minecraft:air` used only to keep headroom clear above the climbing road.)

### What Pass 3 Must Do (Structure and Envelope)
1. **Hollow the interior volumes & carve the vaulted Tollhouse Arch (`x = 417..419, z = -1..2`):** Open the 3-block-high carriageway arch under the gatehouse, carve the interior rooms (low vestibule `y = -52..-50` inside the spruce wing, taller hearth room rising into the rafters inside the stone core, stone threshold `x = 416..417`, and east alcove recess `x = 424..425, z = -7..-5`) so Stations 5, 6, and 7 open up.
2. **Replace blocky roof volumes with real stairs, slabs, eaves, and rafters:** Build the dominant roof in `deepslate_tile_stairs` and `deepslate_tile_slab` (including a newer spruce/oak repair patch on one slope), the smaller wing roof and porch canopy in `spruce_stairs`/`spruce_slab`, and shape the chimney crown and parapet.
3. **Carve & dress openings and carpentry:** Add the low spruce door (`x = 412, z = -4`), the two-block east alcove casement (`x = 425, z = -7..-6`), the south bridge-glance window (`x = 420..421, z = -2`), warm west/porch landmark windows, timber porch posts, and roof rafters.
4. **Upgrade road, steps, terrain materials, and broken bridge:** Convert the climbing road into zero-jump `stone_brick_slab` / `andesite_slab` steps, differentiate cut `tuff` / `andesite` / `gravel` talus from dressed `stone_bricks`, add fallen bridge arch ribs and pier teeth at the chasm lip, and place the enclosed ravine pool (`water[level=0]` at `y = -60` closed on all sides) plus wind-bent alpine spruces (`persistent=true`).

### Changes Recommended for the Plan (`design.md` & `views.txt`)
- **Spruce wing footprint & height:** In `design.md`, the spruce wing works much better slightly tighter and lower (`x = 411..416, z = -9..-4`, walls `y = -52..-49`, ridge at `y = -45` instead of `y = -44` over `410..416, z = -10..-4`) so the dominant slate roof (`y = -46..-40`) commands the silhouette.
- **Asymmetric east cliff:** Keep the SE cliff higher (`y = -41`) and the NE shoulder lower/stepped back (`y = -46`) flanking the diagonal saddle (`z = -3..3`) so the gorge feels natural rather than walled-in.
- **Views (`views.txt`):** Already updated `views.txt` with `iso NW day`, `elev S`, and improved exterior station coordinates (`s1` `yaw 246`, `s2` at `412 -54 1`, `s4` at `416 -52 -2.5 yaw 115`). Keep checking `s5`, `s6`, `s7` in Pass 3 once the interiors are carved.
