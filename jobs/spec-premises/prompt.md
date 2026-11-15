# Spec premises: what the 3.2d-3 spec says already exists, checked against the code

**Consumer:** Terra's slices of the 3.2d-3 spec (`D:\path-tracing-wt\sets\jobs\3.2d-3\last.md`). The post-mortem's rule 3 (`C:\Users\PASCAL\Desktop\path tracing\docs\reference\postmortem-2026-11-14.md`): test each spec's load-bearing premise before a builder starts. Most blocked runs on 14 November came from untested premises.

One premise has already failed: the spec reuses "the clock/comparator native module", and no JNI or native module exists anywhere in the repo.

You read only. Read every file by its absolute path.

## Read

- **The spec,** in full: `D:\path-tracing-wt\sets\jobs\3.2d-3\last.md`.
- **The code it names:**
  - `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\*.rs`
  - `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\*.rs`
  - `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\*.rs` and `engine\plan\src\bin\*.rs`
  - `C:\Users\PASCAL\Desktop\path tracing\engine\world\src\*.rs`
  - `C:\Users\PASCAL\Desktop\path tracing\bot\src\pathtracing\*.java` and `bot\src\pathtracing\mixin\*.java`
  - the unmerged plain path: `D:\path-tracing-wt\plain\engine\plan\src\plain.rs`
  - the scripts under `C:\Users\PASCAL\Desktop\path tracing\scripts\`
  - the jobs it cites under `C:\Users\PASCAL\Desktop\path tracing\jobs\`

## For every claim in the spec that something exists

Look for words like existing, reuse, current, the same, keep, already, remains. Give:
- **the spec's line and its words;**
- **EXISTS**, with the file and line, and whether it does what the spec assumes;
- **DIFFERS**, with what it does instead, and the file and line;
- **MISSING**, with the files checked;
- **which slice depends on it**, from section 7's table (A to K, P1 to P4).

Examples to check, among all the others:
- "the existing arc/read certificate";
- "the existing F CPU worker";
- "section role planes … AllZero/AllOne/Words";
- "the existing word predicates/keys";
- "the shared deviation.rs comparator" and `compare_event`;
- "the existing packet hooks";
- `Goal::touched`;
- `words::recurrence`;
- "jobs/3.2d-3-oracle";
- `verify.ps1` and its cases;
- "the existing same-host QPC clock contract";
- "the existing native regression gate";
- "pattern::in_scope";
- "the reverse footprint index";
- "the frozen baseline bundle".

## Output

Write `C:\Users\PASCAL\Desktop\path tracing\jobs\spec-premises\report.md`, appending as you go. Its parts:

1. **A summary:** counts of EXISTS, DIFFERS and MISSING, and per slice whether it can start as written.
2. **The table.**
3. **For each MISSING or DIFFERS:** the smallest correction to the spec.

Cite every claim by file and line. Never guess: write "not found in" with the files checked.
