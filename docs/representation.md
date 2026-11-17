# The representation: the framework

How the bots see the world, so that creative travel, cooperation and building cost little for thousands of bots. Approved by Julien on 9 November (S44–S48). Each claim names the test that would kill it, and a claim becomes law only once its rung passes. A fix bolted onto a claim means the claim is wrong (ES 19).

## The problem, the bottleneck, the floor

- **The problem.** Thousands of bots move, use tools, cooperate and build in one deterministic, local, dissipative world. Compute should scale with **what is new** (distinct situations, goals, surprises), never with bots × ticks × places × action sequences. That is ES's rule, and it is where the slack comes from (S46).
- **The bottleneck.** Compiling the law into a small vocabulary of reusable moves that snap together. Minecraft is chess where you first have to invent the pieces.
- **Fermi floors** (floors used to compare designs, never promises, S46). For 10⁴ bots, 10³ of them building, and about 10⁶ distinct patterns in a 10k × 10k area:

  | Work | Floor |
  |---|---|
  | compiling the vocabulary, once per pattern | about 4 core-hours at 0.5 µs per tick, done in idle time |
  | the medium per toolkit class | seconds |
  | fields for 10² goals | about 1 s in total |
  | bot decisions | 0.1 core |
  | edits during building | 0.2 core |
  | surprises | 0.1 core |
  | the server | about 3 ms per tick with one mind and lazy bots, against about 300 ms naive |

  The vocabulary's size and the server decide the outcome.

- **Thinking floors** for the one loop ([learning.md](learning.md) section 0; approved S165). Inputs: the law's tick 0.6–0.95 µs; an arc certificate about 0.7 µs; a memory miss about 80 ns; a new joint pattern about 10³ verifications × 12 ticks ≈ 12 ms of one core; the gait 4.276 blocks per 12 ticks. Derivations: `jobs/unified-plan/plan.md` section 7.

  | Case | Thinking floor | Measured |
  |---|---|---|
  | one million blocks, plain, bare legs, Peaceful | about 5 µs (one label, one memo, about 20 binade events, a two-yaw count) | straight800: 27–29 ms cold |
  | his 21 tracks | 10–100 µs warm; 12 ms per new joint pattern, once | 0.69–30.5 s |
  | `hardtrack` (easy-medium by his ruling, S166) | 0.5–1 ms warm; cold learning fits inside its 260-tick first ladder | no sentence |
  | 10⁴ varied blocks | first word about 100 µs; planning outruns the body by about 3×10⁵ | not run |
  | 10⁴ bots | 0.03 core; 240 KB hot | — |

## 1. One law, three grains

Mojang's code and the server are nature and the judge. **The law** is the generative structure we write to reproduce them (S46). It comes at three grains, each a coarse summary of the one below, as molecules give gas dynamics and gas dynamics gives fluid flow:

| Grain | What it is | What it answers |
|---|---|---|
| Tick | the recurrences (friction × 0.91, jump 0.42 + 0.2 along yaw, collision clamps) | verification |
| Word | the law integrated from one event to the next (section 3) | exact costs, error margins, combinations |
| Medium | words summarized over a region, never averaged: the hull of its gait velocities per toolkit class as the lower bound, a witnessed chain as the upper bound, plus long edges (portals, pearls, ice roads) ([scales.md](scales.md) 4.3) | arrival-time fields, lower bounds, crowd flows |

Each place is looked at with the coarsest grain whose error cannot change the decision (sharp where it matters, blurry elsewhere).

- *Killed if:* a decision made at a coarse grain is overturned by a finer one more often than its stated error allows.

## 2. Coordinates

- **A bot's state** at the planning grain is: which word it is in, its phase in that word's cycle, and where the word sits on the walkable surface. Straight motion is then a constant plus a counter.
- **The world's state** is which words exist where.
- **Programs plus scars** (ES `shipmem`, S59). Everything is stored as an exact program plus sparse scars, each scar an exact difference from the program, never an average ("NO ELITE style").
  - The world is `worldgen(seed)` plus its edits.
  - A swarm on a mission is the mission's program plus each bot's deviations.
  - A bot inside a word is (word, phase): the word's per-cycle increments live once in the shared cache, as in ES's cycle memo.
  - Departures on a grid and formations on a lattice cost about 0 bits per bot; random ones cost about 14. So missions emit programs.
