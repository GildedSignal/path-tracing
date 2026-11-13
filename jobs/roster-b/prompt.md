# Job roster-b: the job-attempt census, finished

**Why.** Julien has added a Google AI Pro plan; Argon and Hydrogen join the clan. `docs/clan.md` assigns every job by band, and its rows are claims. Our own job record is the only primary source with skin in the game. Your earlier run (`jobs/roster/last.md`) finished the **GLM census** — 106 lanes, 72 reproduced, 32 leads, 13 of them with a recorded 429 — and left the **job-attempt census** unfinished. This job finishes exactly that, and re-does nothing.

**Consumer.** `docs/clan.md` sections 3 and 4b, which decide who gets every job from here on.

**You have no network.** Everything is on disk.

**Paths.** Your worktree holds only the branch. Everything below is in the **main checkout**, and you must open it by its absolute path, because it is not in your worktree:
`C:\Users\PASCAL\Desktop\path tracing\` — in particular `jobs\`, `docs\HANDOFF.md`, `docs\clan.md`, `lab\results\`, `AGENTS.md`.
Your previous run's own report is at `C:\Users\PASCAL\Desktop\path tracing\jobs\roster\last.md`, and the census you already wrote is at `D:\path-tracing-wt\roster\docs\research\roster-evidence.md` with `D:\path-tracing-wt\roster\scripts\roster-census.ps1`. **Copy both into your worktree and extend them. Do not rewrite the GLM half.**

**Branch.** `D:\path-tracing-wt\roster-b` on `wt/roster-b`.

**Build. One table, every row cited.** For every `jobs\<id>\` that has a `prompt.md`, one row:

| column | how it is filled | what is forbidden |
|---|---|---|
| job id | the folder name | — |
| model | from the prompt, the `Built-by:` trailer in `git log --format='%H%n%an%n%s%n%b' --all`, or HANDOFF's text | `unknown` if the record does not say it — **never infer from style, quality or tone** |
| shape | one of `tight-spec-build`, `open-ended-build`, `measurement-probe`, `judge`, `jar-read`, `doc-or-spec`, `port-delta` | inventing a shape |
| outcome | one of `done`, `done-with-findings`, `incomplete`, `stopped-on-missing-input`, `judged-FAIL` | grading it yourself |
| net lines | from the diff of the commit that carries it, `+N/-M`; `0` where nothing was committed | guessing |
| what stopped it | **one sentence quoted verbatim** from `last.md` | paraphrase |

Then three short sections:

1. **Where the record separates two models.** Name the job ids on both sides. State for each whether it is a **result** (the difference repeats across at least three attempts of the same shape) or a **lead** (seen once or twice). Your earlier run found three candidates — carry them forward with that label applied.
2. **Where it cannot separate them.** List every pair the record does not distinguish. This section is as valuable as the one above: a pair that lands here is decided by cost, not by skill, and saying so saves us from inventing a reason.
3. **The confounds, by name.** For each `stopped-on-missing-input` and each `incomplete`, say in one line whether the record shows the **spec** was at fault, the **inputs** were missing, or the **model** fell short — and `not determined` where it does not show. A model judged on a bad spec is the most expensive mistake this census can cause.

**The one rule.** Every claim cites a file by path. A claim you cannot cite does not go in. "The record does not say" is a correct answer and costs one line.

**Must not.** Rank by reputation or by anything you know about these models from outside this repo. Re-run or re-grade the GLM lanes. Touch `engine/`, `bot/`, `lab/`, or any `jobs/` file. Commit, push or merge.

**Done means.** `docs/research/roster-evidence.md` under 200 lines holding both halves; `scripts/roster-census.ps1` runs under Windows PowerShell 5.1 from the repo root and reprints every number in it; and a final message giving the row count, the count of `unknown` model rows, the results-versus-leads split, and the pairs the record cannot separate.
