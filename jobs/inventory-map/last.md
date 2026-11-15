Written to [report.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/inventory-map/report.md).

# Inventory Report: Docs vs Code & Julien's Words (S132, S133)

> Read-only inventory across `docs/` (`engine.md`, `representation.md`, `scales.md`, `ladder.md`, `learning.md`, `footprints.md`, `server.md`, `vocabulary.md`, `PRINCIPLES.md`, `HANDOFF.md`), `docs/reference/` (`julien-2026-11-04.md`, `julien-2026-11-09.md`, `julien-2026-11-10.md`, `research-2026-11-05.md`, `leads.md`; `private.md` excluded), `engine/{law,world,words,plan}/src/`, `D:\path-tracing-wt\plain\engine\plan\src\plain.rs`, `bot/src/pathtracing/`, and `scripts/`.

---

## Summary

- **Mechanism Map Counts (35 rows in Part A):**
  - **BUILT:** `6`
  - **PARTIAL:** `14`
  - **DOC ONLY:** `15`

### Top 10 Blocking `DOC ONLY` Items

1. **Executor re-anchoring, certified tube clearance (`width.clearance`), and veto-risk pricing** â€” blocks closed-loop live play (`Recorder.playPlan` blindly feeds `planInputs[planTick++]`), safe execution under packet/clock jitter, Step 9 Play, and any local repair (`bot/src/pathtracing/Recorder.java:319-321`, `engine/words/src/segment.rs:117-118`, `docs/engine.md:80-86`).
2. **Tool, item-use, hotbar, and block-interaction actions in `Input`, `PlayerState`, `Program`, and Law** â€” blocks 5 of the 18 tracks (mud, doors, trapdoors, amethyst/wood placement), `Slice B / R-lego`, pearls/buckets/boats/mining conjugates, and every item in `Rediscovery` (`engine/law/src/state.rs:7-17,53-121`, `engine/words/src/program.rs:136-155`, `bot/src/pathtracing/TickInput.java`).
3. **Local repair (inverted footprint index per section, slack timing wheel, incremental `Field` updates)** â€” currently `flight.rs:134` rebuilds `ProposalTable::default()`, `Reach`, and `Field::build` from scratch on every click; blocks `R-retarget`, `S121` fast mind-changing near real players, and dynamic worlds (`docs/engine.md:56-79`, `docs/scales.md:7,109`).
4. **Separating word vs certificate vs decision cuts (`vocabulary.md` v3) + compound-word grammar (`writes`/`needs`)** â€” blocks `3.2d`, causes non-issue crouch-walk failures on `butoonpressureplates` and 430 redundant jump decisions on `straight800` (`engine/words/src/segment.rs:81`, `engine/plan/src/search.rs:405-414`, `docs/HANDOFF.md:45`).
5. **Double-entry obligation ledger, ruin barriers, identical-block restoration, and item `T`-prices-as-fields** â€” blocks any reversible world-editing move (`S32`/`S46` per-mission restoration) from composing safely (`docs/representation.md:147-150`, `docs/scales.md:121`).
6. **One mind in the server (server-side Fabric mod: packet injection, tracking filter, spatial index, delta stream, outcomes -> deviations)** â€” blocks `Slice C` (5 bots sharing a tool), `100` bots on a local server, and automated server-deviation port growth (`bot/fabric.mod.json:7` is `"environment": "client"` only; `docs/server.md:24-79`).
7. **Law read backwards (`writes`->`reads`), Pareto effect archive, FunSearch/LLM word invention, and idle Play-with-body ZPD slice** â€” blocks `learning.md` Phase 3 word discovery, `R-lego`, and Julien's `S32` litmus test (`docs/scales.md:39-47`, `docs/learning.md:3,36`).
8. **Hierarchical far-field multipoles (Barnes-Hut `theta`, velocity convex hull, constriction-only CRP portal tables, ALT landmarks)** â€” blocks `R-far` (10kâ€“1M block trips), `R-scale`, and `Tracked request #1 (FMM)` (`docs/scales.md` 4.3, `docs/HANDOFF.md:87`).
9. **Closed-form movement recurrences & generator chunk/world storage (`worldgen(seed)+edits`)** â€” blocks `Âµs`-scale long-horizon trajectory evaluation, `100kâ€“1M` block straight Fermi floors, and chunk memory scaling (`docs/engine.md:88-92`, `docs/representation.md:41-45`).
10. **Swarm local rules, worldtube chunk tickets (levels 33/32/31), and lazy bots (`push_pair` producer, ski rental, auction, Pigou/PIC)** â€” blocks `Slice C` multi-bot bottlenecks and `100..10^4` bot scaling (`docs/server.md:51-59,104-110`, `docs/scales.md:54-61,85-89`).

---

## Part A: Mechanism Map