- **Memory follows distinctions** (ES's H_live). The number of new distinctions per second (edits, deviations, new patterns), not the bot count, decides what fits.
- **The planner's coordinate is T itself:** wavefronts are lines of equal T, and every route goes down T. Building T is the work; following it is free (the clay mug, principle 7).
- **Why the vocabulary is small** (S46): every regime has drag (air, water, ground friction, boats, elytra), so every regime settles to a steady motion. A sprint-jump is a 12-tick cycle at 7.127 m/s; an elytra glide at fixed pitch settles to one speed and one sink rate; a boat on ice settles to its top speed.
  - Words are few per regime, and regimes add rather than multiply. Continuous choices (pitch, aim) are a word's arguments, not new words.
  - First evidence (9 November, the probe beside the law): 54 tiny feedback programs gave 10–15 attractors per ground type. Among them, unnamed, were the head-hitter (9.734 m/s, period 3) and the ice tunnel (19.345 m/s). Held keys missed both; jumping only while on the ground found them.
- *Killed if:* words per regime ≫ 10³ on the legs ladder.

## 3. The word: the one object

A **word** is a feedback program: keys, aim and item use chosen from what the body touches, run from a start class until an exit event. It is not a tick-by-tick recording, so small errors are pulled back onto the same exit (a guided missile, not a sniper shot).

**Its signature:**
- start class and exit class;
- cost in ticks;
- width (how far the start may be off and still reach the same exit);
- footprint (cells read and written, including the body's swept volume);
- resources used and produced;
- obligations (edits still to undo).

**Five roles, one record:** an edge for the search; a certificate, valid while its footprint is unchanged; a cache entry: proposals keyed by the class of state the law reads, each certified by a replay keyed by its footprint's contents (vocabulary.md, section 2); a worldtube, for coordination; a training example, later. Words are stored as generators, never as their output (ES 6).

**Kinds:** gaits (repeat on uniform ground); transitions (link gaits across steps, edges, gaps); operators (use a tool).

**Grammar** (S46):
- Two words chain when one's exit class matches the next one's start class.
- Order matters exactly when their footprints overlap (Bernstein's conditions; trace monoids).
- Sequences that recur compile into **compound words** (a "nether portal: 8× travel for T to build", carrying its preconditions and obligations). Each compound word removes search depth from every later problem it appears in.
- New words come from gaps, from R&D, and from LLM proposals (farms, hubs). The law verifies them; use keeps them.
- The record is frozen in `vocabulary.md` (Phase 3.1).

**The cache's unit is the segment, not the word** (Designer, 9 November): a segment runs from one event to the next, a long word is a chain of interned segment ids as in Hashlife, and every word takes a count ([vocabulary.md](vocabulary.md) sections 1 and 5). Per segment, plains give about 10⁵–10⁶ distinct patterns, which a swarm's warm cache covers. Classes that behave the same under the kit's verbs merge (section 10).

*Killed if:* on natural terrain the table certifies no more ticks per µs than the same search without it, its regret exceeds the bracket the class states, or the vocabulary grows as fast as the pieces walked ([vocabulary.md](vocabulary.md), section 11). A hit rate cannot kill it: an empty class would score 100%. Or a narrow word fails within its width under the measured noise.

## 4. A tool is a law input you carry

The law reads the world (blocks, fluids, entities) and the body (attributes, equipment, effects). A tool is one of those inputs, carried:

| Tool | What it carries |
|---|---|
| bucket | a water block |
| block | a floor |
| scaffolding | a ladder |
| boat | a fast surface and a body that ignores falls |
| pearl | a teleporter |
| pickaxe | the power to delete stone at a rate |
| potion | a different body |

Consequences:
- **Local and temporary:** a tool is released within reach (4.5 blocks) or on oneself, and lasts until undone, so restoration is an obligation tied to a place.
- **Its value lies downstream:** a placement is judged by how much it lowers T where it is applied, never by distance to the goal. That is why a bucket placed upward looks good, not bad.
- **It deforms the light cone:** water lets you go down at 78 m/s and up at swimming speed; pearls carry you 50 blocks in about 3 s.
- **At a distance it's the same thing:** a tool in another bot's hand, or a feature in the world, is the same input at a price: zero in hand, the carrier's T, or the walk to it. Tools, cooperation, logistics and terrain are one concept.
- No new search code per tool (AGENTS.md).

*Killed if:* adding a tool needs search or geometry code.

## 5. The math

| Math | Its job here |
|---|---|
| (min, +), Bellman | routes and fields: composition where order matters, exact over whole ticks |
| (max, +), Cuninghame-Green; Baccelli et al. | synchronization: a step starts when the last thing it waits for is done (cooperation, building) |
| Footprints: Bernstein, trace monoids | what commutes: reuse, local repair, never trying every ordering of independent bots |
| Merging: Nerode, Birkhoff | states with the same futures merge; motion settles onto attractors |
| Bounds: A*; delete relaxation; the light cone | the medium is optimistic, words are feasible; refine only where the gap could change a decision; pruning only by proof |
| Events: kinetic data structures | time advances when a certificate breaks, not every tick |
| Bit-parallel (min, +) (ES) | the light cone spreads by shift and OR over occupancy bitmaps: one step for 64 cells per operation |
| The allocator: one queue ordered by slack (ES 12) | each bot's next decision deadline against the compute it needs; idle time precomputes words and recompresses; first-move latency at p99 is its measure |

**The far field** (multipoles, a tracked request) is the medium's bounds, portal tables and labels. The gap between lower and upper bound is the stated error. Min-plus has no general low-rank expansion (the subcubic-equivalence wall, foundations research), so compression comes from bounds, refined where the gap could change a decision, and from keeping a far feature by its effect on T, never its size ([scales.md](scales.md) 4.3).

## 6. Creativity, cheap and shared

- **Pulled by need.** Discovery starts where the bracket is widest: the medium's T far above what the kit's light cone allows, where bots need to cross. A specific effect has few producers, so working backward branches about 5 ways per step ([scales.md](scales.md) 4.1, which holds how creativity arises, stays and compounds).
- **Found once, used by all.** A discovery becomes a word in the medium; every bot sees the road, and the pattern key shares it.
- **Chains.** The relaxed medium (every tool effect available where it could apply, at its least cost) makes each preparatory step look downhill.
- **R&D** (S46) is a barbell: most compute goes to demand, a small slice to cheap trials with rare large payoffs, kept for their option value.
- *Killed if:* discovery needs ≫ 10³ verifications per gap.

## 7. One mind in the server

Bots always join a server Julien fully controls, and he sees them as real players (S46).
- **One engine** drives every bot from inside the server. World changes reach it once, not once per bot. Bots don't receive each other's movement, because the engine already knows it (no n² tracking).
- **Particles and fluids:** spread-out bots ride their goal's field independently; dense bots behave like a fluid, with the game's own player push as the local physics.
- **Coupling only where footprints meet.** Reservations exist only for scarce things (one bucket, a one-wide passage), with priority to the least slack.
- **Ant behaviour without ant code.** Planned routes form a demand field (stigmergy). Build if the total T saved beats the cost (the threshold). Congestion raises T (negative feedback). Claims stop duplication.
- **Lazy bots.** A bot on a certified path isn't ticked until something can see or touch it, or it reaches a decision.
- **Two evaluation orders, chosen by a dimensionless number** (ES, S59): events per cell per light-crossing, a Knudsen-like number, not a category of object.
  - **Sparse** (≪ 1): certificates, the causal frontier and the slack queue, on the CPU. This covers lone and lazy bots.
  - **Dense** (near 1): cells in lockstep, on SIMD or the GPU. This covers crowds pushing each other, fights and mob swarms, where certificates break every tick and stop paying.
  - Both give the same bits. The GPU stays unqualified until a measured crossover exists (ES fermi.md).
- *Killed if:* server ms per tick per bot stays above its floor once lazy bots exist.

## 8. Missions, restoration, prices

- **Missions** (S46). A mission minimizes real time to completion, including its cleanup; planning time counts; a trip is a mission of one. Restoration is per mission: at completion the world is as it was (identical block states, fluids, redstone; no item left lying; inventories whole), so obligations pass to the nearest capable bot. Irreversible effects are barriers (claim of S32 kept). An order may relax this later.
- **Prices are fields.** An item's price is the T to get another one from its nearest source (chest, farm, mob). Farms are words that lower prices, and cheap tools make the swarm more creative. This answers S13. Until sources exist, a consumable has a budget per command: zero by default, and on a track the count he carried (S28). (S166.)
- **Health and food** regenerate: they are constraints, not prices.

## 9. Building is the same problem

A blueprint is a list of edits known in advance, and workers plan against the forecast world W(t). A worker's goal is any spot from which its block is in reach and in sight. A block can go in at max(its support exists, a worker arrives) + placing time: (max, +) over (min, +) travel. The schedule keeps as many productive work spots open as possible, and scaffolding words are valued by the work spots they open. The build order: [scales.md](scales.md) 4.5.

## 10. The space under a kit: a crystal with defects

Approved S165. It answers S40 ("we're not really doing x,y,z but something else my question is what is this something else?") and it is ES's thesis, "symmetry is compression", applied to terrain. The loop that reads this space is [learning.md](learning.md) section 0.

**Four folds** remove the dimensions of the raw problem (the law's state × the inventory × the world's edits × the tick):

| Fold | His words | What it removes |
|---|---|---|
| Symmetry: translation over uniform ground | "i get the modulus of my best tool" (S120) | position: a gait is (word, phase, count) |
| Dissipation: every regime has drag | "i think the regimes should fall out of hand" (S40) | velocity, beyond momentum's memory |
| Events: one small rule between contacts | "ticks are just an execution layer" (S118) | ticks |
| Kit classes: a block is its answers to the kit's verbs | "A flower is air." "unless i need to place a block and it bothers me" (S134) | the block alphabet |

- **Momentum's memory is derived.** A sprint-jump cycle on stone shrinks a velocity difference by 0.546 × 0.91¹¹ ≈ 0.19, about 2.4 bits; 53 bits ÷ 2.4 ≈ 22 cycles ≈ 268 ticks, against 263 measured (job 3.2d-3-P1). Closer than that, order matters and the state is carried; farther, a place has a cost per gait.
- **Words live in velocity space** (S131: "maybe we control in derivative directly?"). There the tick is `v ← d·v + a·u(yaw)`, a contraction onto the gait; position is its running sum, read out by exact integer maps per binade.

**The objects.**
- **Zone:** the extent over which one word's certificate holds, found by one corridor query from where the bot is toward where it goes. Never a partition drawn in advance (S132: "they should arise based of needs, some regions are more compressible than others"; X9).
- **Joint** (the node): where one word may end and another begin: an event of the law at a face. Lips, landings, class boundaries, a wormhole's mouth, the cell an edit would change. The standable cell is not a node (S91: "blocks don't have intrinsic T cost"); it stays nature's address.
- **Set** (what a joint holds): the arriving states from which a known sentence reaches the target, stored as **families** and **points** (S182, narrowed after kill test 11 by S188, `jobs/set-rederive/proposal.md`). The key is the **exact arriving state**: every field's bits the law reads as a value (velocity, flags, gait phase, the binade; `jobs/u2-redteam/report.md` C1), except the one coordinate a held word leaves free. Fields the law reads only through comparisons (exhaustion, saturation, food, fall distance) travel as uses with guards, and the supporting block is kept relative to the body. A **family** is an interval of the free coordinate's lattice, cut into **pieces** where the word's swept box crosses a face between cells of different class and at the law's own contact comparisons around that face (the 1e-7 approach and overlap shells, the 1e-5 hit test). All are monotone thresholds computed from the footprint and the exact integer maps (P2), never found by sampling, so there are no more of them than the faces the word crosses and their contact outcomes. Inside a piece the exit is a monotone map of the entry: P2's map in free motion; the face's exact coordinate where the word clamps the free coordinate (a **point**, one per value of the law's non-monotone flags, today only the minor-collision test, `movement.rs:818`); or the entry's earlier image where the law's contact rule holds the body (a **hold**: within 1e-7 of the face and followed by another collider in the law's visit order, `shapes.rs:399`, `movement.rs:613`; at most 2×10⁻⁷ wide, merged by the next clamp; S203, `jobs/pin-rederive/proposal.md`). Points and holds that become bit-identical merge. Each piece carries its T and next word (over points, a bracket that closes when they agree), its premises (S95) and the prices of what is in hand (S96).
- **Edge:** a word between two joints, with T as a step function of the entry piece.
- **Site:** joints closer together than momentum's memory. It hands upward one portal table (entry class → exit class, T), shared by content hash. A plain has no sites.
- **Label:** a joint's T to and from a few hubs, goal-free; two labels answer a far query in one merged sweep. **Field:** T-to-go on joints for one goal, shared by every bot going there (S19).

**The kit, folded in** (section 4: no new search code per tool).
- **Edits are tolls with a debt** (S32). Under a kit that can restore it, a wall is a slow medium: "a wall is a two second delay not a wall" (X4), plus the T to put the identical block back (S12). Routes bend at its interface by Fermat's rule. Edits whose undo closes inside the gait's period (place ahead, break behind) are gaits with their own speed; the rest are ledger debts (scales.md 4.9). Allowed on tracks whenever restoration before arrival is certified; elsewhere only with a kit given for it, until the ledger has flown. (S166.)
- **Wormholes** (S132: "a nether portal is a manifold … as if it is always there but you need to do a weird path to access it"). A pearl is an edge not bounded by the legs' c, whose landing is a cone (its spread is random), with its item, 5 damage and a 20-tick cooldown. A portal is an edge that exists once built: 80 ticks of wait, an 8:1 chart, a debt on both sheets; the weird path is the compound word that builds it.
- **The light cone is the kit's:** the least over legs at their c, each wormhole as (to its mouth) + toll + (from its exit), and each carried gait (boat, elytra) at its own c. A bound that assumes no item is false for a kit that holds one.
- **Other players** are wildcards with cones. A certificate is checked against a cone once, at the earliest tick the cone can reach its worldtube (S121).

## 11. Kept from earlier drafts (the register points here)

- **The four questions** (X3). Break a wall or go around: the wall is a medium with a toll and a debt or it is a defect, and the bracket decides. One-high holes: the hole's cells fail the body's shape query, for that pose. A bucket is excellent where its edge undercuts the bracket. Sprinting is needed where the landing's set holds only sprint-speed plateaus.
- **Local consistency, repair in both directions** (S23). T-to-go at a joint equals the least over its edges of (the edge's T + T-to-go beyond). An edit that lowers a T spreads by one more `min`; one that raises it voids only the sets whose premises read the change (learning.md section 1; engine.md, "Local repair").
- **Ridges** (S23; S132: "water can branch on saddle points"). The bot flows down T-to-go and decides only where two words differ. In whole ticks most ties are worthless and are broken by a fixed order at no cost; a real saddle is a joint where the choice moves T by a tick or more.
- **Still open:** goals as reach regions (a builder's goal is any spot from which its block is in reach, section 9).

## The hard-to-undo choices (red-teamed before they freeze)

The set record (families, pieces and points; premises; prices) and the label format (section 10); the word record and its canonical pattern key; footprints recorded by the law; determinism, with randomness as an input; the server boundary (Phase 2.1); mission and obligation semantics. Everything else can be changed and is built when the scoreboard calls for it.
