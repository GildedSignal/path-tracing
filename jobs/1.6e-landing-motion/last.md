# Job 1.6e — Damaging landing motion

Both original edges recordings still fail integrity at tick 174. The client-side effect is accounted for bit for bit. The server-side origin remains unverified in both recordings. Keep L0 open and retain the originals.

## 26.2 law

- On the accepted `ServerGamePacketListenerImpl.handleMovePlayer` path, the server installs the packet position and ground state, then calls `doCheckFallDamage`. `Entity.checkFallDamage` accumulates descending displacement cast to float and invokes the supporting block's `fallOn` on landing. The exported catch floor at this landing is stone. Its base `Block.fallOn` passes `DamageSources.fall()` and multiplier 1 to `LivingEntity.causeFallDamage`. The recorded descent and floor clip predict one damage under `calculateFallDamage`. This identifies the vanilla source for that landing.
- `LivingEntity.hurtServer` broadcasts the damage event on the fresh hurt path, then calls `markHurt` if the source and hit permit it. Vanilla `minecraft:fall` is in `no_knockback` and absent from `no_impact`. Fall damage can therefore raise `hurtMarked` without adding knockback. `ServerEntity.sendChanges` consumes that flag and sends a motion packet to trackers and self with the entity's velocity **at send time**. There are other motion packet emitters. The packet contains an entity ID and a vector encoded by `Vec3.LP_STREAM_CODEC`. It carries no cause or originating tick.
- `ClientPacketListener.handleSetEntityMotion` dispatches on the client main thread, looks up the packet entity and calls `lerpMotion`. The local player inherits `Entity.lerpMotion`, which immediately replaces delta movement for a finite vector. The damage event and `handleSetHealth` are separate handlers. The latter applies the server's health value through `LocalPlayer.hurtTo`.

The fall path explains why vanilla can emit this packet. The stored vector and health observation leave the actual emitter and any intervening impulse unobserved. Even an exact payload match against a possible send state admits another emitter sending those same bits.

## Saved sequence

Both rates have these exact client END observations. The decimals round-trip to the raw bits in the files.

| Client row | Position (x, y, z) | Velocity (x, y, z) | Health |
|---|---|---|---|
| 172 | (5647.423844295794, -59.78410565446128, 4102.871442849924) | (0.04838569107015112, -0.7787179775404599, 0.05356056445871318) | 20 |
| 173 | (5647.492217739546, -60.0, 4102.925703149763) | (0.06221983560726432, -0.0784000015258789, 0.04937687427568898) | 20 |
| 174 | (5647.5921565066765, -60.0, 4102.92920182679) | (0.054566573191599486, -0.0784000015258789, 0.001910277878728182) | 19 |
| 175 | (5647.718401580361, -60.0, 4102.864282523734) | (0.06892981823809537, -0.0784000015258789, -0.035445943585646546) | 19 |

The landing is in row 173. On row 174 the collision request's Y has the exact packet bits and the resolved Y is positive zero. The packet's X and Z are zero at both rates. It resets horizontal momentum before the next movement, while the floor clips either vertical request. Applying the full decoded vector as a pre-physics replacement on row 174 gives 420 of 420 bit-exact movement ticks in each diagnostic. Omitting it gives x = 5647.654376342284 on row 174. A second application on row 175 would give x = 5647.663835007169. The Rust estimate computes landing damage in row 173 and the client observes the server health update in row 174.

| Rate | Packet and requested Y | Compatible input to vanilla vector quantization |
|---|---|---|
| 20 TPS | -0.0783739241897089 | -0.0784000015258789, the client END Y in row 173 |
| 200 TPS | -0.58243300982726 | -0.5824428773508716, the client END Y in row 169 |

The quantization check shows how different server send states could yield the recorded vectors. Neither candidate is a server observation. The collision and replay localize the effect to the boundary after row 173 movement and before row 174 movement. The original END bucket gives no handler-side phase stamp.

## Witness needed on the isolated rig

The minimum causal observation is an independently witnessed fall-triggered mark linked to **this outbound packet instance**, joined to **this client application**. Use the opt-in server-phase probe's arm/seal pattern on disposable copies. Keep the existing resource trace and its checks separate. Instrument these boundaries before the new live capture.

