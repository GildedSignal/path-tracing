# The clan: who does what, and why

The team table in [work.md](../work.md) says who holds each role. This file says **how a job is assigned**, and it is the file that changes when a companion joins or a measurement moves someone.

Julien, S10: "plan for the whole build with the appropriate sub-agents doing the right things with the right intellect tackling things in the right order." S12: GPT "tends to be good with well formed tasks while you do better when taste is required."

## 1. The law of assignment: who catches the error

A companion is not assigned by how smart it is. It is assigned by **what catches it when it is wrong** (ES 2: error type matters more than error size; SR-71: seal only what can't be corrected).

Every job falls into one of three bands.

| Band | What catches a wrong answer | What that buys | What it costs when we get it wrong |
|---|---|---|---|
| **Checked** | a machine: a bit-exact gate, a witness red-then-green, an independent reproduction | the cheapest capable mind wins, and volume beats depth — a wrong answer is nearly free, because it is caught | little, so run many lanes |
| **Judged** | another mind, adversarially, in a fresh context, from a different family | decorrelated error: the judge is the only check, so it must not share the builder's blind spot | an undetected FAIL merges, and everything downstream rests on it |
| **Frozen** | nothing, later | only taste pays here: a representation, a spec before its first run, a gate's hash | the whole structure above it, re-derived (AGENTS, "No patches") |

So: **Checked work goes to whoever is cheapest and fast enough. Judged work goes to a family the builder is not in. Frozen work goes to the best mind we have, and is red-teamed before it freezes.** Cost decides inside a band, never across one.

A fourth thing exists and is not a band: **orchestration**, one mind holding the whole context. It is frozen work that never stops being frozen, so it does not rotate.

## 2. What a third family buys

The clan has two families that build and judge, Anthropic and OpenAI, plus GLM's lab, with Mistral (Vibe) beside it; Google is Julien's own tool (Antigravity, piloted by him). A third judging family would buy, beyond its tokens:

1. **The judge can be chosen, not conceded.** For each builder, the judge is the family whose errors are least like the builder's. Which family that is, is a measurement, not an opinion (section 4).
2. **What a machine cannot check can be put to three.** Two minds that disagree leave the integrator to break the tie by taste; three that disagree say the question is underspecified, which is information. Used only where no machine can check, because where one can, a vote is fake work.
3. **A blind spot shared by two families stops being invisible.** What any two vendors' training distributions get wrong together is unmeasured.

## 3. The companions and routing

Names are the vendors' own. Every row is a claim, and section 4 is what turns a claim into evidence or removes it. Launch and meter commands are in [work.md](../work.md).

| Companion | Family | Band claimed | Claim and launch notes |
|---|---|---|---|
| Opus | Anthropic | Frozen, Judged | structural ports (`wt/port-state-spine`), one-way-door audits, red-teams before freezing. Runs headless or as sub-agent |
| Fable | Anthropic | Orchestration | holds the whole project beside Julien; develops core planner/representation directly with him and routes porting lanes |
| Sonnet | Anthropic | Judged | adversarial reading of a diff against a spec, fresh context. Native sub-agent only (batch judges into one) |
| Haiku | Anthropic | Checked | runs, log digests, completeness |
| Astra | OpenAI | Frozen (convergent), the core | the core and the important builds, a bound, a monotonicity, a closed gap; the Codex window is hers (S174, S209). Effort pinned high; resumes itself until `STATUS: DONE` or `BLOCKED`; commits are the integrator's with `Built-by:`. Copy git-ignored `engine/world/tests/fixtures/world` into a new worktree |
| Terra | OpenAI | Checked (build) | a spec that leaves nothing open, built exactly, unattended; backfills the leftover ~20% of the Codex window (S209). ≤ 4 GPT; with Astra, one GPT job beside her (S15); same launch rules as Astra |
| GLM | Z.ai | Checked (wide) | parameter table rows, Mojang scars, and jar bit-witnesses against the generator basis, plus compiled `lab/wiki/` notes ([lab/glm.md](../../lab/glm.md)); no second checker session — judged by jar witnesses, the integrator, and live flights. `.\scripts\glm-local.ps1 -Lenses "<job-id>"` (reads `jobs\<id>\prompt.md`): commit the prompt first, because lanes run from a checkout of HEAD |
| Vibe | Mistral | none yet: the Checked band's bake-off first (section 4) | the same lane as GLM (table rows, scars, jar witnesses), a second family beside it whose rows check GLM's; runs through the same lane runner: `.\scripts\glm-local.ps1 -Model vibe` runs Mistral's Vibe CLI headless (`D:\path-tracing-deps\vibe`, its model `mistral-vibe-cli-with-tools` priced 0 per call), `-Model mistral/mistral-large-4` runs omp on the API (key stored with `scripts\set-key.ps1 -Provider mistral`). His Pro plan includes, each calendar month, €25.5 of API use (what omp spends) and €255 of Vibe & Studio use; pay-as-you-go is off, so use stops at the cap and costs nothing more |

## 4. The bake-off: no row stands on reputation

A companion's row is a hypothesis, settled the way everything here is settled — against a known answer, by a check we already trust (`lab/glm.md`, "Canaries").

**The protocol.** Give the newcomer a job **we have already done**, whose answer is on disk and whose grader is a script, and grade it blind.

- **Checked band:** a lens from `lab/queue.txt` that GLM has already reproduced. Grade: did it reproduce, in what wall time, at what cost, and did its own independent checker agree? A newcomer that produces a confident wrong result its own checker passes is worse than useless, and is refused the band.
- **Judged band:** a diff we have already judged, including at least one whose verdict was **FAIL**, with the verdict withheld. Grade both halves: did it find the blocking finding, and did it invent blocking findings that are not there? A judge that fails everything is not a judge.
- **Frozen band:** a spec we have already red-teamed, with the red-team's report withheld. Grade: how many of the known changes it finds, and whether it finds one we missed.
- **Orchestration support:** a known contradiction in the corpus. Grade: found, with both files named, or not found.

**Canaries both ways.** Some graded items are subtly wrong on purpose. A companion that passes a planted wrong one does not get its band, whatever it scored elsewhere.

**The cost column is not optional.** Two companions within noise of each other are the same companion, and the cheaper one takes the work (ES 27: the goal, never a proxy).

**But the currency is window-hours, not dollars** (S102). Every family here is a flat subscription: we spend *a meter*, and an unspent meter is worth nothing when it resets. What matters is which meter the work lands on. `.\scripts\meters.ps1` reads every meter, so re-anchor whenever a number surprises you.

Windows: Zai 5 h; Codex 5 h and weekly; the native plan 5 h, weekly, and weekly per model.
**So: inside a band, spend the least contended meter; the native plan runs the integrator and is spent last. On a shared meter, big models first** (S114): unspent capacity is lost at reset, and a small model cut off mid-task costs far less than a big one. Launch Astra (or a native Opus) at the start of a window, and let Terra and the lanes fill what is left. "Fill the Anthropic context" means load the integrator with context, not drain its quota (S102).

**Window discipline:**
1. **Close every design question before a builder starts.** Open classification or representation questions belong to Designer or integrator work first.
2. **One deliverable per job, the files to read named, small ones inlined.** Read only what the spec names.
3. **A long job writes its output to a file as it goes** so a report survives a quota death.
4. **Measure with a command, not a job.** Flights, scaling and timings run as plain commands at no meter cost; jobs are for building.
5. **Never resume into a spent meter** (`gpt.ps1` stops on "usage limit"), and never launch a lane whose meter `meters.ps1` shows above 90%.
6. **Lanes run at Normal priority** (S158, "no need for low priority"); lower them to Idle only while a live gate or his game runs (S107).
7. **Queue the next window before this one ends:** a detached launch scheduled for the reset loses no window to the integrator being away.
8. **Admin goes through three scripts:** `launch.ps1` (start, with slot refusals), `status.ps1` (every lane, every report not committed), `meters.ps1 -WatchAt 80` ([work.md](../work.md)).

## 5. Launching and platform traps

A job is `jobs/<id>/prompt.md` in, `jobs/<id>/last.md` out ([work.md](../work.md)). Every runner prepends the goal and principles verbatim (`scripts/gpt.ps1`).

**Windows traps.** `Start-Process` does not quote arguments and this path has a space; prompts go on stdin, never `-p`; never write a BOM (`[Text.UTF8Encoding]::new($false)`); under PowerShell 5.1 a native command's stderr under `ErrorAction Stop` throws (git, javac), and `git commit -F -` does not read a here-string: use a message file; cite the main checkout by absolute path in any prompt; Codex `workspace-write` mounts `.git/` read-only even inside standalone D: clones (denying `index.lock`, so `git commit` and `bench-judge.sh`'s `git stash create` fail inside Codex — the integrator commits verified trees and runs `bench-judge.sh` outside).

