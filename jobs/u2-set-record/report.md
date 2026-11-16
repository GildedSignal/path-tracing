# U2 Draft: The Set Record (One-Way Door), Reach Between Faces, and Lenses for Kill Tests 10 & 11

**Consumer:** Opus red-team of the set record (`docs/HANDOFF.md`, U2) before it freezes, then Terra's U2 build. Gates: Kill tests 10 and 11 (`docs/learning.md` §6) and the stalled joint on `fences`.

---

## 1. The Set Record (One-Way Door)

### 1.1 Conceptual Role
Under `docs/learning.md` §0–2 and `docs/representation.md` §10, standable cells are no longer nodes (`S91`). Nodes are **joints** (events of the law at a face: lips, landings, class boundaries, wormhole mouths, edit cells). A **set** is what a joint holds: the arriving states from which a known sentence reaches the target, represented strictly through the law's own comparisons — never an arbitrary grid, tolerance, or cell-centre assumption.

Crucially, to enable cross-world pattern reuse (`docs/learning.md:66`, `jobs/unified-plan/learning-review.md:70–76`) without losing goal-specific costs, the design cleanly separates:
1. **The Goal-Free Joint Pattern Record (`JointPatternSet`)**: local transition geometry, plateaus of the free offset coordinate, local relative read footprint, exit state classes, and per-plateau witnesses — reusable wherever the same `(LabelId, joint_pattern_content, entry_class)` recurs.
2. **The Goal-Bound Joint Instance (`JointNode`)**: anchored at a concrete `(face, goal, tick_epoch)`, binding `T_to_go`, `prices`, concrete `ReadCertificates` premises, `via` outgoing joint, and `slide_want` (computed in `(min, +)` over the folded graph).

### 1.2 Key: `LabelId` and Joint Pattern Content
- **Outer / Kit-Medium Key — `LabelId` (`D:\path-tracing-wt\3.2d-3\engine\law\src\codec.rs:128–133, 176–178, 233–254`):**
  - `LabelId(pub(crate) u32)` interns `Label { law_schema, body_kit, support: &[ClassId], medium: &[ClassId] }` by exact canonical byte equality (`Interner::intern_label`, hash bucket only).
  - Changing the kit swaps the `ProjectionTable` (`codec.rs:141–170`), producing a different `LabelId` only where the kit's verb answers (`LegsRole`, `mining_progress`, `UnportedVerb`) actually differ.
