# Port delta: boat mount, positionRider yaw clamp and dismount location

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-boat-rider-attachment` already settled `AbstractBoat.getPassengerAttachmentPoint`; do not re-ask the seat offset.

**Consumer:** `docs/plan.md` step 8 (carried movement: boat), batch `P4a`, and `engine/law/src/movement.rs:124-129`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- mounting (`AbstractBoat.interact` -> `Player.startRiding`, to locate): reach/slot checks (`canAddPassenger`, `maxPassengers`, seat 0 vs seat 1) and initial passenger state writes (`setPos`, `yRot`, `setSprinting(false)`?);
- `AbstractBoat.positionRider` / `clampRotation` (to locate): exact yaw clamp (`±105.0f` relative to hull `yRot`, `Mth.wrapDegrees`, `yBodyRot`, to locate);
- dismounting (`LivingEntity.stopRiding` -> `AbstractBoat.getDismountLocationForPassenger` / `DismountHelper`, to locate): exact candidate exit-offset search order and safe-floor / hitbox-fit checks when pressing sneak (`shift_key_down`).

**Build a jar-backed reproduction:** run `reproduce.sh` over the mount/clamp/dismount bytecode and invoke `clampRotation` and `getDismountLocationForPassenger` directly from the jar for (1) yaw clamp at `+120°` and `-120°` relative to hull, and (2) dismount on open ground vs blocked left side. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any yaw-clamp bound, wrapping rule, or dismount exit coordinate that disagrees with direct jar execution.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
