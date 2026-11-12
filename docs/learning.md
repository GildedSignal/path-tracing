# Learning: one loop at every scale

**Status: draft, 12 November 2026. Red-teamed the same day (Fable, fresh context): DO NOT FREEZE.** Julien's turns S83–S89 ([reference](reference/julien-2026-11-10.md)). Section 1 stands. Section 2 is the loop as corrected. Section 3 is what waits for demand. [vocabulary.md](vocabulary.md), [scales.md](scales.md) and [representation.md](representation.md) win wherever this draft disagrees, until he rules. **Nothing here is built before the first flight says where the bots fail.**

## 1. What stands

- **Learning and acting are different processes** (S85). Acting says what is known. Learning opens on a failure.
- **Only an exact layer decides; a lossy layer orders or bounds** (S84, S87). The server judges. The law certifies. The class and the bias order. A bracket discards by proof or ranks. This is what keeps the thin bridge.
- **The puzzle is the unit of demand** (S87–S88): from this state, in this world, reach that goal. A map a human built to be hard, a trip, a command. Momentum carries across jumps, so a puzzle's start state is part of it; whether the unit is one jump or a course between checkpoints is his to say.
- **Nothing is removed for disuse** (S88). What matters is the size of the consequence, not how often it comes. A refuted word is tagged under the law version that refuted it, not deleted.
- **Failures stay small** (S89, Taleb): by the jump, never by the trip; live, only a certified sentence whose exit has a certified way to a safe state. A fall over the void is ruin, not a small failure.
- **Zipf and Heaps emerge** (S89). They are read, never targeted.
- **Learn the function, not the jumps** (S90: "find a function and you see where it solves things"). Between events the law is one small rule per tick (`movement.rs`, `travel_air`): add an input vector whose size the class's facts choose (on ground, sprinting, the friction of the block below) and whose direction the keys and the yaw choose, move, multiply by friction × drag, apply gravity. So a word is one rule applied k times, a run-up is that rule's approach to its gait, and what a jump can reach is computed, not searched. A class of jumps is the region of puzzles one function solves: the function comes first and the class falls out of it. Measured so far: gaits and free arcs are exact arithmetic. Lane `generative-function` tests the rest.
- **Four dimensions** (S90: "you think in 3D not in 4D"). A tube's cells carry their ticks. A piston, a clock, another bot's edit at a known tick are facts in space and time, not surprises. The bot knows when it arrives, so thinking done while the body carries out a certified sentence costs no T.
- **Minimise T, thinking included** (S90). There is one objective. Whether to think or to take the known route is the same sum: time already spent, plus thinking that the body must wait for, plus the route that results. It needs an estimate of the thinking a puzzle will take, and the two envelopes give it: inside what the vocabulary reaches, a lookup; outside what the law allows, none, go round; between the two, heavy-tailed.
- **The zone to learn in** (S90) is that same band: puzzles the law proves possible and the vocabulary cannot yet do, nearest the vocabulary first. No number of ours is in it.
- **Decided by him, 12 November:** the idle slice stays; one bot first; his runs replayed through the law as a diagnostic, never as a hint.

## 2. The loop, as corrected

1. **Say what you know.** The context pulls the words; the grammar chains them; the law replays the sentence.
2. **A failure names the bottleneck.** No sentence arrives, or his time beats ours. And a small slice of idle time keeps searching words that are trusted, drawn in proportion to T × traffic (vocabulary.md section 11): where a sentence always arrives, as on open terrain, nothing ever fails, and without this slice a missing gait is never found.
3. **Search in the law.** Every exact state not proven dominated is kept: the class orders them and never merges them (vocabulary.md section 3). The order must be fair, so that every proposal is reached at some budget; a cap on an unfair order is the bias deciding. The search is a semi-decision: it can find, it cannot always refuse. Its budget comes from the one objective (section 1): think as long as the expected T saved, over every bot and trip that will meet this puzzle, exceeds the T the thinking costs. The law has no backward step (clamps and rounding lose information), so "from both ends" means forward search from a set of takeoff states, and a takeoff left out of that set is not discarded by proof.
4. **Keep the fact as a program.** The solved puzzle is filed as its sentence (programs and arguments, each ending on an event) with the contents of the cells it read. Raw inputs are only a witness: a one-ulp law fix breaks a tick-by-tick recording and may spare a program (representation.md: a word "is not a tick-by-tick recording").
5. **The envelope is derived, not swept.** Arcs and gaits are exact arithmetic (`arc-certificate`, `gait-modulo`), so the set of goals a collision-free arc reaches from a takeoff state is computable. "Is this displacement inside it" is then an exact test with no number of ours, and that test is the key by which a rare word is found by need. Until it is derived, "need" is not a key.
6. **Two exams.** After a law change, re-prove the facts whose footprints read what changed (lazily, scales.md section 2). After a vocabulary change, re-solve a sample of puzzles by saying what is known. The second tests the vocabulary; the first cannot.
7. **A bound is a certificate.** It carries its footprint, the law's hash, the toolkit and its premises (the speed limit used). An upper bound has a witness to replay; a lower bound has none, so a wrong one is silent and prunes the true best forever: lower bounds are re-derived, never trusted across a law change. With one mind there is one store; merging matters only if lab learners feed it.
8. **A new kind of word is exposed in proportion to its confirmation.** One bot first. The server confirms about 10⁵ times slower than the law proposes, so the unconfirmed backlog is its own queue, ordered by T × traffic.

