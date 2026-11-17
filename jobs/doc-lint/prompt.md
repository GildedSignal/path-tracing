# Lint our docs against his words

**Julien (S211, verbatim):** "for the linting i meant our own docs should be lintet based on what i said looking for issues from time to time."

**Why it matters:** on 16 November the plan forked into an "easy" path and a "hard" path. Nobody had re-read his words, and the docs had drifted from them for two days (`docs\reference\postmortem-2026-11-16.md`). This lint exists to catch that kind of drift early. Your first corpus pass (`jobs\corpus-contradictions\`) found exactly such things; this is the same job, aimed at **his words** as the reference.

You read only, and write only your report. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## The reference: his words
- `docs\reference\julien-2026-11-04.md`, `julien-2026-11-09.md` and `julien-2026-11-10.md`: his turns, verbatim. The last runs to S211 (17 November). Read all of them.
- `docs\PRINCIPLES.md`: the goal, his philosophy, and the register (each statement, its status and where it landed).
- Not `docs\reference\private.md`: it is private, and out of scope.

## What is linted: the living docs
`AGENTS.md`, `docs\PRINCIPLES.md` (its register and contradictions table), `docs\plan.md`, `docs\HANDOFF.md`, `docs\learning.md`, `docs\representation.md`, `docs\scales.md`, `docs\vocabulary.md`, `docs\engine.md`, `docs\clan.md`, `docs\ladder.md`, `docs\server.md`, `docs\footprints.md`, `lab\glm.md`.

## What to look for
1. **Drift:** a doc that says or implies something his words contradict or never said. Quote both.
2. **His rules not written down,** or written only in the register: a rule he stated ("a smaller model never writes for a bigger one", "explain means explain", "no patches") that a doc a model reads before acting doesn't carry.
3. **The register's statuses:** a row marked Followed or Adopted whose "where it landed" doesn't hold it. Turns with no row. Rows that reword him instead of quoting him.
4. **Stale claims:** a fact about the build or the clan that today's docs or his later words supersede, for example "cannot run commands", old meter figures, retired roles, names the plan no longer uses.
5. **Made-up names without plain names** (S179: "say names i'm over those made up names").
6. **Requests he made twice with no owner** in HANDOFF's tracked requests.
7. **Contradictions between docs.**
8. **Budgets:** HANDOFF ≤ 120 lines, each spec in `docs\` ≤ 200 lines, AGENTS.md ≤ 60 lines. Also garbled characters (mojibake such as `Ã`, `â€`).

## Write `jobs\doc-lint\report.md`
- **Findings, most consequential first.** Each has:
  - the doc and line;
  - what it says;
  - his words, with their key (S-number) and file;
  - why it's an issue;
  - the smallest fix, as an exact replacement where possible.
- **Findings you are unsure of,** marked as such.
- **What you checked and found clean,** one line per doc.

Never reword his quotes. Never guess a line number. Write the report as you go.
