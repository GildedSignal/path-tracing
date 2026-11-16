# The unified plan: one loop from the million-block line to the hardest parkour

**Status:** a proposal for Julien to rule on (PRINCIPLES 24). Designer: Fable 5.2 sub-agent, fresh context, 16 November. Nothing here is built, and nothing is to be built before he says so. Companion files: [audit.md](audit.md) (what the current plan missed, removed or wired badly), [sources.md](sources.md) (what was read and how far), [questions.md](questions.md) (his decisions).

**Approved** by Julien on 16 November (S165), its recommendations confirmed (S166), and folded into the living docs ([fold.md](fold.md)); the docs now hold the truth and this file is the argument behind them. **On the word "hard"** (S166): the track `hardtrack` "is not even hard btw it's easy-medium"; "hard tracks are tracks that have many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that." Below, "the hard track" means `hardtrack`, an easy-medium track; the hard extreme is his definition.

**Asked for by** S164: "make it unified and coherent … the goal is to reduce T thinking time while solving the most complex puzzle possible by folding the space such that it becomes trivial, we might not even need A* like algos." And S161: "we can't have two paths like that "easy" or "hard" it should be the same mechanism".

## 0. The plan in six sentences

1. **The world under a kit is a crystal with defects.** Where the ground gives the same answers to the kit's verbs, the law has a translation symmetry, a held word settles onto its gait, and travel is a count. A defect is a place where that symmetry breaks: an edge, a wall, a class change, a tool's reach, another player's cone.
2. **Thinking happens only at defects.** Between two defects one word repeats and nothing is chosen (learning.md:17). So thinking is proportional to the defects the route can touch, never to its length: "the terrains complexity should be a mirror of compute's complexity" (S131).
3. **One operation serves every scale: extend the biggest word until its certificate breaks.** "you start with the biggest one i think it breaks and then you go smallest and smallest" (S91). On a plain it breaks at B after 233,000 cycles. On a parkour it breaks at the first lip.
4. **One rule allocates all compute: narrow the bracket that could change the decision due soonest** (scales.md:7, approved S65, never wired to the planner). A bracket that is already closed opens nothing below it: "stop thinking about the lower level ones" (S131).
5. **The map is a T-to-go on joints, built backward, shared, and kept.** The bot reads it and replays nothing it has already certified. Search over exact states lives in learning, once per joint pattern (learning.md:15).
6. **A kit changes the crystal, not the loop.** A pickaxe turns a wall from a defect into a slow medium. A pearl adds a term to the light cone. A portal is an edge that is always there at a toll (S132).

## 1. The space

### 1.1 What is being folded

The raw problem is a time-optimal control problem on the law's state (65 fields, state.rs) × the inventory × the world's edits × the tick. Bellman named its cost the curse of dimensionality. Four folds remove the dimensions, and each is already in his words:

| Fold | His words | What it removes | Evidence |
|---|---|---|---|
| **Symmetry** (translation over uniform ground) | "under the right representation many symmetries can appear" (ES 32); "i get the modulus of my best tool" (S120) | position: a gait is (word, phase, count) | the state minus position recurs bit for bit; block crossings follow by exact floor arithmetic (wiki `gait-modulo`, 0 disagreements over 7,090,461 ticks) |
| **Dissipation** (every regime has drag) | "regimes should fall out of hand" (S40); "situations recur" (S46) | velocity, beyond momentum's memory | sprint-jump on stone: bit-exact recurrence after 263 ticks, period 12 (P1 `last.md:7`) |
| **Events** (the law is one small rule between contacts) | "you just compute the parabola and voilà" (S44); "ticks are just an execution layer" (S118) | ticks | between events the tick is one affine rule (wiki `generative-function`; event-forms report §1A) |
| **Kit classes** (a block is its answers to the kit's verbs) | "A flower is air." "unless i need to place a block and it bothers me" (S134) | the block alphabet (32,366 states) | buttons and plates have air's collision shape (HANDOFF:45); slice B's codec |

**The 263 ticks are derived, not found.** One sprint-jump cycle has 1 ground tick (drag 0.6 × 0.91) and 11 air ticks (drag 0.91), so a velocity difference shrinks by 0.546 × 0.91¹¹ ≈ 0.19 per cycle, about 2.4 bits. A double has 53 bits: 53 ÷ 2.4 ≈ 22 cycles ≈ 268 ticks. Measured: 263. So **momentum's memory length is (bits that matter) ÷ (bits forgotten per cycle)**, a number the law gives per ground. Where joints are closer than that, order matters and the state must be carried. Where they are farther, a place has a cost per gait (learning.md:14). No constant of ours is in it.

**"maybe we control in derivative directly?" (S131).** Yes: in velocity space the law's rule is `v ← d·v + a·u(yaw)`, a contraction toward a fixed point, and a gait is that fixed point. Position is only the running sum. So words are planned where the dynamics is trivial (ES representation.md:9: "every quantity is kept in the form in which its change is an addition"), and position is read out by exact integer maps per binade (spec.md:50-52, premise P2).

### 1.2 The objects (what a node, an edge and a set are)

- **Word.** Unchanged (representation.md §3): a feedback program with a signature. One addition: every word takes a **count**. A gait is a word with a large count. A jump is a word with count 1. There is no other difference between them.
- **Zone.** The extent over which one word's certificate holds: its worldtube, checked once (S117). A zone is **found by need**, by the corridor query from where the bot is toward where it goes (wiki `gait-corridor`: validity is one query, the first cycle whose shifted footprint reads a differing cell). It is never a partition drawn in advance: "they should arise based of needs, some regions are more compressible than others" (S132); cells "should be emerging … like how doors break it" (X9).
- **Joint** (the node). A place where one word may end and another begin: an event of the law (vocabulary.md §4) at a face. Lips, landings, class boundaries, the mouth of a wormhole, the cell an edit would change.
- **Set** (what a joint holds). The arriving states from which a known sentence reaches the target, with its T. It is stored as the law's own comparisons: the yes-or-no class, plus **plateaus** of the one coordinate a held word leaves free (its offset along its line). A plateau is an interval between two single-ulp breakpoints, found by float bisection and two replays. Measured: 5 plateaus on a stone-to-ice edge, and 12 → 14 → 12 ticks over a one-block gap, every boundary a clean single-ulp flip (glm-digest:21). So a set is a short sorted array of bit patterns with a T and a next word on each. It also carries its **premises** (cells at ticks, items in hand, S95) and the **prices** of what is in hand (S96).
- **Edge.** A word between two joints: generator and arguments, T as a step function of the entry plateau, worldtube, width (delay tolerance and clearance), `uses`, `needs`, `writes`, obligations (vocabulary.md §8).
- **Site.** Joints closer together than momentum's memory. Inside a site, order matters and sets carry state. A site hands upward one **portal table**: entry class → exit class, T. Sites with the same content share one table by content hash (Hashlife, representation.md:78). This is where "more compressible" becomes a mechanism: a plain has no sites at all.
- **Label.** A joint's T to and from a few hubs, kept goal-free. Two labels answer a far query by one merged sweep with no graph search (Abraham et al., §4: the query is "a single sweep through the labels, similar to mergesort").
- **Field.** T-to-go on joints for one goal: the min-plus closure of the above. Every bot going there shares it (S19).

**What died:** the standable cell as a node with its own T (S91, X9, learning.md:16), and the per-cell field that defeated fences (learning.md:15). The cell stays as nature's address for reads.

### 1.3 The kit folded in

- **Momentum** lives in sets, inside sites only. A long ice lane is a gait whose exit speed is a function of its count: one row of the generator store.
- **Kits are classes** (S134). One fine world, one projection table per kit (spec.md:29). A kit change swaps a pointer (scales.md:15).
- **Edits are tolls with a debt** (S32, S13). A wall under a silk-touch pickaxe is not a defect, it is a slow medium: "a wall is a two second delay not a wall" (X4), plus the T to put the identical block back (S12). Edits whose undo closes inside the gait's period (place ahead, break behind, within reach) are **gaits with their own speed**: speed bridging and tunnelling are rows of the same generator store, with an item balance of zero per cycle. Other edits are ledger debts (scales.md 4.9) whose undo must be reachable before arrival; the debt's T is a premise the backward pass carries. Least-time routes through slow media bend at the interface (Fermat; the ant trails of leads.md:41; Mitchell and Papadimitriou's weighted regions). A flower is a defect only for the place verb.
- **Wormholes.** A pearl is an edge whose length is not bounded by the legs' c: a 1.5-blocks-per-tick throw plus the thrower's momentum, 5 damage, a 20-tick cooldown, and a landing that is a cone, because the spread is drawn from the pearl's own random source (glm-digest:33). So its certificate has a width, and its landing set must contain the cone. A **Nether portal is an edge that exists once built**: 80 ticks of wait, a 10-tick cooldown, an 8:1 chart on x and z, an exit that snaps to the nearest portal within 16 or 128 blocks, and a restoration debt on both sheets (glm-digest:31-32; redteam-opus.md:111). "as if it is always there but you need to do a weird path to access it" (S132): the weird path is the compound word that builds it, and its price is that word's T plus its debt.
- **The light cone must know the kit.** Today's bound divides the gap by the terrain's c with the premise "no item" (`goal.rs:4-7,29-32`). With a pearl, a boat or an elytra in the kit that bound is false, and a false lower bound "prunes the true best forever" (learning.md:34). The bound becomes the least over the kit's terms: legs at their c; each wormhole as (to its mouth) + toll + (from its exit); each carried gait at its own c. It is a min of a few simple terms, which is what a tropical polynomial is.
- **Elytra** is a regime with drag, so it has gaits: a glide at fixed pitch settles to one speed and one sink rate (representation.md:48). Pitch is a continuous argument, so the word family is a table over (height to lose, speed), compiled in learning; R-brach stays its falsifier (S17). The law is not ported (movement.rs:130-132).
- **Boats** are a carried gait whose zone is water or ice. "it's just on where i park my bike that it'll matters" (X16): the decisions are the mount and the dismount joints.
- **Other players** are wildcards. Each has a cone. A certificate is checked against a cone once, at the earliest tick the cone can reach its worldtube, on the timing wheel (spec.md:84). Only the segments a cone enters are rethought (S121).

## 2. The loop, the same at every scale

```
solve(from, to, kit, deadline):
  1. bracket.  LB from the kit's light cone.  UB from what is known:
               a zone's kernel, two labels merged, or nothing yet.
  2. closed?   If no narrowing can change the next word, emit that word and stop.
  3. extend.   Take the biggest word the kit holds for this displacement.
               One presence query over its worldtube returns its count
               and the first exception that breaks it.
  4. cut.      The exception's joints split the problem. Each part is
               solve() again, given the sets on its two ends.
               Parts run in parallel.
  5. compose.  Sets come backward in min-plus. The point goes forward.
  6. allocate. Among open brackets, work on the one whose narrowing
               could change the decision due soonest.
```

- **Step 1 is the hardness signal** (S121: "a cheap way to tell hey this seems hard"). It costs one lookup. A closed bracket means "go". A wide one says where the gap is and what kind: unknown terrain, an unlearned joint, a cone.
- **Step 3 is "presence first"** (S120: "i don't care where they are just if they are present first"). Role planes answer "any exception in this tube" 64 cells per operation (spec.md:37); a homogeneous source answers in one read (spec.md:39).
- **Step 4 is "segments in parallel"** (S120, S131). The parts are independent given the **sets** on their ends, not single states, so no part waits for another.
- **Step 5 is learning.md §1** unchanged: forward owns history, backward carries premises and prices.
- **Step 6 is the allocator** (ES 12; scales.md:7). It is also "if we're smart … conditions to fill to use more compute or different type of compute" (S131): the kind of gap chooses the kind of work.
- **The wiggle** (S94, S131: "loose in my predictions so that i can explore more paths and then wiggle the most promising ones"). Candidates are compared by their brackets first, which is loose and cheap. Only the best is tightened. The local signal he asked for (S120: "an extremely simple metric that's local perhaps that tells hey i'd like to get jiggled") is the joint's own table: if a neighbouring plateau has a smaller T-to-go and the upstream set can reach it, the joint wants a slide; otherwise it wants none. It is one comparison per joint and it replays nothing.
- **Moving at once** (S120). The first word is emitted when step 2 closes for it, not when the route is whole. Everything after is thought while the body moves (learning.md:21), ending each committed prefix at a joint with a certified way on.
- **Changing your mind** (S121). A new goal is `solve` from the observed state. Labels, tables and the generator store are resident, so the cost does not depend on what was consumed (spec.md:90).

**Learning** is the same loop reaching a joint that has no set. It opens an exact search over takeoff states for that joint pattern only (learning.md:30), keeps the plateaus under the pattern's content key, and never searches that pattern again. This is the one place exact-state search survives.

## 3. Every scale, and what each hands the others

| Scale | Lives there | Hands up | Hands down |
|---|---|---|---|
| Tick | the client, the only per-tick runner (spec.md:77) | deviations at events; packets | nothing |
| Joint (about 10 ticks) | words, arcs, plateaus | exact T per plateau, width, the slide signal | which plateau is wanted |
| Site (10² ticks) | sets, where order matters | a portal table by entry class; its content hash | which portals the corridor uses; the deadline |
| Zone (10² to 10⁶ ticks) | one word and a count | a closed bracket (LB = UB), its event list | where the big word must end |
| Trip (10³ to 10⁵) | labels, landmarks, the corridor | the bracket gap and its cause | the corridor, the deadlines |
| World | the kit's cone with its wormhole terms; prices | lower bounds; what an item is worth | what the kit could hold (S96: "keep it for there") |
| Vocabulary's lifetime | generators, joint patterns, laws | new words; surprises | what to learn next (T × traffic) |

- **One message both ways:** a certificate with a bracket, a footprint and a deadline (scales.md:7). Nothing else crosses a scale.
- **What far away means** (S64). A far thing is kept if removing it would move a bracket by more than the slack of the decision it feeds, and dropped otherwise. That measures a feature by **its effect on T, not its size in blocks**, so the thin bridge (S84) is kept at any distance and a forest of flowers is not. The mathematics is persistence (Edelsbrunner, Letscher, Zomorodian: features ranked by "life-time or persistence within the filtration"). It sharpens scales.md:66 ("topology exact, geometry coarse"), which had no criterion.
- **Multipoles, in the form min-plus allows** (S6, tracked). A cluster of joints seen from far is one source: its least offset plus the distance to its centre, with an error of at most the cluster's own radius in ticks. Near, it is opened. That is Barnes and Hut's opening angle on brackets (scales.md:65) and it is the portal table of §1.2 seen from outside.
- **Cutting points** (S131) are joints. **Zones from need** (S132) are certificate extents. **Parallel segments** are step 4. **Water** (S131, S132): the bot flows down T-to-go; it branches where two words tie or differ ("water can branch on saddle points"). In whole ticks most ties are fat and worthless, so they are broken by a fixed order at no cost (redteam-opus.md:75-81); a real saddle is a joint where the choice moves T by a tick or more.
- **How many decisions a terrain holds.** A route class is a way round the obstacles. Open ground has one. Each obstacle the kit cannot cross adds one. So the count of real decisions is bounded by the count of obstacles inside the route's ellipse, which step 3 returns for free. A pickaxe removes obstacles from that count.

## 4. As the bot

**A million blocks, bare legs, plain, Peaceful.** A click. My state arrives as bits (slice A). I ask for the bracket: I have only legs, so my biggest word is sprint-jump. I extend it toward B. The source is the generator plus its scars, and there are no scars in the tube, so the presence query is one read and the word does not break before B. Its kernel answers in closed form: my transient from the store (263 ticks, a memo hit), then cycles of 4.276060366589263 blocks per 12 ticks, crossing about 20 binades whose ticks are known by integer division, with two neighbouring yaws whose counts land me in B's box (spec.md:55-57). LB equals UB. I emit: word, yaw, count, yaw, count, stop. I thought for microseconds and I am moving. For the next 2.8 million ticks nothing is thought; chunk arrivals renew my lease and the wheel sleeps.

**`hardtrack`, easy-medium by his ruling (S166)** (his 428 ticks against a 579-tick witness and a planner with no sentence, report.md:50). A click. My biggest word, sprint-jump toward B, breaks at once: B is 35 blocks up and the tube is full of exceptions. That count is my hardness signal. The exceptions' faces are my joints: about a hundred top faces, the ladders, the lips. Extending from A, every word but one breaks within a few blocks, and the bracket says all routes begin on the first ladder (report.md:21). So I climb. That word takes 260 ticks (shortcut.md:6), and I think during it. Sets come backward from B: the final ladder's catch set; behind it, every face from which some word lands in that set. One of them is the top of the alignment wall. Behind that, the end of the southern ice lane. Nobody told me the southern lane was a decoy, and nobody told me it was not: the reach relation between faces holds his shortcut because it holds **every** face, which is exactly what the track's own certificate lacked (shortcut.md:14-16). Three joints are narrow, with zero ticks of slack early (report.md:22-24). If their patterns are new, learning searches each one's takeoff states exactly and keeps the plateaus. By the top of the ladder the point goes forward through the sets, sliding only at the joints that ask.

**Ten thousand varied blocks.** The far map gives a corridor from labels and the kit's cone, about 10³ cells (scales.md:65). Step 2 closes for the first word, and I move. Ahead of my body the loop certifies joint after joint, 10⁵ times faster than I consume them. A river: with logs in hand, the bound for a boat gait undercuts my sentence, and that gap asks for a craft-and-mount compound (S96). A player walks near my corridor: the wheel wakes one check at the first tick their cone could touch my tube, and only that segment is rethought.

All three ran steps 1 to 6. Nothing in the loop knew which one it was in.

## 5. Do we need A*?

Not in acting, and here is exactly what remains.
- **No best-first search over exact states.** That is what found 0 of 18 tracks alone (learning.md:15), and it is what the 3.2d-3 spec kept as "existing F" for "exceptional spans" (spec.md:115).
- **Inside a zone:** a closed form. No graph exists.
- **Between known joints:** two labels merged. No graph is walked.
- **At first sight of a site:** one backward wave over its joints. This is a shortest-path pass, and I do not hide it. It differs from A\* in three ways: it runs on the folded graph, whose size is the defect count; it runs once per goal and is shared and kept; and it never revisits a node, because T only grows along a route ("thanks to T the mingled parkour 4D path just is not", S89). Sethian's fast marching rests on the same causality.
- **In learning:** exact search over one joint's takeoff states, once per pattern.

This is "we need to leverage proper path finding … we're not remaking the wheel" (S92) and "a representation where you don't need a policy because the map is clear on where to go" (X16) held together: the wheel turns in learning and on the folded graph, and acting reads.

## 6. The machine

The machine is a Ryzen 7 7800X3D (8 cores, 96 MB of L3, AVX-512), 31 GB, a Radeon RX 7900 XTX (engine.md:114). A tick is 50 ms.

- **Latency before throughput.** T counts from the click's server tick (S124). Any thinking that fits before the next dispatch opportunity costs zero ticks, so the acting path is sized in microseconds and guarded at p99 (ES 12): no allocation, no lock, no boxed key, no hash of a hash. One pinned thread acts. Results are published by a pointer swap.
- **The L3 is the resource.** A DRAM miss is about 80 ns and an L3 hit about 10 (Fermi inputs; redteam-opus.md:32 used 5 to 80). The hot set must fit 96 MB: the generator store, the active corridor's sets and labels, the presence planes. At 8 bytes an entry that is 12 million entries. Abraham et al. report a continental query at about "five random accesses to main memory" (TR, experiments); the same budget here is about 0.4 µs.
- **Data shaped like its access.** A set is a sorted array of breakpoints: found by a 16-lane compare and a popcount, with no branch to mispredict. A label is two parallel arrays swept once. A presence plane is a bitboard: a section is 4,096 bits, eight 512-bit words. A bot's hot state is a handle and a phase (scales.md:106).
- **Integer min-plus.** Times are whole ticks, so `min` and `+` are exact and associative and run 16 lanes wide (scales.md:108).
- **Threads.** Step 4's parts go to the other cores. Learning runs on the idle cores, by slack. Today `engine/` has no thread at all (HANDOFF:27; design.md:44).
- **The GPU, where it pays.** A dispatch and a read-back cost about 100 µs (a Fermi input, to be measured). An acting query costs microseconds, so the GPU can never sit on the acting path. It pays in learning, where a batch is millions of independent items: compiling joint patterns, and waves over large sites. This is where "A* or something … on gpu in parallel" (S131) lands. It stays unqualified until a measured crossover (ES fermi.md:24).
- **Idle time computes ahead** (ES 17): labels for known terrain, joint patterns by T × traffic, the corridor beyond the committed prefix.

## 7. Fermi floors

Inputs, all stated: the law's tick 0.6–0.95 µs (engine.md:11); an arc certificate about 0.7 µs (HANDOFF:45); a DRAM miss 80 ns; a new joint pattern about 10³ verifications × 12 ticks ≈ 12 ms of one core (scales.md:37 gives 10 ms for 10³ × 100 ticks at 10⁷ ticks per second); the gait 4.276 blocks per 12 ticks (P1). These are floors, to compare designs, never promises (S46).

| Case | Inputs | Thinking floor | T it serves | Today |
|---|---|---|---|---|
| **1M line**, Peaceful | 1 label read; 1 memo application; k ≈ 20 binade events × about 10 integer divisions; a two-vector lattice step count ≈ log₂(10⁶ ÷ 4.28) ≈ 18 | **about 5 µs**, 10⁻⁴ tick | 10⁶ ÷ 4.276 × 12 = 2.81×10⁶ ticks (39 h) | straight800: 26.8–29 ms cold (S131; design.md:20), 23 s before that |
| **His 21 tracks** | J = 2 to about 30 joints; per joint one plateau lookup, one arc certificate, one presence query ≈ 2 µs | **10–100 µs warm** (agrees with HANDOFF:45); cold adds 12 ms per new pattern, about 40 ms on 8 cores for 30 | 44 to 2,315 ticks | 0.69–30.5 s (HANDOFF:45) |
| **`hardtrack`** (easy-medium, S166) | about 10² faces (to count) × 10–30 reach neighbours × 100 ns; a wave over faces × arriving-speed classes (about 4 each, an input to count: about 400 nodes); one 428-tick replay | **0.5–1 ms warm**, 0.02 tick; cold 0.05–0.5 core-seconds per narrow joint, all inside the 260-tick ladder | ≤ 428 ticks | no sentence (report.md:50) |
| **10⁴ varied** | J ≈ 2–5×10³ (class boundaries every 2–5 blocks, redteam-opus.md:36); far map 10³ cells × 80 ns | **first word about 100 µs**; 4–10 ms in total, spread over the trip; planning outruns the body by about 3×10⁵ | about 2.8×10⁴ ticks (23 min) | not run |
| **10⁴ bots** | 1.7 joints a second each on varied ground × 2 µs; 24 bytes hot each | **0.03 core**; 240 KB hot, inside one core's L2 | — | — |

On the million line, the Normal-difficulty answer is two words: food reaches 6 after 330 cycles, about 1,407 blocks (glm-digest:11), and the rest is walked. The food crossing is one more event in the same list, server-driven.

## 8. The one-mechanism test, run on this design

| Suspect | Is it a second path? |
|---|---|
| A zone's closed form | No. It is the same `extend` with a large count. The kernel is what every word has. |
| Labels against the wave | No. A label is a kept wave. Both are min-plus over joints. |
| Learning's exact search | A second **process**, by his ruling (S85), opened by a missing set, not by a kind of terrain. |
| Kits, wormholes, edits | Data: projection rows, generator rows, terms of the cone. |
| The GPU | An evaluation order of learning's batches. Same bits (ES representation.md:70). |

One real risk remains: learning's search could grow back into a planner if a site's joints are searched together instead of one pattern at a time. Kill test 1 watches for it.

## 9. Kill tests

1. **One mechanism.** Straight800 and the hard track are answered by one entry point. The gate records which stages ran and refuses a stage that only one of them can reach. Planted fault: a fast path for uniform ground must turn the gate red.
2. **Compute mirrors terrain.** Thinking work units against (exceptions in the tube + events), over the 21 tracks, the straights and the hard track. Killed if work grows with length on uniform ground, or does not grow with defects.
3. **The map holds his shortcut.** The reach relation over the hard track's faces, built with no knowledge of his run, contains the wall-top route, and the bot's T is at most 428. Killed if a route a human found is not in the map.
4. **Plateaus are few.** Per joint pattern on natural terrain, the plateau count does not grow as bisection is refined. Killed if it does: the set would not be a finite object. Today's evidence is two patterns (glm-digest:21).
5. **Patterns bend** (S81). New joint patterns per thousand joints fall along a long walk. Killed if they stay level.
6. **The overturn rate** (scales.md:73): a fine grain overturns a closed coarse bracket.
7. **The thin bridge.** A one-block bridge planted across a chasm 10³ blocks away is in the far map. Killed if the coarsening drops it.
8. **The slide signal.** Joints that ask for a slide gain at least a tick; a sample of those that do not gains nothing under search. Killed if the signal does not predict.
9. **Thinking never holds the body.** p99 of (first control tick − earliest dispatch opportunity) is 0 whenever the first word's bracket is closed at the click.
10. **The cone with a kit.** A pearl sentence never beats the kit's lower bound. Killed if it does: the bound is inadmissible.

## 10. The order of work

**What stands from 3.2d-3.**
- **Slice A** (resident channel, observed bits, the hop clock) and **slice B** (one codec, the label that keys the store, role planes, the proof spine). They carry no terrain kind.
- **P1** (the transient memo: PASS WITH FINDINGS, then resolved). **P2** (binade maps), **P3** (two yaws), **P4** (the law's own food guard). They test the kernel that **every** zone uses, so they are renamed from "plain prerequisites" to "the kernel".
- **Slices C and D** (the store and its event maps), **F** (the client runner and the shared comparator), **G** (the wheel, retarget, rolling horizon).

**What is replaced.**
- **Slice E**, the "plain adapter", and **slice H**, "existing F per exceptional span". They are the fork (postmortem-2026-11-16.md:15).
- **Slice K**'s gates, which test a plain case and a current-tracks case separately; and the "LATER, OWN DESIGN DOOR" for sets, preimages and the backward solver (spec.md:135), which sent the approved mechanism (learning.md:15) to later.
- Spec lines resting on refuted premises: U2's food super-period (spec.md:139; glm-digest:14) and a monotone portal map (glm-digest:24-25).

**The new order.** Each step ends in a flight he can use, and each is a ladder rung that can fail.

| # | Step | Consumer and gate |
|---|---|---|
| U0 | He rules; the integrator folds this into learning.md, representation.md, scales.md and vocabulary v3 the same day (postmortem "What changes") | questions.md |
| U1 | The kernel: slices C and D as specced, with `count` an argument of every word | the million-line thinking gate, through `solve` |
| U2 | Joints and sets: the set record (plateaus by bisection, premises, prices), the reach relation between faces from mirrored reach tables (`backward-tools`: 84.5% of 5,587 goals in one word), one worldtube check per arc | kill tests 3 and 4; fences' stalled joint; red-team the set record first, it is a one-way door |
| U3 | The loop: `solve` (bracket, extend, cut, compose, allocate), the slide signal, the kit's cone | kill tests 1, 2, 8; his 21 tracks and the hard track on one binary; flight: he races it |
| U4 | The runner and the wheel: slices F and G, rolling horizon | kill test 9; retarget at tick 1000 |
| U5 | The kit's verbs: place, break, interact in `Input` and the law (port-backlog P2a–c), edits as tolls with debts, the ledger | his five blocked tracks; R-lego |
| U6 | The far map: portal tables by content hash, labels, landmarks, coarsening by persistence | kill tests 5, 6, 7; R-far at 10³ then 10⁴ |
| U7 | Wildcards: cones on the wheel | a player crossing the corridor; only one segment rethought |
| U8 | Carried gaits and wormholes: boat, elytra, pearl, portal (port-backlog P4a–c) | kill test 10; R-brach; his list (ladder.md:107) |
| U9 | Threads for step 4, then the GPU for learning, each after a measured crossover | cost per bot at N bots |

U1 and U2 are independent and can run together. U5 and U8 are pulled by the port, which GLM lanes already feed.

## 11. What I am not sure of

- **Plateaus on natural terrain.** Their finiteness is measured on two patterns. Kill test 4 is the first thing to run.
- **The switch count.** The classical result that a time-optimal linear system with real eigenvalues switches at most (dimension − 1) times is recalled, not read; the law is discrete and its yaw is coupled across axes. Treated as a prediction for idle search to check, not as a premise.
- **Label size on terrain.** Road networks have small labels because few roads carry all long trips. Open ground does not; I rely on zones answering in closed form so that only defects need labels. This is a Fermi argument until U6 measures it.
- **The far wave's cost** on a first sight of very large broken terrain (a cave system). The opening angle bounds it in principle (redteam-opus.md:43-45); not measured.
- **Frazzoli's maneuver automaton** is the closest prior form of §1 (gaits as relative equilibria, transitions as maneuvers, planning as choosing counts). I read its abstract only; the paper should be read in full before U1 freezes.
