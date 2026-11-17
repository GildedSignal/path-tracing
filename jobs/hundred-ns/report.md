# The idiot index, a swarm on a bounded map, and the 100 ns rethink

Opus (designer, fresh context), 17 November. Saved verbatim by the integrator: the sub-agent's harness refused the write. Spot-checked by the integrator against `95552dc`: `goal.rs:7` (`C = 19.902886/20`, blue ice's speed), `terminal.rs:19` (the stop key carries `gait:u64`, a kernel id), `tests/zones_prereqs/u1.rs:60-69` (a cold solve on a fresh store before every warm sample). Read: PRINCIPLES 1–47 and the named register rows, plan.md, learning.md 0 and 6, representation.md 10, scales.md, engine.md, vocabulary.md, the u1 judge item 8, the u2b checkpoint, the sweep log `D:\path-tracing-evidence\u2b\integrator-sweep-20261117.log` and the other logs there, and the code at `95552dc` (`engine\plan\src`: solve, store, events, lattice, terminal, joints, state_view, inline, goal, source). Nothing was built or run. Measured numbers are quoted with their source; every floor states its inputs; inferences are marked.

## 0. Fermi inputs (this machine, a 7800X3D)

About 5 GHz (0.2 ns a cycle). L1 hit about 1 ns, L2 about 3 ns (Zen 4 published 4–5 and 14 cycles), L3 about 10 ns, memory 80 ns (scales.md 4.7); L2 1 MB a core, L3 96 MB. A dependent f64 add about 0.8 ns; a 64-bit divide about 3 ns; a 128-bit divide (library) 15–30 ns. A std `HashMap` probe with SipHash on a 30–60-byte key: 30–60 ns (Fermi input, not measured). The law's tick 0.6–0.95 µs (engine.md, measured). A player state about 360 bytes, six cache lines, from the field list (`law/src/state.rs:53-121`), plus a heap vector of speed modifiers. The million line on the axis: 2,806,325 ticks (log line 297), so 0.3564 blocks a tick; the sprint-jump cycle is 12 ticks.

## 1. Part 1: the idiot index

### 1.1 What was measured, and what the measurements hide

| Run (same build, `95552dc`) | axis X | axis Z | oblique |
|---|---|---|---|
| Warm, back to back (`U2B_WARM`, log 584–598) | 57.3 µs (p99 61.3) | 69.0 (p99 78.2) | 91.7 (p99 105) |
| Warm, each right after a cold solve on another store (`U1_SOLVE` warm, log 292–312) | 112 (p99 156) | 124 (p99 176) | 167 (p99 227) |
| Cold, fresh store (`U1_SOLVE` cold) | 199 ms | 215 ms | 192 ms |
| Stages back to back: entry / bearing / inverse / output | 3.4 / 10.8 / 30.6 / 12.0 | 4.2 / 10.7 / 41.5 / 11.9 | 4.8 / 3.0 / 60.1 / 23.1 |
| Stages after a cold solve | 12.2 / 14.7 / 64.1 / 18.5 | 13.1 / 14.8 / 75.0 / 18.5 | 13.8 / 5.4 / 110.9 / 33.5 |

**57 against 112 µs is the same code.** The U1 harness runs a cold solve on a fresh store before every warm sample (`tests/zones_prereqs/u1.rs:60-69`); the u2b harness does not (`tests/u2b_cost.rs:45-47`). The cold solve builds about 1.7 MB and evicts the warm store from L1 and L2. Whenever anything runs between two solves (another bot, the server, learning), the warm solve costs 55–75 µs more. Inferred: at about 10 ns an L3 line that is about 5×10³ lines, so a warm solve touches about 350 KB. **A swarm interleaves; 112–167 µs is the number to beat, and the bench should report it.**

From the same harness: a 1,000-block warm solve 61.8 µs; a 10⁴-block oblique one 93.5 µs (`u1_native_ten_thousand`), against 112 µs at 10⁶. From 10³ to 10⁶ blocks adds about 10 binade crossings, so each costs about 0.3 µs summed over the walks that repeat it (inferred); the other half of the solve is independent of length.

### 1.2 The floor of the warm million line, stage by stage

