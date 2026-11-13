# Job 3.2d-recheck: did the rewrite actually fix the 16 defects?

You red-teamed this specification and returned **REWRITE** with 16 defects and no clean axis. It has been rewritten. Your job now is narrow and concrete: **check each of your own 16 findings against the new text, and look for defects the rewrite introduced.**

Both documents are inlined below; you need no file reads, and if a read is refused, continue from what is here.

**For each of the 16, return one line:** the number, then `FIXED`, `PARTIAL` or `NOT FIXED`, then the sentence in the new spec that settles it (quote a few words), or what is still missing. Be strict: a defect is `FIXED` only if a builder could not now do the wrong thing, not merely if the spec mentions the subject.

**Then, separately:**
- **New defects introduced by the rewrite.** A rewrite that defends against 16 attacks usually over-corrects somewhere. Look especially for: rules that now contradict each other; a requirement so strict that no implementation can satisfy it, which pushes an unattended builder toward a quiet tolerance; and places where the new three-valued `unknown` machinery is required but its outcome is left undefined.
- **Is the spec now buildable?** It must be satisfiable by a competent builder working unattended for hours. Say plainly if some requirement has no achievable implementation, and which.
- **Overspecification check.** The project re-derives a wrong representation rather than patching it, so a spec that dictates mechanism where it should state a requirement is a real cost. Name any place the rewrite now over-constrains.

**Do not** rewrite the spec or design a planner. Do not restate your original findings at length — you wrote them, they are below for reference. No padding; a generic observation counts against the report.

**End with one verdict word:** `FREEZE` (build it as written), `AMEND` (fix the named items, no re-review), or `REWRITE` (still structurally wrong). At most 45 lines.

---

# YOUR ORIGINAL 16 DEFECTS

1. **Kill table can pass without the intended result — Step 6, line 63; axes 1, 7.** “Moves” is undefined, and only one of four needs to move. A changed expansion count, an uncertified intermediate, or one easier sentence caused by a different budget could qualify while the other three remain stalled. **Fix:** freeze a per-track definition: a newly certified end-to-end arrival under matched conditions, or a separately named and replayed intermediate witness; report each of the four independently.

2. **Known tracks can be memorized — Steps 2–6, lines 32–64; axes 1, 7.** Nothing excludes track-specific prebuilt answers or tests reuse on changed starts and terrain. Replay would certify those answers without showing generalization. **Fix:** freeze a small, undisclosed variation set before implementation and require the same solver and prebuilt artifacts to run it without track-specific inputs or answers.

3. **The supposed bounds can be false — Step 1, line 26; audit #2, #9; axes 2–4.** Observed corner escapes invalidate an unqualified corner-derived enclosure; calling it a “bound” does not make it sound. Final replay rejects a bad *candidate*, but cannot recover a route omitted by an unsound bound or membership test. The no-clamp rule is compatible with soundness only if non-monotone cases receive a proven conservative treatment or remain unresolved. **Fix:** require proof for every bound used to exclude states or certify membership; otherwise label it ordering-only and preserve the uncertainty. No clamp is required.

4. **Set entry decides despite the exact-layer rule — Step 4, lines 44, 48; audit #2, #9, #15; axes 2–5.** “Switch fires” and fallback opens *only* on failed set entry, but the spec does not establish exact, decidable membership for approximate preimages. A false yes can suppress fallback; a false no can redirect execution. **Fix:** distinguish proven membership, proven non-membership and unknown; specify that uncertain results cannot suppress exact checking or discard alternatives.

5. **Certification and restoration are not explicit return conditions — Step 1, line 26; Step 6, lines 64, 68; audit #1; axis 2.** “Final replay decides” does not state that *every returned sentence*, including fallback and improvement paths, must replay from the exact start and satisfy arrival, survival and restoration. The predecessor’s judge found restoration was not established. **Fix:** impose those checks on every return path and require refusal otherwise.

6. **The measured joint cause is overstated — Step 3, line 40; axis 3.** The 42 replays depart on tick 1, and the stored hop starts pre-accelerated; the report explicitly says neither establishes the stall’s cause or reachability. “Because” and “run-up is the bottleneck” promote correlation to diagnosis. **Fix:** state these as observations and retain competing causes as unresolved.

