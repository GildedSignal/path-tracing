# Job 3.2d pass 2: the three-answer spine, and what a set is

**Why this is its own job.** Job `3.2d-sets` asks for a spine, a backward solver, a measurement and a held-out run in one unattended pass. That was too much for one job and the builder said so after finishing the first piece. This is the second piece, and it is deliberately narrow: **the representation only, with no solver and no measurement.** Build it so well that the solver has nothing left to decide.

**Consumer.** Pass 3, the backward solver, which cannot be written until these types exist. Through it, the first flight.

**Branch.** You are on `wt/3.2d-sets` in `D:\path-tracing-wt\sets`, at `0c543a3`, where pass 1 left Step 0 fix 1 done: the `19.902886` b/s average now orders and never excludes, and exact-state dominance uses the recorded state bits including the tick. **Keep that; build on it.** The full specification is `jobs/3.2d-sets/prompt.md` in the main checkout at `C:\Users\PASCAL\Desktop\path tracing` — read it whole, because this job is two of its sections and must not contradict the rest. Its bibliography, `jobs/planner-premises/last.md`, carries the file and line behind every constraint.

## Build exactly two things

### 1. The spine

Every test the planner makes about a set — "is this state a member", "may this set be discarded", "does this bound exclude this candidate" — returns one of **proven yes**, **proven no**, or **unknown**.

- Make it a type, not a convention. A `bool` anywhere in this path is a defect: it is how an `unknown` silently becomes a yes. The compiler should make the wrong thing hard to write.
- **Only a proven answer may act.** An `unknown` may order, and may do nothing else: never discard, never certify membership, never suppress a fallback, never end a search.
- **An exhausted budget or an unavailable proof produces a recorded refusal**, carrying what was asked and why it stopped — never a yes, never a no.
- Write the tests that prove the spine holds: a discard attempted on an `unknown` must be impossible or must refuse; a refusal must carry its reason; an `unknown` must be unable to certify membership. **These tests are the deliverable as much as the type is.**

### 2. The set

A set is **arriving states**, never cells. The cell stays nature's address for block reads, footprints and premises, and is not a node.

- **It carries everything the law reads.** Face and class booleans and a velocity interval are not enough: position within the face, airborne status, and every other law-relevant field belong to it. In your report, state for each field whether the set represents it **exactly**, **bounds** it, or **conservatively discards** it, and the same for the correlations between fields. A field that is neither represented nor conservatively over-approximated is a soundness bug.
- **Bounds are derived, never chosen.** Each bound traces to a comparison the law makes, and that trace is recorded. A tolerance, an epsilon, a grid spacing, a bucket width, a threshold or a fixed subdivision depth chosen by you is forbidden; a derived interval endpoint is not.
- **Corner checks order; they do not decide.** The tick is corner-monotone on 255,136 of 255,150 one-tick member-ticks (`lab\results\local-20261113-1727-1-0-monotone-tick`). The 14 exceptions have a named cause — an `acos` ratio rounding above 1 at `movement.rs:802-803`, yielding `NaN` on `minor_horizontal_collision` and flipping `sprinting` at `movement.rs:217` — and k-tick escapes peak at **0.427% at k = 3**. **You may not clamp that ratio**: the law must equal Mojang's. So a corner-derived enclosure is `unknown` wherever a non-monotone case is not separately proven conservative, and the spine must carry that.
- **Splitting** happens only where the law makes a comparison. **Dropping** needs a proof; with no proof the set is recorded unresolved and kept.

## Not in this job

Do not build, and do not half-build: the backward solver, switches, aim, the proposal lookup, the premise and footprint check, the timing accounting, the restoration proof, the baseline comparison, the kill table, or the held-out transformations. If the spine or the set forces a decision about one of those, **write the decision down in your report and leave the code alone** — pass 3 needs that note more than it needs the code.

**Must not.** Add a tolerance, an epsilon, a retry, a clamp — including on the `acos` ratio — or a per-block branch. Let an `unknown` become a yes or a no anywhere. Represent a set by cells, a grid, or a field. Touch `bot/`, `docs/`, the gates, the fixtures, or the law's arithmetic. Commit, push or merge.

**Done means.** The spine and its tests; the set type with its field-by-field account; `cargo test --offline` from `engine/` passing **except** the known-broken `engine/plan` doctest at `walk.rs:4`, which imports `law::Refusal` unavailable to rustdoc — you may fix that import and say so, but may not disable or ignore any other test; `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants; a run record under `D:\path-tracing-evidence\`. Final message: the field-by-field account, where `unknown` can arise and what the caller must then do, every decision you wrote down for pass 3, the net lines, and anything left out.