Keeping today's representation (exact maps per binade, two yaws, the stop on B stored):
- **Entry, about 0.15 µs:** the label (4 cell reads, one interned probe, about 45 ns), the yaw pair (16 steps over the sine table, about 65 ns), the bundle (one probe, about 30 ns).
- **Bearing, about 0.05 µs:** a 2×2 system on dyadic integers, a dozen 128-bit products, two divides.
- **Inverse, about 0.5 µs:** one exact chain per axis, about 14–21 crossings each at about 10 ns (one dense table read and an integer multiply-add): 0.3–0.4 µs; one divide for the last binade; the stop on B for the phase the touch tick fixes (one or two probes, about 10 ns each).
- **Output, about 0.1–0.2 µs:** three or four word records, the corridor box from stored per-phase extrema, one corridor query.

**The floor is about 1 µs** (the plan's ~5 µs had slack). Back to back, 57–92 µs is 55–90× the floor; interleaved, 112–167 µs is 110–165×.

### 1.3 The table, ranked by excess per trip

Weights (S83: how often and how fundamental): a trip has s = 1–10 solves on the click path (the click plus S189's retargets) and 10–100 joint decisions off it. A share P = 0.1–1 of solves start a new bearing or from a moving state (1.4). Excess per trip = (measured − floor) × runs per trip; click-path items in ticks (T is the one currency), the rest in CPU.

| # | Item (grain) | Measured | Floor | Ratio | Excess per trip | Head (S83) | Delete, or the smarter form |
|---|---|---|---|---|---|---|---|
| 1 | The click path outside the planner (holistic) | dry-run click 302 ms after `cargo run` was removed (HANDOFF, 14 November, flying stack `wt/3.3g-play`) | dispatch at the next tick, planner in the bot's process: ≤ 1 tick | ≥ 6 | 5–50 ticks of T | work where it does not matter: µs polished while 300 ms sits on the path | fly through the direct channel already built (JNI, `slice A`); measure T from the click's server tick (S124). Not re-measured on the core: measure it first |
| 2 | A new bearing or a moving start is a cold bundle (meso, macro) | 192–215 ms; 4,785 law ticks; 82 kernels prepared (log 291, 297; `PROBE learned=82`) | two yaw kernels from a yaw-free generator, 2 × 275 ticks × 2 axes × ~3 ns ≈ 3.3 µs, stop families keyed yaw-free: **about 10 µs**. Even against its own law ticks: 2.9–4.5 ms | 44–70 against its law ticks; ~2×10⁴ against the derivation | 2×10⁴–2×10⁶ µs CPU; 0.4–40 ticks when the first word waits on it | not leveraging symmetry: the aim is an argument (S202), yet kernels are keyed by yaw and the start's velocity bits (`store.rs:193-201`). Duplicate: up to 147 kernel slots per bundle (`solve.rs:90-105`), four legs read. Input that does not matter: each prepared tick clones a 360-byte state and hashes all its fields (`store.rs:211-231`). Work where it does not matter: the stop set bisects the 64-bit ordered domain with two full readouts a step, per tick, per axis (`terminal.rs:42-53`) where the exact map inverts with one divide | store the generator (engine.md): per label and gait, the yaw-free vertical cycle plus a per-tick horizontal multiplier and acceleration; a yaw's kernel is two scalar recurrences, certified once against the law for all 65,536 yaws; the stop on B as families over the aim |
| 3 | The lower bound is blue ice's speed on every terrain (holistic) | `Goal::C` = 0.995 blocks a tick (`goal.rs`); on stone the bracket at the click is [1.0×10⁶, unknown] against a true 2.8×10⁶ ticks, so it never closes by bound | the kit's cone on the certified label: the hull of its gait speeds (scales.md 4.3); the corridor query already proves the label | — | puts items 2 and 4–9 on the click path | not leveraging a stored result (the corridor certificate) | a label-scoped lower bound closes the first word's bracket at the click (learning.md 0, step 2): the first word in about 0.2 µs, the rest on deadlines |
| 4 | The warm solve's cache footprint (micro, macro) | +55–75 µs interleaved (1.1) | about 16 lines, about 1 KB | ~350 | 55–750 µs | input that does not matter: 360-byte phase states read for y, pose and velocity only (`store.rs:297-305`, `state_view.rs:17`); graphs as `Arc` nodes of `Vec<i128>` behind a `RefCell<HashMap>`; word steps with 48 inline events | phase records ≤ 40 B; graphs as a dense array per map indexed by exponent, phase, parity; a bundle record ≤ 1 KB |
| 5 | The distance walked again and again (micro) | about 10–20 full-length walks per solve, counted from `solve.rs:109-209` (first crossing, anchor, both bearings' two legs, each leg's envelope, each step's event trace); about 0.3 µs a crossing summed (1.1) | one chain per axis | 5–10 | 40–500 µs | duplicate work | walk once per axis, keep the crossings as the solve's table; every readout reads it |
| 6 | The stop on B looked up 36 times (micro) | 12 phases × up to 3 legs; each builds a request, SipHashes a ~56-byte key, compares the 32-byte law hash, scans rows linearly, reads out a state (`joints.rs:145-152`, `terminal.rs:23-71`); inferred 25–35 µs from the 1,000-block solve's 38.9 µs inverse | 1–2 probes, about 20 ns | ~10³ | 25–350 µs | work where it does not matter; not leveraging the bound | the touch tick fixes the phase; prune by lower bound first (u1 judge fix c); a content key into a dense table; the law hash checked once when the store opens |
| 7 | Output (micro) | 12–23 µs | 0.1–0.2 µs | ~100 | 12–230 µs | work where it does not matter: 14–21 binade events per axis per word emitted, sorted, deduplicated (`solve.rs:51-57`), read by nobody, since the client runs the law and re-anchors at its own events | emit words, counts and the corridor box; events only when looked at (engine.md, "How it computes") |
| 8 | The bearing on an axis (micro) | 10.8 µs on X or Z; 3.0 µs oblique | about 0.05 µs | 60–215 | 3–110 µs | near the axis the closed form overflows 128 bits; `counts` then binary-searches the count cap with about 32 feasibility tests and 256-bit bit-by-bit products (`lattice.rs:31-49`, `167-175`) | the closed form in 256 bits, or two divides under the bound (idea 4) |
| 9 | Entry (micro) | 3.4–4.8 µs | about 0.15 µs | 22–32 | 3–50 µs | duplicate and input that does not matter: the motor key recomputed field by field per kernel in the bucket, two SipHash probes, the 263-tick transient read out (`store.rs:171-201`) | one content key per observed state, computed once at the client's event |
| 10 | A warm decision on a seen track (micro, meso) | **not timed in any log.** By the code every solve calls `route` (`solve.rs:74`), which allocates the start state's full bits and SipHashes them (`joints.rs:301`), clones each step, and on a new goal rebuilds the backward field over every stored edge (`joints.rs:284-299`) | one piece probe, 20–50 ns | unknown | 10–100 decisions a trip | not leveraging: the pieces exist (`family_wave.rs`) but the route is keyed by exact states, so a bot one ulp off misses; the zero-allocation gate passes only because the million line's joint store is empty | the route reads the pieces (families, points, holds) keyed by joint pattern and free coordinate; time it in the gate for his shortcut (`u2_kill10_discovered_program_current_record`) |

At its floor, learning a joint pattern takes 2,894–4,838 law ticks for 2–12 pieces (`U2B_ALL`), under representation.md's 1.2×10⁴; its wall time is not logged.

### 1.4 The four grains and the four heads

- **Micro (one solve): items 4–9.** Duplicate walks, a hot set two orders too large, lookups the touch tick already fixes, events nobody reads. The ~1 µs floor is reachable without changing the representation.
- **Meso (a trip whose goal changes again and again, S189).** A retarget from a moving state is a cold bundle (item 2): bundle and kernels are keyed by the start's velocity bits, and a gait has 12 phases a yaw. Inferred from the keys: every mid-run retarget to a new bearing costs about 200 ms, about 4 ticks.
- **Macro (a swarm over a session).** A bundle is about 4,785 states × 360 B ≈ 1.7 MB (inferred). One label's standing starts alone need 65,536 bundles, about 110 GB, against 31 GB RAM and 96 MB L3. The store cannot stay warm, so item 2 runs on most trips.
- **Holistic.** T is lost on the click path (items 1–3), not in the warm solve. At 57 µs thinking costs an expected 0.001 tick a decision (57 µs of the 50 ms to the next dispatch); transport about 6 ticks, a cold bundle about 4.
- **By head:** duplicate 2, 5, 9; input that does not matter 2, 4, 9, 10; work where it does not matter 1, 2, 6, 7, 8; not leveraging 2 (the generator's yaw symmetry), 3 (the corridor certificate), 6 (the bound), 10 (the families), and S83's guided missile: the client computes the position bits as it moves, so the planner need not (idea 4).

## 2. Part 2: a swarm on a bounded map

### 2.1 Inputs (change any)

| Input | Range | Middle |
|---|---|---|
| Map side | 10³–10⁴·⁵ blocks | 10⁴: 10⁸ columns, 3.9×10⁵ chunks |
| Bots N | 10–10⁴ | 10⁴ |
| Session | 1–24 h | 8 h = 5.76×10⁵ ticks |
| Trip length | log-uniform 10²–10⁴ blocks | median 10³ |
| Dwell at the goal | 0–1× the trip | 1× |
| Retargets per trip (S189) | 0–9 (1–10 click-path solves) | — |
| Joint decisions per trip | 10–100 on natural ground; ~0 on a plain | — |
| Block edits | 10–10³ a second | — |
| Kit changes | 1–10 per bot an hour | — |
| Speeds | sprint-jump 0.356 blocks a tick (7.127 m/s); elytra with rockets 1.5–1.75 (30–35 m/s, an input from play, to measure once ported) | — |
| Click path today | ~6 ticks transport (item 1) + 4 ticks × P cold bundle | — |

### 2.2 Total T, and how much is thinking

A trip's T on legs is 280–28,000 ticks, median 2,800 (140 s): about 100 trips per bot per session (10–1,000). At N = 10⁴: 10⁶ trips, about 2.9×10⁹ moving bot-ticks. A decision on the click path loses an expected t / 50 ms ticks (t < 50 ms):

| On the click path | Ticks lost a decision | Per trip (1–10 solves) | Share of a 2,800-tick trip | Of a 280-tick trip |
|---|---|---|---|---|
| 112–167 µs (today, interleaved) | 0.0022–0.0033 | 0.002–0.033 | 10⁻⁶–10⁻⁵ | 10⁻⁵–10⁻⁴ |
| 57 µs | 0.0011 | 0.001–0.011 | 4×10⁻⁷–4×10⁻⁶ | 4×10⁻⁶–4×10⁻⁵ |
| 5 µs | 10⁻⁴ | 10⁻⁴–10⁻³ | 4×10⁻⁸–4×10⁻⁷ | ×10 |
| 1 µs | 2×10⁻⁵ | 2×10⁻⁵–2×10⁻⁴ | 10⁻⁸–10⁻⁷ | ×10 |
| 100 ns | 2×10⁻⁶ | 2×10⁻⁶–2×10⁻⁵ | 10⁻⁹–10⁻⁸ | ×10 |
| Cold bundle 200 ms (P = 0.1–1) | 4 | 0.4–40 | 0.014–1.4% | 0.14–14% |
| Transport today | ~6 | 6–60 | 0.2–2% | 2–21% |

Session at N = 10⁴: transport 6×10⁶–6×10⁷ bot-ticks; cold bundles 4×10⁵–4×10⁷; warm thinking 10³–10⁴ at 57 µs, 2–20 at 100 ns. **Below about 1 ms a decision, thinking does not move T; transport and cold work on the click path do.**

### 2.3 CPU per bot per trip, beside the server's 50 ms tick

Warm decisions (10–100 a trip plus the click-path solves): 1.1–17 ms at 112–167 µs; 0.6–5.7 ms at 57 µs; 0.05–0.5 ms at 5 µs; 0.01–0.1 ms at 1 µs; 1–10 µs at 100 ns. Cold bundles: 200 ms × P × s = 20 ms–2 s a trip today, about 10 µs derived. New joint patterns: 3–12 ms each; new per trip falls along the walk (Heaps' law), 10–100 early (30 ms–1.2 s), about zero late.

The swarm per tick: N = 10⁴ starts about 36 trips a second, so 360–3,600 joint decisions a second, 18–180 a tick:

| Per decision | Core-ms per 50 ms tick | Share of one core |
|---|---|---|
| 112–167 µs | 2–30 | 4–60% |
| 57 µs | 1–10 | 2–20% |
| 5 µs | 0.09–0.9 | 0.2–2% |
| 1 µs | 0.02–0.18 | 0.04–0.4% |
| 100 ns | 0.002–0.018 | < 0.04% |

Cold bundles today: 36 × P × s × 200 ms = **0.7–72 cores**; derived, about zero. For scale, representation.md's server floor for 10⁴ lazy bots is about 3 ms a tick (about 300 ms naive).

### 2.4 Where 100 ns changes the balance

**Per-tick decisions** (crowds: `Entity.push` rewrites both velocities on every overlapping tick, scales.md 4.4; chases; moving goals; fights): 10⁴ × 20/s = 2×10⁵ decisions a second: 22 cores at 112 µs, 11 at 57 µs, 1 at 5 µs, 0.2 at 1 µs, 0.02 at 100 ns. Keeping planning under the server's ~3 ms a tick needs t ≤ 3 ms / 10⁴ = **300 ns: the anchor. 100 ns matters exactly when bots decide every tick; decisions at the rate of events are satisfied at about 5 µs.**

**Compute against lookup.** At 100 ns recomputing costs about one memory miss (80 ns): a stored answer beyond L3 costs as much to fetch as to derive. Store only what is costly to derive (joint sets: ms of law ticks; generators) or hot enough for L3; derive the clear-terrain answer every time. At 57 µs every stored answer was ~700× cheaper than recomputing, which is why goal-keyed caches looked attractive.

**Where 100 ns does not matter:** T for clicked goals (2.2); learning, ms a new pattern, which dominates CPU until patterns saturate; edits (2.5).

### 2.5 What to store, what to recompute

| What | Keyed by | Bytes | Invalidated by | Fits | At 100 ns |
|---|---|---|---|---|---|
| Yaw-free generator | label, gait program, start class | transient 263 ticks × (multiplier, acceleration) × 8 B × 2 ≈ 4 KB plus the cycle; × 3 gaits × 10² labels ≈ 1.3 MB | a law change | L2/L3 | store |
| A yaw's kernel | the above + aim | ~4.4 KB; × 65,536 ≈ 290 MB a label | a law change | no | derive (~3 µs); a few hundred hot |
| Today's bundle | label, start's motor key, yaw pair | ~1.7 MB | — | 56 fit in L3 | delete |
| Stop on B as families | label, gait, phase, face class; aim as the family coordinate | 24 faces × 12 phases × 3 words × ≤ 4 pieces × 32 B ≈ 110 KB a label; ~11 MB for 10² labels | a law change | L3 | store |
| Joint pattern sets | pattern under kit class | ≤ 12 pieces × 32 B ≈ 400 B; 10⁴–10⁶ patterns (representation.md: 10⁶ in 10k × 10k) → 4–400 MB per kit class | a law change only (content-keyed: an edit changes which pattern a place has, not the pattern) | hot part L3, rest RAM | store (ms to learn) |
| Place → pattern index | joint | 4–8 B × J, J = 10⁶–10⁷ (one per 10–100 m² of natural ground, an input) → 4–80 MB | an edit in the footprint | RAM | store |
| Field per shared goal (T-to-go on joints) | goal | 10³–10⁵ joints × 4 B = 4–400 KB | edits in footprints, local repair | L2/L3 | store for goals many bots share (S19); derive for one-off clicks |
| Field over clear zones | — | — | — | — | never stored: the generator read |
| A bot's hot state | bot | 24 B (scales.md 4.7) → 240 KB at 10⁴ | each event | L2 | store |

**Edits** (10–10³ a second): a corridor is about 3 × 10³ of the map's 10⁸ columns, so an edit falls in 3×10⁻⁵ × N tubes (0.3 at 10⁴): 3–300 re-solves a second, ≤ 2% of a core even at 57 µs. Near sites an edit can change a joint's pattern; at 1–10% unseen, 10³ edits a second cost 0.03–1.2 cores of learning. That is the real cost of edits, and 100 ns does not touch it.

### 2.6 A kit change: elytra fed by a farm

- **The cone changes everywhere**, 4–5× the legs (1.5–1.75 against 0.356 blocks a tick). By the pace layers (scales.md 2), a kit change swaps a pointer to another kit class's projection; bare-legs sets, fields and generators stay valid, just no longer least.
- **Under elytra the world is mostly sky.** Joints only at launch sites, landing faces and obstacles. A field per goal is about 10³ joints × 4 B (inferred): cheap to store. The first word's bracket is a closed form (distance over elytra speed plus launch and landing): cheap to recompute. Either way µs; 100 ns changes nothing.
- **What is expensive:** the elytra law (HotSpot's `Math.cos`, `Math.sqrt`, engine.md Phase 11); learning launch and landing patterns (ms each, once); and the rockets: the cone depends on how many are in hand. If the count enters a key, every rocket spent re-keys everything. It must travel as a price (S96; S188: resources out of the key): a rocket is worth the T it saves where used, and a farm prices it through the T to restock.

## 3. Part 3: red-team of the 100 ns rethink

### Idea 1, thinking has a light cone: keep, changed

- **Deciding case: his neo jumps (S189).** A neo needs a run-up with an exact phase, so the jump 20–30 blocks ahead (55–85 ticks) decides the first word's yaw and start tick. Momentum's memory is about 268 ticks to bit identity (representation.md 10: 0.19 a cycle over 22 cycles), so any place within it can change the first word.
- **"The first word is proven"** holds when its bracket closes against every word whose place lies within momentum's memory, measured for the width the word there needs (vocabulary.md 7). So a decision's deadline is (earliest arrival at its place by the kit's cone) − (that memory for that width). A narrow word reaches far back; a wide one barely. On the million line nothing lies within 268 ticks but the first transient, so the first word is proven at the click once item 3 is fixed.
- **Every word, not only straight lines:** yes, as the bracket's closure (learning.md 0, step 2), with "within the memory horizon" as the footprint of the first word's certificate. A word beyond the horizon cannot change it, because the law contracts: a consequence of the dissipation fold, not a heuristic.
- **Goals changed again and again (S189):** the deadline queue is goal-specific and thrown away; rebuilding costs one warm solve (~1 µs at the floor). Goal-free layers (generators, labels, pattern sets) persist. What is thrown away today and must not be is the bundle for a moving start (item 2): the real cost of a retarget.
- **Elytra:** deadlines use the kit's fastest term (conservative, earlier) and are recomputed when the kit changes, an event.
- **A second path?** No, provided the deadline queue is step 6 of the one loop (allocate) and the first word always leaves through step 2's closed check, on every terrain. A planted "first word only for straight lines" shortcut must turn kill test 8 red.
- **Gate:** kill test 15 (thinking never holds the body), plus a planted change inside and outside the memory horizon: inside, the first word changes; outside, never.

### Idea 2, the answer is a field read at the bot's state: change

- **In clear zones the field is not data.** T = transient + count × period + stop, a closed form of the generator. Storing it per goal over a zone is a goal-keyed cache (the u1 judge's warning, 8e); at 100 ns its read costs as much as deriving it (2.4). Keep it as a read of the generator.
- **At joints the field is the sets** (families, points, holds) read at the arriving state: keep. Key the sets by joint pattern under kit class, not by goal; per goal store only T-to-go on joints for goals many bots share (S19).
- **Case:** the stop on B "looked up by face class" is keyed by the gait kernel's serial id (`terminal.rs`: `gait:u64` is `WordKernel.id`), so bearings one yaw index apart share nothing: 65,536× more sets than the physics asks. Make the aim the family's free coordinate. A serial id is never a content key.
- Size and invalidation: 2.5.

### Idea 3, content keys and data shaped like the read: keep

- **Cases:** the per-binade graph probed twice through SipHash under a RefCell (`events.rs:165-174`) → a dense array per map; the law hash compared on every stop lookup (`terminal.rs:69`) → once when the store opens; phases as 360-byte states → records ≤ 40 B; motor keys as heap vectors (`store.rs:17-25`) → one content hash per observed state.
- **"No branches"** is right for lookups and wrong for the law: the law's comparisons are branches and stay bit-exact (SR-71).
- **Gate:** the interleaved warm solve within 2× of back to back.

### Idea 4, binade crossings under a certificate: change; the strongest of the four

- **As proposed it fails.** Independent exact binades do not exist: the entry into binade k+1 is binade k's exit, carried in the low bits. The translation premise's red shows it (log: an entry 1 ulp apart gives an exit 2 ulp apart, "parity matters"); the binade map is not a translation.
- **Changed: bound instead of walking.** Each addition rounds by at most half an ulp whatever the parity, so the float position stays within Σ ½ ulp of the real-number recurrence. On the million line the top binade [2¹⁹, 2²⁰) has ulp 2⁻³³ ≈ 1.16×10⁻¹⁰ over about 1.33×10⁶ ticks; the lower binades add a third: about **1.0×10⁻⁴ blocks**.
- **The sliver.** T is exact unless the goal's face lies within that bound of a tick's swept edge: about 2 × 1.0×10⁻⁴ / 0.356 ≈ **6×10⁻⁴ per goal at 10⁶ blocks**. At 10⁴ blocks the bound is 28,000 × 9×10⁻¹³ ≈ 2.5×10⁻⁸, so about 10⁻⁷; at 10³, about 10⁻¹⁰. Only those walk the chain (~0.3 µs). The parity tie picks which side a value rounds to, not how far, so it does not break the bound.
- **What it gives:** T and the counts from a few divides (~0.15 µs) for 99.94% of million-block goals. **What it does not give:** the end state's bits. The contract does not need them: the client runs the law and computes them as it moves (S83's guided missile). Where a downstream piece needs them (the stop on B, the next joint), the bounded entry interval either sits inside one piece (bracket closed) or straddles an edge (walk the chain to refine): the one loop's bracket, no second path.
- **Gate:** on 10⁴ random goals at 10³–10⁶ blocks, the bound's T equals the chain's T wherever the bound claims closure; a native replay agrees on a sample; the measured sliver rate falls within the estimate.

### What would change, and whether it is a one-way door

- **engine.md, the contract's outputs:** from "a plan of words … and the predicted T" to the first word proven, a bracket on T, the queue of open decisions with deadlines, the rest of the plan as it closes, and each word's end state as an interval (bits only where a piece needs them). The anytime guarantee already allows it, and no frozen gate reads more than T and the replayed end. A two-way door, but kill test 17 and the U1 end-bits check then walk the chain in the test, not in acting.
- **The scoreboard:** T end to end unchanged; CPU per bot per trip and per decision, measured interleaved (1.1), cold bundles and new patterns counted.
- **Principles:** none of his words change. Principle 9 ("the best compute is no compute; reuse it") read with a boundary: reuse what costs more to derive than to fetch. SR-71 applied to T: T exact by proof; bits kept only where nothing corrects them.
- **One-way doors:** only the set record, through idea 2's aim as the stop family's coordinate. Red-team that together with the set record.

## 4. What to build next, layers from the bottom (S191)

1. **Measure honestly.** The interleaved warm solve as the headline; counters for full walks, hash probes, phase records touched; a timer on the seen-track route; the click path end to end through the direct channel (item 1). Gate: this report's numbers reproduce.
2. **The yaw-free generator.** Per label and gait, the shared vertical-and-flag cycle plus per-tick horizontal multiplier and acceleration; a yaw's kernel is two scalar recurrences. Keys exclude the start's resources (S188) and carry a gait's phase, not its velocity bits. Gate: bit-equal to the law's tick for all 65,536 yaws over the transient plus two cycles, on each bare-legs label of the fixture (65,536 × ~300 ticks × 1 µs ≈ 20 s). Kill: a yaw whose contact outcome differs appears as a piece of the label's family over the aim, never as a branch.
3. **Data shaped like the read.** Phase records, dense graph arrays, content keys, the law hash at store open, a bundle record ≤ 1 KB. Gate: interleaved within 2× of back to back.
4. **One chain per axis per solve,** kept as the solve's table; events derived only when looked at; the stop on B looked up for the phase the touch fixes, after the lower bounds. Gate: one walk per axis by counter; a warm solve ≤ 2 µs interleaved.
5. **The bound.** T from the real-number recurrence ± Σ ½ ulp, the chain only in the sliver; counts from the closed form, no search over the count cap. Gate: as in idea 4.
6. **The kit's cone per certified label,** closing the first word at the click; the deadline queue as step 6. Gate: kill test 15 and the planted change inside and outside the memory horizon.
7. **The stop on B as families over the aim;** the seen-track route over pieces, no allocation. Gate: his shortcut still 158 ticks, now timed, zero allocations on a seen track.
8. **Then 100 ns is a measurement.** Resident in L1/L2, the parts add to about 50–120 ns (label and yaw ~20–65, bundle ~5–30, two divides ~10, one stop probe ~10, emitting the word ~5). Reachable only with layers 2–6; impossible with the exact chain on the click path (≥ 28 dependent crossings at ~10 ns).

In parallel, outside the planner: plan.md step 4 (the bot's runner) closes item 1, the largest T item.
