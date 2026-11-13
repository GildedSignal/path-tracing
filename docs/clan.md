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

**But the currency is window-hours, not dollars** (S102). Every family here is a flat subscription, so a price per million tokens is not what we spend — we spend *a meter*, and an unspent meter is worth nothing when it resets. GLM costs about $5.01 per completed task against Haiku's $0.41, and that is irrelevant: what matters is which meter the work lands on.

**The meters, and the order they are spent in.** They are independent, and Antigravity is the reason there are more of them than there are families: its Gemini models, its Claude models and its GPT models each draw on **their own meter**, separate from Google's and separate from Julien's native plans.

| Meter | Spend it | Why |
|---|---|---|
| GLM's 5-hour window | **first**, always full | nothing else contends for it; ~18 lanes per window |
| Antigravity's Gemini meter | first, for Checked-band lanes | included in the new plan, otherwise idle |
| **Antigravity's Claude meter** | **before the native Anthropic plan** | it buys Claude-shaped work — judging, red-teaming — *without touching the reserve* |
| ~~Antigravity's GPT meter~~ | **does not exist** | `agy models` carries only `gpt-oss-120b`, the open-weights one. Codex stays the only GPT route |
| Codex (Terra, Astra) | freely; it drains fast (~4 jobs an hour), so rank jobs by what they unblock | the build lane |
| **Native Anthropic (Opus, Sonnet, Fable, Haiku)** | **last. It is the reserve, and it is exhausted last** | it is the only meter that runs the integrator, and the integrator cannot be delegated. Spending it on work another meter could have done is the one unrecoverable waste here |

**So the rule is: inside a band, spend the least contended meter, and treat the native Anthropic plan as the one you do not spend.** Dollars decide only where two companions draw on the *same* meter, or where one must be bought outside a plan.

This reverses the first reading of S102. "Fill the Anthropic context" means **load this session with context** — the integrator must hold the whole project — not *spend the Anthropic quota hardest*. Context in; quota conserved.

## 4b. What the public record says, and where it contradicts section 3

Measured 13 November, independent harnesses preferred over vendor tables (the same model and benchmark differed by 6.8 points between Anthropic's number and Artificial Analysis's, so assume a similar premium on Google's and OpenAI's tables, which have no independent counterpart at all).

- **Argon loses every benchmark shaped like our work.** Terminal-Bench 4.0: 57.4 against Opus 5.5's 59.6 independent / 66.4 vendor. FrontierSWE v2: 55.0 against Astra's 65.5. Terminal-Bench-Science: 57.6 against Astra's 68.1. It leads on legal, finance, video — and on **graph traversal: GraphWalks 84.2, against Astra 71.8 and Opus 5.5 66.8**. That one number is the only public support for its row here, and it happens to be exactly the shape of a corpus pass. Its row is therefore narrowed to that and nothing else, and it is the most expensive route per task on the roster, with its price doubling on 1 December.
- **Nobody has publicly shown any model on this roster is good at 1M tokens.** Artificial Analysis dropped its long-context reasoning benchmark; no MRCR or fiction.live entry exists for any of them. Every "1M context" figure is a capacity, not a quality. GraphWalks is the single exception above.
- **The one number worth most** is Sonnet 5.5 at 63.9% on Terminal-Bench 4.1 **in Google's own comparison table** — a competitor reporting a rival ahead of its own model is the least corruptible evidence available, and it holds up the judge row.
- **Terra's vendor claims scope adherence and blocker reporting**, and our own record confirms it: all four jobs of this wave reported precisely what they had not done rather than claiming completion. But its factuality error rate is 4.3% at max effort and **16.7% at low**, so low effort is unusable for anything load-bearing. `scripts/gpt.ps1` does not set an effort today and must pin one.
- **GLM is the weakest-supported row we have**, and honesty requires saying so: index 32 against Hydrogen's 54, slow (27.6 min/task), very verbose, with reproduced endless-reasoning and degeneration failures in agentic tool use, and a **31.5% success rate for indirect prompt injection** against Argon's 0.7% and Opus's 1.0%. The last is not academic: GLM's lanes read Mojang's jar and decompiler output, which is exactly the untrusted-artefact channel that number measures. It keeps its band on the window rule above and on 72 reproduced results out of 106 lanes — **not** on being good or cheap, and the injection figure is a real exposure to close.
- **METR, the only independent party that looked for it, found no large improvement in judgement, taste or foresight** in the model our Designer row rests on, and published no time-horizon figure at all. Treat long unattended autonomy as unmeasured for every companion here; the 18-hour-run stories are vendor anecdotes.

## 5. How to launch one, from a cold session

Everything below is copy-pasteable from the repo root. A job is always a folder: `jobs/<id>/prompt.md` is the spec, and `jobs/<id>/last.md` is the report it leaves. Every runner prepends the goal and philosophy sections of PRINCIPLES verbatim, so no prompt repeats them.

