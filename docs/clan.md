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
| **Argon** | **Google** | **claimed: orchestration support** | **holds more of the corpus at once than any of us: contradiction and completeness passes over every doc, job report and run record together** |
| **Hydrogen** | **Google** | **claimed: Checked (wide)** | **a second cheap-wide lane beside GLM, on the same lens queue, graded against GLM's own reproduced answers** |

**Why those two claims and not others.** Our bottleneck is not building — Terra builds. Per [HANDOFF](HANDOFF.md) it is the planner's representation (taste, and Opus has it), measurement throughput (GLM has it), and **the integrator's own context**, which Julien named in S102. A frontier model whose published edge is long context is pointed at the third. If the measurement says otherwise, the rows change; they are written here so they can be wrong in public.

## 4. The bake-off: no row stands on reputation

A companion's row is a hypothesis, settled the way everything here is settled — against a known answer, by a check we already trust (`lab/glm.md`, "Canaries").

**The protocol.** Give the newcomer a job **we have already done**, whose answer is on disk and whose grader is a script, and grade it blind.

- **Checked band:** a lens from `lab/queue.txt` that GLM has already reproduced. Grade: did it reproduce, in what wall time, at what cost, and did its own independent checker agree? A newcomer that produces a confident wrong result its own checker passes is worse than useless, and is refused the band.
- **Judged band:** a diff we have already judged, including at least one whose verdict was **FAIL**, with the verdict withheld. Grade both halves: did it find the blocking finding, and did it invent blocking findings that are not there? A judge that fails everything is not a judge.
- **Frozen band:** a spec we have already red-teamed, with the red-team's report withheld. Grade: how many of the known changes it finds, and whether it finds one we missed.
- **Orchestration support:** a contradiction we know is in the corpus — the HANDOFF reading corrected in [learning.md](learning.md) is one, and the register holds others. Grade: found, with both files named, or not found.

**Canaries both ways.** Some graded items are subtly wrong on purpose. A companion that passes a planted wrong one does not get its band, whatever it scored elsewhere.

**The cost column is not optional.** Two companions within noise of each other are the same companion, and the cheaper one takes the work (ES 27: the goal, never a proxy).

## 5. How a job is assigned, in practice

1. Name the band. A machine checks it → Checked. Only another mind checks it → Judged. Nothing checks it later → Frozen.
2. Checked: the cheapest companion whose bake-off row covers that shape, as many lanes as its window sustains.
3. Judged: a family the builder is not in, fresh context, the spec in hand and the verdict unwritten.
4. Frozen: the best mind, then a red-team by a different family before it freezes.
5. Record the outcome. The census in `docs/research/roster-evidence.md` is regenerated by `scripts/roster-census.ps1`, so a row moves on evidence rather than on how a job felt.

## 6. Open

- Argon and Hydrogen have **never run here**. Every row for them in section 3 is a claim with zero evidence behind it.
- Whether a headless, detachable path exists for them on the AI Pro plan decides whether they join as companions at all, or only as thinking partners Julien talks to. Until that is settled, nothing in the plan depends on them.
- Which family's errors are least like which is unmeasured. Until it is, the judge rule stays "the other family", as AGENTS.md has it.
