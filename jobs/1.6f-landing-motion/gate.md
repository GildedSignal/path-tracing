# 1.6f live landing provenance

Frozen before the first live run. The integrator owns the live rig.

## Captures

Use two stopped disposable copies of the same reference lab. Record in normal difficulty, with natural regeneration on, mob spawning off, a fresh PathBot save per copy and the recorder's settled start. Run edges, full-blocks and ice once in each copy, with the fixture driver at seed base 1, 400 input ticks and 20 released-key ticks. For edges, check every applied input against the original 1.6e archive as well as the generated input. Full-blocks and ice must have no own-entity motion packets in the fenced interval. The original strict checker must still refuse the two archived edges recordings and the injected gate 1.2 teleport.

Set the server target to 20 in the first copy and use the vanilla client clock. Set it to 200 in the second and enable the lab client's server-following clock and full tick demand. Record both configured rates and the client replay wall time. The accelerated replay must execute above 100 client ticks per wall second. This measurement checks that the accelerated path ran. Packet provenance uses connection identity, order and phase.

## Pass line

- One complete fenced landing chain per edges capture. The accepted move's connection ordinal, ground state and position must identify a recorded airborne-to-ground transition. At the accepted fall check the base stone hook must invoke effective damage from the actual `minecraft:fall` source, and the same source instance must cause `hurtMarked` to go false to true.
- That mark must be the sole cause at the tracker consume. Refuse any additional damage, mark, competing velocity setter, unconsumed mark, competing correction or extra motion send/receive. Link the consume by packet instance to actual dispatch on the observed player's connection and its encoded packet. Fence that connection in wire order. Compare recipient UUID, own entity ID, ordinal and all three decoded raw bits to the client handler. Re-encode the raw consume vector offline using the staged 26.2 packet codec and compare the emitted bytes.
- Require the actual `lerpMotion` call between recorder END and the next START. Independently check the player tick count, handler order, raw anomaly row and pre/post velocity. The application row follows the landing but its distance from the landing is unconstrained. Reject any in-tick or otherwise ambiguous application. Only a verified input can replace the Rust state velocity at this boundary.
- Every replay row must be bit-exact for position, velocity, angles, fall distance, movement flags and every collision request and result. Compare damage input order and all recorded resource checkpoints on the separate 1.5h server timeline.
- Plant an extra receipt with the identical payload and a new ordinal without a send, shift the application row while preserving the player clock stamp, and swap mark and consume with renumbered server events. Each must refuse for its planted fault. Preserve the original raw anomalies, inputs, world, snapshots, client handler rows and server event and wire rows. The capture and verification logs retain the first failing case and its diagnostic.

## Integrator run

Integrate the client clock from `wt/clock` at `388a1bc`. The runtime pieces are `MinecraftClockMixin.java`, its entry in `bot/pathtracing.mixins.json`, `ClockAudit.java`, and the `FollowTickRate` / `FullTickDemand` launcher options. Preserve the new landing mixin entries and Recorder hooks when combining the branches. The capture does not require the separate clock gate or clock audit logging. The run script refuses an unintegrated launcher before touching either copy.

Prepare the disposable copies as for [1.5h](../1.5h-resource-order/gate.md), each with its own unused port pair and fresh PathBot save. Put both `resource-pair.disposable` and `landing-motion.disposable` at each copy's root. Keep the matching Fabric API in each server's mods. Set `PATHTRACING_DEPS` to the staged dependency root and close all other bot clients. Only the driver starts and stops the copied instances under their rig locks.

```powershell
$env:PATHTRACING_DEPS = 'D:/path-tracing-deps/data'
$archives = 'D:/path-tracing-wt/clock/lab/gates/results'
$a20 = '20261106T1257229131311Z'
$a200 = '20261106T1249512984107Z'
$edges20 = "$archives/$a20-fixtures/l0_${a20}_edges_1.jsonl"
$edges200 = "$archives/$a200-fixtures/l0_${a200}_edges_1.jsonl"
$teleport = 'D:/path-tracing-evidence/20261105T0838027621258Z-gate-1.2/g12_20261105T0838027621258Z_negative.jsonl'
& .\jobs\1.6f-landing-motion\run.ps1 -Rig20 'D:/path-tracing-evidence/1.6f/20' `
  -Rig200 'D:/path-tracing-evidence/1.6f/200' -Port20 25567 -Port200 25568 `
  -ReferenceServer 'D:/path-tracing-lab/server' -Edges20 $edges20 -Edges200 $edges200 -Teleport $teleport
```

The raw reports live under `lab/gates/results`. To recheck saved captures, pass their report directories as `-Capture20` and `-Capture200` to `verify.ps1` with the same archive and reference-server arguments. The verifier stops on the first failure. A refusal leaves the prior L0 gate open.