| # | Mechanism / Invariant / Structure / Rule | Status | Evidence (Doc & Code) | Consumer | Duplicates / Unification |
|---|---|---|---|---|---|
| 1 | **Lab clock sync & 2000 TPS ceiling** (`/tick rate`, `fullTickDemand`, `ClockAudit`) | **BUILT** | Doc: `docs/ladder.md:37`, `PRINCIPLES.md:108,200`. Code: `bot/src/pathtracing/mixin/MinecraftClockMixin.java:19-48`, `ClockAudit.java:6-68`, `ClockQualification.java:4-8`. | `L0`, `feedback-loop` | â€” |
| 2 | **Footprints & `RecordingWorld` & `PatternTable<V>`** (FNV-1a 128, `Shape` interning, `u16` ids, binade `in_scope`) | **BUILT** | Doc: `docs/footprints.md:1-200`, `docs/ladder.md:71-75`. Code: `engine/law/src/footprint.rs:10-346`, `engine/law/src/pattern.rs:64-440`. | `L0`, `words`, `local repair` | â€” |
| 3 | **Tracks harness, RCON reset, stick `Pointer` & survival kit capture/recovery** | **BUILT** | Doc: `PRINCIPLES.md:100,176,186,206`. Code: `Pointer.java:1-56`, `Tracks.java:1-605`, `TrackStore.java:1-311`, `TrackMode.java:1-238`, `TrackHumanClient.java:1-74`, `TrackInventory*.java`, `TrackUsage.java:1-28`, `TrackRcon.java:1-58`, `TrackDetector.java:1-66`. | `L0`, `first flight`, `tracks` | Duplicate track/settle/touch conventions in Java (`Tracks.java`, `TrackDetector.java`) vs Rust (`flight.rs`, `goal.rs`). |
| 4 | **Braking input against motion** in base programs | **BUILT** | Doc: `PRINCIPLES.md:170`. Code: `engine/words/src/program.rs:136-155` (`tiny_programs()` includes back/reverse yaw). | `R1..R3` | â€” |
| 5 | **Monotone positive tick metric `T` & admissible `gap / C` lower bound** | **BUILT** | Doc: `PRINCIPLES.md:124`, `docs/engine.md`. Code: `engine/plan/src/goal.rs:7,33-36` (`C = 19.902886/20.0`), `engine/plan/src/search.rs:180,496`. | `Scoreboard`, `R1..Rfar` | `Goal::C` assumes no knockback/items/boats/elytra. |
| 6 | **26.2 registry extractors & static lookup tables** (`BlockDumper`, `models.tsv`/`maps.tsv`/`blocks.tsv`/`census.tsv`, `use-effects.tsv`) | **BUILT** | Doc: `PRINCIPLES.md:115,233`. Code: `bot/src/pathtracing/BlockDumper.java:34-229`, `engine/world/src/snapshot.rs:13-97`, `engine/world/src/items.rs:1-14`. | `L0`, `law`, `words` | `WorldExporter.java:1-262` exports live box state separately from `BlockDumper` / Anvil `.mca`. |
| 7 | **26.2 Law (`tick`/`try_tick`) & 5 Exactness Classes** (bit-exact movement, server resources, Elytra `Math.cos` exception) | **PARTIAL** | Doc: `docs/engine.md:15,27-39`. Code: `movement.rs:56-1097`, `shapes.rs:1-598`, `blocks.rs:1-240`, `attributes.rs:1-81`; **missing/refused**: `effects`, `boat`, `riding`, `fall_flying`, `flying`, Lava `entityInside`, `unsupported_fall_on`, `Inside::Unsupported` (`movement.rs:121-135,839,1054,1093`); `world_hook` (`world.rs:214`) not run; no GPU advisory floats, mobs, redstone, falling blocks, or per-word exactness choice (`vocabulary.md:115`). | `L0`, `R1..R10`, `R-brach` | â€” |
| 8 | **Non-movement law modules** (`ServerResourceEvent`, `MiningClock`, `push_pair`, `boat::tick`, `items::use_effects`) | **PARTIAL** | Doc: `docs/footprints.md:39-52`, `docs/scales.md`, `PRINCIPLES.md`. Code: isolated in `movement.rs:1134`, `mining.rs:104`, `entity_contact.rs:15`, `boat.rs:29`, `items.rs:8`; **zero** wiring into `movement::tick`, `Input`, `PlayerState`, or `engine/plan`. | `Slice B`, `Slice C` | `PlayerState` (`state.rs:53-121`) carries passive flags (`boat`, `using_item`, `health`, `food_level`) without inventory/clock/contact producers. |
| 9 | **Strict IEEE & 65536 `Mth.sin`/`cos` parity** across planner & law | **PARTIAL** | Doc: `docs/engine.md:15,27`. Code: `engine/law/src/math.rs:1-47`, `engine/words/src/program.rs:100`, `wt/plain/plain.rs:180`; **divergent**: `engine/plan/src/reach.rs:42-45` uses `f64::to_radians().cos()/sin()`. | `L0`, `R1` | Unify `Reach::turn` on `law::math::sin`/`cos`. |
| 10 | **Vanilla judge** (no Lithium/C2ME on server & bot client) | **PARTIAL** | Doc: `docs/engine.md:16`, `docs/HANDOFF.md:104`, `PRINCIPLES.md:101`. Code: `scripts/setup.ps1:17,78` separates `PlayerOnlyMods` from server/bot mods; no runtime/launch assertion verifying mod exclusion on rig. | `L0`, `Scoreboard` | â€” |
| 11 | **World reader (`WorldReader` Anvil `.mca`, `WorldSnapshot`, dimensions)** | **PARTIAL** | Doc: `docs/engine.md`, `PRINCIPLES.md:144`. Code: `region.rs:55-226`, `inflate.rs`, `nbt.rs`, `snapshot.rs:103-220`; **missing**: Overworld `-64..319` only (no Nether/End sheets), static box (panics outside bounds `snapshot.rs:167-169`), reopen `.mca` required for edits, no live delta ingestion. | `L0`, `R1..Rfar` | `region.rs`, `WorldSnapshot`, and `WorldExporter.java` are separate world representations. |
| 12 | **Yes/No Word Class (`class(state, reads)`) & non-issue equivalence** | **PARTIAL** | Doc: `docs/vocabulary.md:25-33`, `PRINCIPLES.md:254`. Code: `engine/words/src/class.rs:1-110`, `state_reads.rs:1-146`; **gap**: `butoonpressureplates` 2.8x slowdown (`HANDOFF.md:45`) shows unused block flags split air-equivalent regimes. | `R1..R3`, `butoonpressureplates` | â€” |
| 13 | **Word Programs (`Predicate` -> keys, aims, item use, block interact)** | **PARTIAL** | Doc: `docs/vocabulary.md:47-56`, `docs/representation.md:27`. Code: `engine/words/src/program.rs:1-155` has 14 `Predicate`s + 54 movement-only `tiny_programs()`, 64-yaw + `refined_yaw`; **missing**: item use, attack, hotbar slot, block interact/break in `Input` (`state.rs:7-17`) & `TickInput.java`. | `Slice A` -> `Slice B` | â€” |
| 14 | **Segment unit, event cuts, width (`early`/`late` + `clearance`) & record fields** | **PARTIAL** | Doc: `docs/vocabulary.md:9-12,66-86,113-117`. Code: `engine/words/src/segment.rs:1-141` cuts on discrete state OR `BodyBlock` change (`81`), records `Uses`/`Guards` and `early`/`late`; **missing**: `clearance` (`"not built"`, `117-118`), `writes`/`needs`/`obligations`, separating word/cert/decision cuts. | `R1..R3`, `straight`, `R-lego` | Overlaps with `docs/engine.md` certificate definition; unifying fixes `straight800` 430 cuts. |
| 15 | **Gaits & cycle discovery (`(segment, phase)` recurrence)** | **PARTIAL** | Doc: `docs/vocabulary.md:60-62`, `docs/representation.md:45`, `PRINCIPLES.md:256`. Code: `engine/words/src/gait.rs:10-146`, `engine/plan/src/arith.rs:67-125` (notes food clock unproved `64-65`), `D:\path-tracing-wt\plain\engine\plan\src\plain.rs:229-283`; no unified gait corridor query. | `R1..Rfar`, `straight` | **3 duplicate implementations** (`gait::Gait`/`Discovery`, `arith::Cycle`, `plain.rs` fold). |
| 16 | **`ProposalTable`, Zipf/Heaps/Surprise/Regret counters & symmetry detection** | **PARTIAL** | Doc: `docs/vocabulary.md:104-109`, `PRINCIPLES.md:79,256`. Code: `engine/words/src/table.rs:1-320` implements counters & `regret()`; `pattern.rs:15-48` keeps identity `FRAMES[0]` only (`13-14`, other 7 fail bit-exact replay); **gap**: `flight.rs:134` allocates a fresh empty `ProposalTable` per click. | `words`, `learning.md` | â€” |
| 17 | **Concrete-state planner / sets-backward point-forward (`3.2d`) & `plain` path** | **PARTIAL** | Doc: `docs/learning.md:15-20`, `docs/vocabulary.md:97-100`, `docs/HANDOFF.md:22,45`. Code: `main` `search.rs:1-513` runs A*/Field search with per-tick `arith::free_tick` (`arith.rs:32-55`), full-route `polish::slide` (`polish.rs:27-64`), per-click `Reach::compile`/`Field::build` (`flight.rs:134`); `wt/plain-path` `plain.rs:204-404` does presence+modulus+1 corridor check on empty lines; `3.2d-3` still in design. | `first flight`, `straight`, `R1..R3` | `search.rs` duplicates gait/arc checks vs `words`; `certify` + `slide` + `Recorder` duplicate full replays. |
| 18 | **Anytime `<= 1 tick` contract & off-game-thread execution** | **PARTIAL** | Doc: `docs/engine.md:107-110`, `docs/ladder.md:21`. Code: `flight.rs:141-144` budget check; `plan-track.rs:1-44` runs offline out-of-process; **missing**: live `<= 50 ms` thinking (measured 0.69â€“30.5 s), persistent engine process, non-`settled` start (`plan-track.rs:23`), `Bot.java:23,111` calls `StraightLine` synchronously on client thread. | `Pointer` live play, `Scoreboard` | Two unrelated planner entrypoints: `Bot.java`/`StraightLine.java` vs `Tracks.java`/`plan-track.exe`. |
| 19 | **Scheduled dynamic timeline & live deviation comparator** | **PARTIAL** | Doc: `docs/ladder.md`, `docs/server.md:45-49`. Code: `time.rs:1-68` defines `Scheduled` (unused by `flight.rs`, passes `Still`); `deviation.rs:1-80` checks fields vs `Recorder` and counts packets, but stops at first mismatch & skips 15 `NEVER` fields; `Recorder.playPlan` has zero live feedback. | `L0`, `first flight` | â€” |
| 20 | **Always-on invariants, inline hot buffers & fast offline/live audit loop** | **PARTIAL** | Doc: `docs/ladder.md:26,38,71-75`, `docs/scales.md:114`, `PRINCIPLES.md:62,64`. Code: `movement.rs:598` & `blocks.rs:13` use inline 32-slot buffers; L0 gate + Rust tests exist; **missing**: automatic per-run margin/refusal log, automated invariance checks (cone, unused item, thread determinism), `plan`/`words` slack allocator (heap `Vec`/`HashMap` everywhere). | `Scoreboard`, `L0..R10` | â€” |
| 21 | **Local repair: inverted footprint index per section, slack queue & incremental `Field`** | **DOC ONLY** | Doc: `docs/engine.md:56-79`, `docs/scales.md:7,109`. Checked: `engine/law/src/{footprint.rs, pattern.rs}`, `engine/words/src/{segment.rs, table.rs}`, `engine/plan/src/{reach.rs, flight.rs, search.rs, time.rs}` â€” none exist (`Field::build` & `ProposalTable` rebuilt fresh per click). | `R-retarget`, `Slice C` | Worldtube invalidation (`scales.md`, `server.md`, `engine.md`) all share this unbuilt index. |
| 22 | **Executor re-anchoring, certified clearance bounds & veto-risk pricing** | **DOC ONLY** | Doc: `docs/engine.md:80-86,103`, `docs/scales.md:114`, `PRINCIPLES.md:237`. Checked: `Recorder.java:319-321`, `Tracks.java`, `Bot.java`, `segment.rs:117-118`, `walk.rs` (`playPlan` blind-feeds `TickInput`; `clearance` not built). | `R1+noise`, `first live correction` | â€” |
| 23 | **Closed-form movement recurrences & generator chunk storage (`worldgen(seed)+edits`)** | **DOC ONLY** | Doc: `docs/engine.md:88-92`, `docs/representation.md:41-45,66`, `PRINCIPLES.md:120-121,233`. Checked: `region.rs`, `snapshot.rs`, `arith.rs`, `reach.rs`, `movement.rs`, `math.rs` (raw Anvil + dense `u16` snapshot + tick-by-tick simulation; only `BlockDumper.java:161-197` dumps block offset formulas). | `R-far`, `10^4 scale` | â€” |
| 24 | **Three grains (`Medium` tier) & coarse Hashlife merged segment cache** | **DOC ONLY** | Doc: `docs/representation.md:25-33`, `docs/scales.md:22-31`. Checked: `engine/words/src/*`, `engine/plan/src/*`, `pattern.rs` (only single-segment `Word` + `Tick` exist; no `Medium` or coarse merged Hashlife tier). | `R-far`, `straight100k` | â€” |
| 25 | **Compound words, word grammar & Bernstein write-set commutativity (`writes`/`needs`)** | **DOC ONLY** | Doc: `docs/vocabulary.md:11-12,74-93`, `PRINCIPLES.md:220,259`. Checked: `engine/words/src/*`, `engine/plan/src/search.rs` (flat single segments only). | `Slice B (R-lego)`, `Rediscovery` | â€” |
| 26 | **Creativity / R&D / Play pipeline** (law read backwards `writes`->`reads`, Pareto effect archive, FunSearch/LLM word invention, Play-with-body ZPD slice, move-37 log) | **DOC ONLY** | Doc: `docs/scales.md:39-47`, `docs/representation.md:70-75,125-128`, `docs/learning.md:3,36`. Checked: `engine/words/src/*`, `engine/plan/src/*`, `bot/src/pathtracing/*` (approved S93 to wait until after first flight). | `learning.md`, `R-lego`, `Rediscovery` | `scales.md` 4.1 R&D barbell & `learning.md` Step 9 Play describe the same discovery pipeline. |
| 27 | **Tools, item use, block edits & portals in planner** (bucket metric, placement/breaking conjugates, pearls, Ender chests, Nether sheet) | **DOC ONLY** | Doc: `docs/representation.md:47,68`, `docs/scales.md` 4.6, `PRINCIPLES.md:127,130-131,144,154-155,158,160`. Checked: `state.rs:7-17,53-121`, `program.rs:136-155`, `TickInput.java`, `plan/src/*`, `world/src/region.rs`. | `Slice B`, `5 blocked tracks`, `Rediscovery` | â€” |
| 28 | **Double-entry obligation ledger, ruin barriers, identical-block restoration & item `T`-prices** | **DOC ONLY** | Doc: `docs/representation.md:147-150`, `docs/scales.md:121`, `docs/engine.md`, `PRINCIPLES.md:129,166,201,223`. Checked: `state.rs`, `segment.rs`, `search.rs`, `goal.rs`, `flight.rs` (only post-hoc `TrackUsage.java` records items used). | `Slice B (R-lego)`, `S32/S46 restoration` | â€” |
| 29 | **Hierarchical far-field multipoles & universal scale interface `(LB, UB, footprint, deadline)`** (Barnes-Hut `theta`, velocity hull, constriction CRP tables, ALT landmarks) | **DOC ONLY** | Doc: `docs/scales.md:7-9,4.3`, `docs/engine.md:65`, `PRINCIPLES.md:122-123,125`. Checked: `engine/plan/src/{reach.rs, goal.rs, search.rs, time.rs}` (single-scale cell grid only). | `R-far`, `R-scale`, `Tracked #1 (FMM)` | â€” |
| 30 | **One mind in the server (Fabric server mod)** (tracking filter, `WAYPOINT_RECEIVE_RANGE=0`, spatial index, bulk save, packet injection, self channel, outcomes->deviations, sleeping=0) | **DOC ONLY** | Doc: `docs/server.md:24-49,79`, `docs/representation.md:133-143`, `PRINCIPLES.md:222,228`. Checked: `bot/fabric.mod.json:7` (`"environment": "client"`), `PathTracingMod.java`, `scripts/setup.ps1`. | `Slice C`, `100..10^4 bots` | â€” |
| 31 | **Lazy bots, worldtube chunk tickets (levels 33/32/31), presence tool & separate shared-memory IPC engine** | **DOC ONLY** | Doc: `docs/server.md:51-68,104-110`, `docs/scales.md:54-61,106-107`. Checked: `bot/*`, `engine/*`, `scripts/setup.ps1`. | `Slice C`, `100..10^4 bots` | â€” |
| 32 | **Swarm local rules & coordination** (ski rental, Graham slack, Bertsekas auction, Pigou queuing, Physarum, PIC fields) | **DOC ONLY** | Doc: `docs/scales.md:62-89`, `docs/representation.md:133-138`, `PRINCIPLES.md:105,178`. Checked: `engine/plan/src/*`, `engine/law/src/entity_contact.rs` (`push_pair` isolated). | `Slice C (5 bots)`, `100..10^4 bots` | â€” |
| 33 | **GPU Vulkan integer min-plus & bitboard light cones** (sparse vs dense Knudsen) | **DOC ONLY** | Doc: `docs/engine.md:116-120`, `docs/representation.md:118,139-142`, `PRINCIPLES.md:133,289`. Checked: `engine/Cargo.toml`, `engine/law/src/*`, `engine/plan/src/*` (unqualified until crossover). | `10^3..10^4 scale` | â€” |
| 34 | **Micro layout (`24B` hot bot, `1-6 KB`/chunk class space, SIMD tick deltas) & reverse-demolition `(max,+)` building** | **DOC ONLY** | Doc: `docs/scales.md:93,100-112`, `docs/representation.md:151-157`. Checked: `state.rs` (65 unpacked fields + heap `Vec`), `snapshot.rs` (`u16` dense box), `plan/src/*`, `words/src/*`. | `10^4 scale`, `Commissions (Phase 6)` | â€” |
| 35 | **Higher ladder rungs (`Legs`, `R1..R10`, `R-retarget`, `R-scale`, `R-far`, `Rediscovery`, `R-lego`, `R-brach`) & live mobs/redstone/Monte Carlo** | **DOC ONLY** | Doc: `docs/ladder.md`, `PRINCIPLES.md:128,137,177,193,198,204,293`. Checked: `engine/plan/src/*`, `movement.rs`, `scripts/play-track.ps1`, `HANDOFF.md:45,53`. | `Scoreboard`, `Phases 2-9` | â€” |

