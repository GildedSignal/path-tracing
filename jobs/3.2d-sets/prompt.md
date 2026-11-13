# Job 3.2d: the planner, rewritten — sets backward, the point forward, one replay

**Version 2, after a red-team verdict of REWRITE.** The first draft of this spec was attacked by two families; one returned 16 defects and no clean axis. Every change below is traceable to one of them. Its predecessor, job 3.2b, was judged **FAIL**, so this is the third attempt and the bar is correspondingly high.

**Why.** This is the gate to the first flight: one bot racing his 18 tracks. **Consumer:** that flight, and nothing else.

**Branch.** `D:\path-tracing-wt\sets` on `wt/sets`, cut from `7713f9d`. Anything outside your branch is in the main checkout at `C:\Users\PASCAL\Desktop\path tracing` and must be opened by absolute path.

**Read**, in this order: `AGENTS.md`; `docs/PRINCIPLES.md` goal and philosophy; **`jobs/planner-premises/last.md`** — the audited list of 19 constraints with the file and line behind each, which is this spec's bibliography; `jobs/3.2d-redteam-terra/last.md` — the 16 defects, so you can see what each rule below is defending against; `jobs/3.2b-judge/last.md`; `docs/learning.md` section 1; `docs/vocabulary.md` sections 3, 7, 12; `jobs/3.2f-b-joints/last.md` and `jobs/3.2e-probes/last.md`; `docs/engine.md`.

---

## The spine: three answers, never two

Every test this planner makes about a set — "is this state in it", "can this set be discarded", "does this bound exclude this candidate" — returns one of exactly three answers: **proven yes**, **proven no**, or **unknown**. This is the rule the rest of the spec hangs on, so build it first and make it impossible to bypass in the types.

- **Only a proven answer may act.** An `unknown` may **order**, and may do nothing else. It may never discard a candidate, never certify membership, never suppress a fallback, and never end a search.
- **`unknown` is recorded, not resolved.** When a budget runs out, or a proof is unavailable, the result is a **recorded refusal** carrying what was being asked and why it stopped. It is never silently turned into a yes or a no, and the run makes no completeness claim over any part of the space where a refusal was recorded.
- **What a finite budget means, so you do not have to invent it.** A search that exhausts its budget with alternatives still unresolved **may return a certified sentence** — the planner is a semi-decision procedure and is expected to answer (`learning.md` section 2, step 3). It must then report that sentence as **certified but not claimed optimal**, and list the unresolved alternatives it did not settle. What it may never do is report such a run as complete, as a proof of optimality, or as evidence that no better route exists. Returning nothing when a sentence was certified is also wrong.
- This exists because the final replay is **not** a safety net for this class of error. A replay rejects a bad candidate, but a route wrongly excluded by an unsound bound or a false membership test is never generated, so the replay never sees it and the loss is silent and permanent.

## Step 0 — the judge's four fixes

Nothing else is built until these hold; before them the old numbers are not comparable.

1. **No bound discards without a proof.** `c = 19.902886` b/s is a **3-tick average**, not a per-tick bound, so it may order and must never exclude. Either prove a per-tick horizontal bound covering an arbitrary start velocity — stating the proof and its assumptions in the report — or prune **only** by exact-state dominance. This applies to *every* bound the planner uses to exclude or to certify, not only to `c`.
2. **Thinking time counts, and is clocked, not argued.** Record the actual wall clock from click to restored arrival, the conversion you use from wall time to server ticks (state the conversion; a tick is 50 ms and rounding is yours to declare), and every post-click computation. For each pre-built artefact record its **provenance**: what built it, when, and from what. **What is outside T is the artefact's construction, never its use** — reading a key and performing a lookup are work, they happen after the click, and their measured cost is counted inside T like any other. A table whose provenance is not recorded is treated as built after the click, in full.
3. **"Sets backward" must be backward.** The old code was one forward search ordered by a field over cells, which is why its comparison meant nothing. Sets are carried backward along the chain, the point is chosen forward, joints slide inside their sets.
4. **Premises are checked against recorded read footprints at ticks** — never a supplied epoch, never a rescan. The comparison is: for each cell the footprint records as read at tick t, the contents now at that cell at that tick against the contents recorded then; a difference invalidates, identical contents do not, and a cell absent from the footprint is never consulted. Prove it with an **edit test**: one edit to a cell an affected set's footprint read (it must invalidate), and one unrelated edit (it must not), both at ticks, including the two un-evented reads named in Step 4.

## Step 1 — what a set is

A set is **arriving states**, never cells. The cell stays nature's address for block reads, footprints and premises, and is not a node. No cell carries a T of its own, and there is no grid of ours.