## 5b. How a job is assigned, in practice

1. Name the band. A machine checks it → Checked. Only another mind checks it → Judged. Nothing checks it later → Frozen.
2. Checked: the cheapest companion whose bake-off row covers that shape, as many lanes as its window sustains.
3. Judged: a family the builder is not in, fresh context, the spec in hand and the verdict unwritten.
4. Frozen: the best mind, then a red-team by a different family before it freezes.
5. Record the outcome so a row moves on evidence rather than on how a job felt.

## 5c. Operational rules (S128)

1. Every launch goes through `launch.ps1`, detached, at Normal priority (S158); anything that must outlive a turn is a detached process, never an in-session background task. Test launcher changes with `-DryRun`.
2. Every job writes as it goes and ends with a status (GPT `STATUS:`). An empty run is a launcher fault: fix it before rerunning.
3. Test each spec's load-bearing premise with one command before launch.
4. **The Codex window is Astra's** (S209: "the goal is to maximize astra more"). Terra backfills the leftover ~20%. Red-team native Opus; judge native Sonnet. GLM extracts parameter rows, scars, jar witnesses, and `lab/wiki/` notes paced by `scripts/glm-chain.ps1`. A smaller model never writes for a bigger one (S210; [work.md](../work.md)).
5. Read meters raw before stating them, naming which column is used or remaining. A reading older than its window's last reset (the Codex meter's `as of` is its newest session) is stale, not a fact: say so, and never hold a launch on it. **The usage check is one step: `get_usage` (the native Anthropic plan: 5-hour, weekly, Fable weekly) and `meters.ps1` (Codex, GLM) in the same message, before every launch** (S168). Launch no native sub-agent above 80% of the native 5-hour window.
6. Read-only companions get their evidence made for them (failure lines, block dumps, logs).
7. Live gates only on a free machine, with a fresh evidence folder, the gate's Java at High, and a CPU sampler beside it.
8. Script bugs beyond a couple of commands go to a Terra job, not into the integrator's context.
9. Terra halts the whole job at its first open question: settle every design question in the prompt ("decisions already made"), name the independent steps, and resume with a continuation, never a bare retry. A verdict goes to `jobs/<id>/last.md`, never only into chat.
10. **Our docs are linted against his words, from time to time** (S210, S211: "our own docs should be lintet based on what i said looking for issues"). A native Sonnet sub-agent reads every living doc beside his verbatim turns and the register, and reports drift, contradictions, stale claims, his rules not written down, and made-up names (`jobs/doc-lint`). It runs after each phase and whenever a doc has been rewritten at length. The integrator applies the fixes.

## 6. Google

Julien pilots Antigravity himself; no runner here calls Google. A third judging family is open (section 2).

## 7. Open

- Which family's errors are least like which is unmeasured. Until it is, the judge rule stays "the other family" ([work.md](../work.md)).