7. **The preimage claim exceeds its evidence — Step 2, line 34; axes 3, 5, 6.** A sample of translated takeoffs does not prove inversion is computable for arbitrary velocities, programs, collisions, ticks or coordinates; “computed rather than searched” also dictates a mechanism. **Fix:** limit the measured claim to its tested domain, and require sound preimages or an explicit unresolved result elsewhere.

8. **The set’s state is incomplete or ambiguous — Step 1, line 24; Step 5, line 57; audit #8; axes 2, 5.** Face, booleans, velocity interval and ticks do not specify position within the face, airborne status, or other law-relevant state; a single velocity interval also leaves dimensionality and correlations unclear. **Fix:** require preservation of all state needed by the law and specify which fields and correlations the set represents or conservatively retains.

9. **The baseline is not reproducible or necessarily fair — Step 0, lines 17–20; Step 6, lines 62, 68; audit #3, #17; axes 2, 5, 7.** “Fixes” include a separate backward solver, so it is unclear what remains the 3.2b baseline. Caps, hardware, inputs, precomputation, termination and timing treatment are not matched or defined. **Fix:** identify the baseline algorithm and applicable corrections, then freeze identical run conditions and cap semantics for both methods.

10. **Thinking-time accounting remains gameable — Step 0, line 18; Step 6, lines 64, 68; audit #5; axes 2, 7.** A key requiring no post-click work does not establish that the table was available before the click, or account for loading, lookup, parallel work and tick rounding. **Fix:** record the actual click-to-restored-arrival clock, pre-click artifact provenance and all post-click wall time, with the conversion to server ticks specified.

11. **The scoreboard cannot judge performance — Step 6, line 64; axis 7.** One per-track T against Julien supplies neither the proven lower bound and independent-search best T nor a distribution; failures and timeouts can also disappear behind a dash. **Fix:** report the bracket or explicitly mark each unavailable component, plus repeated-run distributions, budgets and failure counts.

12. **Proof and termination duties conflict operationally — Steps 1–2, lines 27–32; Step 6, lines 62–68; axes 4, 5.** “Never dropped without proof” and exact preimages leave no specified outcome when proof cannot be obtained within a finite cap; an unattended builder is pushed toward an implicit tolerance or silent discard. **Fix:** define unresolved sets and budget exhaustion as recorded refusals, with no correctness or completeness claim.

13. **Local repair has no decisive acceptance check — Step 0, line 20; audit #7; axes 2, 5.** The requirement says what to invalidate but does not specify how changed contents at recorded ticks are compared, including the two un-evented reads. A cache can appear to pass without demonstrating both affected and unaffected sets. **Fix:** require recorded-read evidence and an edit check covering an affected read and an unrelated change, without an artificial epoch.

14. **Class-only lookup is over-prescribed from a timing sample — Step 4, line 47; audit #18; axes 2, 6.** The measured lookup cost does not establish that every context sharing a class has the same applicable proposals. **Fix:** retain the speed objective but require evidence that lookup preserves applicable proposals, or report the constraint as unresolved.

15. **The five expected failures are not checked as missing-word failures — Step 6, line 63; axis 1.** A timeout, unsound pruning, or unrelated defect can produce the expected failure and appear to confirm the table. **Fix:** report the actual refusal or missing capability and distinguish it from budget exhaustion and other failures.

16. **Snapshot bounds were dropped — Steps 1–6, lines 24–68; audit §3 open question 6; axis 2.** Offline execution does not itself guard reads outside snapshot halos. **Fix:** require bounds-checked reads and a recorded refusal or explicit supported world source on an out-of-snapshot lookup.

**Clean axes:** None.

REWRITE

---

# THE REWRITTEN SPEC

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
- This exists because the final replay is **not** a safety net for this class of error. A replay rejects a bad candidate, but a route wrongly excluded by an unsound bound or a false membership test is never generated, so the replay never sees it and the loss is silent and permanent.

## Step 0 — the judge's four fixes

Nothing else is built until these hold; before them the old numbers are not comparable.

