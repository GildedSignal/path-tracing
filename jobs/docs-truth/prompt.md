# Docs truth: every claim in the main docs against the code and today's state

**Consumer:** the phase-end cleanup (AGENTS.md: "make the docs true"; "truth is rewritten in place"). The last pass was 14 November. Since then:
- the zones design and its red-team;
- Astra's 3.2d-3 spec, peaceful only;
- slices A and B on the integration branch;
- the hard track, built in Julien's world;
- the plain-path FAIL;
- new GLM results and the Hydrogen audits.

You read only, and write only your report. Read every file by its absolute path.

## Read

- **The docs:** `C:\Users\PASCAL\Desktop\path tracing\docs\HANDOFF.md`, `engine.md`, `representation.md`, `scales.md`, `vocabulary.md`, `learning.md`, `footprints.md`, `ladder.md`, `server.md`, `clan.md`, and `C:\Users\PASCAL\Desktop\path tracing\AGENTS.md`.
- **The code:**
  - main: `C:\Users\PASCAL\Desktop\path tracing\engine\*\src\*.rs`, `bot\src\pathtracing\*.java`, `scripts\*.ps1`;
  - the integration branch: `D:\path-tracing-wt\3.2d-3\engine\` and `D:\path-tracing-wt\3.2d-3\bot\`.
- **Today's records:**
  - `C:\Users\PASCAL\Desktop\path tracing\jobs\inventory-map\report.md` (the mechanism map);
  - `jobs\zones-design\design.md` and `redteam-opus.md`;
  - `jobs\3.2d-3\spec.md`;
  - `jobs\3.2d-3-A-judge\last.md` and `jobs\3.2d-3-B-judge\last.md`;
  - `jobs\hard-track\report.md` (on main);
  - `jobs\port-backlog\report.md`;
  - `jobs\spec-premises\report.md`.

## Write

`C:\Users\PASCAL\Desktop\path tracing\jobs\docs-truth\report.md`, appending as you go.

1. **Per doc:** every statement that is now false, stale, or contradicted by code or by a later decision. Give:
   - the doc's file and line, quoted;
   - the evidence (code file and line, or the record that supersedes it);
   - replacement text, in the doc's own plain style.
2. **Budgets:** HANDOFF ≤ 120 lines, each spec in `docs/` ≤ 200 lines, AGENTS.md ≤ 60. Report each file's line count. Where a file is over its budget, propose what to cut or move.
3. **Copies instead of pointers:** places where two docs state the same fact. Propose which one keeps it and which one points to it.
4. **Never touch Julien's words:** the register in PRINCIPLES.md and `docs\reference\` are his verbatim turns and are not edited. Report only where a doc misquotes him.

Cite every claim by file and line. Never guess: write "not found in" with the files checked.
