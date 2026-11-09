# The engine

This file covers what the engine is for, the contract it keeps, and how Emergent Space's engine rules map onto it. It holds current truth only; history lives in git.

## Why there is an engine

"run minecraft literally but 100x faster to predict the future … we don't care about textures … and we don't care about what's outside the light cone." (S6) "i want to rewrite the engine to run faster because it'll be simpler and therefore can could explore it" (X6).

The planner has to know what the bot's inputs will do, far faster than the game's 20 ticks a second, and exactly wherever a decision depends on it. That is all the engine is.

**Speed.** The law costs about 0.6–0.95 µs per player tick on one core of the 7800X3D: median 590–940 ns across walking, sprint-jumping, under a ceiling and bumpy ground (job 1.1, 9 November; regenerate with `cargo run --release --bin bench` from `engine/`). That's about 5×10⁴ real time, bit-identical, against a Fermi floor of about 0.1–0.3 µs. Before job 1.1 it was 26–59 µs, all of it allocation. Choosing what to simulate stays the larger lever.

## The law

- **Nature and the judge** are Mojang's 26.2 code and the server. **The law** is the generative structure we write to reproduce them (S46), at three grains: the tick recurrence, words and the medium ([representation.md](representation.md)). The engine is the Rust law (his choice, S11).
- **The judge is the game itself.** The bot's client runs Mojang's code with no game-logic mods: Lithium and C2ME are removed from it and from the server. The other performance mods stay, because they only render, cache or network.
- **Seal only what can't be corrected (SR-71).** The law may be loose where the server corrects for free (walking across a field; a cow treated as the cone of where it might be). It must be bit-exact, behind frozen gates, where nothing can correct it afterwards (narrow words: a gap at the jump limit, a clutch).

## How the law grows (S46)

The port covers everything the bot can interact with, and everything that can interact with the bot. Rendering, sound and textures are out. It grows from three sources:
1. **The ground layer.** The obvious mechanical laws (shapes for every block state, fluids, placement, breaking, item use, entity hitboxes and pushing, damage, effects, boats, elytra). GLM lanes write them, one rule each, read from the bytecode with a witness. An independent GLM checks each, and Terra integrates them in batches.
2. **Deviations.** Inside the server, every prediction of a server-owned outcome is compared with what the server did. Each mismatch becomes an anomaly with a minimal witness, then a law delta and a regression test.
3. **Demand.** A new word needs a law that isn't there yet.

Deviations only catch the law's false positives. A law that wrongly says "impossible" never shows up, because the planner never tries it. So the ground layer is checked against the jar directly, and R&D runs candidate words on a sandbox copy of the server.
- **Arithmetic copies Java exactly.** That means Java's float and double semantics (strict IEEE), `Mth.sin` and `Mth.cos` (a 65,536-entry table), the same order of operations, and never a fused multiply-add.

Exactness classes:

