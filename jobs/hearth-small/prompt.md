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
Each pass is a separate run. **You render your own commands** with the preview renderer (below): isometric views by day and by night, plans, elevations, sections, and first-person views from eye positions you choose. Render, look at the images, fix, and render again as often as you need within a pass. The images go in `jobs\hearth-small\views\pass-<n>\`, with a `views.md` index.

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

## How you render (you can run commands, in a sandbox)

Run exactly this, from any directory (PowerShell; every argument quoted):

First join the passes so far, in order (this pass: 2 and 3), then render the joined file:

```
Get-Content 'C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\pass-2.txt', 'C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\pass-3.txt', 'C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\pass-4.txt' | Set-Content -Encoding ascii 'C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\so-far.txt'
& 'D:\path-tracing-deps\data\java\java-runtime-epsilon\bin\java.exe' '-Djava.awt.headless=true' 'D:\path-tracing-wt\hearth-small\tools\preview\Preview.java' 'C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\so-far.txt' 400 -61 -20 440 -16 20 'D:\path-tracing-deps\minecraft-26.2-client.jar' 'C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\views.txt' 'C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\views\pass-4'
```

It takes about 10 seconds and writes one PNG per line of `views.txt`, plus `views.md`, into `views\pass-4\`. The earlier passes' images are in `views\pass-2\` and `views\pass-3\`. Open the PNGs with your file tool and look at them. Do not use `scripts\preview.ps1`: its JDK path is refused in the sandbox. The renderer draws unknown blocks as plain cubes and names them in `views.md`: an unknown id there is a typo to fix. You may write only in `C:\Users\PASCAL\Desktop\path tracing\jobs\hearth-small\` and `D:\path-tracing-wt\hearth-small\`. Place nothing in the game.

## This run is pass 4: rooms, light and traces

Passes 1 to 3 are done: `design.md`, `views.txt`, `pass-2.txt`, `pass-3.txt`, `report-pass-1.md` to `report-pass-3.md`, and the images in `views\pass-2\` and `views\pass-3\`. Re-read them, the brief and his taste first. Your own list of what pass 4 must do is at the end of `report-pass-3.md`, with its plan changes (the alcove's stone piers at z = -8 and -5); they apply now.

1. Open pass 3's images, day and night, and write in `report.md` what they show before you change anything.
2. Write `pass-4.txt`: the vestibule, the hearth (no fire, no lava: the warm light comes from blocks behind bars, as you planned), the kitchen corner, the sleeping alcoves, the window seat; lanterns where people live and work, and nowhere else; the traces of time and use, each a consequence of a use, a material or a period (the brief's rule), never scattered texture. Chests, barrels and the lectern stay empty.
3. Render (join passes 2–4, then render). Look at every view, above all the interior stations 5–7 and the night views: the warm window must read as the landmark, the rooms in shadow and warmth (In Praise of Shadows), not lit like a shop. Write what reads well, what reads badly and what surprised you.
4. Fix and render again, at least twice, notes per render in `report.md`. Check that nothing floats, every double block has both halves, water is closed on every side and below.
5. End `report.md` with the command count of `pass-4.txt`, what pass 5 (critique and repair) must look at first, and what you would change.

Write `report.md` as you go. Launch nothing else.
