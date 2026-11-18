# The overhaul: every issue seen on 18 November, fixed at its source (S227–S239)

One rule over all of it: **a fix is code or a deletion, never a new paragraph.** Each item says what is done, how a machine shows it is done, and who does it. Items in the same wave are independent and run in parallel.

## The issues, in one table

| # | Issue | Source of it | Fix (item) |
|---|---|---|---|
| 1 | The new core has never flown in his game | it was built beside the flight path; "every phase ends in a flight" was keyed to phase ends that never came | A1 |
| 2 | Docs not read, then said to be read | reading is a choice; the docs are too long to read whole | B1–B3 |
| 3 | Docs stale (AGENTS.md names Fable; history everywhere) | "update the doc" done as appending; nothing flags a false line | B1, B4 |
| 4 | Rules forgotten inside a long context | position decides weight; compaction keeps a summary | C1 |
| 5 | Job ends missed; he had to wake me | waiters armed by hand | C3 |
| 6 | Specs and judges checked against the last spec, not the docs | the prompt holds what I remember to put in it | C2, C4 |
| 7 | Wrong framing, made-up and wrong numbers (S223, S225) | a plausible number answers; nothing pins it to a script | C5 |
| 8 | Over-precision and prevention: tests as big as the code, frozen internal gates, the GLM checker | prevention feels safe; nobody pays for it | D1, D2 (tests already cut, `75b2b90`) |
| 9 | Busy work: wiki, lenses, digests, the register | "use every meter before it resets" | D3 (wiki and lenses already gone) |
| 10 | Companions over-restricted, over-monitored | rules written after each failure | D4 |
| 11 | Code never shrinks; cleanups lost | the cleanup trigger was "end of phase" | E1, E2 |
| 12 | Nothing of the new core on `main`; pushes rare | merging was never a trigger | E3 |
| 13 | Dead things kept: 256 GLM worktrees, ~45 branches, 137 job folders, 1.9 GB of results, the old A\* planner, the `words` crate | no deletion trigger | E2, E4 |
| 14 | Meters invisible to him in the app | `meters.ps1` only prints when run | F1 |
| 15 | My vocabulary instead of his; acting when asked to explain | habits, not checks | C1 (the card) |

## Wave A: the game is the judge (now; Astra while Codex is open)

**A1. Fly the new core in his game.** `plan-track` gets a mode that calls `solve` and expands its words into the tick script the mod already plays, through the law. `play-track.ps1` is unchanged.
- Done when: `play-track` flies `straight100`, `straight300`, `straight800`, `stonesimple` and ten new flat goals in his game with the new core; the track clock's T equals the predicted T; the deviation tool shows no delta; thinking per command is printed.
- Who: Astra builds the export as part 0 of `jobs/flat-ground` (prompt updated and shown to him first, S230); the integrator flies it when his PC is free.
- From then on **every builder's gate is a flight**, and the only tests kept are those the game judges.

**A2. Flat ground as the docs say** (`jobs/flat-ground`, already drafted): the gait stored once, the yaw put in, nothing relearned; judged in game by A1.

## Wave B: the map (the integrator, now, while every doc is in context)

**B1. `map/`**, loaded whole by the hook, 40 KB cap, present tense, structure first then links:
`goal.md` (goal, scoreboard, two extremes, floors in orders of magnitude) · `principles.md` (the holistic synthesis of his philosophy and current beliefs, each line citing its turn) · `architecture.md` (law at three grains, the one loop, the objects, min-plus and max-plus, the kit) · `vocabulary.md` (his words) · `work.md` (who does what, what is closed because irreversible) · `now.md` (HANDOFF, rewritten freely).
- Done when: the hook loads it, the size check passes, and a fresh read of `map/` alone answers "what are we building, how, and what is next".

**B2. `map/detail/`**: representation, engine, learning, scales, ladder, deduplicated, history removed, read through links.