1. On the server, observe the accepted landing move and its `fallOn` → `hurtServer` → `markHurt` path for PathBot. Record the source registry key and the false-to-true transition, with event order and server world/player ticks. Link the accepted move to the client row by its connection-scoped outgoing move ordinal. A damage amount, a nearby damage event, or a pre-existing mark supplies no provenance.
2. Carry that provenance to the actual `hurtMarked` consume/send branch in `ServerEntity.sendChanges`. Track competing marks and non-landing impulses up to send so a shared pending mark or unrelated velocity change remains ambiguous. Record the same packet's pre-encode vector and its ordinal in the bot connection's complete stream of own-entity motion packets, including unknown emitters. Use ordered start/end fences on that connection to align the streams. Missing or ambiguous evidence fails. The staged 26.2 codec must independently reproduce the client's decoded vector bits from the observed send vector.
3. At the client `lerpMotion` call inside `handleSetEntityMotion`, capture the matching receive ordinal, entity ID, pre/post velocity bits and the movement phase using recorder START/END boundaries and player tick count. Capture the health handler's phase separately if comparing client health by tick. Validate the full ordered stream one to one, then replay the replacement at its observed phase and compare the subsequent movement. A genuine delay can move the application to a different row, so derive timing from the handler and replay rather than hard-coding a landing-to-packet tick distance. Refuse any phase the replay cannot represent.

Record the same stock normal-difficulty edges sequence at both rates. Retain the packet in the raw recording even when its cause is later verified. An injected teleport must still fail. Plant a second own-entity motion packet with the exact payload after the witnessed one. Its new ordinal must fail even with a verified original. For an offline shifted-row witness, preserve the handler's phase stamp and move the row so the timing check must fail. No changes to the old control, L0 pass line or fixture publication are needed to gather this evidence.

## Separate server resources

The settled captures started with different saturation and exhaustion. At 20 TPS the coupled endpoint diagnostic matches all five RCON fields at offset 6. At 200 TPS the sample has offset bracket [57,58], so the strict endpoint comparison is unavailable. The RCON ends have saturation 11 vs 13 and exhaustion bits `3f055ed9` vs `3fa915d8`, with health 20, food 20 and timer zero in both. These observations supply no motion provenance and no resource snapshot substitutes for a replay output.

## Reproduce offline

From the worktree root, with the staged Rust toolchain on PATH.

```powershell
$results = 'D:/path-tracing-wt/clock/lab/gates/results'
$capture20 = '20261106T1257229131311Z'
$capture200 = '20261106T1249512984107Z'
$env:L0_EDGES_20 = "$results/$capture20-fixtures/l0_${capture20}_edges_1.jsonl"
$env:L0_EDGES_200 = "$results/$capture200-fixtures/l0_${capture200}_edges_1.jsonl"
$teleport = 'D:/path-tracing-evidence/20261105T0838027621258Z-gate-1.2/g12_20261105T0838027621258Z_negative.jsonl'
$job = './jobs/1.6e-landing-motion'
& "$job/check-refusals.ps1" -Edges20 $env:L0_EDGES_20 -Edges200 $env:L0_EDGES_200 -Teleport $teleport

$env:CARGO_HOME = 'D:/rust/cargo'
$env:RUSTUP_HOME = 'D:/rust/rustup'
$env:CARGO_MANIFEST_DIR = (Resolve-Path engine/law).Path
$target = 'D:/path-tracing-target/1.6e'
cargo build --offline --release --lib --manifest-path engine/Cargo.toml --target-dir $target
rustc --edition 2024 --test --extern "law=$target/release/liblaw.rlib" "$job/audit.rs" -o "$target/audit.exe"
& "$target/audit.exe" --exact edges_motion_effect --nocapture
```

For the law trace, run `javap -c -p` on the methods above and `net.minecraft.network.LpVec3` in the staged 26.2 jars. The damage type and tags are in `data/minecraft/damage_type/fall.json` and `data/minecraft/tags/damage_type/no_{impact,knockback}.json` inside the server jar.
