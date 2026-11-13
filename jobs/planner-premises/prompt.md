# Job planner-premises: everything the planner's next spec must not contradict

**Why.** The planner is the gate to the first flight. Its first version was judged **FAIL**, and the rewrite (job 3.2d) has to respect decisions scattered across a dozen files, two judges' reports, a red-team that exists only as a list in HANDOFF, his own turns, and two lab lenses that landed tonight. The integrator is writing that spec now. A premise it misses is a one-way door: the spec freezes, a builder spends hours, and a judge fails it for contradicting something nobody re-read.

**Consumer.** Job `3.2d`'s spec, written immediately after you. Nothing else.

**You are reading, not deciding.** Do not design the planner, do not propose a representation, do not say what the spec should do. Your output is a list of constraints that already exist, each with the file that imposes it.

**Read, following every citation:**
- `docs/learning.md` whole — section 1 especially, "the map plans; the search only reads it", and the corrected reading of the run records.
- `docs/vocabulary.md` whole, section 12.4 especially (version 3, approved S101: the planner does not choose at a body-block change).
- `docs/HANDOFF.md` — "Now", items 1 to 3, and the in-flight table rows 1 and 2. The Opus red-team's required changes to 3.2d exist **only** as prose in this file; extract every one of them as a separate constraint.
- `jobs/3.2b-judge/last.md` — the four fixes the judge's FAIL demands.
- `jobs/3.2b-planner/prompt.md` and `jobs/3.2d-sets/prompt.md` — the latter is the **stale draft** the red-team sent back; treat it as a source of what was wrong, never as current.
- `jobs/3.2f-stalled-joints/last.md` and `jobs/3.2f-b-joints/last.md` — the two joint measurements, the second of which answered what the first could not.
- `jobs/3.2e-probes/last.md` — the two probes, one of which was inconclusive and says why.
- `lab/wiki/monotone-tick.md` and `lab/wiki/jump-puzzle.md`, `lab/wiki/backward-tools.md`, `lab/wiki/gait-corridor.md`.
- `docs/engine.md` for the law's contract, `docs/representation.md` and `docs/scales.md` for the framework, `docs/PRINCIPLES.md`'s register for the rows that mention the class, the bound or the search.
- `docs/reference/julien-2026-11-10.md` — S95 to S101 and X14 to X16.

**Return**, at most 70 lines, in four lists:

1. **Hard constraints** — things the spec must satisfy or be wrong. One line each: the constraint, the file and line that imposes it, and whether it comes from a judge's FAIL, a red-team, Julien's own words, or a measurement. Rank by what breaks if it is missed.
2. **Settled facts the spec can rely on** — measurements that are reproduced and therefore usable as premises, each with its number and its run folder. The two lenses from tonight belong here if they qualify; say plainly if one does not.
3. **Open questions the spec must not assume away** — things the record explicitly leaves undetermined. 3.2f-b's "not determined by this measurement" is one; find the others. For each, say what a spec is allowed to do while it stays open.
4. **Contradictions between these sources.** Where two of these files cannot both be right about the planner, name both and say which is later. Do not resolve it — flag it, because the integrator has to.

**The one rule.** Every line cites a file by path, and a line number where you can give one. A constraint you cannot cite does not exist. If a file the list above names is missing or unreadable, say so in your report rather than working around it silently.
