# Footprints and determinism

The word's footprint makes it a certificate, a cache entry and a worldtube
([representation.md](representation.md), section 3). This spec says what the
law reports it read, what makes a run repeat bit for bit, and the pattern key.
The Designer red-teamed the key (FREEZE WITH CHANGES); this is the changed key.
Code: `engine/law/src/footprint.rs`, `engine/law/src/pattern.rs`.

## 1. The adapter

`law::footprint::RecordingWorld::new(&world)` wraps any `World` and is itself a
`World`, so every law entry point accepts it. No law arithmetic changed.

- `tick(&mut state, &input) -> TickRecord { initial, input, footprint }`: one
  tick, with the entry `PlayerState` and the controls. The adapter keeps the
  union across calls, so a run of ticks is the same call repeated.
- `server_event(&mut state, event) -> ResourceRecord { initial, event, footprint }`.
- `footprint()` clones the union; `take()` returns it and clears it.
- `Footprint { reads, swept, environment }`, with `contains(p)` and `extend(&other)`.

`reads: BTreeMap<BlockPos, u8>` maps each cell to the union of the queries
that touched it: `Block` 1, `Shape` 2, `Fluid` 4, `Hooks` 8, `Loaded` 16.
`swept: BTreeSet<BlockPos>` holds the cells of the box enclosing the body
before and after its move, and before and after its pose change: worldtube data.
`environment` holds the world-wide values the law read: `MinY`, `FastLava`,
`Difficulty`, `NaturalRegeneration`, the omission certificates `HasFluid` and
`HasInsideEffects`, and each `ExtraCollisions { bounds, context, shapes }` query.

The law tags its reads through `hook_block`, `fluid_block` (both return `block`
by default) and the no-op observers `observe_region` and `observe_sweep`. A fast
path that skips cells first reports the whole region it was asked about, so a
skipped cell is still a dependency. The record may hold cells whose change
alters nothing, never the reverse. Contract: the wrapped world is immutable
while recording and keying; a collision-shape override reads only its own block
and the context.

## 2. Randomness and time are inputs

The ported law draws no server randomness and reads no clock (`verify.ps1` greps
for it). What varies with time enters as a value:

| Boundary | Recorded value |
|---|---|
| movement tick | the entry `PlayerState` and `Input` |
| server regeneration | `ServerResourceEvent::Regeneration(server_player_tick)` |
| coupled resource tick | `tick_server_resources(state, world, server_player_tick)` = `Regeneration`, then `FoodTick` |
| mining | `MiningClock::new(game_ticks)`, then the ordered `MiningAction`s and `MiningSample`s |
| entity contact | both `ContactEntity` states and their order in `push_pair` |

`PlayerState::bits()` is every field as raw bits; same bits, same snapshot,
same inputs give the same bits (tested). A ported law that draws randomness will
take the drawn value as an input at its boundary.

## 3. The key

`PatternTable<V>` (`law::pattern`) keys a word by what the law read and the state
it started from. `insert(&world, &footprint, &start, value)` records an entry;
`lookup(&world, &start)` returns a `Hit { entry, shift, exact }` or nothing.

**Cells: `reads` only, each as its queries see it.** `cell_digest(block, loaded, mask)`
is FNV-1a 128 over the content those queries return:
- `Shape`: the five collision shapes as `World::collision_shape` resolves them
  (`shape`; `above`, else `shape`; `below`, else `shape`; `sneak`, else `above`; `inside_cell`, else `below`),
  each as its grid cells, empty subdivisions included; and whether it is powder
  snow, which replaces its shapes by context;
- `Fluid`: the fluid (kind, height, flow, raw amount and falling) and the flow
  predicates (blocks motion, four solid faces);
- `Hooks`: every field but the five collision shapes, the fluid included;
- `Block`: the whole model (shapes and hooks);
- `Loaded`: the loaded bit, and only where it was read.

This is exact by induction: a cell whose queried answers are equal gives the law
the same answers, so the same queries follow. Names never enter: 30,951 states
share 519 models, and two models the law resolves alike digest alike. `Block` is
destructured exhaustively, so a new law field is a compile error in the key.