---

## Duplicates & Unification Opportunities

1. **Three separate gait / cycle detectors (`words`, `plan::arith`, `wt/plain`):**
   - `engine/words/src/gait.rs:10-146` (`Gait`, `discover_gaits`, `recurrence`, `Discovery`) vs `engine/plan/src/arith.rs:67-125` (`Cycle`, `layered`) vs `D:\path-tracing-wt\plain\engine\plan\src\plain.rs:229-283` (cycle fold). `search.rs:336` uses `arith::Cycle`, while `plain.rs` uses `words::recurrence` with its own folding loop and food inequality (`plain.rs:319-342`), and `arith.rs:64-65` skips the food clock. **Unify** into one `words::gait` module carrying verified displacement, food consumption, and corridor clearance.
2. **Tick-by-tick free-arc scanning vs single-pass worldtube verification:**
   - `engine/plan/src/arith.rs:32-55` (`free_tick`) runs `law::try_tick` on `Free` AND scans swept cells/extra collisions tick-by-tick (`S117` ES violation), while `walk.rs:75-111` (`certify`) records the tube tick-by-tick afterward, and `plain.rs:345-380` verifies corridor cell digests (`law_digest`/`inert`) once. **Unify** on checking a trajectory's worldtube once against the snapshot/partitioned world.
