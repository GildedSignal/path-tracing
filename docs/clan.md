# The clan: who does what, and why

The team table in [AGENTS.md](../AGENTS.md) says who holds each role. This file says **how a job is assigned**, and it is the file that changes when a companion joins or a measurement moves someone.

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

Until 13 November the clan had two families, Anthropic and OpenAI, plus GLM's lab. With two families the judge rule ("by the other family") is forced: there is only one other family. Google is the **third**, and that is the real purchase, not the tokens.

1. **The judge can be chosen, not conceded.** For each builder, the judge is the family whose errors are least like the builder's. Which family that is, is a measurement, not an opinion (section 4).
2. **What a machine cannot check can be put to three.** Two minds that disagree leave the integrator to break the tie by taste; three that disagree say the question is underspecified, which is information. Used only where no machine can check, because where one can, a vote is fake work.
3. **A blind spot shared by two families stops being invisible.** Our whole record was produced by two vendors' training distributions. We have never measured what both get wrong together.

## 3. The companions

Names are the vendors' own. Every row is a claim, and section 4 is what turns a claim into evidence or removes it.

| Companion | Family | Band claimed | The claim, in one line |
|---|---|---|---|
| Opus | Anthropic | Frozen, Judged | taste: finds the one-way door and the wrong abstraction before either freezes |
| Fable | Anthropic | Orchestration | holds the whole project; writes the specs; decides the merges |
| Sonnet | Anthropic | Judged | adversarial reading of a diff against a spec, fresh context |
| Haiku | Anthropic | Checked | runs, log digests, completeness |
| Terra | OpenAI | Checked (build) | a spec that leaves nothing open, built exactly, unattended |
| Astra | OpenAI | Frozen (convergent) | the math Terra stalls on: a bound, a monotonicity, a closed gap |
| GLM | Z.ai | Checked (wide) | many cheap narrow questions, each reproduced by a second session ([lab/glm.md](../lab/glm.md)) |
| **Argon** | **Google** | **claimed: corpus traversal; untrusted reading; primary sources** | **three rows, none of them building. It walks the corpus and says what contradicts what; it reads the channel where our exposure is measured and worst; and it is the mind for his primary sources** |
| **Hydrogen** | **Google** | **claimed: Checked (wide)** | **a second cheap-wide lane beside GLM, on the same lens queue, graded against GLM's own reproduced answers. Run at `high`: its terminal score is identical at `xhigh`, so `xhigh` is pure waste** |

**Why Argon gets three rows and none of them is building.** Its benchmarks say plainly that it should not build and should not judge (section 4b). What they also say is that it is the best thing we have for three jobs we had nobody for:

1. **Corpus traversal.** GraphWalks 84.2 against Astra's 71.8 and Opus's 66.8 — a 17-point lead, and the only place it leads anything shaped like our work. That benchmark *is* multi-hop traversal over a graph, and this corpus is a graph: HANDOFF points at jobs, jobs at run folders, run folders at wiki pages, the register at his turns. "Where does `learning.md` disagree with HANDOFF" is that query, and we know such disagreements exist, because one was corrected this week.
2. **The untrusted channel, which is the real purchase.** Argon's indirect-prompt-injection success rate is **0.7%**, the lowest on the roster, against Opus's 1.0% and **GLM's 31.5%**. GLM's lanes read decompiled Mojang bytecode and scraped wiki pages all day. That 31.5% is a measured exposure with no fix beside it; Argon is the fix for the reading half of it.
3. **His primary sources.** The tracked request from S86–S89 — read directly whatever a design leans on, and mine his book list and Readwise in depth — is long-document synthesis, not terminal work. That is where Argon's wins actually are, and that request is owed.

So Argon is not weak; it was pointed at the wrong band by the first draft of this file. Hydrogen's row is unchanged: a second wide lane, graded against GLM's own reproduced answers.

## 4. The bake-off: no row stands on reputation

A companion's row is a hypothesis, settled the way everything here is settled — against a known answer, by a check we already trust (`lab/glm.md`, "Canaries").

**The protocol.** Give the newcomer a job **we have already done**, whose answer is on disk and whose grader is a script, and grade it blind.

- **Checked band:** a lens from `lab/queue.txt` that GLM has already reproduced. Grade: did it reproduce, in what wall time, at what cost, and did its own independent checker agree? A newcomer that produces a confident wrong result its own checker passes is worse than useless, and is refused the band.
- **Judged band:** a diff we have already judged, including at least one whose verdict was **FAIL**, with the verdict withheld. Grade both halves: did it find the blocking finding, and did it invent blocking findings that are not there? A judge that fails everything is not a judge.
- **Frozen band:** a spec we have already red-teamed, with the red-team's report withheld. Grade: how many of the known changes it finds, and whether it finds one we missed.
- **Orchestration support:** a contradiction we know is in the corpus — the HANDOFF reading corrected in [learning.md](learning.md) is one, and the register holds others. Grade: found, with both files named, or not found.

