# Learning: one loop at every scale

**Status: approved by Julien three times.** Learning as one queue on 16 November (S169), folded into section 2. The learning idea on 12 November (S93), after a Fable red-team whose changes are written in. The unified plan on 16 November (S165: "this is a beautiful plan i approve it"; S166: the recommendations "seem sensible"), folded in the same day as section 0. His turns: S83–S96, S116–S134, S161–S166 ([reference](reference/julien-2026-11-10.md)). Section 0 is acting. Section 1 is what stands. Section 2 is learning's loop. The full argument, Fermi inputs and sources: `jobs/unified-plan/plan.md`.

## 0. Acting: one loop for every terrain

**One mechanism** (S161: "we can't have two paths like that "easy" or "hard" it should be the same mechanism"). No kind of terrain has its own code path. A spec, a design or a judge's prompt is checked against this section before it is accepted (postmortem-2026-11-16).

**The two extremes, held at once** (S131), are the test:
- a one-million-block straight line on plain ground with bare legs;
- a hard track. His definition (S166): "hard tracks are tracks that have many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that." The track `hardtrack` in his world is not one: "it's easy-medium" (S166).

**Why one loop can serve both.** Under a kit the world is a crystal with defects ([representation.md](representation.md) section 10). Where the ground gives the same answers to the kit's verbs, a held word settles onto its gait and travel is a count. A defect is where that symmetry breaks. Thinking happens only at defects, so it follows the terrain's complexity, never its length: "the terrains complexity should be a mirror of compute's complexity" (S131).

```
solve(from, to, kit, deadline):
  1. bracket.  LB from the kit's light cone. UB from what is known:
               a zone's kernel, two labels merged, or nothing yet.
  2. closed?   If no narrowing can change the next word, emit it and stop.
  3. extend.   Take the biggest word the kit holds for this displacement.
               One presence query over its worldtube returns its count
               and the first exception that breaks it.
  4. cut.      The exception's joints split the problem. Each part is
               solve() again, given the sets on its two ends, in parallel.
  5. compose.  Sets come backward in min-plus. The point goes forward.
  6. allocate. Work on the open bracket whose narrowing could change
               the decision due soonest (scales.md section 2).
```

