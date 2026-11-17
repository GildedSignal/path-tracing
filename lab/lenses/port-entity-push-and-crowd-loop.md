# Port delta: LivingEntity.pushEntities loop and solid entity colliders (boat/shulker)

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-push-threshold` and `port-ordered-push-query` already settled `Entity.push(Entity)` and `EntitySectionStorage` iteration order; do not re-ask those.

**Consumer:** `engine/law/src/entity_contact.rs`, `engine/law/src/movement.rs`, batch `P4d`, `docs/HANDOFF.md:41` Anomaly 5.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.pushEntities` (called at `aiStep @817`, to locate): exact search box (`getBoundingBox()` vs inflated?), `EntitySelector.pushableBy(this)` predicate, `MAX_ENTITY_CRAMMING` (`>= 24` cramming damage `6.0f`!), and exact `doPush` order relative to `travel` (`@617`) and `aiStep` (`@817`);
- `AbstractBoat.push(Entity)` and `AbstractBoat.canCollideWith` / `Shulker.canBeCollidedWith` (to locate): vertical overlap gate (`other.getBoundingBox().minY <= this.getBoundingBox().minY` in `AbstractBoat.push`?), and which entities contribute solid `VoxelShape` boxes in `Level.getEntityCollisions` (`Entity.collide` `@15`, to locate).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LivingEntity.pushEntities`, `AbstractBoat.push`, `AbstractBoat.canCollideWith`, and `Level.getEntityCollisions`, plus a jar-backed call verifying `AbstractBoat.push` when a player stands on top of a boat vs beside it, and listing every `canBeCollidedWith` / `canCollideWith` overrider in the server jar. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `pushEntities` search box, cramming threshold, `AbstractBoat.push` Y-gate, or `canBeCollidedWith` class list that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
