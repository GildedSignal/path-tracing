# Post-mortem: companion usage, 14 November 2026 (S128)

Asked for by Julien (S128): "make a post-mortem on sub-agent usage so that next time we do better with less fuss and less issues and more efficient". The day ran about 30 companion jobs across Terra, Astra, Argon, Hydrogen, GLM, and native Opus and Sonnet. This lists what cost runs, meter or his attention, the fix already in place, and the rules that follow. The rules are in [clan.md](../clan.md) section 5c; this file is the evidence.

## What went well (keep)

- **Other-family review paid every time.** The native Sonnet judge on 1.3d was precise (PASS WITH FINDINGS, one open point that GLM then settled). The Opus red-team of Astra's spec found 13 real faults, including that the first flight could not pass by construction and that a whole section was a second copy of the law.
- **Argon's two reads, the ES audit and the butoon diagnosis, changed the design.** Each was cited by file and line and checked by the integrator in one command before use.
- **Evidence made by the integrator for read-only companions** (block dumps from `world-probe`, the planner's failure lines) let Hydrogen diagnose tracks it cannot open.
- **GLM's lab** reproduced everything it was given (13 lenses today), with an independent checker.

## What cost us, by cause

| # | Incident | Cost | Fix in place |
|---|---|---|---|
| 1 | **Antigravity runs ended empty** after one denied file read (twice), then with a one-line stub | ~3 Argon runs ≈ 30% of its weekly limit | permissions grant the `%20` path form; the report is appended every 3–4 files read (`gemini.ps1`) |
| 2 | **Opus on Antigravity** produced nothing and spent the Claude+GPT meter for the week | the week's Claude meter | Antigravity is Sonnet only (S115); Opus runs natively |
| 3 | **Terra stopped after minutes** with "incomplete" | several relaunches | a status contract plus resumes in the same session (`gpt.ps1`) |
| 4 | **Resumes ran into a spent meter**, four times | four wasted starts | resumes stop on "usage limit" |
| 5 | **The contract over-restricted reading** ("read only the files the spec names") and blocked 3.3h at once | one run, one slot-hour | the contract now limits documentation, never the code a job changes |
| 6 | **A spec premise was never tested**: 3.3h's "replay until bits rejoin" can't happen after an aim change (position shifts for good) | one Terra run | rule 3 below |
| 7 | **Scheduled launches and the GLM runner died** with the session at compaction or a limit | three launches missed, GLM idle 13:00–15:53 | everything that must outlive a turn is a detached process (`launch.ps1`, `follow-up.ps1`, timed `Start-Process`) |
| 8 | **Slot counting raced**: counting `codex.exe` missed jobs between resume runs, so 1.3d ran beside Astra and 3.3h | over the slot rule | `launch.ps1` counts job scripts |
| 9 | **Testing that fix launched a real duplicate job** | seconds of a run, an overwritten log | `launch.ps1 -DryRun`; scripts are tested dry, never live |
| 10 | **Shared cargo target** across worktrees: builds overwrote each other's binaries | confusion, rebuilds | `gpt.ps1` gives each job its own `CARGO_TARGET_DIR` |
| 11 | **The fixture world is git-ignored**: new worktrees and main lacked refreshed regions, so tests failed at merge | three hand copies, one failed merge test | `launch.ps1` copies the fixture into every new worktree |
| 12 | **Meters misread or assumed**: "unknown until spent" (false), which Antigravity column is used vs remaining, Zai having no weekly, Argon's limit being its own | his time, twice ("i don't believe you", "are you sure") | rule 5 below; `meters.ps1` reads `agy /usage` raw |
| 13 | **GLM paced by guess**: first ~1.5 lanes (runner bug), then 3 lanes on track to hit the cap 2 h early | idle GLM, then risk of capped lanes | `glm-queue.ps1` paces 1–8 lanes to the meter and is restart-safe |
| 14 | **Hydrogen idle all day** while Argon's small weekly limit was spent | about half of Argon's week | Hydrogen first for reading (rule 4); up to three Hydrogen lanes beside Argon |
| 15 | **Live gate refusals from load bursts** (L0 clock 994, then 919 TPS), and a reused evidence folder | three L0 attempts | a fresh evidence folder per run and a CPU sampler beside it; see rule 7 |
| 16 | **The integrator spent its own context debugging scripts** (PowerShell quoting, .NET paths, regexes) | context and time | rule 8 |

## The rules that follow (now in clan.md 5c)

1. **Every launch goes through `launch.ps1`**, detached, Idle priority. Check with `-DryRun` first when anything about the launcher changed.
2. **Every job writes as it goes and ends with a status.** GPT: `STATUS: DONE/BLOCKED`, resumed in-session otherwise. Antigravity: `report.md` appended every few files. A job that ends empty is a launcher fault: fix the launcher before rerunning.
3. **Test each spec's load-bearing premise with one command before launch.** If the spec says "X happens" (bits rejoin, a file exists, a meter has room), run the command that shows it. Most of today's blocked runs were untested premises.
4. **Route by meter and need, big models first** (S113, S114): reading goes to Hydrogen first, with Argon only for the hardest reads (its weekly limit is ~10 runs). Design goes to Astra; building to Terra; red-teams to native Opus; judges to native Sonnet; lab truth to GLM, paced by `glm-queue`. Antigravity's Claude only for Sonnet.
5. **Read meters raw before stating them** (`meters.ps1`, `agy /usage`, `omp usage`), and say which column means what. Never "unknown"; never a guess.
6. **Read-only companions get their evidence made for them**: failure lines, block dumps, logs, inlined or as files they can read. They cannot run commands.
7. **Live gates**: only when the machine is free (S126 games), a fresh evidence folder, the gate's Java at High priority, a CPU sampler beside it. A refusal names its cause before any rerun.
8. **The integrator orchestrates.** Script bugs that take more than a couple of commands go to a Terra job with a spec, not into the integrator's context.

## Rough cost per need (anchors, 14 November; S113)

| Need | Who | Meter cost |
|---|---|---|
| a code read or audit (~300–450k in) | Argon | ~10% of its weekly limit |
| the same read | Hydrogen | ~0–1% of the Gemini weekly |
| a build step | Terra | ~5–8% of Codex 5 h |
| a design | Astra | ~25–30% of Codex 5 h |
| a lens | GLM | ~7–10% of Zai 5 h per lane-hour |
| a red-team or a judge | native Opus / Sonnet | ~80–150k tokens of the native plan |

## The guards added after it (S129)

Julien: these failures "should NOT happen again, there should be tests". Now:
- **scripts/test-antigravity.ps1** must pass before any Antigravity job: every launcher script parses; Opus is refused by `launch.ps1` and by `gemini.ps1` (checked by message, since a parse error also throws); a live Hydrogen-low probe (~20k tokens, under 1 point of the Gemini 5 h meter) reads by six path forms with no denial. Its first runs reproduced the empty-run failure in 11 s: **a relative path or a `file:///` link is denied and ends the run**. The contract now requires absolute drive-letter paths, and the probe passes.
- **`launch.ps1`** refuses every Antigravity job until the probe's stamp matches the current `gemini.ps1` and `launch.ps1`.
- **`gemini.ps1`** refuses to start a run its meter cannot afford (per-model budget: Argon 15, Hydrogen 15, Sonnet 10 points of remaining), and every two minutes kills a run that has taken more than its budget or whose report has not grown for 20 minutes, keeping what the report holds. The kill path has not yet fired on a live run.
