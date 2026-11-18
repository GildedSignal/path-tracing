# The clan: who does what, and why

The team table in [work.md](../work.md) says who holds each role. This file says **how a job is assigned**, and it is the file that changes when a companion joins or a measurement moves someone.

Julien, S10: "plan for the whole build with the appropriate sub-agents doing the right things with the right intellect tackling things in the right order." S12: GPT "tends to be good with well formed tasks while you do better when taste is required." S102 adds the Google AI Pro plan, and with it Argon and Hydrogen.

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

The clan has three families — Anthropic, OpenAI and Google — plus GLM's lab. Google as a third family is the real purchase, not the tokens:

1. **The judge can be chosen, not conceded.** For each builder, the judge is the family whose errors are least like the builder's. Which family that is, is a measurement, not an opinion (section 4).
2. **What a machine cannot check can be put to three.** Two minds that disagree leave the integrator to break the tie by taste; three that disagree say the question is underspecified, which is information. Used only where no machine can check, because where one can, a vote is fake work.
3. **A blind spot shared by two families stops being invisible.** What any two vendors' training distributions get wrong together is unmeasured.

## 3. The companions and routing

Names are the vendors' own. Every row is a claim, and section 4 is what turns a claim into evidence or removes it. Launch and meter commands are in [work.md](../work.md).

| Companion | Family | Band claimed | Claim and launch notes |
|---|---|---|---|
| Opus | Anthropic | Frozen, Judged | taste: finds the one-way door and the wrong abstraction before either freezes. Native sub-agent (never on Antigravity, S115); one at a time |
| Fable | Anthropic | Orchestration | holds the whole project; writes the specs; decides the merges |
| Sonnet | Anthropic | Judged | adversarial reading of a diff against a spec, fresh context. Native sub-agent (batch judges into one) or Antigravity (`-ReadDir <worktree>`, ≤ 2 Sonnet, weekly meter, inline the diff) |
| Haiku | Anthropic | Checked | runs, log digests, completeness |
| Astra | OpenAI | Frozen (convergent), the core | the core and the important builds, a bound, a monotonicity, a closed gap; the Codex window is hers (S174, S209). Effort pinned high; resumes itself until `STATUS: DONE` or `BLOCKED`; commits are the integrator's with `Built-by:`. Copy git-ignored `engine/world/tests/fixtures/world` into a new worktree |
| Terra | OpenAI | Checked (build) | a spec that leaves nothing open, built exactly, unattended; backfills the leftover ~20% of the Codex window (S209). ≤ 4 GPT; with Astra, one GPT job beside her (S15); same launch rules as Astra |
| GLM | Z.ai | Checked (wide) | the port only (S173): jar-backed port deltas, each reproduced by a second session ([lab/glm.md](../../lab/glm.md)). `.\scripts\glm-local.ps1 -Lenses "<job-id>"` (reads `jobs\<id>\prompt.md`): commit the prompt first, because lanes run from a checkout of HEAD |
| **Argon** | **Google** | **claimed: corpus traversal; untrusted reading; primary sources** | builds tooling and lints our docs against his words (S211); walks the corpus and says what contradicts what; reads untrusted artefacts (decompiled Mojang bytecode, scraped wiki pages); reads primary sources directly and mines his book list and Readwise (S86–S89). ≤ 1 Argon |
| **Hydrogen** | **Google** | **claimed: Checked (wide)** | premise checks, digests and builds (S173, S197, S210), reporting to the integrator, never writing for a bigger model. Run at `high`: its terminal score is identical at `xhigh`, so `xhigh` is pure waste. ≤ 2 Hydrogen |

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

Windows: Zai 5 h; Codex 5 h and weekly; Antigravity's Gemini models 5 h and weekly, its Claude and GPT models one shared weekly, Argon its own weekly; the native plan 5 h, weekly, and weekly per model.
**So: inside a band, spend the least contended meter; the native plan runs the integrator and is spent last. On a shared meter, big models first** (S114): unspent capacity is lost at reset, and a small model cut off mid-task costs far less than a big one. Launch Astra (or Argon, or a native Opus) at the start of a window, and let Terra and the lanes fill what is left. "Fill the Anthropic context" means load the integrator with context, not drain its quota (S102).

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

A job is `jobs/<id>/prompt.md` in, `jobs/<id>/last.md` out ([work.md](../work.md)). Every runner prepends the goal and principles verbatim (`scripts/gpt.ps1`, `scripts/gemini.ps1`).

**Antigravity traps.** It soft-denies silently: a refused action exits 0 with a note on stderr, so read the report, not the exit code; a command rule matches the WHOLE command line (`command(cargo)` does not allow `cargo --version`), so commands go through the sandbox, never an allow-list; under `--sandbox` a path form without a drive letter (`/C:/…`, `file:///…`) breaks every command ("non-absolute file path"), and only a write rule with no trailing slash makes a folder writable; an empty report on one denied `read_file` is common, so inline what a job must read. Its meters read headless at no model cost: `/usage` on stdin prints each group's share **remaining** and its reset (S112; `meters.ps1` does it). A spent one also fails a run with `RESOURCE_EXHAUSTED … Resets in <h>`. Slugs (re-run `agy models`): `gemini-4-argon-{low,medium,high}`, `gemini-4-hydrogen-{low,…,xhigh}`, `claude-{opus,sonnet}-5-5-{low,medium,high}`; no frontier GPT, so Codex is the GPT route.

**Windows traps.** `Start-Process` does not quote arguments and this path has a space; prompts go on stdin, never `-p`; never write a BOM (`[Text.UTF8Encoding]::new($false)`); under PowerShell 5.1 a native command's stderr under `ErrorAction Stop` throws (git, javac), and `git commit -F -` does not read a here-string: use a message file; cite the main checkout by absolute path in any prompt.