**The footprint's form is shared.** A `Shape` is the sorted list of (offset from
the start block, query mask) plus the environment values read (MinY, FastLava,
Difficulty, NaturalRegeneration), interned once for every entry that has it.
MinY enters relative to the start block; the law reads it only when the block
below the body is unloaded.

**Every read is relative to the body.** The fluid check (`update_fluid`,
Level.hasChunksAt) used to ask `loaded` at absolute chunk corners, so a shift
not a multiple of 16 could hit while the law tested other chunks. It now asks
each cell of the same columns: a chunk is loaded exactly when its cells are, so
the law is bit-identical and the read is keyed like any other. Test:
`a_shift_across_a_chunk_edge_cannot_hit_when_the_law_sees_it`.

**Ids.** Each digest (cells and environment) is interned to a dense `u16`; the
digest table is the persistent part, and an id is a handle to content. A `Key`
is `{ start: u32, shape: u32, ids: [environment, cell...] }`, held in std
`HashMap`s (seeded SipHash) with equality on ids. `World::model_id` gives a
cell's row in the static generated table (`WorldSnapshot` has it; the law never
reads it), and the table caches each (model, mask, loaded) slot's digest and id
for its life in a dense array: a known cell costs two world reads and an index.
Worlds without model ids digest per call, memoized by model address.

**Start state: bit-equal, for now.** The start's `bits()` with the position and
the supporting block taken relative to the start block, and the tick counter `t`
left out (the law only counts it), are interned to the `start` id. A hit
requires the same bits. Gait cycles on an attractor have one start state, so
they recur. A width-wide start class (Phase 3) needs the footprint's union over
the width, since a different start can read different cells.

**Translation scope lives in the entry.** `Entry { origin, scope, probe, value }`.
`scope` is, per axis, the least cell to the greatest cell plus one over `reads`
and `swept`. An integer shift is bit-exact while every coordinate keeps its
binade: one sign and one exponent `k` over the shifted range, with `k <= 23` so
the shift is also a multiple of the f32 ulp (the law casts some coordinates to
f32). `in_scope(scope, shift)` checks that; `Hit::exact` carries it. Inside the
scope the recorded bits, shifted, are the law's; outside it the word replays.
Relative positions are exact inside the scope (Sterbenz), which is where the
start key needs them.

