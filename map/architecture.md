# Architecture

The structure, in the order it is built (bottom up, S191). Each part links to its detail.

## 1. The law: nature reproduced (detail/engine.md)

- **Nature and the judge** are Mojang's 26.2 code and the server. **The law** is the Rust structure we write to reproduce them (S46), bit-exact for player movement and collision, server-exact for health, food and breaking, behaviour-exact for fluids, mobs and redstone.
- It comes at **three grains**: the tick recurrence (verification), the word (exact costs between events), the medium (bounds over regions).
- It grows from three sources: the obvious mechanics read from the jar, deviations seen against the game, and what a new word needs (S46).
- **Generator basis & compression:** the law is structured as ~5–6 composable, parameterized generators plus data tables and thin Mojang scars (dual-face: `k = 1` tick oracle at contacts and `k = Δt` event word; true non-affine kernels like elytra, pistons, `PortalForcer` stay separate). Keeping the core compact preserves L1/L3 cache and one agent's context window; deletions are proven sound against jar witnesses, replays, and live flights.
- Arithmetic copies Java exactly: IEEE floats and doubles, the 65,536-entry sine table, the same operation order.

## 2. The space under a kit: a crystal with defects (detail/representation.md section 10)

Four folds remove the raw problem's dimensions:

| Fold | Removes |
|---|---|
| translation over uniform ground ("the modulus of my best tool", S120) | position: a gait is (word, phase, count) |
| dissipation: every regime has drag | velocity, beyond momentum's memory (~268 ticks on stone) |
| events: one small rule between contacts ("ticks are just an execution layer", S118) | ticks |
| kit classes: a block is its answers to the kit's verbs ("A flower is air", S134) | the block alphabet |

- **Words live in velocity space:** on ground each horizontal axis is `v ← (v + input(yaw)) × friction × drag`, a contraction onto the gait; position is its running sum, read by the exact mantissa/parity cycle skip in `advance_cycles`.
- **The objects:** a **zone** is where one word's certificate holds, found by one corridor query; a **word** (`Word` = `Atomic` or recursive `Compound(Vec<WordId>)` with `AimRule` incl. frame-relative `Click(KitTemplate)`, universal `self_chains`, Zipf rank, and rotation-invariant `DefectSig`) is a coordinate-free local program learned raw on section-backed terrain with `TopoField` and merged across tracks; a **route certificate** (`CachedRoute` + `NoRegen` resource guard + `ObligationLedger` restore debt) is recorded once for the winning path only (`RecordingWorld`) and invalidated locally by block edits inside its footprint; a **site** (`WordStore.routes` keyed by `goal.cell`) hands instant cached segments back; a **label** is T to and from hubs; a **field** is T-to-go for one goal.
- **A tool is a law input you carry** (bucket = water block, block = floor, pearl = teleporter, boat = a surface that ignores falls). An edit is a toll with a debt (its undo); a portal or a pearl is a wormhole. No tool gets search code.
- **Two-regime singularity boundary (other players):** outside contact, the crystal stays pure (`(word, count)`, `(min, +)`, `(max, +)`) and chooses where and when contact happens; inside melee reach, an autonomous rival acts as a branching singularity handed off to a short-horizon dense solver on the same `k = 1` generator kernel, leaving `Word` certificates strictly pure (`found / forbidden / searched / refused`; reference [pvp-strategy-2026-11-22.txt](../archive/research/pvp-strategy-2026-11-22.txt)).

## 3. The one loop: acting on every terrain (detail/learning.md section 0)

```
solve(from, to, kit, deadline):
  1. bracket   LB from the kit's light cone; UB from what is known.
  2. closed?   Exact cached route match (NoRegen guard) or straight gait closes at once.
  3. extend    The biggest word the kit holds; one presence query over its
               worldtube gives its count and the first exception.
  4. cut       Bisects the straight extend to the last clear prefix before the defect.
  5. propose   Phase A queries WordStore (fine -> coarse -> Zipf head) in raw law::try_tick.
     -> learn  Phase B explores raw only on miss on the sections with a TopoField Dial BFS.
  6. certify   Single RecordingWorld pass on the winning route + NoRegen damage guard,
               promoting recursive compounds before returning.
```

