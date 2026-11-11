# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

1. Read [PRINCIPLES.md](PRINCIPLES.md) (with "From Emergent Space": ES is Julien's engine and the reference for representation), [AGENTS.md](../AGENTS.md), [representation.md](representation.md) (the framework), [vocabulary.md](vocabulary.md) (the frozen word record), then this file.
2. Run `git status --short`, `git log -8 --oneline` and `git worktree list`. Read `lab/wiki/index.md` for what GLM has settled.
3. Julien's turns of 9 November are in [reference/julien-2026-11-09.md](reference/julien-2026-11-09.md) (S44–S63), and the register in PRINCIPLES cites each.

## Now (10 November, evening)

**The near goal (S55): about 5 bots with pathfinding and swarm behaviour working very well, before any serious optimization.** "Vanilla in behaviour, not pure vanilla." The critical path: L0 passes, then the word compiler (3.2a), the planner (3.2b), the executor, and the first flight (the bot races his 18 tracks, his T the bar). Then 5 bots on his server (a trimmed [server.md](server.md)). Swarm 0 and the scale work are parked (S53: 10–50 bots for now).

**Models.** GPT is out until **13 November, 13:29** (weekly limit, hit at 23:30 on 9 November). Claude's 5-hour window ran out at about 02:50. GLM's window holds about 18 lanes, and its usage bar lags. Julien now orchestrates with Fable (S63), and gave one Fable sub-agent at low effort for the job that most needs intelligence (S66).

**In flight, in order of what to do next:**

| # | Branch | State | Next action |
|---|---|---|---|
| 1 | `wt/l0-veto` (`4bada51`) | Corrections as recorded server events (S62, ladder.md's L0 pass line): recorder, `apply_position_packet`, replay, driver's veto rate, a planted 1-ulp control. 130 tests green; the saved ceilings capture replays all 420 rows. **Run live on 11 November** (quiet machine): preflight and audit pass with 0 corrections; the full gate keeps 32 of 32 runs on full blocks, slabs and stairs, step-ups and ceilings, then refuses on `ladders-vines` run 1 (seed 33): `missing hurtMarked consume`, a server velocity write (y = −0.0784) that the replay never consumes (`D:/path-tracing-wt/l0-veto/lab/gates/results/20261111T0926556663607Z-fixtures/0958-verify.log`). Water to corner not run yet. | A builder names the 26.2 rule behind that write (which path sets `hurtMarked` on a ladder course), ports it as a recorded server event, and re-runs. Then, on a **quiet machine** (no GLM, GPT or builds), run `jobs/1.3-l0-close/run-live.ps1 -Target D:/path-tracing-target/l0-veto-v1 -Evidence D:/path-tracing-evidence/l0-veto-20261109` (preflight, audit, full gate, all 13 courses), then judge and merge. Each new first divergence: name the 26.2 rule, port it, re-run. |
| 2 | `engine/words` (on main, 11 November) | Job [3.2a](../jobs/3.2a-segments-and-gaits/prompt.md), built by the Fable sub-agent (S66), judged by Sonnet: pass. Segments, events, 54 programs, gaits (30 per open ground, 48 under a ceiling; the head-hitter at 19.902886), the proposal table as one `law::pattern` table. The law reports state reads through `World::read_state`; its arithmetic is unchanged (12 of 12 mutants caught). **The class is provisional** (S67: no made-up numbers): bins at 0.3/0.7 and 4 velocity bits give 98% reuse and 241 classes on `stonesimple` against 40% and 1,998 at the record's 1/16. | Talk with Julien first (S67): the class as the move's precondition, cut by the law's own comparisons. Then the branch-count test, a Designer red-team, vocabulary v2, and job 3.2b. |
| 3 | job [1.5](../jobs/1.5-parallel-live/prompt.md) (spec only) | Over-capture live recordings, pin cores, several clients at once (S61). | After L0 merges. This removes the "quiet machine" constraint. |
| 4 | Ground layer, batch 2 | Reproduced in `lab/wiki`, not yet in the law: the use-item slowdown (the law panics `unimplemented!` on it today), scaffolding descend, placement replaceability, boats (rider), elytra start, fluid source conversion (fix over-conversion first), falling blocks, the water column under a broken floor (GLM `1452-1-3`: the checker reproduced it and a `candidate.patch` is in the run folder, but the harness graded it lead on a malformed dossier). | Port only what has a consumer: the use-item slowdown now (bots eat); the rest at Phase 4. One builder, witness red to green per delta. |

**Done on 9 November** (all on `main`, pushed):
- **Law speed:** 0.6–0.95 µs per tick (`cargo run --release --bin bench` from `engine/`).
- **The world reader** (`engine/world`) **and every block in his 18 tracks** (2.1a, 2.1b): 30,951 of 32,366 states modeled from one rule read from the jar, each refusal named, block data stored as its generator (90 KB).
- **Footprints, determinism and the pattern key** ([footprints.md](footprints.md)): the law reads no randomness and no clock; the key covers the identity frame only (Mojang's sine table breaks turns); 2–4 µs per lookup.
- **Ground batch 1:** swimming's flight and passenger terms; knockback as a recorded event.
- **L0:** the sprint flag and pose are server-driven inputs (engine.md). Full blocks, slabs, step-ups and 3 ceilings runs pass.
- **The lab wiki** (`lab/wiki/`, compiled by `scripts/lab-wiki.ps1` after each GLM batch; lanes read it first).
- **The word record is frozen** ([vocabulary.md](vocabulary.md), S60): the class proposes, the replay certifies.
- **The server spec** ([server.md](server.md)): red-teamed, with his rulings.
- **The ES refinements** (S59): representation.md, AGENTS.md, glm.md.

**Anomalies (open; each blocks only the claims that depend on it):**
1. **L0's step-up mutant may be equivalent.** GLM `local-20261109-2251-1-0` (reproduced) claims no vanilla geometry or jump arc can tell `mutant-step-up` from vanilla, yet L0's controls reported every mutant caught. Settle which holds before L0's verdict counts.
2. **The archived full-blocks server-saturation check** fails identically on main and `wt/segments` (bits 1100480512 vs 1101004800), after movement passes.
3. *Settled 10 November:* the terrain-only c is **19.902886 blocks/s** (a 3-tick orbit on blue ice under a 2.0 ceiling; GLM `local-20261110-1056-1-4`, reproduced). The old 19.810930 divided 199 tick-deltas by 200; `Speeds.java` was right. Phase 3's bounds use this c.
4. **Scaffolding's bottom cap is missing from the block table** (GLM `local-20261110-1452-1-4`, reproduced). The exporter samples four collision contexts, none with the feet inside the cell, so 14 `scaffolding[bottom=true,distance≥1]` states have no 2-pixel cap: vanilla stops a descending body at y+0.125 and the law falls through. It blocks the scaffolding-descent track. The fix is a fifth context in the exporter, not a per-block branch.
5. **Entity push has no producer in the law** (GLM `local-20261110-1452-1-5`, reproduced). `push_pair` matches the jar, but nothing calls it: two overlapping players get a velocity rewrite every tick in vanilla and none in the law. It blocks slice C's claims (five bots on one crossing), not slice A.
6. *Settled 10 November:* backward momentum buys nothing on a 1-block pillar (GLM `local-20261110-1452-1-2`, reproduced): the largest gap is 4.0625 from rest and 4.6250 with arrival speed (T=12), with or without backward input. The rediscovery rung drops it from the elite list.
7. **A certifying replay costs about 45 µs per segment, not the record's 1–6 µs** (job 3.2a, judged: lookup 3.0, the law alone 5.8 over 1.55 ticks, the rest is recording). The law makes about 558 `block()` calls per tick for about 95 distinct cells (the builder's one-off probe), and the recorder pays a map insert for each. It blocks 3.2b's Fermi (vocabulary.md, line 19). `WorldSnapshot` is also 3.7 µs per tick unrecorded against 0.6 on `BoxWorld`.
8. **The state-read hook costs the law 3–5%** (sprint-jump 609 ns per tick against 590 on the old main, 7 samples).
9. **3.2a's "second column" test is near-tautological** (the judge): it shifts the same start by whole blocks. The informative number, the hit rate from arbitrary offsets, is printed and not gated. The class redesign (S67) replaces the test.

**Open for Julien:** the class (S67): he wants it to emerge from the law, with no made-up number, and to talk before anything changes. Also parked from S67: the debug world type as a one-world check of the whole block table (verify first that a survival player collides there). He approved the plan review on 10 November (S65): it is [scales.md](scales.md), with presence as a tool, the three thin slices before scale work, and R&D's wider objective. He asked for GPT as a sub-agent (S65), but GPT is still out until 13 November 13:29 (tried 16:52: usage limit).

## Roadmap (approved 9 November, S48; reordered 10 November, S65)

Every phase ends in a **flight**, something Julien uses in the game, and has a **kill criterion** that re-derives the representation instead of patching it. After Phase 3 the scoreboard reorders the rest, largest gap first.

0. **One line of truth.** Done.
1. **The ground.** Done except L0 and job 1.5. Flight: the bot's runs on his tracks replayed offline at ≥ 10⁴× real time; his T is the bar (S56). Kill: no path below about 5 µs means the `World` interface is wrong.
2. **One mind in the server, v0.** [server.md](server.md): bots that don't observe, world changes streamed once, the vanilla client's packets, a self channel, lazy bots on quiet footprints, the engine as a separate process; bots are players (S52). Build the deviation stream and a sandbox server at 100×. Flight: Julien sees 10–50 bots as players (S53). The lone-bot chunk-ticking wall waits for Phase 5.
3. **Slice A, the vocabulary on legs, and the first flight.** The record is frozen. The compiler (3.2a): segments, gaits, the class-keyed proposal table. The planner (3.2b): search over certified segments with the bit-parallel light cone and the slack allocator. The executor: re-anchors at every event, prices veto risk (S62). Rungs: R1–R3, R-retarget, his tracks; the rediscovery count starts. Flight: the bot races his tracks. Kill: segments per regime ≫ 10³, or a proposal hit rate under about 50% on natural terrain.
4. **Slice B, one tool (S65).** R-lego: blocks and one gap past the sprint-jump limit ([ladder.md](ladder.md)). It needs the placement law (pinned in the wiki), operator segments with `writes`, `needs` and obligations, the law read backwards, the archive by effect, the obligation ledger and the discoveries log ([scales.md](scales.md) 4.1, 4.9). Kill: ≫ 10³ verifications per gap, a tool needing search code, or a found word that isn't reused.
5. **Slice C, five bots sharing one tool on his server (S55, S65).** Server v0 as specified, plus one reservation. Rungs: a shared crossing; a rolling bridge cleaned up by the last bot. Flight: he watches five bots cooperate.
6. **Then scale, by the scoreboard:** the medium and far field (R-scale, R-far), tube tickets and lazy bots, more tools (R6, R7, R8, R-brach, R10, the Baritone benchmark), the swarm at 10³ and 10⁴, the economy and building ((max, +), reverse demolition, prices as fields), then the cherry (learned ordering, the LLM overseer, the GPU where profiling points, the human-limits profile, and his easter eggs, S66: idle bots near someone typing look at them, sneak and nod; when a bot falls to its death the others mock it). The lab proposes words from 3.2a's merge on.

## Scoreboard

Rows: T over his time and over the bracket's upper bound; first-move latency at p99; cost per bot and server ms per tick at N bots; the deviation rate and the **veto rate** per capability (S62); the overturn rate per grain (scales.md 4.3); segments per regime and the proposal hit rate, from superflat to random terrain; distinctions created per second (ES's H_live); the rediscovery count; feedback-loop time. Each row shows its gap to its Fermi floor, in orders of magnitude.

## Tracked requests (AGENTS.md: a request made twice gets an owner)

| Request | Owner | Where it lands |
|---|---|---|
| Multipoles (FMM) for how T acts at a distance (S6, ten times in ES) | integrator, Phase 3 | the medium's far field: bounds and landmarks, with the gap between lower and upper bound as the stated error |
| Hundreds, then thousands, of bots with shared work (S18, S35–S36, S44–S45) | integrator, Phases 2 and 5 | one mind in the server |
| 100× iteration (S32, 6 November, S44) | integrator, Phase 1 | the clock, the offline fast loop, job 1.5 |
| Creativity that arises, stays and compounds, cheaply (S32, S44, S64) | integrator | [scales.md](scales.md) 4.1, with R-lego as its falsifier (slice B) |

## When a job ends

- **GPT:** `.\scripts\gpt.ps1 -Job <id> -Worktree <path> -Model gpt-6.1-sol|gpt-6.1-terra`. GPT can't commit; read `jobs/<id>/last.md` and `git -C <worktree> status`, then commit with `Built-by:`.
- **Claude builders:** Opus sub-agents in a worktree; they commit on their branch.
- **Then:** a Sonnet judge in a fresh context; fixes go back to the builder; squash-merge to main (keeps large WIP blobs out of the public history); run `cargo test` from `engine/` on main; push.
- **GLM:** `scripts\glm-local.ps1 -Lenses "<space-separated>" -Minutes 60`, at most about 9 lanes per window. The wiki recompiles itself; commit `lab/wiki/`. Read `checker.log` for HTTP 429 before trusting a "lead".

## Traps

- **Disk:** C: has about 8 GB free; the toolchain, builds and lab worlds go on D:. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=/d/rust/rustup`, `CARGO_HOME=/d/rust/cargo`, `CARGO_TARGET_DIR=D:/path-tracing-target`.
- **Live runs need a quiet machine** until job 1.5: CPU contention makes the client lag and the server correct it. No GLM, GPT or builds during a live gate. Gates take the rig lock and never run while Julien drives the bot.
- **Vanilla judge:** Lithium and C2ME never go into the bot's client or the reference rig. His own server may take a mod that keeps behaviour vanilla, once a need shows up (S55).
- **Codex sandbox:** no `%LOCALAPPDATA%` (use `scripts/stage-deps.ps1`), no git metadata, no access to the stored GLM key.
- **Reference rig:** `PATHTRACING_ROOT=D:\path-tracing-lab`, port 25566. Run gate scripts with Windows PowerShell 5.1, but write game files as ASCII: its `-Encoding UTF8` adds a BOM, `options.txt` then resets to defaults, and the accessibility screen blocks quick play.
- **Untracked `jobs/<id>/last.md`** in main blocks squash-merging a branch that tracks it: move it aside first.