3. **Redundant full-trajectory replays during search, polishing, and playback:**
   - `search.rs:228,510` calls `certify`, `polish.rs:27-64` (`slide`) calls `certify` from **tick 0** for every Â±1 tick/aim joint nudge (causing the `O(L^2)` 23â€“30 s blowup on `straight800`, `HANDOFF.md:45`), and `flight::write_script` / `Recorder` replay again. **Unify** via local segment replay to bit-equal rejoin plus keeping a single valid witness.
4. **Two unconnected planners & track geometry conventions (Java vs Rust):**
   - `bot/src/pathtracing/Bot.java:23,111` calls Java `StraightLine.java:1-13` synchronously on the client thread for `goto`/`come`, while `Tracks.java` + `play-track.ps1` shell out to Rust `plan-track.exe` (hardcoding `tracks.txt` and refusing non-`settled` starts at `plan-track.rs:23`). Likewise `TrackDetector.java:5` (`touchesB`) & `Tracks` (`onA`/settle) duplicate `goal.rs:19` (`Goal::touched`) & `flight.rs` settle (`HANDOFF.md:45` 1-tick convention discrepancy). **Unify** `Bot.java` and `Tracks.java` behind one persistent off-thread Rust planner protocol and one shared goal/start definition.
5. **Fresh per-click `ProposalTable` / `Reach` / `Field` vs `Reach::turn` trig mismatch:**
   - `engine/plan/src/flight.rs:134` instantiates a fresh `ProposalTable::default()`, compiles `Reach::compile` over flat floor + 64 aims, and builds `Field::build` from scratch on every click; meanwhile `ProposalTable::invent` re-simulates `tiny_programs()` on misses (`search.rs:428-470`), and `Reach::turn` (`reach.rs:42-45`) uses `f64::to_radians().cos()/sin()` instead of `law::math::sin`/`cos`.
