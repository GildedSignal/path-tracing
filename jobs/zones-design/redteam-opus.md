# Red-team of the zones design (Opus 5.5, native, fresh context, 15 November)

Design: `jobs/zones-design/design.md` (145 lines). Authority: S120, S130–S134. Read against engine.md, representation.md, scales.md, vocabulary.md, learning.md, PRINCIPLES (ES), the inventory, the plain-path verdict, the 3.2d-3 spec and its red-team, `plain.rs`, `movement.rs`, `math.rs`, `region.rs`, the bot's Recorder, Tracks and packet mixin. Nothing was run.

## 1. Verdict: FREEZE AFTER FIXES, for R0, R1, R2 and R4 only

The direction serves S131 ("the terrains complexity should be a mirror of compute's complexity"). The parts that amend 3.2d-3 (resident planner, role codec, generator store, executor contract) can freeze once findings 1–3, 8 and 12 are written in.

**R3 (zone tree), R5 (portals, field, segments) and R6 (cones) do not freeze.** Each has an untested premise or a rule of ours (findings 4–7, 9, 10). Re-derive them after tests T5–T9 in section 4. This is how the last red-team cut section 5 of 3.2d-3.

## 2. Findings, by severity

**1. Blocker: the bare-kit million-block line cannot be "q cycles plus a food guard"** (§0.1, §4.2 line 111, §6 Q1).
- *Why.* The bare kit holds no food, so "eating must be a word in the cycle" is impossible.
  - One sprint-jump cycle (12 ticks, ~4.28 blocks) costs 0.2 for the jump (`movement.rs:381`) plus 0.1 × cm on the one ground tick (`movement.rs:1115-1118`): about 0.24 exhaustion.
  - From spawn bars (20 food, 5 saturation), food reaches 6 after about 19 × 4 / 0.24 ≈ 320 cycles, so **≈1.4k blocks**.
  - After that, `update_sprinting` refuses (`movement.rs:219`), and the true answer is two words: sprint-jump, then the best non-sprint gait for ~998.6k blocks. That is T ≈ 4.6M ticks, about 64 h of game time, or about 190 Minecraft nights of hostile spawns.
- *The switch tick is server-driven.* The client reads `food_level` from a packet, and the exhaustion is the server's `checkMovementStatistics` (`movement.rs:1103`). That is engine.md's server-driven class, so the 3.2d-3 outer gate `|T_live − N_arithmetic| ≤ 1` cannot hold at 1M.
- *Peaceful changes everything.* On peaceful the guard is void (`plain.rs:339`) and nothing spawns.
- *Fix.* The gate names its difficulty and its mobs.
  - The thinking gate runs on peaceful.
  - The non-peaceful line is a two-word sentence: the food crossing is an event with a re-anchor, and surprises (mobs) are reported as growing with distance. ES never promised O(1) total compute, only compute ∝ surprises.
  - The live run needs `/tick` acceleration and a pregenerated strip (finding 11).

**2. Blocker: one yaw misses B at a million blocks** (§4.2; 3.2d-3 §2 line 30).
- *Why.* Aim goes through the 65,536-entry table (`math.rs:2-11`), and jump boost and input vector share the index. The half-bin error is 4.79×10⁻⁵ rad, so the lateral miss at 1M is up to **47.9 blocks**.
- *Where it bites.* Touching a 1-block detector needs about 0.8 blocks of lateral error, so a generic bearing fails beyond **~17k blocks**. At 800 blocks the miss is 0.04 blocks, which is why it never showed. Axis-aligned lines are exact (index 49152: sin = −1, cos = 0).
- *Consequence.* Past that distance, plain.rs ends in `End::Pass` ("neither touches", line 305) and 3.2d-3 drops the whole route into the hard search.
- *Fix.* Either the 1M gate is axis-aligned, or the closed form becomes a two-yaw lattice problem: integers q₁, q₂ with q₁·D_i + q₂·D_{i+1} landing in B's box. That is O(1), but the plain path has five decisions, not three. State which.

