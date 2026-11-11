# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

1. Read [PRINCIPLES.md](PRINCIPLES.md) (with "From Emergent Space": ES is Julien's engine and the reference for representation), [AGENTS.md](../AGENTS.md), [representation.md](representation.md) (the framework), [vocabulary.md](vocabulary.md) (the frozen word record), then this file.
2. Run `git status --short`, `git log -8 --oneline` and `git worktree list`. Read `lab/wiki/index.md` for what GLM has settled.
3. Julien's turns of 9 November are in [reference/julien-2026-11-09.md](reference/julien-2026-11-09.md) (S44–S63), those of 10 and 11 November in [reference/julien-2026-11-10.md](reference/julien-2026-11-10.md) (S64–S71), and the register in PRINCIPLES cites each.

## Now (11 November, afternoon)

**Start here, in this order.** A new session cannot message an old sub-agent, so it reads results from disk.
1. **GLM batch 5** (11 November, 17:56, three lanes, 60 minutes): `l0-powder-snow-freeze` (row 1; engine.md already classes `ticksFrozen` as server-driven, read as input, never simulated, yet the law increments it at `movement.rs:170`), `certificate-size` (his question: bigger or smaller certificates, measured on his tracks), `step-up-reachability` (anomaly 1). The 16:19 launch never started: the secret scrub ran one grep per file over 110,727 result files (fixed, `e6bc400`).
2. **The Designer red-teams the yes-or-no class** (approved by Julien, 11 November: discrete facts only, a cut only where surprises concentrate, from the law's own comparison). Then vocabulary v2, with the certificate store question (below), then job 3.2b.
3. **The certificate store** (`engine/words/src/table.rs`, `certificates`): Julien said drop it once its reason is known. Its reason: draft 1 of the word record reused an exact hit with no replay; draft 2 made every use replay but kept the exact key, and 3.2a built it. Exact starts recur only on gait attractors (the cycle memo). Decide in vocabulary v2.
4. **Certificate size** (his, 11 November: "of course a parabola should be one certificate … we need to fermi this too"): the Fermi waits on lane `certificate-size`.

**The near goal (S55): about 5 bots with pathfinding and swarm behaviour working very well, before any serious optimization.** "Vanilla in behaviour, not pure vanilla." The critical path: L0 passes, then the word compiler (3.2a), the planner (3.2b), the executor, and the first flight (the bot races his 18 tracks, his T the bar). Then 5 bots on his server (a trimmed [server.md](server.md)). Swarm 0 and the scale work are parked (S53: 10–50 bots for now).

**Models.** GPT is out until **13 November, 13:29** (weekly limit, hit at 23:30 on 9 November). Claude's 5-hour window ran out twice on 10 November, and his Fable usage was nearly gone on 11 November at midday: spend Claude on judgment only, and give GLM everything a machine can check (S70). GLM's window holds about 18 lanes, and its usage bar lags. Julien now orchestrates with Fable (S63), and gave one Fable sub-agent at low effort for the job that most needs intelligence (S66).

**In flight, in order of what to do next:**

| # | Branch | State | Next action |
|---|---|---|---|
| 1 | `wt/l0-veto` (`b928a3e`) | Corrections as recorded server events (S62). The landing grammar is the jar's (GLM `l0-hurtmarked-writes`, reproduced; Sonnet judge PASS): damage, mark, writes, consume, dispatch, the consume bit-equal to the last write (`4e4e122`; the judge's text-order clock check removed in `b928a3e`). **Live gate, 11 November, afternoon** (target `l0-veto-v2`, evidence `l0-veto-20261111`): preflight and audit pass with 0 corrections; the full gate keeps **90 of 91 runs** (every run of 11 courses, powder snow 1 and 2), then refuses **powder snow run 3 (seed 91), tick 111: `END freeze counter`, law 3, recorded 4** (`.../20261111T1453381612953Z-fixtures/2573-verify.log`). The recorded counter runs 1, 2, 4, 4, 6, 7 over ticks 109–114; the law adds 1 per tick. Lead: `ticksFrozen` is synched entity data, so the server's value may overwrite the client's (a server event, like the landing). Edges and corner not run yet. | GLM lens `l0-powder-snow-freeze` pins the rule. If the capture holds the data, port and judge; if not, the recorder records it (a builder). A changed frozen source needs a fresh target: `oracles/1.6g-terrain-corpus/prepare.ps1 -Target <new> -ReferenceServer D:\path-tracing-lab\server` (no stderr redirect under PS 5.1), a fresh `-Evidence` with `archived-clock` copied over, then `run-live.ps1` on a quiet machine. Then judge and merge. |
| 2 | `engine/words` (on main, 11 November) | Job [3.2a](../jobs/3.2a-segments-and-gaits/prompt.md), built by the Fable sub-agent (S66), judged by Sonnet: pass. Segments, events, 54 programs, gaits (30 per open ground, 48 under a ceiling; the head-hitter at 19.902886), the proposal table as one `law::pattern` table. The law reports state reads through `World::read_state`; its arithmetic is unchanged (12 of 12 mutants caught). **The class is provisional** (S67: no made-up numbers): bins at 0.3/0.7 and 4 velocity bits give 98% reuse and 241 classes on `stonesimple` against 40% and 1,998 at the record's 1/16. | Talk with Julien first (S67–S69). Where the talk stands: the class is a cache key, and 1/16 was a rounding grid with no law behind it. Errors are misses (a search, 1–5 ms) and surprises (one wasted replay); neither gives a wrong path and neither compounds, since every use replays from the exact state, and both are counted per class for free. The integrator's proposal, not yet approved: discrete facts only (no position grid, no velocity bits), then add a cut only where surprises concentrate, taken from the comparison the law made there (his reading: "booleans and bounding", ES's tube intersections). GLM `class-without-numbers` (reproduced): on all 16 standable tracks the built class is the most expensive of four variants; dropping horizontal position costs 27–51% less, discrete facts only is cheapest on 6; misses (2,000 µs) dominate surprises (45 µs), and surprises concentrate on `movement_this_tick`, then crouching; no cell read removes any. Then a Designer red-team, vocabulary v2, and job 3.2b. |
| 2b | on main (`20f2809`) | Job [1.6](../jobs/1.6-read-cost/prompt.md), merged and pushed 11 November: a certified replay 47 → 11 µs p50 per segment (p90 6–8× lower), lookup about 5; the recorder's byte window and the snapshot's reach visit (most cells Shape-only; Sonnet judge: contract unchanged). Derived floor about 2 µs. Not built: the lazy certificate key, recording only on a miss (parked), a shared answer for the four corner suffocation checks. | Settles anomaly 7. The miss price (a local search, 2–20 ms derived) is still unmeasured. |
| 3 | job [1.5](../jobs/1.5-parallel-live/prompt.md) (spec only) | Over-capture live recordings, pin cores, several clients at once (S61). | After L0 merges. This removes the "quiet machine" constraint. |
| 4 | Ground layer, batch 2 | Reproduced in `lab/wiki`, not yet in the law: the use-item slowdown (the law panics `unimplemented!` on it today), scaffolding descend, placement replaceability, boats (rider), elytra start, fluid source conversion (fix over-conversion first), falling blocks, the water column under a broken floor (GLM `1452-1-3`, reproduced, with a `candidate.patch` in the run folder; the harness misfiled it and 12 others as leads on a first-line rule, fixed 11 November). | Port only what has a consumer: the use-item slowdown now (bots eat); the rest at Phase 4. One builder, witness red to green per delta. |

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
7. *Settled 11 November (job 1.6):* the replay now costs about 11 µs. **Was: a certifying replay costs about 45 µs per segment, not the record's 1–6 µs** (job 3.2a, judged: lookup 3.0, the law alone 5.8 over 1.55 ticks, the rest is recording). The law makes about 558 `block()` calls per tick for about 95 distinct cells (the builder's one-off probe), and the recorder pays a map insert for each. It blocks 3.2b's Fermi (vocabulary.md, line 19). `WorldSnapshot` is also 3.7 µs per tick unrecorded against 0.6 on `BoxWorld`.
8. **The state-read hook costs the law 3–5%** (sprint-jump 609 ns per tick against 590 on the old main, 7 samples).
9. **3.2a's "second column" test is near-tautological** (the judge): it shifts the same start by whole blocks. The informative number, the hit rate from arbitrary offsets, is printed and not gated. The class redesign (S67) replaces the test.

**Open for Julien:** the class (S67–S69, row 2): he wants it to emerge from the law, with no made-up number, and to talk before anything changes; he has not yet answered the "discrete facts only" proposal. Also parked from S67 (GLM `debug-world` is reading the jar for it): the debug world type as a one-world check of the whole block table (verify first that a survival player collides there). He approved the plan review on 10 November (S65): it is [scales.md](scales.md), with presence as a tool, the three thin slices before scale work, and R&D's wider objective. He asked for GPT as a sub-agent (S65), but GPT is still out until 13 November 13:29 (tried 16:52: usage limit).

## Roadmap (approved 9 November, S48; reordered 10 November, S65)

Every phase ends in a **flight**, something Julien uses in the game, and has a **kill criterion** that re-derives the representation instead of patching it. After Phase 3 the scoreboard reorders the rest, largest gap first.

0. **One line of truth.** Done.
1. **The ground.** Done except L0 and job 1.5. Flight: the bot's runs on his tracks replayed offline at ≥ 10⁴× real time; his T is the bar (S56). Kill: no path below about 5 µs means the `World` interface is wrong.
2. **One mind in the server, v0.** [server.md](server.md): bots that don't observe, world changes streamed once, the vanilla client's packets, a self channel, lazy bots on quiet footprints, the engine as a separate process; bots are players (S52). Build the deviation stream and a sandbox server at 100×. Flight: Julien sees 10–50 bots as players (S53). The lone-bot chunk-ticking wall waits for Phase 5.
3. **Slice A, the vocabulary on legs, and the first flight.** The record is version 2 (11 November). The compiler (3.2a): segments, gaits, the class-keyed proposal table. The planner (3.2b): search over certified segments with the bit-parallel light cone and the slack allocator. The executor: re-anchors at every event, prices veto risk (S62). Rungs: R1–R3, R-retarget, his tracks; the rediscovery count starts. Flight: the bot races his tracks. Kill: segments per regime ≫ 10³, or vocabulary.md section 11's three lines on natural terrain (certified ticks per µs with the table, regret, the vocabulary's growth).
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
| Explanations in his vocabulary, grounded in the whole context: the why, the constraints, the scale (S67, S75–S77) | integrator | read every doc and his turns before explaining; speak in the docs' terms (word, certificate, worldtube, light cone, T), never in implementation shorthand |

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
