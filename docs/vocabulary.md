# The vocabulary: the word record (version 2, 11 November)

The word is the one object (representation.md, section 3): an edge for the search, a certificate, a cache entry, a worldtube and a training datum. This spec fixes its record, and it changes only by a new version.
- **Version 1** (frozen 9 November, S60) took the Designer's seven changes to draft 1, which keyed segments on the exact start bits: food, exhaustion and position never repeat, so its hit rate on natural terrain was about 0% by construction.
- **Version 2** (11 November, S79–S81) takes the red-team's five changes to the class (a Fable sub-agent in the Designer's role), approved by Julien: the class is the law's facts with no chosen number; the search never merges by class; idle time audits hits for lock-in; a gait is an exact recurrence; kill lines that can fire.

## 1. Segments and words

- **A segment** is the unit of compilation: a feedback program run from one event to the next (section 4).
- **A word** is a chain of segments, with its cost the sum of theirs in (min, +). Distinct pieces are computed once, and everything larger is references to them, as in Hashlife.
- **Compound words** (chains promoted because they recur) keep their idea. Their rule (minimum description length, ES 24) waits until search depth calls for it.
- **Why segments, not words:** a whole-word key matches with probability about 0.9 per terrain column, so a 30-block word matches with about 0.2%. One changed block voids it.

## 2. Two roles: the class proposes, the replay certifies

The cache does two different jobs, kept apart:
- **The class** (section 3) names a piece so that it is recognised. A hit is a **proposal**: a program, its arguments, the exit kind and a tick range.
- **The certificate** is the concrete segment, replayed through the law from the bot's real state, with the footprint that replay records (footprints.md). Nothing reaches the executor without a replay. Nothing is filed by its exact start except a gait's cycle (section 6).
- **Why always replay:** a certified replay costs about 11 µs and a lookup about 5 (job 1.6, measured). What the cache saves is the **search**: inventing a piece from about 54 programs × aims is about 10⁴ law ticks, at about 2 µs per tick on real terrain about 20 ms (derived, not measured). A wrong proposal costs one replay ("the simulation decides, heuristics schedule", ES 22).
- **ES's lesson holds:** ES rejected a quantized memo for kicks because re-rounding at every kick is a random walk (ES 13). Here the class only proposes, and the bits come from the replay, so errors never compound.
- **Every surprise is data** (Julien, S81). A proposal whose replay reaches another exit shows where the class is too coarse; a replay the client or the server contradicts shows where the law is wrong or incomplete, or where the server does something we did not consider (the deviation stream, engine.md). Each is logged with its replay as the witness and never suppressed: this is how the port finds its own gaps.

## 3. The class: the facts the law decides on

- **What it is for.** The class names a piece so that it is recognised. A hit proposes; the replay from the exact state certifies. The class never merges states for the search: the closed set and every pruning use exact states or a proof (section 10).
- **Recorded like cell reads.** The class is built from the state fields the law reads before it writes them, by the footprint's induction. Fields overwritten before they are read drop out: the previous input, yaw and pitch, `movement_this_tick`, crouching and the `was_*` flags.
- **One rule for each field.**
  - A field whose values come from the block table or the toolkit (attributes, pose, the stuck multiplier) enters by its bits: it is a parameter of the generator.
  - A field that accumulates (position, velocity, fall distance, counters, resources) enters only as the yes-or-no answers of the comparisons the law makes on it: on ground, the collisions, in fluid and its band, climbable, a counter running, at rest. No grid and no chosen number.