**Canaries both ways.** Some graded items are subtly wrong on purpose. A companion that passes a planted wrong one does not get its band, whatever it scored elsewhere.

**The cost column is not optional.** Two companions within noise of each other are the same companion, and the cheaper one takes the work (ES 27: the goal, never a proxy).

**But the currency is window-hours, not dollars** (S102). Every family here is a flat subscription: we spend *a meter*, and an unspent meter is worth nothing when it resets. What matters is which meter the work lands on.

**What each model costs, per kind of work** (rough anchors from 14 November, S113: not a science, just enough to route work by what it needs; `.\scripts\meters.ps1` reads every meter, so re-anchor whenever a number surprises you):

| Need | Model | Rough cost | Anchored on |
|---|---|---|---|
| a jar law, a measurement, a port delta | GLM lane (~1 h, author + checker) | ~8–10% of Zai's 5 h | 41% for ~5 lanes |
| a narrow build with a closed spec | Terra run (5–7 min) | ~5–8% of Codex's 5 h | mostly the ~1.3M input it reads, so name the files |
| hard design within a spec | Astra run (~30 min) | ~25–30% of Codex's 5 h | 43% → 99% with Astra and ~4 Terra |
| one full Codex 5-hour window | | ~15% of Codex's weekly | 10% → 25% |
| a corpus audit, a contradiction hunt | Argon (~300k in) | ~7% of Argon's own weekly | one audit |
| a short judge with the diff inlined | Sonnet on Antigravity (~2 min) | small | 65k in, 16k out |
| a big design | Opus on Antigravity | most of the Claude-and-GPT weekly | 233k in, 119k out, 2M cached; it also seems to have a 5-hour limit (13 November) |
| judging GPT's work | native Sonnet sub-agent | ~100k tokens for three diffs | batch several diffs per judge |
| orchestration | native integrator | ~40% of its 5 h per hour at high context | 70% after 1 h 40 |
| wide cheap lanes | Hydrogen | unknown | no run has survived its permission denials yet |
| measurement | a plain command | no meter | flights, scaling and timings were all commands |

Windows: Zai 5 h; Codex 5 h and weekly; Antigravity's Gemini models 5 h and weekly (2% and 1% used on 14 November), its Claude and GPT models one shared weekly, Argon its own weekly; the native plan 5 h, weekly, and weekly per model.
**So: inside a band, spend the least contended meter; the native plan runs the integrator and is spent last.** "Fill the Anthropic context" means load the integrator with context, not drain its quota (S102).

**The protocol, from what wasted the 14 November windows:**
1. **Close every design question before a builder starts.** Four Terra runs wrote zero lines (3.3c, 3.3d twice, 3.2h) on an open classification or representation question: that is Designer or integrator work, done first.
2. **One deliverable per job, the files to read named, small ones inlined.** A Terra run that reads the corpus costs ~1.3M input tokens; the status contract in `gpt.ps1` now says to read only what the spec names.
3. **A long job writes its output to a file as it goes.** The Opus design died with the meter and its 119k output tokens were lost; a report written progressively survives a quota death.
4. **Measure with a command, not a job.** The 18-track flight, the straight tracks and the thinking split were plain commands the integrator ran at no meter cost; jobs are for building.
5. **Never resume into a spent meter** (`gpt.ps1` stops on "usage limit"), and never launch a lane whose meter `meters.ps1` shows above 90%.
6. **Lanes run at Idle CPU priority** (`launch.ps1` does it), so a live gate shares the machine (S107).
7. **Queue the next window before this one ends:** a detached launch scheduled for the reset (see HANDOFF) loses no window to the integrator being away.
8. **Admin goes through three scripts:** `launch.ps1` (start, with slot refusals), `status.ps1` (every lane, every report not committed), `meters.ps1 -WatchAt 80` in the background (wakes the integrator on a crossing). Background waiters that exit on a report replace polling.

## 4b. What the public record says, and where it contradicts section 3

Measured 13 November, independent harnesses preferred over vendor tables (the same model and benchmark differed by 6.8 points between Anthropic's number and Artificial Analysis's, so assume a similar premium on Google's and OpenAI's tables, which have no independent counterpart at all).

