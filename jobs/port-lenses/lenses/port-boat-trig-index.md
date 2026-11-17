# Port delta: boat.rs trig index signature (Mth.sin/cos (D)F vs (F)F)

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Anchor to `engine/law/src/boat.rs:5-11, 44-45` and `jobs/port-backlog/report.md` #46 (`port-boat-rider-attachment`) & #56 (`port-boat-turn-trace`).

**Consumer:** `engine/law/src/boat.rs` (`seat` and `tick`) and batch `P4a` (`jobs/port-backlog/report.md`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `Vec3.yRot(float)` (called by `AbstractBoat.getPassengerAttachmentPoint`, to locate) and `AbstractBoat.controlBoat` (to locate): inspect the exact `invokestatic` descriptors of `Mth.sin` and `Mth.cos` — whether each site calls `Mth.sin(F)F`/`Mth.cos(F)F` (float-index `(int)(value * 10430.378f)`) OR `Mth.sin(D)F`/`Mth.cos(D)F` (double-index `(int)(value * 10430.378350470453)`), and check `Entity.positionRider` / `AbstractBoat.positionRider` as well;
- verify whether `boat.rs:6-8` (`seat`) and `boat.rs:44-45` (`tick`) match those exact signatures on the `49/11520` yaws where `(F)F` and `(D)F` diverge.

**Build a jar-backed reproduction:** run `reproduce.sh` dumping `javap -c` for `Vec3.yRot`, `AbstractBoat.getPassengerAttachmentPoint`, and `AbstractBoat.controlBoat`, and running a jar-backed witness over yaw angles where `Mth.sin(F)F` differs from `Mth.sin(D)F`, comparing raw bits against `boat.rs::seat` and `boat.rs::tick`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `Mth.sin`/`cos` descriptor `(F)F` vs `(D)F` or raw bit mismatch on the 49 divergent yaws against `boat.rs`.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
