Written to [report.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/u2-set-record/report.md) and drafted four GLM lenses in `jobs/u2-set-record/lenses/`:
- [trial-plateau-refined.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/u2-set-record/lenses/trial-plateau-refined.md) (Kill test 11)
- [hardtrack-face-reach.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/u2-set-record/lenses/hardtrack-face-reach.md) (Kill test 10 independent check)
- [trial-plateau-entry-speed.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/u2-set-record/lenses/trial-plateau-entry-speed.md) (Set record entry-speed/phase premise)
- [reach-face-overhang.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/u2-set-record/lenses/reach-face-overhang.md) (Face-overhang reach & `fences` stalled joint premise)

Below is the full contents of [report.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/u2-set-record/report.md):

---

# U2 Draft: The Set Record (One-Way Door), Reach Between Faces, and Lenses for Kill Tests 10 & 11

**Consumer:** Opus red-team of the set record ([HANDOFF.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/HANDOFF.md), U2) before it freezes, then Terra's U2 build. Gates: Kill tests 10 and 11 ([learning.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/learning.md) Â§6) and the stalled joint on `fences`.

---

## 1. The Set Record (One-Way Door)

### 1.1 Conceptual Role
Under [learning.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/learning.md) Â§0â€“2 and [representation.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/representation.md) Â§10, standable cells are no longer nodes (`S91`). Nodes are **joints** (events of the law at a face: lips, landings, class boundaries, wormhole mouths, edit cells). A **set** is what a joint holds: the arriving states from which a known sentence reaches the target, represented strictly through the law's own comparisons â€” never an arbitrary grid, tolerance, or cell-centre assumption.

