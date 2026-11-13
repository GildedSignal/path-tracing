# Job 3.2d: the planner, rewritten — sets backward, the point forward, one replay

**Why.** The first planner was judged **FAIL** and an Opus red-team sent its successor's draft back for a rewrite. This is that rewrite. It is the gate to the first flight: one bot racing his 18 tracks. Everything it must not contradict was audited into `jobs/planner-premises/last.md`, with the file and line behind each constraint — **read that report first; it is the specification's own bibliography.**

**Consumer.** The first flight (HANDOFF roadmap item 2). Nothing else.

**Branch.** `D:\path-tracing-wt\sets` on `wt/sets`, cut from `7713f9d` (where `wt/planner` left the table). Paths outside your branch are in the main checkout at `C:\Users\PASCAL\Desktop\path tracing` and must be opened by absolute path.

**Read**, in this order: `AGENTS.md`; `docs/PRINCIPLES.md` goal and philosophy; `jobs/planner-premises/last.md` (the audit); `docs/learning.md` section 1; `docs/vocabulary.md` sections 3, 7, 12; `jobs/3.2b-judge/last.md` (the FAIL); `jobs/3.2f-b-joints/last.md` and `jobs/3.2e-probes/last.md`; `docs/engine.md`. The lab dossiers you will rely on are named inline below, each in `lab\results\`.

---

## Step 0 — the judge's four fixes, before any new set work

Nothing below is built until these four hold, because the old planner's numbers are not comparable until they do.

1. **The bound stops discarding.** `c = 19.902886` b/s is a **3-tick average** and is not a proof per tick, so it may **order** and must **never discard**. Either prove a per-tick horizontal bound that covers an arbitrary start velocity — and if you prove one, state the proof and its assumptions in the report — or prune **solely by exact-state dominance**. A bound without a proof that removes a candidate is the single worst failure this job can have.
2. **Thinking time counts.** T runs from the click to arrival, so every table compiled **after** the click is inside T, reach tables included. A table may be outside T only if it is built before the click **and keyed so the lookup needs no post-click work**; say in the report which tables are pre-built, what their key is, and what each costs.
3. **"Sets backward" must actually be backward.** The old code was one forward search ordered by a field over cells, which is why its comparison meant nothing. This is a separate solver: sets carried backward along the chain, the point chosen forward, joints slid inside their sets.
4. **Premises are checked against recorded read footprints at ticks.** Never a caller-supplied epoch, never a rescan. A premise is "these cells held these contents at these ticks"; invalidation touches only the sets whose footprint reads what changed.

## Step 1 — what a set is

A set is **arriving states**, never cells. Its members are bounded by the law's own comparisons: the supporting surface face, the class's yes-or-no answers, an interval of velocity, and the ticks at which it is valid. **No cell carries a T of its own, there is no grid of ours, and no number in a set is chosen by you.** The cell stays nature's address for block reads, footprints and premises, and is not a node.

- **Checking a set.** `monotone-tick` is reproduced: the law's tick is corner-monotone on **255,136 of 255,150** one-tick member-ticks, so a set may be checked at its corners. The **14 exceptions are not noise and you must handle them by name**: at `movement.rs:802-803` an `acos` ratio rounds above 1 and yields `NaN` on `minor_horizontal_collision`, flipping `sprinting` at `movement.rs:217`. Escapes over k ticks peak at **0.427% at k = 3**. You may **not** clamp that ratio — the law must equal Mojang's, and a clamp of ours is a patch. So a corner check **orders and bounds only**; it never decides, and the final replay decides. Dossier: `lab\results\local-20261113-1727-1-0-monotone-tick\`.
- **Splitting.** A set splits only where the law makes a comparison, never at a spacing of yours.
- **Dropping.** A set is never dropped without proof.

## Step 2 — takeoffs are a preimage, not a shifted surface

The takeoff set of a word is **the preimage of the next set under that word's run**. The convenient reading — "the landing surface shifted back by the arc" — holds **only for a collision-free arc at a fixed velocity**, and you must not use it anywhere else.

Two reproduced facts make this computable, and you should lean on both: free-air arcs have **0 of 12,759** world-dependent cuts with **56,575 of 56,575** exact floor and landing-tick predictions at about 700 ns per arc (`local-20261112-1251-1-0-arc-certificate`), and jump takeoffs are **108 of 108 bit-equal under translation** as a function of `(v_start, program, k, f)` (`local-20261113-0958-1-0-generative-function`). That second one is why a takeoff can be computed rather than searched.

## Step 3 — no gaits, and the run-up is the bottleneck

**Do not rely on gaits.** None is reached on his tracks: a stone sprint-jump gait needs **265 ticks, 22 cycles** from rest, and on the 18 parkour tracks a corridor degenerates to **1.07 cells per run**. A run-up is the per-tick rule applied **k times**, nothing more.

Treat this as the job's central difficulty, because two independent measurements agree on it. `backward-tools` (reproduced) certifies **4,722 of 5,587** standable goals from mirrored reach tables in one replay, and of its false yeses **89% are run-up mismatch** against 2.4% from the flight half. 3.2f-b found that all **42** stalled-joint replays on `fences` and `fencemaze` depart on **tick 1**, because the stored hop begins at a pre-accelerated contact that a centre-at-rest replay cannot reproduce. So a set must carry an **explicit entry-velocity interval**, and reaching it is a k-tick run-up to be solved, not assumed.

## Step 4 — switches, aim, and where the search opens

- **A switch fires when the exact state enters the next word's takeoff set.** Between switches the running word continues and nothing is chosen.
- **The planner does not branch at a body-block change** (vocabulary 12.4, S101), while segments, certificates and the proposal table keep that cut.
- **Aim is solved from the sets it joins** and then refined. It is **not** 64 replays per decision; the old 1,536 proposals per decision are what made the first planner see two decisions deep.
- **Look the proposal table up by class alone.** A multi-mask lookup measured 22.1 µs against 100 ns single-mask, 49× its floor.
- **Exact-state search opens only where the exact state fails to enter the next set.** `Order::Admissible` **stays**, as the offline judge for the bracket — the earlier draft deleted it and the red-team overruled that.
- **Every hop's full four-dimensional worldtube is checked** against real block collision shapes at ticks, walked hops included.

## Step 5 — what must not be assumed away

The record leaves these open, and a spec that quietly closes one is wrong:

1. Whether a valid run-up to the stored contact is reachable from the search's own states on `fences` and `fencemaze` is **not determined**. You may build sets with explicit entry-velocity intervals and k-tick run-ups; you may not assume the joint is reachable or that it is not.
2. **`redfakemaze` has no chain to A at all** — 0 anchored against 13,549 unanchored expansions. This is later than, and corrects, the text in `learning.md` and HANDOFF that calls it a stall at a claimed joint. Do not plan around a joint it never reaches.
3. Whether momentum alone explains `grassjumpeasy`'s 12 tools is unsettled; carry full arriving state, never rest at a cell centre.
4. The two un-evented world reads from `generative-function` — a wall contact ending on one axis while the other stays collided, and sneak `maybeBackOffFromEdge` at `movement.rs:577` — are the only misses in an otherwise 98.01% bit-exact per-tick rule. Sets must not assume they are evented.

## Step 6 — the measurement that decides

- **Rerun the 3.2b baseline with step 0's fixes, at cap 1 and cap 20.** Without it there is no comparison and the job proves nothing.
- **Freeze this kill table before the first run and do not edit it afterwards.** Expected to stay failures, for a missing word: `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `ametwoodbox`, `mud`. Expected to move: `fences`, `redfakemaze`, `fencemaze`, `amertystcoral`. **If none of the four moves, this spec is wrong and says so** — report it as a kill, not as a tuning problem.
- Report per track: whether a sentence arrives, its T against his, expansions, and the thinking time inside T.

**Must not.** Add a tolerance, an epsilon, a retry, a per-block branch, a jump catalogue, or a clamp — including on the `acos` ratio. Write new search or geometry code per tool (a capability enters as law and toolkit data; if one seems to need its own search, stop and say so). Use MCTS or a field over a grid. Touch `bot/`, `docs/`, the gates, the fixtures, or the law's arithmetic. Edit the kill table after a run. Commit, push or merge.

**Done means.** The baseline rerun at both caps; the new solver measured on all 18 tracks against that baseline; `cargo test --offline` from `engine/` passing; and a final message with the per-track table, which of the four expected movers moved, what step 0's bound became (proved, or ordering-only), which tables are pre-built and their keys, the net lines, and every constraint from the audit you could not satisfy and why. **A spec that fails its own kill table honestly is a result; one that passes by editing the table is a failure.**
