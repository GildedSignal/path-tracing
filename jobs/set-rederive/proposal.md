# How a set is stored: Opus's re-derivation after kill test 11 (17 November)

Saved by the integrator from the Opus Designer's final message; the harness blocks sub-agent writes. The experiment crate is in `kt11x/`. Build it with `CARGO_TARGET_DIR=D:\path-tracing-target\set-rederive cargo build --release --offline`. Modes: none, `image`, `pieces`, `mono`, `probe`. Variables: `WORLD=gap|step`, `YAW=`, `WT=`. It depends read-only on the `wt/u2-astra` crates.

**Verdict on the diagnosis:**
- Kill test 11 did not fail because sets cannot be finite. It failed because the record cuts the input side of a tick where the law has already forgotten the free coordinate.
- The translation premise was stronger than anything needs. Monotonicity, which holds, is enough.

## 1. Diagnosis

### Kill test 11
**What the test measures.** `engine/plan/tests/u2.rs:335-341` keys a plateau on the full per-tick event signature, `words::segment::joint_events`. That signature masks only the body-block crossing (`segment.rs:94-96`). The counter at `u2.rs:320-332` finds one cut per pair of differing samples, so it reports a lower bound that rises with resolution even on a finite partition. The gap pattern has real pieces only 22 and 13 ulps wide, at s = 96.33688815656168.

**Where the growth comes from.** `kill11.log` reproduces exactly (7, 11, 17), and pushing further gives:
- the test's own counter, at 64 to 65,536 divisions: 7, 11, 17, 55, 201, 715;
- every cut between samples, at 2^6 to 2^18 samples: 13, 15, 21, 59, 205, 719, 2,745, about 3.8× per 4× more samples;
- pieces by ticks to exit: 4 at every resolution.

At 2^18, 2,741 of the 2,744 cuts differ in a single bit, `minor_horizontal_collision`, at ticks 12–13. All of them lie inside the window where the body falls into the gap and hits its far wall.

**The cause is one line of the law,** `engine/law/src/movement.rs:804-820`: `acos(k/√(a·b)) < 0.1396`.
- At yaw 270 the intended vector is exactly (0.98, 0).
- After the wall clamps x, the moved vector is (dx, 0), exactly parallel, so the cosine is 1 up to rounding.
- When it rounds above 1, `acos` returns NaN and the flag reads false. Rust and Java both return NaN above 1.
- Which way it rounds depends on dx, and dx depends on s.
- The flag has a consequence: a collision that is not minor stops sprint (`movement.rs:228`). In `probe` mode, 1,848 of 20,000 starts lost sprint on the next tick, the same count as the starts whose flag was false.

**Oblique yaws show no growth.** At yaws 275.625, 264.375 and 281.25 the gap gives 6 pieces at every resolution from 2^6 to 2^18. A new one-high step at x = 100 behaves the same way: at yaw 270 it reaches 11,147 pieces at 2^18 and keeps growing; at yaw 275.625 it holds at 14.

**That partition cannot be stored.** No small automaton describes it. The flag depends on a rounded square, a square root and a division. Addition on the bits is finite-state (a carry); multiplication is not (Büchi 1960; Blumensath and Grädel 2000, both recalled). The only bound is the window's lattice: 0.4 block ÷ 2^-46 ≈ 2.8×10^13 values of s.

**On the output side the set is tiny** (`image` mode, 65,537 samples). Exit classes, taken as the exit state minus x:

| Pattern | Exit classes | Note |
|---|---|---|
| gap, yaw 270 | 7 | 4 once fall distance and the support block's absolute address are also removed |
| step, yaw 270 | 6 | once x and the resources are removed |
| step, yaw 275.625 | 10 | once x and the resources are removed |

Every class is one of two kinds:
- a **family**: as many distinct exit x as samples, a pure translation;
- a **point**: one exit x, where the law forgot s.

The rounding flag occurs only inside points.
- **On the gap,** the 2,742 signature pieces of the window give two exact states, sprint kept and sprint lost. They become bit-identical after 2 ticks of the next word and stay so through 60.
- **On the step,** the clip-and-land pair (5,672 and 6,975 samples, one exit x each) never merges in 60 ticks. That's a glancing contact, where sprint survives.

**Re-cut correctly, the count is stable.** Cutting only on events up to the first tick that clamps the free axis, with the flag masked (`pieces` mode), gives 5 pieces on the gap and 8 on the step. That holds at every resolution from 2^6 to 2^18, for yaw 270 and 275.625 alike. This is kill test 11 passing on the right object, on the patterns where it failed.

### The translation premise
**The witness** is at `u2.rs:301-318`: vx = 0.25 + 2^-47 at x = 100. On [64, 128) one ulp is 2^-46, so the added velocity is exactly a half-ulp, and round-half-to-even lets the parity of x decide.

**P2 already encodes this:** `t2.rs:56-59`, `Equal => ((parity+n)&1)^poison`, with two parity states per binade (`t2.rs:66-71`).