**B3. `archive/`**: his verbatim turns, the register (today's PRINCIPLES), post-mortems, research, finished jobs. Never loaded; searched when needed.

**B4. AGENTS.md to about 20 lines**: role, `map/` is the truth and is loaded, the irreversible list, the if-then graph. No facts the environment already knows (model names, meters): hooks print those live.

## Wave C: the if-then graph, in code (the integrator, with B)

| If | Then | Code |
|---|---|---|
| a session starts, resumes, clears or compacts | `map/` loads whole | `SessionStart` hook (built) |
| he sends a message | the 10-line heuristics card is injected; his message is appended verbatim to the archive | **C1** `UserPromptSubmit` hook |
| any launch | the map's architecture and principles are appended to the prompt; every judge gets "does this obey the map?" | **C2** `launch.ps1` |
| a big-model launch (Astra, Sol, Opus, an Argon build) | the prompt-writing skill is injected; refused without the shown-to-him marker | **C4** `PreToolUse` hook on `launch.ps1` |
| a job ends | I am woken; its judge launches, from the other family | **C3** a background `SessionStart` hook that wakes the model, plus the waiter |
| a number is reported as progress | it comes from the scoreboard script, beside its floor | **C5** `scripts/scoreboard.ps1` |
| a commit touches `map/` | size cap, no history wording, links resolve | **C6** git `pre-commit` |
| a commit touches `engine/` | it touches `map/` or carries `Map: unchanged, because …` | **C6** git `pre-commit` |
| a push | no private email, no Antigravity log | git `pre-push` (built) |
| a meter passes 80% | he and I are told | `meters.ps1 -WatchAt 80` (built) |

The busy-work test: at each step's cleanup, an edge that never fired or never caught anything is deleted. The antifragile rule: each new failure changes an edge, never adds prose.

## Wave D: agency, not prevention

- **D1.** No test unless the game judges it (S235). Done: the suite holds only game-judged tests (`75b2b90`).
- **D2.** GLM's checker session is removed: one author per prompt; deltas are caught in the game.
- **D3.** "Use every meter" is retired: work enters only through a bottleneck on the goal (S87). An idle meter is not waste.
- **D4.** Companion contracts (`gpt.ps1`, `gemini.ps1`): read the whole repository, run anything in the worktree, decide how to reach the goal, and write down a decision instead of stopping on it. Closed: push, his world and server while he plays, secrets, git history, other branches, paid launches.
- **D5.** Memory shrinks to what is about him (about 8 of 33 files); rules move into code or `map/work.md`.

## Wave E: less code, more `main`

- **E1. Net lines on every merge:** a merge that adds lines must say what they buy; the pre-merge check refuses it otherwise. Each step ends by deleting what it made dead.
- **E2. Delete the dead code:** the old A\* planner (`flight.rs` modes 3/5/b, `planner.rs`) and whatever of the `words` crate `solve` does not call. Done when: the workspace builds, the flights pass, net lines are negative.
- **E3. Merge and push often:** the live line (`wt/warm-floor`: the core, joints and sets, the simplest case, the test cut) goes to `main` now; then every judge pass is merged and pushed the same hour.
- **E4. Archive, never lose:** the 256 GLM worktrees, merged or abandoned `wt/` branches, finished job folders, `lab/results` (1.9 GB) move to `D:\path-tracing-archive`; worktrees are pruned. Nothing is permanently deleted without his yes.

## Wave F: what he sees

- **F1.** A band under the app's context meter showing every meter (native 5-hour and week, Codex 5-hour and week, Zai, Antigravity Gemini, Claude and Argon), read from a cache that `meters.ps1` refreshes every few minutes.

## Order and who

1. **Now, in parallel:** Astra on A1 + A2 (Codex is open; about one run left this week) · the integrator on E3 then B1–B4 and C1–C6 · E4 by the integrator (mechanical).
2. **After A1 builds:** the integrator flies it in his game (needs his PC free).
3. **Then:** D2, D4, D5, E1, E2, F1.
4. **Last:** Argon lints `map/` against his verbatim turns once.

What is not done today is in `map/now.md`, never only in chat.