**3. High: the Fermi numbers do not add up** (§0.1, §4.2 line 112).
- *A big tree costs more than 10 µs.* "~10⁴ tree nodes … ~1–10 µs" would need ≤1 ns per node. A node visit costs ≥ one cache miss (5–80 ns), so 10⁴ nodes is 0.05–0.8 ms. The 10 µs holds only when one node covers the line.
- *The stripped tick is unmeasured.* "0.1 µs per stripped tick" is engine.md's floor, not a measurement: the measured cost is 0.6–0.95 µs (`engine.md:11`).
- *The transient dominates.* It runs about **274 ticks** before bit-exact recurrence: the judge's fold mutant first fails at tick 275 (`verdict.md:15`), and the velocity contracts only about 5× per cycle. Each click therefore pays 0.2–0.3 ms of tick-by-tick generator, plus a boxed `HashMap` key per tick (`plain.rs:186,236`).
- *One division is really one per event.* The fold needs a division per known event (finding U1): about 20 binades from the origin to 2²⁰, the food crossing and a yaw switch. That is O(log D).
- *Natural terrain.* Under bare legs, class boundaries occur every 2–5 blocks, so 10⁵–10⁶ leaves per 1M line. Each landing at a new height breaks the cycle, so thinking is O(landings): ~2.3×10⁵ × ~12 µs ≈ 3 s.
- *Fix.*
  - State one floor per case: one zone; k events; varied terrain.
  - Memoise transients in the generator store under the recurrence key: the state minus position, which recurs from rest.
  - Varied terrain gets a **rolling horizon**: commit the certified prefix, plan ahead while the body moves (learning.md §1, "thinking done while the body carries out … costs no T"). Gate it by first-control tick plus planning rate ≤ travel rate.
  - Note that µs and ms thinking give the same integer T when both fit before the first eligible tick. µs is bought for the swarm budget (representation.md: bot decisions 0.1 core at 10⁴ bots), which is a reason to state, not to hide.

**4. High: the light-cone ellipse explodes without an opening angle** (§4.2 line 110).
- *Why.* The ellipse's half-width is (D/2)√(r²−1) for a bracket ratio r = UB/LB. At D = 1M and r = 1.1 that is 229k blocks, so the query covers ~10¹¹ blocks². Before a UB exists the ellipse is unbounded.
- *Fix.* Query with scales.md 4.3's opening angle: cells grow with distance from the current route, with a loss of ~θ²/4. On a plain line LB = UB, and the ellipse is the segment.

**5. High: the zone tree is D1 again, once per kit** (§3.2; Q2).
- *Why.* It sits beside the snapshot, and classes are per kit (S134), so every tool adds a tree. Feeding it client chunks over the pipe makes a fifth copy.
- *What already exists.*
  - `models.tsv` (518 rows) is already the legs-only projection: it has no hardness, tool or drop, so silk touch can't be expressed there.
  - `region.rs:34` already keeps Mojang's `Section { palette, indices: None }` for single-value sections. That is "an ocean is one node" in Mojang's own format, and `snapshot.rs` then flattens it into a dense `u16` box.
- *Fix.*
  - Label leaves at block-state grain and keep one tree.
  - Each kit is a lookup table from fine id to class (scales.md:15: "an inventory change swaps a pointer").
  - The tree **replaces** `WorldSnapshot`: the law's `World` reads palettes through it.
  - The near field arrives as block and section deltas through the existing hooks (`ClientPacketListenerMixin.java:118,123`), never as re-sent chunks.

**6. High: emergence hides a tie rule** (§3.2 line 84).
- *Why.* "Merge neighbours whose descriptions are equal; split where the description changes" is not unique. Merge order and the choice between several change coordinates are rules of ours, and a minimum 3D box partition is NP-hard, so any greedy order is a chosen number in disguise (S132).
- *Fix.* A **hash-consed octree on the binary address**.
  - The arity is 2³, from the dimension.
  - Node sizes are the coordinate's own bits, like binades.
  - It is canonical, deduplicated as in Hashlife (representation.md:78), and an edit costs O(depth).
  - Zones are the maximal uniform nodes. Mojang's 16³ section is a storage address inside it, not a zone size.

**7. High: portals in position × velocity × phase are neither finite nor certified as written** (§3.4).
- *Why.* Entry states are continuous, and "width w" is undefined: vocabulary.md §7 has delay tolerance, and clearance is unbuilt (`segment.rs:117`). Certifying a band means either interval arithmetic over the law, which is a second law, or sampling, which is no proof.
- *Fix (exact, no epsilon).*
  - A portal is (gait, phase, an interval of the translation offset s).
  - Along one word with fixed inputs, float + and × by positive constants and the min/max clamps are monotone in s. So each law comparison flips at most once, and the intervals are finitely many.
  - Find the endpoints by float bisection (≤64 steps) and replay both.
  - An off-gait entry stays a proposal plus a replay (vocabulary.md §2).
  - *Premise to test (T7).* The axis order (`|dx| < |dz|`) and the step-up choice compare velocities, not s.

