# U1 Spec Report: The Kernel (Slices C and D, with `count` on Every Word)

**Consumer:** Terra's U1 build on `wt/3.2d-3` (`docs/HANDOFF.md:70`, `jobs/unified-plan/plan.md:195`), gated by Kill Test 17 (`docs/learning.md:119`) and Kill Test 8's uniform-ground planted-fault check (`docs/learning.md:110`).
**Status of Frazzoli (`jobs\frazzoli\report.md`):** **Pending.** `C:\Users\PASCAL\Desktop\path tracing\jobs\frazzoli\report.md:1-20` currently holds only the section-0 context header from our docs while Argon reads Frazzoli's primary papers in parallel. Per `docs/HANDOFF.md:70` and `jobs/unified-plan/sources.md:55`, Frazzoli's paper must be read in full before U1 freezes.

---

## 1. Premises Checked Against `wt/3.2d-3` and `P1`/`P2`/`P4`

### 1.1 On `D:\path-tracing-wt\3.2d-3` (`wt/3.2d-3`)

| Item / Symbol | Status | Evidence (`file:line`) |
|---|---|---|
| Workspace crates (`law`, `world`, `words`, `plan`, `native`) | **EXISTS** | `D:\path-tracing-wt\3.2d-3\engine\Cargo.toml:2` |
| One codec (`Interner`, `Label`, `LabelId`, `ClassId`, `LegsRole`, `ProjectionTable`, `StateQuestion`, `state_key`, `SCHEMA`) | **EXISTS** | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec.rs:12,28,30,72,128,137,141,173,177,180` |
| State class sharing `ClassId(u16)` (`TableFull` at `u16::MAX` = 65,535) | **DIFFERS** (shared cap risk) | `codec.rs:173,208` caps `ClassId` at `u16::MAX`; `jobs\3.2d-3-B-judge\last.md:27` and `jobs\spec-fixes\report.md:252-254` flag that Slice C must decide whether state recurrence keys get their own ID space |
| State comparisons (`state_answers`, `food_level > 6`, `health > 0.0`) | **EXISTS** | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec_state.rs:7-86` (`:17` `food_level > 6`, `:18` `health > 0.0`, `:25` `±29999999.0` clamps) |
| Section role planes (`Plane`, `SectionRoles`, SIMD `masked_pair`) | **EXISTS** (standalone) | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec_planes.rs:7-93` (not yet wired into `resident::Source` or `plan`) |
| Proof spine (`Source`, `SnapshotSource`, `Binding`, `Answer<Q>`, `ReadsHold`, `TubeEmpty`, `ReadCertificates`) | **EXISTS** (`TubeEmpty` unproduced; `SnapshotSource` ignores tick) | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec_proof.rs:12-183` (`SnapshotSource` at `:19-44`, `ReadCertificates` at `:95-183`; `jobs\3.2d-3-B-judge\last.md:29,38`) |
| Observed bits (`PlayerState::bits`, `from_bits`, `from_observed_bits`) | **EXISTS** | `D:\path-tracing-wt\3.2d-3\engine\law\src\state.rs:202-249,284-386` |
| Player half-width & strict intersection (`Aabb::player`, `Aabb::intersects`) | **EXISTS** (differs from literal `0.3`) | `D:\path-tracing-wt\3.2d-3\engine\law\src\shapes.rs:111-114` uses `r = (0.6f32 / 2.0f32) as f64` (`0.30000001192092896`); `:203-205` uses strict `<` and `>` |
| 65,536-entry trig table (`SIN`, `SIN_SCALE`, `sin`, `cos`) | **EXISTS** | `D:\path-tracing-wt\3.2d-3\engine\law\src\math.rs:1-17` (`SIN_SCALE = 10430.378350470453`) |
| Movement law (`try_tick`, `update_sprinting`, `food_tick`, `server_regeneration`) | **EXISTS** (sprint guard unconditional on `difficulty`) | `D:\path-tracing-wt\3.2d-3\engine\law\src\movement.rs:140-233` (`:219,225` check `food_level > 6` without checking `difficulty`); `jobs\spec-fixes\report.md:37` cites `:1276` skipping food drain on `difficulty == 0` |
| Pattern scope (`in_scope`, `FRAMES[0]`, `PatternTable`) | **EXISTS** (`by_model` removed in B) | `D:\path-tracing-wt\3.2d-3\engine\law\src\pattern.rs:15-23,52-62,340-342` (`jobs\3.2d-3-B-judge\last.md:26`) |
| `Arguments` with `count` on every word | **DIFFERS** (`count` missing) | `D:\path-tracing-wt\3.2d-3\engine\words\src\program.rs:84-93` has `{ yaw_index: u8, refined_yaw: Option<f32>, pitch: f32, max_ticks: u32 }` — no `count` field. Also `yaw_index: u8` is 64 bins (`:97`), while `refined_yaw` carries 65,536 `sin.bin` yaws (`:86,96`) |
| `GeneratorStore` keyed by `LabelId` | **MISSING** | Not found in `D:\path-tracing-wt\3.2d-3\engine\{law,words,plan,native}\src\`. Legacy `words::gait::Discovery::memo` (`gait.rs:37`) uses `PatternTable` and `fed` (`gait.rs:45,72-77`); `words::recurrence` (`gait.rs:55-64`) encodes all non-`t`/pos fields instead of read-before-write motor bits |
| `plan::solve` single entry point | **MISSING** | Not found in `D:\path-tracing-wt\3.2d-3\engine\plan\src\lib.rs:1-18`. Legacy `Planner` (`search.rs:115-513`, with `fed` at `:132-134`) and `Reach::turn` (`reach.rs:42-45` using `f64::cos/sin`) remain |
| `Goal::zone`, `Goal::touched`, `Goal::lower_bound` | **EXISTS** (bare legs only) | `D:\path-tracing-wt\3.2d-3\engine\plan\src\goal.rs:4-41` (`C = 19.902886 / 20.0` at `:7`; `zone` ±`1e-3` at `:15-18`; `touched` at `:19-21`) |
| `arith::Cycle`, `free_tick` | **EXISTS** (to replace/adapt) | `D:\path-tracing-wt\3.2d-3\engine\plan\src\arith.rs:32-125` |
| `walk::Step`, `Lane`, `settle`, `certify` | **EXISTS** | `D:\path-tracing-wt\3.2d-3\engine\plan\src\walk.rs:10-111` |
| `native::resident` JNI worker (`"pathtracing-F"`, `Source`, `legacy_sentence_adapter`) | **EXISTS** (`Source` not `codec::proof::Source`) | `D:\path-tracing-wt\3.2d-3\engine\native\src\resident.rs:17-52,125,170-204`; `lib.rs:9-132`; `bot\src\pathtracing\{Bot,NativeChannel,PlayerSnapshot,ResidentRunner,ResidentSource}.java` |
| `WorldSnapshot` & `tracks.txt` (`straight100/300/800`) | **EXISTS** | `D:\path-tracing-wt\3.2d-3\engine\world\src\snapshot.rs:1-80`; `tracks.txt:19-21` |
| Generator-backed superflat `Source` (for 10⁶ blocks offline) | **MISSING** | Not found in `D:\path-tracing-wt\3.2d-3\engine\world\src\lib.rs:1-44` or `plan\src\lib.rs:1-18` |
| `jobs\3.2d-3\verify.ps1` | **EXISTS** (A skeleton only) | `D:\path-tracing-wt\3.2d-3\jobs\3.2d-3\verify.ps1:1-66` handles `Slice -Slice A` and `Prerequisites`; throws on other slices/cases (`:16,31`) |
| Frozen 13-mutant footprint gate | **EXISTS** | `D:\path-tracing-wt\3.2d-3\jobs\1.2-footprints\verify.ps1:1-72` |

### 1.2 On `P1`, `P2`, and `P4` Branches

| Job / Branch | Status | Evidence (`file:line`) |
|---|---|---|
| **P1 (`wt/3.2d-3-P1`)** | **EXISTS on `wt/3.2d-3-P1`; MISSING on `wt/3.2d-3`** | `D:\path-tracing-wt\3.2d-3-P1\jobs\3.2d-3-P1\last.md:1-41` (PASS, judge findings resolved); `D:\path-tracing-wt\3.2d-3-P1\engine\plan\tests\zones_prereqs\t1.rs:1-219` (`transient_ticks=263`, `cycle_period=12`, increment `(0.0, 0.0, 4.276060366589263)`, `hot_apply` 48/53.6/78 µs debug, 8 world reads, 0 tick calls); manifest at `D:\path-tracing-evidence\3.2d-3\T1\manifest.json`. `zones_prereqs` is not yet on `wt/3.2d-3`. |
| **P2 (`wt/3.2d-3-P2`)** | **MISSING / IN FLIGHT** | `D:\path-tracing-wt\3.2d-3-P2\jobs\3.2d-3-P2\last.md:1-27` and `t2.rs:1-16` hold the first run's failed literal-`0.3` premise witness (`shapes.rs:112-113` uses `(0.6f32 / 2.0f32) as f64`); P2 is currently rerunning with the law's half-width. No passing binade integer map, inverse stop, or `T2\manifest.json` exists yet. |
| **P3 (`T3` two-yaw lattice)** | **MISSING** | Not yet run; `D:\path-tracing-wt\3.2d-3\engine\plan\tests\zones_prereqs` does not exist on `wt/3.2d-3` and P3's spec (`jobs\spec-fixes\report.md:186-204`) depends on P2's binade maps. |
| **P4 (`wt/3.2d-3-P4`)** | **DIFFERS / PARTIAL** | `D:\path-tracing-wt\3.2d-3-P4\jobs\3.2d-3-P4\last.md:1-29` and `t4.rs:1-42`: offline `t4_food` PASS (240 ticks on Peaceful, food 7 and 20 keep `food_level` unchanged and sprint allowed; food 6 keeps sprint false; `state_answers` matches; unknown food word refused). Live `-Case T4Food` was NOT run (forbade server/client launch) and no `T4\manifest.json` was written. |

---

## 2. Decision-Free Terra Job Spec for U1

> **Note for integrator:** When materializing `jobs/u1/prompt.md`, prepend `docs/PRINCIPLES.md` sections "The goal" and "The philosophy" verbatim per `AGENTS.md`. Do NOT launch until Frazzoli (`jobs/frazzoli/report.md`) is complete and P1+P2+P4 are merged onto `wt/3.2d-3` (or run Part A first if splitting — see §3).

### Header
- **Job ID:** `u1`
- **Base branch:** `wt/3.2d-3` (after merging `wt/3.2d-3-P1`, `wt/3.2d-3-P2`, and `wt/3.2d-3-P4` onto `wt/3.2d-3`)
- **Worktree:** `D:\path-tracing-wt\u1` on branch `wt/u1`
  - Launch command: `.\scripts\launch.ps1 u1 terra -Worktree D:\path-tracing-wt\u1 -Base wt/3.2d-3`
  - Fixture copy (handled by `launch.ps1` / verified before run): `engine/world/tests/fixtures/world` present in `D:\path-tracing-wt\u1\engine\world\tests\fixtures\world`.
  - Target dir: `$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\u1'`
  - Evidence dir: `D:\path-tracing-evidence\u1`
