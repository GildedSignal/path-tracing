# The warm solve at its floor: layers 1–4 of the idiot-index report (S218)

Builder: Sol 6.2. Worktree `D:\path-tracing-wt\warm-floor`, branch `wt/warm-floor`, made from the tip of `wt/u2b-astra` (the simplest case right, step 2b; judged in parallel). The integrator commits; you never run `git commit`, `launch.ps1`, `gpt.ps1` or `follow-up.ps1`, and use no network, game, server or client.

## Why

Julien asked what a million-block straight line would take in 100 ns (S212–S214); Opus's idiot-index pass answered (`jobs/hundred-ns/report.md`, read it all first, especially sections 1.1–1.4, item table 1.3 and section 4). Julien chose its layers 1–4 for you (S218). They take the warm solve from 112–167 µs (interleaved) toward its ~1 µs floor **without changing the representation**: families, points and holds (representation.md section 10) and the set record stay exactly as they are. Layers 5–7 (the error bound, the label's own cone, the stop on B as families over the aim) are not yours: they touch the set record, a one-way door, and get their own red-team.

Read also: `AGENTS.md` (no patches, no new search code, one mechanism, seal only what can't be corrected), `docs/plan.md`, `docs/learning.md` sections 0 and 6 (the kill tests), `docs/representation.md` section 10, `docs/engine.md` ("How it computes": store the generator), `docs/vocabulary.md`, and `jobs/u2b/last.md` with `jobs/u2b/verify.ps1` (every gate you must keep green).

## Layer 1: measure honestly (first; nothing else changes until it is in)

1. **The headline is the interleaved warm solve.** For the three million-block goals (axis X, axis Z, oblique; the cells in `tests/zones_prereqs/u1.rs` and `tests/u2b_cost.rs`), report warm solves both ways: back to back, and each preceded by a cold solve on a fresh store (the U1 harness's way, `u1.rs:60-69`). n ≥ 100 each; min, median, p99, max, all samples. Windows' timer ticks at 100 ns: time a batch where a single sample is under 10 µs.
2. **Counters, in the solve, read by the tests:** full-length walks of the distance per solve (a walk = iterating binade crossings from start to end on one axis); hash-map probes per solve; distinct phase records read; bytes allocated (already zero; keep it so).
3. **A timer on a warm decision on a seen track:** time the warm solve in his shortcut's gate (`u2_kill10_discovered_program_current_record`, 158 ticks against his 169), same statistics. It is not timed anywhere today.
4. **Gate:** on the unchanged code, the numbers reproduce the report's (57/69/92 µs back to back, 112/124/167 µs interleaved, within run-to-run spread, stated). Record them as the baseline.

## Layer 2: the yaw-free generator

Today a cold bundle costs 192–215 ms and 4,785 law ticks because kernels are prepared per yaw and keyed by the start's velocity bits (`store.rs:193-231`). Replace that with the generator:

- **Derived from the law, never a second copy of it.** Per label and gait program, run the law once and record, per tick, the operands the law actually applies to the horizontal axes (the per-tick multiplier and acceleration, as `store.rs` already records "the actual addition operand and the clamp bit") together with the yaw-free vertical, ground and flag cycle. The law's recurrence is untouched; observation is added the way `law/src/contact.rs` was.
- **A yaw's kernel is then two scalar recurrences** evaluated from those operands and the table's sine and cosine for that yaw, in the law's own operation order, from any starting velocity: so a moving start needs no bundle of its own. Keys exclude the start's resources (S188) and carry a gait's phase, never velocity bits.
- **Certificate, one mechanism:** a yaw for which the law's contact or clamp outcome differs from the generator's must appear as a piece of the label's family over the aim (a range of yaw indices with its own law-derived operands), never as an `if` on the yaw. If that cannot be done without a branch on terrain kind or yaw, stop and write `STATUS: BLOCKED` with the case.
- **Gate:** bit-equal to `law::try_tick` for all 65,536 yaws over the transient plus two cycles, on each bare-legs label of the fixture, from standing and from at least three moving starts (about 20 s). A planted one-ulp change to one operand must turn it red. Report the cold solve's new time and law ticks, and what still costs (the stop sets stay per kernel until layer 7; measure them).

## Layer 3: data shaped like the read

Report items 4 and 9 and idea 3: phase records of at most 40 bytes holding what the solve reads (position operands, y, pose, the flags it tests), not 360-byte player states; the per-binade graphs as a dense array per map indexed by exponent, phase and parity, not `Arc<Vec<i128>>` behind `RefCell<HashMap>`; one content key per observed state computed once, not a motor key rebuilt per kernel; the law hash checked once when the store opens, not on every stop lookup (`terminal.rs:69`); a bundle record of at most 1 KB. No hashing of keys larger than 16 bytes on the warm path.
**Gate:** the interleaved warm solve within 2× of the back-to-back one; counters show the phase records read and the probes per solve.

## Layer 4: one walk per axis

Report items 5–8: walk the binade chain once per axis per solve and keep the crossings as the solve's table; every readout (anchor, both bearings' legs, envelopes, the touch) reads the table. Derive events only when something looks at them; the output is words, counts and the corridor box (engine.md). Look up the stop on B for the phase the touch tick fixes, after the lower bounds prune candidates (the u1 judge's fix c), not 36 times. On the axis, replace the count-cap binary search (`lattice.rs:31-49`, `167-175`) with the closed form in wider integers.
**Gate:** the walk counter reads exactly one per axis per solve; the interleaved warm million line at or under 2 µs median on all three goals (the ~1 µs floor plus slack). If it misses, say exactly what remains with a profile; never loosen the target.

## What must stay true (every layer)

- Every gate in `jobs/u2b/verify.ps1` keeps its recorded outcome: the frozen point-or-hold gate (`engine/plan/tests/u2b/pins_point_hold.rs`, SHA256 `B78EAAF3A53DE7B0A70E1A2F3DA12B5246437962F7909000301309190468FF82`, never edited), the four old-object reds stay red and byte-unchanged, the core's 13 mutants stay caught, zero copies and zero allocations on the warm path, his shortcut at 158 ticks, the million line's planted shortcut still fails, the same work at 10³, 10⁴ and 10⁶ blocks. Same T and same end-state bits as before on every goal you time.
- No branch on terrain kind, distance, axis or straight line in `solve`; no tolerance, epsilon or retry; no new search. A number in code comes from the law or is derived.
- Tests answer one question fast (AGENTS.md); no exhaustive sweeps beyond the yaw gate.

## Output

`jobs/warm-floor/last.md`, written as you go: per layer, what changed (files), the gate's result with its numbers, the before/after table (back to back and interleaved, the three goals, the seen-track decision, the cold solve), the counters, and what remains against the ~1 µs floor. A `jobs/warm-floor/verify.ps1` that replays every gate. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`. Independent layers that fail do not stop the others: name them and continue.
