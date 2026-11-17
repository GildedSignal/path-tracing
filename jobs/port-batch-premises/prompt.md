# Port batches: the exact edits, checked against the code

**Consumer:** the integrator, who writes Terra's prompts for the three port batches in `jobs\port-digest\report.md` section 4 (player movement and resources; the groundwork for editing the world; block shapes when placing), plus the small hook-call follow-up. Specs are the integrator's (S175). Your job is the premise check and the exact edit list, so that the prompts leave nothing open.

You read only, and write only your report. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- `jobs\port-digest\report.md`, all of it, and the ten wiki pages it wrote: `jobs\port-digest\wiki\*.md`, now in `lab\wiki\`.
- The result folders it cites: `lab\results\local-20261117-*`. Read each `dossier.md` and `check.md`; the checker wins.
- The engine code on main: `engine\law\src\*.rs`, `engine\world\src\snapshot.rs`, `engine\world\tools\ModelTable.java`.
- `docs\plan.md`: steps 5 and 8.

## Write `jobs\port-batch-premises\report.md`
For each batch (and the hook follow-up):
1. **Premises:** every file, function and line the batch touches. Mark each EXISTS, DIFFERS or MISSING on main, with the evidence.
2. **The exact edits:** for each change, the file, the function, the current lines, and what the jar says they must become (class, method, offsets, from the dossier). Never write code. State what changes, precisely.
3. **The test that proves each edit bit-exact:** name it, its inputs, and the jar fact it checks. Prefer a recorded run where one exists (`lab\tracks\runs\`, the L0 kept runs).
4. **Conflicts:** any pair of batches that would touch the same file, and any edit that would change a frozen gate (the footprint mutants, L0). Say which batch must go first.
5. **Open questions** the integrator must close, each with the options and your recommendation.

Cite file:line for every claim. Where something is absent, write "not found in" with the files checked.