- **Argon loses every benchmark shaped like our work.** Terminal-Bench 4.0: 57.4 against Opus 5.5's 59.6 independent / 66.4 vendor. FrontierSWE v2: 55.0 against Astra's 65.5. Terminal-Bench-Science: 57.6 against Astra's 68.1. It leads on legal, finance, video — and on **graph traversal: GraphWalks 84.2, against Astra 71.8 and Opus 5.5 66.8**. That one number is the only public support for its row here, and it happens to be exactly the shape of a corpus pass. Its row is therefore narrowed to that and nothing else, and it is the most expensive route per task on the roster, with its price doubling on 1 December.
- **Nobody has publicly shown any model on this roster is good at 1M tokens.** Artificial Analysis dropped its long-context reasoning benchmark; no MRCR or fiction.live entry exists for any of them. Every "1M context" figure is a capacity, not a quality. GraphWalks is the single exception above.
- **The one number worth most** is Sonnet 5.5 at 63.9% on Terminal-Bench 4.1 **in Google's own comparison table** — a competitor reporting a rival ahead of its own model is the least corruptible evidence available, and it holds up the judge row.
- **Terra's vendor claims scope adherence and blocker reporting**, and our own record confirms it: all four jobs of this wave reported precisely what they had not done rather than claiming completion. But its factuality error rate is 4.3% at max effort and **16.7% at low**, so low effort is unusable for anything load-bearing. `scripts/gpt.ps1` now pins high.
- **GLM is the weakest-supported row we have**, and honesty requires saying so: index 32 against Hydrogen's 54, slow (27.6 min/task), very verbose, with reproduced endless-reasoning and degeneration failures in agentic tool use, and a **31.5% success rate for indirect prompt injection** against Argon's 0.7% and Opus's 1.0%. The last is not academic: GLM's lanes read Mojang's jar and decompiler output, which is exactly the untrusted-artefact channel that number measures. It keeps its band on the window rule above and on 72 reproduced results out of 106 lanes — **not** on being good or cheap, and the injection figure is a real exposure to close.
- **METR, the only independent party that looked for it, found no large improvement in judgement, taste or foresight** in the model our Designer row rests on, and published no time-horizon figure at all. Treat long unattended autonomy as unmeasured for every companion here; the 18-hour-run stories are vendor anecdotes.

## 5. How to launch one, from a cold session

A job is a folder: `jobs/<id>/prompt.md` is the spec, `jobs/<id>/last.md` the report. Every runner prepends PRINCIPLES' goal and philosophy verbatim.

| Companion | Launch | Notes |
|---|---|---|
| Terra (build), Astra (hard design within a spec) | `.\scripts\launch.ps1 <id> terra -Worktree D:\path-tracing-wt\<name> [-Base <ref>]` (or `astra`) | ≤ 4 GPT, ≤ 2 with Astra; effort pinned high; resumes itself until `STATUS: DONE` or `BLOCKED`; commits are the integrator's, with `Built-by:`. The fixture world is git-ignored: copy `engine/world/tests/fixtures/world` into a new worktree |
| Sonnet on Antigravity (judge) | `.\scripts\launch.ps1 <id> sonnet [-ReadDir <worktree>]` | ≤ 2 Sonnet or 1 Opus; weekly meter; inline the diff |
| Argon (read, traverse), Hydrogen | `.\scripts\launch.ps1 <id> argon` (or `hydrogen`) | ≤ 1 Argon or 2 Hydrogen; read-only |
| Native Claude sub-agent | the integrator's Agent tool, fresh context | one at a time; batch judges into one |
| GLM | `.\scripts\glm-queue.ps1 -Queue lab\queue.txt` (re-reads the file each batch) or `.\scripts\glm-local.ps1 -Lenses "<a b>"` | **commit the lens first**: lanes run from a checkout of HEAD. Read `checker.log` for 429 before trusting a "lead" |

**Antigravity traps.** It soft-denies silently: a refused action exits 0 with a note on stderr, so read the report, not the exit code; an empty report on one denied `read_file` is common, so inline what a job must read. Its meters read headless at no model cost: `/usage` on stdin prints each group's share **remaining** and its reset (S112; `meters.ps1` does it). A spent one also fails a run with `RESOURCE_EXHAUSTED … Resets in <h>`. Slugs: re-run `agy models` (13 November: `gemini-4-argon-{low,medium,high}`, `gemini-4-hydrogen-{low,…,xhigh}`, `claude-{opus,sonnet}-5-5-{low,medium,high}`; no frontier GPT, so Codex is the GPT route).

**Windows traps.** `Start-Process` does not quote arguments and this path has a space; prompts go on stdin, never `-p`; never write a BOM (`[Text.UTF8Encoding]::new($false)`); under PowerShell 5.1 a native command's stderr under `ErrorAction Stop` throws (git, javac), and `git commit -F -` does not read a here-string: use a message file; cite the main checkout by absolute path in any prompt.

