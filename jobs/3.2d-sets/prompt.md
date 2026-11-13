# Job 3.2d: the map is sets of states (draft for the red-team; not frozen, not launched)

**Why.** Job 3.2b measured it on Julien's 18 tracks: where the backward sets reach A the first sentence comes after 10–179 expansions; without them A\* over exact states fills 1.5 M nodes in about 3,000 expansions and its bound discards none. The map plans and the search reads it (`docs/learning.md` section 1; S97, X16). But the map is built of cells: `engine/plan/src/reach.rs` gives each standable cell a T (a block with its own T, ruled out in S91), takes off from cell centres after a run-up from rest on a flat floor, skips the tube of a walked hop, and keys hops by whole-cell displacement. Ten tracks get no sentence and he beats six of the eight that do.

**Consumer.** The executor and the first flight (HANDOFF roadmap item 2); learning's failure list (`docs/learning.md` section 6, test 1).

**Read.** `docs/PRINCIPLES.md` (goal, philosophy), `docs/learning.md` (all), `docs/vocabulary.md` (sections 3, 4, 7, 10, 12), `docs/representation.md` sections 2, 3, 5, Julien's turns S90–S99 and X9, X16 in `docs/reference/julien-2026-11-10.md`, `engine/plan` (all), `engine/words`, `lab/wiki/arc-certificate.md`, `gait-modulo.md`, `generative-function.md`, and `gait-corridor.md` if it has a reproduced result.

**Branch.** `wt/sets`, cut from `wt/planner` (`7713f9d`). Changes in `engine/plan`; in `engine/words` only where a step names it.

## Build, in this order

1. **A set is states.** A backward set is the arriving states one word carries into the next set, written as the comparisons the law makes: the supporting surface (a top face from the block table's shapes, not a cell), the facts of the class (on ground, sprinting, the friction below), the velocity the word needs as an interval, and the ticks over which its premises hold. No cell centre, no grid on position or velocity, no number of ours.
2. **Built from the words themselves.** The set of takeoffs from which a word lands in a set is computed from that word's own run through the law: a free arc is the same wherever it starts (`arc-certificate`), so the takeoffs are the landing surface shifted back by the arc; a gait's reach is its cycle's increments (`gait-modulo`). `Reach::compile`'s synthetic floor from rest goes: the run-up is the word before, and its exit velocity is what the next set asks for. One source of truth with the proposal table.
3. **The tube of a set is checked along its length**, walked hops included, against the block table's real shapes (a fence, a door, a pane are thinner than a cell). If every member's tube is clear the set stands whole; if none is, it falls; otherwise it is split, and only there. Count the splits.
4. **A goal is a region to touch**, by the detector's own rule, whether or not a body can stand on it.
5. **Decisions at switches.** A segment still ends on the events of vocabulary.md section 4, but "the body's block changes" is a check on the certificate, not a node the planner chooses at. Between two switches one word repeats: a gait by its cycle and a count, valid as far as the ground's generator says. Aim is solved from the two sets it joins, then refined; it is not enumerated over 64.
6. **Forward, slide, certify** as in 3.2b: the point inside each set from the exact state; slides tightest joint first, kept only if the replayed T falls; one replay from the exact start certifies the sentence or it is not returned. Premises as in 3.2b (S95).
7. **Exact-state search is learning's.** It opens only at a joint where no set arrives, from the exact state there, toward the next set, with a budget stated in ticks of thinking; the report says where it opened and what it found. `Order::Admissible` over the whole track is removed.
8. **The flight report** as in 3.2b, plus per track: decisions (switches and narrow joints), sets built, splits, replays per joint, failed replays, the bracket, and the width (clearance, delay tolerance) of the tightest joint.

## From 3.2b's judge (Terra, 13 November: FAIL, `jobs/3.2b-judge/last.md`), to fix here

3.2b does not merge alone; this job is cut from `wt/planner` and carries its fixes.
- **The bound must be a proof or it discards nothing.** 19.902886 blocks/s is an average over a 3-tick orbit, used as a per-tick maximum, from a start that may already move faster. Either derive a per-tick bound from the law's rule with its premises stated (and the start's velocity inside them), or keep the number for ordering only and discard by exact dominance alone.
- **Thinking counts from the click.** Whatever is built after the command (reach tables included) is thinking, unless it was built before the command and is keyed so that it is reused; say which.
- **The join asks the world, not a cache keyed by an epoch someone supplies.** A changed cell breaks the sets that read it whatever the epoch says, and a set's premises are the cells its check actually read, recorded, not a reconstructed scan.
- **Name what is measured.** 3.2b's "sets backward, points forward" was one forward search ordered by a field over cells. This job's sets are carried backward through the chain and the point is chosen inside each on a forward pass; the report keeps the two apart.

## Measure, against 3.2b on the same 18 tracks, same words

Sentences found; T against his; thinking time; decisions per sentence against its switches; the ten failures one by one (moved or not, and why); the six tracks behind him.

## Kill

The cell-type failures (fences, redfakemaze, amertystcoral, ametwoodbox, and the three unexplained) and the six tracks behind him do not move: then cells were not the bottleneck, and the report says what is. Decisions grow with a track's length. A set needs geometry code of its own beyond the block table's shapes and the law's arithmetic: stop and describe it.

## Constraints and must not

No tolerance, no special case per block, no catalogue of jumps, no number of ours. Block interaction (doors, trapdoors, placing) is not in this job: those tracks stay failures, named as missing words. The jar suite stays 6 of 6; the footprint mutants 12 of 12; every returned sentence is certified. Do not touch `bot/`, the gates, `docs/` or the law's arithmetic; run nothing live; do not push or merge.
