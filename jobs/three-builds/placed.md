# The three builds, placed (17 November, 16:45–16:47)

Placed in his superflat overworld through a datapack of 131 functions of up to 10,000 commands each. The server parsed every command when loading them, with no errors, so every block state is valid in 26.2. A sample of 200 random commands per build was then checked in the world: 600 of 600 in place. The datapack was removed afterwards; regenerate the commands with each job's `gen.rs` and `jobs/three-builds/buildc.rs`.

| Build | Box | Commands placed | Blocks | Arrive at |
|---|---|---|---|---|
| The Last Hearth | x 2000..2179, z −60..59, y −61..120 | 41,122 | 534,940 | its trail's east end, `/tp 2181 -60 0` |
| The City That Borrowed a River | x 2300..2749, z −125..124, y −61..80 | 68,796 | 3,254,005 | its west road, `/tp 2298 -60 0` |
| The Hollow Meridian | x 2300..2749, z −700..−260, y −61..170 | 1,180,945 | 7,609,237 | its quarry road, `/tp 2525 -60 -258` |

- **Links across the plain:** a 3-wide dirt path at y −61 from x 2180 to 2299 along z −1..1 (the refuge to the city), and from z −259 to −126 along x 2524..2526 (the observatory to the city).
- **The sites were never generated before:** no region files at x 2000..2749.
- **The backup from before placing:** `D:\path-tracing-evidence\world-backup-20261117-before-three-builds\` (the overworld folder and level.dat). To undo, stop the server and copy `overworld` back over `server\world\dimensions\minecraft\overworld`.
- **Integrator fixes to Hydrogen's generators** (in commit c00ccce): two integer types; the observatory's quarry road started 2 to 3 blocks inside its box; a floating lectern set on its floor.
- **Cost:** Gemini 5-hour 0 → 15%, week 37 → 42%, for all three (about 10 minutes each, run in parallel).
