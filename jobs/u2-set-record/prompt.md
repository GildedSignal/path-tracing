# U2 draft: the set record (a one-way door), reach between faces, and the lenses for kill tests 10 and 11

**Consumer:** the Opus red-team of the set record, which runs before it freezes (`docs\HANDOFF.md`, U2), and then Terra's U2 build. Kill tests 10 (the map holds his shortcut) and 11 (plateaus are few, which "runs first") in `docs\learning.md` section 6.

You read only, and write only your report and the lens drafts. Root `C:\Users\PASCAL\Desktop\path tracing\`; read every file by its absolute path.

## Read
- **The approved plan:** `docs\learning.md` (sections 0, 1, 2, 6), `docs\representation.md` (sections 10–11), `docs\scales.md`, `docs\vocabulary.md`, `jobs\unified-plan\plan.md` (the set record, plateaus by bisection, premises, prices, the reach relation from mirrored reach tables, one worldtube check per arc), `jobs\unified-plan\learning-review.md` (the four-way status per joint).
- **Evidence:**
  - `jobs\hard-track\shortcut.md` and `julien-428.jsonl`;
  - the hard-track-2 reach certificate on branch `wt/hard-track-2` (`D:\path-tracing-wt\hard-track-2\engine\plan\tests\hard_track.rs`, `jobs\hard-track-2\report.md`, `old-graph.md`);
  - GLM's `trial-offset-monotone` and `port-wormhole-facts` results under `lab\results\` (folders ending in those names, with DONE), and `lab\wiki\index.md`;
  - the backward-tools lane (`lab\results\*backward-tools*`).
- **The code:** `engine\plan\src\` (`reach.rs`, `goal.rs`, fields), `engine\law\src\` (arcs, footprints).
- **The GLM lens format:** read three existing lenses in `lab\lenses\`, among them `trial-offset-monotone.md` and `port-sprint-window.md`.

## Write
1. **`jobs\u2-set-record\report.md`:**
   - **The set record:** its fields, the key (`LabelId` and joint pattern content), plateaus, premises (cells, ticks, kit), prices, the four-way status, the law hash and witness; how it is built (bisection over takeoff states), stored, invalidated by footprint, and read by `solve`.
   - **The reach relation between faces:** how it is derived from the mirrored reach tables without a search over inputs; how the hard-track-2 certificate's machinery is reused or replaced.
   - **Every one-way-door choice in the record, with its alternatives:** questions for the red-team.
   - Fermi floors with stated inputs.
   - Premises checked: EXISTS, DIFFERS or MISSING, with file:line.
2. **GLM lens drafts,** in `jobs\u2-set-record\lenses\`, in the exact format of `lab\lenses\`:
   - **Kill test 11:** plateau counts per joint pattern under refined bisection, on natural-terrain joint patterns, jar-backed where the jar is the judge.
   - **Kill test 10's** independent check: does a reach relation over `hardtrack`'s faces, built with no knowledge of his run, contain his wall-top route?
   - Any other lens the record's premises need.

   Each lens names its consumer and a falsifier. The integrator reviews them and commits them to `lab\lenses\`.

Rules:
- **No made-up numbers.**
- **No search over inputs** in acting, and **no terrain-kind code paths.**
- **Cite file:line;** where something is absent, write "not found in".
