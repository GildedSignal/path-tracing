# GLM: many cheap minds, no authority

Julien, 5 November (S20): "leverage good intelligence massively BUT without creating busy work, destroying things and taking your thinking away they need to be autonomous to some degree"; "Be very careful with no creating fake work". It follows ES's GLM lab (`lab/README.md` and `lab/heuristics.md` there), cut down to what paid off.

## The one rule that prevents fake work

**A GLM result counts only if a machine can check it.** It must come with:
- a command that reproduces it, run by a separate GLM session that never saw the author's reasoning;
- a failing case, a measured number that a committed script regenerates, or bytecode cited by class, method and offset.

Prose, surveys, plans and "ideas" are not results. "I looked here and found nothing, and this is why" is a legitimate result, and costs one line.

## What GLM does here

Each run has a lens: a question family where a found answer is checkable.

| Lens | It looks for | A result is |
|---|---|---|
| **Breaker** | where the Rust law and the bot's client disagree; where a plan beats the light-cone bound (impossible) or loses to the offline search or Julien's time | a recording or seed that fails the replay harness or the bracket, reproduced |
| **Worst cases** | inputs for the extreme tracks of a ladder: ties, grazes, far coordinates, gap limits | a fixture that breaks something, or survives with its margin measured |
| **Jar reader** | precise questions about Mojang's code that a spec needs | the answer with `javap` evidence, re-derived by the checker |
| **Pre-freeze red-team** | how a gate could pass while the thing is wrong | a concrete passing-but-wrong case, before the gate freezes |
| **Probe** | a measurement the plan needs (border states per chunk, table sizes) | numbers plus the script that regenerates them |

GLM never builds production code (GPT does), never designs (Opus does), and never writes docs.

## Autonomy of inquiry, no authority

- Inside its lens, a run picks its own question and does real work: code, experiments, derivations.
- It writes only in its own run folder (`lab/results/<run>/`) on its own branch. It never touches `main`, `docs/`, `engine/`, `bot/`, the gates or the fixtures.
- An independent GLM checker reproduces each claimed result. A result that doesn't reproduce is graded **lead** and kept, never deleted.

## The integrator's attention is the scarce resource

- The integrator reads verdict lines only, plus the reproduced results, ranked by what they unblock.
- A lens that produces no reproduced result in five runs is paused, and its prompt rewritten or retired.
- Canaries: now and then a checker gets a dossier with a known answer, some subtly wrong. A checker that passes a bad one is rewritten before its grades count.
- The lab itself is on the scoreboard: reproduced results per window, and what each one changed.

## Pacing and the key

- GLM's binding limit is its 5-hour token window. In ES, about 12 parallel sessions spent it in under two hours. Start with three runs per window, and pace by `omp usage`.
- The key is stored by `scripts\set-glm-key.ps1` (one click, a Windows dialog, encrypted for Julien's account). It goes only into an omp process's environment, and runners delete any output file that contains it.
- **GLM runs on GitHub's machines** (S21), never on Julien's PC: it runs with full autonomy (`--approval-mode=yolo`). Its job gets the key as a secret and no git credential; results come back as workflow artifacts.