- **Resources** are guards (the law's own thresholds) and deltas in `uses`.
- **The world part** is the pattern key of the cells a proposal's replay read. It orders proposals and never excludes one: on a mismatch the class's other proposals are replayed before any search.
- **A new distinction** enters only when a counter asks for it and a consumer needs the prediction, and only as a comparison the law makes at that place. The red-team found surprises concentrate on the tick the body crosses into the next block, where the law makes no such comparison; none is added (section 12).
- **Counted per class:** misses, surprises, refusals, proposals replayed per decision, and the regret found when idle time searches a sample of hits.

## 4. Events: one rule

A segment ends at the first tick where **any discrete field of the state changes**:
- the booleans (on ground, collisions, in fluid, climbable, and the others);
- pose, sprinting, and the supporting block's cell or contents;
- the program's active row;
- **or the body's block changes.**

This one rule catches what a list misses: stone to ice, soul sand or honey; step-ups onto carpet, slabs or mud, where `on_ground` stays true; cobwebs and berry bushes; losing sprint; taking damage. The block-change cut keeps segments short for recognition; a segment without a bound would bring back the whole-word key (0.9ⁿ). Members of one class that reach different exits are a width question (section 7), not a determinism one. Whether a certificate must end where a piece ends is open (section 12).

## 5. The program

- **A program** is a small table from predicates to inputs.
- **Predicates** are facts of the law's state:
  - on ground, collisions, in fluid and its height band, climbable, the jump delay at zero, sprinting;
  - **support ends this tick** (the law's own `can_fall`), for edge-timed takeoffs and neo run-ups;
  - **ticks since the segment started.** In the air every contact predicate is constant, so mid-air strafe switches, neo jumps and releasing jump at the right tick need it.

  Predicates are reads, so they enter the class.
- **Inputs:** the keys, the aim, item use, and **block interaction** (doors, trapdoors, gates, buttons).
- **Aims** are sine-table indices on a grid of 64 yaws, which loses about 0.1% of heading against about 2% at 16. Narrow segments refine their aim by local search and keep it as an argument. There's no list of named technique aims: a catalogue would block rediscovery (ES 4).
- **Compiling:** gaits are the attractors of tiny programs per regime (the 9 November probe found 10–15 per ground type). Transitions come from a local exact search between gaits.

## 6. Gaits

- **A gait is an exact recurrence:** after one cycle of its program, every field of the state but position has the same bits, and position has advanced by the cycle's displacement. A yes-or-no class alone cannot pick gaits out, since every steady walk is a cycle of classes; the bits do.
- **A long run** is the cycle repeated, and a bot is (segment, phase), the phase being the tick count modulo the period, as in ES's cycle memo: the cycle's increments live once in the shared cache. This is the one place where exact starts recur, so the exact-start store keeps gait cycles only.
- **Each cycle is still replayed** until section 12's question 2 is answered.

## 7. Width: the price of risk, not a gate

The server's noise isn't a cloud in position. It's a one-tick input delay, port errors (the deviation stream) and entity pushes. Each certified segment records:
- **delay tolerance:** pass or fail when its inputs arrive one tick late or one tick early;
- **clearance:** the smallest margin to a collision threshold, read from the collision code during the replay.

The executor re-anchors at every event by replaying from the observed state, so width prices risk in the search; it doesn't gate reuse. Narrow segments (gaps at the jump limit, clutches, edge back-off) get bit-exact gates (SR-71).

## 8. The record

| Field | What it holds |
|---|---|
| class | the facts of section 3 |
| world part | the pattern key of the cells the last replay read; it orders proposals |
| program, arguments | what to run (section 5) |
| exit | the exit kind (the discrete field that changed) and the tick range seen |
| cost | ticks, from the replay |
| width | delay tolerance and clearance (section 7) |
| `uses` | resources consumed and produced, as deltas, and their guards |
| `needs` | held but not used up: a pickaxe, a bucket |
| `writes` | each cell written: before and after, including hook effects (a pressure plate opening a door) |
| obligations | the writes whose undo is owed: the cell, its state before, and the region the undo can be reached from; its footprint is the undo's, derived when looked at |
| swept | the union box of the body's path, for reservations; per-tick detail comes from the replay |

## 9. Chaining

- **A then B** when B's proposal applies to A's exit state, and B's replay from that state is certified.
- **B is keyed against the world after A's writes,** so operator chains (a bucket placed, then the fall) compose.
- **Order matters only where footprints or write sets overlap** (Bernstein's conditions, with write sets): two bots' segments with disjoint ones commute.
- **An edit inside a certificate's footprint** voids that segment only.

## 10. Search with this record

- **Nodes** are concrete states. Expansions are the proposals of the node's class, each certified by replay.
- **The closed set and every pruning use exact states, or a proof of dominance under the law, never the class** (ES 22: pruning is deciding). A sprinting and a standing arrival in one block are different states; merging them is R-brach's kill ("states were merged too coarsely at speed").
- **Fermi for a 1,000-block route:** about 300 decisions × k proposals replayed × 10 for A*'s overhead × (5 + 11 µs); with k = 10, about 0.5 s. k is counted, not assumed (section 11).
- **The allocator** (one queue ordered by slack, representation.md, section 5) schedules compilation; idle time compiles ahead and searches a sample of hits for regret.

## 11. Counters and kill tests

- **Certified ticks per µs**, with the table against the same search without it, weighted by ticks, on natural terrain (default terrain from a frozen seed, offline, planner in the loop; the fixture is built and frozen before its first run). **Kill:** at or below 1, the table costs more than it saves. The search costs about 54 programs × 1.55 ticks × 2 µs ≈ 170 µs per aim, so the table pays only while the proposals replayed per decision stay under about 15 per aim.
- **Regret:** idle time searches a sample of hits and counts the ticks a better piece would have saved (lock-in: a hit suppresses the search, and the replay certifies that a move works, never that it is the best; the ants of scales.md 4.4). It is the class grain's overturn rate. **Kill:** regret beyond the bracket the class states.
- **The vocabulary's growth** (Julien, S81): invention should bend like Heaps' law (new words per thousand pieces walked fall as the walk grows) and use should follow Zipf (a few words carry most ticks), so each invention is amortized. The failure mode is words that don't generalize: data created for nothing. Count words used once, and new words per thousand pieces along the walk. **Kill:** the vocabulary grows as fast as the pieces walked, with no bend, on natural terrain.
- **Segments per regime:** kill at ≫ 10³.
- **Certification:** every reused segment was replayed bit for bit; the audit checks it.
- **The first flight:** the bot races Julien's 18 tracks on certified chains, with his T as the bar.

## 12. Open

1. **Certificate size** (Julien, S79–S80: "of course a parabola should be one certificate … we need to fermi this too"). Today a certificate ends where a piece ends, about 1.55 ticks, about 8 per jump. The integrator's reading, not yet agreed: a certificate lasts until a wildcard's light cone (humans, the server's randomness as cones, our own execution noise) can reach its worldtube; deterministic events are checks, not ends. GLM lane `certificate-size` measures it.
2. **When a gait cycle's bits drift slowly** (food, the sine table): is replay per cycle needed, or does the cycle memo hold across cycles within the footprint's translation scope?
3. **Block crossings as a modulus** (Julien, S81: "a cycle of gate is related to modulous"). On a gait, position advances by the same displacement each cycle, so the ticks where the body crosses into the next block follow from it by arithmetic modulo the block. If the law confirms it (GLM lane `gait-modulo`), the surprises of section 3 become arithmetic and the block-change cut stops mattering on uniform ground.
