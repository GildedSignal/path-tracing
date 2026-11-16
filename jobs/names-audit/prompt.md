# Plain names: every code name in the docs, and what it should say instead

**Julien (S179, verbatim):** "what's up with those made u1 p3 and so on that means nothing say names i'm over those made up names just say what they are. you make up those things and anywhere i look in the doc it's not even explained".

**Consumer:** the integrator's housekeeping pass, which replaces code names in the living docs with plain names.

You read only, and write only your report. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
`AGENTS.md`, and every file in `docs\` except `docs\reference\`. That folder is Julien's verbatim words and is never edited. Also read `jobs\unified-plan\plan.md`, so you know what each step is.

## Write `jobs\names-audit\report.md`
1. **Every code name** in those files, with file:line for each occurrence. That covers step and slice codes (U0–U9, P1–P4, T1–T11, slices A–K, R0–R6), rung codes (R-brach, R-lego, R-far), test numbers used as names, phase numbers like 3.2d or 1.6g, job ids used as if the reader knew them, and any other label a newcomer could not decode.

   His turn keys (S6, X2 and so on) are citations, not names. Leave them alone.
2. **For each code name:** what it is, in at most eight plain words, and the plain name to use instead. For example, U1 becomes "the core: word store, counts and events", and P2 becomes "the exact integer maps test".
3. **For each doc:** the replacement text, line by line, ready to apply. Where a code must stay as an identifier (a branch, a job folder), keep it in backticks after the plain name, never alone.
4. **Every term the docs use without defining it.** Say where a definition should go.

Do not edit anything; the integrator applies the changes.