## 5b. How a job is assigned, in practice

1. Name the band. A machine checks it → Checked. Only another mind checks it → Judged. Nothing checks it later → Frozen.
2. Checked: the cheapest companion whose bake-off row covers that shape, as many lanes as its window sustains.
3. Judged: a family the builder is not in, fresh context, the spec in hand and the verdict unwritten.
4. Frozen: the best mind, then a red-team by a different family before it freezes.
5. Record the outcome so a row moves on evidence rather than on how a job felt.

## 5c. Operational rules (S128)

1. Every launch goes through `launch.ps1`, detached, at Normal priority (S158); anything that must outlive a turn is a detached process, never an in-session background task. Test launcher changes with `-DryRun`.
2. Every job writes as it goes and ends with a status (GPT `STATUS:`; Antigravity `report.md` every few files). An empty run is a launcher fault: fix it before rerunning.
3. Test each spec's load-bearing premise with one command before launch.
4. **The Codex window is Astra's** (S209: "the goal is to maximize astra more"). Terra backfills the leftover ~20%. Red-team native Opus; judge native Sonnet. GLM writes port code paced by `glm-queue`. Hydrogen and Argon read and build on their own Google meters. A smaller model never writes for a bigger one (S210; [work.md](../work.md)).
5. Read meters raw before stating them, naming which column is used or remaining. **The usage check is one step: `get_usage` (the native Anthropic plan: 5-hour, weekly, Fable weekly) and `meters.ps1` (Codex, GLM, Antigravity) in the same message, before every launch** (S168). Launch no native sub-agent above 80% of the native 5-hour window.
6. Read-only companions get their evidence made for them (failure lines, block dumps, logs). **Argon and Hydrogen run commands with `launch.ps1 <id> argon|hydrogen -Commands -Worktree D:\path-tracing-wt\<id>`** (S210): agy's `--sandbox`, `command(*)`, drive-letter path forms only, and writable mounts for the worktree, the job folder, `D:\path-tracing-target\agy-<id>` and `D:\rust`. Every other write is refused. `test-antigravity.ps1` builds and runs a ten-line Rust program this way before any Antigravity job may start.
7. Live gates only on a free machine, with a fresh evidence folder, the gate's Java at High, and a CPU sampler beside it.
8. Script bugs beyond a couple of commands go to a Terra job, not into the integrator's context.
9. Terra halts the whole job at its first open question: settle every design question in the prompt ("decisions already made"), name the independent steps, and resume with a continuation, never a bare retry. A verdict goes to `jobs/<id>/last.md`, never only into chat.
10. **Our docs are linted against his words, from time to time** (S210, S211: "our own docs should be lintet based on what i said looking for issues"). Argon reads every living doc beside his verbatim turns and the register, and reports drift, contradictions, stale claims, his rules not written down, and made-up names (`jobs/doc-lint`). It runs after each phase and whenever a doc has been rewritten at length. The integrator applies the fixes.

## 6. How Google is reached, and the one decision it needs

The path is **Antigravity CLI** (`agy`), version 1.5.8, signed in on Julien's AI Pro account (S102). Headless, as `scripts/gemini.ps1` calls it: the prompt goes in on **stdin** (past the command-line limit, so `-p` is not usable), the effort is **baked into the slug** rather than passed as `--effort`, and file access is granted by `permissions.allow` rules in a settings file passed with `--settings`.

There are **two routes, and they are not the same risk**:

| Route | Reaches | Standing | Cost |
|---|---|---|---|
| **A — `agy` signed in with the AI Pro account** | **Argon and Hydrogen.** The only route to Argon at all | **grey.** Google's own FAQ says headless `-p` "is intended for direct scripting and CI workflows", and in the same document that wrapping `agy` inside third-party coding agents "is not supported" and that third-party access can mean suspension or termination. A script Julien runs is the supported side; the integrator spawning it is the contested side, and Google has not answered the question where it was asked | included in the plan |
| **B — `GEMINI_API_KEY`, `modelProvider: gemini`** | **Hydrogen only.** Argon has no API-key path | **clear.** This is the FAQ's own recommendation for third-party agents | **free** on Hydrogen's free tier (which trains on the content), $1.50/$7.50 per 1M paid. Argon is $4/$20 |

**Julien chooses route A (S102)**, on his own account and his own plan, and it is signed in. Route B stays available for Hydrogen at no cost.

**Three things must be proved by an actual run before anything depends on this** (each has a known failure):
1. **Headless print mode on Windows.** Issue #318 reproduces on Windows 11 (print timeouts with the turn in progress, stream-json emitting only `init`). Smoke-test `agy -p "reply OK" --output-format json` redirected to a file before any wrapper is written.
2. **The silent soft-deny.** In headless there is no prompt, so an action the permissions do not allow is denied and **the run still exits 0**, with only a stderr notice. An exit code is therefore not a gate here; the gate must read the report.
3. **Credentials from a detached process.** `agy` keeps them in the Windows Credential Manager, not a token file. Whether a `Start-Process` child can read them is undocumented.

Settings are written with `-Encoding utf8NoBOM` (PowerShell writes a BOM by default, the same trap `options.txt` has).

## 7. Open

- Argon's corpus row stands; its untrusted-channel and primary-sources rows are not yet tested.
- Route A is **Julien's call** (S102): it trades a grey-zone ToS reading against the only access to Argon.
- Which family's errors are least like which is unmeasured. Until it is, the judge rule stays "the other family" ([work.md](../work.md)).
