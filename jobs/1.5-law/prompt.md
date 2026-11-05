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

## Answers to the second run's questions

- **Braking:** the test measures; it doesn't pass or fail on a single protocol. Report the ticks until the `aiStep` threshold zeroes the velocity after steady walking and after steady sprinting, on stone and on ice, for "release only" and for "reverse for k ticks, then release", with k from 1 to 6. Name the fastest protocol. Your probe (hold reverse goes negative on tick 2; reverse once and release stops on tick 5; release only stops on tick 8) is the right shape.
- **No network in the sandbox:** use no crates at all, dev-dependencies included. Parse the recorder's JSON lines by hand. The format is fixed and flat (job 1.2, section A.3).
- **Don't commit:** the sandbox can't write the repo's git metadata. Leave every change in the worktree; the integrator commits it with `Built-by: gpt-6.1-sol`.

## Answers to the third run's question, and how to proceed

- **Option-dependent `LocalPlayer` behaviour is out of scope.** The bot's client runs with `autoJump:false`, `toggleSprint:false`, `toggleCrouch:false` and `sprintWindow:0` (set by `scripts/setup.ps1`), so every movement in a recording comes from an explicit input. Port the law as it runs with those options; treat sprinting as on exactly while the sprint input is held, as `LocalPlayer` does with them.
- **Your braking table is accepted.** Keep it in `engine/README.md` and as the braking test.
- **Don't stop again for scope.** Take the most literal reading of this spec, write each assumption down in your final message, and keep porting. Stop only for something that blocks the build.

## The fourth run stopped

The previous run's process died at 23:08 on 4 November, in the middle of a `javap` read, with no error. Its work is still in the worktree: `engine/law/src/` (`lib.rs`, `math.rs`, `movement.rs`, `shapes.rs`, `state.rs`, `world.rs`, `sin.bin`, `build.rs`), `engine/law/tests/`, `engine/probes/`. **Continue from it.** Read what is there, finish the port and the tests, and don't start over. Delete `engine/law/src/write-probe.tmp`. The cargo target dir is `D:/path-tracing-target` (set in `engine/.cargo/config.toml`), so `engine/target/` should not be needed: say whether something still writes there.

## What to build

A Cargo workspace at `engine/`, holding one library crate, `engine/law`.

### Setup

- Stable Rust, GNU toolchain (already installed, under `D:\rust`).
- No nightly features.
- No dependencies at all.
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
- Do not commit (see the answers above).

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
