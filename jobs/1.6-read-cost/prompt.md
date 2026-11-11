# Job 1.6: what a certified replay really costs, and what it should cost

**Why.** Every use of a cached move is replayed from the bot's exact state, with its footprint recorded, so that the replay certifies it (`docs/vocabulary.md`, line 19: "a 12-tick replay costs 1–6 µs"). Job 3.2a measured, and its judge confirmed, per segment of about 1.55 ticks: the law alone 5.8 µs, the recorded segment 41 µs, with the certificate 45 µs, the lookup 3 µs. That is about 50 times the record's assumption per tick, and it is paid on every step of every plan, so the planner (3.2b) and the swarm's cost per bot rest on it. Julien (S71): "this is very expensive and fondamental ... holistically there might be more to it than just that."

**Consumer.** Job 3.2b's Fermi; the roadmap's kill line for the ground ("no path below about 5 µs means the `World` interface is wrong").

**Read.** `docs/PRINCIPLES.md` (goal, philosophy, "From Emergent Space"), `docs/footprints.md`, `docs/representation.md`, `docs/vocabulary.md`, `docs/engine.md`, `AGENTS.md`. Code: `engine/law` (`world.rs`, `footprint.rs`, `pattern.rs`, `movement.rs`, `shapes.rs`, `state_reads.rs`), `engine/world` (`WorldSnapshot`), `engine/words`.

**Branch.** `wt/read-cost`, worktree `D:\path-tracing-wt\read-cost`, cut from `main`.

## The question, whole

Do not start from "make the recorder faster". Start from what is paid and why, at every layer, and ask at each whether the work should exist at all (the best compute is no compute).

1. **Measure first, with a committed harness.** Per tick and per segment, on flat stone and on at least three of his track fixtures: calls to each `World` method, distinct cells, which law function made each call, and how many are repeats. A time breakdown that sums to the total: the law's arithmetic, the world's answers, the recording, the key, the certificate store, allocation. The known leads, none of them confirmed:
   - about 558 `block()` calls per tick for about 95 distinct cells per segment, each paying a map insert;
   - `WorldSnapshot` costs 3.7 µs per tick unrecorded against 0.6 on `BoxWorld`, because `stonesimple` answers `has_fluid` and `has_inside_effects` true and the fast paths switch off;
   - segments last 1.55 ticks on average, so fixed costs per segment dominate;
   - the certificate store grows by one start state per replay;
   - the state-read hook costs the law 3 to 5%.
2. **Why does the law read so much?** It is a port of Mojang's code, which re-reads cells in each of its checks. Our law must give the same bits, not make the same calls. What is the smallest set of reads a tick needs, and what interface between the law and the world gives it (a neighbourhood fetched once, a swept box, something else)?
3. **Is recording each read the right way to get a footprint?** Alternatives to weigh from first principles, with their errors: a footprint derived from the body's swept box, with no recording (an over-approximation: what does a larger footprint cost in false invalidations and lost reuse?); a footprint attached to the cached proposal and only checked on replay; recording on a miss only. Which roles of the footprint (`docs/footprints.md`) need the exact set, and which only need a bound?
4. **Is replaying every use still right at the real price?** The record's argument was that a replay costs about as much as a lookup. State what the argument becomes at the measured cost and at the cost you can reach.
5. **The floor.** A Fermi for the least a certified segment can cost on this machine, with its inputs, and the gap between that and what you reach.

## Build

What the measurements justify, in order of gain per line. Constraints:
- The law's arithmetic does not change: the jar suite stays 6 of 6, and `jobs/1.2-footprints/verify.ps1` (offline) still catches all 12 mutants.
- If a footprint or pattern key changes meaning (for instance from exact to a bound), do not build it: describe it, with the measurement that argues for it. That is a one-way door for the integrator and Julien.
- No tolerance, no special case per block, no cache keyed on anything that is not what the law reads.

## Must not

Touch `bot/`, the gates, or `docs/vocabulary.md`. Run anything live. Push or merge.

## Done means

A final message with: the measured breakdown (before), what you changed and the breakdown after, as p50 and p90 over at least 2000 samples, with the machine's other load stated (GLM lanes may be running: counts are exact, timings are not; give ratios and say when you measured); your answers to questions 2 to 5, each marked measured, derived or guessed; what you recommend that you did not build, and what would prove it wrong; every anomaly; the commits on `wt/read-cost`.

A GLM lane (`recorder-cost`) works on the narrow form of this in parallel; its run folder will appear under `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261111-*-recorder-cost`. Read its `dossier.md` and `check.md` if they exist when you are near the end, and say where you agree or differ.
