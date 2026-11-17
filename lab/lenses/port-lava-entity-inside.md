# Port delta: LavaFluid.entityInside freeze clear, ignite and deferred damage

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Anchor to `engine/law/src/movement.rs:1051-1056` (`Inside::Lava => refuse!("FluidState.entityInside: LavaFluid CLEAR_FREEZE, LAVA_IGNITE and deferred lava damage")`).

**Consumer:** `engine/law/src/movement.rs:1054` (refused `FluidState.entityInside` for `FluidKind::Lava`), `jobs/event-forms/report.md` §4.7, and `P3a`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LavaFluid.entityInside` (and `InsideBlockEffectType.CLEAR_FREEZE`, `LAVA_IGNITE`, and `Entity.lavaHurt`, to locate):
- exact order and effects when a player overlaps lava during `Entity.checkInsideBlocks`: `ticksFrozen` reset (`CLEAR_FREEZE` -> `0`), `remainingFireTicks` (`LAVA_IGNITE` -> `igniteForSeconds(15.0f)` = `300` ticks? check fire resistance / `fireImmune`), and when `lavaHurt` applies `4.0f` damage (`invulnerableTime` cooldown interaction, server-only vs client).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LavaFluid.entityInside`, `InsideBlockEffectType`, and `Entity.lavaHurt`, plus a jar-backed witness printing `(ticksFrozen, remainingFireTicks, health, invulnerableTime)` before and after 1 tick of lava contact from `ticksFrozen = 50, remainingFireTicks = 0, health = 20.0f`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `ticksFrozen`, `remainingFireTicks`, or `lavaHurt` damage/cooldown value that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