| Class | What | Judged against |
|---|---|---|
| bit-exact | player movement and collision: position, velocity, on-ground, flags, tick by tick | the bot client's own simulation (recorder) |
| server-exact | health, food, saturation, exhaustion, fall damage, breaking progress | the server (read over RCON) |
| behaviour-exact | fluids, falling blocks, mobs, redstone | replays, at the resolution the decisions need |
| advisory | floats on the GPU, learned estimates, forecasts | nothing: a wrong value may only cost speed, never change T |
| server-driven | state the server computes and the client only receives: speed modifiers by attribute packet (powder-snow frost, effects), `ticksFrozen` decay; entity data the server echoes to its own player (the sprint flag, pose, the swimming flag: `ServerEntity.sendDirtyEntityData`, applied without the client's setters, 9 November) | recorded per tick and read by the law as input, never simulated, because the packet's arrival tick is network timing. Found in the 26.2 jar on 5 November: freezing runs only in `ServerLevel` |

There is one known exception. Elytra flight (`LivingEntity.updateFallFlyingMovement`) calls `Math.cos` and `Math.sqrt`. `Math.cos` goes through HotSpot's own implementation, so the port must reproduce it bit for bit or give it a class of its own (Phase 11).

## What it keeps and what it drops

**Keeps:**
- the bot's motion and collisions;
- block shapes and properties;
- the effects of items that change motion or the world;
- fluids, entities and circuits inside the light cone.

**Drops:**
- rendering, sound and textures;
- networking, except the bridge to the bot;
- anything outside the light cone.

## How it computes: a causal chain

The rules come from ES's engine ("compute only surprises"), mapped onto Minecraft. A surprise is an event that breaks a certificate. Cost follows surprises, never time or space: anything that can be derived when it is looked at (a cone bound, a steady speed, an arc) is derived then, not kept up to date.

| ES | Here |
|---|---|
| certificate | an edge of the geometry or an entry in a portal table: "this crossing takes k ticks from these entry states, within width w", valid until a dependency changes |
| dependencies | the block footprint the claim read, plus the toolkit it assumed |
| event | a contact (landing, wall, ceiling), an edit (a block placed or broken, by anyone), a command (a new goal), a toolkit change |
| slack | the ticks until the bot's light cone reaches the certificate, minus the work to repair it. Least slack is renewed first |
| unit | one pure, bounded job (a local solve, a table refresh, a replay): same inputs, same bits; it can run anywhere, early, twice, or be thrown away |
| one rule at every scale | cells and regions expose one interface: border states, tables, next expiry, deadline |
| error types | bounded error is allowed; a random walk only with a correction that bounds it (re-anchoring on the observed state); secular drift is forbidden; chaotic divergence is physics, judged by statistics |
| correctness independent of allocation | the same plan and the same T whatever the thread count, GPU on or off, budget or schedule. Budgets are counted in work units, never wall-clock time, so every run replays |
| value of computation | shape only where it can change a decision; idle time computes ahead and recompresses |
| the simulation decides, heuristics schedule | a heuristic may reorder work, never remove a candidate. Pruning is deciding, so a candidate is removed only by proof: a bound from the light cone or the bracket, or dominance under the law. Learned estimates are advisory (above) |
| ruin is excluded, not priced | an irreversible loss (death, an item lost for good, a world change) is a barrier from the first rung, never a cost in ticks. Tokens (Phase 7) price only what can be spent and replaced |
| the player is the only wildcard | here: Julien (his commands and edits) and the bot's own execution noise. Their light cones bound where certificates can break. Mobs are precomputed as law (S20), in the behaviour-exact class; how their random choices enter is Phase 8's design |

## Local repair

"I don't want to have to recompute a whole region because one block changed" (X2):
- A block change notifies the certificates whose footprint contains that block. Each one is renewed or dropped.
- A table above it is refreshed only where one of its portal times actually moved.
- Improvements travel the same way. A new tool or a newly opened gap can lower a table entry away from the current route, so dependents of the toolkit are re-checked when it changes. "an old route can remain valid while becoming poor" is the case this covers (X2).

## Error bars with feedback

"it's also ok if we're not perfect … error bars in their head and update based on feedback" (X2):
- Each move is certified with a width: how far the entry state may be off and still reach the same future.
- The executor re-anchors on the observed state every tick.
- The planner re-solves locally only when the bot leaves its tube.
- Narrow moves are where precision is spent; wide ones are never refined.

## Continuous where it helps

Between events, motion is a closed-form recurrence: velocity is a geometric series per tick, with ratio friction × 0.91 on the ground. So the engine may jump from event to event. Every certificate it issues is still checked against the ticked law.

**Store the generator, not the generated** (ES 6). What is stored is the recurrence and its parameters (friction, drag, jump power, a ladder's rectangle, the modifiers), keyed by those parameters and shared by every place and bot that uses them. Arcs and trajectories are derived when looked at, never stored tick by tick. Differences below `aiStep`'s thresholds (9×10⁻⁶ on h², 0.003 vertically) vanish in the law itself, so states that differ only there merge.

## The contract

**Inputs:**
- the world: region files for static worlds, the block updates the bot's client streams, and later a read-only server observer;
- the bot's observed state each tick;
- the toolkit;
- goals.

**Outputs:**
- tick-stamped controls (move, strafe, jump, sneak, sprint, yaw, pitch, hotbar, use, attack), each with the certified tube it relies on;
- the predicted T.

**Guarantees:**
- deterministic: the same inputs give the same plan;
- anytime: a first plan within one tick of the goal arriving, refined while the bot moves;
- the exactness classes above;
- the planner never runs on the game's thread.

## The machine

**Hardware:** Windows 11; Ryzen 7 7800X3D (8 cores, 16 threads); 31 GB of RAM; Radeon RX 7900 XTX.

**Compute split:**
- The float law runs on the CPU threads.
- The GPU runs through Vulkan. It takes integer min-plus over ticks (exact and associative, so it is safe there) and bitboard light cones.
- Float work on the GPU is advisory.

**Toolchain:** Rust through rustup with the GNU toolchain. There is no C++ toolset or Windows SDK on this machine. The toolchain and build output live on D:, because C: has about 10 GB free.

## Facts verified so far

All of these come from 26.2. To regenerate them, run `javap -cp <client jar> -c -p -constants <class>`, where the client jar is `%LOCALAPPDATA%\ImmersiveMinecraft\data\libraries\com\mojang\minecraft\26.2\minecraft-26.2-client.jar`.

**`LivingEntity.travelInAir`:**
- The ground friction f is the block's friction when on the ground, and 1 in the air, adjusted by `FRICTION_MODIFIER` (`clamp(1 − (1 − friction) · m)`, with m = 1 by default).
- Acceleration is `speed · 0.21600002 / f³` only when f > 0.6; otherwise it is `speed`.
- Horizontal drag is f × 0.91, adjusted by `AIR_DRAG_MODIFIER`. Vertical velocity is updated as (vy − gravity) × 0.98.

**`jumpFromGround`:**
- vy = max(`JUMP_STRENGTH` × block jump factor + jump boost, vy).
- Sprinting adds 0.2 along the yaw, through the sine table.

**`Player.getFlyingSpeed`** (acceleration in the air): 0.02, or 0.025999999 when sprinting.

**`aiStep`:**
- Horizontal velocity is zeroed when its square is below 9 × 10⁻⁶.
- Each vertical component is zeroed when below 0.003.

**Attributes:**
- movement speed 0.1 (the player's base), × 1.3 when sprinting (`ADD_MULTIPLIED_TOTAL` 0.3);
- jump strength 0.42; gravity 0.08; step height 0.6;
- safe fall distance 3; block interaction range 4.5; sneaking speed 0.3.

**Fall damage** = floor(fall power × the block's landing multiplier × the fall-damage multiplier attribute). Hay supplies a 0.2 landing multiplier. A bed halves the fall distance before fall power is calculated.

**`FoodConstants`:**
- Sprinting needs food above 6.
- Exhaustion: sprinting 0.1 per metre, a jump 0.05, a sprint-jump 0.2, each healed health point 6. Four exhaustion cost one food point.
- Regeneration: one health point every 80 ticks at food ≥ 18, or every 10 ticks when saturated.

**Blocks whose movement data differs:**
- ice, packed ice and frosted ice: friction 0.98;
- blue ice: friction 0.989;
- slime: friction 0.8;
- soul sand: speed 0.4;
- honey: speed 0.4, jump 0.5.

Every other effect is a code hook. In `net.minecraft.world.level.block`, 25 classes override `entityInside`, 9 override `fallOn` and 7 override `stepOn`.

**Ender pearl:**
- thrown at 1.5 blocks per tick;
- gravity 0.03; drag 0.99 per tick (0.8 in water);
- 5 damage on landing, a 20-tick cooldown, and a random spread drawn from the server's RNG.

**Derived by [Speeds.java](../lab/numbers/Speeds.java)** (run it with `java lab/numbers/Speeds.java`). All figures are steady speeds in m/s.

Horizontal:
- walking 4.317;
- sprinting 5.612;
- sprint-jumping 7.127, or 9.734 under a 2-block ceiling;
- sprinting on ice 5.404, slower than on stone;
- sprint-jumping on ice 9.241, or 19.345 under a ceiling (blue ice: 19.903).

Vertical:
- the ladder 2.352;
- terminal fall speed 78.4.

Falls and pearls:
- A 23-block fall does 20 damage, enough to kill from full health.
- A pearl flies about 52 blocks in 59 ticks.
