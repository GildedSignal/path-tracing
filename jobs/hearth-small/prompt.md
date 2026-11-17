# The Last Hearth, small: a refuge built in five passes

**Julien (S205, verbatim):** "ok that was too much for hydrogen to handle, based on my taste and the previous prompts write a new prompt that's more sized like the pagoda with mulitple passes in how it is builted that would be better."

Your pagoda for him (`C:\Users\PASCAL\Desktop\path tracing\jobs\pagoda\`, 797 commands) was very good: he made it the benchmark for the swarm's building (S197). Then three builds of 535,000 to 7.6 million blocks, written as generators, came out too big to be good. **This build is the size of the pagoda, written by hand, block by block, in five passes, and you see your work between passes.**

## The brief
- **Read in full:**
  - `D:\path-tracing-evidence\three-builds\architecture-brief.md`, section "I. The Last Hearth" above all, and sections 5–7;
  - `D:\path-tracing-evidence\three-builds\taste-profile.md`, his taste.
- **Both are private** (the repository is public). Never copy their text into a file under `C:\Users\PASCAL\Desktop\path tracing\`. Your notes may name ideas.
- **The idea, at this size:** on a rocky spur at the edge of a ravine, the stub of an old stone road ends at a broken bridge. Beside the bridge's surviving abutment stands a small refuge:
  - a stone core from the old toll post;
  - a later spruce extension;
  - one dominant roof and one smaller added one;
  - a warm window that is its landmark at night;
  - a low later door, a vestibule, a hearth room, and an alcove whose seat and two-block window look into the far side of the ravine.
- **The register is restraint:** Frieren, Bashō, *In Praise of Shadows*. Someone has cared for this place for longer than anyone remembers, without being thanked.
- **Details are consequences of a use, a material or a period of construction:** a repaired threshold, worn steps, a newer roof timber, soot above the chimney, moss where it is damp and shaded. They are never scattered texture.

## Your box
Superflat overworld, Minecraft Java 26.2.
- **Box:** x 400..440, z −20..20, y −61..−16. The ground's top surface is y −61 (players stand at y −60), and superflat has no relief: the spur and the ravine are yours to make inside the box. The ravine may be built up from y −61, since the ground can't be dug below −63.
- **Approach:** the road arrives from the west, entering the box at x = 400.
- The integrator checks the box is empty before placing; stay strictly inside it.

## Hard constraints
1. **Every block id and state exists in 26.2.** Check each id in `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv`. Use the correct states for stairs (`facing`, `half`, `shape`), slabs (`type`), lanterns (`hanging`), logs (`axis`), trapdoors, doors and double plants (both halves).
2. **Leaves are `persistent=true`.** Falling blocks rest on support.
3. **No lava or fire.** Water is only sources in basins closed on every side and below.
4. **No entities and no block-entity data.** Chests, barrels and lecterns are placed empty, for travellers to fill.
5. **Commands only:** `/fill` and `/setblock` with absolute coordinates inside the box. No `replace`; each `/fill` holds at most 32,768 blocks.
6. **No generator.** Write the commands by hand, so that you decide every block. Aim for the pagoda's order of size: about 600 to 2,500 commands in all.

## The five passes
Each pass is a separate run. **Between passes, the integrator renders your commands** with a preview renderer: isometric views by day and by night, plans, elevations, sections, and first-person views from eye positions you choose. It puts the images in `jobs\hearth-small\views\pass-<n>\`, with a `views.md` index. The next pass begins by looking at them.

| Pass | What it does | Writes |
|---|---|---|
| **1. The idea and the plan** (this run) | the site's story; the reason for each mass; the walk as 6 to 8 stations, each with its coordinates and what the visitor sees and feels there; ASCII plans of the site and each level; two ASCII sections; the palette, each block with its reason and its id checked; **the views to render**: for each station, an eye position (x y z yaw pitch) and whether by day or night, plus any plan or section cuts you want | `design.md`; `views.txt`, in the renderer's format, below. **No commands yet.** |
| **2. Landform and massing** | the spur, the ravine, the road and the bridge's surviving abutment, and the building as plain volumes in at most four materials, without openings or details. Judge the proportions and silhouettes in the views. | `pass-2.txt` |
| **3. Structure and envelope** | walls and frame, the roofs in stairs and slabs (eaves, ridges, the chimney), the openings, the broken bridge, the steps and paths | `pass-3.txt` |
| **4. Rooms, light and traces** | the vestibule, the hearth, the kitchen corner, the sleeping alcoves, the window seat; lanterns where people live and work; the traces of time and use | `pass-4.txt` |
| **5. Critique and repair** | look at every view, by day and by night. List the ten weakest things against the brief's tests (section 6) and his taste, fix each, and say what you fixed. | `pass-5.txt` (corrections, which may carve earlier passes); then `build.txt` = passes 2–5 concatenated |

**In every pass from 2 on:**
- Open the previous pass's images and write what they show in `report.md` before you change anything: what reads well, what reads badly, and what surprised you.
- Change `views.txt` if a new view would help you judge.
- Re-read your commands against your plans: a floating block, a stair facing the wrong way, a door without its top half, an opening that doesn't line up with its room.
- Keep the passes in order: a later pass may replace earlier blocks, and must say which ones and why.

**`views.txt` format** (one view per line):
- `iso NE day`, `iso SW night`;
- `plan top`, `plan -58`;
- `elev N`;
- `section x 420`;
- `eye <x> <y> <z> <yaw> <pitch> <fov> <day|night> <name>`. Yaw is Minecraft's: 0 faces +z (south), 90 faces −x (west), 180 faces −z, 270 faces +x. y is the feet; the renderer adds the eye height.

## This run is pass 1
Read the brief and his taste in full. Read the pagoda's `design.md` and `build.txt` to see what worked, at this scale, in your own hand. Then write `design.md` and `views.txt` in `C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\`, and start `report.md` with what you read and decided. You read, reason and write only into that folder. Launch nothing.
