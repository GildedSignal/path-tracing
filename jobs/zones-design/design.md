# Zones, portals and flow: thinking that mirrors the terrain

**Asked for by** S120, S130–S134. Julien: "the terrains complexity should be a mirror of compute's complexity here our path should take µs not tens of ms" (S131). Zone sizes "should arise based of needs, some regions are more compressible than others" (S132).

**Status:** design for the Opus red-team. After it, Astra folds it into the 3.2d-3 spec (`D:\path-tracing-wt\sets\jobs\3.2d-3\last.md`), and Terra builds the slices.

## 0. Consumers and gates

The design is held against **two extremes** (S131), plus everything that flies today.

1. **A one-million-block straight line**, bare kit, on plain ground.
   - Thinking must not grow with distance.
   - T is end to end, from the click's server tick to arrival (S124).
   - The gate is a Fermi floor, then a measurement: about 10 µs of thinking once the planner is resident (section 4).
2. **The hardest tool-heavy 3D parkour**: branching, combinatorial, with tight timings. Terra builds it later, backward from B, with a hardness certificate per element (S132). The gate is correctness first, with compute in proportion to its complexity.
3. **Today's 21 tracks** keep T or improve it, and L0 is unchanged.

## 1. What we measured (evidence)

**The plain path** (`wt/plain-path`) runs at 29.11 ms median cold on straight800, T 2248 (main 2316). The Terra judge returned FAIL in `D:\path-tracing-wt\plain-judge\jobs\plain-path-judge\verdict.md`. Its three blocking findings:
- **Per-tick work in the decision:** the remainder is simulated tick by tick, and the fallback depends on the per-tick script run (S117).
- **An invented food bound** instead of counted debits from the observed bars.
- **Digest equality without a byte comparison.**