Crucially, to enable cross-world pattern reuse ([learning.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/learning.md#L66), [learning-review.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/learning-review.md#L70-L76)) without losing goal-specific costs, the design cleanly separates:
1. **The Goal-Free Joint Pattern Record (`JointPatternSet`)**: local transition geometry, plateaus of the free offset coordinate, local relative read footprint, exit state classes, and per-plateau witnesses â€” reusable wherever the same `(LabelId, joint_pattern_content, entry_class)` recurs.
2. **The Goal-Bound Joint Instance (`JointNode`)**: anchored at a concrete `(face, goal, tick_epoch)`, binding `T_to_go`, `prices`, concrete `ReadCertificates` premises, `via` outgoing joint, and `slide_want` (computed in `(min, +)` over the folded graph).

### 1.2 Key: `LabelId` and Joint Pattern Content
- **Outer / Kit-Medium Key â€” `LabelId` ([codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L128-L133), [L176-L178](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L176-L178), [L233-L254](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L233-L254)):**
  - `LabelId(pub(crate) u32)` interns `Label { law_schema, body_kit, support: &[ClassId], medium: &[ClassId] }` by exact canonical byte equality (`Interner::intern_label`, hash bucket only).
  - Changing the kit swaps the `ProjectionTable` ([codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L141-L170)), producing a different `LabelId` only where the kit's verb answers (`LegsRole`, `mining_progress`, `UnportedVerb`) actually differ.
- **Joint Pattern Content Key (`JointPatternKey`):**
  - Canonical `(LabelId, env_class: ClassId, shape: ShapeId, cell_classes: Box<[ClassId]>, entry_class: ClassId, face_geometry_class: ClassId)`:
    - `shape`: sorted `(relative_offset_from_joint: [i32; 3], query_mask: u8)` ([pattern.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/law/src/pattern.rs#L83-L87)).
    - `cell_classes`: `ClassId` per cell in `shape` from `codec::Interner` (byte-equal canonical answers, NOT unverified `u128` FNV digests; [codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L16-L20), [L201-L212](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L201-L212)).
    - `env_class`: canonical `EnvironmentRead` answers (`MinY` relative to joint Y, `FastLava`, `Difficulty`, `NaturalRegeneration`; [codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L59-L67)).
    - `entry_class`: read-before-write discrete law comparisons of the arriving `PlayerState` (`codec_state::state_answers`, [codec_state.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_state.rs#L7-L86)), plus `(word_id, phase)` when arriving on a gait, or speed-band comparisons inside a site.
    - `face_geometry_class`: canonical face shape/height interval (`VoxelShape::grid_cells()` of the joint's support face, e.g. slab/stair/climbable band `low..high`).
  - **Identity frame only ([footprints.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/footprints.md#L121-L142)):** turned/mirrored frames fail bit-exact replay (`1e-17` to `3e-5` due to Mojang `sin.bin` asymmetry), so the canonical key keeps the identity frame only; hashed turn symmetry may propose initial bisection brackets, never certify.

### 1.3 Fields of the Set Record

#### A. Goal-Free Pattern Record (`JointPatternSet`, keyed by `JointPatternKey`)
Stored in SoA / parallel flat slices for cache-dense branchless SIMD lookup ([scales.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/scales.md#L127), [plan.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/plan.md#L133)):
- **`law_hash: [u8; 32]` (or pinned `u64` + schema):** digest of `engine/law/src/` + `models.tsv` ([learning.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/learning.md#L69-L70)). Invalidates stale lower bounds and plateaus on any law delta.
- **`domain: (f64, f64)`:** exact span `[s_min, s_max]` of the free offset coordinate `s` on the joint's face (including body overhang `Â±0.3` where unblocked).
- **`intervals` (partitioning `domain` completely, sorted by `lo`):**
  - `lo: Box<[f64]>`, `hi: Box<[f64]>` â€” single-ULP breakpoints (`hi[i].next_up() == lo[i+1]` when contiguous, total IEEE order).
  - `status: Box<[IntervalStatus]>` â€” four-way status ([learning-review.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/learning-review.md#L74)):
    - `Found`: certified plateau; has `delta_t`, `word`, `args`, `exit_class`, `exit_offset_map`, `width`, `witness_id`.
    - `Forbidden`: proven unreachable / outside derived law envelope (`ReachLaw` / `arc-certificate` / `gait-modulo`).
    - `Searched { work_units: u32 }`: `k` work units spent without finding a valid word (NOT a proof of impossibility).
    - `Refused { refusal_id: u16, witness_id: u32 }`: law hit an unported mechanic (`law::Refusal`, demand on the port).
  - `delta_t: Box<[u32]>` â€” ticks for the outgoing word on this plateau (`u32::MAX` if not `Found`).
  - `word: Box<[WordId]>` + `args: Box<[Arguments]>` (including `count` and `yaw_index`/`refined_yaw`, [vocabulary.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/vocabulary.md#L57)).
  - `exit_class: Box<[ClassId]>` â€” discrete exit state class (`state_answers`), plus `exit_delta_pos: [i32; 3]` (relative target face offset).
  - `width: Box<[Width]>` â€” `early: bool`, `late: bool` ([segment.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/words/src/segment.rs#L120)), plus `clearance` once defined ([vocabulary.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/vocabulary.md#L71)).
  - `read_footprint_id: Box<[u32]>` â€” relative cell offsets + query masks read across the plateau (union over plateau endpoints + interior sample).
  - `witness: Box<[Witness]>` â€” compact replayable witness (`entry_state_template_id`, `s: f64`, `word`, `args`, `expected_end_digest`) kept out-of-line from the hot `lo/hi/delta_t/word` scan arrays so hot acting lookups stay in L3 ([scales.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/scales.md#L126)).

#### B. Goal-Bound Instance Record (`JointNode`, per `(JointAddr, GoalId, entry_class)`)
- **`pattern_key: JointPatternKey`** (points into shared `JointPatternSet`).
- **`t_to_go: Box<[u32]>`** (parallel to `intervals`; `delta_t + downstream_t_to_go`, `u32::MAX` if no route).
- **`next_joint: Box<[Option<JointId>]>`** (parallel to `intervals`).
- **`alternate: Option<Alternate>`** ([learning-review.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/learning-review.md#L116): second-best `(word, next_joint, t_to_go)` at this joint for O(1) fallback on certificate break).
- **`premises` (cells, ticks, kit):**
  - `cert_id: usize` in `ReadCertificates` ([codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L95-L183)) binding:
    - **Cells:** `(BlockPos, Query mask, ClassId)` for every cell read by the joint's standing, run-up, tube, and landing ([reach.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/reach.rs#L142-L145), [codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L89)), plus `EnvironmentRead` and `ExtraCollisions` probes.
    - **Ticks:** validity window `(from_tick: u64, until_tick: Option<u64>)` ([reach.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/reach.rs#L147)), plus per-read ticks for scheduled/4D worldtubes ([codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L89)).
    - **Kit:** `kit_id` / `ProjectionTable` binding ([codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L144)).
- **`prices: Box<[(ResourceKey, i32)]>`:** marginal `T` saved downstream by holding each kit item / consumable / state resource at this joint (`S96`, [learning.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/learning.md#L56), [plan.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/plan.md#L40)).
- **`slide_want: bool`:** true iff a neighbouring plateau in this set has `t_to_go` lower by `>= 1` tick AND the upstream set can reach that plateau ([learning.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/learning.md#L35), [plan.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/plan.md#L82)).

### 1.4 Lifecycle: Build, Store, Invalidate, Read
1. **Build (bisection over takeoff states, in Learning only):** triggered when `solve` hits a missing/open joint, queued in the single learning allocator ([learning-review.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/learning-review.md#L30-L34)). Checks derived envelope (`Forbidden`); runs analytic geometric pre-cuts (face edges / `VoxelShape` planes Â± `0.3` overhang + binade boundaries) and coarse scan; bisects IEEE `f64` bits (`â‰¤ 64` halvings) at every transition in `(exit_kind, exit_block, delta_t, next_word)` without assuming monotonicity ([check.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md#L22-L50)); replays both single-ULP endpoints once through `law::try_tick` under `RecordingWorld`.
2. **Store:** pattern records live in a global resident store keyed by `JointPatternKey` via `codec::Interner`; hot SoA arrays stay in L3 while cold `witness` and footprint details live out-of-line; goal-bound `JointNode`s live in the goal's shared `Field`.
3. **Invalidated by Footprint:** registered in `ReadCertificates` reverse index (`BTreeMap<BlockPos, BTreeMap<u64, BTreeSet<(usize, usize)>>>`, [codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L99-L100)). On edit, `ReadCertificates::edit` ([codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L150-L166)) re-queries affected reads only; equal `ClassId` preserves the set; changed `ClassId` voids only that `JointNode` and updates upstream `(min, +)` or swaps to `alternate`. On `law_hash` change, re-proves lazily by footprint and re-derives `Forbidden` bounds.
4. **Read by `solve` ([learning.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/learning.md#L16-L28)):** validates `ReadCertificates::validate` in O(1) ([codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L141-L147)), locates plateau via branchless SIMD compare + popcount ([scales.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/scales.md#L127)), reads `(t_to_go, word, args, prices, slide_want)`, and emits `next_word` without replaying certified worldtubes.

---

## 2. The Reach Relation Between Faces (No Search Over Inputs)

- **Why existing `reach.rs` and `hard_track.rs` must be unified:**
  - [reach.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/reach.rs#L32-L127) builds from rest on 1 floor, uses non-bit-exact `f64` `sin/cos` `turn()`, restricts takeoffs to cell centres `[x+0.5, z+0.5]`, and stores scalar `t` per cell; yet `backward-tools` ([dossier.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261113-1845-1-1-backward-tools/dossier.md#L15-L16)) proved mirrored lookup certifies 84.5% of standable goals (`4,722/5,587`) in 1 word at `~215â€“225 Âµs/goal`, with 89% of failures being `runup-mismatch`.
  - [hard_track.rs](file:///D:/path-tracing-wt/hard-track-2/engine/plan/tests/hard_track.rs#L155-L387) (`ReachLaw`, `FreeSpace`, `SupportGraph`) is a conservative non-executable over-approximation tailored to `hardtrack` (`:353â€“356` integral-Y / 3-friction assertions).
- **Derivation without input search:**
  1. Precompute Pareto-front reach tables (`Arc` displacements, `dh_band`, `runup`, `exit_speed_class`) per `(ground_class, ceiling_band, entry_speed_class)` using actual `words::Arguments` yaws (`sin.bin`, no `f64` trig `turn()`).
  2. Extract all support/slime/climbable faces with `[x, x+1] x [z, z+1]` geometry + `Â±0.3` body overhang.
  3. In the backward wave from goal `B`, mirror reach table entries onto `F_to` to intersect candidate takeoff faces `F_from`, filter via `FreeSpace` / `SectionRoles` SIMD bitplanes ([codec_planes.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_planes.rs#L55-L71)) + altitude-slab connectivity, and certify each surviving edge with **one** `RecordingWorld` worldtube check ([vocabulary.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/vocabulary.md#L20)).
- **`hard-track-2` reuse vs. replacement:**
  - **Reuse & generalize:** `ReachLaw::derive()`, `flight()`, `rebound()` ([hard_track.rs](file:///D:/path-tracing-wt/hard-track-2/engine/plan/tests/hard_track.rs#L157-L246)) for `Forbidden` bounds and kit light cones; `FreeSpace::attachments` (`Â±0.3` overhang, `Pose::Swimming`) + `labels(lo, hi)` ([L254-L334](file:///D:/path-tracing-wt/hard-track-2/engine/plan/tests/hard_track.rs#L254-L334)) as a fast topological filter backed by `SectionRoles`.
  - **Replace:** `Support` (`:141â€“153`) with general `ProjectionTable` faces; `SupportGraph::build` single-speed edges (`:371â€“386`) with mirrored multi-ground/speed reach tables + one worldtube check per arc.

---

## 3. One-Way-Door Choices & Alternatives (Questions for Opus Red-Team)

1. **Plateau domain geometry:** 1D offset `s` along word line + analytic orthogonal band `[w_lo, w_hi]` vs. 2D face rectangles `[u_lo, u_hi] x [v_lo, v_hi]` vs. 1D `s` per `(entry_class, yaw)` â€” how do oblique/90Â° turns compose cleanly?
2. **Sub-grid slivers & non-monotonicity ([check.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md#L26), [L40-L50](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md#L40-L50)):** Should analytic `VoxelShape` Â± `0.3` pre-cuts + "`Forbidden` by envelope proof only (never by missed coarse sample)" be mandatory?
3. **Goal-free `JointPatternSet` vs. goal-bound `JointNode` split:** Confirm separating `delta_t`/`lo..hi`/`witness` (goal-free) from `t_to_go`/`prices`/`ReadCertificates` (goal-bound).
4. **Shared `u16` `ClassId` capacity ([last.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/3.2d-3-B-judge/last.md#L27)):** Split `ClassId(u16)` (cell/role answers) from `StateClassId(u32)` / `JointPatternId(u32)` (matching `LabelId(u32)`) before `TableFull` hits at 65,535?
5. **Four-way `Status` granularity:** Per-interval covering `[s_min, s_max]` completely vs. per-joint status + `Found` list.
6. **`Premises` storage:** `ReadCertificates` proof handle + `ClassId`s vs. `wt/3.2d-3`'s `reach.rs:145` `Vec<(BlockPos, Option<Canonical>)>` boxed bytes.
7. **Automated `law_hash` (`build.rs` over `engine/law/src/` + `models.tsv`) vs. manual `Binding::version: u64` ([codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L52)).**
8. **Hot SoA vs. cold `Witness` arena:** Keeping hot `lo/hi/t_to_go/word` (~24 B/plateau) in L3 while storing compact witnesses (~32 B) out-of-line.
9. **Turned frames ([footprints.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/footprints.md#L121-L142)):** Identity-only certification vs. allowing hashed turned frames (`pattern::model_digest`) to seed initial bisection brackets.
10. **Multi-cell staircase/fence joints:** How `JointPatternKey` scopes stepped/fence faces without fragmenting uniform staircases.

---

## 4. Fermi Floors (Stated Inputs)

Inputs: law tick `0.6â€“0.95 Âµs` ([engine.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/engine.md#L11)); 7800X3D L3 `96 MB` (`~10 ns` hit, `~80 ns` DRAM miss, [scales.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/scales.md#L126)); `~5` plateaus/edge ([check.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md#L22)); `24 B`/plateau hot (`120 B` < 2 cache lines); `32 B`/witness cold; `â‰¤ 64` halvings Ã— `2` Ã— `12` ticks (`~1,500` ticks/breakpoint, [learning-review.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/unified-plan/learning-review.md#L64)); `hardtrack` `~10Â²â€“3Ã—10Â³` faces (`old-graph.md:3`, `plan.md:147`).

- **Hot plateau lookup in `solve`:** **20â€“50 ns / joint** (`ReadCertificates::validate` + 16-lane SIMD compare + popcount).
- **Slide signal check:** **< 10 ns / joint** (`t_to_go` compare in L3, 0 replays).
- **Cold `JointPatternSet` build:** **~5 ms seeded / ~12 ms cold** (idle cores, 0 T cost while moving).
- **Face-reach graph on `hardtrack`:** **0.5â€“1 ms warm** (`~10Â²â€“3Ã—10Â³` faces Ã— `~20` mirrored lookups + `~20` worldtube checks).
- **Footprint edit invalidation:** **~0.3â€“1 Âµs / affected set** (`ReadCertificates::edit`).
- **Memory for `10â´` varied-terrain joints (`10âµ` patterns):** **~2 MB hot / ~18 MB total** (fits in 96 MB L3).

---

## 5. Premises Checked

| Premise | Status | Location & Evidence |
|---|---|---|
| `LabelId` and `Interner::label` | **EXISTS (`wt/3.2d-3` only)**; **MISSING on `main`** | [codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L128-L133), [L176-L178](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L176-L178), [L233-L254](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L233-L254); not found in `engine/law/src/` on `main` |
| `ClassId`, `Canonical`, `ProjectionTable` | **EXISTS (`wt/3.2d-3`)**; **DIFFERS** (`u16` shared with state classes) | [codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L16-L20), [L141-L174](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L141-L174); [3.2d-3-B-judge/last.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/3.2d-3-B-judge/last.md#L27) |
| `SectionRoles` SIMD presence planes | **EXISTS (`wt/3.2d-3`)**; **MISSING on `main`** | [codec_planes.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_planes.rs#L33-L93) |
| `ReadCertificates` reverse-indexed 4D proof | **EXISTS (`wt/3.2d-3`)**; **MISSING on `main`** | [codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L88-L183) |
| `StateReads` & `state_answers` | **EXISTS** | [state_reads.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/law/src/state_reads.rs#L29-L106); [codec_state.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_state.rs#L7-L86) |
| `Set` record (`plateaus`, `prices`, 4-way `status`, `witness`) | **MISSING**; `reach.rs::Cell` **DIFFERS** | [reach.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/reach.rs#L130-L148); [wt/3.2d-3 reach.rs](file:///D:/path-tracing-wt/3.2d-3/engine/plan/src/reach.rs#L145) |
| `law_hash` (auto-derived) | **MISSING** | [codec.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec.rs#L12); [codec_proof.rs](file:///D:/path-tracing-wt/3.2d-3/engine/law/src/codec_proof.rs#L52); not found in [Cargo.toml](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/law/Cargo.toml) |
| `Width { early, late }` & `clearance` | **EXISTS** (`early`, `late`); **MISSING** (`clearance`) | [segment.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/words/src/segment.rs#L117-L120) |
| Mirrored reach tables (`Reach`, `Field`) | **DIFFERS** (1 floor, `f64` `turn()`, cell centres) | [reach.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/reach.rs#L32-L127), [L179-L277](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/reach.rs#L179-L277) |
| `hard-track-2` reach certificate (`ReachLaw`, `FreeSpace`, `SupportGraph`) | **EXISTS (`wt/hard-track-2`)**; **DIFFERS** (conservative non-executable bounds) | [hard_track.rs](file:///D:/path-tracing-wt/hard-track-2/engine/plan/tests/hard_track.rs#L155-L387) |
| `hardtrack` shortcut geometry on `main` | **EXISTS** (old alignment wall `-85 -37 -99 -85 -32 -96`) | [hardtrack.build.txt](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/tracks/hardtrack.build.txt#L43-L45); [shortcut.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/hard-track/shortcut.md#L8-L9) |
| Double-tap `sprintWindow` in `engine/law` | **MISSING** in `engine/law` (bytecode pinned in `port-sprint-window`) | [julien-428.jsonl](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/hard-track/julien-428.jsonl#L1); [shortcut.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/jobs/hard-track/shortcut.md#L3); [HANDOFF.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/docs/HANDOFF.md#L46) |
| Kit-aware `Goal::lower_bound` | **DIFFERS** (bare-legs `C` only) | [goal.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/goal.rs#L7), [L29-L36](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/goal.rs#L29-L36) |
| `solve` loop & local `slide_want` | **MISSING** (`Planner` A* & full-replay `slide`) | [search.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/search.rs#L115-L513); [polish.rs](file:///C:/Users/PASCAL/Desktop/path%20tracing/engine/plan/src/polish.rs#L27-L64) |
| Plateaus finite & clean ULP flips | **EXISTS (2 patterns)**; **DIFFERS** on monotonicity (`12â†’14â†’12`, sub-grid sliver) | [trial-offset-monotone check.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md#L22-L50) |