6. **Fragmented law modules vs `PlayerState`/`tick()`:**
   - `MiningClock` (`mining.rs:104`), `ServerResourceEvent` (`movement.rs:1134`), `push_pair` (`entity_contact.rs:15`), `boat::tick` (`boat.rs:29`), and `use_effects` (`items.rs:8`) are duplicated/isolated outside `tick()` and `Input`, requiring caller-side synchronization.
7. **Overlapping doc specs across `scales.md`, `learning.md`, `vocabulary.md`, `server.md`:**
   - `vocabulary.md` segment/width/writes vs `engine.md`/`scales.md` certificates/worldtubes (coupling caused `butoonpressureplates` / `straight800` splits; unify via 3-boundary split in `vocabulary.md:113-117`); `scales.md` 4.1 (backwards law / effect archive / R&D slice) vs `learning.md` Step 9 (Play / ZPD puzzles / bottleneck discovery) should share one effect-archive pipeline; `engine.md` local repair, `server.md` worldtube tickets/lazy bots, and `representation.md` `H_live` all share one unbuilt `Footprint` section index.

---

## Part B: Julien's Words â€” Lost / Unimplemented / Open / Repeated

### 1. His words that no register row cites (or distinct ideas omitted by a broad key citation)

**A. Turns with NO register row citing their S/X key at all (`docs/PRINCIPLES.md:97-305`):**

- **S38** (`julien-2026-11-04.md:217-219`):
  > "bug, when doing a track it deletes sulfur spike from my inventory"
  *(Substantive bug report in track inventory handling; zero register row.)*
- **S102** (`julien-2026-11-10.md:255-257`):
  > "i've just got a new google ai pro plan so our starting goal will be to add that new companion to the clan and have a terra researcher make reserach for which model will be better for which task so, argon, hydrogen, opus, sonnet, glm, astra, terra. you will be orchestrator and since you have so much to manage you'll use some context so you need to give my antrhopic context filled up more than the others and do some massive parallel work too, we are lucky we have a lot to do."
  *(Referenced in `AGENTS.md` / `clan.md` / `HANDOFF.md:8`, but **completely absent from `PRINCIPLES.md` register** â€” jump skips directly from `S101` at line 276 to `S104` at line 277.)*
- **S103** (`julien-2026-11-10.md:260-262`):
  > (Fragments preserved from session of 13â€“14 Nov): `"terra is very often idle you should have a list for it"; "check this session's work before building on it"`
  *(No register row in `PRINCIPLES.md`.)*
- **S110** (`julien-2026-11-10.md:291-297`):
  > "what's the thinking time for these tracks?" / "(asked to measure the split between load and search) yes" / "fermi on the thinking floor for such simple straight line paths"
  *(Cited in `HANDOFF.md:45`, **no register row in `PRINCIPLES.md`** â€” register skips from `S109` at line 282 to `S112` at line 283.)*
- **S111** (`julien-2026-11-10.md:300-305`):
  > "i'm pretty sure we're orders of magnitude too high, we need a track that's long and straight."
  *(Cited in `HANDOFF.md:45`, **no register row in `PRINCIPLES.md`**.)*
- **X8** (`julien-2026-11-10.md:213-215`):
  > '"backward reachable sets combined with forward choices, rather than the direct exact-state A* search." in my algo how do they work together? i'm surprised it's not the same algo. how does it connect to cells and what are cells really?'
- **X10** (`julien-2026-11-10.md:221-223`):
  > "anyway for testing and training i'm thinking that parkour maps are perfect, ones with a nice progression system to but also puzzle maps after the goal is to know in depth minecraft mechanics to go from point A to point B in the shortest amount of time. Is there other things like that that you can think of that would be very great for training?"
- **X11** (`julien-2026-11-10.md:225-227`) & **X12** (`julien-2026-11-10.md:229-231`):
  > "fighting too matters" (X11)
  > "i want to be the best in the world at everything in minecraft with that bot because those are such simple yet powerful ideas that i'm sure i can get there, combat is spacing and mechanical knowledge and you have to be smart about it and there's so much creativity same with parkour and they have skin in the game then once everything is done with perfect knowledge i'll be able to cook something from incomplete knowledge and then from just the screen's info data" (X12)
  *(Combat/PvE creativity and the post-perfect-knowledge roadmap to incomplete knowledge and screen-only vision are completely unrepresented in `PRINCIPLES.md` register.)*
- **X14** (`julien-2026-11-10.md:237-239`):
  > "anyway is what i'm currently aiming for and doing is there some kind of unification that can do better for cheaper that i'm missing?"
- **X15** (`julien-2026-11-10.md:241-243`):
  > "why am i using A* in the first place? Wouldn't monte carlo like in chess be better?"
- Procedural / meta turns with no register row: **S7** (`julien-2026-11-04.md:33`, resume after limit), **S9** (`julien-2026-11-04.md:41`, wrapper introducing X1â€“X6), **S31** (`julien-2026-11-04.md:175`, `"check and if so make me a report on what you think the glm issue is"`), **S73** (`julien-2026-11-10.md:58`, `"where are we in the overall plan?"`), **S74** (`julien-2026-11-10.md:62`, `"pretty sure in the previous session i had a fable work on it but got cut off..."`), **X7** (`julien-2026-11-10.md:211`, GitHub intro), **X13** (`julien-2026-11-10.md:235`, summary pointer to `private.md` only, mentioned inside S101 at `PRINCIPLES.md:276`).

