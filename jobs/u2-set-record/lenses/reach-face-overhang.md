# Does mirrored face-overhang reach close fences' stalled joint without search over inputs?

**Inputs:** `engine/law`, `engine/words`, `engine/plan/src/reach.rs`, `engine/world/tests/fixtures/`, `lab/results/local-20261113-1845-1-1-backward-tools/dossier.md`.

**Consumer:** U2 face-reach build and gate (`docs/HANDOFF.md` U2: "the stalled joint on `fences`", `docs/learning.md` section 1). In `backward-tools`, stone-only from-rest cell-centre lookups failed on 89% of false-yes attempts due to `runup-mismatch`, and `docs/learning.md` section 1 notes `fences` and `fencemaze` stalled at one joint the old cell field claimed and no exact state realised.

**Build an offline Rust probe** (inside your run folder only) on all loadable track fixtures (`engine/world/tests/fixtures/`, specifically isolating `fences`, `fencemaze`, `birchparkourmedium`, `stonestaircase`, `butoonpressureplates`):
1. Replace `reach.rs`'s single-floor from-rest `Reach::compile` with multi-ground + entry-speed-class (`cold`, `warm`, `steady`) reach tables built using actual `words::Arguments` yaws (no `f64` trig rotation `turn()`).
2. Replace cell-centre `[x+0.5, z+0.5]` takeoffs/landings with full face rectangles (derived from `world.collision_shape` top `VoxelShape` planes — notably narrow fence/wall tops and climbables) extended by `±0.3` body overhang.
3. Run the backward face-reach wave on each track and check every edge by **one** `RecordingWorld` replay from the mirrored takeoff state on the face (not cell centre).

**Report:**
- across all loadable fixtures (and per-track for `fences`, `fencemaze`, `birchparkourmedium`, `stonestaircase`, `butoonpressureplates`): standable/reachable faces, certified 1-hop and multi-hop goals (against `backward-tools`'s `4,722 / 5,587 = 84.5%`), and failure breakdown (`runup-mismatch`, overhang-miss, fence/shape collision);
- exact diagnostic on `fences`'s stalled joint: why the old cell field claimed it, and whether face-overhang + multi-ground mirrored tables certify a valid chain across it (with `T` and exact witness) or expose a missing word/program.

**Falsifier:**
- `fences` still stalls with no certified face-reach chain after adding face-overhang (`±0.3`) and multi-ground/entry-speed reach tables (without searching inputs), OR
- `runup-mismatch` false-yes rate stays above 20% once per-ground run-up tables are used.

**Out of scope:** changing `engine/law`, `engine/words`, or `engine/plan`. Leave the rig alone.
