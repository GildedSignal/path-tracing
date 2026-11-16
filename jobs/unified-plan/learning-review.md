# Learning, attuned to the unified plan: a review

**Status:** a proposal for Julien (PRINCIPLES 24). It edits no doc. Fable 5.2 sub-agent, 16 November, after the U0 fold ([fold.md](fold.md)).

**Asked for by** S166: "have it think on the learning part to see if we missed improvements … learning should be as natural as possible with all the things that i said like play and taleb coded, we should look at the incentives and if it is antifragile among many other talebian concepts, also let's think a bit like waitzkin, elon, ferriss and naval." And S165: "we might be so fast that a ladder climb grants us compute to learn so much that it can improve itself real time."

**Read for this:** `docs/learning.md` as folded; his turns S83–S98, S122, S148, S165, S166 and X16 again, in full. His own highlights, through six Readwise searches (Taleb's four books, *The Art of Learning*, Isaacson's *Elon Musk*, Ferriss, the *Almanack of Naval Ravikant*): about 250 highlights returned, all read; every quote below marked "(his highlight)" is one of them. **The books themselves were not opened;** a highlight is his selection from a primary source, not the source.

## 0. The review in seven sentences

1. **Learning has no cost in T while a certified word runs,** so its budget is not a slice we choose: it is every idle core for as long as the body is busy.
2. **That budget is about a hundred times the route's own demand** (section 1), so the surplus is the R&D and play budget, and the barbell appears without a percentage.
3. **Learning should be one queue, one operation and one record,** as acting is one loop. Today learning.md section 2 has nine steps and five sources of work (failure, the idle slice, two exams, play, R&D); that is how a second planner would grow.
4. **What is learned changed with the plan and the doc has not caught up:** the unit is now the joint pattern and its plateaus, and "does it transfer" is a hash hit, not a hope.
5. **Three things are missing outright:** surprises that helped, a census of demand from the known world, and learning by merging (removing distinctions).
6. **The system is antifragile to law errors and edits, and fragile in three places:** one law shared by every bot, success with no stressor, and the store itself.
7. **Most of what Waitzkin, Musk, Ferriss and Naval say is already in the mechanism; five ideas change it,** and four do not fit and are named as such.

## 1. The economics: what a ladder climb buys

Inputs, all stated: a tick is 50 ms; the machine has 8 cores, one pinned to acting (scales.md 4.7), so 7 learn; a new joint pattern costs about 12 ms of one core (representation.md, "Thinking floors"); the first word of `hardtrack` is a 260-tick ladder.

- **The ladder:** 260 ticks = 13 s × 7 cores = 91 core-seconds ≈ **7,500 new joint patterns**. The track has three narrow joints. The climb could learn the whole track many times over. S165 is right by two orders of magnitude.
- **Ordinary travel:** one sprint-jump cycle is 12 ticks = 0.6 s × 7 cores = 4.2 core-seconds ≈ **350 patterns per cycle**, against at most one new joint consumed per cycle.
- **So on-route demand uses about 1% of the free compute.** The other 99% has nothing it must do. That remainder is what R&D, play and regret sampling spend. Nobody sets its size.

This replaces two lines of learning.md: "a small slice of idle time" (section 2, step 2) and the budget rule "think as long as the expected T saved … exceeds the T the thinking costs" (step 3). While a certified word runs, thinking costs no T (section 1, "Four dimensions"). The only cost left is the other thing the core could have learned.