## 5b. How a job is assigned, in practice

1. Name the band. A machine checks it → Checked. Only another mind checks it → Judged. Nothing checks it later → Frozen.
2. Checked: the cheapest companion whose bake-off row covers that shape, as many lanes as its window sustains.
3. Judged: a family the builder is not in, fresh context, the spec in hand and the verdict unwritten.
4. Frozen: the best mind, then a red-team by a different family before it freezes.
5. Record the outcome. The census belongs in `docs/research/roster-evidence.md`, regenerated by `scripts/roster-census.ps1`, so a row moves on evidence rather than on how a job felt. **Neither file is on `main` yet**: the job that writes them stopped twice and is owed a new spec, so today only the GLM half exists, in `D:\path-tracing-wt\roster-b`.

## 6. How Google is reached, and the one decision it needs

The old Gemini CLI is not the path: Google stopped serving AI Pro, Ultra and free individuals there on 18 June 2026. Jules is not the path either — it winds down on 31 December 2026 and went read-only on 11 November, two days before Julien's plan. The path is **Antigravity CLI** (`agy`), **installed and signed in on his AI Pro account since 13 November**, version 1.5.8. Headless, as `scripts/gemini.ps1` actually calls it: the prompt goes in on **stdin** (it is far past the command-line limit, so `-p` is not usable), the effort is **baked into the slug** rather than passed as `--effort`, and file access is granted by `permissions.allow` rules in a settings file passed with `--settings`.

There are **two routes, and they are not the same risk**:

| Route | Reaches | Standing | Cost |
|---|---|---|---|
| **A — `agy` signed in with the AI Pro account** | **Argon and Hydrogen.** The only route to Argon at all | **grey.** Google's own FAQ says headless `-p` "is intended for direct scripting and CI workflows", and in the same document that wrapping `agy` inside third-party coding agents "is not supported" and that third-party access can mean suspension or termination. A script Julien runs is the supported side; the integrator spawning it is the contested side, and Google has not answered the question where it was asked | included in the plan |
| **B — `GEMINI_API_KEY`, `modelProvider: gemini`** | **Hydrogen only.** Argon has no API-key path | **clear.** This is the FAQ's own recommendation for third-party agents | **free** on Hydrogen's free tier (which trains on the content), $1.50/$7.50 per 1M paid. Argon, for comparison, goes to $4/$20 on 1 December |

**Julien chose route A on 13 November (S102)**, on his own account and his own plan, and it is signed in. Route B stays available for Hydrogen at no cost.

**Argon's first run earns its corpus row** (`jobs/corpus-contradictions/`, 303k in / 31k out, 439s, `gemini-4-argon-high`). Graded against contradictions the integrator knew and did not disclose: it found the retired-Sol line still standing in HANDOFF, the body-block-change cut still in `vocabulary.md` against 12.4, this file's own stale launch prose, and HANDOFF over its line budget — **twelve in all, with line-level citations**. It **correctly refused** the planted-looking trap: the two ice speeds in PRINCIPLES against HANDOFF are not a contradiction, and it said why (19.345 is regular ice, 19.903 is blue ice), rather than inflating its count. It invented none, and named what it could not reach. The row stands.

**Three things must be proved by an actual run before anything depends on this** (each has a known failure on record):
1. **Headless print mode on Windows.** Issue #318 is still reported reproducing on Windows 11 as recently as release 1.4.10 — print timeouts with the turn in progress, stream-json emitting only `init`. Smoke-test `agy -p "reply OK" --output-format json` redirected to a file before any wrapper is written.
2. **The silent soft-deny.** In headless there is no prompt, so an action the permissions do not allow is denied and **the run still exits 0**, with only a stderr notice. An exit code is therefore not a gate here; the gate must read the report.
3. **Credentials from a detached process.** `agy` keeps them in the Windows Credential Manager, not a token file. Whether a `Start-Process` child can read them is undocumented.

Settings are written with `-Encoding utf8NoBOM`: a BOM made `settings.json` be rejected as malformed until release 1.5.8, and PowerShell writes one by default — the same trap `options.txt` already has on the rig.

## 7. Open

- **Hydrogen** has never run a real job here: its row is still a claim with no evidence. Argon's corpus row is earned (section 6); its other two rows, the untrusted channel and the primary sources, are not yet tested.
- Route A was **Julien's call** (S102): it trades a grey-zone ToS reading against the only access to Argon. He chose it; `agy` is installed and signed in.
- **Hydrogen's first job** is `anomaly-1-stepup` (14 November), a read job whose answer the integrator can check against the code.
- Which family's errors are least like which is unmeasured. Until it is, the judge rule stays "the other family", as AGENTS.md has it.
