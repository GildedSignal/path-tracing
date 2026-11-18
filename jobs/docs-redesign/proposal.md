# Proposal: the docs as a map that leads, enforced by code (18 November, S236)

Julien (S236): the docs "guide and [are the] source of directional truth, the code has to catch up to it"; "architectural detail and then links to more detail"; "no stale things"; "a synthesis of [PRINCIPLES] that's holistic"; "soft enforcement is not enforcement".

## What I measured

- **Read at every session today:** 11 files, 308 KB, about 108k tokens. PRINCIPLES alone is 109 KB (38k tokens), mostly the register of 236 turns.
- **Never read but in `docs/`:** another 284 KB (his verbatim 155 KB, research 87 KB, post-mortems 20 KB).
- **History inside the living docs:** PRINCIPLES 64 dates and 67 "was / superseded / once"; HANDOFF 19 dates; clan.md 15 dates, most of it vendor benchmarks from 13 November.
- **Duplication:** "store the generator" in 4 docs; "one mechanism" in 6 places plus memory; the launch and meter rules in AGENTS, clan 5 and 5c, HANDOFF and 8 memory files; the thinking floors in representation, HANDOFF, plan and a job report.
- **Memory:** 33 files, 48 KB, mostly copies of rules already in AGENTS and clan.

## The failure modes, and the incentive under each

| Failure (post-mortem) | The incentive that produced it |
|---|---|
| Docs not read, then claimed read (16 and 18 Nov) | reading is optional and costly; a summary is enough to answer; nothing checks |
| Design lived in job files, docs went stale (16 Nov) | folding into the docs has no consumer; the next launch is what gets rewarded |
| Specs checked against the last spec, judges against the spec (16, 18 Nov) | a judge sees only its prompt, and the integrator writes both |
| The wrong number reported for days ("warm", S223) | gates passed read as progress; a fast number pleases |
| Over-precision, tests as big as the code (S230, S235) | prevention feels safe; frozen gates cost forever and nobody pays |
| The wiki, lenses, digests, the register (S234, S235) | **"use every meter before it resets" (S100, S104)**: work was invented to fill meters |
| **Every post-mortem added prose rules** | a rule in text looks like a fix; so the docs grow, get read less, and fail more. This is the loop behind all the others |

The last row compounds: 14 November added 8 rules to clan.md, 16 November 4 more, 18 November 7 proposed. Each made the next failure likelier.

### From his own corrections, not only the post-mortems (the verbatim record, S1–S236)

| Mode | His words, a few of many | Root |
|---|---|---|
| Context not held | S77 "read the doc please all of it"; S58 "don't forget about ES again"; S117 "in case you forgot"; S163–S164; S231 | reading is a choice, and attention fades over a long context |
| Lost in detail, over-precision | X "You're getting lost in detail doing too much work where it does not matter"; S69 "aren't you overthinking it"; S232 "so much overthinking" | precision feels safe; nothing prices it |
| Made-up or wrong numbers | S67 "1/16 feels like a made up number"; S132 "don't already start with made up fake numbers"; S112 "i don't believe you"; S223; S225 "be more rigorous with your framing" | a plausible number answers the question; checking it costs a command |
| Waste in the bot's thinking | S110–S118: "why does it replay the route? is it anxious"; "430 decisions … wasteful"; "re-test tick by tick should never happen"; S187 "do we need clone?" | the code is optimised locally instead of held against its Fermi floor |
| Fake data, proxies | S124 "we're not already starting to fake it with fake data"; S220 "wasn't that old news"; S223 | a gate passed reads as progress |
| Wrong mind for the job | S104 a weaker orchestrator; S174 "why is terra building the core"; S175 "why would hydrogen write the specs"; S177 60% of a window re-sent | routing by what is free, not by what the job needs |
| Usage blindness | S168 "you're at 90% since you didn't realize"; S226 | meters are read only when remembered |
| Acting when asked to explain; my names, not his | S78 "explain better"; S179 "when i ask to explain you explain you don't do"; S179 "made up names"; S222 "steps and pass" | doing feels more useful than answering |
| Duplicate and busy work | S221 "it was duplicate, that was unserious"; S234–S235 | idle meters read as waste |
| A second path, a bandaid | S160 "that smells like bandaid"; S161 | a special case passes the test today |
| Companions blocked by us | S129 empty Antigravity runs; S210 "Hydrogen can't run code? what?"; 14 Nov: the contract that forbade reading code; Terra halting at its first open question | we restrict instead of letting the branch and the game catch errors |