**On numbers in a set.** A set's bounds are *derived*: each one traces to a comparison the law makes, and you should record that trace. What is forbidden is a number **you** choose — a tolerance, an epsilon, a grid spacing, a bucket width, a threshold, a fixed subdivision depth. A derived interval endpoint is not one of those; a rounded or widened one is.

- **It must carry everything the law reads.** Face and class booleans and a velocity interval are **not** sufficient: position within the face, airborne status, and any other law-relevant field are part of the state. State explicitly which fields the set represents exactly, which it bounds, and what correlations between them it keeps or conservatively discards. A field you neither represent nor conservatively over-approximate is a soundness bug.
- **Corner checks order; they do not decide.** `monotone-tick` is reproduced: the tick is corner-monotone on **255,136 of 255,150** one-tick member-ticks. The **14 exceptions have a named cause** — at `movement.rs:802-803` an `acos` ratio rounds above 1 and yields `NaN` on `minor_horizontal_collision`, flipping `sprinting` at `movement.rs:217` — and escapes over k ticks peak at **0.427% at k = 3**. You may **not** clamp that ratio: the law must equal Mojang's. So a corner-derived enclosure is `unknown`, not proven, wherever a non-monotone case is not separately proven conservative. Treat it accordingly under the spine.
- **Splitting** happens only where the law makes a comparison. **Dropping** needs a proof, and where no proof is available the set is recorded as unresolved and kept.

## Step 2 — takeoffs

The takeoff set of a word is the **preimage of the next set under that word's run**. Two measurements bear on this and neither is a general result, so do not treat them as one:

- Free-air arcs: **0 of 12,759** world-dependent cuts, **56,575 of 56,575** exact floor and landing-tick predictions, ~700 ns per arc (`local-20261112-1251-1-0-arc-certificate`).
- Jump takeoffs: **108 of 108** bit-equal under translation as a function of `(v_start, program, k, f)` (`local-20261113-0958-1-0-generative-function`).

The convenient reading — "the landing surface shifted back by the arc" — is permitted **only** for a collision-free arc at fixed velocity, which is the tested domain. Outside it, produce a sound preimage or record `unknown`. **How** you compute a preimage is yours to choose: this spec requires soundness and a stated domain, not a mechanism.

## Step 3 — the run-up, stated as what was measured

Do **not** rely on gaits: a stone sprint-jump gait needs **265 ticks, 22 cycles** from rest, and corridors degenerate to **1.07 cells per run** on these tracks. A run-up is the per-tick rule applied k times.

Two observations, reported as observations: `backward-tools` certifies **4,722 of 5,587** standable goals from mirrored tables, and **89%** of its false yeses are run-up mismatch against 2.4% from the flight half; and all **42** stalled-joint replays on `fences` and `fencemaze` depart on **tick 1**, the stored hop beginning at a pre-accelerated contact. **The measurements explicitly do not establish the cause of the stalls, nor whether a valid run-up is reachable from the search's own states.** Competing causes stay open. Design sets to carry an explicit entry-velocity interval because the state demands it, not because the stall has been diagnosed.

## Step 4 — switches, aim, lookup, fallback

- **A switch fires when the exact state is a proven member of the next word's takeoff set.** `unknown` membership does not fire a switch, does not suppress fallback, and does not discard the alternatives.
- **The planner does not branch at a body-block change** (vocabulary 12.4, S101); segments, certificates and the proposal table keep that cut.
- **Aim is solved from the sets it joins**, then refined — not 64 replays per decision.
- **Proposal lookup.** A multi-mask lookup measured 22.1 µs against 100 ns single-mask, 49× its floor, so looking up by class alone is the speed objective. It is **not** assumed sound: show that every context sharing a class has the same applicable proposals, or record the constraint as unresolved and keep the slower lookup.
- **Exact-state search opens wherever membership is not proven**, which includes `unknown`. `Order::Admissible` **stays** as the offline judge for the bracket.
- **Every hop's full four-dimensional worldtube** is checked against real block collision shapes at ticks, walked hops included.
- **Reads are bounds-checked.** A lookup outside the snapshot's halo is a recorded refusal, or names the supported world source it came from. Offline execution does not make this safe by itself.
- Two world reads are **not** evented and sets must not assume they are: a wall contact ending on one axis while the other stays collided, and sneak `maybeBackOffFromEdge` at `movement.rs:577`.

## Step 5 — what every returned sentence must satisfy

