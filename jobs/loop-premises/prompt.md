# The loop on real tracks: what exists, what is missing

**Consumer:** the integrator's spec for step 3 of `docs\plan.md`, "the loop on real tracks". In it, one `solve` runs his tracks offline and is compared with his recorded times. It is the first real verdict on the new algorithm, and it comes right after joints and sets (step 2, which Astra is building now).

You read only, and write only your report. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- `docs\plan.md`, `docs\learning.md` sections 0 and 6 (kill tests 8, 9, 14), and `jobs\unified-plan\plan.md` sections 2 and 4 (the loop; the walks as the bot).
- The core: branch `wt/u1-astra` (`D:\path-tracing-wt\u1-astra\engine\plan\src\solve.rs`, `store.rs`, `events.rs`, `lattice.rs`, `source.rs`) and its record `jobs\u1\last.md` there.
- Joints and sets, in progress: `jobs\u2\spec.md`, and `D:\path-tracing-wt\u2-astra\jobs\u2\last.md` if present.
- **What runs tracks today:** `engine\plan\src\bin\plan-track.rs`, `engine\plan\src\flight.rs`, the track readers (`lab\tracks\*.json`, `engine\world\tests\fixtures\tracks.txt`), the world snapshot and region reading (`engine\world\src\`), including reading a track's dimension (the maps work, merged on main).
- **His tracks and runs:** `jobs\flight-corpus\report.md`, the first test set you drew up.
- **Replaying his runs:** `engine\law\tests\replay.rs` (it refuses `sprintWindow` ≠ 0 at line 366); the lab wiki page `lab\wiki\port-sprint-window.md`, if it is settled.

## Write `jobs\loop-premises\report.md`
1. **What step 3 needs, piece by piece:**
   - a `CorridorSource` over real region files of any dimension, beside the superflat generator;
   - the start state from his recording, or from a settled start on A;
   - the goal from the track, including the clicked face;
   - `solve`'s output turned into a T comparable with his clock;
   - the replay check;
   - the record per run.

   For each, EXISTS (file:line), PARTIAL, or MISSING.
2. **The comparison with his times:** how his T is counted (the track clock's start and stop rules) against how `solve`'s T is counted, so the two are the same number. Cite the code of both.
3. **His recordings:** which can be replayed through the law today; what the double-tap sprint port needs before his `sprintWindow` 7 runs replay; and whether that is settled in the wiki.
4. **The first-flight set** from `flight-corpus`: which tracks step 3 can run once joints and sets land, which need editing the world (step 5), which need carried movement (step 8), and which need ports still missing.
5. **Open questions** for the integrator, each with the options and your recommendation.

Cite file:line for every claim. Where something is absent, write "not found in" with the files checked.
