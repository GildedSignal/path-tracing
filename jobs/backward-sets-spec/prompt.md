# Freeze closed backward state-set spec for Astra (`flight-ladder-sets`)

**Goal:** Design and freeze the exact, closed implementation spec (`jobs/flight-ladder-sets/prompt.md` + `jobs/backward-sets-spec/report.md`) connecting `solve.rs` corridor exceptions directly into the existing monotone backward state-set wave (`joints.rs`, `families.rs`, `family_wave.rs`) with site pattern hashing, so Astra climbs his tracks under one mechanism without inventing greedy forward `cut.rs`.

Designer: Claude Opus (read-only over code; write ONLY inside `jobs/backward-sets-spec/` and `jobs/flight-ladder-sets/prompt.md`). Base checkout: `C:\Users\PASCAL\Desktop\path tracing` (`main` at `7b7aaaee`). Do not edit `engine/`, `bot/`, `map/`, or any other worktree.

## Why this design must freeze before Astra codes

- Astra's previous checkpoint (`c74c4a5c` on `wt/flight-ladder`) replaced `joints.rs` / `family_wave.rs` with a bespoke greedy forward DFS (`joints/cut.rs`: aim straight at a block face, clone `PlayerState` per tick), which failed to turn, compose momentum intervals, or scale across sites.
- Meanwhile `main` (`7b7aaaee`) **already contains** `Family::learn` (`engine/plan/src/families.rs`), `JointStore::learn_family` / `wave()` / `route()` (`engine/plan/src/joints.rs`), and `family_wave::extend` (`engine/plan/src/family_wave.rs`), plus `-1,845` lines of dead A\* clutter removed.
- Yet `solve.rs:234-279` only calls `store.joints.borrow_mut().route(from, to)` on cached routes and immediately returns `PlanOutcome::NeedsJoints` at `solve.rs:269` on any `CorridorResult::Exception`.

## Read these exact files first

1. `C:\Users\PASCAL\Desktop\path tracing\map\{goal,principles,architecture}.md` and `map\detail\{representation,learning,scales}.md` (especially `architecture.md` §3, `learning.md` §0–§2, and Kill tests 8–14).
2. `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\{solve.rs, joints.rs, families.rs, family_wave.rs, faces.rs, source.rs, store.rs, flight.rs}` and `engine/native/src/resident.rs`.
3. `C:\Users\PASCAL\Desktop\path tracing\jobs\es-audit\report.md` (especially sections 1, 4, 6), `jobs\set-rederive\proposal.md`, and `jobs\emergent-properties\catalog.md` (entries `1.2`, `1.3`, `1.4`, `1.5`, `1.6`).

## Decisions this spec must lock down completely (leave ZERO design guesses for Astra)

1. **How `CorridorResult::Exception` invokes the one loop (`architecture.md` §3 steps 3–5):**
   - Exact prefix handoff: where along `crossing` before `first_pos` the takeoff state `s_pre` and approach window sit, how `Family::learn` + `JointStore` discover candidate joint sets (approach + jump/step + landing + turn/yaw aims on `65,536`-entry sine table) within budget, and how a tail after the defect recursively calls `solve()` onto `B` (so `straight -> defect -> straight` composes cleanly in `(min, +)` without re-searching uniform segments).
   - How multi-defect chains (`fencemaze`, `stonestaircase`, `practice1–10`) compose backward sets `(families/pieces/points/holds)` across consecutive joints preserving momentum `v` (no cell-centered reset).
2. **Site pattern key & caching (`ES 6`, `scales.md`, `learning.md`):**
   - Exact canonical content key of a site (relative `legs_presence` / shape footprint around the defect, invariant to translation / decorative flowers), how `GeneratorStore` caches & invalidates it (`ReadCertificates` footprint), and exact guarantee that a repeated command / second sighting reuses cached sets in microseconds (`O(0)` cold search).
   - Explicitly prohibit greedy forward DFS (`DemandAt`-style per-tick `PlayerState` cloning inside `RecordingWorld`), per-terrain/block branches (`stairs`, `fences`, `flowers`), or grid quantization (`1/16`).
3. **Wire & execution compatibility (`main` path):**
   - Exact interaction with `events::Map` (`Extension` + tail `Crossing`) vs `solve_track` and `resident.rs` (expanding corridor/observation bounds cleanly when detours/turns step wider than the initial `±4` halo), plus which `es-audit` path wins belong in this pass vs later (e.g., arc certificate on free air segments, local slide in continuous sets, keeping flat ground median `<= 15 µs`, `0` refused).
4. **Honest falsifiers & ladder gates inside `jobs/flight-ladder-sets/prompt.md`:**
   - Rung order: `grassjumpeasy`, `flowers`, `stonestaircase`, `fences`, `fencestraightbridge`, `fencemaze`, `practice1–10`, `birchparkourmedium`, `mangrovejumpeasy`, `doorcorridor`, `trapdoorparkour`.
   - Exact metrics per rung (`predicted T`, `his best T`, `thinking ns/µs` cold vs repeat, `law_ticks`, or exact first `Refusal`/bracket).
   - Anti-cheat / emergent tests matching `catalog.md` (`1.2`, `1.3`, `1.4`): resolution-invariant piece count, unused-item invariance, zero `cut.rs`-style branches, `real_time.rs` `<= 15 µs` median.

## Deliverables (and stop cleanly)

1. Write `C:\Users\PASCAL\Desktop\path tracing\jobs\backward-sets-spec\report.md` summarizing every frozen interface, invariant, data flow diagram, and rejected shortcut (plus any blocker found in existing `families.rs` / `family_wave.rs` / `faces.rs`).
2. Write `C:\Users\PASCAL\Desktop\path tracing\jobs\flight-ladder-sets\prompt.md` opening with `# Backward state-set wave: steps, fences, stairs, and parkour ladder` and `**Goal:** ...` in closed GPT-ready form (complete decisions, zero ambiguities, exact files & test commands). Do NOT touch `shown.txt` or launch Astra yet.
3. End with `STATUS: DONE` or `STATUS: BLOCKED: <reason>`.
