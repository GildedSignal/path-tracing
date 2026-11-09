# Job lab-wiki: the GLM lab's memory

**Why.** GLM is now our main external builder, and its 5-hour token window binds: about 18 lanes per window (lab/glm.md). Each lane spends a good share of its tokens rediscovering the setup and earlier findings, and lanes re-ask settled questions. ES's lab keeps a wiki compiled from its runs and read before any new dossier. S59 adopted it here (lab/glm.md, "The lab's memory: the wiki").

**Branch.** `wt/lab-wiki`, worktree `D:\path-tracing-wt\lab-wiki`, cut from `main`. The run folders to read are in the main checkout: `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-*`. They're git-ignored and read-only for you.

## Build

1. **A compiler,** `scripts/lab-wiki.ps1` (Windows PowerShell 5.1, like the other scripts). It reads every run folder under a results directory, given as a parameter, and writes `lab/wiki/`:
   - **One page per lens,** `lab/wiki/<lens>.md`, entries newest first. Each entry has:
     - the run id and date;
     - the grade;
     - the claim (the dossier's `Claim:` line, cut to about 300 characters);
     - the falsifier, if the dossier states one;
     - the checker's key discrepancy, if any;
     - the run folder's name.
   - **Grades come from `check.md`'s `Grade:` line.** A checker or author that died on HTTP 429 (look for `429` and `rate_limit` in `checker.log` and `author.log`) is graded **rate-limited**, never "lead". A missing dossier is **no dossier**.
   - **`lab/wiki/index.md`:** one line per lens, with counts per grade and the last run date, then a short list of the reproduced results, newest first, one line each. Keep it under about 100 lines.
   - **Deterministic output:** running it twice gives the same files. It never copies anything that looks like a key or token. The runner already deletes files containing the GLM key; check again and refuse to write a line that matches the stored key pattern.
2. **The lanes read it.** In `lab/run.sh`, the author and checker prompts tell the model to read `lab/wiki/index.md` and its own lens's page first, not to re-ask a question already graded reproduced or falsified, and to build on the leads.
3. **It runs after each batch.** `scripts/glm-local.ps1` calls the compiler on `lab\results` when the batch ends. The integrator commits `lab/wiki/`, which is not ignored.

## Test, fast

- Run the compiler on the existing `local-20261109-*` folders. The 1653 batch's checkers that died on 429 must come out **rate-limited**. The 1554 batch has six reproduced results; they must appear in the index.
- A tiny fixture (two fake run folders under `lab/wiki/tests/`) checks the grade rules and that output is stable. One script, one question each.

## Must not

- Touch `engine/`, `bot/`, `docs/`, the gates, or any run folder.
- Start a GLM run (it would spend the window).
- Push or merge.

## Done means

The compiled `lab/wiki/` from today's runs (index plus pages) is committed on the branch, along with the script changes and the fixture test. The final message gives the index's first 20 lines and the counts per grade.
