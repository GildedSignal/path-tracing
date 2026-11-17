# The loop on his tracks (step 3 of the plan)

Builders (S225): **Sol 6.2 builds Part A** (his tracks as inputs and the harness), now; **Astra 6.2 builds Part B** (closing the loop in `solve`) after her current job, every new command on flat ground in microseconds (`jobs/warm-floor`), is judged. Written and checked by the integrator; no Opus red-team (S226: the native week is nearly spent). Every fact below that came from a companion's report was checked by the integrator against the code on 17 November; the rest is cited to his words or the docs.

## Why

`solve` has passed every offline gate on the million line, and joints and sets hold his shortcut (158 ticks against his 169 after the ladder). But `solve` has never run one of his tracks end to end. Today it stops at the first corridor exception with "needs joints" (`engine/plan/src/solve.rs:204` at `e6de417`); kill test 10 reached 158 through test-only wiring (`tests/u2.rs:278`), not through `solve`. This step closes the loop of `docs/learning.md` section 0 on real terrain and gives the first verdict against his times.

**His bar (S189, verbatim):** "the practice tracks are neo jumps so you really need to match my T or faster since there's a floor underneath and you could get close without making a neo jump, not all my scores are perfect though as those are hard." And S166: hard tracks "have many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that."

## Read first

`docs/PRINCIPLES.md` (goal, philosophy), `AGENTS.md`, `docs/plan.md` (steps 2b, 3, 4), `docs/learning.md` sections 0 and 6 (the loop; kill tests 1, 8, 9, 10, 14, 15), `docs/representation.md` sections 10–11 (families, points and holds; the space), `docs/vocabulary.md`, `docs/engine.md` (the contract), `docs/scales.md` section 2. The joints-and-sets code you build on: `engine/plan/src/{solve,joints,families,family_wave,terminal,faces,reach,source,goal,walk,flight}.rs` and `jobs/u2b/last.md`.

## Base, and who owns which files

Astra is making every new command on flat ground take microseconds on `wt/warm-floor` (from `e6de417`), rewriting the per-command path in `solve.rs`, `store.rs`, `events.rs`, `lattice.rs` and `terminal.rs`. Two builders never share files (AGENTS.md). So:
- **Part A** (inputs and harness, Sol) touches none of those files and starts now, on `wt/loop-tracks` from `e6de417`. Time every track per real command, as `engine/plan/tests/real_time.rs` does (in `D:\path-tracing-wt\real-time`): one living store, a new goal each time, never a repeated goal as the thinking time (S223).
- **Part B** (closing the loop in `solve`, Astra) starts only after `wt/warm-floor` is judged; the integrator then merges it into `wt/loop-tracks`. Part A's builder does not write code in those files.

## Part A: his tracks as inputs, and a harness that counts like his clock