**B. Distinct ideas inside cited S-turns that the corresponding register row omits:**

- **S54** (`julien-2026-11-09.md:156`, register `PRINCIPLES.md:230` cites only `"store only what's seen and the causal chains"`):
  > "Also why don't we run bots on gpu?"
- **S44** (`julien-2026-11-09.md:17,19,23,41,43,47,59,87,93-95,108`, register `PRINCIPLES.md:216` cites only the opening goal/cleanup paragraph):
  - Readwise/library search on cooperation in nature (`17`): `"from my books in the library, readwise and cooperation in nature find out about local vs global rules that allow the creation of rich cooperation of individuals."`
  - Farms, shulkers, top-down System-2 LLM overseer & bot classes/factions (`19`): `"there will be an LLM controlling the swarm and interacting with other players on the server more in a top down system 2 style maybe even there will be factions or classes of bots"`
  - Incomplete transformations (`23`): `"The representation must preserve the value of incomplete transformations."` (quoting, followed by `"interesting"`)
  - Cooperative instant pearl chunk-loading (`41`) & self-growing vine ladder for an incoming swarm (`43`): `"using a vine way in advance for an incoming swarm could be high yield action as a self growing ladder that needed only a single block to be placed"`
  - 3D printing / inverse-removal building & worker placement (`59, 61`): `"the inverse process feels the same like particles removing all the matter in an efficient manner... 3D printers do it but not in parallel... Now that I think of it it's really a problem of worker placement more than construction one"`
  - Fast human-bot dialogue / low-latency LLM harness (`93-95`): `"imagine you want to say 'hey buddy give me the wood' it would instantly realize that you have a shulker full of wood... need a fast and cheap end to end LLM"`
  - Point-cloud FMM + adaptive resolution (`108`): `"something like a point of cloud with FMM and adaptative resolution could be better?"`
- **S77** (`julien-2026-11-10.md:74`, register `PRINCIPLES.md:252` groups S75â€“S77 only as vocabulary/reading docs):
  > "for the shape what about a half eaten cake? is a lily pad on water an issue you can walk on it on swim under it?"
- **S96** (`julien-2026-11-10.md:175`, register `PRINCIPLES.md:270` omits crafting from inventory):
  > "also unrelated but if you're on a frozen river and you have logs the bot need to think to make a boat."
- **S131** (`julien-2026-11-10.md:417`, register `PRINCIPLES.md:302` omits derivative control):
  > "derivate are expensive so maybe we control in derivative directly? no idea really."

---

### 2. Register rows marked `Adopted` whose mechanism Part A found `DOC ONLY`

*(All references are `docs/PRINCIPLES.md` row line numbers; `Part A` rows 21â€“35 hold the `DOC ONLY` evidence.)*

- **`PRINCIPLES.md:105` (`S6`, `Adopted` as stigmergy):** World-stored reuse / cached trails / refraction landmarks (`ProposalTable` is wiped per click, `flight.rs:134`; no stigmergy trails).
- **`PRINCIPLES.md:113` (`S6`, `Adopted`):** Merged states & learned ordering that only schedules (`Phase 10`).
- **`PRINCIPLES.md:123` (`S6`, `Adopted`):** Hierarchy across all scales in space & time + scale rung (`R-scale`).
- **`PRINCIPLES.md:125` (`S6`, `Adopted`):** CRP-style precomputed tables customised per toolkit (`Phase 4`).
- **`PRINCIPLES.md:126` (`S6`, `Adopted`):** Goal-free shaped space & `R-retarget` rung (`Field::build` is one-shot per fixed goal B).
- **`PRINCIPLES.md:127` (`S6`, `Adopted`):** Water bucket rewriting the vertical metric & cliffs as one-way edges (`Phase 5`).
- **`PRINCIPLES.md:128` (`S6`, `Adopted`):** Monte Carlo robustness under jitter & stochasticity (`Phase 9`).
- **`PRINCIPLES.md:130` (`S6`, `Adopted`):** Run, jump into void, place block under feet via arc/face intersection.
- **`PRINCIPLES.md:131` (`S6`, `Adopted`):** Ender pearl & food/health token accounting (`Phase 7`).
- **`PRINCIPLES.md:133` (`S6`, `Adopted`):** Vulkan GPU min-plus/bitboards + 16-thread float law (`Phase 10`).
- **`PRINCIPLES.md:139` (`X2`, `Adopted`):** Coordinates per regime checked against the law (raw Cartesian + 64 yaws only).
- **`PRINCIPLES.md:141â€“143,146` (`X2`, `Adopted`):** Min-plus portal tables preserving every opening (`141`), adaptive hierarchy refined by slack (`142, 146`), and local repair via footprint index (`143`).
- **`PRINCIPLES.md:144` (`X2`, `Adopted`):** Nether as a second sheet via portals (`Phase 11`; `region.rs`/`snapshot.rs` Overworld-only).
- **`PRINCIPLES.md:145` (`X2`, `Adopted`):** Closed forms between events (tick-by-tick `movement.rs` everywhere except `plain.rs` cycle displacement folding).
- **`PRINCIPLES.md:148` (`X2`, `Adopted`):** Improvement detection when new tools/shortcuts make old routes poor (`Phase 4`).
- **`PRINCIPLES.md:150,158,166,171,201` (`X2, X4, S12, S13, S32`, `Adopted`):** Edits as conjugates (`AÂ·BÂ·Aâ»Â¹`), bucket undo traveling with bot, Silk Touch / `identical block` restoration, dropped void block token cost, per-arrival restoration.
- **`PRINCIPLES.md:154,155,160` (`X4, X5`, `Adopted`):** Body folded into terrain, tools rewriting classes of places, toolkit-parameterized reach (`state.rs`/`program.rs` have zero tools/items).
- **`PRINCIPLES.md:177,195` (`S17, S18`, `Adopted`):** `R-brach` elytra brachistochrone (`movement.rs:131` refuses `fall_flying`).
- **`PRINCIPLES.md:178` (`S18`, `Adopted`):** Multi-bot scoreboard reporting cost per bot (`Phase 10`).
- **`PRINCIPLES.md:185` (`S23`, `Adopted`):** Local consistency, repair in both directions, ridges (`representation.md` claims 13â€“14).
- **`PRINCIPLES.md:193` (`S20`, `Adopted`):** Mobs precomputed as law (`Phase 8`).
- **`PRINCIPLES.md:198` (`S32`, `Adopted`):** `R-far` 10,000-block journey in naturally generated terrain.
- **`PRINCIPLES.md:204` (`S32`, `Adopted`):** `Rediscovery` rung (litmus test, combining functions like Legos).
- **`PRINCIPLES.md:220â€“223` (`S46`, `Adopted`):** Compound words with grammar + LLM temporary farms (`220`), fixed R&D compute slice (`221`), bots as real players driven by one engine in the server (`222`), per-mission T and obligation passing between bots (`223`).
- **`PRINCIPLES.md:241` (`S64â€“S65`, `Adopted`):** New `scales.md` mechanisms (law read backwards, Pareto effect archive, FunSearch, move-37 log, chunk tickets 33/32/31, presence tool, Barnes-Hut `theta`/hull/ALT, ski rental/auction/Pigou/PIC, reverse-demolition `(max,+)`, shulker/Nether hub logistics, 24B bot state, 1â€“6 KB/chunk class space, SIMD deltas, timing wheel, double-entry obligation ledger).
- **`PRINCIPLES.md:258` (`S83`, `Adopted`):** Three decoupled boundaries (word vs certificate vs decision).
- **`PRINCIPLES.md:267` (`S93`, `Approved` `learning.md`):** Play-with-body ZPD slice, sets backward / point forward / one replay, bottleneck learning. *(And **`PRINCIPLES.md:276` (`S101`, `Approved`)** removing planner decision at body-block crossings (`vocabulary.md:117`) remains `DOC ONLY` on `main` at `segment.rs:81` & `search.rs:405-414`.)*

