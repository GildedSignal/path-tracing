# Three builds for Julien: what all three share

**Julien (S204, verbatim):** "alright i want you to have hydrogen build three builds for me inspired by what gpt said i used it to gather my taste and create those builds with all the details"

He used GPT to gather his taste. Read both documents in full before designing; they are the brief:
- `D:\path-tracing-evidence\three-builds\architecture-brief.md`: the three concepts, the theory, the order of design and the tests of success;
- `D:\path-tracing-evidence\three-builds\taste-profile.md`: his taste.

Both are private (the repository is public): never copy their text into any file under `C:\Users\PASCAL\Desktop\path tracing\`. Your design notes may name ideas and cite them; they may not reproduce the profile.

Your last build for him, the pagoda (`C:\Users\PASCAL\Desktop\path tracing\jobs\pagoda\`), was very good: he made it the benchmark for the swarm's building (S197). These three are bigger and deeper. **Don't half-bake them** (S195): every scale gets its full attention, from the horizon down to the hand.

## One civilization, three sites
The three builds are one world (the brief, section 4). They sit side by side in his world, in **his superflat overworld**: Minecraft Java 26.2, ground top surface at y = −61, so players stand at y = −60. Bedrock is at y = −64 and the build limit is y = 319. Superflat has no terrain, so **each build makes its own landscape** (mountains, gorge, river, ridges) as part of the build. That landscape must look like geology, never a stack of boxes, and it must meet the flat plain at its box's edges with believable slopes.

| Build | Box (inclusive) | Joins the others at, on its box edge at ground level y = −60 |
|---|---|---|
| The Last Hearth (the pass) | x 2000..2179, z −60..59, y −61..120 | its trail leaves the east edge at (2179, −60, 0) |
| The City That Borrowed a River | x 2300..2749, z −125..124, y −61..80 | its west road enters at (2300, −60, 0); its north road leaves at (2525, −60, −125) |
| The Hollow Meridian (the quarry mountain) | x 2300..2749, z −700..−260, y −61..170 | its quarry road leaves the south edge at (2525, −60, −260) |

The plain between the boxes stays flat; the integrator links the three routes across it afterwards. **The whole region's story:** stone was cut from the Meridian's mountain and carried south down the quarry road to build the city's bridge and waterworks; the city traded west over the pass, where the refuge stands on the old road.

**Shared materials, so a visitor can reconstruct the history** (the brief: "the same distinctive stonework"):
- **The quarry's stone is tuff:** raw `tuff` in the mountain; cut as `tuff_bricks`, `polished_tuff` and `chiseled_tuff_bricks` with their stairs, slabs and walls. It is the city's oldest masonry (the first bridge, the waterworks) and the Meridian's original quarry structures. The refuge has at most one cut-tuff stone, carried there for a reason.
- **The pass's local stone** is stone and andesite (stone bricks where shaped).
- **Timber is spruce** from the forests of the pass, carried to the city and up to the observatory. Other woods appear only where a building's history gives a reason.
- **Copper** marks the newer technology: the later waterworks repairs and the observatory's later instruments. Its oxidation stages tell its age.

Check every block id against the list below; how each build uses them is its own design.

## Hard constraints (all three)
1. **Every block id and state must exist in 26.2.** The authoritative list is `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv`; check every id there. Use correct states for stairs (`facing`, `half`, `shape`), slabs (`type`), lanterns (`hanging`), doors, beds and double plants (both halves), logs (`axis`), and trapdoors.
2. **Leaves:** every leaf block is `persistent=true`, or it will decay.
3. **Falling blocks** (sand, gravel, concrete powder) always rest on support.
4. **Water and lava:**
   - No lava and no fire.
   - Water is source blocks in basins whose every side and bottom is solid.
   - A waterfall or channel flow is allowed only where every block the flow can reach lies inside the box and ends in a contained basin. No water may reach a box edge.
5. **No entities and no block-entity data:** no `/summon` and no NBT in braces. Chests, barrels, lecterns, item-holding blocks and furnaces are placed empty; use and players will fill them, which is the brief's point about traces of use.
6. **Commands only:** `/fill x1 y1 z1 x2 y2 z2 <block>[states]` and `/setblock x y z <block>[states]`, with absolute coordinates strictly inside your box.
   - No `replace` filters, no structure blocks, no other commands.
   - Each `/fill` holds at most 32,768 blocks.
   - Order: landscape, then structure, then details, then interiors and light. Later commands may carve earlier ones (`air` fills).
7. **Redstone only where the brief earns it** (the Meridian's instrument): few, reliable, low-overhead, event-driven. No clocks that run all the time.
8. **Lighting:** no hostile spawning on paths and in rooms people use (light it), but motivated light only: lanterns where people work and live, warm windows, darkness where the brief wants darkness (the city's lower district, the hollow's basin).

## How you may write the commands
Large landscapes are too many commands to write by hand. You may write a **generator**: one Rust source file `gen.rs`, using the standard library only.
- It is deterministic: any noise is your own seeded function, with no randomness from the system.
- It reads nothing.
- It writes only `build.txt` into the folder named by its first argument.
- No network, no process spawning, no unsafe code.

The integrator reads it, then compiles and runs it with `D:\rust\cargo\bin\rustc.exe -O gen.rs`. If your harness lets you run commands, compile and run it yourself and check its output. You may also mix a generator for the landscape with hand-written commands for the architecture. Either way, `build.txt` is the deliverable.

## Write, in your job folder `C:\Users\PASCAL\Desktop\path tracing\jobs\<your job>\`
1. **`build.txt`** (or **`gen.rs`** that writes it): the commands, in order, with `#` comment lines naming each part.
2. **`design.md`:**
   - the design, scale by scale, following the brief's order of decisions: the landscape's structure; the reason for occupation; the essential spaces and movement; construction logic; form language; the experience in motion and time; evidence of occupation, craft and history;
   - **ASCII plans** at the site scale and of each important level, and **ASCII sections** through the key spaces, so the design can be reviewed before it is placed;
   - **the walk:** the main route described step by step, with coordinates, as the brief describes it;
   - **the tests:** the brief's section 6 questions for your build, each answered honestly;
   - **a self-check:** every block id verified against `blocks.tsv`; every coordinate inside the box; every `/fill` within the limit; the leaves, water and falling-block rules held;
   - the counts of commands and blocks.
3. **`report.md`:** progress as you go, and at the end what you would improve with more time.

Iterate until you would be proud to show it. Before finishing, re-read the build against the plans and sections and fix every mismatch: a floating block, a stair facing the wrong way, a door without its top half, water that can escape.

You read, reason and write only into your job folder. Launch nothing.
