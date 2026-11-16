Continue the same job. Your stop was right: the spec's half-width literal `0.3` is false. The integrator's decision:

- **Never type the body's half-width.** Take it from the law's own expression, `(0.6f32 / 2.0f32) as f64` (= 0.30000001192092896), by calling or reusing the law's function or constant that computes it (find it in `engine\law\src\` and cite file:line). The same applies to every other dimension the stop slab uses (height, eye height): take each from the law.
- **Then finish everything you skipped:** the binade map's exactness, the segment counts for 100, 10⁴ and 10⁶ blocks, the inverse stop at the detector edge with the law's half-width, the timings, the poison test, and the remaining gates.
- **The plan has changed since your prompt** (S165): P1–P4 are now "the kernel" that every zone uses (`docs\learning.md` section 0; `docs\HANDOFF.md`, U1). Nothing in your scope changes.

If another literal in the spec disagrees with the law, use the law and record the difference in `last.md`; that is not a reason to stop. You are the Terra builder: never run `launch.ps1`, `gpt.ps1` or `follow-up.ps1`. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