---

### 3. Register rows still `Open` from before 14 November

*(Register cutoff: S104 at `PRINCIPLES.md:277` marks 14 November; rows below are from 4â€“13 November, `PRINCIPLES.md:97â€“276`.)*

1. **`PRINCIPLES.md:132` (`S6`, 4 Nov) â€” `Open`:** `"boats nullify fall damage; lava is an issue"` â€” *"the boat fall rule has not been checked in 26.2's code yet."*
2. **`PRINCIPLES.md:172` (`S13`, 4 Nov) â€” contains `Open` clause:** `"Basically don't modify the world to get around there's many ways to move in minecraft."` â€” *"Open: whether a tool use that leaves no trace (a water bucket placed and taken back) counts as a modification"* (later framed by S32 at line `201` as same state on arrival at B).
3. **`PRINCIPLES.md:188` (`S23`, 5 Nov) â€” contains `Open` clause:** `"these bots will form an army of builders to speak of simply"` â€” *"Open: goals as reach regions (`representation.md`, 'Still open')."*
4. **`PRINCIPLES.md:196` (`S13`, 4 Nov) â€” `Open`:** `pricing consumables: "i'm unsure here honnestly"` â€” *"Proposal: spending a consumable needs a budget he gives per command (zero by default)..."*
5. **`PRINCIPLES.md:202` (`S32`, 5 Nov) â€” `Open, later`:** Limit velocities and inputs to his own human capability â€” *"an optional human-limits profile, after the benchmark (Phase 12). Nothing built."*
6. **`PRINCIPLES.md:215` (6 Nov chat) â€” `Tracked` / open design:** `"The server on which the bots run can be improved on too keep that in mind. I hate redundant compute..."` â€” *"Bot server design and the particle-system analogy are open."*
7. **`PRINCIPLES.md:245` (`S67`, 11 Nov) â€” `Open`:** Class extrapolation & no made-up numbers (`1/16`) â€” *"the class in `engine/words` is provisional; no chosen number enters the record (`HANDOFF`, row 2)."* (Still listed under `"Open for Julien:"` in `HANDOFF.md:65`.)
8. **`PRINCIPLES.md:247` (`S68`, 11 Nov) â€” `Open`:** `"yes or no questions, interesting kinda ES coded in light cones / world tubes intersections so booleans and bounding"` â€” *"the form the class takes if a cut is needed: the law's own comparisons."*
9. **`PRINCIPLES.md:254` (`S79`, 11 Nov) â€” `Open` clause:** `"whether it is more efficient to have bigger or smaller certificates â€¦ maybe certificates should be rougher since they won't be broken often â€¦ we need to fermi this too"` â€” *"Open: certificate size, GLM lane `certificate-size`."*
10. **`PRINCIPLES.md:255` (`S80`, 11 Nov) â€” `Open`:** `"so certificates are entropy dependent? so basically only based on player's agency..."` â€” *"a certificate lasts until a wildcard's light cone (humans, the server's randomness as cones, our own execution noise) can reach its worldtube; deterministic events are checks, not ends (the integrator's reading, to agree with him)."*
11. **`PRINCIPLES.md:272` (`X9, X16`, 13 Nov) â€” `Open`:** `"My goal is to make a representation where you don't need a policy because the map is clear on where to go"` / straight lines & contention points â€” *"his own account of how he navigates, to hold the planner against."*

*(Related pre-14 Nov pending/parked rows not labeled `Open`: **`PRINCIPLES.md:122`** FMM multipoles `[Tracked, design open]`; **`183`** least action & **`211`** claim 15/recurrence probe & **`248`** S69 proposal `[Answered, awaiting ruling]`; **`271`** S97 A\*/cells `[nothing built until he rules]`; and **`244`** S66 easter eggs, **`246`** S67 debug world type, **`257`** S82 agonbox `[Parked]`.)*

---

### 4. Requests he made twice or more (and `HANDOFF.md` tracking/owner status)

#### A. Already tracked in `docs/HANDOFF.md:83â€“94` with an owner (8 items)

1. **Multipoles (FMM) for how T acts at a distance** (`S6`, `~10x` in ES) â€” Owner: `integrator, Phase 3` (`HANDOFF.md:87`).
2. **Hundreds -> thousands of bots with shared work / swarm / one server mind** (`S18`, `S35â€“S36`, `S44â€“S45`, `6 Nov chat`, `S53`, `S64`) â€” Owner: `integrator, Phases 2 and 5` (`HANDOFF.md:88`).
3. **100Ã— iteration / accelerated testing clock** (`S6`, `6 Nov`, `S32`, `S44`) â€” Owner: `integrator, Phase 1` (`HANDOFF.md:89`).
4. **Creativity that arises, stays, and compounds cheaply** (`S32`, `S44`, `S46`, `S64`, `S97â€“S98`, `S122`) â€” Owner: `integrator` (`HANDOFF.md:90`).
5. **Explanations in his vocabulary, grounded in the whole context** (`S67`, `S75â€“S77`) â€” Owner: `integrator` (`HANDOFF.md:91`).
6. **Idiot-index pass & Fermi floors (micro, meso, macro, holistic)** (`S45`, `S83`, `S110`, `S120`, `S131`) â€” Owner: `integrator, after the first flight` (`HANDOFF.md:92`).
7. **Primary sources read directly; cases & extremes; book list & Readwise** (`S42`, `S44`, `S64`, `S86â€“S87`, `S89â€“S90`, `S116`) â€” Owner: `integrator` (`HANDOFF.md:93`).
8. **Use every meter before the integrator's runs out / leverage GLM massively** (`S20`, `7 Nov`, `S47`, `S51`, `S70`, `S100`, `S102`, `S104`, `S123`, `S125`) â€” Owner: `integrator` (`HANDOFF.md:94`, plus GLM queue `HANDOFF.md:39`).

