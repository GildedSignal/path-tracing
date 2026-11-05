# The GLM lab

Julien, 29 September: "Our goal is to make the fastest and most robust engine/compiler ever for that type of physics"; "GLM should have more autonomy not what it could change but on what it could experiment on"; "I don't like pass/fail we just need a good system that make sure it propose gold."

Many GLMs, each with a lens, get **autonomy of inquiry and no authority.** They choose what to experiment on, build and run anything on the lab's Linux machine, and never change `main`. They run on agonbox as the isolated user glmlab (no access to Julien's files, no route to his network), or on GitHub's machines, never on his Mac. An independent GLM checks every result. Only gold reaches the integrator.

## One run

1. **The author** reads PRINCIPLES, this file, the [board](board.md), the [wiki](wiki/index.md) (the lab's memory: the current best and the open directions per topic) and its lens. It picks its own question, aimed at a bottleneck on the board or at a bigger one it argues for. Then it does real work: derivations, code, experiments. It may edit any file in the container to experiment. `run.sh` saves the edits as a patch and restores the tree.
2. **Held out before the run.** Before its decisive experiment, the author writes `predictions.md` in its run folder: the cases it has not tuned on, and what it expects there.
3. **The checker** is a fresh GLM session with no author context. It reproduces the work from the dossier's commands, runs the held-out cases plus adversarial cases of its own, and checks novelty against `docs/HISTORY.md` and the docs. Then it grades the work and writes the abstract.
4. **Gold is checked twice.** A second checker runs blind to the first; gold stands only if both grade it gold (two independent finders, arch-v3 §3).
5. **Canaries calibrate the checkers.** `lab/canaries/` holds dossiers with known answers, some subtly wrong. Now and then a checker grades one blind; a checker lens that passes a bad canary is rewritten.
6. **The integrator** reads only gold abstracts, at most three a day, ranked by gain × probability ÷ cost of adoption. It records each decision (adopted or rejected, and why) in the board's taste log. Every author reads that log: it is how the lab learns what gold looks like.

## The dossier (`lab/results/<run>/dossier.md`)

- **Claim:** one sentence.
- **Bottleneck:** which one, the expected gain in orders of magnitude, and what it lets us delete.
- **Evidence:** the numbers, plus the exact commands that reproduce them (each ≤ 60 s where possible).
- **Predictions:** point to `predictions.md` and say what happened there.
- **What would make it wrong,** the cost of adopting it, and its fit with PRINCIPLES (quote the line).
- **The lens:** what this lens may have made you miss.

"Nothing worth adopting, and here is why" is a legitimate dossier. One substantive result beats five shallow ones.

## Grades, with reasons, never bare

- **Gold:** reproduced, and the held-out predictions held. It moves a board bottleneck by 10× or more, deletes a component, or shows that a premise is wrong. It can be adopted within one step.
- **Silver:** real, but one named experiment away from gold. The next run picks that experiment up.
- **Lead:** not reproduced, marginal, off-goal, or already known or tried (cite the H entry). Leads are kept, never deleted.

## Rules

- Every prompt begins with PRINCIPLES.md's goal and philosophy (`run.sh` prepends them).
- Floats may explore; only integers decide canonical outcomes.
- Cloud timings are Linux/x86: report orders of magnitude only. The integrator re-measures on the M1.
- Results live only in `lab/results/<run>/` on the run's own branch. Never edit or push `main`, the docs, the crates or frozen evidence.

## Running (never on Julien's Mac)

- **Pacing:** GLM's binding limit is its 5-hour token window, not its month. About 12 parallel sessions exhausted it in under 2 hours (29 September), and every later run got a 429 and came back VOID. So run at most 3 rung runs per window, and schedule second authors for the next window.
- **The host is GitHub Actions** (`.github/workflows/glm-lab.yml`), every 5 hours, so the lab runs when Claude and Codex are out. It takes three runs from `lab/next.sh`: the integrator's queue `lab/queue.txt` first, then the lens rotation. GLM's job holds a read-only token and no git credential; a separate clean job publishes each finished `lab/results/<id>/` on its own `claude/lab-<id>` branch, and refuses any other path. The secret `ZAI_API_KEY` is Julien's. `lab/setup.sh` installs omp, pinned by hash.
- The lenses are in [lenses/](lenses/). The integrator owns [board.md](board.md), `checker.md` and this file.

## Directed rungs
- `lab/run.sh rung <file> [minutes]` runs a rung file that is committed and unchanged: prose, a `configs` block (`<name> candidate|negative`) and a `criteria` block (`<id> behaviour|type|cost <metric> <op> <threshold>`).
- The author writes `metrics.json`, a checker reproduces it as `metrics-check.json`, and `verdict.py` decides: PASS (after a blind second checker), KILL, VOID or DISPUTED. A KILL kills configurations, never the method.
- [canaries/](canaries/) holds rung files with known verdicts: a checker lens that misgrades one is rewritten before real rungs count.

## Writing rung criteria (lessons from r2)

- **Controls on both sides.** Every rung has a `positive` control that must pass (an ideal configuration: the oracle itself, or the law-native exact reference) and a `negative` that must fail. If either misbehaves, the verdict is VOID.
- **No trajectory identity.** Never demand long-run identity with an external float truth. Rounding makes that a random walk, so the demand is error size in disguise. Judge behaviours and statistics.
- **Robust growth exponents.** Judge them on the median seed or on an ensemble fit, never on the worst seed of a noisy estimator.