- **CPU priority:** Idle (`launch.ps1`). Offline only; launch no game, server, or client.
- **Decisions already made (halt on nothing; ask zero questions):**
  1. State recurrence keys in `GeneratorStore` use their own `u32` handle (`MotorKeyId(u32)`) keyed on canonical `codec::state_key(StateQuestion::Recurrence, &motor_words)` bytes inside `GeneratorStore` — do **not** intern motor keys into `Interner`'s shared 16-bit `ClassId` table (`TableFull` at 65,535).
  2. `count: u32` (default `1`) is added directly to `words::program::Arguments`.
  3. The any-bearing native yaw domain is the 65,536-entry `law::math::{sin, cos}` `sin.bin` table (`index: u16` in `0..65536`, `yaw = (((index as f64 + 0.5) * TAU / 65536.0) / 0.017453292f32 as f64) as f32` via `Arguments::refined_yaw`); ties pick the lowest `0..65536` index.
  4. `Goal::touched` inverse uses the law's exact half-width `r = (0.6f32 / 2.0f32) as f64` (`shapes.rs:112`) and strict AABB intersection (`shapes.rs:203-204`).
  5. Food on Peaceful (`difficulty == 0`): `food_tick` skips food drain (`movement.rs:1276`), and `update_sprinting` (`movement.rs:219, 225`) keeps its unchanged `food_level > 6` guard. Evaluate actual observed bars with zero `fed` substitution and zero invented food-drain switch.
  6. `solve` has **one** entry point and **zero** terrain-kind branches. Every query against terrain (superflat generator or later voxel/octree sources) goes through a single `CorridorSource` trait.

