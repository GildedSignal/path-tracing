# The vocabulary: the word record (Phase 3.1, draft)

The word is the one object (representation.md, section 3): an edge for the search, a certificate, a cache entry, a worldtube and a training datum. This spec fixes its record. It is a one-way door: the Designer red-teams it, Julien reviews it, and then it freezes. Draft of 9 November.

## 1. Segments, words, compound words

- **A segment** is the unit of compilation and of the cache: a feedback program run from one event to the next (section 3). It is keyed by the pattern key (footprints.md, section 3).
- **A word** is a chain of segment ids, with its cost the sum of theirs in (min, +). Words are what the search and the executor use; segments are what gets stored and shared. This is Hashlife's split: distinct pieces are computed once, and everything larger is references to them.
- **A compound word** is a chain promoted to one entry because it recurs. It is kept while it shortens the description of the routes actually used by more than it costs to store (minimum description length, ES 24), and pruned otherwise.
- **Why segments:** a whole-word key matches with probability about 0.9 per terrain column, so a 30-block word matches with about 0.2%, and one changed block voids it (the Designer, 9 November). Per segment, natural terrain gives about 10⁵–10⁶ distinct patterns, which a warm cache covers.

## 2. The segment record

| Field | What it holds | Why |
|---|---|---|
| `id` | an interned index | references in chains |
| `program` | the feedback rule (section 4) and its arguments | what the executor runs |
| `start` | the start state's bits, relative to the start block (footprints.md: `t` left out) | the start class; bit-equal for now |
| `exit` | the exit event's kind (section 3), the exit state's bits relative to the start block, and the tick it fires | what the next segment must start from |
| `ticks` | the exact cost | T |
| `key` | the pattern key and its entry: footprint shape, interned cell ids, translation scope, extra-collider box | reuse anywhere the contents match |
| `swept` | the cells the body's box passed through, per tick | the worldtube: coordination and reservations |
| `width` | the start perturbations that still reach the same exit kind and exit class (section 6) | robustness under the server's noise |
| `uses` | resources consumed and produced | prices |
| `obligations` | edits the segment leaves to undo, and the world hooks it triggers (pressure plates, tripwires, buttons) | restoration and the deviation stream |

A movement segment has no resources and no obligations, except world hooks. Operator segments (Phase 4) fill both.

## 3. Events: where segments cut

A segment ends at the first event, read from the law's own state after a tick:
- **ground contact changes:** takeoff (`on_ground` true to false) and landing (false to true);
- **horizontal collision** starts or ends;
- **a vertical collision with the ceiling**;
- **fluid** entered or left (the law's fluid heights);
- **climbable** entered or left;
- **pose changes** (crouch, swim);
- **the program's own switch:** a rule in its table changes;
- **the steady-state cut:** a gait whose state repeats with period p, cut every p ticks (section 5).

Events are facts of the law's state, never declared per block or per place (ES 4). A segment never spans two events, so the same local situation always produces the same segment.

## 4. The program: a feedback rule over contact

- **A program** is a small table from contact predicates to inputs. The predicates are the law's state facts: on ground, horizontal collision, in fluid and its height band, climbable, the jump delay at zero, the sprint state. The inputs are the keys (forward, back, left, right, jump, sneak, sprint), the aim, and item use.
- **The aim comes from a discrete set,** because a continuous aim never repeats and the key would never be consulted. Draft: 16 yaws on the sine table's grid, plus the yaws the elite techniques need (the 45° strafe, run-up aims), and pitch only where a word needs it (looking down for a block).
- **Arguments are part of the key's start class:** two programs that differ only in an argument are different segments.
- **Compiling programs:** gaits are the attractors of tiny programs per regime (the 9 November probe: 54 programs, 10–15 attractors per ground type). Transitions come from a local exact search from one gait's exit class to another's start class.

## 5. Gaits and the steady state

- **A gait** is a program whose state, after warm-up, repeats bit for bit with period p: the exit state of a cycle equals its start state, relative to the start block.
- **Chaining a gait with itself** is then free: a 100-block run is one segment id repeated, and the bot's state is (segment, phase). That's ES's cycle memo.
- **Warm-up** segments (from rest to the attractor) are ordinary transitions.
- **A gait with no event** (walking on flat ground) is cut at its period. If the period is one tick, it's cut every k ticks, where k is the shortest run that crosses a block boundary on every axis it moves along. Open (section 9).

## 6. Width

- **The width is the set of start states** that reach the same exit kind and the same exit class, so the executor can re-anchor on the observed state and still use the segment.
- **For now the start is bit-equal** and the width is measured, not used for reuse: the compiler replays the segment from perturbed starts (position and velocity, per axis, at the scale of the server's observed noise) and records the largest box that keeps the exit.
- **Narrow segments** (gaps at the jump limit, clutches, edge back-off) have small widths. They get bit-exact gates (SR-71); wide segments never need them.
- **Merging start classes** follows the law: states that differ only below `aiStep`'s thresholds have the same future (Nerode; engine.md, ES 6). A wider start class needs the footprint's union over the class (footprints.md, section 3).

## 7. Chaining: the grammar

- **Segment A then B is valid** when A's exit state equals B's start state, after translating by the exit's block offset, with the shift inside B's scope.
- **The cost of a chain** is the sum of ticks. Its footprint is the union of its segments' footprints, each shifted.
- **Order matters only where footprints overlap** (Bernstein's conditions): two bots' segments with disjoint footprints commute, so they're never ordered against each other.
- **A segment is valid while its footprint's contents are unchanged:** an edit inside a footprint voids that segment only, never the whole word.

## 8. Lookup, reuse and compilation

1. **At a decision,** the planner asks the table for segments that start from the bot's current state (bit-equal start, translated), in this world: `PatternTable::lookup`.
2. **An exact hit** (shift in scope, extra colliders empty) is reused as recorded, with no replay.
3. **An inexact hit** (shift out of scope) replays through the law. A replay that matches is reused; one that doesn't is a miss.
4. **A miss compiles:** gaits first, then the transitions the search needs, each entered into the table.
5. **The allocator** (one queue ordered by slack, representation.md, section 5) decides what to compile when; idle time compiles ahead along likely routes.

## 9. Open (for the red-team)

1. **The width's definition and measurement:** which perturbations, at what scale, and is the largest box the right shape?
2. **The steady-state cut** for period-1 gaits: is "every k ticks" right, or should a period-1 gait be one segment of unbounded length, valid while its footprint holds?
3. **The aim set:** 16 yaws plus named ones, or a grid derived from the sine table's exact values?
4. **The start class beyond bit-equal:** when to merge by the `aiStep` thresholds, and whether that changes the key.
5. **The exit class for chaining:** bit-equal exit to start loses chains whose bits differ below any effect. Should chaining use the merged class of question 4?
6. **Obligations and world hooks:** is "a list of edits to undo" enough for Phase 4's operators (a bucket placed, a block broken), or do obligations need their own footprint?

## 10. Rungs and kill tests

- **The pattern-cache hit rate,** per segment and weighted by ticks, with refused keys counted apart, on natural terrain and on the curve from superflat to random terrain (the degradation benchmark). Kill: below about 50% on natural terrain.
- **Segments per regime:** kill at ≫ 10³.
- **Exact reuse:** every exact hit, replayed in a sampled audit, matches bit for bit. A single mismatch is an anomaly that blocks the claim.
- **The first flight:** the bot races Julien's 18 tracks on chains of these segments. His T is the bar.
