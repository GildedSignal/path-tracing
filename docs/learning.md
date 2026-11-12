# Learning: one loop at every scale

**Status: draft, 12 November 2026. Not frozen, not built.** Julien's turns S83–S89 ([reference](reference/julien-2026-11-10.md)). It goes to a red-team before anything is built from it. It extends [vocabulary.md](vocabulary.md) (the word) and [scales.md](scales.md) (every scale under one rule); where they disagree, they win until he rules.

## 1. The problem

The law tells exactly what one move does, for about 1 µs per tick. Choosing among moves is the cost. Trying everything from one state costs 28 ms (job 3.2c), and a hard jump is far beyond that. So the bot must act from what it already knows, and learn only where what it knows fails. Learning and acting are different processes (S85). This is a coupled adaptive system: what it knows shapes what it looks at (S84), and what it looks at shapes what it learns. Section 5 treats that as the danger it is.

## 2. Maps and territory (S87)

| Layer | What it is | Exact? | May do |
|---|---|---|---|
| The server | the territory | — | judge |
| The law | our port of it | yes, until the server says otherwise | certify |
| A solved puzzle | a start, a world, a goal, the inputs that arrive | yes, under one law version | be replayed |
| A word, an envelope | a solution cut at events; the family it belongs to | yes for the word, measured for the envelope | be proposed, then replayed |
| The class, the bias | which situations are alike; which word usually wins | no | order |
| A bracket | at least this long, at most this long | bounds | discard by proof, rank |

**One rule: only an exact layer decides. A lossy layer orders or bounds.** Every layer is rebuilt from the one above it in this table. The thin bridge (S84) is lost only when a lossy layer is allowed to decide.

## 3. The loop

The unit is a **puzzle**: from this state, in this world, reach that goal. A parkour jump is one (S87–S88); so is "cross this ravine", and "reach that biome". A puzzle is real demand: a map a human built to be hard, a trip a bot was asked to make, a command.