**How it is spent, ordered by expected T saved, never delaying acting.**
- **One queue** (the allocator's, ordered by slack; no new scheduler). An item is "narrow this bracket". Its value is (T it could save) × (the traffic that will meet it) ÷ (its cost in core time). Its deadline is the tick the body reaches the joint it feeds.
- **Deadline work first,** because it is the only work with a deadline: joints on the committed route that have no set; then the open brackets on that route, widest first (a better sentence may exist: a slide, a bigger word, a kit option such as S96's boat).
- **Then everything else by value alone:** alternates at joints whose width is near zero; joints near the route; patterns counted in the known world (section 3.2); play's own puzzles (section 4).
- **Improving itself in real time.** A better certified suffix replaces the current one only at a joint the body has not committed to, and only if its T is lower by at least a tick. A tie keeps the incumbent. That is the whole swap rule; it needs no threshold.
- **Never delaying acting.** Learning runs on the other cores at idle priority and publishes by pointer swap. The gate is section 7, test L1.

## 2. One queue, one operation, one record (Musk's algorithm, in his order)

His highlight: "Step one should be to question the requirements … Make them less wrong and dumb … And then delete, delete, delete." And: "Always wait until the end of designing a process—after you have questioned all the requirements and deleted unnecessary parts—before you introduce automation."

**1. Question each requirement of learning.md section 2.**

| Step today | Still needed? |
|---|---|
| 1. say what you know | Yes. It is acting (section 0). |
| 2. a failure names the bottleneck; plus an idle slice | The failure, yes: a missing set or an open bracket **is** the demand. The slice as a quantity, no: it emerges (section 1). |
| 3. search in the law | Yes, one joint pattern at a time. It is the one operation. |
| 4. keep the fact as a program | Yes, as the set's plateaus with a witness. |
| 5. the envelope is derived | Yes, and it should be promoted: see "smaller circles", section 5. |
| 6. two exams | Exam 1 (re-prove by footprint after a law change), yes. **Exam 2, no:** a new word can only lower a min, so it never breaks a set; "is there now a better sentence" is an open bracket like any other, already in the queue. |
| 7. a bound is a certificate | Yes. |
| 8. exposure by confirmation; "kind" undefined | Yes, and "kind" now has a definition: a generator row, or a joint family (one class signature). |
| 9. play | Yes, as a **source of queue items**, not a process (section 4). |

**2. Delete.** Nine steps become four, and five sources of work become one queue:
1. Act on what is known; a missing set or an open bracket is the demand.
2. One queue orders all learning work (section 1).
3. One operation: search one joint pattern exactly; keep its plateaus, its witness and its status.
4. The server confirms, through ordinary trips first and idle bodies second.

Failure-driven learning, regret sampling, R&D, play and exam 2 differ only in **who put the item in the queue**. That is the one-mechanism test for learning. His highlight gives the check on the deletion: "if you do not end up adding back at least 10% of them, then you didn't delete enough."

**3. Simplify** only what is left. **4. Accelerate** the slowest loop, which is not compute: the server confirms about 10⁵ times slower than the law proposes (learning.md section 2, step 8). Confirmation should ride on trips that happen anyway: every executed joint is a free confirmation of its pattern. **5. Automate last:** the learned ordering, LLM proposers and the GPU stay parked (learning.md section 3) until the four-step loop has run. His algorithm agrees with the parking.

**The idiot index of the one operation.** A cold pattern is searched at about 10³ verifications × 12 ticks ≈ 10⁴ law ticks. But a set is a few breakpoints, and a breakpoint is found by bisection: at most 64 halvings × 2 replays × 12 ticks ≈ 1,500 ticks. So once one member of a joint family is known, its neighbours can start their bisection from its breakpoints. The index is about 7 for every pattern after the first in a family. This is a Fermi claim to test, not a measurement.

## 3. What is missing

**3.1 Surprises that helped.** The deviation stream logs a mismatch as a law error to fix (S81: "a gold mine for fixing issues"). When the bot arrived **sooner** than predicted, the same record is also a candidate word: a push, a knockback, a slide the law did not expect. "even a zombie can be precomputed and can even help us" (X2). Nothing today keeps the favourable half. One extra field does: the sign of the T difference. A favourable surprise goes to the queue as "what produced this, and where else".

**3.2 A census of demand from the known world.** Learning waits for a bot to fail. But "we suppose the player is omniscient" (S6): the terrain is on disk. The joint patterns in a region can be counted by content hash before any bot goes there. A pattern's traffic is then (its count along likely corridors) × (trips), known in advance. It is still pulled by need, since goals and corridors weight it; it is not a sweep. This is Naval's leverage in his highlight, "products with no marginal cost of replication (code and media)": one learned plateau serves every bot at every place that pattern occurs, and the census says how many places that is.

**3.3 Learning by merging.** Every step today adds. None removes a distinction. Two pattern keys whose plateau tables are byte-equal under a kit are one class, and finding that is learning: "A flower is air" (S134) should be something the system discovers, by hashing tables (ES 32: symmetry is found), not only something the codec is told. It shrinks the store, raises the hit rate, and is the "delete" step as an operator. Test L4.

**3.4 Three statuses, not two.** learning.md section 5 says ""not found yet" and "the law lacks it" look the same from inside". They need not. A joint's record carries one of: **found** (plateaus, witness); **forbidden** (outside the derived envelope: a proof); **searched** (k work units, nothing yet: not a proof); **refused** (the law hit an unported mechanic: a demand on the port, with the refusal as its witness). His highlight from *The Black Swan* is the rule: there is "No Evidence of Disease" and no such thing as "Evidence of No Disease". `hardtrack`'s certificate confused the two (audit D6).

**3.5 Transfer, redefined.** Kill test 2 asks whether "a solution … transfer[s] to other jumps of the same shape". With pattern keys it is exact when the key matches, so the open question moves up one level: does a joint **family** share a function, so that new members cost a bisection (section 2) or nothing (section 5)? The measure is the cost of the n-th member of a family, which should fall toward zero.

## 4. Play

His words: play is "how childs get better and test themselves instead of doing nothing" (S97); it "could also create its own puzzles which is important and how humans learn and test" (S98); "we need to find a zone of proximal development" (S90).

- **Where the edge is.** The plan makes it exact: a joint whose set is empty, a plateau whose width goes to zero (X16: "contention points … at the edge of what i think i can do"), a bracket that stays open. Play's puzzles are those, moved one step toward the law's envelope until the set vanishes (learning.md section 2, step 9). No number of ours defines the zone.
- **Two kinds of play, two scarce resources.** Thinking play spends free compute, which is abundant (section 1). **Body play spends a body's ticks against the server, which is the scarce thing.** So idle bodies should do only what compute cannot: confirm unconfirmed patterns, in order of T × traffic, where a miss is safe.
- **It must transfer** (S98's condition, and S87's warning: "the issue of attention and opportunity cost and fake demand"). A play result earns nothing by being solved. It earns when demand later hits it. The measure: the share of demand joints whose pattern was first learned in play. Test L3.
- **Its incentive, and how it is gamed.** Rewarded by solved puzzles, play would make easy ones. Rewarded by novelty, it would make puzzles no trip meets. So its items are perturbations of patterns that trips **did** meet, and its share of the queue follows its past hits. That is skin in the game for a proposer: his highlight, "You will never fully convince someone that he is wrong; only reality can."
- **Hormesis.** His highlight: "Undercompensation from the absence of a stressor … degrades the best of the best." Where every sentence arrives, nothing fails and nothing is learned (learning.md section 4, "Success: silent decay"). Play is the stressor for trusted words: the most-used gaits get perturbed first.

## 5. Waitzkin

| His idea | Here | Change? |
|---|---|---|
| **Numbers to leave numbers.** His highlight: "mastery involves discovering the most resonant information and integrating it so deeply and fully it disappears" | search, then a plateau that is only read | Already the mechanism. Add its measure: the share of acting decisions answered with zero law ticks, per map. |
| **Making smaller circles.** His highlights: "plunge into the detailed mystery of the micro in order to understand what makes the macro tick"; "Depth beats breadth any day of the week" | "we should not learn jumps but classes of jumps and uncover the generative function" (S90) | **Yes.** For the family that carries the most T × traffic, stop accumulating patterns and **derive**: check whether its breakpoints are given by the closed-form envelope (learning.md section 2, step 5). When they are, the family is one generator row and its stored patterns are deleted. The circle got smaller and kept its power. |
| **The soft zone.** His highlight: "first we learn to flow with whatever comes. Then we learn to use whatever comes to our advantage. Finally … create our own earthquakes" | flow: re-anchor at events. Use: section 3.1. Earthquakes: play | **Yes, the middle stage** (3.1). The first and third exist. |
| **The second mistake.** His highlight: "The first mistake rarely proves disastrous, but the downward spiral of the second, third, and fourth error creates a devastating chain reaction" | learning.md section 4: "Edits break certificates, bots replan, bots edit: cascades. Open: no model" | It names an open hole. No mechanism proposed here; it needs its own thought with him. |
| **Incremental against entity theory** | a failure names a bottleneck (S86); a failed experiment kills the configuration, never the method (ES 9) | Already there. Section 3.4's "searched" status is its record: never "cannot". |
| **Chunking.** His highlight: "my trained mind is not necessarily working much faster … my conscious mind has less to deal with" | sites, portal tables, compound words; decisions counted at switches | Already there. |
| **Endgames first** | scales.md 4.8; the order U1 (kernels), U2 (joints), U3 (the loop) | Already there. |
| **Stress and recovery.** His highlight: "Players who are able to relax in brief moments of inactivity are almost always the ones who end up coming through" | the ladder climb | It is section 1. The body's quiet moments are when the mind works. |

## 6. Taleb

**The asymmetry test** (his highlight, tagged favourite): "anything that has more upside than downside from random events (or certain shocks) is antifragile; the reverse is fragile."

| Shock | After it, is the system better? |
|---|---|
| a deviation (the law was wrong) | **Yes.** Witness, law delta, regression test (S81). |
| an edit in a footprint | **Yes, weakly.** Local repair; the new terrain's joints are learned. |
| a wildcard's cone crossing a tube | Neutral today. It would gain if each forced re-solve were kept as an alternate (below). |
| a law update | **Fragile if stored facts are recordings; robust as programs** (learning.md section 2, step 4). It gains only if re-proving is cheaper each time: kill test 5. |
| the law wrong in a way all words share | **Fragile.** "one large failure, all bots at once" (learning.md section 4). |
| no shock at all | **Fragile.** His highlight: "Success brings an asymmetry: you now have a lot more to lose than to gain." |

**The concepts, each mapped or refused.**
- **Barbell.** His highlight: "the barbell [is] a domestication, not the elimination, of uncertainty." Safe side: only certified sentences ever act. Wild side: unlimited offline trials, each costing core-milliseconds, each able to save T for every bot for good. Section 1 shows the split needs no ratio.
- **Convexity and optionality.** His highlight: "an option is what makes you antifragile." Options the plan already holds: prices in sets (an item kept "for there", S96), intermediate words kept though nothing wants them (scales.md 4.1). **One to add: alternates.** The backward wave already computes the second-best plateau at a joint; keeping it costs bytes and turns a broken certificate into a lookup. His highlight: "Redundancy is ambiguous because it seems like a waste if nothing unusual happens. Except that something unusual happens—usually."
- **Do not judge the wild side by its hit rate.** His highlight: in trial and error "the sample track record will tend to underestimate the long-term average; it will hide the qualities, not the defects." R&D and play are judged by their best hits and by test L3, never cut for a low average.
- **Via negativa.** Section 3.3 (merging) and section 2 (deleting steps). Also: "A dominated word is dropped by proof" (scales.md 4.1) stays the only removal of knowledge; nothing is removed for disuse (S88).
- **Ruin and ergodicity.** A bot that dies, loses an item for good, or leaves damage it cannot restore has ended its own series, however good the average over bots. The docs already make ruin a barrier, not a price (engine.md). **Learning has its own ruin, the store:** one wrong plateau trusted by every bot. Every entry therefore carries its law hash and its witness, and a refuted entry is tagged, never silently replaced (learning.md section 1). Body play happens only where every joint has a certified way to a safe state, and never with edits before the ledger has flown. His highlight: "the first step toward antifragility consists in first decreasing downside."
- **Small failures.** His highlight: "For the fragile, the cumulative effect of small shocks is smaller than the single effect of an equivalent single large shock." Failures by the joint, never by the trip (learning.md section 1). Already there.
- **Skin in the game.** Proposers pay by losing queue share (section 4). The lab's independent checker is the same rule for lenses. His highlight: "Things designed by people without skin in the game tend to grow in complication" is the argument for one queue and for "no new search code".
- **Iatrogenics** (his highlight, from Farnam Street: "any effect resulting from intervention in excess of gain"). Three interventions that could cost more than they give: re-proving everything after a law delta (kill test 5; re-prove by footprint only); sliding a joint whose bracket is closed (the slide signal forbids it); swapping a sentence mid-flight for a tie (section 1's swap rule forbids it).
- **Narrative fallacy.** His highlight: "favor experimentation over storytelling, experience over history, and clinical knowledge over theories." A word's record holds a program and a witness, never a name or a reason (no catalogue, ES 4). For us: a claim of hardness or impossibility is a machine certificate or it is a lead. `hardtrack`'s "only combination that completes" was a story.
- **Lindy.** His highlight: "its life expectancy lengthens with time, conditional on survival." **A modest fit.** A pattern that has survived many confirmations and several law versions has earned its place in the hot set; a young one gets less exposure, which the doc already says. I would not build more on it.
- **The repeated text.** His highlight: "the reading of a single text twice is more profitable than reading two different things once." Re-solving a map he already picked, after the vocabulary changed, teaches more than a new easy map. It is the regret item on a known map, and it agrees with "depth beats breadth".

## 7. Ferriss and Naval

**Ferriss.**
- **Deconstruct:** the minimal learnable unit is the joint pattern; above it the family; above it the site. Already the plan.
- **Select (80/20):** "words … will be zipf's law coded" (S81). A few patterns carry most ticks; the queue's T × traffic is the selection. His highlight: "material beats method": what is learned matters more than how. So the queue's order is the main lever, and test L2 audits it.
- **Minimum effective dose.** His highlight: "Boiled is boiled. Higher temperatures will not make it "more boiled"." **A stop rule the doc lacks:** a joint whose bracket is closed (UB = LB) gets no more compute, ever, until its certificate breaks.
- **Sequence:** his highlight from *Ultralearning*: "start with a skill that they don't have all the prerequisites for … go back a step". It is S88: "since they are hard they add real needs to get better." Already the plan: a hard map names its own prerequisites.
- **Stakes:** his time is the bar; the server confirms.
- **Does not fit:** "aim for a roughly 85 percent success rate" (his highlight, Scott Young). It is a number of ours. The zone is defined by the two envelopes with no number (learning.md section 1).

**Naval.**
- **Leverage:** section 3.2.
- **Judgment.** His highlight: "Leverage is a force multiplier for your judgment." With this much leverage, a wrong ordering wastes all of it. So the queue logs, for each item it ran, the T it expected to save and the T it did save. Test L2.
- **Rederive.** His highlight, tagged favourite: "If you can't rederive concepts from the basics as you need them, you're lost. You're just memorizing." The store is a cache of what the law implies, never knowledge of its own. It is "store the generator" (S6), and the reason a law change is survivable.
- **Compounding:** stable subassemblies (scales.md 4.1); the measure is kill test 4, cost per new puzzle falling.
- **Specific knowledge** ("will feel like play to you but will look like work to others"): the patterns of his own server's builds. A light fit: it is what play on his world accumulates.

## 8. Incentives: what each loop rewards, and where it runs away

| Loop | Rewards | Gamed or runs away as | Guard |
|---|---|---|---|
| the queue (T × traffic ÷ cost) | common, cheap joints | rare blockers starve (S88: "something that is important but appears rarely matters") | T saved for a joint with no sentence is the whole detour, so a rare blocker ranks high; nothing is deleted for disuse |
| the hit rate, Heaps' bend | few new patterns | coarse keys that merge unequal things | byte equality before any merge (slice B); merged tables must be equal, not close |
| "decisions are switches" | few switches | decisions hidden inside big compound words | evaluations are counted beside decisions |
| thinking time on tracks | fast answers on known tracks | tables warmed by the test tracks | held-out maps; no track-keyed entry (kill test 7) |
| his time as the bar | matching him | copying his route | only his T is used, never his path (S56) |
| play | solved puzzles | easy or unmet puzzles | section 4 |
| free compute while a word runs | — | a slow first word chosen because it buys thinking time | the first word is chosen by its bracket alone; T is the only objective |
| regret sampling | improvements to good words | endless polishing | the stop rule (minimum effective dose) |
| a lower bound | pruning | a wrong bound silently prunes the best forever | re-derived per law version (learning.md section 2, step 7) |
| us, the builders | gates passed | a second mechanism that passes its gate | kill test 8; "the goal, never a proxy" (ES 27) |

## 9. Kill tests for these proposals

- **L1. Learning never delays acting.** The acting path's p99 is the same with learning on and off.
- **L2. The queue's judgment.** Realized T saved against expected, by rank. Killed if rank does not predict.
- **L3. Play transfers.** The share of demand joints first learned in play is above zero and rises. Otherwise play is fake demand.
- **L4. Merging pays.** Stored bytes per distinct behaviour fall along a long walk.
- **L5. The asymmetry.** Planted shocks on the sandbox (a law fault, edits, pushes): after each class of shock, the time to recover falls when the class repeats. Killed if every repeat costs the same.
- **L6. One mechanism.** One queue, one search operation, one record. A planted second scheduler must turn the gate red.
- **L7. Families.** The cost of the n-th member of a joint family falls toward a bisection, then toward zero when its envelope is derived.

## 10. What I am not sure of

- **The swap rule mid-flight** is simple, but a swap changes other bots' reservations on tubes; with several bots it may need more than "strictly lower T".
- **The census** (3.2) could become a sweep if corridors are drawn too wide. It is bounded by real goals, and I have not shown that bound.
- **Cascades** (the second mistake) have no model here either.
- **The Fermi of section 2** (a family's later members at about one seventh the cost) rests on plateaus being few and breakpoints moving smoothly within a family. Kill test 11 of learning.md and L7 are its tests.
- **I read highlights, not books.** Where a concept of Taleb's was not among his highlights (ergodicity by name), I used the idea as the docs already state it and did not quote.