**The premise that matters is monotonicity.** A correctly rounded sum is monotone in its inputs (IEEE 754-2019, §4.3 and §5.1, recalled), and a clamp is a min. In `mono` mode, 1.6 million consecutive-ulp steps inside the gap's translating pieces each moved the exit by exactly 1 ulp; inside the pinned window, by 0.

**What monotonicity justifies:**
- the footprint as the per-tick hull of the two endpoints' boxes (spec rule 3, H2);
- the monotone pullback (H1, `joints.rs:282`);
- storing the exit at lo and at hi (rule 4).

Interpolating between those endpoints is wrong: the exit is evaluated by P2's map.

### The law fact that bounds the count
**Pieces.** Between collisions each axis moves by correctly rounded sums, so each comparison with a face flips once along s. Pieces per (family, word) are at most 1 + Σ over ticks of the faces between cells of different class inside that tick's endpoint hull. That bound is the word's footprint, not a number of ours.

**Points.** The only rounding-noise predicate in the movement law is `movement.rs:818`. The other nonlinear expressions (`:936`, `:961`, `:1108-1117`) are monotone thresholds. It is evaluated only on a horizontal-collision tick (`:805`), and its noise needs the moved vector exactly parallel to the intended one, which happens when the clamped axis is the motion axis. So it adds at most 2 points per pin and never adds pieces.

## 2. Candidates

**Fermi inputs:**
- the law's tick, 0.6–0.95 µs (0.75 used);
- a DRAM miss, 80 ns;
- a word of L = 12 ticks;
- W = 10–30 reach words per joint (`plan.md` §7);
- P = 5–8 pieces (measured);
- 2 points per pin (measured);
- one threshold found by 64-step bisection on P2's map, at about 50 ns per step (a Fermi figure): about 3 µs per cut;
- the hard track (S166): J = 10^3 joints over a great span.

### A. Families and points (recommended)
**What it is:**
- **The key:** the exact arriving state minus the free coordinate.
- **A family:** a lattice interval of the free coordinate, in integer ulps of its binade. It is cut into pieces only where a face between cells of different class enters the per-tick endpoint hull. The threshold is found by bisection on P2's map, never by sampling.
- **A translating piece's exit** is P2's monotone map of its entry.
- **A pinned piece's image** is at most 2^f exact points, with f = 1 today, one per value of the flag. Points that become bit-identical merge.
- **T** is pulled back per piece. Over points it is a bracket [min, max] that closes when they agree.

**Exactness:** two endpoint replays plus a sandwich. A monotone function equal at both ends of an interval is constant on it. The flag is the one exception: it is carried as both values, never cut on.

**Compression:** a pinned piece becomes 1–2 points. On the gap, about 0.4 block of s became 2 states, then 1. A translating stretch is one family.

**One mechanism:** the million line is one family through the gait, with no cuts, plus the core's terminal threshold. The hard track uses the same operations.

**Fermi:**
- **Cold:** (P × 3 µs) + (P + 3) × L × 0.75 µs ≈ 0.1 ms per (family, word), so about 1–3 ms per new family at a joint. The plan's figure is 12 ms per new joint pattern.
- **Warm:** a hash probe (80 ns), a sorted-array lookup (~10 ns), and one map evaluation or point read: about 0.1–0.2 µs per joint.
- **Hard track:** about 3 s cold, spread over thinking while moving; about 0.2 ms warm.
- **Million line:** one extra lookup of about 0.1 µs.
- **Memory:** about 256 B per (family, word), so about 7.7 MB cold for the hard track and about 64 KB hot.

**Kill test:** computed pieces must equal the sampled pre-pin partition at every resolution. Every sampled exit must equal its piece's prediction bit for bit, and at every pin the endpoints must be equal except for the declared flag. Any mismatch kills it, and an undeclared noise predicate shows up the same way.

### B. Exact points only, merged when bit-identical
**What it is:** today's singletons, plus merging.

**Exactness:** trivial.

**Compression:** only at pins, rest and gait recurrence. None on translating stretches.

**One mechanism:** yes.

**Fermi:**
- **Hard track:** about 0.27 s cold per trip, paid again on every trip that is not a bit-identical retrace, so compute grows with trips × joints.
- **Memory:** about 720 B per node, about 22 MB per trip, growing.

**Kill test:** a second trip whose start differs by one ulp costs as much learning as the first (learning.md kill test 4).

### C. Decision plateaus on the input side
**What it is:** the S182 record with the signature replaced by (T, next word, exit class), still found by bisection.

**Exactness:** only if every cut can be found, which is impossible on rounding-noise dust.

**Already killed** by the step at yaw 270: 6, 8, 8, 20, 66, 268, 2,122 decision pieces at 2^6–2^18 samples, still rising. The two points never merge, so the noise reaches the decision.

## 3. Recommendation: A
**Not a patch:** no block, tolerance or retry is added. The law's predicates are split by a property of the law: monotone ones cut, the one non-monotone one is carried. The flag list lives in `law/src`, under `law_hash`.