**Extra colliders.** A query that returned shapes refuses the entry
(`ExternalCollider`). Empty queries are kept as one box, the union of their
bounds relative to the origin, and re-queried at the hit (with the start's
collision context; vanilla's entity and border colliders do not read it).

**Frames: the identity only.** `model_digest(block, frame)` turns a model's content
by any of the eight horizontal frames, and on the real table it finds the turns
(`world/tests/pattern.rs`: 519 models, 519 digests, 506 closed under the frames;
the 13 others are cakes, candles and the sniffer egg). But the law does not
replay turned runs bit for bit (`world/tests/frames.rs`, 62 runs on Julien's 18
tracks, 60 ticks at most, stopped where the snapshot ends):

| Frame | free yaw | yaw on a multiple of 90 |
|---|---|---|
| identity | 32/32 | 30/30 |
| 1 turn | 18/32 | 1/30 |
| 2 turns | 17/32 | 0/30 |
| 3 turns | 18/32 | 9/30 |
| mirror | 26/32 | 5/30 |
| mirror, 1 turn | 6/32 | 0/30 |
| mirror, 2 turns | 5/32 | 1/30 |
| mirror, 3 turns | 5/32 | 6/30 |

The first gaps are 1e-17 to 3e-5 blocks: Mojang's sine table is not symmetric
under `yaw + 90` or `-yaw`, even at axis yaws. Every frame but the identity is
dropped, so the canonical frame is the identity and no offsets are mapped.
Symmetry stays found by hashing, for a later key that replays turned words.

**Refusals** (`PatternError`): `ExternalCollider`; `UnsupportedHook`
(`Inside::Unsupported` or `unsupported_fall_on` in a Hooks-read cell);
`NonFiniteGeometry`; `TableFull` (more than 65,535 contents).

## 4. What the key leaves out, and why

- **`swept`**: the worldtube, for coordination; the key needs only what was read.
- **Answers no query asked for**: a Shape-only cell accepts any model with those resolved shapes;
  the reach visit (job 1.6) leaves 78–92% of cells Shape-only (judged: 193,668 such edits, bit-equal).
- **`HasFluid` and `HasInsideEffects`**: forcing both true leaves every tick
  bit-identical (`speed_differential.rs`); keying them would split a pattern by
  water a kilometre away.
- **`t`**: counted, never read.
- **Turned frames**: they fail replay (section 3). **Vertical flips**: gravity.
- **Moving colliders**: refused, not keyed.

## 5. Gates and evidence

| Question | Test |
|---|---|
| Can a cell outside `reads` change the tick? | `law/tests/footprints.rs` `outside_footprint_changes_cannot_alter_tick` |
| Can an edit that keeps a cell's answers change the tick? (the digest decides which edits keep them) | `edits_that_keep_a_cells_answers_cannot_alter_tick` |
| Same inputs, same bits? Per-tick footprints sum to the run's? | `recording_replays_bits_and_accumulates_tick_worldtubes` |
| Query kinds and raw fluid neighbours visible? Server time an input? Extra colliders keep bits? | the other three in `footprints.rs` |
| Does a hit need the same contents and the same start bits? | `law/tests/pattern.rs` `translated_copies_hit_and_one_block_or_one_bit_misses` |
| Is an in-scope shift bit-exact, across 2^n and y = 64? Across a chunk edge? | `shifts_inside_the_binades_replay_bit_exact`, `a_shift_across_a_chunk_edge_cannot_hit_when_the_law_sees_it` |
| Are turns found by hashing? Is unknown physics refused? | `turned_models_are_found_by_hashing`, `unknown_physics_is_refused` |
| Real models: separated, symmetry found? Which frames replay? | `world/tests/pattern.rs`, `world/tests/frames.rs` |
| Bytes and µs per entry? | `world/tests/key_cost.rs` (release) |
| Fast paths equal to the full scan? | `law/tests/speed_differential.rs`, `shapes.rs` |

The shift gate replays a 20-tick sprint-jump cycle at 14 shifts. The 6 in scope
are bit-exact; of the 8 out of scope (crossing 1024, 2048, 2^20, y = 64 and
y = 128), 7 differ. The scope is needed, and not too tight.

Cost on Julien's tracks (`key_cost.rs`, release; v1 by `jobs/1.2-footprints/v1-cost.rs`
at commit 8801299; v2 is commit 8711ead, digests memoized per call), per entry:

| Word | Read cells | v1 bytes | Bytes now, own + shared | µs to key: v1, v2, now cold, now warm | µs to look up: v2, now |
|---|---|---|---|---|---|
| 1 tick (32 words) | 90 | 30,028 | 301 + 1,144 | 161, 25, 18, 3.2 | 20, 2.2 |
| 10 ticks (26 words) | 174 | 57,736 | 471 + 2,813 | 230, 30, 23, 6.0 | 25, 4.4 |

Cold is a fresh table, where each (model, mask, loaded) slot is digested once;
warm re-keys the same words. Most of the shared bytes are start states (about 90
words of bits each), shared by every entry that starts alike.

`jobs/1.2-footprints/verify.ps1` runs the workspace, plants twelve mutants (five
fast paths, the single-box path, a missing footprint, a facing that does not
turn, an unkeyed fluid, an unkeyed sneak shape, an unbounded scope, chunk-corner
loaded reads) and requires a running test to catch each, then audits randomness.
`retained-replay.ps1` requires identical replay verdicts on `main` and the branch.

## 6. Open
- Unkeyed 128-bit FNV: a collision gives one content a wrong id; storing contents would rule it out.
- No test drives a body onto a Shape-only powder-snow cell (the random worlds never fall onto one); `collision_cells.rs` only guards that the reach visit never skips powder snow.
- Start states dominate the bytes; a start class would shrink them.
