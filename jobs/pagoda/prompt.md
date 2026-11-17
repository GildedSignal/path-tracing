# A pretty pagoda next to spawn

**Julien (S195, verbatim):** "have hydrogen design a pretty pagoda in my map next to spawn i'm curious of what it can do, then report usage used for it don't let it half bake it"

This is a design job, and taste matters: a pagoda he will look at in his world. Take the time it needs. Do it fully and well, not half-baked.

You read, reason and write only into `C:\Users\PASCAL\Desktop\path tracing\jobs\pagoda\`.

## The site
- **His world:** Minecraft Java 26.2, superflat. The ground's top surface is at y = −61, so players stand at feet y = −60. World spawn is (0, −60, 0).
- **Your box:** x from 20 to 52, z from −16 to 16, y from −60 up to −10 at most. That's beside spawn and clear of it. The integrator checks the box is empty before placing; stay strictly inside it.
- **Leave a clear path:** keep a walkable approach from spawn (from the west face, toward x = 20) and an entrance on that side.

## Design
- **A classical multi-tiered pagoda:** an odd number of storeys (five or seven), each smaller than the one below, with upturned eave corners, a stepped stone base or platform, columns, balconies or railings on each storey, and a finial spire on top.
- **Interior:** reachable and walkable. A stair or ladder up through the storeys to the top balcony, with light inside, so it isn't dark.
- **A small garden or courtyard** at its foot if it fits: a path, lanterns, a few plants, perhaps a small pond.
- **A coherent palette** that looks good in daylight: for example dark oak or spruce for the frame, red or crimson for the columns, deepslate tiles or a dark stair-block roof, a stone or polished-blackstone base, lanterns. Use stairs, slabs, trapdoors and walls for the upturned eaves and the details. Full cubes alone look crude.
- **Symmetric about its central axis;** a square or octagonal plan is your choice.

## Hard constraints
1. **Every block id and state must exist in 26.2.** The authoritative list is `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv`; check each id there. Use correct states for stairs (`facing`, `half`, `shape`), slabs (`type`) and lanterns (`hanging`). A block that isn't in the list is not allowed.
2. **Commands only:** `/fill x1 y1 z1 x2 y2 z2 <block>[states]` and `/setblock x y z <block>[states]`, one per line, absolute coordinates inside the box. No other commands, no `replace` filters, no structure blocks.
3. **Each `/fill` holds at most 32,768 blocks,** the game's default limit.
4. **Order:** structure first, then details, then the interior and the lights.

## Write
1. **`jobs\pagoda\build.txt`:** the commands, in order. Lines starting with `#` are comments naming each part (base, storey 1, eaves …).
2. **`jobs\pagoda\design.md`:**
   - the design: dimensions, storeys, the palette and why;
   - **an ASCII plan of each storey** and **an ASCII side elevation**, so the design can be reviewed before it's placed;
   - a self-check: every block id was verified against `blocks.tsv`, every coordinate is inside the box, and every `/fill` is within the limit;
   - the counts of commands and blocks.

Iterate until you would be proud to show it. Before finishing, re-read your own build against the elevation and fix any mismatch: a missing eave corner, a floating block, a stair facing the wrong way.
