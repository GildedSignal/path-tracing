# Movement law checkpoint

The Rust workspace is scaffolded; the movement port and replay harness have not started. No L0 result is claimed. The development dependencies have been removed as instructed; the crate now has no dependencies. Nothing has been committed.

## Decision needed: client input semantics

The recorder applies raw keys through `KeyMapping.setDown`, but the 26.2 client has additional input behaviour outside the methods listed for job 1.5:

- `LocalPlayer.move` calls `updateAutoJump`. It can set `autoJumpTime = 1`; the next `LocalPlayer.aiStep` calls `ClientInput.makeJump`, even with the jump key released.
- `LocalPlayer.aiStep` reads `Options.sprintWindow()` to implement double-tap sprinting. It can call `setSprinting(true)` without the sprint key being held.
- `scripts/setup.ps1` copies the pack's options into the bot's client and does not override either setting. Job 1.2's recording format does not capture these settings.

Should L0 freeze auto-jump off and the double-tap sprint window to zero, or should job 1.5 include these option-dependent `LocalPlayer` behaviours, with their configuration supplied in fixture metadata? Neither policy is assumed here. The job explicitly requires stopping on a scope ambiguity instead of choosing.

Inspect the source with the staged tools, from the repository root:

```powershell
& D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe -cp D:/path-tracing-deps/data/libraries/com/mojang/minecraft/26.2/minecraft-26.2-client.jar -c -p -constants net.minecraft.client.player.LocalPlayer
```

The disassemblies from the earlier attempt remain ignored under `.bytecode/`. `probes/ReadMethods.ps1` selects complete methods from them, for example:

```powershell
& engine/probes/ReadMethods.ps1 client.player.LocalPlayer @('aiStep', 'move', 'updateAutoJump')
```

## Braking recurrence probe

Run from the repository root:

```powershell
& D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/java.exe engine/probes/Braking.java
```

This is a cardinal, yaw-zero ground recurrence, not a full player tick, server measurement, or L0 gate. It reaches steady velocity over 2,000 forward ticks, then measures 400 ticks per protocol. Releasing forward or pressing back stops sprinting, as in `LocalPlayer.shouldStopRunSprinting`; reverse acceleration is walking acceleration. The ground, food level and friction stay fixed. The probe measures the first subsequent `aiStep` threshold event and the first such event after reverse input ends; these coincide in all 28 measured cases. Tick 1 is the first tick after forward input ends.

Ticks until the horizontal velocity is zeroed by the threshold:

| Ground / initial gait | Release | Reverse 1 | Reverse 2 | Reverse 3 | Reverse 4 | Reverse 5 | Reverse 6 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Stone / walk | 8 | 5 | 8 | 10 | 11 | 12 | 13 |
| Stone / sprint | 8 | 6 | 8 | 10 | 11 | 12 | 13 |
| Ice / walk | 38 | 36 | 35 | 33 | 30 | 25 | 7 |
| Ice / sprint | 40 | 39 | 38 | 36 | 35 | 32 | 27 |

Each reverse protocol means reverse for k ticks, then release. The fastest measured protocols are reverse once on stone (5 ticks after walking, 6 after sprinting) and reverse six times on ice (7 ticks after walking, 27 after sprinting). These are minima over the requested k = 0..6 only. Reversing for at least two ticks on stone overshoots on tick 2 in both gaits; none of the measured ice protocols overshoots. The probe emits every individual result as CSV, including negative-velocity and threshold timing.

## Remaining work

All requested movement, collision, fluids, block hooks, poses, exhaustion and damage functions remain to be ported. No Rust unit tests, replay harness, fixture format, or mutant behaviour exists yet. The feature names exist only in the manifest. There are no `unimplemented!` branches yet because the Rust library is empty. Cargo test and the four mutant builds have not been run for this checkpoint; an empty-library pass would not validate the requested work.