1. **Terrain from real region files, any dimension.** A `CorridorSource` (and the law's `Source`) over Anvil regions, beside the superflat generator. Today `WorldReader` hardcodes `dimensions/minecraft/overworld/region` (`engine/world/src/region.rs:72`) and `WorldSnapshot` refuses any non-overworld dimension type (`snapshot.rs:133-134`). Resolve `minecraft:overworld` and `pathtracing:<name>` to their region folders natively in Rust; no Java shell-out. Read from a copy of the world, never the live server folder. A read outside what is loaded is a refusal with its position, never a panic and never a guessed block. The corridor query answers exceptions from the region data, so compute still follows defects, not length (kill test 9).
2. **The goal is his track's B and its clicked face** (S170). Merge `wt/track-face` (judged or not: its goal and `bFace` parsing are what you need; its judge runs separately) and key every cache that holds a goal by cell and face (the field cache in `joints.rs` compares cells only).
3. **The start.** Settled on A as the live bot is (`Tracks.java` settles at A + 0.5 with zero horizontal velocity), with the armed food and saturation, not `PlayerState::default()`'s saturation 5.0. **Time on A is free** (S27): the clock starts when the body leaves A (`TrackDetector`: the first tick not standing on A) and stops at the first tick whose swept box touches B's face. A run-up on A is the bot's to use.
4. **The harness** (`engine/plan/tests/tracks.rs` or a `bin`, your choice, one entry point): for each track, load the world box around A and B, settle, `solve`, replay the plan through `law::try_tick` on the same world (the witness), and write one record per run: inputs for exact replay, the clock T (comparable with his), thinking ns cold and warm, the scoreboard T (clock plus thinking in ticks, S124), stages run, work units, corridor exceptions and events, every refusal with the missing capability named, and his best T from `lab/tracks/runs/<track>.jsonl`.
5. **The set of tracks** (`lab/tracks/*.json`; runs in `lab/tracks/runs/`): the ten Diversity practices (`practice1`–`practice10`, dimension `pathtracing:diversity_parkour`, all `bFace` up; his best T: 8, 13, 11, 20, 11, 11, 11, 8, 11, 11), the old 21 (`engine/world/tests/fixtures/tracks.txt`, overworld fixture), `hardtrack` (428), and `infinity_tower_1`–`36`. Tracks that need editing the world (`amertystcoral`, `ametwoodbox`, `doorcorridor`, `doorparkourblue`, `mud`, `trapdoorparkour`) or unported mechanics are run anyway and must refuse with the capability named: a refusal is a result, a patch is not.

**Part A's gate:** the harness runs every listed track and writes its record. On tracks the current `solve` already finishes (the straights, `stonesimple`, `flowers`), its clock T equals the witness replay's tick for tick, and a planted off-by-one in the clock (counting the last tick on A) turns the comparison red.

## Part B: close the loop in `solve`

`learning.md` section 0, steps 1–6, end to end: bracket; closed → emit; extend the biggest word until its certificate breaks; **cut** at the exception's joints; solve each part from the sets on its two ends (learning a joint pattern the store lacks, `families.rs`, once, keyed by content); **compose** backward in min-plus (`family_wave.rs`) and forward with the point; allocate work to the open bracket whose narrowing can change the decision due soonest. One entry point for every track, the straights and `hardtrack` alike (kill test 8). No best-first search over exact states in acting (learning.md section 0, "Is A* needed?"); search exists only inside learning a joint pattern.

## The gate (frozen before its first run; record the hashes of the gate files)

1. **His practice tracks:** on every Diversity practice the bot's clock T is at most his best (8, 13, 11, 20, 11, 11, 11, 8, 11, 11). Where the bot is slower, the record names the joint where it loses (compare against his recorded run's joints only as a diagnostic, never as a hint, S56).
2. **One mechanism** (kill test 8): `straight800` and `hardtrack` through the same entry point; a planted fast path for uniform ground turns the gate red.
3. **Compute mirrors terrain** (kill test 9): thinking work units against (corridor exceptions + events) over the old 21, the straights and `hardtrack`; killed if work grows with length on uniform ground.
4. **The shortcut through `solve`** (kill test 10): `hardtrack` end to end at most his 428, with no use of his inputs.
5. **Every listed track** has a record: a witnessed plan with its clock T and his, or a refusal naming the missing capability. No track crashes the harness.
6. Every gate of `jobs/u2b/verify.ps1` keeps its recorded outcome.

## Must not

Use his recorded inputs as hints; add a per-terrain path, a tolerance, an epsilon or a retry; touch the law's recurrence (observation only); touch `bot/`, `server/`, `clients/`, his tracks or his runs; run the game or the server; run `launch.ps1`, `gpt.ps1` or `follow-up.ps1`; commit (the integrator commits with `Built-by:`).

## Output

`jobs/loop-tracks/last.md`, written as you go: per part, files changed, the gate results with numbers, the table of every track (his T, the bot's clock T, scoreboard T, thinking cold/warm, work units, exceptions, refusal), and what remains. A `jobs/loop-tracks/verify.ps1` replaying every gate. Independent parts that fail do not stop the others. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
