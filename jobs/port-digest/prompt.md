# Overnight port digest: what GLM settled, checked, and what it unblocks

**Consumer:** the integrator, in the morning, and `lab\wiki\`, the lab's settled index. GLM is not strong (S173), so every result is read against its independent checker before it counts.

You read only, and write only your report and wiki drafts. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- `jobs\port-lenses\report.md` and the lenses it lists: tonight's questions.
- Every folder under `lab\results\` whose name ends in one of those lens names and holds a `DONE` file. In each, read `dossier.md` and `check.md`. The checker's grade wins where it disagrees with the author.
- `lab\wiki\index.md`, two existing wiki pages for their format, and `docs\plan.md` (steps 5 and 8).
- The engine code each result touches (`engine\law\src\`), to say what is already ported.

## Write `jobs\port-digest\report.md`
1. **Per result:**
   - the lens;
   - the checker's grade;
   - the settled fact in one or two lines, with jar class and offsets;
   - whether the engine has it already (file:line), has it partly, or lacks it;
   - the plan step or track it unblocks;
   - the port delta a builder would write: file, function, what changes, and the test that would prove it bit-exact.
2. **Results that failed or were refuted:** what went wrong, and whether to re-ask a narrower question.
3. **Wiki drafts** for each settled result, in `jobs\port-digest\wiki\`, in the format of `lab\wiki\`. The integrator copies them over.
4. **The morning's port batch:** the deltas ready for a builder, grouped by file so that no two jobs touch the same file.

Cite every claim by file and line. Never guess.
