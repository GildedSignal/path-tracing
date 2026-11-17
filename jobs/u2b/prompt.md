# Families and points, and the simplest case right (one job, S187)

**Julien's rulings:**
- **S188:** "Yes, adopt it". Sets are stored as families and points, with resources and the supporting block out of the key.
- **S186:** "why do we need to clone? can't we have a smarter operation? … it is important to get even the simplest case right, if we can't make them right the rest will compound badly".
- **S187:** "wait and do it all at once".

**Read:**
- **The design you build:** `C:\Users\PASCAL\Desktop\path tracing\jobs\set-rederive\proposal.md`, all of it. Section 4 is the builder's list. The experiment behind it is in `jobs\set-rederive\kt11x\`.
- **The rulings in the docs:** `docs\representation.md:179` (the set, as ruled) and `docs\learning.md` kill test 11 (pieces are few).
- **The plan:** `docs\plan.md`, steps 2 and 2b.
- **Your own records:**
  - joints and sets: `D:\path-tracing-wt\u2-astra\jobs\u2\last.md`;
  - the core: `D:\path-tracing-wt\u1-astra\jobs\u1\last.md`;
  - the core's judge: `jobs\u1-judge\last.md`, its five-step path to ~5 µs.

**You are the builder (S174).** Where the design is under-determined, use your judgement under the plan's rules, and record each decision and its reason in `last.md`.

## Base and rules
- **Worktree:** `D:\path-tracing-wt\u2b-astra`, branched from `wt/u2-astra`, which holds joints and sets on top of the core. It runs at Normal priority, offline, with `CARGO_TARGET_DIR=D:\path-tracing-target\u2b`. No game, server or client.
- **Never run** `launch.ps1`, `gpt.ps1` or `follow-up.ps1`.
- **A failed premise stops only what depends on it.**

## Part 1: families and points
Do everything in section 4 of the proposal:
- the law's declared list of non-monotone fields;
- `EntryTemplate` with resources as uses and guards, the supporting block relative to the body, and the lattice coordinate;
- `Plateau` becomes `Piece`;
- `learn()` finds cuts by bisection on the exact maps, never by sampling;
- `consume()`;
- `wave()` over (family, piece) and points.

**Test first:** `u2_pieces_computed_and_closed`, frozen before its first run, on the gap and the step at yaws 270 and 275.625, `hardtrack-lane-wall` and `glm-ice`. Opus expects 5, 8, 2 and 2 pieces. Keep the old kill-test-11 and translation-premise tests as recorded reds of the old object, never weakened. Add the monotone-premise test.

## Part 2: the simplest case right
In a **warm** solve of the straight million-block line (axis-aligned and oblique):
1. **Zero player-state copies and zero heap allocations.** Count both: copies through the state type, allocations through a counting global allocator in the test binary. Assert zero, so it cannot be faked. A probe reads the stored phase template by reference and computes position by the exact map.
2. **The stop on B is a joint.** "Which last word, from which gait phase, touches this kind of face" is computed once per (floor label, gait, face class), as the preimage from the face, and stored in the set store as a family or points. A warm solve looks it up. It is never re-searched.
3. **Apply the core judge's five-step speed path** where it keeps results bit-exact.
4. **Measure** the warm million line, as min, median, p99 and max, against the plan's ~5 µs floor. If it misses, say exactly what remains, with a profile.

## Gates (all must pass, or be recorded red with the reason)
- part 1's test;
- kill test 10 (his route, against his 169);
- the two-speeds soundness test;
- one mechanism: the planted shortcut must fail, through the counters;
- the zero-copy and zero-allocation asserts;
- the core's regression;
- the 13 footprint mutants;
- the 3 core mutations;
- the workspace.

**`jobs\u2b\last.md`:** the files changed, each decision, the timings as distributions, and the replay commands. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