**Every** sentence this planner returns — from the main path, from a fallback, from an improvement pass, from anywhere — is replayed end to end by the law **from the exact start state**, and must show arrival, survival, **and restoration of the world** (3.2b's judge found restoration was not established). A sentence that fails any of these is refused, not returned and not reported as a success.

## Step 6 — the measurement that decides

- **The baseline must be reproducible.** The baseline is `Order::Admissible` as 3.2b left it, plus **only** Step 0 fixes 1 and 2 — the bound stops discarding, and thinking time is clocked. Fixes 3 and 4 define the new method and do **not** apply to it. Run both at **cap 1 and cap 20**, where a cap counts expansions and termination is cap exhaustion or a certified sentence, whichever comes first; state it precisely if you must refine that. Match the **accounting** of precomputation, not the artefacts: the two methods legitimately need different ones, so list each method's artefacts with its build cost and say which are inside T. Same hardware, same inputs, same timing treatment.
- **A held-out set, frozen before you implement anything, and not chosen by you.** Write the variation set out **first**, as a committed file, before any solver code: changed start states and terrain shifts derived from a rule stated in that file. Then do not look at its results until the end, and never tune against it.
  - **Instance data is allowed; answers and tuning are not.** The solver of course receives the track's geometry and its start and goal — that is the problem. What must not exist anywhere is a stored answer, a per-track parameter, a table keyed by track identity, or anything selected by trying the tracks. If a pre-built artefact was built from the 18 tracks, say so: that is the thing most likely to be memorisation in disguise.
- **The kill table, frozen before the first run and never edited after it.** Expected to stay failures *for a missing word*: `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `ametwoodbox`, `mud`. Expected to move: `fences`, `redfakemaze`, `fencemaze`, `amertystcoral`.
  - **"Moves" means**: a newly certified end-to-end arrival under matched conditions; or, failing that, a replayed witness that reaches a **named state the baseline provably never reached** — the joint it was stalled at, by name — under the same conditions. A changed expansion count, a different budget, an uncertified intermediate, or a replay that reaches somewhere already reachable is **not** movement. Report the four independently.
  - **A failure must be the right kind, and shown to be.** For each expected failure, report the planner's actual recorded refusal, and show the missing capability **caused** it: name the point where no word applies, and show that the run did not instead stop on budget exhaustion, a timeout, or an unrelated defect — if it did, say so and the table is not confirmed for that track.
  - **If none of the four movers moves, this spec is wrong.** Report that as a kill, not as a tuning problem.
- **The scoreboard.** Per track: whether a sentence arrives; T against his; the **bracket** — the proven lower bound and the best T from the independent search — or an explicit mark for each component that is unavailable and why; a **distribution over repeated runs**, never an average alone — **5 runs per track**, with failures and timeouts reported as their own counts beside the distribution and never dropped from it or folded in as a large value; expansions; the budget; the refusal counts; and the clocked thinking time inside T.
- **Write a run record** (AGENTS) under `D:\path-tracing-evidence\`: inputs for exact replay, each invariant's worst margin, and every refusal.

**On what this spec dictates, and why.** A red-team called several rules below mechanism rather than requirement, which would normally be a fair objection. They are kept because they are **not this spec's choices**: "sets backward, the point forward" is the judge's third FAIL fix and his S91/S94/X16; no grid field and no MCTS are AGENTS' "No new search code" and audit constraint 19; "never a rescan" is the judge's fourth FAIL fix; splitting only at a law comparison is the red-team's own earlier requirement. Inside those, the mechanism is yours — how you compute a preimage, order a frontier, or represent a set is not prescribed, and a better representation found within them is welcome. If one of these settled rules is what blocks you, **stop and say so**: that is a re-derivation for the integrator and Julien, not something to work around.

**Must not.** Add a tolerance, an epsilon, a retry, a per-block branch, a jump catalogue, or a clamp — including on the `acos` ratio. Turn an `unknown` into a yes or a no anywhere. Write new search or geometry code per tool; if a capability seems to need its own search, stop and say so. Use MCTS or a field over a grid. Touch `bot/`, `docs/`, the gates, the fixtures, or the law's arithmetic. Edit the kill table or the held-out set after a run. Commit, push or merge.

**Done means.** Step 0's four fixes; the baseline rerun at both caps under stated identical conditions; the solver measured on the 18 tracks **and** the held-out set; the scoreboard above; the run records written. **Gates:** `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants, and `cargo test --offline` from `engine/` passing **except** the known-broken `engine/plan` doctest at `walk.rs:4`, which imports `law::Refusal` unavailable to rustdoc — you may fix that import, and if you do, say so; you may not disable or ignore any other test.

Final message: the per-track table, which of the four movers moved and by the definition above, the held-out result, what Step 0's bound became (proved, or ordering-only), the provenance of every pre-built table, every place you recorded `unknown` or a refusal, the net lines, and every audited constraint you could not satisfy and why. **A spec that fails its own kill table honestly is a result; one that passes by editing the table, by memorising the tracks, or by resolving an `unknown` is a failure.**
