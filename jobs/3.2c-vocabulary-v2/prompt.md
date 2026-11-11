# Job 3.2c: vocabulary v2 in the code (the class, the store, the counters)

**Why.** The word record is now version 2 (`docs/vocabulary.md`, 11 November, S79–S81), after a red-team Julien approved. `engine/words` still follows version 1: its class rounds position and velocity with chosen numbers, its proposal table finds a word only when the nearby cells match, every replay files a certificate under an exact start nothing reuses, and tick ranges only widen. The planner (3.2b) must be built on the record as it now stands.

**Consumer.** Job 3.2b (the planner) and the first flight; vocabulary.md section 11's kill lines, which need these counters to exist.

**Read.** `docs/PRINCIPLES.md` (goal, philosophy, From Emergent Space), `docs/vocabulary.md` (all of it, version 2), `docs/footprints.md`, `docs/representation.md` sections 2, 3 and 5, `docs/scales.md` 4.1 and 4.4, Julien's turns S67–S81 in `docs/reference/julien-2026-11-10.md`. Code: `engine/words` (all), `engine/law/src/state_reads.rs`, `engine/law/src/pattern.rs`. Evidence: `lab/results/local-20261111-1108-1-2-class-without-numbers/` (its `candidate.patch` built class variants in `class.rs` and a 16-track protocol in `tests/class_variants.rs`; reuse what fits).

**Branch.** `wt/class-v2`, worktree `D:\path-tracing-wt\class-v2`, cut from `main`.

## Build, in this order

1. **The class (section 3).** One rule per field the segment read before writing it. A field whose values come from the block table or the toolkit (attributes, pose, the stuck multiplier) enters by its bits. A field that accumulates (position, velocity, fall distance, counters) enters only as the yes-or-no answers of comparisons the law itself makes on it: find them in `engine/law`, name each with its file and line, and add none of your own. `Grid`, `BODY_EDGES`, `SIXTEENTHS` and the velocity bits go. No chosen number enters.
2. **The world part orders, never excludes.** Proposals live under the class alone; each keeps the pattern key of the cells its last replay read. A lookup returns every proposal of the class, those whose world part matches here first. A miss is a class with no proposal at all.
3. **Outcomes, not widening ranges.** A proposal predicts its most frequent (exit, ticks) outcome and counts each outcome it has seen; "as proposed" means the replay matched that prediction.
4. **The store keeps gait cycles only (sections 2 and 6).** No certificate is filed per replay; `Certified` keeps the replay's footprint and computes its key only if asked. A gait is an exact recurrence: after one cycle every field but position has the same bits, and position has moved by the cycle's displacement. `discover_gaits` finds gaits that way and keeps each cycle once (the cycle memo).
5. **Inventing a piece (the miss).** The record's compile step: from a state, run every program at every aim to its event, certify each, learn each. Time it: this is the miss price, never measured until now.
6. **The counters (sections 3 and 11).** Per table: hits, misses, surprises, refusals; proposals returned and replayed per decision; regret (for a sample of hits, invent from the same state and count the ticks the best piece reaching the same exit kind would have saved); growth along a walk (new proposals per thousand pieces walked, uses per proposal, proposals used once).
7. **Every surprise is a witness (section 2).** Each surprise writes one record that replays exactly: the start state's bits, program and arguments, the predicted outcome and the actual one. Never suppressed.

## Measure

On his 18 track fixtures, with the class lane's walk protocol (64 seeded runs, the 54 programs, arbitrary offsets): for v1's class and v2's, the hit rate, as-proposed, proposals replayed per decision (p50, p90, max), the measured miss price, the replay and lookup prices, and the cost per certified tick with the table against inventing every piece (vocabulary.md section 11's first kill line). The growth curves and the Zipf counts per track. Regret on a sample. Report n, min, p50, p90, p99 and max, the machine's load, and every anomaly.

## Constraints

- The law's arithmetic does not change: the jar suite stays 6 of 6, and `jobs/1.2-footprints/verify.ps1` still catches 12 of 12 mutants. `jobs/3.2a-segments-and-gaits/verify.ps1` passes or its changes are explained.
- No tolerance, no special case per block, no catalogue of aims or techniques. If a needed comparison is not in the law, write it down: that is a finding, not a patch.
- If the cells' role in the key would change meaning beyond step 2, stop and describe it.

## Must not

Touch `bot/`, the gates, `docs/` or `engine/law` beyond reading. Run anything live. Push or merge.

## Done means

A final message with what you built per step, the measurements above with v1 against v2, the comparisons the class uses (each with its law line), what you did not build and why, every anomaly, and the commits on `wt/class-v2`.
