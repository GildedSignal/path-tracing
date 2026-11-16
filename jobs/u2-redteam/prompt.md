# Red-team: joints and sets (step 2 of docs/plan.md), before it freezes (a one-way door)

You are the Opus Designer, red-teaming in a fresh context (AGENTS.md: the Designer "red-teams each one-way door before it freezes"; S174: "opus and astra for important things"). You read, run read-only commands and small offline experiments, and write only your report. You do not build, edit code or docs, commit, or launch any game, server or client.

**Why:** every later step reads the set record: the loop (U3), the runner (U4), the kit's verbs (U5), the far map (U6). A wrong key or a wrong plateau rule is paid by everything after it. Julien (S161): "it should be the same mechanism". Kill tests 10 (the map holds his 428 shortcut) and 11 (plateaus are few) are in `docs\learning.md` section 6.

## Read (root `C:\Users\PASCAL\Desktop\path tracing\`)
- **The draft under attack:** `jobs\u2-set-record\report.md`, all of it. Section 3 lists the one-way-door choices.
- **The plan:** `docs\plan.md` (step 2, joints and sets, in plain names).
- **The approved plan:**
  - `docs\learning.md`: sections 0, 2 and 6;
  - `docs\representation.md`: sections 10 and 11;
  - `docs\scales.md`;
  - `docs\vocabulary.md`;
  - `jobs\unified-plan\plan.md`.
- **Frazzoli:** section 3.2 of `jobs\frazzoli\report.md`.
- **Evidence:**
  - `jobs\hard-track\shortcut.md` and `julien-428.jsonl`;
  - branch `wt/hard-track-2` (`D:\path-tracing-wt\hard-track-2\engine\plan\tests\hard_track.rs`, `jobs\hard-track-2\report.md`);
  - `lab\results\*trial-offset-monotone*\check.md`.
- **Code:** `engine\plan\src\` and `engine\law\src\` on main.

## Attack
For each one-way-door choice in the draft's section 3, and anything else you find:
1. Does it make one terrain kind take a different code path? That is the one-mechanism test.
2. Does it smuggle in a tolerance, an epsilon, or a made-up number?
3. Does it reintroduce search over inputs in acting? Search over exact states belongs to learning (`docs\learning.md` section 1).
4. Is the key right? Can two situations with different futures get the same key, which is unsound, or can identical situations get different keys, which wastes the store?
5. Is invalidation exact? Can an edit change a plateau without touching its footprint?
6. Does it hold his 428 shortcut? Would the reach relation built from it contain the wall-top route? Run a small offline check against the old hard-track build if you can.
7. Fermi: do the stated floors hold, and with which inputs?

## Write `jobs\u2-redteam\report.md`
- **First line:** `FREEZE`, `FREEZE AFTER FIXES` or `REDESIGN`.
- **Findings by severity,** each with file:line and the smallest fix.
- **The decisions a builder needs closed,** each with your recommendation.

No made-up numbers; cite everything. End your final message with `STATUS: DONE`.