1. **Say what you know (System 1).** The context pulls the words that apply; the grammar chains them (an exit that satisfies the next entry); the law replays the chosen sentence. Microseconds.
2. **A failure names the bottleneck.** No sentence arrives, or a human's time on the same puzzle beats ours. Only then does search open (System 2). Work is admitted by a bottleneck on a goal someone has, never by a gap alone: lower bounds are loose, so everything looks improvable, and that is fake demand (S87).
3. **Search in the law, where failing is free.** From both ends: which takeoff states land on the goal, which run-ups reach one of them. An archive keeps the best exact state reaching each context, and returning to it costs nothing. If the puzzle resists, move the goal nearer until it yields, then walk it back, carrying the solution. Discard only by proof. Complete from the alphabet: one tick of held keys and an aim is always a legal move, so no capability depends on having a word.
4. **Keep the fact.** The solved puzzle is filed with its inputs: append-only, never revised.
5. **Compress.** Cut the solution at events; what no word covers is a new word. Slide the goal until the word fails: that boundary is its **envelope**, its generator, and the distance to it is the word's margin against our own execution noise. A sentence becomes a word only if the whole vocabulary gets shorter by it (the test DreamCoder uses), and only after two different puzzles needed it.
6. **Teach System 1.** The bias learns the whole ranking the search found, with the gaps in T between the alternatives, not only the winner (Expert Iteration's finding). It never excludes.
7. **The exam.** Every solved puzzle is replayed after any change to the law or the vocabulary. A word that no longer solves its puzzle is refuted.
8. **New puzzles, only at the edge.** A point just outside an envelope that a real puzzle came close to. Never puzzles for their own sake.

## 4. What compounds, what is removed

- **Compounds:** solved puzzles; envelopes; the law's agreement with the server; brackets, which only tighten inside one law version and one untouched region, so two bots' knowledge merges by taking the larger lower bound and the smaller upper bound, in any order.
- **Removed:** what the server or the exam refutes; attention to words proven dominated in a context and for a kind of exit (they stay in the archive); mechanisms no bottleneck asked for; numbers of ours.
- **Never removed for disuse (S88).** Keeping a rare word costs bytes; lacking it costs the map. What matters is the size of the consequence, not how often it comes (Taleb). A frequent word is found by context; a rare one by the need it answers, so every word is indexed by its effect as well.
- **Zipf and Heaps are not designed (S89).** They emerge. They are read as health signs, never used as targets.

## 5. The coupled system: each loop, how it runs away, what damps it

| Loop | Runs away as | Damper | Measured by |
|---|---|---|---|
| Words shape what is seen; what is seen shapes the words | lock-in: no word, so never tried | completeness from the alphabet; a human's time as an outside bar | puzzles where a human beats us |
| Search teaches the bias; the bias steers the search | self-confirmation | the bias orders, proof discards; optimism comes from bounds, not from tuned noise | exits no proposal reached |
| The law changes; every word rests on it | mass invalidation | words are a cache; replay the stored inputs, re-search only the failures | lane `law-change-rebuild` |
| The law is wrong in a way the vocabulary shares | coherent error, invisible offline | a word is trusted only after the server has confirmed its kind; checks ranked by damage if wrong, not by likelihood | deviations per new kind of word |
| Bots edit; certificates break; bots replan; bots edit | cascades | a certificate no longer than what breaks it pays for; claims in the medium | open: no model yet |
| Learning costs T now to save T later | endless study, or none | work only on bottlenecks, each search capped and escalated on return | search time per puzzle against T saved |
| Success: nothing fails, so nothing is learned | silent decay | hard maps keep arriving; no evidence of a faster way is not evidence of none | bracket width where words are trusted |

**Speeds must stay apart.** Server version, law, puzzles, words, bias, plans: each changes faster than the one before it. A slower layer changes only when the faster ones have settled, and one layer at a time, or two learners chase each other.

**Failures must stay small.** Many small failures teach; one large failure breaks (Taleb). So: failure by the jump, not by the trip; live, act only on a certified sentence with a way out; first cut the downside, then let the rare large gain arrive.

## 6. Cases and extremes

| Case | What happens | Verdict |
|---|---|---|
| No vocabulary at all | every puzzle is searched from the alphabet | holds, slow |
| 10⁹ words | lookup by context stays one hash; lists per context must stay short | open: needs dominance per context and exit, or a finer class |
| A puzzle with no solution | the envelope sweep or a reach bound proves it; otherwise capped, marked open, retried at growing intervals | holds if the bound exists |
| A solution with no margin | found in the law, fails live by a tick | open: execution fidelity is its own bottleneck (L0, job 1.5) |
| A human edits mid-jump | the certificate breaks in the air | open: sealed recovery words (SR-71), priced by the human's light cone |
| A mechanic the port lacks | the puzzle cannot be stood on | holds: the map decides what is ported next |
| Mojang updates | every upper bound is void; puzzles stay; replay, then re-search the failures | holds if lane `law-change-rebuild` says breakage is small |
| 1,000 bots, one site | invalidation storms; everyone picks the same route | open: congestion must enter T |
| A long trip | the same loop on coarser puzzles; regions summarised by their best possible crossing, bridges kept as exact facts | designed in scales.md, unbuilt |
| A world that moves (pistons, clocks) | a word must carry the world's phase, as a gait carries its own | open |
| Far from the origin | floats coarsen; a word proven near spawn may surprise | holds: replay certifies; expect more surprises |
| The mind's process dies | every bot stops | open: one mind is one point of failure |
| Tuned on parkour only | words that do not transfer to terrain | open: his real trips must be in the exam |

## 7. Kill tests

1. Search from both ends does not beat forward search by a wide factor on his hardest jumps (lane `jump-puzzle`).
2. A solution's inputs do not transfer to other jumps of the same shape, so envelopes are points, not families.
3. A small law change breaks most stored solutions, or re-proving them is not cheap (lane `law-change-rebuild`).
4. Cost per new puzzle does not fall as puzzles accumulate.
5. The exam grows faster than the vocabulary's use.

## 8. Sources, and how far each was read

Read in full, 12 November: Anthony, Tian, Barber 2017 (Expert Iteration: the apprentice imitates the search's whole distribution; shown on 9×9 Hex only, no convergence proof); Silver et al. 2017 (AlphaZero: exploration by root noise with one tuned constant per game, which we may not copy); Ecoffet et al. 2021 (Go-Explore: "detachment" is forgetting how to reach a place already visited, the archive and free return cure it, and a separate phase makes brittle solutions robust); Ellis et al. 2021 (DreamCoder: a routine joins the library when it shortens library plus programs; replays guard the recogniser against forgetting); Wang et al. 2019 (POET: hard environments were not solved directly nor by a plain curriculum, only through stepping stones with transfer). Taleb: his own highlights of *Antifragile*, *The Black Swan*, *Fooled by Randomness*, *Skin in the Game*; Taleb and Douady 2013, abstract only. Isaacson's *Elon Musk*, his highlights (question the requirement, delete, automate last).

**Not reached, so unverified:** Sutton, Precup, Singh 1999 (options); Kahneman and Klein 2009 (when intuition can be trusted); Frazzoli, Dahleh, Feron 2005 (maneuver automaton: gaits and the manoeuvres between them); March 1991 (exploration and exploitation); Simon 1962 (stable intermediate forms). What this draft takes from them is from memory and is marked here as such.