1. **No bound discards without a proof.** `c = 19.902886` b/s is a **3-tick average**, not a per-tick bound, so it may order and must never exclude. Either prove a per-tick horizontal bound covering an arbitrary start velocity — stating the proof and its assumptions in the report — or prune **only** by exact-state dominance. This applies to *every* bound the planner uses to exclude or to certify, not only to `c`.
2. **Thinking time counts, and is clocked, not argued.** Record the actual wall clock from click to restored arrival, the conversion you use from wall time to server ticks, every post-click computation including table compilation and loading, and for each pre-built artefact its **provenance**: what built it, when, from what, and the key that makes its lookup need no post-click work. A table is outside T only if that provenance is recorded.
3. **"Sets backward" must be backward.** The old code was one forward search ordered by a field over cells, which is why its comparison meant nothing. Sets are carried backward along the chain, the point is chosen forward, joints slide inside their sets.
4. **Premises are checked against recorded read footprints at ticks** — never a supplied epoch, never a rescan. Prove it with an **edit test**: one edit that an affected set's footprint read (it must invalidate), and one unrelated edit (it must not), both at ticks, including the two un-evented reads named in Step 4.

## Step 1 — what a set is

A set is **arriving states**, never cells. The cell stays nature's address for block reads, footprints and premises, and is not a node. No cell carries a T of its own, there is no grid of ours, and no number in a set is chosen by you.

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

- **The baseline must be reproducible.** Name the baseline algorithm and exactly which of Step 0's corrections apply to it; run both methods under identical conditions — same hardware, inputs, precomputation, cap semantics, termination rule and timing treatment — at **cap 1 and cap 20**. State what differs, if anything, and why.
- **A held-out set, frozen before you implement anything.** Build a small variation set — changed start states and shifted terrain on tracks you do not tune against — and run the **same solver with the same pre-built artefacts** on it, with no track-specific input or stored answer anywhere in the system. A planner that only works on the 18 known tracks has memorised them and has not solved anything.
- **The kill table, frozen before the first run and never edited after it.** Expected to stay failures *for a missing word*: `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `ametwoodbox`, `mud`. Expected to move: `fences`, `redfakemaze`, `fencemaze`, `amertystcoral`.
  - **"Moves" means**: a newly certified end-to-end arrival under matched conditions, or a separately named intermediate witness that replays. A changed expansion count, a different budget, or an uncertified intermediate is **not** movement. Report the four independently.
  - **A failure must be the right kind.** For each expected failure, report the actual refusal or the missing capability, distinguished from budget exhaustion, a timeout, or an unrelated defect. An expected failure produced for the wrong reason does not confirm the table.
  - **If none of the four movers moves, this spec is wrong.** Report that as a kill, not as a tuning problem.
- **The scoreboard.** Per track: whether a sentence arrives; T against his; the **bracket** — the proven lower bound and the best T from the independent search — or an explicit mark for each component that is unavailable and why; a **distribution over repeated runs**, never an average alone; expansions; the budget; the failure and refusal counts; and the clocked thinking time inside T.
- **Write a run record** (AGENTS) under `D:\path-tracing-evidence\`: inputs for exact replay, each invariant's worst margin, and every refusal.

**Must not.** Add a tolerance, an epsilon, a retry, a per-block branch, a jump catalogue, or a clamp — including on the `acos` ratio. Turn an `unknown` into a yes or a no anywhere. Write new search or geometry code per tool; if a capability seems to need its own search, stop and say so. Use MCTS or a field over a grid. Touch `bot/`, `docs/`, the gates, the fixtures, or the law's arithmetic. Edit the kill table or the held-out set after a run. Commit, push or merge.

**Done means.** Step 0's four fixes; the baseline rerun at both caps under stated identical conditions; the solver measured on the 18 tracks **and** the held-out set; the scoreboard above; the run records written. **Gates:** `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants, and `cargo test --offline` from `engine/` passing **except** the known-broken `engine/plan` doctest at `walk.rs:4`, which imports `law::Refusal` unavailable to rustdoc — you may fix that import, and if you do, say so; you may not disable or ignore any other test.

Final message: the per-track table, which of the four movers moved and by the definition above, the held-out result, what Step 0's bound became (proved, or ordering-only), the provenance of every pre-built table, every place you recorded `unknown` or a refusal, the net lines, and every audited constraint you could not satisfy and why. **A spec that fails its own kill table honestly is a result; one that passes by editing the table, by memorising the tracks, or by resolving an `unknown` is a failure.**
