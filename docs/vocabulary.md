# The vocabulary: the word record (Phase 3.1, draft 2)

The word is the one object (representation.md, section 3): an edge for the search, a certificate, a cache entry, a worldtube and a training datum. This spec fixes its record. It is a one-way door, so the Designer red-teams it and Julien reviews it before it freezes.
- **Draft 1** (9 November) keyed segments on the exact start bits. The Designer showed that food, exhaustion and position never repeat, so the hit rate on natural terrain would be about 0% by construction.
- **Draft 2** takes the Designer's seven changes.

## 1. Segments and words

- **A segment** is the unit of compilation: a feedback program run from one event to the next (section 4).
- **A word** is a chain of segments, with its cost the sum of theirs in (min, +). Distinct pieces are computed once, and everything larger is references to them, as in Hashlife.
- **Compound words** (chains promoted because they recur) keep their idea. Their rule (minimum description length, ES 24) waits until search depth calls for it.
- **Why segments, not words:** a whole-word key matches with probability about 0.9 per terrain column, so a 30-block word matches with about 0.2%. One changed block voids it.

## 2. Two roles: the class proposes, the replay certifies

The cache does two different jobs, kept apart:
- **The class** (section 3) is the cache key. A hit is a **proposal**: a program, its arguments, the exit kind and a tick range. Quantization lives only here.
- **The certificate** is the concrete segment, replayed through the law from the bot's real state. Its key is footprints.md's (exact reads, translation scope), unchanged. Nothing reaches the executor without a replay.
- **Why always replay:** a 12-tick replay costs 1–6 µs, about the same as a 2–4 µs lookup, so skipping it saves nothing. What the cache saves is the **search**: about 54 programs × aims × 12 ticks, roughly 10⁴ ticks or 1–5 ms per decision, comes down to about 20 µs. A wrong proposal costs one replay ("the simulation decides, heuristics schedule", ES 22).
- **ES's lesson holds:** ES rejected a quantized memo for kicks because re-rounding at every kick is a random walk (ES 13). Here the quantized class only proposes, and the bits come from the replay, so errors never compound.

## 3. The class: the state the law reads

- **Recorded like cell reads.** The class holds the state fields the law reads before it writes them, recorded by the same induction as the footprint's cell reads. Fields the law overwrites before reading drop out: the previous input, yaw and pitch, `movement_this_tick` and the `was_*` flags.
- **Resources leave the key.** Food, saturation, exhaustion, the food timer and health become **guards** (food above 6 throughout, health above 0) plus **deltas** in `uses`. They change every sprint tick, so keying on them would stop gaits repeating and reruns hitting.
- **Position** is relative to the body's block, quantized to 1/16 block. Exact position keeps adding up and never repeats.
- **Velocity** is kept on attractor bits where the segment starts on a gait, and on a grid otherwise.
- **The world part** of the class is the pattern key of the cells the proposal's replay read, the last time it was compiled.

## 4. Events: one rule

A segment ends at the first tick where **any discrete field of the state changes**:
- the booleans (on ground, collisions, in fluid, climbable, and the others);
- pose, sprinting, and the supporting block's cell or contents;
- the program's active row;
- **or the body's block changes.**

This one rule catches what a list misses: stone to ice, soul sand or honey; step-ups onto carpet, slabs or mud, where `on_ground` stays true; cobwebs and berry bushes; losing sprint; taking damage. The block-change cut keeps segments short. A segment without a bound would bring back the whole-word key (0.9ⁿ). Members of one class that reach different exits are a width question (section 7), not a determinism one.

## 5. The program

- **A program** is a small table from predicates to inputs.
- **Predicates** are facts of the law's state:
  - on ground, collisions, in fluid and its height band, climbable, the jump delay at zero, sprinting;
  - **support ends this tick** (the law's own `can_fall`), for edge-timed takeoffs and neo run-ups;
  - **ticks since the segment started.** In the air every contact predicate is constant, so mid-air strafe switches, neo jumps and releasing jump at the right tick need it.
  
  Predicates are reads, so they enter the class.
- **Inputs:** the keys, the aim, item use, and **block interaction** (doors, trapdoors, gates, buttons).
- **Aims** are sine-table indices on a grid of 64 yaws, which loses about 0.1% of heading against about 2% at 16. Narrow segments refine their aim by local search and keep it as an argument. There's no list of named technique aims: a catalogue would block rediscovery (ES 4).
- **Compiling:** gaits are the attractors of tiny programs per regime (the 9 November probe found 10–15 per ground type). Transitions come from a local exact search between gait classes.

## 6. Gaits

- **A gait is a cycle of classes:** the exit class of its last segment is the start class of its first. This covers the head-hitter, whose period is cut by ceiling events.
- **A long run** is the cycle repeated, and a bot is (segment, phase), as in ES's cycle memo. Each cycle is still replayed; the class only names what to run.

## 7. Width: the price of risk, not a gate

The server's noise isn't a cloud in position. It's a one-tick input delay, port errors (the deviation stream) and entity pushes. Each certified segment records:
- **delay tolerance:** pass or fail when its inputs arrive one tick late or one tick early;
- **clearance:** the smallest margin to a collision threshold, read from the collision code during the replay.

The executor re-anchors at every event by replaying from the observed state, so width prices risk in the search; it doesn't gate reuse. Narrow segments (gaps at the jump limit, clutches, edge back-off) get bit-exact gates (SR-71).

## 8. The record

| Field | What it holds |
|---|---|
| class | the read state from section 3, plus the world part |
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

- **Nodes** are concrete states. Expansions are the proposals for the node's class, each certified by replay. The closed set recognizes repeats by class, which is why section 3 merges what the law can't tell apart.
- **Fermi for a 1,000-block route:** about 300 decisions × 10 proposals × 10 for A*'s overhead × 5 µs ≈ 0.15 s.
- **The allocator** (one queue ordered by slack, representation.md, section 5) schedules compilation, and idle time compiles ahead.

## 11. Rungs and kill tests

- **The proposal hit rate,** per segment and weighted by ticks, with refused keys counted apart, on natural terrain and along the curve from superflat to random terrain. Kill: below about 50% on natural terrain.
- **Segments per regime:** kill at ≫ 10³.
- **Certification:** every reused segment was replayed bit for bit; the audit checks it.
- **The first flight:** the bot races Julien's 18 tracks on certified chains, with his T as the bar.

## 12. Open

1. Position quantization: is 1/16 block right, or should it come from the law's own thresholds?
2. The velocity grid off the attractors.
3. When a gait cycle's class repeats but its bits drift slowly (food, the sine table): is replay per cycle enough, or does the executor need a check per event?