**Replacement for `docs/representation.md:179`:**

> - **Set** (what a joint holds): the arriving states from which a known sentence reaches the target, stored as **families** and **points** (S182, narrowed after kill test 11, `jobs/set-rederive`). The key is the **exact arriving state**: every field's bits the law reads as a value (velocity, flags, gait phase, the binade; `jobs/u2-redteam/report.md` C1), except the one coordinate a held word leaves free. Fields the law reads only through comparisons (exhaustion, saturation, food, fall distance) travel as uses with guards, and the supporting block is kept relative to the body. A **family** is an interval of the free coordinate's lattice, cut into **pieces** only where the word's swept box crosses a face between cells of different class. The cuts are computed from the word's footprint and the exact integer maps (P2), never found by sampling, so there are no more of them than the faces the word crosses. Inside a piece the exit is that monotone map of the entry. Where the word clamps the free coordinate, the law forgets it: the piece's image is a few exact **points**, one per value of the law's non-monotone flags (today only the minor-collision test, `movement.rs:818`), and points that become bit-identical merge. Each piece carries its T and next word (over points, a bracket that closes when they agree), its premises (S95) and the prices of what is in hand (S96).

**Two parts narrow S182 and need his ruling:**
1. **Resources leave the key.** On the step at T = 21, 19,573 samples pinned to one exit x still gave 34 exact states differing only in exhaustion, from the centimetre-rounded walk distance (`movement.rs:1117`).
2. **The support block is stored relative to the body.**

**Lines that follow, for the integrator:** representation.md:180, :192, :199; learning.md:35, :52, :74-75, :127 (kill test 11 now applies to pieces), :145.

**Anomaly for HANDOFF:** running exactly along an axis into a wall keeps or drops sprint according to how a cosine rounds. That's nature's own behaviour, faithfully ported, and only axis-aligned yaws hit it. The bot's 64-yaw grid includes those yaws.

## 4. What the builder changes on `wt/u2-astra`
**Stays:**
- the exact entry keys;
- certificates bound to (pattern, piece index, `law_hash`);
- the wave over certified edges;
- the face inventory, the four statuses and kill test 10;
- `pullback` (`joints.rs:282`);
- H2's endpoint hull and H1's refined partition, now justified by monotonicity;
- the singleton facts, which become the "point" case.

**Changes:**
1. **`law`:** a declared list of non-monotone fields, today `minor_horizontal_collision` only, with its source line.
2. **`EntryTemplate`** (`joints.rs:10-41`):
   - exhaustion, saturation, food and fall distance leave the key and travel as uses with guards (`Uses`/`Guards` in `segment.rs`);
   - the supporting block is stored relative to `floor(position)`;
   - the float-slope chart (`:16-23`) is replaced by the lattice coordinate on the dominant axis.
3. **`Plateau` becomes `Piece`** (`joints.rs:55-59`, `:214`): lo and hi as lattice bits, the exit as `Map(P2 id)` or `Points([≤ 2^f])`, the uses as an interval, the certificate, the status.
4. **`learn()`** (`:153-185`), with no divisions and no sampling:
   - replay the entry;
   - find the faces that change class inside the per-tick hull;
   - find each threshold by bisection on `events::Map::advance`;
   - replay lo and hi;
   - check the pre-pin signature (flag masked), and at a pin, bit-equality except the flag;
   - replay the tail from the pinned state with each flag value.
5. **`consume()`:** the key, s in [lo, hi], the guard premises, the certificate.
6. **`wave()`:** nodes become (family, piece) pairs and points. Over points with different T, the certified upper bound is their max and the lower bound their min.
7. **`u2.rs`:**
   - keep `u2_kill11_refined_bisection_diagnostic` and `u2_translation_premise` as recorded reds of the old object, never weakened;
   - add a monotone-premise test that keeps the tie witness (steps of 0 or 2 ulps, still monotone).

**The test that must pass first,** frozen before its first run: `u2_pieces_computed_and_closed`.
- **Inputs:** the gap and step patterns at yaws 270 and 275.625, `hardtrack-lane-wall` and `glm-ice`, against 2^18 samples plus ±8 ulps around every computed cut.
- **Checks:**
  - (a) the sampled pre-pin partition equals the computed pieces;
  - (b) every exit is bit-equal to its piece's prediction, with resources inside the piece's use interval;
  - (c) at every pin, the endpoints differ only in the declared flag.
- **Expected:** 5 pieces on the gap, 8 on the step, 2 on each of the last two.

Kill test 11 is then this test, applied to its own word, "pieces".

**Sources,** all recalled and not read in this job: IEEE 754-2019 §4.3 and §5.1; the Java SE `Math.acos` documentation; Nerode 1958; Büchi 1960; Blumensath and Grädel 2000; Basch, Guibas and Hershberger 1999. Mojang's `LocalPlayer.isHorizontalCollisionMinor` was not re-read: the port at `movement.rs:804-820` was relied on.