### Exact Files to Create and Edit

**Edit:**
1. `engine/words/src/program.rs`: add `pub count: u32` to `Arguments` (`Default` = `1`).
2. `engine/words/src/gait.rs`: remove/deprecate `fed` usage from active paths (if `gait.rs` tests call `fed`, keep existing tests green or migrate them off `fed` without changing law behavior; `words::recurrence` stays intact if existing tests rely on it, while `store.rs` owns the read-before-write `motor_key` from P1).
3. `engine/words/src/lib.rs`: export `pub mod store;` (or place `store` in `plan` — **decision made:** place `GeneratorStore` in `engine/plan/src/store.rs` so it can depend on `law::codec`, `law::try_tick`, `words`, and `Goal` without circular crate dependencies).
4. `engine/plan/src/lib.rs`: export `pub mod events;`, `pub mod lattice;`, `pub mod solve;`, `pub mod source;`, `pub mod store;`, and re-export `solve::{solve, Bracket, Kit, Plan, PlanOutcome, Stage, WordStep}`.
5. `engine/plan/tests/zones_prereqs/main.rs`: register `mod u1;` (keeping `t1`–`t4`).
6. Any workspace call sites initializing `Arguments { ... }` without `..Arguments::default()` (add `count: 1` or `..Arguments::default()`).