| Companion | Launch | Notes |
|---|---|---|
| **Terra** (build) | `.\scripts\gpt.ps1 -Job <id> -Worktree D:\path-tracing-wt\<id> -Model gpt-6.1-terra` | four at once, **fresh first prompts only**; resume with `-Resume <session id>`. It cannot write git metadata, so the integrator commits with `Built-by:` |
| **Astra** (hard math) | same, `-Model gpt-6-astra` | sparingly, and only one GPT job beside it |
| **Argon** (read, traverse) | `.\scripts\gemini.ps1 -Job <id> -Model gemini-4-argon-high` | read-only; the report is the CLI's JSON response. `-Write <worktree>` opts into editing |
| **Hydrogen** (wide, cheap) | `.\scripts\gemini.ps1 -Job <id> -Model gemini-4-hydrogen-high` | never `xhigh`: its terminal score is identical and costs more |
| **Sonnet / Opus on Antigravity** | `.\scripts\gemini.ps1 -Job <id> -Model claude-sonnet-5-5-high` (or `claude-opus-5-5-high`) | **this is the meter that spares the native Anthropic plan.** Prefer it for judging and red-teaming |
| **Claude sub-agents** (native) | the integrator's own Agent tool, fresh context | the reserve. **One at a time**, and only where the native plan is the only thing that can do it |
| **GLM** (lab lanes) | `.\scripts\glm-local.ps1 -Lenses "<space-separated>" -Minutes 60`, or `.\scripts\glm-queue.ps1` for [lab/queue.txt](../lab/queue.txt) | ~18 lanes per 5-hour window, no slot limit. Read `checker.log` for HTTP 429 before trusting a "lead" |

**Reading Antigravity's meters.** There is no usage readout in the CLI and Google publishes no numbers, but the meter announces itself when it runs out: a run returns `status: ERROR` with `RESOURCE_EXHAUSTED (429): Individual quota reached … Resets in 4h34m41s`, and **that reset time is the 5-hour window**. Its Gemini and Claude meters are genuinely separate — the Claude one was exhausted on 13 November while Argon kept running on the same account. So the honest way to report Antigravity usage is: not readable until it is spent, then exact.

**An empty report is a real failure mode, and it is not always the model's.** Every `agy` run so far logs exactly one auto-denied `read_file`. A job that must read many files shrugs it off and recovers; a job whose **first instruction is a single critical read** is killed by it and returns nothing after burning its tokens — this cost two models three runs each on the same job. **So inline the artifact a job must have into its prompt** rather than pointing at it, and leave file reads for what is merely useful.

**Model slugs**, from `agy models` on 13 November — re-run it rather than trusting this list, because it is the authority and this is a copy: `gemini-4-argon-{low,medium,high}` (no xhigh), `gemini-4-hydrogen-{low,medium,high,xhigh}`, `gemini-pro-orchestrator-g1-pro`, `claude-opus-5-5-{low,medium,high}`, `claude-sonnet-5-5-{low,medium,high}`, `gemini-3.8-flash-{low,medium,high}`, `gpt-oss-120b-medium`. There is **no frontier GPT on Antigravity** — only the open-weights one — so Codex stays the GPT route.

**Running detached**, so a job outlives the session that started it (S100):
`Start-Process powershell -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','.\scripts\gemini.ps1','-Job','<id>','-Model','gemini-4-argon-high') -WindowStyle Hidden`

**The five traps that have each cost an hour:**
1. **`Start-Process` does not quote its arguments**, and this repo's path has a space in it. Every runner quotes any argument containing whitespace; a new one must too, or the model receives `with` as a flag.
2. **A job prompt is far past the Windows command-line limit.** It goes in on **stdin**, never in `-p`.
3. **Antigravity soft-denies silently.** In headless mode there is no approval prompt: a disallowed action is refused, a note goes to **stderr**, and the run **still exits 0**. The exit code is not a gate — read the report and the stderr.
4. **A job prompt must cite the main checkout by absolute path** for anything not on its branch. Two jobs of the 13 November wave could not read reports they were told to read, because the worktree predated them.
5. **Never write a BOM.** PowerShell's `-Encoding UTF8` adds one; it breaks Antigravity's `settings.json` before 1.5.8 and resets the rig's `options.txt` to defaults. Use `[Text.UTF8Encoding]::new($false)`.

**Before launching anything**, check what is already running, or you will exceed a slot limit and drain a window for nothing:
`Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match 'exec -m' }` for GPT jobs, `Get-Process agy` for Google ones, and `Get-Content lab\results\queue.log -Tail 3` for the GLM queue.

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
- Route A is **Julien's call, not the integrator's**: it trades a grey-zone ToS reading against the only access to Argon, on a plan he has just paid for. Nothing is installed and nothing is signed in until he decides.
- Which family's errors are least like which is unmeasured. Until it is, the judge rule stays "the other family", as AGENTS.md has it.
