# Port delta: Jump Boost, Speed and Slowness kinematics

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Anchor to `engine/law/src/movement.rs:121-123` (`state.effects` refusal) and `jump_from_ground`.

**Consumer:** Julien's track `frosted_ice` ("powerful jump boosts", `jobs/flight-corpus/report.md` §4.1.6) and `docs/plan.md` step 5/8.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.getJumpBoostPower` (or the 26.2 method adding `MobEffects.JUMP_BOOST` in `LivingEntity.jumpFromGround`, to locate): exact float/double formula for amplifier `0, 1, 2, ...` (`0.1f * (amplifier + 1)` or exact 26.2 bytecode), where it adds into `jumpPower`, and whether `JUMP_BOOST` also modifies `calculateFallPower` / safe fall distance (`LivingEntity.calculateFallPower`, to locate);
- `MobEffects` registration (to locate) for `SPEED` and `SLOWNESS`: exact `AttributeModifier` operation (`ADD_MULTIPLIED_TOTAL` etc.) and per-level amount (`0.2f`, `-0.15f`, or exact 26.2 constants) wired into `MOVEMENT_SPEED`.

**Build a jar-backed reproduction:** run `reproduce.sh` executing the jar's `MobEffects` attribute values and jump/fall-power arithmetic for Jump Boost I/II and Speed I/II / Slowness I, printing raw float/double bits. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any amplifier formula, attribute operation, or fall-distance offset that disagrees with the 26.2 jar bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