**8. High: integer T makes "decide only on ridges" vacuous** (§2 ridges; §3.5 line 105; Q3).
- *Why.* Ties are fat: at 800 blocks every yaw index within ~±10 of the bearing reaches the detector in the same or the next tick (0.8-block tolerance / 800 blocks ≈ 10⁻³ rad ≈ 10 bins). A tie has no value in T, the one currency, and descent on integer terraces is undefined. "Breaking ties by the wider tube" needs clearance, which is unbuilt.
- *Fix.*
  - Keep 3.2d-3's definition (§2 line 31): a decision is a committed word, argument or stop, counted at switches.
  - Ties are broken by a fixed canonical order (lowest index, FIFO) at zero cost.
  - Width enters as a second key only where it goes to zero (SR-71).
  - Saddles (S132) are switches where two words' T differ by ≥1 tick across the state.

**9. High: the promoted field has no domain** (§3.5).
- *Why.* Per cell, it brings back "a block with its own T" (S91; `learning.md:14,16`), the field that defeated fences.
- *Fix.* The field is the min-plus closure of the portal tables toward B (CRP's overlay): values on portal bands, closed form inside plain zones. It is one object with §3.4 (U5).

**10. High: a second search engine** (§4.3 line 113, §4.4 lines 115–119; AGENTS "No new search code").
- *Why.* "A GPU wavefront", "the wavefront in that zone" and "a backward producer search" are all new. 3.2d-3 §4 allows only F, the existing forward planner scoped per span, and sent the backward solver to its own door (line 134). The GPU float law is advisory (`engine.md:119`).
- *Fix.* Use F on CPU threads only. The GPU waits for a measured Knudsen crossover (`representation.md:139-142`).

**11. Medium: the chunk wall is in the law, and it is 4D** (Q1b).
- *Why.* In a chunk the client hasn't received, vy becomes −0.1 (`movement.rs:444-450`), and fluids skip (`875`). So `loaded(p, t)` is a server-driven premise. The disk's far field is not what the client holds at the tick the body arrives.
- *The fixture wall.* At 1M, a ±10-chunk strip is ~1.3M chunks: ~7 h and 7–13 GB at scales.md 4.2's rate, on a C: drive with 10 GB free. Reading ~2,000 region files makes the tree O(D) to build.
- *Fix.*
  - The 1M fixture is a superflat world without structures (Classic Flat has villages), and its far field is the generator (scales.md 4.3: "the generator is the far field"), O(1). Saved chunks are scars.
  - `Loaded` is a premise, and chunk arrivals are events.
  - Tree construction is pre-click with provenance (3.2d-3 §5 cold rule), stated as such.

**12. Medium: per-tick work creeps back in.**
- *The cone test.* §4.6, re-evaluated each tick, is per-tick. Schedule each player's check at the tick their cone can first reach the corridor, on the timing wheel (`engine.md:63`, `scales.md:109`).
- *Counted food debits.* Over 2.8M ticks (3.2d-3 line 33) they are O(D). Use U2.
- *"Leaves its tube".* §4.5 needs a width that doesn't exist. Compare exact bits at events, minus the server-driven fields, with `deviation.rs`.
- *"Independent given portal states".* §4.3: those states aren't known before the previous span runs. Either run speculatively (3.2d-3 §4), or solve per band at cost × band size. Choose one.

**13. Medium: editing is misstated** (§2 line 73).
- *Why.* `PRINCIPLES.md:317` marks it **Settled (S32)**: edits are allowed with restoration. An edit is a toll **plus a debt** whose T depends on the later route, because the undo must be reachable.
- *Fix.* Edits whose undo closes inside the gait period (place ahead, break behind within 4.5) form a medium with its own c, the "floor you carry". The others are ledger debts (scales.md 4.9).

**14. Medium: the hard-parkour extreme has no path through R0–R6.**
- *Tools.* Item use, hotbar, break and place are absent from `Input`, `PlayerState`, the programs and the law (inventory rows 8, 13, 27), so gate 2 has no consumer before "Later".
- *Nether portal.* Opening it edits **both sheets**. The game builds the exit portal and its platform, and restoring that means mining obsidian.
  - The exit search radius makes the chart many-to-one: the nearest existing portal wins, not a clean 8:1.
  - There is a portal wait, and the Nether sheet is unread (inventory row 11).
- *Pearls.* A pearl costs:
  - the item, whose restoration is priced;
  - 5 HP, which costs 30 exhaustion (7.5 food) to regenerate;
  - a 20-tick cooldown.

  It also lands with server-RNG spread (`engine.md:163-166`), so the wormhole's exit is a distribution, not a certificate. The thrower's momentum adds in (S122).
- *Two more things the design leaves out.* Entities as tools in the world ("a boat on the ocean", S131) are missing from presence, and so is the wiggle (S131).
- *Fix.* List them as R7+ with these tolls, and freeze the 1M gate and the parkour gate separately.

**15. Low.**
- The judge's finding 4 also asks for the no-return proof (`verdict.md:9`), so R0 must carry it.
- The design's cost table sums to ~24 ms against 29.1 ms measured, and the difference is unattributed.

## 3. Unifications missed

- **U1. Every wall is an event under the existing rule.** vocabulary.md §4 says "any discrete field of the state changes". The exponent of x, `food_level`, sprint-allowed, `loaded`, a zone boundary and a yaw switch are all discrete fields. So the fold is closed form between the members of one merged list of events known in advance, O(#events), with no special case.
- **U2. Food is a cycle of cycles.** Per-phase exhaustion amounts are constants (cm rounding, `movement.rs:1116`), so (phase, exhaustion bits) recurs after some number of 4.0 wraps. The store keeps the super-period's debits, which is ES's cycle memo at two scales.
- **U3. One intern table.** A zone's label (support class, medium class) **is** the generator-store key of §3.3, minus the kit: §3.1–3.3 share one intern table, and a leaf looks up its word directly.
- **U4. The class set is a speed.** A node's class set gives c_max, its refractive index: the monopole, scales.md 4.3's hull, and the LB the far field needs.
- **U5. Portals, field and width are one thing.** Portals plus the field are one min-plus overlay. Band width = vocabulary §7's width = S132's hardness certificate = where SR-71 spends precision = learning.md's "where a word's width goes to zero".
- **U6. One allocator.** The cone test is certificate slack, and §4.4's three cases are scales.md §2's single rule: compute goes to the bracket whose narrowing changes the soonest decision.
- **U7. The tree is the E1 planes, reduced upward.** 3.2d-3's E1 section bitplanes are the tree's leaf level, and the tree is their OR-reduction upward. Build one, not both.
- **U8. One partition.** Kit classes are projections of one fine partition (pace layers).
- **U9. One comparator.** The executor's event check is the deviation stream's comparator, `deviation.rs`.

## 4. What must be tested first (one measurement per load-bearing premise)

| # | Premise | Command or measurement |
|---|---|---|
| T1 | The transient is ~274 ticks and memoisable | `cargo test --offline -p plan --test plain straight -- --nocapture`: read "cycle … from tick f" (`plain.rs:399`) on straight100/300/800; check that the recurrence key from rest is identical across clicks |
| T2 | The per-binade closed form equals the tick fold | a unit test on the `Plain` context: fold from x = 2¹⁹−100 to 2²¹+100 tick by tick against the per-binade closed form, bit-compared |
| T3 | The yaw miss | the arithmetic above, plus `yaw_toward` for B = (10⁶, y, 3.3×10⁵): the lateral miss of the folded line |
| T4 | Food crossing and its timing | (a) fold the exhaustion from (20, 5, 0) and report the tick food reaches 6 and the super-period; (b) live, one ~1.5k-block straight on Normal: the sprint-stop tick against the server food packet tick in the recorder rows |
| T5 | The loaded frontier leads the body | `LandingClientTrace` chunk rows against position on a long straight at the rig's tick rate: the minimum lead, and any tick where `loaded(below)` is false |
| T6 | Palettes carry presence | along straight800 and a natural fixture, the share of sections that are single-value or whose palette ⊆ plain classes (`region.rs:34`) |
| T7 | Monotone in s | sweep float offsets of a sprint-jump approach over a stone→ice edge and a 1-block gap: are the exit tick and exit bits monotone; how many breakpoints |
| T8 | Plateaus are fat | straight800: T for each yaw index within ±50 of the bearing; count those at T_min |
| T9 | Tree size | frozen-seed natural fixture: class boundaries per 1,000 blocks and octree leaves, bare kit against a pickaxe kit |
| T10 | 1M fixture cost | superflat without structures: generation rate and bytes per 1,000 chunks, extrapolated to the strip |
| T11 | Jar facts | `javap` on 26.2: the portal wait time and the exit search radii (PortalForcer); the pearl's throw inaccuracy; LocalPlayer's tick in an unloaded chunk |
