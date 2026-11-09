# Job 2.1b: every block in Julien's tracks loads

**Why.** S55: about 5 bots with pathfinding working very well. Julien's 19 hand-built tracks (`lab/tracks/<name>.json`, A and B corners, in his world `server\world`) are the bar for the first flight. The world reader (2.1a, `engine/world`) refuses any block outside a hand-picked family list (`tools/ModelTable.java`, `FAMILIES`). So tracks with fences, walls, gates, doors, carpets, mud, flowers, coral, amethyst, buttons or pressure plates can't load, and the planner can't plan on them.

**Branch.** `wt/track-blocks`, cut from `main` after the 2.1a merge. Worktree on D:. Toolchain as in 2.1a.

## Build

1. **Replace the family list with a rule** (AGENTS.md, "No patches": no per-block branches). A block state is modeled when all of these hold, all checked by the generator from the 26.2 jar:
   - **Its collision shape is local to the state:** `ModelTable`'s `LOCAL` getter never fires, for each collision context the law uses. Fences, walls, gates and doors keep their connections and hinges in the state, so they should pass.
   - **Its offset doesn't move its collision:** offset type `NONE`, or an empty collision shape (flowers, grass). A block with an offset collision box (bamboo, pointed dripstone) is refused for now, with that reason.
   - **Every movement hook it overrides is one the law models:** `entityInside`, `stepOn`, `fallOn`, the post-fall rebound, friction, speed and jump factors, climbability (from tags). The census of which classes override which hook comes from the jar (reflection over the class hierarchy), not from a list.
   - **A hook that only changes the world** (pressure plates, tripwires, buttons, and the like) doesn't change the player's movement this tick. The block loads with a `world_hook` flag, so the engine knows the bot triggers something. If a hook does both, it's refused.
2. **The law's flags** (`fence`, `wall_or_gate` in `engine/law/src/world.rs`) come from the same generated table (the fence and wall tags), not from names.
3. **Every refusal names its reason:** which hook, the offset, or a non-local shape.

## Test, fast

- **The census:** every block state in the 26.2 registry lands in exactly one bucket: modeled, or refused with a named reason. The test prints the counts per reason.
- **Julien's tracks:** for each of the 19 tracks, load the box from A to B widened by 4 blocks in x and z and by 3 in y, from a copy of `server\world` (open read-only, as 2.1a does). Each box loads, or the test lists the refused states and why. Save the boxes as small fixtures (chunks cut as in 2.1a's `copy-fixtures.ps1`), so the test runs offline.
- **Bit-exact shapes:** for one state from each newly modeled family (an oak fence with two arms, a cobblestone wall, an open and a closed fence gate, a door in each half, a carpet, mud), the law's boxes equal the jar's `getCollisionShape(...).toAabbs()` exactly.
- No sweeps beyond this (AGENTS.md, "Tests answer fast").

## Must not

- Change the law's movement code, beyond reading flags from the table.
- Open `server\world` for writing.
- Run live gates.
- Push or merge.

## Done means

`cargo test` from `engine/` is green. The census counts and the per-track result (loaded, or which states were refused and why) are in the final message. Commits on `wt/track-blocks` end with `Built-by:`.