Where its ~27 ms goes (builder's breakdown):

| Step | Cost |
|---|---|
| process start and model-table parse | ~4 ms |
| A's column read | 5.2 ms |
| corridor read | 8.5 ms |
| writing every tick | 4.8 ms |
| the decision itself | ~1.4 ms |

**The inventory** (`jobs\inventory-map\report.md`) checks 35 adopted mechanisms: 6 built, 14 partial, 15 doc only. Among the doc-only ones:
- local repair;
- executor re-anchoring and widths;
- tools and edits in the planner;
- multipoles;
- closed forms and generator storage;
- GPU;
- the server-side mind.

It also found zero threads anywhere in `engine/`.

**Duplicate work**, from the integrator's trace and the inventory:
- **D1. The world is held four times:** server, client chunk cache, region-file copy, planner snapshot. The planner decodes from disk what the client holds in RAM, and its copy can be stale.
- **D2. The trajectory is simulated three or more times per click:** to decide, to write the per-tick file, and by the client. The search adds `certify` per edge, `polish::slide` replaying from tick 0 per slide (O(L²)), and `walk::certify` again.
- **D3. "A trajectory slides unchanged" exists four times:** `arith::Cycle`/free arc, `reach.rs` tables, `gait.rs` memo, `plain.rs` fold. `reach.rs:42-45` also uses `f64` trig instead of the law's sine table.
- **D4. "What the law reads here" is keyed four ways:** footprints, pattern keys, class/table, `law_digest`, with role bitplanes coming in 3.2d-3.
- **D5. The backward T-to-go field (`reach.rs`) exists but only orders A\*.** It is rebuilt per click (`flight.rs:134`), along with a fresh `ProposalTable`.
- **D6. One process per click:** tables are parsed and thrown away.
- **D7. The planner re-simulates settling** instead of taking the bot's observed bits (`plan-track.rs:23`).
- **D8. Messages take the long way:** RCON → chat → mod → file, with replies through a log polled every 50 ms. The Java `StraightLine` planner and the Rust planner are two unrelated entry points, and goal and touch are defined twice (`TrackDetector.java` and `goal.rs`).

## 2. The space

- **The state is a point** (PlayerState). The law is a flow on it, one step per tick.
- **A held word is a current.**
- **A gait is a limit cycle**: every field recurs but position, which advances. In spacetime it is a stretched spring.
- **Contacts are events** that cut the flow. Between them the motion is closed form.
- **The world enters only through the law's reads at events**, in the cells the tube touches.
- **Classes are relative to the kit's verbs** (S134):
  - legs ask collision, support at 0.2 and 0.5, inside, hooks;
  - a pickaxe adds hardness, tool, ticks, drop (silk touch splits classes);
  - blocks in hand add replaceable and face.

  With bare legs there are few colours; every tool adds some.
- **The metric** is T in whole ticks: terraces and cliffs, not slopes.
  - **Ridges** are exact ties between routes, and they are the only real decision points.
  - **Direction matters (Finsler):** falling is nearly free and climbing is paid.
  - **Wormholes:** a pearl is a cheap one. A Nether portal is a built one with an 8:1 chart change: a manifold you open by building a frame, whose toll is the build word (S132).
  - **An edit** is a toll that changes a cell's class (bridging carries a floor with you; "a wall is a two second delay not a wall", X4). Whether an edit is a toll or a ruin is the open contradiction "Editing to get around" in PRINCIPLES, and it is Julien's ruling.

## 3. The representation

**3.1 One role-class codec** (replaces D4).
- A class = intern(verb, query mask, the canonical bytes of what that verb's queries read).
- Lookup by digest, then byte equality.
- Footprints, pattern keys, the proposal class and the presence check all use it.

**3.2 Zones that emerge** (S132: no fixed sizes).
- A zone is a box whose cells all fall in one class set under the kit.
- It is built bottom-up by merging neighbours whose descriptions are equal. It is split only at a coordinate where the description changes, so the boundaries are the cutting points.
- Each tree node holds the set of classes beneath it (a bitset over interned classes) and its bounds. This is the multipole: an ocean is one node, a village is many.
- It is updated by block events, leaf then ancestors.
- **Near field:** fed from the bot client's loaded chunks (no disk).
- **Far field:** from saved regions now, and from `worldgen(seed) + edits` or the server's world later.
- Terrain not yet known is planned optimistically by the light-cone bound and refined as chunks arrive (anytime).

**3.3 One generator store** (replaces D3).
- Key: (law hash, body and kit pack, support class, medium class).
- Value: the word's cycle (period, displacement, read template, resource debits, phase contract).
- Compiled once and kept resident. The plain fold, free arcs, reach tables and gait memo all read it.
- Yaw is an argument, never a key.

**3.4 Portals** (engine.md's certificate, finally built).
- At a zone boundary the interface is a band of states in position × velocity × phase. Momentum crosses it.
- A portal entry reads: "from these entry states, k ticks to these exit states, width w", valid until a dependency changes (an edit inside its footprint, or a kit change).

**3.5 The field, promoted** (replaces D5).
- The backward T-to-go is kept per goal and shared by every bot going there.
- Inside a zone whose metric is certified constant, it is closed form: lower bound = upper bound.
- Elsewhere it is a bound that orders work.
- The bot flows down it and decides only on ridges (ties), breaking ties by the wider tube.

## 4. The loop

1. **Click.** The resident planner receives the bot's observed state bits and the goal on one direct channel (removes D6, D7, D8; every hop still counts in T, S124).
2. **Top down.** The planner queries the zone tree inside the light-cone ellipse from A to B, for this kit.
   - If one plain zone covers the line, the answer is closed form: q cycles plus the remainder from the generator store, the food guard from the cycle's debits, and the binade boundaries crossed (section 6).
   - Fermi for a million blocks: ~10⁴ tree nodes at most along a line in a varied world (fewer on plain), one 12-tick cycle (~1 µs at ~0.1 µs per stripped tick), one division. That is ~1–10 µs.
3. **Segments.** Otherwise the route is cut at zone boundaries. Given its portal states, each segment is independent and solved in parallel (CPU threads first; a GPU wavefront where many states matter, later).
4. **What gets more compute.** The bracket gap and its cause decide:
   - a geometry gap goes to the wavefront in that zone;
   - a timing gap goes to the exact law;
   - a moving thing (a player or entity whose cone reaches the corridor's worldtube before the bot passes) goes to the cone test.

   A\* or a backward producer search runs only inside spans the representation cannot certify.
5. **Execution.** The planner sends the word × count plus its certificate. The client is the only per-tick runner (removes D2). The executor compares the observed state with the generator's prediction at events, re-anchors, and asks for a local re-solve only where the bot leaves its tube.
6. **Other players (S121).** The cone test runs per segment: the player's distance to a corridor point, minus their top speed × (ticks until the bot reaches it + the lag since they were seen). Only segments that go negative are re-thought. Nothing else is touched.

## 5. Build order (for Astra to fold into 3.2d-3)

1. **R0** Resident planner, a direct channel and `from_bits`. Fix the judge's findings 1, 2 and 4: no per-tick work in the decision, counted food debits, byte equality.
2. **R1** One role-class codec (3.1).
3. **R2** The generator store (3.3). The plain path becomes closed form, with D3's four copies deleted.
4. **R3** The zone tree from client chunks and edits (3.2).
5. **R4** The executor contract: word × count, certificates, event checks, re-anchoring.
6. **R5** Portals, the promoted field, segments in parallel.
7. **R6** Players' cones.
8. **Later:** the GPU wavefront, tools and edits as tolls, the Nether chart.

## 6. Questions for the red-team

1. **The 1M-block line hits three walls the 800-block one hides.** Is the design honest about each?
   - Food: sprint-jumping a million blocks spends thousands of food points, so eating must be a word in the generator's cycle.
   - The client only holds chunks near it, so the far field comes from disk or the seed.
   - Position precision changes at each binade, so the fold must split at |x| = 2^k, which is known in advance.
2. Is the zone tree a second copy of the world (D1 again)? Should it replace the snapshot rather than sit beside it?
3. Integer T makes ties common. Does "ridges are the only decisions" still bound the branching, or do plateaus make everything a ridge?
4. Does any step reintroduce per-tick thinking, a made-up number (tree arity, band widths), or a tolerance?
5. Is anything here a second search or geometry engine (AGENTS: "No new search code")?
6. What is missing for the hard-parkour extreme?
