# 1.9g mining progress differential

Freeze before the first differential run. Use the staged 26.2 server jar with SHA-256
`183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8`.
Verify the relevant Mojang method and offset windows using `javap -c -p`. Compile
the independent Java clock and execute its arithmetic on the JVM in interpreter mode.
The Java clock transcribes the pinned instructions for the traced paths and projects
them onto the public adapter contract. This is an offline differential.

| Mojang method | Bytecode offsets and behavior |
|---|---|
| `BlockBehaviour$BlockStateBase.getDestroyProgress` | 0–14 delegates through the block |
| `BlockBehaviour.getDestroyProgress` | 0–19 reads hardness and returns +0 for exactly -1. 20–49 chooses 30 or 100 and evaluates `speed / hardness / divisor` in two f32 divisions |
| `ServerPlayerGameMode.handleBlockBreakAction` START | 252–257 records `gameTicks`. 263–349 reads the state, runs hit hooks and evaluates current progress. 351–376 destroys on non-air and progress ≥ 1. 429–481 arms active work and sends the crack stage |
| Same method, STOP | 487–531 gates on position and non-air. 505–514, 548–557 use current progress × f32(wrapped elapsed + 1). 559–599 destroys at ≥ 0.7. 600–627 hands off only if the delayed slot is empty and copies the original START tick |
| `ServerPlayerGameMode.tick` | 0–14 increments the signed clock and selects delayed first. 17–78 clears delayed work on air or calls the feeder and destroys at ≥ 1. 81–153 checks active only when delayed was absent, clears on air and discards the feeder's completion value |
| `ServerPlayerGameMode.incrementDestroyProgress` | 0–30 reevaluates current progress and uses wrapped `gameTicks - startTick + 1` before the int-to-f32 conversion. 32–74 handles crack stages and returns the fraction |

Compare every emitted row against the Rust entry point, including the selected world
lookup, post-event game tick and both live slots with original start ticks, evaluated
fraction and intent with expected block identity. Compare raw f32 bits, with NaNs
compared as one class because payloads are outside the Java float contract.

The fixed fixture generator covers one representable float below, at and above each
of the three decision boundaries and checks that those exact fractions were reached.
It covers f32 division order, hardness -1 and zero, both divisors, signed zero,
subnormal and nonfinite arithmetic, reevaluation with changing inputs, an active
fraction above 1 without STOP, clock rollover, int-to-f32 rounding near 2²⁴ and
elapsed + 1 overflow from a synthetic clock snapshot. It also covers a new active
START while older delayed work remains, STOP on the new active with a busy delayed
slot both below and at the threshold, and clearing or completing delayed work
without sampling active work in the same tick. Adapter controls exercise rejection,
out-of-scope action refusal, air and replacement. Synthetic clock changes exist only
in the verifier.

Pass requires zero differences on every row and a difference on exact equality
for the disposable Rust mutant that changes only delayed completion from ≥ 1 to
> 1. Use the same JVM rows and Rust verifier for both binaries. Run all rows,
retain the raw first mismatch and report fixture and mismatch counts by entry point.

Accepted START takes an eligible non-air state after evaluation of the hit hooks.
START requires an empty active slot and a different position from any delayed work.
STOP requires a live active slot and its position. Duplicate or switched START,
STOP against retained inactive positions, and ABORT are outside the gate. Air at
STOP leaves the clock alone, as in the jar. Air on a selected tick clears that slot.
A replacement identity invalidates the selected slot and explicitly refuses the
transition. This adapter rule bounds the vanilla comparison to a stable target.
Missing observations mean air, so the caller must supply an authoritative sample
or abort the surrounding simulation before invoking this law. Reach, eligibility,
hit hooks, crack-stage packets, acknowledgments and the result of the destroy
attempt belong outside the clock. A destroy intent must be checked against its
identity and applied by the adapter at the same event before proceeding.