**Create:**
1. `engine/plan/src/store.rs`: **Slice C — `GeneratorStore` keyed by `LabelId`**.
2. `engine/plan/src/events.rs`: **Slice D (part 1) — Binary64 binade integer maps (from P2), inverse `Goal::touched` with `r = (0.6f32 / 2.0f32) as f64`, and merged U1 event stream (binade crossings + food guard + loaded/zone/yaw deadlines)**.
3. `engine/plan/src/lattice.rs`: **Slice D (part 2) — Two-yaw bracket and Euclidean quotient/floor-sum integer lattice solver over `0..65536` `sin.bin` yaws**.
4. `engine/plan/src/source.rs`: **Uniform corridor presence interface `CorridorSource` + `GeneratorSuperflatSource` (superflat layer spec + sparse `BTreeMap<BlockPos, Block>` scars journal, implementing `law::World`, `law::codec::proof::Source`, and `CorridorSource`)**.
5. `engine/plan/src/solve.rs`: **Single entry point `solve(from, to, kit, source, store, codec, deadline)`** (steps 1–6: `bracket`, `closed?`, `extend`, `cut`, `compose`, `allocate`, plus `Stage` execution trace).
6. `engine/plan/tests/zones_prereqs/u1.rs`: **Kill Test 17 (10⁶-block offline superflat Peaceful gate, both axis-aligned and oblique `(1000000, -61, 330000)`), Kill Test 8 planted-fault test, P2/P4 kernel integration checks, and `ReadCost`/hot-latency distribution (`n >= 100` in `--release`)**.
7. `jobs/u1/verify.ps1` and `jobs/u1/last.md`.

### Specification of Each Component

1. **`count` on every word (`engine/words/src/program.rs`, `engine/plan/src/solve.rs`):**
   - `Arguments` gains `pub count: u32` with `Default` `count: 1`.
   - Every `WordStep` emitted by `solve` carries `program: u16`, `args: Arguments` (with `args.count >= 1`, `args.refined_yaw: Some(yaw)`), `ticks: u32`, `expected_end_t: u64`, `events: Vec<Event>`, and `certificate: ReadCertificateRef` / `CorridorProof`.

2. **Generator Store keyed by `LabelId` (`engine/plan/src/store.rs`):**
   - Outer key: `law::codec::LabelId` derived via `Interner::label(Label { law_schema: codec::SCHEMA, body_kit, support: &[Support02, Support05], medium: &[BodyFeet, Inside] })` from the entry state's support and body/feet cells.
   - Inner key: `(MotorKeyId, program: u16, yaw_index: u16, pitch_bits: u32)` where `MotorKeyId(u32)` interns canonical `codec::state_key(StateQuestion::Recurrence, &motor_words)` bytes (exact read-before-write motor fields, `position.y - anchor_y`, `food_level > 6`, `health > 0.0`; **no** `t`, **no** `x/z`, **no** absolute `main_supporting_block`, **no** numeric `food_level`/`saturation`/`exhaustion`/`food_tick_timer`, **no** `fed`).
   - Yaw arguments share the outer `LabelId`; never key `LabelId` on yaw.
   - Each `WordKernel` entry stores:
     - `transient_ticks: u32`, `transient_states: Vec<PlayerState>`, `transient_endpoint: PlayerState`, `transient_reads: Footprint`;
     - `cycle_period: u32`, `cycle_increments: Vec<Vec3>` (`movement_this_tick` operands before position addition), `cycle_clamped: Vec<[bool; 3]>`, `cycle_level: Vec<f64>`, `cycle_displacement: Vec3`, `cycle_states: Vec<PlayerState>`, `cycle_reads_relative: [[i32; 2]; 3]`;
     - `yaw_switch_transients`: transition summary from `yaw_i` cycle endpoint motor state to `yaw_{i+1}` cycle recurrence (or prepared on demand/cached under `(LabelId, program, yaw_i, yaw_{i+1})`, reusing the same preparation function — never run during hot `solve` once prepared);
     - `prepared_yaws` coverage bitset/set over `0..65536`. Missing coverage returns `PlanOutcome::Unknown(Refusal::MissingPreparedCoverage)` (never a hidden hot-path tick loop).
   - Preparation uses `law::try_tick` + `RecordingWorld` off-query (e.g. `store.prepare(...)`). During `solve`, **zero** calls to `law::try_tick` occur on prepared hits (verified by a counted `World` wrapper in tests).