- The bot moves at once when the first word's bracket closes; the rest is thought while it moves (free compute).
- A worldtube is checked once, when its certificate is made; the server judges afterwards.
- Changing its mind is `solve` from the observed state.
- No A\* in acting: inside a zone the answer is a closed form; across known joints, two labels merged; at a new site, one backward wave, kept.

## 4. Learning: one queue, one operation, one record (detail/learning.md section 2)

Learning opens where no set arrives or a bracket stays open: search in the law, one joint pattern at a time, kept by content key, never searched again. Its budget is the free compute inside certified words. A record is found, forbidden, searched or refused; "not found" never means impossible.

## 5. The math

| Math | Its job |
|---|---|
| (min, +), Bellman | routes and fields, exact over whole ticks |
| (max, +) | synchronisation: building, cooperation |
| footprints, Bernstein's conditions | what commutes, local repair |
| merging by equal futures (Nerode) | states that the law treats alike are one |
| bounds: the light cone, A\*'s admissibility | prune only by proof |
| events (kinetic data structures) | time advances when a certificate breaks |

## 6. Many bots: one mind in the server (detail/scales.md, detail/server.md)

Bots are real players, driven ES-native by one engine toward a `10⁹`-bot target. Fields are shared; coupling happens only where footprints meet. Quiet streams stay folded in ES `shipmem` (`Form-P` + `Ef` scars + `seal()`) on `calendar::Wheel` (L3-resident); contacts promote to dense `Battle` (`push_pair`) and dissolve back to `Form-P`. Pure observer extraction reads only visible bots (`< 1 ms` max frame, zero mutation). One `16³` section + `4×4×4` 64-bit mask spine serves `O(1)` uniform-box queries (`law::face::advance`), reversible 4D `World(t)` edits, and `SpaceTimeIndex` wake-ups (streams, arrows, periodic `WorldGait`s). Building is (max, +).

## 7. The machine

Ryzen 7800X3D (8 cores), RX 7900 XTX, Windows 11. Threads for the loop's parts; the GPU only for learning's batches after a measured gain; the acting path in microseconds, no allocation, no lock.

## 8. The code (where things are)

| Crate | Holds |
|---|---|
| `engine/law` | `basis/` (the generators), `core.rs`, `face.rs`, `sim.rs` (`WorldDelta`), `dismount.rs`, `shapes.rs`, `state.rs`, fluids/mining/place/interact/boats/portals/pistons/combat |
| `engine/plan` | `solve`, `cut`, `defect` (`TopoField` and the missing-cell guard), `kit` (`KitView`, `ObligationLedger`), `word_store` (`Word`, `NoRegen`), `certify`, `store`, `lattice`, `events`, `goal` (`Goal { cell, face }`: B's clicked face, the detector's zero-epsilon touch); `bin/plan-track`, `tests/generalize`, `tests/kit_unfold`, `tests/face_touch` |
| `engine/world` | 26.2 and 26.3 region palettes; version-matched law snapshots (`StateId`, `STATE_TO_MODEL`, `STATE_TO_BLOCK`, `data/cell_rule_index.tsv`); `16³` paletted sections with `4³` summary masks (`World::uniform`) |
| `engine/words` | the programs the words run (`tiny_programs`) |
| `engine/native` | the bot's mod channel (JNI: `resident`, `kit_wire`, `WORD_PLAN`, `KIT_PLAN`; the goal carries B's face) |
| `engine/build` | building as (max, +): `/fill` and `/setblock` blueprints over `StateId` compiled into a Bernstein footprint DAG by a forward birth wave and a backward 4D demolition wave, with emergent scaffolding, then replayed by `verify_4d`; `bin/plan-build` |
| `engine/mind` | the out-of-process server mind: `WorldMirror` (`StateId` sections from the server's chunks and `.mca` disk sections), shared-memory rings (`shm`, `protocol`), and the bot pool (Virtual, visual projection, materialized) driving `plan::solve` |
| `bot/` | the Fabric mod: the bot, `KitAction`, the stick, tracks (B's clicked face), recording, playback |
| `server-mod/` | the dedicated-server Fabric mod (`SwarmManager`, `ShmBridge`, `WorldBridge`, `VisualProjector`) and, in `client/`, the swarm client that draws every bot in one instanced call |
| `tools/preview` | the offline build preview: views of a command list rendered from the local client jar (`scripts/preview.ps1`) |
