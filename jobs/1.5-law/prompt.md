# Job 1.5: the movement law in Rust

You are the builder for a well-formed task. Design choices are not yours. If the spec is ambiguous or wrong, stop and write the question in your final message instead of guessing.

## Context (read these first)

- `AGENTS.md`, `docs/engine.md` (the law, the exactness classes, the verified facts) and `docs/ladder.md` (rung L0).
- `lab/numbers/Speeds.java`: a small Java model of the ground and air law, with known steady speeds. It is a sanity check, not the source.
- **The source of truth is the 26.2 client jar.** It is unobfuscated, so read the bytecode with `javap -cp <jar> -c -p -constants <class>`.
  - The jar is `%LOCALAPPDATA%\ImmersiveMinecraft\data\libraries\com\mojang\minecraft\26.2\minecraft-26.2-client.jar`.
  - `javap` is in `%LOCALAPPDATA%\ImmersiveMinecraft\data\java\java-runtime-epsilon\bin`.
- **Reading guide only:** Azalea's `azalea-physics` (https://github.com/azalea-rs/azalea, MIT) mirrors vanilla's shape, merger and sweep structure. It is *not* exact: it lacks the friction and drag modifiers, it applies `0.216/f³` on every block (the jar applies it only when friction > 0.6), and it leaves speed factors, stuck factors, exhaustion and fall damage as TODOs. If you copy any of its code, keep its MIT notice in that file and verify every line against the jar.

## Answer to the first run's question

The sandbox cannot read `%LOCALAPPDATA%`, so use the staged copy, which has the same layout:
- the jar: `D:\path-tracing-deps\data\libraries\com\mojang\minecraft\26.2\minecraft-26.2-client.jar`;
- `javap`: `D:\path-tracing-deps\data\java\java-runtime-epsilon\bin\javap.exe`.

`RUSTUP_HOME` and `CARGO_HOME` are set to `D:\rust\rustup` and `D:\rust\cargo`, and `D:\path-tracing-target` is writable for the cargo target dir. If anything is still denied, say exactly which path.

## What to build

A Cargo workspace at `engine/`, holding one library crate, `engine/law`.

### Setup

- Stable Rust, GNU toolchain (already installed, under `D:\rust`).
- No nightly features.
- No dependencies, except `serde` and `serde_json` as dev-dependencies for the replay tests.
- `engine/.cargo/config.toml` sets `[build] target-dir = "D:/path-tracing-target"`, because C: is nearly full.

### What it simulates

One player tick, ported from the bytecode in the same order of operations.

| Class | Methods |
|---|---|
| `LivingEntity` | `aiStep` (velocity thresholds, input scaling, jumping and `noJumpDelay`), `travel`, `travelInAir`, `handleRelativeFrictionAndCalculateMovement`, `getFrictionInfluencedSpeed`, `handleOnClimbable`, `jumpFromGround`, `getJumpPower`, fall damage (`calculateFallPower`, `calculateFallDamage`, `checkFallDamage`), `travelInFluid` (water and lava) |
| `Entity` | `move`, `collide`, step-up, `collideBoundingBox`, the axis order `Direction.axisStepOrder`, `getBlockSpeedFactor`, `getBlockJumpFactor`, `makeStuckInBlock` and how the multiplier is applied, fall distance bookkeeping |
| `Player` | `maybeBackOffFromEdge`, `getFlyingSpeed`, `updatePlayerPose` (standing, crouching, swimming), the exhaustion from movement and jumps, the sprint food threshold |
| `Mth` | the `sin`/`cos` table: 65,536 floats, filled exactly as `Mth` fills it |
| `Shapes` and `VoxelShape` | the parts collision needs |

**Not in scope for this job:** elytra, boats, riding, effects other than none, and `Math.cos`. Leave a clear `unimplemented!` where the code would branch into them.

### Exact arithmetic

- Every float and double operation follows the bytecode: `f2d` where the bytecode widens, float constants kept as `f32`.
- Never `mul_add`, never a fast-math flag.
- Java's `(int)` and `(long)` casts saturate, and NaN becomes 0; Rust `as` does the same. Check `Mth.floor` and friends against the jar.

### Interface

- **World:** a `World` trait answering what the law asks of a level.
  - For each block position: the block state's collision boxes in a given collision context; friction, speed factor and jump factor; whether it is climbable; its fluid; the hooks that apply (stuck multipliers, `fallOn` multipliers).
  - Provide a `BoxWorld` for tests: blocks as lists of boxes with properties.
  - Later the dump from job 1.2 (`blocks.jsonl`) will feed it. Don't parse the dump in this job.
- **Player state:** a `PlayerState` struct holding every field the recorder logs (see `jobs/1.2-recorder/prompt.md`, section A.3), plus whatever hidden state the law needs (`noJumpDelay`, fall distance, exhaustion…).
- **Input:** an `Input` struct with the recorder's fields: forward, back, left, right, jump, sneak, sprint, yaw, pitch.
- **The tick:** `fn tick(state: &mut PlayerState, world: &impl World, input: &Input)`.

### Tests

1. **Unit tests** reproduce `Speeds.java`'s steady speeds on flat ground, to the printed precision:
   - walk 4.317, sprint 5.612, sprint-jump 7.127 m/s;
   - sprint-jump under a 2-block ceiling 9.734;
   - ice 5.404 / 9.241 / 19.345, blue ice 19.903;
   - ladder 2.352;
   - terminal fall 78.4 m/s;
   - a 23-block fall doing 20 damage.

   Add a braking test: forward input, then the opposite key. Report the ticks until the velocity is zeroed by the `aiStep` threshold.
2. **The replay harness:** `engine/law/tests/replay.rs` reads recordings in the recorder's format (section A.3 of job 1.2: inputs and raw bits per tick) together with a fixture world description, replays them through `tick`, and compares positions and velocities bit for bit.
   - Its fixtures folder is `lab/fixtures/`, which may still be empty. When it has no recordings, the test must say so and be ignored, never pass silently.
3. **Mutants for the gate's negative control.** Add four cargo features, each changing one rarely exercised behaviour:
   - `mutant-axis-tie` (reverse the tie order in `axisStepOrder`);
   - `mutant-step-up` (step height off by one ulp);
   - `mutant-edge-backoff` (the sneak back-off step 0.05 becomes 0.06);
   - `mutant-ceiling` (don't zero vy on an upward collision).

   With any one feature on, the build must still compile.

## Constraints

- Touch only `engine/` (and `lab/fixtures/` only to add a README describing the fixture format). Don't touch `bot/`, `scripts/` or `docs/`.
- Comments: say where each function comes from (`LivingEntity.travelInAir`), and nothing more.
- Commit as you go on this branch, with the trailer `Built-by: gpt-6.1-sol`.

## Done means

- `cargo test` passes in `engine/` (with the replay test ignored until fixtures exist).
- `cargo build --features mutant-<each>` passes for each mutant.
- The final message lists:
  - what is ported, and every place you were unsure the port matches the bytecode;
  - the braking result;
  - anything left `unimplemented!`.

## The gate (rung L0 in `docs/ladder.md`, frozen now; run by a fresh judge after the integrator records the fixtures)

- Bit-identical positions and velocities on every tick, against the recorder, over many random input sequences on every terrain fixture.
- Damage and exhaustion equal the server's (from `scripts/server-state.ps1`).
- Zero server corrections in the recordings.
- Each of the four mutants is caught by at least one fixture.
- Ticks per second are recorded.
