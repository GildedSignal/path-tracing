# Backward state-set wave: steps, fences, stairs, and parkour ladder

**Goal:** a click whose way crosses a step, a fence, a stair or a gap is answered by the one loop. The corridor's first exception along the word hands its takeoff states to learning. Learning keeps exact sets by content key. Prefix, site and tail compose in one backward min-plus wave. His tracks climb rung by rung offline, each rung's script landing on B on the predicted tick, ready for the integrator to fly.

Builder: Astra 6.2. Worktree `D:\path-tracing-wt\flight-ladder-sets`, new branch `wt/flight-ladder-sets` from `main` (code base `7b7aaaee` plus this spec). `wt/flight-ladder` (`c74c4a5c`) is reading material only. Do not merge or cherry-pick it: its `joints/cut.rs` is the design this job replaces.

The map (`map/`) is the truth. The rationale, Fermi and blocker list for every decision below are in `jobs/backward-sets-spec/report.md`; read it first. The integrator commits, flies and pushes. You never run `git commit`, `launch.ps1` or `gpt.ps1`. You use no network, game, server or client, and you never open `server/` (his world).

Copy the git-ignored `engine/world/tests/fixtures/world` from `C:\Users\PASCAL\Desktop\path tracing` if your worktree lacks it. Build into `D:\path-tracing-target\flight-ladder-sets`. Write evidence to `D:\path-tracing-evidence\flight-ladder-sets\`.

## His words (verbatim)

- S239: "always test in game that's the only kind of test that matters"
- S161: "what we can't have two paths like that "easy" or "hard" it should be the same mechanism"
- S131: "the terrains complexity should be a mirror of compute's complexity here our path should take µs not tens of ms."
- S166: "hard tracks are tracks that have many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that."
- S120: "i don't care where they are just if they are present first which should be cheap and reduce by a lot the considerations"
- S232: "it should use the biggest bracket and if it fits use it, that's it period"
- S67: "What's that 1/16 of a block? Explain it to me because it feels like a made up number for no reason rather than emerging on properly selected constraints and laws and you know how i hate made up numbers."
- S134: ""A flower is air." unless i need to place a block and it bothers me."

## Where it stands (measured on `main` today, `report.md` section 0)

- `flowers` already reaches B: 48 ticks, his best 48.
- `stonesimple` reaches B: 85, his best 86.
- Every other overworld rung is `NeedsJoints` at `solve.rs:269`.
- The wave exists (`joints.rs:287`, `family_wave.rs:77`), and so do `Family::learn` (`families.rs:110`) and `learn_family` (`family_wave.rs:15`). Nothing calls them.
- Fourteen blockers (B1–B14 in the report) stand between them and acting. The decisions below fix each one.

## The design (reference blueprint — full S237 agency)

Follow the map's principles and architecture: one mechanism (no terrain/block branching), monotone backward state-sets, no search in acting, store the generator, bit-exact against `law::try_tick`, and the scoreboard gates below. Sections 1–9 are a blueprint, not frozen micro-formulas: whenever a local formula, guard, predicate, key, or helper hits a law edge case or you find a simpler way that upholds the principles and passes the gates, fix or simplify it directly from the law, document the change in `report.md`, and keep going.

### 1. Keys, nodes and the resource guard (`joints.rs`, `families.rs`)

1. **`EntryTemplate::write_key` (`joints.rs:38`) and `pinned_key_axis` (`families.rs:97`) skip these fields:**
   - `F::t`;
   - every field with `StateField::is_resource()` (`health`, `food_level`, `saturation`, `exhaustion`, `food_tick_timer`);
   - `F::fall_distance`, `F::yaw`, `F::pitch`, `F::input`.

   Everything else stays as it is today: the free axis keeps only its exponent bits, and the support is stored relative to `floor(position)`.
2. **The guard replaces `Resources::same_bits`** (`joints.rs:25`) everywhere: `consume`, `equivalent_points`, and both `same_bits` checks in `family_wave.rs`, without crediting unguaranteed `food_tick_timer` regen against landing damage.
   - Measure unmitigated damage once at learn time whenever a piece changes health: wrap `&world` in a local `plan`-crate `NoRegen<'a, W>(&'a W)` (`natural_regeneration() -> false`, zero edits to `engine/law`) starting at `start.health.max(20.0)` and record `damage_no_regen = (start_h - end_h).max(0.0)` (zero extra work on non-damaging words).
   - `Guard` carries `class = codec::resource_guards(food, health)`, `fall_distance`, and exact resource bits `(health, food_level, saturation, exhaustion, food_tick_timer)`.
   - A piece admits `e` when `class` and `fall_distance` match AND either `f64::from(e.health) > damage_no_regen` (safe margin: freely admits across timers/saturation/exhaustion at difficulty 0) OR exact resource bits match (razor-thin fallback when survival depends on timer-gated regen).
   - In `wave`/`route` (§6), carry additive `damage_no_regen` across chained edges (never a hardcoded `0.0`) so multi-edge drops cannot die downstream.
   - `UseInterval` (`families.rs:7`) keeps the fifth component (health delta, `end.health − start.health`). Difficulty 0 only (`solve.rs:243`); note that on the type.
3. **`JointStore::node`** (`joints.rs:156`) interns by `write_key(s, 0, None)` plus the guard's `class`. The stored state is a copy with `t = 0`, as `law/src/pattern.rs:317` does.
4. **Certificates use entry-relative ticks:** `before.t − entry.t`. Every `edit` call passes `(0, u64::MAX)`.

### 2. One learning operation: `learn_family`; a point is a family of width 0

1. **`Family::learn`'s `stop` reads the world too** (via a small trait `pub trait StopPred { fn stop<W: World>(&self, world: &W, before: &PlayerState, after: &PlayerState, tick: u32) -> bool; }`, since `World` is not object-safe). Pass `&recording` in `replay` (`families.rs:92`) and in the pinned tail (`:210`, keeping `before` in the loop) so every read made by `stop` enters the piece's footprint certificate; pass `&counted` in the bisection predicate (`:172`, `:179`, where `before` is the state before the probed tick).
2. **A pin is the first tick where contact rises:** `after.horizontal_collision && !before.horizontal_collision` (at `:90` and in `expected_pin`).
3. **When `lo == hi`, replay once:** `right = left.clone()`.
4. **`FamilyPiece` keeps its own reads.** Add `reads: Vec<(u32, Footprint)>`: the timed footprints of the lo and hi replays and of every pinned tail, with entry-relative ticks and the per-tick hull fill now in `family_wave.rs` (the "H2" block). Add `actual: [Option<usize>; 2]`: the index into `Image::Points` of the variant the law actually produced at lo and at hi.
5. **`learn_family`** (`family_wave.rs:15`):
   - It takes `&impl StopPred` (same `stop` trait as `Family::learn`).
   - It captures each piece's certificate from `piece.reads`. Delete the re-replay loop at `:27-38`.
   - It builds `step`, `ticks` and `to` from the `actual[0]` variant when one exists. For a `Map` image it uses `predict(lo)`, unchanged.
6. **Delete what becomes dead:** `JointStore::learn` (`joints.rs:186`) and `MirroredReach` (`joints.rs:338`) are expected dead, and so are `file`, `attempt` and `Attempt` if nothing else reaches them. Let the compiler decide.

### 3. Words, their stop events and their aims (learning only, never acting)

1. **The stop predicate of every site word** returns true at the first tick where any of these holds:
   - **goal touch:** `goal.touched(before, after)`;
   - **landing:** `!before.on_ground && after.on_ground`;
   - **new contact:** item 2 of section 2;
   - **step-up:** `before.on_ground && after.on_ground && after.position.y != before.position.y`;
   - **grounded stall:** `after.on_ground && after.movement_this_tick.is_none()` (the law's own `< 1e-7` movement gate, `families.rs:75`, which stops sneak-at-edge holds and wall pushes from an already-colliding state cleanly without burning `max_ticks`);
   - **ledge takeoff (rising overhang):** `after.on_ground && can_fall(after, world) && !(before.on_ground && can_fall(before, world))`, where `can_fall(s, w)` is zero-offset `law::can_fall(s, w, 0.0, 0.0, f64::from(s.step_height))` (`movement.rs:561`), called through `world` (`&recording`) so its support probe enters the certificate footprint. Because `move_player` resolves `Axis::Y` before `Axis::X`, zero horizontal offset aligns the stop with losing vertical support on the next tick (adjust depth to `f64::from(s.gravity)` only if a measured drop onto a sub-step surface needs it; document any law-driven refinement in `report.md` and keep going);
   - **leaving the site:** `after.bounding_box()` no longer intersects the demand's site box (section 4, item 4).

   The word budget is `Arguments::max_ticks` (256). When it runs out, the word is `Searched`.
2. **Programs:** every id in `words::tiny_programs()` whose `held_input(args)` is `Some`, in id order. That is 24 ids. No program is named or hand-picked.
3. **Aims** (table yaw indices, `lattice::yaw(i)` for the word's `refined_yaw`):
   - the pair `lattice::bracket(dx, dz)` toward `goal.center()`;
   - then, by increasing horizontal distance from the takeoff, the pair toward the centre of each `Face` from `FaceInventory::build` over the demand's site box whose cell differs from the source's base (a scar) or whose top differs from the takeoff support's top;
   - then the four axis indices `0, 16384, 32768, 49152`, which are exact along an axis.

   Remove duplicate `(program, yaw)` pairs. The face centre is an ordering choice: aims that were never tried leave the demand `Searched`, never `Forbidden`.
4. **The family axis:** the dominant axis of the takeoff's velocity (`|vx| >= |vz|` gives 0, otherwise 2). If the velocity is zero, use the dominant axis of `lattice::direction(yaw)`.

### 4. The cut: first exception along the word, the window, the site (`solve.rs`)

1. **Checkpoints.** These are the phase-0 states of the extend's words, in order: checkpoint 0 is `from`, then every period boundary of every emitted word. A counted word with count `q` gives cycles `0..=q`.
   - Read a checkpoint inside a counted word from its `Span` with `events::advance_cycles` (exact). Write a helper that truncates that word at cycle `j`; it returns the `WordStep` with `count = j`, its ticks, and its `StateAt` end.
   - `box(c)` is the corridor box (the same `lo`/`hi` construction as `solve.rs:265-266`) of all motion through checkpoint `c` plus one more period from `c`, ticked with `Motion::tick` over the gait's rows.
2. **First exception along the word.** Only when the full crossing's query is an `Exception`, bisect for the smallest `c*` in `0..=C` whose `corridor_query(box(c*))` is not `Clear`. The box grows with `c`, so presence is monotone and the bisection is exact. Its cell is the exception reported upward. `source.rs` stays as it is.
3. **The window.** The checkpoints `c <= c*` whose horizontal distance to `c*` is at most `R`, the horizontal displacement of the period after `c*`. If `c* = 0`, the window is `{from}`.
4. **The site box.** `box(c*)` joined with the window's bodies, then expanded:
   - by `ceil(R)` cells in x and z;
   - by 1 cell down;
   - by `ceil(apex) + 1` cells up, where `apex = max(phase.after.position.y) − gait.top` over the gait's phases.
5. **Where an extend may run.** It runs from a state `s` only when all of these hold:
   - `s.main_supporting_block` is at `source` floor `y`, or `s` is the click's `from`;
   - `gait.entry(s, top)` returns a phase;
   - the click's goal is set.

   Otherwise the demand's window is `{s}`. Its site box is `s`'s body box, expanded by the click's `ceil(R)` horizontally, 1 down, and `ceil(apex) + 1` up. This is because `GeneratorSuperflatSource::support_cell` (`source.rs:199`) always answers `floor_y`.

### 5. Learning's queue for one click: the site (`solve.rs` drives it, `joints.rs` stores it)

1. **Extend edges.** `pub struct ExtendEdge {pub from:usize, pub to:Option<usize> /*None = B*/, pub ticks:u64, pub words:List<WordStep,12>, pub lo:BlockPos, pub hi:BlockPos}` in `JointStore.extends: HashMap<BlockPos /*goal cell*/, Vec<ExtendEdge>>`.
   - At the cut, file one edge from `node(from)` to `node(w)` for every window checkpoint `w` other than `from`. Its words are the extend truncated at `w`; its box is `box` at `w`.
   - When an extend from a demand is `Clear` to B, file a tail edge `s -> None` with the extend's words and corridor proof, and do not learn at `s`. That is S232.
2. **Demands.** A demand is an exact state, or a family: two or more window states, or the image of a `Map` piece, sharing one `with_axis` key, with domain `[min, max]` on that axis.
   - The queue is a binary heap keyed by `(arrival + ceil(gap(s) / v_ref), sequence)`, where `arrival` is the ticks from the click and `v_ref` is the click's extend's horizontal distance divided by its ticks. This key orders the queue only.
   - A demand is dropped when `arrival + goal.lower_bound(s) >= UB`, the best route found so far. That drop is a proof.
   - A demand whose node is already settled with an earlier or equal arrival is skipped.
3. **Popping a demand `s`:**
   - (a) If a found record exists for its key and domain, push the exits of the pieces already learned, with no law tick.
   - (b) Otherwise try the extend from `s` (section 4, item 5). If it is `Clear`, the demand becomes a tail edge. If it is an exception, the window of `s` becomes new demands, linked by extend edges.
   - (c) Otherwise, call `learn_family(s, axis, domain, phase, program, args, stop)` once for every candidate word (section 3). `phase` is the gait phase of `s` when `gait.entry` returns one, and 0 otherwise; `args = Arguments {refined_yaw: Some(lattice::yaw(i)), ..Default::default()}`. Each new piece's exits become demands, with arrival + piece ticks: `Points` give one node each (only the `actual` variant when the domain has width 0); `Map` gives `predict(lo)` when the width is 0, and otherwise a family demand over `[map(lo), map(hi)]`.
4. **Stop** when one of these holds:
   - the queue is empty;
   - the smallest sound key in the queue is at least UB (the bracket is closed);
   - `work.law_ticks` spent this click reaches `PATTERN_LAW_TICKS × exceptions`.

   `PATTERN_LAW_TICKS = 12_000`, from the map's "about 10³ verifications × 12 ticks" per new joint pattern (`map/detail/representation.md`, thinking floors). `exceptions` is the full crossing's count from the corridor query. Record the click's demand as `Searched {work}` when it stops without closing.
5. **No other search exists.** There is no forward search over points in acting, no per-tick `PlayerState` copy in acting, and no branch on block names.

### 6. The wave and the walk (`joints.rs`, `family_wave.rs`)

1. **`backward()`** (`joints.rs:111`) also relaxes the goal's extend edges. A tail edge goes to the swept terminal with cost `ticks`.
2. **`family_wave::extend`** (`family_wave.rs:77`):
   - Index the family rows and the point nodes by key: one `HashMap<Vec<u64>, Vec<usize>>` per free axis.
   - Precompute `(pattern, piece) -> EdgeId` once; this replaces `:91`.
   - Compare each variant only with the targets whose key matches; this replaces `:101` and `:142`.
   - Include extend edges in the point relaxation at `:150`.
   - Iterate until stable. If the iteration count passes nodes + rows, that is a bug: return an error, do not loop silently.
3. **`route(start, goal)`** (`joints.rs:304`):
   - If the store has no nodes, return `None` with no work, so flat ground pays nothing.
   - Otherwise probe `write_key(start, 0, None)` plus the guard. If no node admits `start`, return `None` without computing a wave.
   - Otherwise compute `wave(goal)`, which is cached per goal cell, and walk.
4. **The walk.** At node `n`, `via = field.via[n]`:
   - An **extend edge** pushes its words; then go to `to`, or finish at B.
   - A **piece** is checked with `consume` (the guard); its word's ticks come from `actual`.
     - Only inside a wider pinned piece with no `actual` for that entry does the walk replay that one word with the law. Count those ticks in a new `Work.acting_law_ticks`.
     - The exit is the node, or the `Map` prediction, which leads to the family row containing it.
     - Remove the `family_entries` exclusions at `:295` and `:311`.
   - **Rebase** every pushed `WordStep`: `expected_end_t` and `end.t` are the previous end plus `ticks`, starting at `start.t`.
5. **Marks.** `FoodGuard` goes on the first word, `GoalBoundary` on the last, and `YawSwitch` wherever `refined_yaw` changes between neighbours, as `solve.rs` does today. Learned words carry no `Span`; extend words keep theirs.

### 7. `solve()` (`solve.rs:226`)

1. **The stage order** for every terrain: `Bracket, ClosedCheck, Extend, Cut, [Site{demands:u32, law_ticks:u64, nanos:u128}], Compose, ClosedCheck, Allocate`. `Site` is a new `Stage` variant, present only when the cut found an exception.
   - `Cut` is the same function for both paths. On `Clear` it returns the extend's words unchanged.
   - The `route` probe at `:234` stays first.
2. **On an exception:**
   - Run the site (section 5).
   - Then `route(from, to)`. `Ready` carries `bracket {lb, ub: Some(T)}`.
   - Otherwise return `NeedsJoints {exception (the cell along the word), exceptions, bracket: {lb, ub: None}}` with the explanation `site searched: demands=…, law_ticks=…, pieces=…`.
3. **`ThinkingCosts`** gains `site_ns`, `site_law_ticks` and `site_demands`.

### 8. In his game (`engine/native/src/resident.rs`, `bot/src/pathtracing/ResidentRunner.java`)

1. **A `law::codec::proof::Source` implementation for the resident `Source`:** `cell(t, p, mask) = codec::cell(known_block(p)?, loaded(p), mask)`, mapping a missing cell to `Refusal::Coverage`; `environment` through `codec::environment`.
2. **Invalidation.** After a delta of kind 1, 2 or 3 is applied (not after the world-join reset), and only when `store.joints.has_certificates()`, call `store.joints.borrow_mut().edit(&codec, p, (0, u64::MAX), &source)` for the edited cell (kind 2) or for every cell of the section (kinds 1 and 3).
   - `JointStore::edit` also drops the extend edges whose box contains `p`, and sets `field = None`.
3. **The corridor grows to what the law read.** `GeneratorSuperflatSource` records the first cell for which `cell()` returned `Coverage` (`first_uncovered: Cell<Option<BlockPos>>`, read through `pub fn first_uncovered()`).
   - After `solve`, `plan()` widens its box to include that cell ± 4 (its existing margin) in x and z, clipped to the received bounds, rebuilds the corridor and solves again.
   - It repeats until nothing is uncovered or the box stops growing. The store keeps everything learned in between.
4. **`inputs()` checks the plan before it is sent.** The replay must first touch B on tick `plan.ticks`, using the same touch rule as `flight.rs`. Otherwise refuse with `replay deviates: first touch {..} predicted {..}`.
5. **The wire.** `WORD_PLAN` appends four words after the inputs: the plan's corridor `x0, z0, x1, z1` (`i64` as `u64`), the hull of every word's `certificate.min/max`.
   - `ResidentRunner.java` expects the length `9+n+count*3+4`, bounds its input loop at `9+n+count*3`, and sets `corridor` from the last four words. This replaces `:63`.
   - Change both sides in the same change.

### 9. The site key: measured, not reused

- At every `learn_family` that files pieces, compute the digest of:
  - the entry key, with each horizontal coordinate replaced by its `(p − floor(p)).to_bits()` and its exponent;
  - the word, `(program, yaw index)`;
  - the sorted footprint answers relative to the entry's floor cell and entry tick: `(Δpos, Δtick, mask, answer)`.
- Count `would_be_translation_hits`: digests that are equal across different absolute offsets. Report the count. Do not reuse across offsets in this job.

### 10. Tests (only what the game, through the law, judges)

1. **New `engine/plan/tests/sets.rs`: one witness test.**
   - Worlds: a `law::BoxWorld`, with a stone floor at y = 0 over x 0..200 and z −8..8, plus either a one-high stone step at x = 100 or a floor gap at x 100..103.
   - Entry: the state at phase 0 after `SPRINT` is held from rest at x = 60.5 until its velocity bits repeat.
   - Yaw indices 49152 (+x, exact) and 50176 (+5.625°).
   - Domain: x in `[98.6, 99.0]`. The site box for the stop is x `[90, 110]`, y `[-4, 4]`, z `[-8, 8]`, and the goal is a cell outside it, so the goal never touches.
   - Learn the family with the section 3 stop, then sample 2^6, 2^10 and 2^14 entries evenly on the ordered lattice, plus ±8 ulps around every computed cut. Replay each with `law::try_tick`.
   - Assert, at every resolution:
     - the sampled partition, by signature and contact letters up to the first pin, equals the computed pieces;
     - every sampled exit matches `predict()` (or for `Points`, one declared variant differing from the others only in `minor_horizontal_collision`) bit-for-bit across all non-resource motion fields (`!f.is_resource() && !matches!(f, F::t | F::yaw | F::pitch | F::input | F::fall_distance)`, matching section 1.1; resources including sprint exhaustion travel as `UseInterval` + `Guard`, never as centimetre cuts on clamped points), AND `piece.uses.contains(&start, &got.end)` holds;
     - the piece count is the same at every resolution.
   - Print the counts; do not pin them.
2. **`real_time.rs`: a decorated-ground series.**
   - Two sources from `GeneratorSuperflatSource::from_world` over a `law::BoxWorld` on `[-80, -64, -80]..[80, -50, 80]`: one plain stone; one decorated with the pattern `(31x + 17z) mod 5` = 0 dandelion, 1 `oak_button[face=floor,facing=north,powered=false]`, 2 `stone_pressure_plate[powered=false]` (all at y = −60), 3 the floor cell as dirt, 4 the floor cell as oak planks.
   - 60 goals evenly spread in the square, from standing at the centre.
   - Assert: the decorated source has 0 scars; T and words are equal per goal across the two sources; 0 refused; median ≤ 15 µs.
3. **Kept and green:** `real_time.rs` (both medians ≤ 15 µs, 0 refused, the repeat series), `hard_track.rs`, `deviation.rs`, `law` `replay` and `jar`, `world`, `native`.

## The ladder (offline first, every rung, in this order)

The controls stay exactly as they are: `straight100` 288, `straight300` 845, `straight800` 2,248, `stonesimple` 85, `flowers` 48.

Then, in order: `grassjumpeasy`, `flowers`, `stonestaircase`, `fences`, `fencestraightbridge`, `fencemaze`, `practice1`–`practice10`, `birchparkourmedium`, `mangrovejumpeasy`, `doorcorridor`, `trapdoorparkour`.

From `engine/`:
```
$env:CARGO_TARGET_DIR='D:\path-tracing-target\flight-ladder-sets'
cargo build --release --offline -p plan --bin plan-track
$env:PLAN_MEASURE_REPEAT='1'
& D:\path-tracing-target\flight-ladder-sets\release\plan-track.exe <track> --start settled --mode solve --out D:\path-tracing-evidence\flight-ladder-sets\<track>.jsonl
cargo test --release --offline -p plan --test real_time --test sets -- --nocapture
cargo test --release --offline -p plan --test hard_track --test deviation
cargo test --release --offline -p law --test replay --test jar
cargo test --release --offline -p world
cargo test --release --offline -p native
```
- `solve_track`'s `MEASURE` line (`flight.rs:116`) adds `site_law_ticks`, `demands`, `pieces`, `nodes`, `repeat_law_ticks`, `acting_law_ticks`, `lb`, `ub`, `slide_yes`, `slide_no` and `would_be_translation_hits`. It also adds the stage list.
  - `slide_yes` and `slide_no` count, over the route's joints, `Field::slide_want` against the interval its upstream piece reaches. No slide is executed.
- **Practice tracks** run with `--world D:\path-tracing-evidence\flight-ladder-sets\practice-world`. That world folder is the integrator's to make, from his `diversity_parkour` dimension at DataVersion 4903, produced only by the game itself.
  - If the folder is missing, or the reader refuses its version, the rung's row is that refusal.
  - Never relax the version check, and never use a hand-converted world.

**Per rung, report one row:**
- track; predicted T; his best (the `his` value `MEASURE` prints);
- the first command's thinking (µs) and the repeat (ns);
- law ticks prepared (gait), site law ticks, demands, pieces;
- `[lb, ub]`; repeat law ticks (must be 0) and acting law ticks;
- or the first refusal: cell, word (program, yaw), bracket, record status.

A rung that cannot be reached is written as its refusal, and you go on to the next.

## Gates and falsifiers (each must be reported with its raw numbers)

1. **One mechanism (kill test 8, catalog 1.2).**
   - The stage lists of `straight800` and of a fence rung differ only by `Site{..}`.
   - `git diff main -- engine/plan/src | grep '^+' | grep -i -E 'fence|stair|flower|trapdoor|door|slab|carpet|minecraft:|DemandAt'` prints nothing.
   - Plant a fast path once (`if exceptions == 0 { return early }` before `Cut`): the stage gate must turn red. Then remove it.
2. **Compute mirrors terrain (kill test 9).** Site law ticks and corridor queries for every rung, against `exceptions`. `straight100/300/800` show 0 site law ticks.
3. **Repeat.** The same command again prepares 0 law ticks and gives the same T on every rung that reached B. Report `repeat_ns` against the 1 µs floor; anything above 100 µs is called out.
4. **Flat ground.** Medians ≤ 15 µs from standing and while moving, 0 refused, unchanged after a site has been learned in the same store. Add one series to `real_time.rs`:
   - insert one stone block into the same source's `scars` at `[0, -60, -200]`;
   - solve once from the start to `[0, -61, -210]`, which learns that step's site;
   - then repeat series 1 on the same store.
5. **Pieces are few (kill test 11, catalog 1.3):** `sets.rs` is green.
6. **A flower is air (catalog 1.4):** the decorated series is green. The unused-item half waits for kits; write that down.
7. **Honest reds to report, not to fix:**
   - the site law ticks a second command from a start one ulp off costs, on `fencemaze` and on `grassjumpeasy` (kill test 4);
   - `would_be_translation_hits`;
   - `fencemaze`'s T against his 66 (turns only on contact);
   - the share of T spent on site words above the floor layer on `birchparkourmedium` and `stonestaircase`.
8. **Expected against seen.** For every rung, `write_script`'s replay touches B on the predicted tick. At each word boundary, the replayed state's `write_key` must equal the planned end's. List every field that differs as a deviation.

## Stop rule (S237 agency)

**Do NOT stop (`STATUS: BLOCKED`) for spec/formula edge cases, guard/predicate/key bugs, witness refinements, or wall-clock timing noise.** You own the implementation from first principles: fix any local law issue directly against `law::try_tick`, simplify where the principles allow, document the decision and raw evidence in `report.md`, and keep pushing straight through Sections 1–10, the full offline ladder, and Gates 1–8 to `STATUS: DONE`.

Stop with `STATUS: BLOCKED` **only** if an external environment block (missing permissions/sandbox/fixture you cannot create) makes progress impossible, or if climbing the ladder fundamentally cannot work without violating **one mechanism** (adding terrain-specific branches or search in acting).

## Output

Write `jobs/flight-ladder-sets/report.md` in your worktree as you go, with these parts:
1. For each of sections 1–10: how the code now does it, with file:line.
2. The ladder table.
3. Each gate with its raw numbers.
4. Net lines per crate, `engine/` and `bot/` separately.
5. What is left: translation reuse, 2D families, turns before contact, extends from raised supports, the slide, the arc certificate in learning, the direct word runner. For each, the rung where it bit and by how many ticks or µs.

The integrator flies each rung you reach in his game; the flight is the gate.
