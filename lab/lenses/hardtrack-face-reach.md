# Kill test 10 (independent check): does a face-reach relation over hardtrack hold his wall-top shortcut?

**Inputs:** `engine/law`, `engine/words`, `engine/plan` (`src/reach.rs`, `src/goal.rs`, `src/walk.rs`), `lab/tracks/hardtrack.build.txt`, `lab/tracks/hardtrack.json`, `jobs/hard-track/shortcut.md`. Do NOT read or feed `jobs/hard-track/julien-428.jsonl` or `lab/tracks/runs/hardtrack.jsonl` into the construction of the reach graph (use `shortcut.md` ONLY after building the graph to verify containment, and never tune any parameter to his run).

**Consumer:** U2 face-reach build and Kill test 10 (`docs/learning.md` section 6, `jobs/u2-set-record/report.md`, `jobs/unified-plan/plan.md` section 10). `jobs/hard-track/report.md:32` failed to find his `T=428` route because its certificates argued prose about intended pockets instead of building reach between every top/climbable face in the box.

**Build an offline Rust probe** (inside your run folder only, using workspace crates):
1. Parse `lab/tracks/hardtrack.build.txt` over the superflat base (`[-120,-64,-120]..[-50,-61,-50]` bedrock/dirt/grass) into a `BoxWorld` (this is the pre-rebuild `hardtrack` fixture containing the west alignment wall `/fill -85 -37 -99 -85 -32 -96`).
2. Extract **all** support and climbable faces in `[-110..=-60, -61..=-20, -110..=-60]` (exposed full/partial top faces, slime tops, climbable cells and climbable tops) with `[x, x+1] x [z, z+1]` face extent + `±0.3` body overhang.
3. Build mirrored reach tables **from the law itself** (no search over inputs during graph construction; no `f64` trig `turn()` — evaluate actual `words::Arguments` yaws through `law::try_tick`), parameterized by ground/ceiling class (`stone`, `blue_ice` open, `blue_ice` under `2.0`-block ceiling, `slime_block`, `ladder`) and entry speed class (rest, warm, head-hitter steady).
4. Derive the directed reach relation `(F_from -> F_to)` between faces by mirroring the reach tables onto `F_to` (checking overhang-extended takeoff faces `F_from`, altitude-slab `FreeSpace` connectivity, and one `RecordingWorld` worldtube check per candidate arc).
5. Run one backward min-plus wave from `B = [-84, -25, -88]` to `A = [-107, -60, -107]`, and replay the resulting shortest face-chain sentence through `law::try_tick` / `plan::certify` to measure `T`.

**Report:**
- total faces, directed edges, mirrored lookups, and worldtube checks;
- whether the reach relation contains every edge of Julien's wall-top route (`[-107,-60,-107] -> [-106,-59..-29,-106] -> [-84,-29,-102] -> [-85,-32,-96] -> [-84,-30,-92] -> [-84,-25,-88]`), with exact ticks and arc witness for each hop;
- shortest certified `T` from settled `A` to `B` found by the face-reach relation alone (against `428` and the `579` witness).

**Falsifier:**
- the reach relation built without `julien-428.jsonl` omits any edge of his wall-top route, OR
- no certified sentence using the alignment wall top exists at `T <= 428` (or name the exact missing primitive/law gap, e.g. double-tap sprint or ladder-catch transition, with its witness).

**Out of scope:** changing `engine/law`, `engine/words`, `engine/plan`, or `lab/tracks/hardtrack.build.txt`. Leave the rig alone.