- **Joint Pattern Content Key (`JointPatternKey`):**
  - Canonical `(LabelId, env_class: ClassId, shape: ShapeId, cell_classes: Box<[ClassId]>, entry_class: ClassId, face_geometry_class: ClassId)`:
    - `shape`: sorted `(relative_offset_from_joint: [i32; 3], query_mask: u8)` (`pattern.rs:83–87`).
    - `cell_classes`: `ClassId` per cell in `shape` from `codec::Interner` (byte-equal canonical answers, NOT unverified `u128` FNV digests; `codec.rs:16–20, 201–212`).
    - `env_class`: canonical `EnvironmentRead` answers (`MinY` relative to joint Y, `FastLava`, `Difficulty`, `NaturalRegeneration`; `codec.rs:59–67`).
    - `entry_class`: read-before-write discrete law comparisons of the arriving `PlayerState` (`codec_state::state_answers`, `codec_state.rs:7–86`), plus `(word_id, phase)` when arriving on a gait, or speed-band comparisons inside a site.
    - `face_geometry_class`: canonical face shape/height interval (`VoxelShape::grid_cells()` of the joint's support face, e.g. slab/stair/climbable band `low..high`).
  - **Identity frame only (`docs/footprints.md:121–142`):** turned/mirrored frames fail bit-exact replay (`1e-17` to `3e-5` due to Mojang `sin.bin` asymmetry), so the canonical key keeps the identity frame only; hashed turn symmetry may propose initial bisection brackets, never certify.

### 1.3 Fields of the Set Record

#### A. Goal-Free Pattern Record (`JointPatternSet`, keyed by `JointPatternKey`)
Stored in SoA / parallel flat slices for cache-dense branchless SIMD lookup (`docs/scales.md:127`, `jobs/unified-plan/plan.md:133`):
- **`law_hash: [u8; 32]` (or pinned `u64` + schema):** digest of `engine/law/src/` + `models.tsv` (`docs/learning.md:69–70`). Invalidates stale lower bounds and plateaus on any law delta.
- **`domain: (f64, f64)`:** exact span `[s_min, s_max]` of the free offset coordinate `s` on the joint's face (including body overhang `±0.3` where unblocked).
- **`intervals` (partitioning `domain` completely, sorted by `lo`):**
  - `lo: Box<[f64]>`, `hi: Box<[f64]>` — single-ULP breakpoints (`hi[i].next_up() == lo[i+1]` when contiguous, total IEEE order).
  - `status: Box<[IntervalStatus]>` — four-way status (`learning-review.md:74`):
    - `Found`: certified plateau; has `delta_t`, `word`, `args`, `exit_class`, `exit_offset_map`, `width`, `witness_id`.
    - `Forbidden`: proven unreachable / outside derived law envelope (`ReachLaw` / `arc-certificate` / `gait-modulo`).
    - `Searched { work_units: u32 }`: `k` work units spent without finding a valid word (NOT a proof of impossibility).
    - `Refused { refusal_id: u16, witness_id: u32 }`: law hit an unported mechanic (`law::Refusal`, demand on the port).
  - `delta_t: Box<[u32]>` — ticks for the outgoing word on this plateau (`u32::MAX` if not `Found`).
  - `word: Box<[WordId]>` + `args: Box<[Arguments]>` (including `count` and `yaw_index`/`refined_yaw`, `vocabulary.md:57`).
  - `exit_class: Box<[ClassId]>` — discrete exit state class (`state_answers`), plus `exit_delta_pos: [i32; 3]` (relative target face offset).
  - `width: Box<[Width]>` — `early: bool`, `late: bool` (`segment.rs:120`), plus `clearance` once defined (`vocabulary.md:71`).
  - `read_footprint_id: Box<[u32]>` — relative cell offsets + query masks read across the plateau (union over plateau endpoints + interior sample).
  - `witness: Box<[Witness]>` — compact replayable witness (`entry_state_template_id`, `s: f64`, `word`, `args`, `expected_end_digest`) kept out-of-line from the hot `lo/hi/delta_t/word` scan arrays so hot acting lookups stay in L3 (`scales.md:126`).

#### B. Goal-Bound Instance Record (`JointNode`, per `(JointAddr, GoalId, entry_class)`)
- **`pattern_key: JointPatternKey`** (points into shared `JointPatternSet`).
- **`t_to_go: Box<[u32]>`** (parallel to `intervals`; `delta_t + downstream_t_to_go`, `u32::MAX` if no route).
- **`next_joint: Box<[Option<JointId>]>`** (parallel to `intervals`).
- **`alternate: Option<Alternate>`** (`learning-review.md:116`: second-best `(word, next_joint, t_to_go)` at this joint for O(1) fallback on certificate break).
- **`premises` (cells, ticks, kit):**
  - `cert_id: usize` in `ReadCertificates` (`codec_proof.rs:95–183`) binding:
    - **Cells:** `(BlockPos, Query mask, ClassId)` for every cell read by the joint's standing, run-up, tube, and landing (`reach.rs:142–145`, `codec_proof.rs:89`), plus `EnvironmentRead` and `ExtraCollisions` probes.
    - **Ticks:** validity window `(from_tick: u64, until_tick: Option<u64>)` (`reach.rs:147`), plus per-read ticks for scheduled/4D worldtubes (`codec_proof.rs:89`).
    - **Kit:** `kit_id` / `ProjectionTable` binding (`codec.rs:144`).
- **`prices: Box<[(ResourceKey, i32)]>`:** marginal `T` saved downstream by holding each kit item / consumable / state resource at this joint (`S96`, `docs/learning.md:56`, `plan.md:40`). E.g. holding a block / bucket / pearl / logs is worth `T_without - T_with` at the downstream joint that needs it.
- **`slide_want: bool`:** true iff a neighbouring plateau in this set has `t_to_go` lower by `>= 1` tick AND the upstream set can reach that plateau (`docs/learning.md:35`, `plan.md:82`).

### 1.4 Lifecycle: Build, Store, Invalidate, Read

1. **Build (Bisection over takeoff states, in Learning only):**
   - Opened when `solve` hits a joint with no set or an `open` gap, queued in the single learning allocator (`learning-review.md:30–34, 54–58`).
   - Step A (*Envelope check*): derive exact envelope (`ReachLaw` / `arc-certificate`). If target face is provably unreachable from `domain`, mark `Forbidden` with zero replays.
   - Step B (*Coarse scan + analytic cuts*): evaluate the candidate word (or exact search for a new pattern, `learning.md:66`) across `domain` at **analytic geometric breakpoints** (face boundaries, `VoxelShape` step/wall planes shifted by body radius `±0.3`, binade boundaries) plus a coarse scan. *(Note: `trial-offset-monotone/check.md:26` proved that coarse grids alone can miss `1.2e-10`–`7.2e-5` slivers and `T` reverses `12→14→12`, so non-monotone interior bracketing is mandatory.)*
   - Step C (*Float bisection*): for every adjacent pair `(s_a, s_b)` whose `(exit_kind, exit_block, delta_t, next_word)` differs, binary-search IEEE `f64` bit patterns (`≤ 53–64` halvings, `plan.md:40`) until `s_b == s_a.next_up()`.
   - Step D (*Certify endpoints*): replay both single-ULP endpoints `s_a` and `s_b` once through `law::try_tick` under `RecordingWorld` to seal `(delta_t, exit_class, footprint, witness)`.
   - If the pattern belongs to an existing joint family, seed Step B from the family neighbour's breakpoints (`learning-review.md:64`).

2. **Store:**
   - Pattern records live in a global resident store keyed by `JointPatternKey`, interned via `codec::Interner` (canonical bytes).
   - Hot arrays (`lo`, `hi`, `t_to_go`, `word`) stay compact and contiguous for L3 residency; cold `witness` and `footprint` details live out-of-line.
   - Goal-bound `JointNode`s live in the goal's shared `Field` (one field per goal shared by all bots, `scales.md:103`).

3. **Invalidated by Footprint:**
   - All cell/environment reads of `JointNode` are registered in `ReadCertificates` (`codec_proof.rs:99–100`), reverse-indexed by `(BlockPos, tick)`.
   - On a block edit at `(pos, tick_range)` or world delta, `ReadCertificates::edit` (`codec_proof.rs:150–166`) re-queries `source.cell(tick, pos, mask)` ONLY for affected certificates.
   - If the canonical bytes map to the same `ClassId`, the set stays valid (e.g., flower ↔ air under legs, `codec.rs:271–280`).
   - If `ClassId` changes, ONLY that `JointNode` is marked invalid (`Refusal::Changed`) and re-queued; upstream `JointNode`s propagate increased `T_to_go` in `(min, +)` (`representation.md:193`), or fall back in O(1) to `alternate` if valid.
   - On a law change (`law_hash` mismatch): sets are re-proved lazily by footprint (`learning.md:69`), and `Forbidden` bounds are re-derived (`learning.md:70`).

4. **Read by `solve` (`docs/learning.md:16–28`):**
   - `solve` reads `JointNode` without replaying worldtubes (`vocabulary.md:20`).
   - Validates `ReadCertificates::validate(&proof, cert_id, &binding)` in O(1) (`codec_proof.rs:141–147`).
   - Locates the plateau containing arriving offset `s` via branchless 16-lane SIMD compare + popcount (`scales.md:127`).
   - Checks `status`:
     - `Found`: reads `t_to_go`, `word`, `args`, `prices`, and `slide_want`. If `slide_want`, posts a bracket-narrowing request to the allocator; never blocks the first word if its bracket is closed (`learning.md:33`).
     - `Searched` / missing: emits a learning demand item for this `JointPatternKey` (`learning-review.md:56`).
     - `Forbidden` / `Refused`: pruned by proof (if `Forbidden`) or logs port demand (`Refused`), switching to `alternate` or next route.

---

## 2. The Reach Relation Between Faces (No Search Over Inputs)

### 2.1 Why Existing `reach.rs` and `hard_track.rs` Must Be Unified and Upgraded
- **`engine/plan/src/reach.rs` (`Reach` & `Field`):**
  - Compiles only from rest on ONE floor block (`reach.rs:51–55`), rotates trajectories with `f64` `sin/cos` (`reach.rs:42–45`, which `docs/footprints.md:139–141` proves is NOT bit-exact under Mojang's `sin.bin`), samples takeoffs only at cell centres `[x+0.5, y+top, z+0.5]` (`reach.rs:190`), uses `Pose::Standing` `free()` (`reach.rs:184`), and drops momentum/entry-speed classes (`t: u32` per cell).
  - Yet `backward-tools` (`lab/results/local-20261113-1845-1-1-backward-tools/dossier.md:15–16`) showed the mirrored-table core works: 84.5% (4,722/5,587) of standable goals certified in 1 word at `~215–225 µs/goal`, and **89% of its 17,357 failures were `runup-mismatch`** (from-rest stone-only run-up assumption), while the flight arc half succeeded ~97.6% of the time.
- **`D:\path-tracing-wt\hard-track-2\engine\plan\tests\hard_track.rs` (`ReachLaw`, `FreeSpace`, `SupportGraph`):**
  - Built as a **conservative over-approximation** to *exclude* illegal shortcuts (proving decoy pockets have no exit edges), NOT to emit executable sentences.
  - Asserts `hardtrack`-specific preconditions (`hard_track.rs:353–356`: integral Y planes, no fluids/inside hooks, only 3 friction values).

### 2.2 Derivation from Mirrored Reach Tables Without Searching Inputs
1. **Precomputed Law-Indexed Reach Tables (Generator Store, offline/once per `(kit, ground_class, entry_speed_class)`):**
   - For each canonical `(ground_class, ceiling_band, entry_speed_class)` and each word in the toolkit (`sprint-jump`, `sprint`, `walk`, `jump`, `sneak`, `climb`, etc., at the law's actual `sin.bin` yaws — NO `f64` rotation `turn()`), run the law once to record relative landing envelopes `Arc`:
     - `path: Vec<[f64; 3]>` (exact tick-by-tick displacements from takeoff contact),
     - `runup` footprint + entry speed class required,
     - `dh_band: (lo, hi]` per tick, `apex`, `grounded: bool`, and `exit_speed_class`.
   - Deduplicate by Pareto front on `(ticks, dh_band, runup_len, apex)` per `(dx_cell, dz_cell, entry_class, exit_class)` (`reach.rs:108–116`).
2. **Face Extraction (Replacing Standable Cell Centres):**
   - A **face** is a support top (`!above.is_block()` or partial `VoxelShape` top `lo..hi`), slime top, or climbable cell/top (`Support::low()..high()` in `hard_track.rs:141–153`), retaining the full `[x, x+1] × [z, z+1]` face rectangle + body overhang `±0.3` (exact `Aabb::player` horizontal radius, `hard_track.rs:264, 274–292`).
   - Adjacent coplanar faces with identical `ClassId` and ceiling clearance may form uniform zones (`extend` with `count`); joints sit only at face boundaries / lips / class changes (`docs/representation.md:178`).
3. **Mirrored Face Lookup (Backward Wave on Folded Graph):**
   - Starting from goal `B`'s faces, pop face `F_to` (with `(entry_class, T_to_go)`) from the min-plus priority queue (`T` only grows, so no node is revisited, `learning.md:39`).
   - Mirror each entry of the relevant `(exit_class -> F_to)` reach table `(dx, dz, lo..=hi)` about `F_to`'s face rectangle (plus overhang `±0.3` — NOT just `[x+0.5, z+0.5]`).
   - Intersect mirrored takeoff offsets with faces `F_from` in the spatial hash/index.
4. **One Worldtube Check Per Arc (`vocabulary.md:20`, `scales.md:69`):**
   - For each candidate `(F_from -> F_to, arc)`:
     1. Fast rejection via `FreeSpace` / `SectionRoles` (`codec_planes.rs:55–71` SIMD bitplane query, 64 cells/op) + altitude-slab connectivity (`hard_track.rs:382–383`) + run-up face `ClassId` match.
     2. If clear in role planes, run **one** `RecordingWorld` replay / `arith::free_tick` tube check from the mirrored takeoff sample on `F_from` to certify the edge, recording its `ReadCertificates` footprint and `JointPatternSet` entry.
   - Zero search over `64 yaws × 54 programs` during face-graph construction!
5. **What of `hard-track-2`'s Machinery Is Reused vs. Replaced:**

| Component in `hard_track.rs` | Disposition in U2 | Role & Adaptation |
|---|---|---|
| `ReachLaw::derive()` (`:157–229`) | **REUSE & GENERALIZE** | Extract law-derived `apex`, `rotation` (`sin.bin` sliding-window enclosure), `speed` per kit into the kit's light-cone / `Forbidden` envelope prover (replacing bare-legs `Goal::C`, `goal.rs:7`). Drop hardcoded 3-block list (`:192`). |
| `ReachLaw::flight()` & `rebound()` (`:230–246`) | **REUSE** | Used to prove `Forbidden` intervals in `JointPatternSet` (zero replays needed when `q.low() > peak` or `gap > reach`). |
| `Support { cell, climb, slime }` (`:141–153`) | **REPLACE** | Generalize to `Face` derived from `ProjectionTable` (`LegsRole` + `VoxelShape` top planes + fluid/climbable/slime hooks), removing `hardtrack`-specific `element()` bitmask (`:145–152`) and integral-Y assertion (`:353`). |
| `FreeSpace::attachments` & `labels(lo, hi)` (`:254–334`) | **REUSE AS FILTER** | Keep the exact rectangle-overhang (`±0.3`) + `Pose::Swimming` clearance + altitude-slab 6-connected component check as a **necessary** (`Forbidden`) geometric sieve before tube checks; back it by `SectionRoles` bitplanes (`codec_planes.rs`) instead of `Vec<bool>`. |
| `SupportGraph::build` edge rule (`:371–386`) | **REPLACE** | Replace single global-speed over-approximation (`dx.hypot(dz) <= reach`) with mirrored `(ground_class, entry_speed_class)` reach tables + one `RecordingWorld` worldtube check per arc (`vocabulary.md:20`). |

---

## 3. One-Way-Door Choices & Alternatives (Questions for Opus Red-Team)

1. **Plateau Domain Geometry (1D offset `s` vs. 2D face `[u, v]`):**
   - *Choice:* `plan.md:40` & `representation.md:179` define plateaus over *one* free coordinate `s` along the held word's line.
   - *Risk:* At turning joints (e.g. `hardtrack` decoy ice lane east `->` alignment wall south `->` final ladder north), arriving `(x, z)` is 2D on the face; projecting 2D arrivals onto a 1D line `s` for the next word loses the orthogonal coordinate `w` unless `w` is quantized/classified or the plateau is 2D.
   - *Alternatives:* (A) Keep 1D `s` per `(entry_class, outgoing_yaw)` and treat orthogonal coordinate `w` via analytic face-band intervals (`[w_lo, w_hi]` inside which `s` bisection holds); (B) Store 2D axis-aligned plateau rectangles `[u_lo, u_hi] × [v_lo, v_hi]`; (C) Keep 1D `s` along the incoming line and sample/bisect outgoing yaw at the joint. **Question:** Which representation keeps sets finite and composes cleanly across 90°/oblique turns without exploding plateau count?

2. **Bisection Blindness to Sub-Grid Slivers & Non-Monotonicity (`trial-offset-monotone/check.md:26, 40–50`):**
   - *Fact:* GLM's independent check on 26.2 found (a) `T` reverses `12→14→12` over a 1-block gap, and (b) a sub-grid sliver (`1.2e-10` to `7.2e-5` wide) existed near the window edge that a `1/2048` coarse grid missed.
   - *Risk:* Pure coarse-grid + bisection can silently miss narrow `Found` slivers (causing sub-optimality) OR falsely mark an interval `Forbidden` if a narrow landing island lies inside a coarse gap!
   - *Alternatives:* (A) Compute **analytic candidate breakpoints** from `VoxelShape` cell edges ± body overhang `0.3` and gait step `Δx` before bisecting (no chosen grid constant); (B) Separate **completeness**: `Found` plateaus certify their own interval `[lo, hi]` by endpoint replay, whereas `Forbidden` can ONLY be assigned by an independent envelope proof (`ReachLaw` / `FreeSpace`), never by "coarse grid missed it" (missed slivers become `Searched` gaps, never `Forbidden`). **Question:** Should analytic shape-edge pre-cuts + `Forbidden`-by-envelope-only be mandatory in the U2 spec?

3. **Goal-Free Pattern Store vs. Goal-Bound `JointNode` Split:**
   - *Choice:* Split `JointPatternSet` (goal-free, keyed by `(LabelId, JointPattern, entry_class)`, holding local `delta_t`, `word`, `exit_class`, `width`, `witness`) from `JointNode` (goal-bound, holding `t_to_go`, `next_joint`, `prices`, `ReadCertificates` premises, `slide_want`).
   - *Alternative:* Store `t_to_go` and `prices` directly inside the set record. (Rejecting this seems clear — mixing goal `B` into the pattern record kills pattern transfer across goals — but confirm if any price or premise belongs in the pattern record as relative offsets.)

4. **Shared `u16` `ClassId` Capacity (`jobs/3.2d-3-B-judge/last.md:27`):**
   - *Fact:* `codec.rs:173` uses `ClassId(u16)` (max 65,535) for BOTH cell/role answers AND `state_answers` (`codec_state.rs`). On natural terrain, segment/joint patterns scale to `10⁵–10⁶` (`representation.md:86`).
   - *Alternatives:* (A) Keep `ClassId(u16)` strictly for per-kit cell/role answers and add `StateClassId(u32)` + `JointPatternId(u32)` (matching `LabelId(u32)` at `codec.rs:177`); (B) Widen `ClassId` to `u32` everywhere (doubles `SectionRoles` / `Label` ID memory). **Question:** Split `ClassId(u16)` vs `StateClassId(u32)` before U2 freezes?

5. **Four-Way `Status` Granularity (Per-Interval vs. Per-Set):**
   - *Choice:* Store `IntervalStatus` (`Found | Forbidden | Searched(u32) | Refused(u16, u32)`) on every interval covering `[s_min, s_max]`, rather than a single status on the whole `JointPatternSet`.
   - *Alternative:* Single status per `JointPatternSet` + list of `Found` plateaus only (leaves unsearched gaps indistinguishable from `Forbidden` gaps).

6. **`Premises` Storage: `Canonical` Bytes vs. `ClassId` + `ReadCertificates`:**
   - *Fact:* `wt/3.2d-3`'s `reach.rs:145` stores `Vec<(BlockPos, Option<Canonical>)>` (boxed byte slices per cell, flagged as a performance regression in `jobs/3.2d-3-B-judge/last.md:26`).
   - *Choice:* Store `ReadCertificates` proof handle (`Proof<ReadsHold>`) + interned `ClassId`s, eliminating per-set boxed bytes and giving O(1) reverse-indexed edit invalidation (`codec_proof.rs:150–166`).

7. **Automated `law_hash` vs. Manual `version: u64`:**
   - *Fact:* `codec_proof.rs:52` has manual `version: u64`; `engine/law` has no auto-derived hash. A silent unbumped law delta breaks `learning.md:70` ("lower bounds are re-derived, never trusted across a law change").
   - *Alternatives:* (A) `build.rs` SHA-256/FNV over `engine/law/src/**/*.rs` + `models.tsv` exposed as `law::LAW_HASH`; (B) Keep manual `version` + CI diff check.

8. **Witness Footprint & L3 Budget (`scales.md:126`):**
   - *Fact:* Full `PlayerState` bits are ~90 `u64` words (~720 B). Storing one per plateau exceeds hot L3 for large maps.
   - *Alternatives:* (A) Keep hot SoA arrays (`lo, hi, t_to_go, word, status`: ~24 B/plateau) separate from cold `Witness` arena (storing `(entry_template_id, s: f64, word, args, end_digest)`: ~32 B); (B) Re-derive witness on demand from `(entry_class, s, word)` (risks non-unique entry state if `entry_class` is coarser than exact bits).

9. **Turned Frames for Bisection Seeding (`footprints.md:121–142`):**
   - *Alternatives:* (A) Strictly identity frame in `JointPatternKey` (4–8× more patterns learned independently); (B) Identity frame in `JointPatternKey` for certification, with hashed turned-model lookup (`pattern::model_digest`) allowed ONLY to seed Step B's initial bisection brackets (learning-review §2 style), replayed in the identity frame before filing.

10. **Granularity of `JointNode` on Smooth / Sloped / Stair Faces:**
    - On staircases (`stonestaircase`) or fences (`fences`'s stalled joint), does every step block get a separate `JointPatternKey`, or does a multi-cell face pattern key capture the staircase step-up as one joint/gait?

---

## 4. Fermi Floors (With Stated Inputs)

**Inputs (all observed in read docs/logs):**
- Law tick: `0.6–0.95 µs` (`docs/engine.md:11`, `jobs/unified-plan/plan.md:141`).
- 7800X3D L3: `96 MB`, hit `~10 ns`; DRAM miss `~80 ns` (`docs/scales.md:126`, `plan.md:132, 141`).
- Plateau count per 1D edge: `~5` on stone→ice (`trial-offset-monotone/check.md:22`, `plan.md:40`).
- Hot plateau tuple (SoA): `lo (8 B) + hi (8 B) + t_to_go (4 B) + word_id (2 B) + status/flags (2 B) = 24 bytes/plateau`; 5 plateaus = `120 bytes` (< 2 cache lines).
- Cold witness entry: `template_id (4 B) + s (8 B) + word_id (2 B) + args (8 B) + end_digest (8 B) ≈ 32 bytes`.
- Bisection per breakpoint: `≤ 64` halvings × `2` endpoint replays × `~12` ticks/jump ≈ `~1,500` law ticks (`learning-review.md:64`); with `~4` breakpoints ≈ `6,000` ticks × `0.8 µs` ≈ **4.8 ms** (1 core); cold search from scratch `~10³ × 12` ticks ≈ **12 ms** (`representation.md:23`, `plan.md:141`).
- `hardtrack` box: `51 × 41 × 51` (`hard_track.rs:345`), `3,013` old supports (`old-graph.md:3`), `~10²` logical face clusters (`plan.md:147`), `~10–30` reach neighbours/face.

| Operation | Inputs & Derivation | Floor | Current / Reference |
|---|---|---|---|
| **Hot plateau lookup in `solve`** | 1 `ReadCertificates::validate` (`~10 ns` L3 hit) + 16-lane SIMD compare + popcount over `≤16` `f64`s (`~10 ns`) | **20–50 ns / joint** | `~225 µs/goal` (`backward-tools`) / `~12.5 µs/tick` (`search.rs`) |
| **Slide signal check** | Compare `t_to_go` of adjacent plateaus in L3 (`≤ 8` comparisons, 0 replays) | **< 10 ns / joint** | `~0.5–20 s` full-sentence replay loop (`polish.rs`) |
| **Cold `JointPatternSet` build (bisection)** | ~4 breakpoints × ≤64 halvings × 2 × 12 ticks × 0.8 µs (seeded) or `10³ × 12 × 0.8 µs` (cold search) | **~5 ms seeded / ~12 ms cold** (on idle cores, 0 T cost while moving) | not built |
| **Face-reach graph on `hardtrack`** | `~10²–3×10³` faces × `~20` mirrored reach table lookups (`~50 ns` w/ `SectionRoles` SIMD) + `~20` worldtube arc checks (`~10 µs` each) | **0.5–1 ms warm** | `no sentence` (`hard-track/report.md:50`) |
| **Footprint edit invalidation** | 1 `ReadCertificates::edit` BTree lookup (`~100 ns`) + `source.cell` + `Interner::find` (`~200 ns`) per affected set | **~0.3–1 µs / affected set** | whole-field rebuild (`reach.rs`) |
| **Memory per 10⁴ varied-terrain joints** | `10⁴` joints × (`120 B` hot plateaus + `64 B` node header) ≈ `1.8 MB` hot; `10⁵` patterns × `160 B` witness/pattern ≈ `16 MB` | **~2 MB hot / ~18 MB total** (fits in 96 MB L3) | — |

---

## 5. Premises Checked (`EXISTS`, `DIFFERS`, or `MISSING`)

| Premise | Status | Location & Evidence |
|---|---|---|
| `LabelId` and `Interner::label` (`law_schema, body_kit, support, medium`) | **EXISTS (on `wt/3.2d-3` only)**; **MISSING on `main`** | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec.rs:128–133, 176–178, 233–254`; not found in `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\` |
| `ClassId`, `Canonical`, `ProjectionTable` (per-kit projection) | **EXISTS (on `wt/3.2d-3`)**; **DIFFERS** (`ClassId` is `u16`, shared with state classes, risks `TableFull` at 65,535) | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec.rs:16–20, 141–174`; `jobs/3.2d-3-B-judge/last.md:27` |
| `SectionRoles` SIMD presence planes | **EXISTS (on `wt/3.2d-3`)**; **MISSING on `main`** | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec_planes.rs:33–93` |
| `ReadCertificates` (4D `(tick, pos, mask, answer)`, reverse-indexed `edit` invalidation) | **EXISTS (on `wt/3.2d-3`)**; **MISSING on `main`** | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec_proof.rs:88–183` |
| `StateReads` & `state_answers` (comparison-only state class) | **EXISTS** | `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\state_reads.rs:29–106`; `D:\path-tracing-wt\3.2d-3\engine\law\src\codec_state.rs:7–86` |
| `Set` record (`plateaus`, `prices`, 4-way `status`, `witness`) | **MISSING**; `reach.rs::Cell` **DIFFERS** (cell-centre, scalar `t: u32`, no plateaus/prices/status/witness) | `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\reach.rs:130–148`; `D:\path-tracing-wt\3.2d-3\engine\plan\src\reach.rs:145` (uses `Option<Canonical>` instead of `ClassId`) |
| `law_hash` (auto-derived hash of law + models) | **MISSING** (`codec::SCHEMA` is a static string; `Binding::version` is manual `u64`) | `D:\path-tracing-wt\3.2d-3\engine\law\src\codec.rs:12`; `codec_proof.rs:52`; not found in `engine/law/Cargo.toml` |
| `Width { early, late }` and `clearance` | **EXISTS** (`early`, `late`); **MISSING** (`clearance`) | `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\segment.rs:117–120` |
| Mirrored reach tables (`Reach`, `Field`) | **DIFFERS** (1 floor from rest, `f64` `sin/cos` `turn()`, cell-centre takeoffs, `Pose::Standing` `free()`) | `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\reach.rs:32–127, 179–277`; `docs/footprints.md:139–141` |
| `hard-track-2` reach certificate (`ReachLaw`, `FreeSpace`, `SupportGraph`) | **EXISTS (on `wt/hard-track-2` test only)**; **DIFFERS** (conservative non-executable bounds; hardcoded 3 frictions & integral-Y assert) | `D:\path-tracing-wt\hard-track-2\engine\plan\tests\hard_track.rs:155–387` |
| `hardtrack` shortcut geometry on `main` | **EXISTS** (`lab/tracks/hardtrack.build.txt` on `main` retains the old alignment wall `-85 -37 -99 -85 -32 -96` where Julien's 428 shortcut works) | `C:\Users\PASCAL\Desktop\path tracing\lab\tracks\hardtrack.build.txt:43–45` vs `jobs/hard-track/shortcut.md:8–9` |
| Double-tap `sprintWindow` in `engine/law` (needed to replay `julien-428.jsonl`) | **MISSING** in `engine/law` (jar bytecode pinned by `port-sprint-window`; replay refuses at `sprintWindow: 7`) | `jobs/hard-track/julien-428.jsonl:1`; `jobs/hard-track/shortcut.md:3`; `docs/HANDOFF.md:46`; `lab/wiki/index.md:105` |
| Kit-aware lower bound (`Goal::lower_bound`) | **DIFFERS** (uses terrain-only `C = 19.902886/20.0`, bare legs only) | `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\goal.rs:7, 29–36`; `docs/engine.md:106` |
| `solve` loop & local `slide_want` | **MISSING** (`search.rs` is exact-state A*; `polish.rs` is full-replay `slide`) | `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\search.rs:115–513`; `polish.rs:27–64` |
| Plateaus finite & clean ULP boundaries | **EXISTS (2 patterns only)**; **DIFFERS** on monotonicity (`12→14→12` on 1-block gap; `1.2e-10`–`7.2e-5` sub-grid sliver missed by 1/2048 grid) | `lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md:22–50` |

---

## 6. GLM Lens Drafts (Placed in `jobs/u2-set-record/lenses/`)

Four lenses are drafted in `lab/lenses/` format:
1. **`trial-plateau-refined.md`** — **Kill test 11** (`docs/learning.md:113`): plateau counts per joint pattern under refined bisection on natural-terrain joint patterns, jar-backed against Mojang 26.2 `LocalPlayer` (specifically checking whether sub-grid slivers emerge under finer initial grids `1/64 … 1/16384` and analytic face-edge cuts).
2. **`hardtrack-face-reach.md`** — **Kill test 10 independent check** (`docs/learning.md:112`): builds a face-level reach relation over `lab/tracks/hardtrack.build.txt` from mirrored reach tables + `FreeSpace` overhang connectivity with NO knowledge of `julien-428.jsonl`, and checks whether it contains his wall-top shortcut (`[-84,-29,-102] -> [-85,-32,-96] -> [-84,-30,-92] -> B`) and what certified `T` it reaches.
3. **`trial-plateau-entry-speed.md`** — **Set-record premise check**: measures plateau count and boundary shifts when entry velocity / cycle phase varies (inside momentum's memory, e.g. on ice/stone transitions and head-hitter exits), jar-backed.
4. **`reach-face-overhang.md`** — **Reach-relation & `fences` premise check**: upgrades `backward-tools`'s cell-centre stone-only mirror to multi-ground + face-edge overhang (`±0.3`) + fences/climbables on the 18 track fixtures (specifically testing whether `fences`'s stalled joint closes without input search).
