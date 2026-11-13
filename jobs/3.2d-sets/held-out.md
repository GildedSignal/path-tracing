# The held-out set for job 3.2d — frozen by the integrator, 14 November 2026

**Status: frozen.** Chosen and committed by the integrator before any solver code exists, so that the builder cannot select it, and not to be edited after the first run. Job 3.2d's spec points here.

**Why it exists.** A planner that solves the 18 known tracks may have memorised them. Nothing in a per-track measurement can tell the difference, because the tracks are the thing being measured. So the same solver, with the same pre-built artefacts and no new tuning, is run on problems derived from the tracks by a rule written down in advance.

## The rule

Three transformations, applied to **every** track the harness already loads. They are deterministic — no seed, no sampling — so the set is reproducible from this file alone, and a builder cannot quietly choose a friendlier instance.

| # | Transformation | What it tests | Counts toward pass/fail |
|---|---|---|---|
| **H1** | **Translate** every block and the start and goal by `(+137, 0, −91)` whole blocks. | That nothing is keyed to absolute coordinates, and that no table was built against the tracks' own positions. Whole-block translation leaves every block shape, collision box and law comparison identical, so **any** change in result is a defect, not a harder problem. | **Yes** |
| **H2** | Keep the track, but start the bot **off centre and moving**: at the start cell, offset `(+0.3, 0, +0.3)` from centre, with horizontal velocity `(+0.08, 0, +0.08)` blocks/tick, on the ground, not sprinting. | The "rest at a cell centre" assumption, which job 3.2e left open — 12 tools on `grassjumpeasy` land from a nearby airborne state but not from rest at centre. A set that cannot carry an arriving state will show it here. | **Yes** |
| **H3** | **Mirror** the track about the x axis (negate x for every block, the start and the goal; mirror the goal's approach accordingly). | Whether anything was learned about these tracks' specific shapes. Note that Mojang's sine table is **not** symmetric under this mirror (`footprints.md`: the pattern key covers the identity frame only), so a mirrored track is a genuinely different problem and some loss is expected and legitimate. | **No — reported separately** |

## How to report it

- **H1 is the sharp one.** The result must be identical to the unmirrored, untranslated run, track for track: same arrival, same T, same certified sentence up to the translation. **Any** difference is a finding, and a failure to arrive where the original arrived is a memorisation signal — report it as such, not as a hard instance.
- **H2** is reported like the main table: arrival, T, expansions, refusals. A track that arrives from rest at centre and not from H2's state tells us the set does not carry arriving state, which is precisely what Step 1 of the spec is for.
- **H3** is reported as a count and a note, never as a pass or a failure.
- Run the **same solver with the same pre-built artefacts** and no track-specific input. If an artefact was built from the 18 tracks, say so here too: that is the thing most likely to be memorisation in disguise.
- Do not look at these results until the main table is finished, and never tune against them.

## What this set cannot tell us

It is derived from the same 18 tracks, so it tests invariance and arriving state — not whether the planner generalises to terrain of a kind it has never seen. That is Phase 3's question (`learning.md`: hand-made maps from simple to the hardest), and this set is not a substitute for it. Say so in the report rather than claiming more.
