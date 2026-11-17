# The Loop on Real Tracks: What Exists, What Is Missing

**Consumer:** Integrator spec for step 3 of `docs/plan.md` ("The loop on real tracks").
**Checked against:** `main` at `C:\Users\PASCAL\Desktop\path tracing`, core `wt/u1-astra` (`D:\path-tracing-wt\u1-astra`, `jobs/u1/last.md` `STATUS: DONE`), joints/sets `wt/u2-astra` (`D:\path-tracing-wt\u2-astra`, `jobs/u2/last.md` in progress), and unmerged `wt/track-face` (`D:\path-tracing-wt\track-face`).

---

## 1. What Step 3 Needs, Piece by Piece

| Piece | Status | What Exists (`file:line`) & What Is Missing |
|---|---|---|
| **1a. `CorridorSource` over real region files of any dimension** | **PARTIAL** | **Exists:** `CorridorSource` trait and `GeneratorSuperflatSource` on `wt/u1-astra` (`D:\path-tracing-wt\u1-astra\engine\plan\src\source.rs:11-113`); Anvil region reader `WorldReader` (`engine/world/src/region.rs:55-126`) and prevalidated box `WorldSnapshot` (`engine/world/src/snapshot.rs:103-220`); Java dimension-layout adapter `TrackWorldView` (`bot/src/pathtracing/TrackWorldView.java:12-88`, invoked via `scripts/track-world-view.ps1:1-13` from `engine/plan/src/bin/plan-track.rs:44-52`).<br>**Missing:** (1) Neither `WorldReader` nor `WorldSnapshot` implements `law::codec::proof::Source` or `CorridorSource` (not found in `engine/world/src/` or `wt/u1-astra/engine/plan/src/source.rs` or `wt/u2-astra/engine/plan/src/source.rs`). (2) `WorldReader::with_cache_limit` (`engine/world/src/region.rs:72`) hardcodes `dimensions/minecraft/overworld/region` and `-64..320` (`region.rs:99`), and `WorldSnapshot::load_with_limit` (`engine/world/src/snapshot.rs:133-134`) rejects any world whose `world_gen_settings.dat` dimension is not `"minecraft:overworld"`. Today multi-dimension tracks (`pathtracing:diversity_parkour`, `pathtracing:infinity_tower`, `pathtracing:parkour_race`, `pathtracing:frosted_ices`) only load by shelling out to `java.exe` (`TrackWorldView`) to hard-link `.mca` files into a temp overworld directory. (3) `WorldSnapshot` panics on any read outside its loaded `[min, max]` box (`snapshot.rs:167-169`) and caps at `SNAPSHOT_BYTES = 128 MiB` (`snapshot.rs:5`), so `corridor_query` over real regions must either bound queries to the loaded snapshot or wrap `WorldReader` + `law::World` with coverage checks. |
| **1b. Start state (from recording or settled on A)** | **PARTIAL** | **Exists:** Settled start on A via `plan::settle` (`engine/plan/src/walk.rs:43-54`, called in `engine/plan/src/flight.rs:99-104`); `PlayerState::from_bits` (`engine/law/src/state.rs:282-373`); `load_state` JSON parser for `"initialPlayer"`-style objects (`engine/law/tests/replay.rs:86-150`, test-only).<br>**Missing:** (1) `plan-track.rs:32` refuses anything except `--start settled` (and falsely reports `PlayerState::from_bits is unavailable`). (2) No library function in `engine/law` or `engine/plan` loads `"initialPlayer"` from `recordings/track-*.jsonl` into `PlayerState`. (3) `settle` initializes from `PlayerState::default()` (`saturation: 5.0`, `state.rs:149`) rather than the `20.0` armed saturation observed in live `Tracks` (`docs/HANDOFF.md:26`; `recordings/track-hardtrack-Brain_Forge-1.jsonl:1`). (4) 18 old tracks + runs 1–2 of `hardtrack` have no `.jsonl` recording at all (`jobs/flight-corpus/report.md:7, 178-179`). (5) `PlayerState` (`engine/law/src/state.rs:52-121`) lacks `sprint_trigger_time` and previous-tick forward impulse needed for `sprintWindow: 7` recordings. |
| **1c. Goal from the track, including clicked face** | **PARTIAL** | **Exists:** On `main`, `Goal { cell }` with `1e-3` inflation (`engine/plan/src/goal.rs:12-21`) and `track_at` reading `"A"`/`"B"` from `lab/tracks/<name>.json` (`engine/plan/src/flight.rs:24-25`). On unmerged `wt/track-face` (`docs/plan.md:57`), `Goal { cell, face: Option<Face> }` with exact closed-plane / open-tangent `touches_box` (`D:\path-tracing-wt\track-face\engine\plan\src\goal.rs:10-76`) matching `TrackDetector` (`D:\path-tracing-wt\track-face\bot\src\pathtracing\TrackDetector.java:26-40`), plus `"bFace"` parsing (`D:\path-tracing-wt\track-face\engine\plan\src\flight.rs:24-35, 116`).<br>**Missing:** `wt/track-face` is not merged into `main`, `wt/u1-astra`, or `wt/u2-astra`. Also, `wt/u2-astra/engine/plan/src/joints.rs:236` caches `Field` by `f.goal.cell == goal.cell` without checking `face`. |
| **1d. `solve`'s output turned into a T comparable with his clock** | **PARTIAL** | **Exists:** `certify` (`engine/plan/src/walk.rs:75-111`) computes `clock: Option<u32>` = `total - left` (ticks after leaving `on_a`) and `flight.rs:205-212` compares `certificate.clock` against `his` and computes `clock + thinking_ticks` (`(thinking * 20.0).ceil()`).<br>**Missing:** `solve` (`D:\path-tracing-wt\u1-astra\engine\plan\src\solve.rs:190, 197`; `D:\path-tracing-wt\u2-astra\engine\plan\src\solve.rs:206, 209`) returns `Plan.ticks` = total trajectory ticks (`end.t - from.t` / `route.ticks`), without subtracting ticks spent `on_a` (`left`), and `Plan` carries no `clock` field or `a: Option<BlockPos>` input. Additionally, `solve` itself is not yet wired to recurse on `Cut` or drive `JointStore::learn` + `wave` (`u2-astra/solve.rs:88-93, 195` still stops at `NeedsJoints` when no pre-built joint chain reaches the goal). |
| **1e. Replay check** | **PARTIAL** | **Exists:** (1) Plan witness replay `certify` (`engine/plan/src/walk.rs:75-111`) and `u1::replay` (`D:\path-tracing-wt\u1-astra\engine\plan\tests\zones_prereqs\u1.rs:11-27`) verify every tick of `Plan.words` through `law::try_tick` against the world (note `u1.rs:36` skips >10,000 blocks). (2) `write_script` (`engine/plan/src/flight.rs:66-86`) exports live-playable JSONL for `Recorder.playPlan` (`bot/src/pathtracing/Recorder.java:41-92`) and `deviation::compare` (`engine/plan/src/deviation.rs:35-80`). (3) Tick-by-tick recording replay `replay_text` (`engine/law/tests/replay.rs:369-416`).<br>**Missing:** No unified step-3 offline witness replay over `Plan.words` + `Region` worlds; and human recording replay (`replay.rs:448-483`) reads `.world.jsonl` from `lab/fixtures/` only (not Anvil regions) and refuses `sprintWindow != 0` (`replay.rs:366`). |
| **1f. Record per run** | **MISSING** | **Exists:** `flight.rs:169-234` prints text log lines; `jobs/u1/verify.ps1` + `report.ps1` (`D:\path-tracing-wt\u1-astra\jobs\u1\last.md:126-129`) write per-gate logs/JSON for `u1`.<br>**Missing:** No structured per-run record (`AGENTS.md`: inputs for exact replay, each invariant's worst margin, refusal, plus kill-test 8/9/14 metrics: work units vs defects/events, bracket LB/UB, thinking ns, `clock_T`, `his_T`) exists for a `solve`-on-tracks runner (not found in `engine/plan/src/` or `scripts/`). |

---

## 2. Comparison with His Times (`T`)

### How His `T` Is Counted (`Tracks.java` & `TrackDetector.java`)
`Tracks.tickBody` runs on `ClientTickEvents.START_CLIENT_TICK` (`bot/src/pathtracing/Tracks.java:114, 261`):
1. **Standing on A (`onA`):** `mc.player.onGround() && mc.player.mainSupportingBlockPos == Some(track.a())` (`Tracks.java:358-362`). While `onA` is true, `TrackDetector.sample` keeps `armed = true, running = false, ticks = 0` (`bot/src/pathtracing/TrackDetector.java:28-38`).
2. **Start (`Event.START`):** On the first `START_CLIENT_TICK` where `onA` is `false`, `armed -> false`, `running -> true`, `startTick = tick`, and `ticks = 0` (`TrackDetector.java:40-46`). Note `previousBox` is updated *after* `sample` (`Tracks.java:331-332`), so `touchesB` on tick `startTick` sweeps `[last_onA_tick.box, first_offA_tick.box]`.
   - Crucially, `Recorder.recordTrack` (`bot/src/pathtracing/Recorder.java:148-173`, `Tracks.java:335, 390, 405, 611`) saves `initialPlayer` from `START_CLIENT_TICK` of the *last `onA` tick* and writes that last `onA` tick as `t = 0` (`pendingDeparture`), because at `START_CLIENT_TICK` of the first off-A tick `mc.player.tickCount` still matches the previous tick (`Recorder.java:152-153`). Thus a finished track recording with `T = 428` (`recordings/track-hardtrack-Brain_Forge-1.jsonl:1, 430-431`) has **429** rows (`t = 0..428`, `"end":{"ticks":429}`): `t = 0` is the departure tick on A, `t = 1` is `START`, and `t = 428` is `STOP`.
3. **Stop (`Event.STOP`):** On each tick while `running`, `ticks = tick - startTick` (`TrackDetector.java:47-49`). When `touchesB(previousBox, currentBox, track.b())` holds (`TrackDetector.java:5-9` with `inflate(1e-3)` on `main`; `D:\path-tracing-wt\track-face\bot\src\pathtracing\TrackDetector.java:26-40` with exact face plane + open tangents on `wt/track-face`), `sample` returns `Event.STOP` (`TrackDetector.java:50-53`), and `Tracks.finish` records `T = DETECTOR.ticks()` (`Tracks.java:414, 416`). If `A` and `B` are crossed on the same tick (`START` -> `STOP` at `Tracks.java:344`), `T = 0`.

### How `solve`'s `T` Is Counted vs. `certify` (`solve.rs`, `joints.rs`, `walk.rs`)
- In `solve` (`wt/u1-astra/engine/plan/src/solve.rs:190, 197`), `Plan.ticks = end.t - from.t` (or `route.ticks` in `wt/u2-astra/engine/plan/src/solve.rs:206`, `joints.rs:102`), which counts **all law ticks from `from.t` until `goal.touched(&before, &state)` first holds** — *including* ticks where the bot is still `onA`.
- In `certify` (`engine/plan/src/walk.rs:76-104`), `left` is set to `Some(total)` on the first tick after which `!(state.on_ground && state.main_supporting_block == Some(a))` (`walk.rs:76, 99`), and `clock = Some(total - left)` (`walk.rs:103`).
- **Alignment rule:**
  - From a **settled start on A** (`from` has `on_ground = true`, `main_supporting_block = Some(a)`): if step 1 leaves A on its very first tick (`left = 1`), `clock = Plan.ticks - 1` (confirming `docs/HANDOFF.md:26`: *"Predicted T is the clock's T + 1 on all 8"*). If the first word takes `k > 1` ticks before leaving A (`main_supporting_block != Some(a)` or `!on_ground`), `clock = Plan.ticks - k`.
  - From a **recording's `initialPlayer` on A**: `initialPlayer` is at the start of `t = 0` (the last `onA` tick), so `Plan.ticks` matches `recording.ticks` (= `his T + 1` when `t = 0` leaves A), and `clock = Plan.ticks - left` equals `his T` directly (`429 - 1 = 428` on `hardtrack` run 3).
  - **Scoreboard `T`** (`docs/plan.md:7`, `flight.rs:205-208`): adds thinking time in server ticks (`clock + (thinking_seconds * 20.0).ceil()`). Step 3 must report `clock` (movement-only, directly comparable to `his T`), `thinking_ns` (cold/warm), and `clock + thinking_ticks` (scoreboard `T`).

---

## 3. His Recordings and the Double-Tap Sprint Port

### Which Recordings Replay Through `engine/law` Today
- **0 of 66 `Brain_Forge` tracks** (`jobs/flight-corpus/report.md:7, 174-185`):
  - **18 old tracks** (`amertystcoral` through `trapdoorparkour`) + **`hardtrack` runs 1–2**: **no `.jsonl` recording exists** (`lab/tracks/runs/hardtrack.jsonl:1-2`; `Recorder.recordTrack` refused with `IllegalStateException: Requires hold-style input: {..., "sprintWindow":7}`).
  - **48 newer tracks with `.jsonl` files** (`hardtrack` run 3, `practice1..10`, `infinity_tower_1..36`, `parkour_race`): all fail `engine/law/tests/replay.rs:366` (`assert_eq!(object.number("sprintWindow"), 0.0, "unsupported double-tap sprint window")`).
  - **`parkour_race` additionally** contains Creative/Survival switches and Creative flight ticks (`recordings/track-parkour_race-Brain_Forge-1.jsonl:10497-10504` per `jobs/flight-corpus/report.md:109, 165, 182`) with no `"anomaly"` marker, which `engine/law/src/movement.rs:133-135` refuses (`state.flying`).
  - **7 tracks with 0 `Brain_Forge` runs:** `straight100`, `straight300`, `straight800` have `PathBot` recordings (`sprintWindow: 0`, bot runs, not human runs); `snow`, `grass`, `infinity_tower_full`, `frosted_ice` have no runs or recordings (`jobs/flight-corpus/report.md:6, 183-185`).
- Note also that `engine/law/tests/replay.rs:448-462` only loads `.world.jsonl` + `.jsonl` pairs from `lab/fixtures/` — it has no Anvil region loader for `recordings/track-*.jsonl` + `server/world`.

### What the Double-Tap Sprint Port Needs, and Wiki Status
- **Settled in the wiki:** **Yes.** `lab/wiki/port-sprint-window.md:4-7` grades `local-20261116-1008-1-0-port-sprint-window` as **`reproduced`** (2026-11-16). Its dossier (`lab/results/local-20261116-1008-1-0-port-sprint-window/dossier.md:1-69`) closes `Options.sprintWindow` (int `0..=10`, default `7`) and `LocalPlayer.sprintTriggerTime` in `LocalPlayer.aiStep` bytecode, and verifies 0 start/end diffs across all 428 ticks of `julien-428.jsonl` (SHA-256 matching `track-hardtrack-Brain_Forge-1.jsonl`).
- **Port delta required in `engine/law` (from `dossier.md:58-60` & `engine/law/src/`):**
  1. Add `sprint_trigger_time: u8` and `prev_forward_impulse: bool` (plus config/state `sprint_window: u8`) to `PlayerState` (`engine/law/src/state.rs:52-121`, `bits()`/`from_bits()`).
  2. Update `update_sprinting` (`engine/law/src/movement.rs:217-233`) before movement/sprint-key check: decrement `sprint_trigger_time` if `> 0`; zero it on `prev_shift || (using_item && !use_item_can_sprint && !riding) || input.back`; if `can_start_sprinting` and `!prev_forward_impulse`, either `set_sprinting(true)` if `sprint_trigger_time > 0` else arm `sprint_trigger_time = sprint_window`. Update `prev_forward_impulse = input.forward && !input.back`. (Note `dossier.md:51` found `hardtrack` run 3 itself never triggers double-tap because `Brain_Forge` used the sprint key and held `W`; `sprintWindow: 7` still must be accepted and modeled for tracks that double-tap `W`.)
  3. Update `engine/law/tests/replay.rs:366` to accept `sprintWindow` in `0..=10` and thread it into `PlayerState`, instead of asserting `== 0.0`.

---

## 4. The First-Flight Set (`jobs/flight-corpus/report.md`)

### Recommended 11-Track First-Flight Set (`jobs/flight-corpus/report.md` §3, lines 130–148)

| Track | Classification for Step 3 | Exact Reason (`jobs/flight-corpus/report.md` & `engine/law`) |
|---|---|---|
| `straight800` | **Step 3 can run** | Flat open stone (`800b`, `BareLegs`). Already runs in `u1`/`u2`. |
| `hardtrack` | **Step 3 can run** | All mechanics (ladders, blue ice, slime, slabs/walls, jump) are in `law`; `itemsUsed: {}`; target of kill test 10 (`jobs/u2/spec.md:11, 39`). |
| `birchparkourmedium` | **Step 3 can run** | `1x1` pillar jumps, `+13 Y` climb, `itemsUsed: {}` (check snapshot `max_y`: `flight.rs:38` uses `n[4] + 8`, `tracks.txt:3` clipped at `-45`). |
| `fences` (+ `fencemaze`) | **Step 3 can run** | `0.25`w fence posts, overhead gates, closed/open gates; bare legs (`fences` stalled joint is a `u2` target, `jobs/u2/spec.md:41`). |
| `redfakemaze` | **Step 3 can run** | Back-step `+1` climb and `1x1` roof jumps; bare legs (`itemsUsed: {}`). |
| `carpetwater` | **Step 3 can run** (best run `84`) / **Step 5** (run `187`) | Best run (`T=84`) used `{}` items (water/carpet in `law`). Run 1 (`T=187`) used `scaffolding: 3` (place -> step 5). |
| `infinity_tower_full` (or `_19`) | **Step 3 can run** (pending block audit) | Progressive `+379 Y` bare-legs climb (`bFace: "up"`); `flight-corpus:63` notes exact interior blocks are unaudited, so `WorldSnapshot` could still meet an unmodeled block or `world_hook`. |
| `butoonpressureplates` | **Step 3 can run** (with **missing port** caveat) | Empty-collision buttons & plates load in `WorldSnapshot` and ran in `PathBot` (`T=198`, `docs/HANDOFF.md:26`), so bare-legs movement runs and tests non-issue class equivalence (`Air`). However, `port-pressure-plate-hook` (redstone activation/release timer) is **not wired** in `engine/law/src/world.rs` (`jobs/flight-corpus/report.md:162-163`). |
| `parkour_race` | **Step 3 bare-legs partial; missing ports for human run** | Survival jumps are described as easy (`bFace: "up"`), **but** (1) `Brain_Forge`'s `16821` run used Creative flight (`state.flying` refused at `movement.rs:133-135`, `port-creative-flight-toggle` not in `law`), (2) `solve.rs:86` refuses `difficulty != 0` (`parkour_race` drained saturation `20 -> 13`, `server_regeneration` isolated in `movement.rs`), (3) `1526.43b` span may exceed `WorldSnapshot`'s `128 MiB` single-box cap (`snapshot.rs:5`) if loaded as one dense cube. |
| `amertystcoral` | **Needs Step 5 (editing the world)** | `Pose::Swimming` crawl (`port-crawl-pose-transition`) and amethyst shapes (`port-amethyst-state`) are in `law`, **but** closing the copper trapdoor onto the body requires empty-hand `interact` (`item-use-writes` / `port-door-trapdoor-toggle` not in `Input` or `world.rs`). |
| `mud` | **Needs Step 5 (editing the world)** | Requires placing `minecraft:mud: 2` (`itemsUsed`) + restoration (`item-use-writes`, `setBlock`, ledger). |

### Full 73-Track Corpus Summary (`jobs/flight-corpus/report.md` §§1–4)
- **Step 3 can run once step 2 lands (64 tracks, pending block audit on unseen interiors):**
  - Old 21 (15): `straight100`, `straight300`, `straight800`, `stonesimple`, `flowers`, `butoonpressureplates` (hook caveat above), `carpetwater` (best `{}` run), `fencestraightbridge`, `grassjumpeasy`, `mangrovejumpeasy`, `stonestaircase`, `birchparkourmedium`, `fences`, `fencemaze`, `redfakemaze`.
  - `hardtrack` (1).
  - Diversity Parkour `practice1..10`, `grass` (11 — exact interior blocks unknown; `practice4`'s `20t` may hide an obstacle).
  - Infinity Tower `infinity_tower_1..36` + `_full` (37 — `_36` carries `egg:1` in kit with `itemsUsed: {}`; interiors unaudited).
- **Needs Step 5 — editing the world (6 tracks in old 21 + `carpetwater` run 1):**
  - `amertystcoral` (empty-hand trapdoor `interact`), `ametwoodbox` (`oak_log: 1`), `doorcorridor` (door `interact`), `doorparkourblue` (door `interact`), `mud` (`mud: 2` place), `trapdoorparkour` (trapdoor `interact`/place), plus `carpetwater` (`scaffolding: 3` run).
- **Needs Step 8 — carried movement & wormholes (0 tracks explicitly):**
  - None of the 73 tracks in `jobs/flight-corpus/report.md` currently lists boat, elytra, ender pearl, or Nether portal (possible boat/ice overlap on `frosted_ice` is not stated in `flight-corpus`).
- **Needs ports still missing (3 tracks: `frosted_ice`, `snow`, `parkour_race`):**
  - `frosted_ice`: **bubble columns** (`Inside::Unsupported("bubbleColumn")` panics at `engine/law/src/movement.rs:1093`; `ModelTable.java` refuses `BubbleColumnBlock`), **status effects / Jump Boost** (`state.effects` refuses at `movement.rs:121-123`; `port-status-effect-kinematics` not wired), and **dynamic fluid spread** (`world.rs` lacks `FlowingFluid.tick`/`spread`/`getNewLiquid`).
  - `snow`: likely **powder snow** (`movement.rs:201-203` only does `+1` freeze; missing `-2/tick` decay, `-0.05` frost speed slow, `%40` freeze damage, and multi-step `StepBasedCollector` flush).
  - `parkour_race`: **Creative flight** (`movement.rs:133-135` refusal) and **Peaceful regen / non-Peaceful food crossings** (`solve.rs:86` refusal; `server_regeneration` isolated).

---

## 5. Open Questions for the Integrator

1. **How should `CorridorSource` read real multi-dimension region files (`minecraft:overworld` + `pathtracing:<name>`)?**
   - *Options:*
     - **A.** Add native dimension path resolution (`minecraft:overworld` -> `dimensions/minecraft/overworld/region`, `pathtracing:<name>` -> `dimensions/pathtracing/<name>/region`) and `Source` + `CorridorSource` directly in `engine/world/src/` (replacing `TrackWorldView.java` + `track-world-view.ps1`).
     - **B.** Keep shelling out to `TrackWorldView.java` from the CLI runner to build temporary overworld symlink views, and only add a `CorridorSource` wrapper around `WorldSnapshot`.
   - *Recommendation:* **Option A.** Forking `java.exe` per track is slow, fails if `pathtracing.jar` isn't built, and hides dimension errors outside Rust. Keeping `WorldSnapshot` (or a lazy chunk-backed `RegionSource` over `WorldReader` for large spans like `parkour_race`'s 1,526 blocks) inside Rust keeps step 3 single-binary and offline-fast.

2. **Which start state is canonical when comparing `solve`'s `T` against `his` in step 3?**
   - *Options:*
     - **A.** Standardize scoreboard comparison on **settled start on A** (matching `Tracks.settle`, with `saturation: 20.0` rather than `PlayerState::default()`'s `5.0`), and use `initialPlayer` from `.jsonl` separately for replay-aligned checks where recordings exist.
     - **B.** Use `initialPlayer` whenever a `.jsonl` exists and `settle` on the remaining 18+ tracks.
   - *Recommendation:* **Option A.** In live flights (`Tracks.java:554-560`), the bot is required to settle at `A + 0.5` with zero horizontal velocity for 5 ticks before arming, whereas `Brain_Forge` had running momentum on A (`track-hardtrack-Brain_Forge-1.jsonl:1`). Comparing settled `solve` against `his` measures what the bot in step 4 can legally achieve; `initialPlayer` runs serve as a warm-start/diagnostic check.

3. **Where is `clock` (`onA` departure offset) computed: inside `solve` or in the track comparator?**
   - *Options:*
     - **A.** Keep `Plan.ticks = end.t - from.t` in `solve`, and compute `clock` in the step-3 track harness/verifier using `walk.rs:76, 99, 103`'s `on_a` rule, reporting `(clock, thinking_ns, clock + thinking_ticks)` alongside `his`.
     - **B.** Pass `a: Option<BlockPos>` into `solve` and subtract `left` inside `Plan.ticks`.
   - *Recommendation:* **Option A.** `solve` is general-purpose (including mid-run replanning in step 4 where there is no A cell); `Plan.ticks` should remain trajectory duration while the track harness aligns to `TrackDetector`'s clock.

4. **Should `wt/track-face` merge before step 3 begins?**
   - *Options:*
     - **A.** Judge and merge `wt/track-face` first (and update `wt/u2-astra/engine/plan/src/joints.rs:236` so `Field` cache keys include `goal.face`, not just `goal.cell`), keeping `face: None` (all 6 exact faces) for the 22 tracks without `"bFace"`.
     - **B.** Run step 3 against `main`'s `1e-3`-inflated `Goal { cell }` and retrofit `bFace` later.
   - *Recommendation:* **Option A.** 49 of 73 tracks in `lab/tracks/` set `"bFace": "up"`, and `jobs/u2/spec.md:31` already plans on `wt/track-face` landing.

5. **Should `port-sprint-window` be ported into `engine/law` alongside step 3, and should `replay.rs` gain an Anvil-backed runner?**
   - *Options:*
     - **A.** Schedule the small `port-sprint-window` Terra delta (`dossier.md:58-60`) and an Anvil-backed recording replay path in parallel with step 3.
     - **B.** Defer `sprintWindow: 7` until after step 3.
   - *Recommendation:* **Option A.** The bytecode law is closed and reproduced in `lab/wiki/port-sprint-window.md`, and 0 of 66 `Brain_Forge` tracks can replay until `replay.rs:366` accepts `sprintWindow: 7`.