3. **Event Maps & Two-Yaw Lattice (`engine/plan/src/events.rs`, `engine/plan/src/lattice.rs`):**
   - **Binade map (consumes P2's exact kernel once P2 lands):**
     - Represent binary64 coordinate $p = (-1)^s m \cdot 2^{e-52}$ as `(sign, exp, mantissa_u64)` (normal, subnormal, `±0.0`).
     - Compute exact dyadic division $d_j / U = N_j + r_j$ ($U = 2^{e-52}$) and round-to-nearest-ties-to-even (`N_j` if $r_j < 1/2$, `N_j + 1` if $r_j > 1/2$, `N_j + ((m + N_j) & 1)` at $r_j = 1/2$).
     - Compose the finite parity graph over period $P = 12$ (or general `cycle_period`), extract prefix + cycle, advance $q$ cycles in $O(1)$ via quotient/remainder (zero $q$-loops).
     - Find next binade-crossing / sign-flip tick via integer ceil/floor division; apply exact boundary tick; continue in next binade.
   - **Food & difficulty guard (consumes P4):**
     - Evaluate `world.difficulty()` and actual observed `state.food_level > 6` and `state.health > 0.0`.
     - If `state.food_level <= 6`, sprint words are ineligible at `tick = 0` (`FoodGuard { tick: 0, sprint_allowed: false }`); if `difficulty == 0` and `food_level > 6`, no food-drain crossing event is scheduled (matching `food_tick` at `movement.rs:1276`). No `fed` or fabricated bars.
   - **Inverse `Goal::touched`:**
     - Using `r = (0.6f32 / 2.0f32) as f64` (`shapes.rs:112`) and strict `<` / `>` (`shapes.rs:203-204`), invert `hull(before_box, after_box).intersects(goal.zone())` through the integer map + stored vertical-phase `y` eligibility (`cycle_level`).
   - **Two-yaw integer lattice (`lattice.rs`):**
     - Given displacement from `from` to `to`, bracket bearing with exact integer cross-products of adjacent `sin.bin` cycle displacement vectors $D_i, D_{i+1}$ (`0..65536`, lowest index on tie).
     - Include start transient `E_0` and yaw-switch transient `E_{i->i+1}` (carried motor state across `i -> i+1`), plus binade regime shifts from `events.rs`.
     - Solve $lo \le E + q_1 D_i + q_2 D_{i+1} \le hi$, $q_1, q_2 \ge 0$, using Euclidean quotient / floor-sum interval arithmetic over exact dyadic integers (work bounded by event regimes and native operand bits; **no** $q$-by-$q$ enumeration, **no** 65,536-yaw sweep in `solve`, **no** LP/SMT solver). Select minimum total ticks, then canonical `(q_1, q_2)`.
   - **Merged U1 event stream:** merge `(BinadeCrossing, FoodGuard, LoadedBoundary, ZoneLabelBoundary, YawSwitch)` ordered by tick.

4. **Uniform Corridor Presence Source (`engine/plan/src/source.rs`):**
   - `pub trait CorridorSource: law::World + law::codec::proof::Source`:
     - `fn corridor_query(&self, codec: &mut Interner, expected_label: LabelId, min_corner: BlockPos, max_corner: BlockPos) -> CorridorResult`
       where `CorridorResult` is `Clear` | `Exception { first_pos: BlockPos }` | `Unknown(codec::proof::Refusal)`.
   - `GeneratorSuperflatSource`:
     - Stores `floor_y: i32`, `floor_block: Block`, `air_block: Block`, `loaded_min: BlockPos`, `loaded_max: BlockPos`, `difficulty: u8`, `natural_regeneration: bool`, `scars: BTreeMap<BlockPos, Block>`, and `provenance_hash: [u8; 32]`.
     - `corridor_query`: checks coverage & loaded bounds in O(1); scans `self.scars.range(min_corner..=max_corner)` (BTreeMap range query — O(1) when empty, O(log S) when scars exist). Never scans 10⁶ cells when scars are empty; returns the first scar whose `LabelId`/role differs from `expected_label` as `Exception { first_pos }`.

5. **Single Entry Point `solve` (`engine/plan/src/solve.rs`):**
   - Signature:
     `pub fn solve<S: CorridorSource>(from: &PlayerState, to: Goal, kit: Kit, source: &S, store: &GeneratorStore, codec: &mut Interner, deadline_ticks: u64) -> SolveResult`
   - `Kit`: in U1, `Kit::BareLegs` (later U5/U8 extend `Kit` variants without changing `solve`'s stages).
   - `Stage` trace recorded on every call (`Vec<Stage>` inside `SolveResult`): `Bracket`, `ClosedCheck`, `Extend`, `Cut`, `Compose`, `Allocate`.
   - Execution flow (no terrain-kind branch anywhere):
     1. **Bracket (`Stage::Bracket`):** `lb = to.lower_bound(from)` (bare-legs kit light cone; `ub = None`).
     2. **Closed? (`Stage::ClosedCheck`):** if `ub == Some(lb)` (or `next_word` bracket closed), emit `Plan` and stop.
     3. **Extend (`Stage::Extend`):** derive `from`'s `LabelId` (via `source.block`/`loaded` + `codec`), look up the kit's biggest applicable word in `store.get(label)` for `(from -> to)` (bracketed yaw pair `[i, i+1]`, `count = 1` initially or `count` bounded by goal distance via `lattice` + `events`), compute the tube bounding box `(min_corner, max_corner)` from the word's trajectory envelope, and issue **one** `source.corridor_query(codec, label, min_corner, max_corner)`:
        - `Clear`: `events::solve_uniform_zone(...)` returns the exact `WordStep` chain (with `count`, merged events, exact stop tick `T`), setting `ub = Some(T)`. Proceed to `Compose` -> `ClosedCheck`.
        - `Exception { first_pos }`: proceed to `Cut`.
        - `Unknown(r)`: return `PlanOutcome::Unknown(r)`.
     4. **Cut (`Stage::Cut`):** split `[from, to]` at `first_pos`'s joint. In U1 (before U2 builds sets/joints), return `PlanOutcome::NeedsJoints { exception: first_pos, bracket }` cleanly with `Stage::Cut` recorded.
     5. **Compose (`Stage::Compose`):** combine backward sets/costs in integer `(min, +)` (in U1: sum `WordStep` ticks + transient + switch + remainder) and advance the committed prefix forward.
     6. **Allocate (`Stage::Allocate`):** select the open bracket whose narrowing could change the soonest decision (in U1: single open bracket `[lb, ub]` closes at `ub` or hands off at `NeedsJoints`).

### Exact Tests (`engine/plan/tests/zones_prereqs/u1.rs`)

1. **`u1_kill_test_17_million_line_peaceful`:**
   - Construct `GeneratorSuperflatSource` (Peaceful `difficulty = 0`, stone at `y = -61` + fixture air at `y >= -60`, zero scars, loaded range covering `[-100, -64, -100] ..= [1_000_100, 0, 330_100]`).
   - Prepare `GeneratorStore` off-command for `LabelId(stone/air)` and sprint-jump `F S j a s n` over the bearing's adjacent `sin.bin` yaw indices.
   - Call `solve` (wrapped in a `CountedSource` counting `block`, `loaded`, `corridor_query`, and `try_tick` calls) for:
     - Axis-aligned `A = (0.5, -60.0, 0.5)` -> `B = (1_000_000, -61, 0)` (and `Z` axis `(0, -61, 1_000_000)`).
     - Oblique `A = (0.5, -60.0, 0.5)` -> `B = (1_000_000, -61, 330_000)`.
   - Assert for every target:
     - `PlanOutcome::Ready(plan)` with `count` carried on every `WordStep` (`count > 100_000` on the main gait legs; transient and switch steps have `count = 1`), `lb <= plan.ticks`, `hot_tick_calls == 0`, `corridor_queries == 1`, and `block_reads <= 8` (label reads only; **zero** per-block or per-tick scan over 10⁶ blocks).
     - Re-run `B = (1_000_000, -61, 330_000)` with a second observed start shifted in `t`, `x`, `z`, `food_level` (`17` vs `20`) inside the same binade: **same `LabelId`**, **zero** re-preparation ticks.
     - Verify the oblique `B = (1_000_000, -61, 330_000)` plan's end state touches `Goal::touched` (via exact P2 binary64 map + final-tick witness replay of the tail ticks from the P2-predicted pre-landing state — NOT a 2.8M-tick full replay inside `solve`, and a full streaming witness check at `10^4` blocks or matched against P3's witness).
   - Measure release thinking time distribution over `n = 100` calls (`min / p50 / p90 / p99 / max` ns) and report against the ~5 µs Fermi floor / ~10 µs target.
2. **`u1_kill_test_8_one_mechanism_and_scar_cut`:**
   - Run `solve` on `GeneratorSuperflatSource` with **one scar** planted at `[500, -60, 0]` (`minecraft:stone` wall).
   - Assert the **exact same `solve`** function transitions `Bracket -> Extend -> Cut` (`PlanOutcome::NeedsJoints { exception: [500, -60, 0], .. }`), proving uniform ground and scarred ground share one entry point and `corridor_query` drives the cut.
   - Verify stage trace on clean superflat contains `[Bracket, ClosedCheck, Extend, Compose, ClosedCheck]` — any bypass skipping `Extend` / `CorridorSource::corridor_query` fails.
3. **`u1_peaceful_food_guard_and_no_fed`:**
   - Run `solve` with observed `food_level = 6` on Peaceful `GeneratorSuperflatSource`: sprint words are blocked by the unchanged law guard (`food_level > 6`), returning `Unknown` / non-sprint refusal with zero `fed` substitution.
   - Run with `food_level = 7`: sprint-jump succeeds with zero food-drain event scheduled on Peaceful (`difficulty == 0`).
4. **`u1_class_id_table_not_exhausted`:**
   - Intern 100,000 distinct motor recurrence keys into `GeneratorStore` while keeping `Interner::classes` small, asserting `MotorKeyId` does not trigger `PatternError::TableFull` on `ClassId(u16)`.

### Gate Commands (all offline, Idle priority, `CARGO_TARGET_DIR=D:\path-tracing-target\u1`)
1. `cargo test --offline --release -p plan --test zones_prereqs u1_ -- --nocapture` (must PASS; log latency distribution and operation counts to `D:\path-tracing-evidence\u1\u1.log`).
2. `cargo test --offline -p plan --test zones_prereqs` (all `t1`, `t2`, `t4`, `u1` tests PASS).
3. `cargo test --offline --workspace` (PASS, 0 failures).
4. `& .\jobs\1.2-footprints\verify.ps1` (13/13 mutants CAUGHT, sources restored).
5. **Kill Test 8 planted-fault gate (run in `jobs/u1/verify.ps1`):** temporarily plant a uniform-ground fast-path bypass in `solve.rs` (skipping `Extend`/`CorridorSource::corridor_query` when `source` has 0 scars, or bypassing `Stage::Extend`), run `u1_kill_test_8_one_mechanism_and_scar_cut`, assert test **FAILS**, restore `solve.rs` byte-for-byte, and re-run to **PASS**.

### What Terra Must NOT Touch
- `engine/law/src/movement.rs`, `engine/law/src/shapes.rs`, `engine/law/src/math.rs`, `engine/law/src/pattern.rs`, `engine/law/src/footprint.rs`, `engine/law/src/world.rs`, `engine/law/src/state.rs`, `engine/law/src/codec*.rs` (law and Slice B codec remain frozen and unchanged; footprint mutants must stay at exact call sites).
- `engine/native/src/*` and `bot/*` (no JNI or Java edits in U1; U4 wires `resident.rs` and `bot/`).
- `engine/plan/src/search.rs`, `engine/plan/src/flight.rs`, `engine/plan/src/reach.rs`, `engine/plan/src/polish.rs` (do not build Slice E or H; do not copy `wt/plain`'s `plain.rs`).
- Do NOT launch any game, server, or client.

---

## 3. What U1 Consumes from P2 and P4, and What Can Start Now

### Consumes from P2 (`wt/3.2d-3-P2`)
- **Binary64 integer binade map (`events.rs`):** exact `(sign, exp, mantissa_u64)` representation, dyadic division `d_j / U = N_j + r_j` with ties-to-even parity, finite parity graph prefix/cycle `O(1)` advancement over `q` cycles, and integer ceil/floor binade-boundary crossing solver.
- **Exact inverse `Goal::touched` (`events.rs`):** using `(0.6f32 / 2.0f32) as f64` half-width and strict AABB intersection (`shapes.rs:112, 203-204`).
- **Wait condition:** `D:\path-tracing-wt\3.2d-3-P2\jobs\3.2d-3-P2\last.md` currently shows the first run stopped on the literal-`0.3` premise while P2 reruns. Until P2 passes its release witness, poison check, and produces `D:\path-tracing-evidence\3.2d-3\T2\manifest.json`, `events.rs` (binade integer map + inverse stop) and the `10^6`-block exact endpoint assertion cannot be frozen.

### Consumes from P4 (`wt/3.2d-3-P4`)
- **Unchanged Peaceful food guard (`t4.rs`):** confirmed offline in `D:\path-tracing-wt\3.2d-3-P4\jobs\3.2d-3-P4\last.md:7` that `food_tick` skips debit at `difficulty == 0`, `update_sprinting` keeps `food_level > 6` on observed bars, and `state_answers` matches tick-for-tick without `fed` or saturation prediction.
- **Note on P4 live gate:** P4's offline test already gives U1 the verified guard behavior it needs for the offline Kill Test 17 gate (`learning.md:119`). P4's live `-Case T4Food` server/client check remains unexecuted (blocked by P4's offline launch rule) and must be scheduled separately before live flights (U3/U4), **not** blocking offline U1.

### What Can Start Now (Before P2 Lands)
1. Merge `wt/3.2d-3-P1` and `wt/3.2d-3-P4`'s offline `t4.rs` onto `wt/3.2d-3` (after judging P4).
2. Wait for Argon's `jobs\frazzoli\report.md` to complete so U1's design freezes with Frazzoli read in full (`HANDOFF.md:70`).
3. Meanwhile, Terra can start **U1 Part A** (independent of P2's binade arithmetic):
   - `count: u32` on `words::program::Arguments` (`program.rs`);
   - `GeneratorStore` keyed by `LabelId` with `MotorKeyId(u32)` (`plan/src/store.rs`, consuming P1's `t1.rs` motor recurrence key and `ReadCertificates`);
   - `CorridorSource` + `GeneratorSuperflatSource` (`plan/src/source.rs`);
   - `solve` entry point & `Stage` trace (`plan/src/solve.rs`) with a clean `BinadeMap` trait/adapter boundary for `events.rs` and `lattice.rs`.
4. Once P2 passes and merges onto `wt/3.2d-3`, Terra drops P2's exact kernel into `events.rs`, completes `lattice.rs`, and runs the full U1 gate (`u1_kill_test_17_million_line_peaceful` + `u1_kill_test_8_one_mechanism_and_scar_cut`).

---

## 4. Open Questions for the Integrator (Not for Terra)

1. **P3 (Two-Yaw Euclidean Lattice): separate prerequisite job vs. folded into U1 Slice D?**
   - *Context:* The prompt lists P1, P2, and P4, while `jobs\3.2d-3\spec.md:111` requires **both P2 and P3** before Slice D, and `jobs\unified-plan\plan.md:108,145,182` specifies two neighbouring yaws for the 10⁶-block line (`B = (1000000, -61, 330000)` has zero single-yaw hits in the 65,536 `sin.bin` table).
   - *Options:*
     - **A.** Launch P3 (`t3_two_yaw`) as a standalone premise job immediately after P2 lands (consuming P2's `t2.rs`), then start/finish U1's `lattice.rs`.
     - **B.** Fold P3's already-written spec (`jobs\spec-fixes\report.md:186-204`) directly into U1's `lattice.rs` + `u1.rs` (keeping `t3_two_yaw` as the isolated unit gate inside `zones_prereqs`).
   - *Recommendation:* **A if Codex 5h quota allows a quick isolated P3 check before U1 (stronger SR-71 isolation), otherwise B** (the spec in §2 already includes the exact Euclidean floor-sum construction so Terra has zero ambiguity either way).

2. **Frazzoli freeze timing (`jobs\frazzoli\report.md`):**
   - *Context:* `jobs\frazzoli\report.md` currently stops at line 20 (context header only) while Argon reads Frazzoli's paper in parallel, and `docs/HANDOFF.md:70` says "Frazzoli's paper read in full first".
   - *Options:*
     - **A.** Wait for Argon's `jobs\frazzoli\report.md` before launching Terra on U1 (check if Frazzoli's maneuver automaton changes `WordStep` / `GeneratorStore` / yaw-switch transient interfaces).
     - **B.** Launch U1 Part A (`count`, `GeneratorStore`, `GeneratorSuperflatSource`) now and reconcile any Frazzoli finding before freezing U1.
   - *Recommendation:* **A** (Argon is already running in parallel; reading its report before launching Terra prevents a wasted Codex run if Frazzoli's trim/maneuver exit-inside-entry condition refines `GeneratorStore`'s switch signature).

3. **P4's missing live `-Case T4Food` manifest vs. U1's offline gate:**
   - *Context:* `D:\path-tracing-wt\3.2d-3-P4\jobs\3.2d-3-P4\last.md:12-14` states `-Case T4Food -Live` was not run and no `T4\manifest.json` exists, so `verify.ps1 -Case Prerequisites -Tests T1,T2,T3,T4` would currently fail on missing T4 manifest.
   - *Options:*
     - **A.** Run `.\jobs\3.2d-3\verify.ps1 -Case T4Food -Difficulty Peaceful -Mobs Off -Live` separately (taking the rig lock on a quiet machine) to produce `T4\manifest.json`.
     - **B.** Accept P4's offline `t4_food` pass for offline U1 (Kill Test 17 is offline Peaceful) and require `-Case T4Food -Live` before U3/U4 live flights.
   - *Recommendation:* **B** for unblocking offline U1, while scheduling **A** before U4 (`ResidentRunner` / live `ClientboundSetHealthPacket` re-anchor).

4. **Whether U1 updates `resident.rs` / `bot/` or leaves `native/` untouched:**
   - *Context:* `docs/HANDOFF.md:70-73` places the million-line offline gate in U1, joints/sets in U2, `solve` + 21 tracks + `hardtrack` in U3, and the client runner (Slice F/G) in U4. Today `resident.rs:170-204` calls legacy `Planner` + `legacy_sentence_adapter`.
   - *Options:*
     - **A.** Keep `engine/native/src/*` and `bot/*` untouched in U1 (`solve` lives in `engine/plan`, gated offline on Kill Tests 17 and 8), and wire `resident.rs` in U3/U4.
     - **B.** Wire `resident.rs` to `solve` in U1 (which would fail live anyway because U4's runner and K's server resource rows don't exist yet).
   - *Recommendation:* **A** — keeps U1 strictly scoped to the kernel (`engine/words` + `engine/plan`), avoids touching `native/`/`bot/` before U3/U4, and matches `HANDOFF.md:70-73`.
