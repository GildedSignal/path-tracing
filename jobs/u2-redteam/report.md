FREEZE AFTER FIXES

# Red-team: joints and sets (step 2), before it freezes

By the Opus Designer, 16 November. The harness blocked the sub-agent's own write, so the integrator saved this text verbatim from its final message. The offline check is in `kt10/` (run `cargo run --release --offline` with `CARGO_TARGET_DIR=D:\path-tracing-target\u2-redteam`; it depends read-only on the `wt/hard-track-2` crates).

## Verdict
These parts hold:
- the split into a goal-free pattern record and a goal-bound instance;
- the four statuses;
- the footprint-driven invalidation;
- the plan for reach between faces.

The key does not: as drafted, two arrivals with different futures get the same key and the same offset s. There are two more structural faults:
- the read path cannot validate any arrival except the witness itself;
- the goal-bound T-to-go is laid on the wrong partition.

All of them can be fixed inside the draft's own structure, so the verdict is **freeze after fixes**, not redesign. One fix narrows the approved text of `docs/representation.md:179` ("the yes-or-no class, plus plateaus of the one coordinate"), approved in S165; Julien should be told.

## Kill test 10, offline on the old build
**Setup:** `jobs/hard-track-2/old.build.txt` (identical to main's `lab/tracks/hardtrack.build.txt`), the law on `wt/hard-track-2`, fixed yaws on the 64-yaw grid. A held word keeps its keys until the body lands after being airborne or catches a climbable, at most 80 ticks.

- **A. Lane end at rest → the alignment-wall top** (x −85, z −99..−96, feet −31). 28 starts at rest on the southern lane give 147 (start, word, yaw) hits, 53 of them from cell centres. Fastest: 10 ticks (FSJ yaw 326.25 from (−86.5, −101.0)). So the topology is in even a from-rest relation sampled at cell centres.
- **B. Wall top at rest → the final ladder.** 18 starts give 75 hits. The highest catch is feet y −31.3235, 12 ticks in. He caught at −30.5, 9 ticks after his jump.
- **E. The T bar.** Starting at rest at his ladder exit (−105.47, −28, −105.56):
  - the search ran five fixed-yaw words, then the first lane word held along the lane, then three held words through the wall top;
  - it found 46 prefixes onto the lane, 4,078 wall hits, 15,404 ladder hits and 3,872 sentences to B;
  - the best takes **158 ticks** after the exit, against his 169: `FS y298.125 x29 | FSJ y270 x23 | FSJ y348.75 x24 (to the wall top) | FSJ y354.375 x9 (to the ladder) | F y0 x73`;
  - if the bot leaves the ladder at his tick 259, its T is about 417 against his 428;
  - this is the law offline, with no server audit. B is touched under `plan::Goal::touched`, the same criterion under which his own run touches B at tick 428.
- **What it means for this step:**
  - The winning lane-to-wall word starts at speed, about 0.45 blocks per tick east, so a from-rest relation must charge a stop at that joint. Braking on blue ice from vx 0.4336 with no keys takes **49 ticks** to vx == 0, against an 11-tick margin.
  - So the map holds his route's topology. Matching or beating 428 needs sets that carry entry speed, and the draft's key does not (C1).
  - **Caveat:** restricting the first lane word to yaw 270 is a prior taken from his route. A and B are unbiased.

## Findings by severity (file:line; smallest fix)

### Critical
**C1. The key is unsound: different futures, same key and same s.**
- **Where:** the draft, `jobs/u2-set-record/report.md:25`. The entry class is `state_answers` plus "speed-band comparisons inside a site".
- **Why it fails:** `state_answers` keeps velocity only as the law's comparisons (`D:\path-tracing-wt\kernel\engine\law\src\codec_state.rs:27-55`), and no law comparison buckets horizontal speed. So "speed bands" either do not exist or are a made-up grid.
- **Witness (lens D):** a start at (−86.5, −28, −101.5) with vz 0.06, word FS yaw 0:
  - vx 0.24 and 0.36 give the same answer to every comparison in `codec_state.rs:30-55`;
  - yet vx 0.24 lands on the wall top (y −31), and vx 0.36 lands on the decoy floor (y −37);
  - vx 0.06 and 0.12 fall to the ground floor (y −60).

  This is the Frazzoli warning (`jobs/frazzoli/report.md:167-168`) and `docs/vocabulary.md:102` ("merging them is R-brach's kill").
- **Fix:** the key holds **the exact bits of every state field except s**, interned as the entry template: velocity, flags, phase, the fractional bits of the perpendicular coordinate relative to the joint, and the binade (H3).
  - Keys then recur exactly where the law pins the state: a settled gait, rest, a climb clamp, a collision that zeroes a component. They never recur inside momentum's memory. That is "inside a site, order matters and sets carry state" (`jobs/unified-plan/plan.md:42`), one rule for every terrain, so kill test 8 still holds.
  - On `hardtrack`, blue ice forgets slowly. Ground drag is 0.989 × 0.91 ≈ 0.90 per tick, so momentum's memory there is longer than the 169 ticks from the ladder to B, and the whole post-ladder route is one site, chained from the ladder-top climb gait. Lens E did exactly that.

**C2. The read path refuses every arrival except the witness.**
- **Where:** the draft, `:55-58` and `:86`, validates `ReadCertificates` against a `Binding`.
- **Why it fails:** `Binding` holds the full `PlayerState::bits()` (`kernel\engine\law\src\codec_proof.rs:48-57`). `answer` returns `StateOrContext` unless the bindings are equal (`codec_proof.rs:136`). A plateau covers many values of s, so every value but one is refused.
- **Fix:** bind a certificate to (JointPatternId, plateau index, law_hash). Acting then checks that the key is equal, that s lies in [lo, hi], and that the certificate is not invalidated. After that change the 20–50 ns floor is plausible. Before it, validation compares about 720 bytes and is wrong anyway.

### High
- **H1. T-to-go sits on the wrong partition, and the continuous exit is lost.**
  - **Where:** the draft, `:51`, lays `t_to_go` parallel to the pattern's intervals. `:44` keeps only an integer `exit_delta_pos [i32;3]`.
  - **Why it fails:** within a plateau the exit coordinate moves with slope 1 in s (`lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md:26`), so a downstream breakpoint can fall inside an upstream plateau.
  - **Fix:** each instance node gets its own breakpoints. These are the pattern's breakpoints plus the preimages of the downstream ones, found by bisection on "lands in downstream plateau k". That is the pullback in `jobs/frazzoli/report.md:160-162`. The pattern stores the exit coordinate's bits at lo and at hi. Within a plateau every field is monotone (`check.md:25`), so each preimage is one interval.
- **H2. A plateau can change without its footprint being touched.**
  - **Where:** the draft, `:46`, takes "the union over the endpoints plus an interior sample". That misses cells read only by interior values of s. The plateau predicate at `:68` (exit kind, exit block, delta_t, next word) also lets the collision sequence change inside a plateau.
  - **Fix:** split plateaus at any change in the law's discrete event signature (`docs/vocabulary.md:39-43`, without the body-block cut). Within one binade, motion inside a plateau is then a pure translation, and the footprint is the per-tick hull of the two endpoints' query boxes: exact, with no sampling.
- **H3. The binade is missing from the key.** Breakpoints are single-ulp bit patterns, and translation is exact only within a binade (`docs/representation.md:174`, P2). A pattern learned near x ≈ 100 has different ulp breakpoints near x ≈ 1000.
  - **Fix:** put the per-axis binade of position in the key, and store s relative to the joint's origin.
- **H4. Forbidden has no footprint and no premises.**
  - **Where:** the draft, `:39`, `:66`, `:129-131`.
  - **Why it fails:** Forbidden is proved with `ReachLaw` and `FreeSpace`, whose premises (integral Y planes, three frictions, no fluids: `D:\path-tracing-wt\hard-track-2\engine\plan\tests\hard_track.rs:344, 353-356`) are asserted, not carried. The proof reads whole altitude slabs (`hard_track.rs:305-333`), but only the instance node's reads are registered (`:78`). So an edit that opens a path never reopens Forbidden.
  - **Fix:**
    - a Forbidden certificate records its premises and the cells of its separating component in `ReadCertificates`;
    - off-premise terrain (slabs, stairs, fluids) gets Searched or Refused, never Forbidden;
    - an edit inside a Forbidden footprint turns it into Searched.
- **H5. The backward wave replays candidate arcs inline, in acting.**
  - **Where:** the draft, `:119-123`.
  - **Why it fails:**
    - The law replays only a joint pattern seen for the first time, which is learning, or a broken certificate (`docs/vocabulary.md:20`). Learning has one queue (`docs/learning.md:64`, kill test 23).
    - The reach tables are keyed by an undefined `entry_speed_class` (`:106`), and C1 shows that no speed class drawn from the law's comparisons is sound.
    - The Pareto step dedups per `(dx_cell, dz_cell)` (`:111`), which brings cell quantisation back.
  - **Fix:**
    - the wave reads only certified edges;
    - an uncertified arc becomes a queue item, with its envelope lower bound as the bracket;
    - mirrored tables are keyed by the exact entry template (C1), so they serve rest, gait and clamp arrivals exactly, and nothing else.

### Medium
- **M1.** The draft writes the half-width as `0.3` (`:34, :113, :117, :131`). The law computes `(0.6f32/2.0) as f64` (`hard_track.rs:264`). P2 already stopped on this exact trap (kernel commit 3d4e52d).
- **M2. The takeoff domain is drawn from geometry, not from the law.**
  - **Where:** the draft, `:34` (the face plus the overhang).
  - **Why it fails:** the law collides on Y first, so a grounded tick can end outside that domain. Witness: his t318 (`jobs/hard-track/julien-428.jsonl`), on_ground with its centre at z −100.4387, 0.56 past the lane edge at −101; the body edge is 0.26 past.
  - **Fix:** the domain is the set of s values the law reports as grounded.
- **M3. One u16 table holds both cell classes and exact states.**
  - `ClassId(u16)` refuses at 65,535 (`kernel\engine\law\src\codec.rs:173, 208`).
  - Exact start states are interned into the same table (`pattern.rs:226`, `StateQuestion::Exact`; `footprint.rs:55`; `words\src\table.rs:200`).
  - **Fix (the draft's option A, now mandatory):** ClassId u16 holds cell answers only. Separate types hold the rest: StateClassId u32 and JointPatternId u32.
- **M4. The face-reach graph's Fermi floor contradicts its own inputs** (`:196`).
  - From those inputs, the lookups alone take 0.1–3 ms. About 20 arc checks of 10 µs per face give 20 ms to 0.6 s cold, not 0.5–1 ms.
  - The "about 20 neighbours" is unmeasured: the old conservative graph has 3,548,489 edges over 3,013 supports, about 1,178 per support (`D:\path-tracing-wt\hard-track-2\jobs\hard-track-2\report.md:9`).
  - With C1's exact templates, budget the cold template arena separately from the hot key ids.
- **M5. The face inventory has no footprint.**
  - **Where:** the draft, `:113`, defines faces by reads such as "!above.is_block()".
  - **Why it fails:** a placed block can create a face no joint sees.
  - **Fix:** register the inventory's reads, and let an edit enqueue new joints.

### Low
- **L1.** "16-lane" over f64 is wrong: an AVX-512 register holds 8 f64, so 16 values take two compares.
- **L2.** The target region carries an epsilon: it is grown by 1e-3 (`engine/plan/src/goal.rs:11-17` on main), and every set's target inherits it. The clicked face (`wt/track-face`) should replace it.
- **L3.** The law_hash must cover `sin.bin`, `models.tsv`, the registry and the law sources. It replaces `Binding::version` (`codec_proof.rs:52`) and the static `SCHEMA` (`codec.rs:12`).
- **L4.** Prices are unspecified. Until two witnessed T values exist, take them from the difference between the kit's light-cone lower bounds.
- **L5.** The draft cites codec code on `wt/3.2d-3`; the same files are on `wt/kernel` (`docs/plan.md:34-35`).

## Decisions the builder needs closed (recommendations)
1. **The plateau coordinate:** use a 1D s along the arriving motion's line (the draft's option C), with everything else exact in the key (C1, H3).
   - Reject 2D rectangles (option B): oblique yaws make oblique outcome boundaries, so the rectangle count grows with refinement and fails kill test 11 by construction.
   - Reject option A's bands in w: they merge values of w whose futures differ at the ulp.
   - Arrivals at rest have no line; key them exactly in both coordinates.
2. **Completeness:** make the draft's option B mandatory.
   - Forbidden only by an envelope proof that carries its premises and footprint. Anything a scan missed is Searched.
   - Seed the cuts from shape planes ± the law's half-width and from family neighbours, with no grid constant.
3. **The goal-free / goal-bound split:** keep it, with H1's refined partition. Local premises are the pattern key. Prices and T-to-go are goal-bound.
4. **IDs:** split them as in M3.
5. **Premises:** store `ReadCertificates` with interned ClassIds, but bind them to the pattern and plateau, not to the state (C2).
6. **law_hash:** the draft's option A, an automatic build hash, with the coverage in L3.
7. **Witness:** per plateau, the s bits plus the end digest. The exact entry template is the key, kept cold and interned. Reject option B: re-deriving the witness from a class is unsound (C1).
8. **Frames:** the identity frame for certifying. Turned frames only seed brackets (the draft's 9B).
9. **Stairs:** do not decide by hand. A stair run is a gait whose count the corridor query finds (step 1, the core) if, and only if, the state recurs bit for bit per step. Measure that before choosing.
10. **The wave** never replays (H5).
11. **The domain** comes from the law (M2), and **the face inventory** is footprinted (M5).