## 3. Parked until a measured bottleneck asks

The bias beyond counts (3.2c measured 99% hits); the puzzle generator at envelope edges (a solved edge makes a new edge: self-feeding, and its puzzles could pass as the "second sighting" of a word); compound words by compression (a shortcut is better kept only when no equal chain exists, the contraction-hierarchy rule, read from memory); merging knowledge across learners.

## 4. Loops that can run away

| Loop | Runs away as | State of the damper |
|---|---|---|
| Words shape what is seen | lock-in | idle regret sampling, in the record; his time, only where he ran |
| The bias steers the search | hardening on censored data | fair order; share of solutions found below rank k. Not built |
| The law changes under the words | damage growing with the store | programs, not recordings; re-prove by footprint. Lane `law-change-rebuild` plants narrow changes only, so it flatters: a rule every tick reads must be added |
| The law is wrong and the words share it | one large failure, all bots at once | exposure by confirmation. "Kind" is not defined; the push between bots has no producer today |
| Search finds the law's bugs first | words built on a bug | scales.md section 4.2 |
| Many bots fail at once (a hub edit, an update) | identical searches open together | one search per content key; fall back to a known sentence |
| Dominance proven once | the dominated route is the one needed under a crowd or a new law | re-prove with the law's hash; keep alternates |
| Edits break certificates, bots replan, bots edit | cascades | open: no model |
| Success | silent decay | hard maps; the idle slice |

**Speeds:** scales.md section 2 is the list. The law is today the fastest-changing layer (a port delta in every GLM batch), which breaks the order the draft assumed. A layer starts only when the one below changes slowly enough that rebuild cost × change rate is below what the layer saves.

## 5. Cases still open

No vocabulary (holds in principle; the alphabet is real yaw, a float, not the 64-aim grid; cost unbounded); 10⁹ words (failing now at 24 proposals: the lookup is 49× its floor, `idiot-index`); an impossible puzzle one sixteenth too far (inside the reach cone, so never proven; an open list with no principled retry); a solution with no margin (noise is a tick's delay, port error and pushes, vocabulary.md section 7); a human edit mid-jump; a mechanic the port lacks ("not found yet" and "the law lacks it" look the same from inside); a binade crossed inside one jump; 1,000 bots on one site (in four dimensions they hold different ticks of the same cells; unmodelled); one mind as one point of failure; words learned on parkour that do not transfer to terrain; a human bar set with another toolkit or by one tick.

## 6. Kill tests

1. **The smallest: fly his 18 tracks with the v2 words and a planner, and count the jumps where no sentence arrives or his T beats ours. Zero means this loop has no demand.** It is job 3.2b, already on the critical path.
2. A solution does not transfer to other jumps of the same shape (lane `jump-puzzle`, questions 3 and 4).
3. A law change to a rule every tick reads breaks most stored programs.
4. Cost per new puzzle does not fall as puzzles accumulate.
5. Exam core-seconds per law change exceed search core-seconds saved between changes.
6. A planted wrong law: does any lower bound survive it?
7. Held out: words learned on parkour, tried on terrain.

## 7. Sources, and how far each was read

Read in full, 12 November: Anthony, Tian, Barber 2017 (Expert Iteration; 9×9 Hex only, no convergence proof); Silver et al. 2017 (AlphaZero; exploration by root noise with a tuned constant per game); Ecoffet et al. 2021 (Go-Explore; its cell archive is a lossy merge we may not copy; its robustify phase is what turns inputs into a policy); Ellis et al. 2021 (DreamCoder; it trains on generated tasks); Wang et al. 2019 (POET). Taleb: his own highlights of the four books; Taleb and Douady 2013, abstract only. Isaacson's *Elon Musk*, his highlights.

**Not reached, so from memory and unverified:** Sutton, Precup, Singh 1999 (keeping primitives beside options keeps optimality; re-deciding mid-option beats running to the end); Kahneman and Klein 2009; Frazzoli, Dahleh, Feron 2005 (chain when one exit set lies inside the next entry set); March 1991 (exploitation drives out exploration); Simon 1962; the contraction-hierarchy witness rule.
