# Architecture

The structure, in the order it is built (bottom up, S191). Each part links to its detail.

## 1. The law: nature reproduced (detail/engine.md)

- **Nature and the judge** are Mojang's 26.2 code and the server. **The law** is the Rust structure we write to reproduce them (S46), bit-exact for player movement and collision, server-exact for health, food and breaking, behaviour-exact for fluids, mobs and redstone.
- It comes at **three grains**: the tick recurrence (verification), the word (exact costs between events), the medium (bounds over regions).
- It grows from three sources: the obvious mechanics read from the jar, deviations seen against the game, and what a new word needs (S46).
- Arithmetic copies Java exactly: IEEE floats and doubles, the 65,536-entry sine table, the same operation order.

## 2. The space under a kit: a crystal with defects (detail/representation.md section 10)

Four folds remove the raw problem's dimensions:

| Fold | Removes |
|---|---|
| translation over uniform ground ("the modulus of my best tool", S120) | position: a gait is (word, phase, count) |
| dissipation: every regime has drag | velocity, beyond momentum's memory (~268 ticks on stone) |
| events: one small rule between contacts ("ticks are just an execution layer", S118) | ticks |
| kit classes: a block is its answers to the kit's verbs ("A flower is air", S134) | the block alphabet |

- **Words live in velocity space:** on ground each horizontal axis is `v ← (v + input(yaw)) × friction × drag`, a contraction onto the gait; position is its running sum, read by exact integer maps per binade.
- **The objects:** a **zone** is where one word's certificate holds, found by one corridor query; a **joint** is where one word may end and another begin (an event at a face); a **set** at a joint holds the arriving states from which a known sentence reaches the target, as **families** cut into **pieces**, **points** and **holds** at the law's own comparisons; a **site** is joints closer than momentum's memory, handing up a portal table; a **label** is a joint's T to and from hubs; a **field** is T-to-go for one goal, shared by every bot going there.
- **A tool is a law input you carry** (bucket = water block, block = floor, pearl = teleporter, boat = a surface that ignores falls). An edit is a toll with a debt (its undo); a portal or a pearl is a wormhole. No tool gets search code.

## 3. The one loop: acting on every terrain (detail/learning.md section 0)

```
solve(from, to, kit, deadline):
  1. bracket   LB from the kit's light cone; UB from what is known.
  2. closed?   If no narrowing can change the next word, emit it.
  3. extend    The biggest word the kit holds; one presence query over its
               worldtube gives its count and the first exception.
  4. cut       The exception's joints split the problem; each part is solve().
  5. compose   Sets come backward in min-plus; the point goes forward.
  6. allocate  Think where it can change the decision due soonest.
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

Bots are real players, driven by one engine in the server. Fields are shared, particles (bots, items, tasks) are individual; coupling only where footprints meet; a bot on a certified path is not ticked until something can touch it. Building is the same problem in (max, +).

## 7. The machine

Ryzen 7800X3D (8 cores), RX 7900 XTX, Windows 11. Threads for the loop's parts; the GPU only for learning's batches after a measured gain; the acting path in microseconds, no allocation, no lock.

## 8. The code (where things are)

| Crate | Holds |
|---|---|
| `engine/law` | the tick law (`movement.rs`, `shapes.rs`, `state.rs`), boats, mining, the sine table |
| `engine/plan` | `solve`, the gait store, joints and sets, lattice, events; `bin/plan-track` (to the game), `bin/deviation` |
| `engine/build` | the 4D reverse-demolition & Bernstein footprint causal DAG compiler for blueprints; `bin/plan-build` |
| `engine/world` | region files, block table, the world interface |
| `engine/words` | the programs the words run (`tiny_programs`) |
| `engine/native` | the bot's mod channel (JNI) |
| `bot/` | the Fabric mod: the bot, the stick, tracks, recording, playback |