#### B. Repeated requests WITHOUT a tracked item + owner in `docs/HANDOFF.md:83â€“94`

1. **No made-up fake numbers (cuts/zones/buckets must emerge from laws and needs)** â€” `S67` (`julien-2026-11-10.md:28`: `"you know how i hate made up numbers"`), `S124` (`376`: `"not already strating to fake it with fake data"`), `S132` (`423`: `"don't already start with made up fake numbers they should arise based of needs"`).
   - *HANDOFF status:* **No tracked row with owner** (`HANDOFF.md:65` only lists `"Open for Julien: the class (S67â€“S69)"`).
2. **Stay ES-faithful: never re-test or model tick-by-tick; use worldtubes, certificates, and generators** â€” `S44` (`julien-2026-11-09.md:25,100`), `S58` (`172`: `"just don't forget about ES again"`), `S83` (`julien-2026-11-10.md:120`), `S117` (`333`: `"not something you re-test tick by tick"`), `S118` (`337`: `"we don't run on ticks for the world model, ticks are just an execution layer"`).
   - *HANDOFF status:* **No tracked row with owner** (ad-hoc Argon audit & `3.2d-3` addendum only).
3. **Run bots / min-plus / parallel search on GPU (Vulkan)** â€” `S6`/`X1` (`julien-2026-11-04.md:29,123`), `S44` (`julien-2026-11-09.md:71,75`), `S54` (`156`: `"Also why don't we run bots on gpu?"`), `S118` (`julien-2026-11-10.md:337`), `S131` (`409`).
   - *HANDOFF status:* **No tracked row with owner** (`"the GPU"` appears unowned in Roadmap step 6, `HANDOFF.md:77`).
4. **Question A\* & cells; find the representation where search/policy disappears (non-Euclidean / continuous / wave-particle / 4D)** â€” `S6`, `S39â€“S40`, `S97` (`julien-2026-11-10.md:181`), `X9`/`X15`/`X16` (`219,243,249`), `S130â€“S132` (`402,409,423`: `"whether we actually need A* like algos"`).
   - *HANDOFF status:* **No tracked row with owner**.
5. **Long-distance & straight extremes (10k -> 100k -> 1M blocks)** â€” `S32` (`julien-2026-11-04.md:183`: `"10k blocks journey"`), `S111` (`julien-2026-11-10.md:304`), `S120` (`345`), `S131` (`413`: `"one million block straight path"`).
   - *HANDOFF status:* **No tracked row with owner** (`HANDOFF.md:22` currently targets `straight800` via `plain-path`).
6. **Elytra (`R-brach` early) & Boat physics / racing / river crafting / lava clutches** â€” Elytra: `S17`, `S18`, `S97`; Boats: `S6`, `S44` (`19`), `S96` (`175`: boat on ice river + crafting from logs), `S97` (`181`), `S98` (`191`), `S122` (`368`), `S131` (`409`).
   - *HANDOFF status:* **No tracked row with owner** (`HANDOFF.md:41,53,63` mention isolated `1.4d`, test list, and anomaly 10).
7. **Minimize real thinking time in `T` (`<= 1 tick` / instant movement unless hard)** â€” `S6`, `S90` (`151`), `S110â€“S111` (`293â€“304`), `S120` (`351`: `"the bot should move instantly"`), `S124` (`376`), `S131` (`407â€“411`).
   - *HANDOFF status:* **No tracked row with owner** (operational target in `HANDOFF.md:22,45`).
8. **Handle sudden commands & change your mind fast (especially near real players)** â€” `S6` (`julien-2026-11-04.md:29`), `X16` (`julien-2026-11-10.md:249`), `S121` (`364`: `"changing your mind fast is extremely important"`).
   - *HANDOFF status:* **No tracked row with owner**.
9. **Combat / PvE mechanics & human-like dialogue / LLM overseer** â€” Combat/PvE: `S97` (`181`), `X11â€“X12` (`227â€“231`); LLM overseer / fast dialogue (`"hey buddy give me the wood"`): `S44` (`19,93â€“95`), `S46` (`124`).
   - *HANDOFF status:* **No tracked row with owner** (`HANDOFF.md:53,77` list them as future/step-6 items only).
10. **Building / gathering / worker placement / multiple objectives & farms** â€” `S23â€“S24` (`julien-2026-11-04.md:109,113`), `S32` (`189`), `S44` (`julien-2026-11-09.md:19,59â€“61,93`).
    - *HANDOFF status:* **No tracked row with owner** (Roadmap step 6 only).
11. **Extreme parkour, speedrunner techniques & self-generated ZPD puzzles** â€” `S32` (`179`), `S87â€“S89` (`136â€“147`), `S97â€“S98` (`181,189`), `X10` (`223`), `S122` (`368`), `S131â€“S132` (`413,423`).
    - *HANDOFF status:* **No tracked row with owner** (Roadmap steps 2â€“3, `HANDOFF.md:73â€“74`).
12. **Properly formulate questions for Julien (`so what do you want from me?`)** â€” `S78` (`julien-2026-11-10.md:78`), `S100` (`200`).
    - *HANDOFF status:* **No tracked row with owner** (`PRINCIPLES.md:253,275` note it as followed process).
13. **Full Rust port of interactions (breaking/placing/entities), not just bare movement** â€” `S44` (`julien-2026-11-09.md:9`), `S46` (`124`), `S51` (`144`), `S108` (`julien-2026-11-10.md:284`).
    - *HANDOFF status:* Tracked via `PRINCIPLES.md:109â€“111` (Codex integrator owns) and `HANDOFF.md:39` GLM queue, not in `HANDOFF.md:83â€“94`.
