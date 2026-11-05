# Job 1.5b: the judge's findings on the Rust law

You built job 1.5 on `wt/law` (`ef3d04b`). A judge from another model family checked it against the bytecode and found it ready for L0 with fixes. Make exactly these changes. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits.

Cargo: `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`, target `D:/path-tracing-target`. The jar and `javap` are under `D:\path-tracing-deps\data` (as in job 1.5).

## Corrections to the 1.5 spec

**The spec was wrong about sprint, not you.** It said to treat sprinting as on "exactly while the sprint input is held". The bytecode latches it. Port `LocalPlayer.aiStep` as it is (offsets about 379 to 484):
- sprint turns on only when `canStartSprinting()` holds and the sprint key is down;
- it turns off only through `shouldStopRunSprinting` (`!isSprintingPossible(flying) || !hasForwardImpulse || (horizontalCollision && !minorHorizontalCollision)`) or `shouldStopSwimSprinting`;
- `isSprintingPossible` includes `flying || !isInShallowWater()`, where `isInShallowWater` is `isInWater && !isUnderWater`;
- so releasing the sprint key while holding forward keeps sprinting, and sneaking doesn't clear it.

The sprint latch is state: keep it in `PlayerState`, take its value from the recording's `isSprinting` at tick zero, and compare it every tick.

## Fixes

1. **Sprint**, as above. Add tests: wading in shallow water while holding sprint and forward (not sprinting); sprinting then sneaking (still sprinting); sprinting then releasing the sprint key while holding forward (still sprinting).
2. **Two epsilons.** `math.rs` `equal` and `shapes.rs` `normalize` use `1e-5` as a double. The jar widens the float `1.0E-5F`, giving `9.999999747378752E-6`. Use that constant in both places, and look for any other float constant written as a double literal.
3. **Bed landing.** `BedBlock.fallOn` halves the fall distance before `calculateFallPower` subtracts the safe distance. That is not a post-multiplier. Model it as the jar does: d = 10 gives damage 2. Keep hay and honey as multipliers, and add a test for each.
4. **The push-out.** Port `LocalPlayer.aiStep` offsets about 209 to 340 (`moveTowardsClosestSpace`, four corners, when `!noPhysics`). If that needs world queries the `World` trait lacks, add them.
5. **Silent gaps become explicit.**
   - The inside-block pass: port the `visitedBlocks` addition and `FluidState.entityInside` from `Entity.lambda$checkInsideBlocks$0` (offsets about 123 to 167), or make them an `unimplemented!` with the reason.
   - Any block with an `entityInside` hook you don't model (bubble column, sweet berry bush and the rest) must reach an `unimplemented!`, never a silent no-op.
   - `fall_reset_ray`: port `Level.clip` with its voxel traversal, and `ClipContext.Block.FALLDAMAGE_RESETTING`'s end gateway and end portal rule, or say exactly where it may differ.
6. **Tests that check the jar, not the port.** `braking_recurrence` comes from `Braking.java` and `braking_player_tick` from the port's own output, so neither is evidence. Keep them as regression tests and label them so. Where you can, generate their expectations with `JarOracle.java` from the real classes, as the collision tests do.
7. **Replay coverage.** The replay harness also compares `minorHorizontalCollision`, `verticalCollisionBelow`, `speed`, `swimming` and `crouching` where the recorder supplies them, and the sprint latch.

## Done means

- `cargo test` passes in `engine/` (replay still ignored without fixtures); `cargo build --features mutant-<each>` passes for all four.
- The final message lists every change, every new `unimplemented!`, and every assumption.
