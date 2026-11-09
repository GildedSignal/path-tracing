# Job 2.1a: read the world from its region files

**Why.** The planner runs as its own Rust process and must know the static world without asking the game (the approved plan, Phase 2.1: "a region-file reader for static worlds"). The movement law (`engine/law`) reads blocks through its `World` trait; this job gives it a real world behind that trait.

**Branch.** `wt/world`, cut from `main` after Phase 0 (the consolidated engine line). Do not commit; the queue runner commits. If the worktree already has uncommitted changes from an interrupted run of this job, continue from them. Cargo settings as in job 1.5b. Stable Rust, **no external crates** (the sandbox has no network): write the decompression yourself.

## Build

1. **A crate `engine/world`** that opens a Minecraft 26.2 world folder read-only and answers "which block state is at (x, y, z)", in the overworld, for any generated chunk.
   - **The Anvil region format:** `region/r.<x>.<z>.mca`, the 4 KiB location table, and each chunk's length, compression byte and payload. Support zlib (2) and uncompressed (3) payloads. For LZ4 (4) and external `.mcc` chunks, return a clear "unsupported" error, never a wrong answer.
   - **Chunk NBT:** sections with their `block_states` palette (names and properties) and packed long arrays, with the bits-per-entry rules of the 1.16+ format, a single-entry palette, and Y from the section index. Read what 26.2 actually writes: take the version from `level.dat` (`DataVersion`), and fail loudly on a chunk whose `DataVersion` differs from it.
   - **A chunk cache** with a fixed memory bound.
   - Never write to the world folder. Open files read-only and allow the server to keep running: copy the region file to a temporary buffer before parsing.
2. **The bridge to the law.** Implement the law's `World` trait over this crate for full blocks and the block shapes the law already models. A block state the law doesn't model returns the law's existing "unmodeled" error, never a guess.
3. **A command-line probe:** `world-probe <world dir> <x> <y> <z>` prints the block state, or a range dump `x0 y0 z0 x1 y1 z1`.

## Test against ground truth, not against yourself

- **The superflat world** at `C:\Users\PASCAL\Desktop\path tracing\server\world` (copy the region files you need into the test fixtures folder; never open the live folder for writing). Its layers are known: bedrock at y = -64, dirt at -63 and -62, grass block at -61, air above.
- **The test courses:** `lab/gates/courses.ps1` on `main` (`git show main:lab/gates/courses.ps1`) lists every `fill` command that built the 14 courses at x = 4096 + 128·i, z = 4096 in that world. Generate the expected block for every cell of at least 3 courses from those commands, and compare every cell. The first course's interior may differ where `wt/fixtures` changed the courses later; report differences, never edit expectations to match.
- **Inflate:** test the decompressor against a few hand-made streams, with stored, fixed-Huffman and dynamic-Huffman blocks.

## Done means

- `cargo test` passes in `engine/`, including the comparison over 3 or more courses with zero mismatches (or a listed, explained set).
- `world-probe` works on the copied fixtures.
- The final message lists every change, the chunk format details you relied on, and every assumption.