- **Step 1 is the hardness signal** (S121: "a cheap way to tell "hey this seems hard let's take some time""). A closed bracket means go.
- **Step 3 is presence first and biggest first** (S120: "i don't care where they are just if they are present first"; S91: "you start with the biggest one i think it breaks and then you go smallest and smallest"). Every word takes a count ([vocabulary.md](vocabulary.md) section 5): on a plain the word breaks at B after 233,000 cycles; on a parkour, at the first lip.
- **Step 4 is segments in parallel** (S120, S131). Parts depend on the sets at their ends, not on single states.
- **Moving at once** (S120: "the bot should move instantly unless things are really hard"). The first word is emitted when its own bracket closes. The rest is thought while the body moves, and each committed prefix ends at a joint with a certified way on.
- **Thinking while moving is free compute** (S165: "a ladder climb grants us compute to learn so much that it can improve itself real time"). On `hardtrack` the first word is a 260-tick ladder.
- **The slide** (S94, S131). Candidates are compared by brackets first, loosely; only the best is tightened. A joint asks for a slide when a neighbouring plateau of its set has a smaller T-to-go that the upstream set can reach, and for none otherwise (S120: "an extremely simple metric that's local"). One comparison, no replay.
- **Changing your mind** (S121) is `solve` from the observed state. Tables are resident, so the cost does not depend on what was consumed.
- **A worldtube is checked once,** when its certificate is made (S117; S120: "if it fail get better simple as that"). (S166.)

**Is A\* needed?** (S132.) Not in acting. Inside a zone the answer is a closed form. Between known joints it is two labels merged. At the first sight of a site it is one backward wave over that site's joints: a shortest-path pass on the folded graph, run once per goal, kept, and never revisiting a node, because T only grows along a route (S89). The first word never waits for that wave when the coarse bracket already fixes it (S166). Best-first search over exact states found 0 of 18 tracks (below) and is gone from acting.

## 1. What stands

- **Learning and acting are different processes** (S85). Acting says what is known. Learning opens on a failure.
- **Only an exact layer decides; a lossy layer orders or bounds** (S84, S87). The server judges. The law certifies. The class and the bias order. A bracket discards by proof or ranks. This is what keeps the thin bridge.
- **The puzzle is the unit of demand** (S87–S88): from this state, in this world, reach that goal. A map a human built to be hard, a trip, a command. Momentum carries across jumps, so a puzzle's start state is part of it; whether the unit is one jump or a course between checkpoints is his to say.
- **Nothing is removed for disuse** (S88). What matters is the size of the consequence, not how often it comes. A refuted word is tagged under the law version that refuted it, not deleted.
- **Failures stay small** (S89, Taleb): by the jump, never by the trip; live, only a certified sentence whose exit has a certified way to a safe state. A fall over the void is ruin, not a small failure.
- **Zipf and Heaps emerge** (S89). They are read, never targeted.
- **Learn the function, not the jumps** (S90: "find a function and you see where it solves things"). Between events the law is one small rule per tick (`movement.rs`, `travel_air`): add an input vector whose size the class's facts choose (on ground, sprinting, the friction of the block below) and whose direction the keys and the yaw choose, move, multiply by friction × drag, apply gravity. So a word is one rule applied k times, a run-up is that rule's approach to its gait, and what a jump can reach is computed, not searched. A class of jumps is the region of puzzles one function solves: the function comes first and the class falls out of it. Measured so far: gaits and free arcs are exact arithmetic. Lane `generative-function` tests the rest.
- **T belongs to the state a bot arrives with, not to a block** (S91: "blocks don't have intrinsic T cost"). Order matters only through that state: the law's state is everything the future depends on. The rule's multiplier makes the state forget (ground friction × drag per tick on the ground, drag alone in the air), so momentum is remembered over a length the ground fixes. Inside that length, costs sit on the passage from one word to the next. Beyond it, a place has a cost per gait, which is what scales.md's far field and representation.md's "recurrent motions joined by transients" (his S40, S46) already hold. Any cost left on a block itself is only a lower bound. An ant trail as a cost is a category error in four dimensions (S93). What survives of it is a count of use: the doorways many tubes pass through become the far field's landmarks (scales.md: "hubs become landmarks by being used"), and the same count weights the idle slice.
- **The map plans; the search only reads it** (S97, X16: "a representation where you don't need a policy because the map is clear on where to go"). Measured on his 18 tracks (job 3.2b; checked by the Opus red-team, 13 November): where the cell field reaches A, 6 tracks find a sentence in 10–78 expansions and 3 stall: fences and fencemaze at one joint the field claims and no exact state realises, while redfakemaze (measured later, job 3.2f-b) never acquires an anchor at all - 0 anchored against 13,549 unanchored expansions, so it has no chain to A; 2 tracks find a sentence without the field reaching A. A\* alone finds 0 of 18: it replays 1,536 proposals per decision (64 aims × 24) at a decision per block crossing, under a bound about 3× below T, so it sees about 2 decisions deep; its bound never fired because it never had a sentence to compare with. So a good map makes the search trivial and a wrong map defeats it; the evidence is against the branching and the cell field, not against exact states. Acting is section 0: sets backward, the point forward, the slide, each worldtube checked once. Standard pathfinding (S92: "we're not remaking the wheel here") turns on the folded graph and in learning: labels and landmarks far, incremental repair when a premise breaks, reservations on tubes. **Search over exact states belongs to learning** (section 2, step 3), opened at a joint where no set arrives, for that joint's pattern only. Baritone, read 12 November, is A\* over hand-written movements on positions only: the wheel, without the state.
- **A set is states, not cells** (S97, X9: cells "are an intermediate scale representation while it should be emerging"). The cell stays as nature's address: block reads, footprints, premises, tubes. It is not the node of the backward pass: a T per standable cell is a block with its own T (S91), and a takeoff from a cell's centre after a run-up from rest forgets the momentum carried across jumps. A set is the arriving states one word carries to the next set, bounded by the comparisons the law makes: which surface, at least what speed, which ticks. A free arc is the same wherever it starts, so the takeoffs that land on a surface are that surface shifted back by the arc. A door is then a premise with a producer, not a wall. The set's record (plateaus between single-ulp breakpoints, premises, prices) is in representation.md section 10; it is a one-way door, red-teamed before it is built (U2).
- **Decisions live at switches** (X16: "it's just on where i park my bike that it'll matters"). Between two switches one word repeats and nothing is chosen; a trip's decisions count its switches and narrow joints, never its length. A regime is dropped only by proof: its lower bound against another's witnessed sentence. Where to refine is where a word's width goes to zero (his "contention points ... at the edge of what i think i can do"); the record already holds clearance and delay tolerance (vocabulary.md section 7).
- **Look backward without inverting** (S91: "not doing some kind of reverse kinetic"). A free arc is the same wherever it starts, so the places from which a tool lands on a goal are the tool's own reach table mirrored about the goal, and the run-up its takeoff needs says how far behind to look. Tools are tried biggest first; a tool "breaks" when its mirrored set holds no standable cell. Lane `backward-tools`.
- **Forward owns history; backward carries premises** (S95: "only forward will go through it and modify the subsequent search"). Only the forward pass knows what the bot has done: a boat pushed, a block spent. A backward set is a condition, not a state: "from here the goal is reached, if these cells hold these contents at these ticks and this is in hand". The forward state joins a set only if it satisfies the premises; a broken premise invalidates the sets whose footprint reads the change and no others, and a missing item becomes a sub-goal. Recalled, not read: goal regression in classical planning (Fikes and Nilsson 1971); incremental replanning (Koenig and Likhachev, D* Lite).
- **The state goes forward, its price comes backward** (S96: "it's an information problem"). The past is known only going forward; what a thing is worth later is known only coming back. So the backward pass carries, beside its sets, a price in T for each part of the state: a block in hand is worth the T it saves at the place that needs it ("keep it for there"). A price orders and bounds; the forward replay decides. The shake moves joints inside one sentence; a change of gait (a boat on an ice river) is another sentence, and the sign that one exists is the gap between the sentence's T and the bound for everything the bot could hold, crafting included, each at its price in T. Recalled, not read: Bellman's cost-to-go; Pontryagin's state and costate; liveness analysis in compilers; causal links in partial-order planning.
- **Four dimensions** (S90: "you think in 3D not in 4D"). A tube's cells carry their ticks. A piston, a clock, another bot's edit at a known tick are facts in space and time, not surprises. The bot knows when it arrives, so thinking done while the body carries out a certified sentence costs no T.
- **Minimise T, thinking included** (S90). There is one objective. Whether to think or to take the known route is the same sum: time already spent, plus thinking that the body must wait for, plus the route that results. It needs an estimate of the thinking a puzzle will take, and the two envelopes give it: inside what the vocabulary reaches, a lookup; outside what the law allows, none, go round; between the two, heavy-tailed.
- **The zone to learn in** (S90) is that same band: puzzles the law proves possible and the vocabulary cannot yet do, nearest the vocabulary first. No number of ours is in it.
- **Decided by him, 12 November:** the idle slice stays (its size is no longer chosen: section 2, S169); one bot first; his runs replayed through the law as a diagnostic, never as a hint.

## 2. Learning: one queue, one operation, one record

Approved 16 November (S169: "the learning plan looks fine for now so i approve it"; the argument is `jobs/unified-plan/learning-review.md`). Learning is as single as acting: **one queue** orders all its work, **one operation** does it (step 3), **one record** keeps it (step 4). Failure, regret, R&D, play and the census differ only in who put the item in the queue. A second scheduler, or joints searched together, is a second planner (kill tests 8 and 23).

**The budget is the free compute inside certified words** (S165: "a ladder climb grants us compute to learn so much that it can improve itself real time"). While a certified word runs, thinking costs no T (section 1). Fermi, inputs stated: a tick is 50 ms; 7 of 8 cores learn; a new joint pattern costs about 12 ms. `hardtrack`'s 260-tick ladder is 91 core-seconds, about 7,500 patterns, for a track with three narrow joints. One sprint-jump cycle is about 350 patterns against at most one consumed. So the route's own demand takes about 1%, and **no slice is chosen**: deadline work first (joints on the committed route with no set, then that route's open brackets, widest first), and everything left is R&D and play. The barbell appears without a percentage.
- **The queue** is the allocator's (scales.md section 2). An item is "narrow this bracket"; its value is (T it could save) × (the traffic that will meet it) ÷ (its cost); its deadline is the tick the body reaches the joint it feeds. It logs what each item was expected to save and what it saved (kill test 19).
- **Improving in real time.** A better certified suffix replaces the current one only at a joint the body has not committed to, and only if its T is lower by at least a tick. A tie keeps the incumbent.
- **The minimum effective dose.** A joint whose bracket is closed gets no more compute until its certificate breaks.
- **Acting is never delayed:** learning runs on the other cores and publishes by pointer swap (kill test 18).

1. **Say what you know.** The context pulls the words; the grammar chains them; section 0's loop composes the sentence from certified pieces.
2. **Demand is a missing set or an open bracket.** A failure names the bottleneck (S86): no sentence arrives, or his time beats ours. Where a sentence always arrives, nothing fails, so three more sources feed the same queue: **regret** on trusted words, in proportion to T × traffic; **surprises that helped**, since an arrival sooner than predicted is a candidate word as well as a deviation ("even a zombie can be precomputed and can even help us", X2); and **the census**: the world is on disk (S6: "we suppose the player is omniscient"), so the joint patterns along likely corridors are counted by content hash before any bot fails there.
3. **Search in the law, one joint pattern at a time.** The search is over the takeoff states of the joint that has no set; its plateaus are kept under the pattern's content key and that pattern is never searched again. Joints searched together would be a second planner (kill test 8). Every exact state not proven dominated is kept: the class orders them and never merges them (vocabulary.md section 3). The order must be fair, so that every proposal is reached at some budget; a cap on an unfair order is the bias deciding. The search is a semi-decision: it can find, it cannot always refuse. Its budget is the queue's (above); when the body must wait for it, the one objective decides: think as long as the expected T saved, over every bot and trip that will meet this puzzle, exceeds the T the thinking costs. A known member of a joint family lends its breakpoints: later members start their bisection there (kill test 24). The law has no backward step (clamps and rounding lose information), so "from both ends" means forward search from a set of takeoff states, and a takeoff left out of that set is not discarded by proof.
4. **Keep the fact as a program, with its status.** A joint's record is one of four: **found** (plateaus and a witness); **forbidden** (outside the derived envelope: a proof); **searched** (k work units, nothing yet: not a proof); **refused** (the law met an unported mechanic: a demand on the port, the refusal its witness). "Not found" is never read as "impossible". Two pattern keys whose plateau tables are byte-equal under a kit **merge** into one class, found by hashing the tables (ES 32): learning also removes distinctions ("A flower is air.", S134). The solved puzzle is filed as its sentence (programs and arguments, each ending on an event) with the contents of the cells it read. Raw inputs are only a witness: a one-ulp law fix breaks a tick-by-tick recording and may spare a program (representation.md: a word "is not a tick-by-tick recording").
5. **The envelope is derived, not swept.** Arcs and gaits are exact arithmetic (`arc-certificate`, `gait-modulo`), so the set of goals a collision-free arc reaches from a takeoff state is computable. "Is this displacement inside it" is then an exact test with no number of ours, and that test is the key by which a rare word is found by need. Until it is derived, "need" is not a key. **Smaller circles** (S90: "we should not learn jumps but classes of jumps and uncover the generative function"): for the joint family carrying the most T × traffic, check whether its breakpoints are the closed-form envelope's; when they are, the family is one generator row and its stored patterns are deleted.
6. **One exam.** After a law change, re-prove the facts whose footprints read what changed (lazily, scales.md section 2). A vocabulary change needs none: a new word can only lower a min, so it breaks no set, and "is there now a better sentence" is an open bracket already in the queue.
7. **A bound is a certificate.** It carries its footprint, the law's hash, the toolkit and its premises (the speed limit used). An upper bound has a witness to replay; a lower bound has none, so a wrong one is silent and prunes the true best forever: lower bounds are re-derived, never trusted across a law change. With one mind there is one store; merging matters only if lab learners feed it.
8. **A new kind of word is exposed in proportion to its confirmation.** A kind is a generator row or a joint family (one class signature). One bot first. The server confirms about 10⁵ times slower than the law proposes, so confirmation rides on trips that happen anyway: every executed joint confirms its pattern. The unconfirmed backlog is ordered by T × traffic.
9. **Play is a source of queue items, with skin in the game** (S97, S98: "how childs get better and test themselves instead of doing nothing"; it "could also create its own puzzles"). Its puzzles sit at the edge the plan makes exact: an empty set, a plateau whose width goes to zero, a bracket that stays open; each is a pattern trips did meet, with one thing moved toward the law's bound until the set vanishes. It is also the stressor for trusted words: the most-used gaits are perturbed first. Thinking play spends free compute. **Body play spends the scarce thing, a body's ticks against the server,** so idle bodies only confirm, by T × traffic, where every joint has a certified way to a safe state, and never with edits before the ledger has flown. A play result earns nothing by being solved: its share of the queue follows the demand that later hits its patterns (kill test 20). R&D and play are judged by their best hits, never cut for a low hit rate. Its price is first-move latency when a command arrives mid-jump. Not built; it needs the executor.

## 3. Parked until a measured bottleneck asks

The bias beyond counts (3.2c measured 99% hits); a puzzle generator beyond play (section 2, step 9; the risk stands: a solved edge makes a new edge, and its puzzles could pass as the "second sighting" of a word); compound words by compression (a shortcut is better kept only when no equal chain exists, the contraction-hierarchy rule, read from memory); merging knowledge across learners.

## 4. Loops that can run away

| Loop | Runs away as | State of the damper |
|---|---|---|
| Words shape what is seen | lock-in | idle regret sampling, in the record; his time, only where he ran |
| The bias steers the search | hardening on censored data | fair order; share of solutions found below rank k. Not built |
| The law changes under the words | damage growing with the store | programs, not recordings; re-prove by footprint. Lane `law-change-rebuild` plants narrow changes only, so it flatters: a rule every tick reads must be added |
| The law is wrong and the words share it (**fragile**) | one large failure, all bots at once | exposure by confirmation, per kind (section 2, step 8); the second-best plateau kept at each joint as an alternate, which the backward wave already computes, so a broken certificate is a lookup. The push between bots has no producer today |
| The store is trusted by every bot (**fragile**: learning's own ruin) | one wrong plateau, used everywhere | every entry carries its law hash and its witness; a refuted entry is tagged, never silently replaced; merges need byte equality |
| Search finds the law's bugs first | words built on a bug | scales.md section 4.2 |
| Many bots fail at once (a hub edit, an update) | identical searches open together | one search per content key; fall back to a known sentence |
| Dominance proven once | the dominated route is the one needed under a crowd or a new law | re-prove with the law's hash; keep alternates |
| Edits break certificates, bots replan, bots edit | cascades | open: no model |
| Success (**fragile**: no stressor) | silent decay | hard maps; play perturbs the most-used gaits first (section 2, step 9) |
| The queue rewards common, cheap joints | rare blockers starve (S88) | a joint with no sentence is worth its whole detour; nothing is removed for disuse |
| Play is rewarded for solving | easy puzzles, or ones no trip meets | queue share follows later demand hits |
| A slow first word buys thinking time | T traded for compute | the first word is chosen by its bracket alone |

**The asymmetry test** (Taleb, his highlight: "anything that has more upside than downside from random events (or certain shocks) is antifragile; the reverse is fragile"). The system gains from a deviation (witness, law delta, regression test: S81) and from an edit (local repair, new joints learned). It is fragile in the three rows marked above. Kill test 22 measures it.

**Speeds:** scales.md section 2 is the list. The law is today the fastest-changing layer (a port delta in every GLM batch), which breaks the order the draft assumed. A layer starts only when the one below changes slowly enough that rebuild cost × change rate is below what the layer saves.

## 5. Cases still open

No vocabulary (holds in principle; the alphabet is real yaw, a float, not the 64-aim grid; cost unbounded); 10⁹ words (failing now at 24 proposals: the lookup is 49× its floor, `idiot-index`); an impossible puzzle one sixteenth too far (inside the reach cone, so never proven; an open list with no principled retry); a human edit mid-jump; a mechanic the port lacks (answered by the four statuses, section 2, step 4); a swap mid-flight that changes other bots' reservations; whether the census stays bounded by real corridors; a binade crossed inside one jump; 1,000 bots on one site (in four dimensions they hold different ticks of the same cells; unmodelled); a human bar set with another toolkit or by one tick.

## 6. Kill tests

1. **The smallest: fly his 18 tracks with the v2 words and a planner, and count the jumps where no sentence arrives or his T beats ours. Zero means this loop has no demand.** It is job 3.2b, already on the critical path.
2. A solution does not transfer to other jumps of the same shape (lane `jump-puzzle`, questions 3 and 4).
3. A law change to a rule every tick reads breaks most stored programs.
4. Cost per new puzzle does not fall as puzzles accumulate.
5. Exam core-seconds per law change exceed search core-seconds saved between changes.
6. A planted wrong law: does any lower bound survive it?
7. Held out: words learned on parkour, tried on terrain.

**Of the one loop** (section 0; approved S165). Each is a gate that can fail, frozen before its first run.

8. **One mechanism.** Straight800 and `hardtrack` are answered by one entry point; the gate records which stages ran and refuses a stage only one of them can reach. Planted fault: a fast path for uniform ground must turn it red. It also watches learning: joints searched together.
9. **Compute mirrors terrain.** Thinking work units against (exceptions in the tube + events), over the 21 tracks, the straights and `hardtrack`. Killed if work grows with length on uniform ground, or does not grow with defects.
10. **The map holds his shortcut.** The reach relation over `hardtrack`'s faces, built with no knowledge of his run, contains his wall-top route (`jobs/hard-track/shortcut.md`), and the bot's T is at most his 428. His T is the bar; his inputs are never used, except replayed as a diagnostic once the sprint window is ported (S56; S166).
11. **Plateaus are few.** Per joint pattern on natural terrain, the plateau count does not grow as bisection is refined. Today's evidence is two patterns (GLM `trial-offset-monotone`). It runs first.
12. **Patterns bend** (S81). New joint patterns per thousand joints fall along a long walk.
13. **The thin bridge** (S84). A one-block bridge planted across a chasm 10³ blocks away is in the far map (scales.md 4.3).
14. **The slide signal.** Joints that ask for a slide gain at least a tick; a sample of those that do not gains nothing under search.
15. **Thinking never holds the body.** p99 of (first control tick − earliest dispatch opportunity) is 0 whenever the first word's bracket is closed at the click.
16. **The cone with a kit.** A sentence using a pearl, a boat or an elytra never beats the kit's lower bound (engine.md, "The contract").
17. **The million line** (S131). Thinking is gated at 10⁶ blocks offline on a generator-backed superflat, Peaceful (S136), and 10⁴ is flown live (R-far); the Normal-difficulty line, two words with the food crossing as an event, waits for the mob port (S166).

The overturn rate (scales.md 4.3) and the Fermi floors these gates are read against (representation.md, "Thinking floors") complete the list.

**Of learning's one queue** (section 2; approved S169).

18. **Learning never delays acting.** The acting path's p99 is the same with learning on and off.
19. **The queue's judgment.** Realized T saved against expected, by rank. Killed if rank does not predict.
20. **Play transfers.** The share of demand joints first learned in play is above zero and rises; otherwise play is fake demand (S87). It restates test 2: transfer is a hit on a pattern key or its family.
21. **Merging pays.** Stored bytes per distinct behaviour fall along a long walk.
22. **The asymmetry.** Planted shocks on the sandbox (a law fault, edits, pushes): the time to recover falls when a class of shock repeats. Killed if every repeat costs the same.
23. **One mechanism.** One queue, one search operation, one record; a planted second scheduler turns the gate red.
24. **Families.** The cost of the n-th member of a joint family falls toward a bisection (about 1,500 law ticks against about 10⁴ cold: a Fermi figure), then toward zero when its envelope is derived.

## 7. Sources, and how far each was read

Read in full, 12 November: Anthony, Tian, Barber 2017 (Expert Iteration; 9×9 Hex only, no convergence proof); Silver et al. 2017 (AlphaZero; exploration by root noise with a tuned constant per game); Ecoffet et al. 2021 (Go-Explore; its cell archive is a lossy merge we may not copy; its robustify phase is what turns inputs into a policy); Ellis et al. 2021 (DreamCoder; it trains on generated tasks); Wang et al. 2019 (POET). Taleb: his own highlights of the four books; Taleb and Douady 2013, abstract only. Isaacson's *Elon Musk*, his highlights. For section 2 (16 November): his highlights of Taleb, Waitzkin's *The Art of Learning*, Isaacson, Ferriss and the *Almanack of Naval Ravikant*, about 250 through six Readwise searches; the books themselves not opened.

For section 0 (hub labels, fast marching, reachable sets, persistence, max-plus methods, the maneuver automaton), each with how far it was read: `jobs/unified-plan/sources.md`. Frazzoli's paper is to be read in full before U1 freezes.

**Not reached, so from memory and unverified:** Sutton, Precup, Singh 1999 (keeping primitives beside options keeps optimality; re-deciding mid-option beats running to the end); Kahneman and Klein 2009; Frazzoli, Dahleh, Feron 2005 (chain when one exit set lies inside the next entry set); March 1991 (exploitation drives out exploration); Simon 1962; the contraction-hierarchy witness rule.
