# Spec fixes: correct the 3.2d-3 spec where its premises failed, now that slices A and B exist

**Consumer:** Terra's next jobs on the integration branch `wt/3.2d-3`, after Codex resets at 21:31:
- P1 (T1), then P2–P4;
- slices C to K.

Without corrections, Terra halts at the first false premise.

You read only. Read every file by its absolute path.

## Read

- **The spec:** `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-3\spec.md`.
- **The premise audit:** `C:\Users\PASCAL\Desktop\path tracing\jobs\spec-premises\report.md` (35 claims: 10 exist, 17 differ, 8 missing). It audited main *before* slices A and B were built.
- **Slices A and B,** now merged on the integration branch, worktree `D:\path-tracing-wt\3.2d-3`:
  - their reports: `D:\path-tracing-wt\r0-channel\jobs\3.2d-3-A\last.md` and `D:\path-tracing-wt\r1-codec\jobs\3.2d-3-B\last.md`;
  - their judge verdicts: `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-3-A-judge\last.md` and `...\3.2d-3-B-judge\last.md`;
  - the code itself under `D:\path-tracing-wt\3.2d-3\engine\` (`native`, `law/src/codec*.rs`) and `D:\path-tracing-wt\3.2d-3\bot\src\pathtracing\`.
- **The plain path:** `D:\path-tracing-wt\plain\engine\plan\src\plain.rs` (unmerged, judged FAIL), which T1 is specified to run against.

## Write

`C:\Users\PASCAL\Desktop\path tracing\jobs\spec-fixes\report.md`, appending as you go.

1. **For each of the audit's DIFFERS and MISSING claims:** is it now satisfied by A or B (with file and line on the integration branch), still missing, or moot? For each still open, give the smallest spec correction as replacement text for the spec line, and the slice it unblocks.
2. **For T1 (P1) specifically:** which code it should run against. The plain path is not on the integration branch. Give a complete, decision-free job spec for Terra's T1 premise job on `wt/3.2d-3`: inputs, the exact test to write, the gate, and what it must not touch. Terra halts at the first open question, so leave none open.
3. **The same for P2, P3 and P4,** one decision-free job spec each. Mark which can run in parallel.
4. **The judges' findings** that a later slice must carry: A's unmeasured per-block chunk strings on the client thread, its playback cancelling on echoed packets, B's byte clones in the hot path, the shared 16-bit id space. Say which slice owns each.

Cite every claim by file and line. Never guess: write "not found in" with the files checked.