**One pattern under most of them:** everything goes through the integrator, the integrator's context decays, and it compensates with restrictions and prose instead of with principles carried into every prompt.

## Agency (S237: "we don't give enough agency and ability to act to the models")

Companions work on their own branches, and the game and a judge check the result, so a mistake costs a fix, not a disaster (S235: "cheaper to fix than to prevent"). So:
- **Every companion may read the whole repository, run any command, build and test in its worktree, and decide how to reach its goal.** The prompt gives the goal, the principles, the map and the gate; not a list of files or steps.
- **Only irreversible things stay closed:** pushing, his world and server while he plays, secrets, git history, other branches, and launching paid runs.
- **A question no longer stops a job:** the companion decides, writes down what it decided and why, and goes on; the judge sees it.

## The few heuristics that are always on, and how to keep them on

His words: proper testing and proper thinking, never mediocrity or averages. In one card of about ten lines, each with its turn:
1. The goal and T; everything is priced in T, thinking included.
2. First principles and the Fermi floor first; the gap in orders of magnitude is the signal.
3. Question the requirement, delete, then simplify (S235).
4. One mechanism; a special case means the representation is wrong.
5. Store the generator; nothing relearned that is already known.
6. Precision only where a decision can flip.
7. Test like a Spartan: extremes and chaos, the game as the judge, distributions and tails, never an average.
8. Expected against seen: a delta is data, fixed when seen.
9. His words and vocabulary, never mine; explain when asked to explain.
10. Measure before claiming; report raw numbers and say what is not done.

**Why context forgets** (his question): instructions far back in a long context get less weight than recent text, and a compaction keeps a summary, not the rules. So a rule's position matters more than how often it was written.

**Skills or hooks:**
- **A skill** is loaded when the model decides its description matches, or when called by name. That is soft for always-on rules. It is right for **procedures used at a moment**: writing a prompt, judging, launching. It arrives fresh exactly when needed.
- **The card goes in a `UserPromptSubmit` hook,** injected with every message he sends: always the most recent instruction in context, about 300 tokens a turn. This is the strongest mechanical guard against forgetting.
- **A `PreToolUse` hook on `launch.ps1`** injects the prompt-writing skill at the moment of a launch, and refuses a big-model launch (Astra, Sol, Opus, an Argon build) unless the prompt has a `shown-to-julien` marker (S230).
- `map/` stays in the `SessionStart` hook, as built today.

## The structure

Three folders, by how often they are read:

1. **`map/`: loaded in full by the hook at every session and compaction.** Hard cap: 40 KB in total (about 14k tokens, an eighth of today). Present tense only. Each section is a few lines of structure, then a link to its detail.
   - `goal.md`: the goal, the scoreboard, the two extremes, the Fermi floors and the measured gap in orders of magnitude.
   - `principles.md`: the holistic synthesis of his philosophy and our current beliefs, rewritten when a belief changes, each line citing its turn (S-number) in the archive.
   - `architecture.md`: the law at three grains, the one loop, the objects (zone, joint, set, word × count, generator, certificate, worldtube, light cone), the math (min-plus for routes, max-plus for synchronisation), the kit as a law input.
   - `vocabulary.md`: his words and their definitions, so nobody imposes their own.
   - `work.md`: who does what and the rules **not** enforced by code (few), pointing at the scripts that enforce the rest.
   - `now.md` (today's HANDOFF): what is in flight, what is next, his open questions. Rewritten freely, no history.
2. **`map/detail/`: read on demand through the links.** Today's representation, engine, learning, scales, ladder and server, deduplicated and stripped of history. Present tense only.
3. **`archive/`: never loaded, searched when needed.** His verbatim turns, the register (today's PRINCIPLES), post-mortems, research, old specs. History also lives in git.

Name: `map` (it leads the territory, as he said: the code catches up to it). `canon` is the alternative.

## Enforced by code, not by text

1. **The hook** loads `map/` and nothing else (detail stays one link away).
2. **A pre-commit check** (`scripts/check-map.ps1`, run by a git hook) refuses a commit when: `map/` is over its byte cap; a `map/` file holds a dated or past-tense history marker ("14 November", "was", "superseded", "previously", "no longer"); a link in `map/` is broken.
3. **Code and map move together:** a commit that touches `engine/` must also touch `map/`, or carry a `Map: unchanged, because …` trailer. The hook refuses it otherwise. The map leads; when the code learns something (a measured number, a new mechanism), the map changes in the same commit.
4. **Every companion prompt gets `map/architecture.md` and `map/principles.md` appended by the launcher,** and every judge prompt a fixed last question: "does this obey the map?". The integrator can no longer forget to include them.
5. **One scoreboard script** prints the headline numbers (thinking per new command, T on his tracks, against the floors). It is the only number reported as progress.
6. **Rules budget:** a post-mortem fix is code, or a deletion. A new prose rule in `map/work.md` must replace one.

## The if-then graph (S238: "a sort of if then graph to follow … if it is busy work then the graph is bad")

Each edge is an event, an action, and the code that fires it. An edge with no consumer, or one that never catches anything, is deleted at the phase cleanup. That is the busy-work test. Each incident changes the graph (adds an edge, moves one, removes one), never the prose. That is the antifragile part: the system gets stronger from each failure.

| If | Then | Fired by |
|---|---|---|
| a session starts, resumes, clears or compacts | `map/` loads whole | `SessionStart` hook (built) |
| he sends a message | the 10-line card is injected | `UserPromptSubmit` hook |
| a big-model launch is attempted | the prompt-writing skill is injected; refused without the shown-to-him marker | `PreToolUse` hook on `launch.ps1` |
| any launch | the map's architecture and principles are appended to the prompt | `launch.ps1` |
| a job ends | its judge launches by the other family, with the map | the job's waiter |
| a commit touches `engine/` | it touches `map/` or says why not | git `pre-commit` |
| a commit touches `map/` | size cap, no history wording, links resolve | git `pre-commit` |
| a push | no private email, no Antigravity log | git `pre-push` (built) |
| a meter passes 80% | he and I are told | `meters.ps1 -WatchAt 80` |
| a measured number is reported | it comes from the scoreboard script, beside its floor | the scoreboard |
| his turn | verbatim to the archive; `map/` changed if a belief changed | `UserPromptSubmit` hook appends the verbatim; the map change stays mine |

## AGENTS.md (S238: "is agent.md good?")

Not good. It is loaded every session through `CLAUDE.md`, so its errors are read every session:
- **Stale facts:** it says the integrator is Fable 5.1 (this session is Opus 5.5), Sol's history, "four Terra" rules from S99.
- **Duplicated:** the team table and slot rules are also in clan.md, HANDOFF and eight memory files.
- **Prevention-heavy:** run records with worst margins, the merge door, component states, every surprise logged: the machinery S235 says costs more than fixing.
- **Links, not loads:** its "read first" list is the soft enforcement that failed.

It becomes about 20 lines: who you are in this project, that `map/` is the truth and is loaded, the irreversible list, and where the enforced rules live (the graph above).

## Incentives changed

- **"Use every meter" is retired.** An unspent meter costs nothing; invented work costs his attention and our context. Work enters only through a bottleneck on the goal (learning.md's own rule, S87).
- **Memory keeps only what is about him** (preferences, how he wants to be spoken to). Rules move to code or `map/work.md`; about 25 of the 33 files go.
- **clan.md becomes a 10-line routing table** in `map/work.md`; its benchmarks go to the archive.

## Order, fastest first

1. Move history to `archive/` (minutes, mechanical).
2. Write `map/` now, while every doc is in my context (I read all 11 in full today; after a compaction that read would cost 108k tokens again).
3. Strip and deduplicate `map/detail/`.
4. Add the checks (pre-commit, hook, launcher appends, scoreboard).
5. Argon lints `map/` against his verbatim turns once, then on each phase.

Astra's flat-ground job is the real bottleneck and quotes its rules inline, so it can launch at 15:21 in parallel; its prompt's doc paths get updated to `map/` before launch.
