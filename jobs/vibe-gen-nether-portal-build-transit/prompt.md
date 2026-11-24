# Vibe live verification: Nether portal ignition, PortalProcessor clock, PortalForcer Y clamp, and 8:1 transit

**Goal:** Execute 26.2 `PortalShape.findEmptyPortalShape`, `FlintAndSteelItem`/`FireChargeItem` ignition (`4x5`..`23x23`, cornerless & invalid frames), `PortalProcessor` (`80`-tick survival / `0`-tick creative / `-4/tick` decay), `PortalForcer.createPortal` fallback Y clamp `min(maxY, minY + logicalHeight - 1) - 9`, and `8:1` coordinate/velocity/yaw axis-swap transit live on `server-26.2.jar`.

Closes the remaining open checker question (`maxY - 9` vs `min(maxY, minY + logicalHeight - 1) - 9` from `port-portal-entry-and-create`) and supplies end-to-end executed portal compounding truth (`build + ignite + 80t wait + 8:1 transit`) for kit unfolding. See `lab/results/local-20261124-155254-27792-1-0-vibe-gen-portals-elytra-piston-live/` for existing portal harness scaffolding.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` + Unsafe Overworld & Nether `ServerLevel`s + real `LevelChunk` + real `PortalForcer` / `PortalProcessor` / `PortalShape`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core shape/ignition/clock witnesses pass, then extend cleanly.

**Live scenes to execute:**
1. **`PortalShape` & ignition (`FlintAndSteelItem.useOn`, `BaseFireBlock.onPlace`):** minimum `4x5` (2x3 interior), maximum `23x23` (21x21 interior), `24x23` rejection, cornerless obsidian validity, `X` vs `Z` axis selection, interior air/replaceable requirement, and exact `NETHER_PORTAL[axis=X|Z]` block writes.
2. **`PortalProcessor` clock & decay:** tick order (`baseTick -> handlePortal` before `aiStep -> checkInsideBlocks`), post-increment `portalTime++ >= getPortalTransitionTime` (`80` ticks Survival vs `0` ticks Creative vs `-4/tick` decay when stepping out before arrival, cooldown behavior).
3. **`PortalForcer.createPortal` Y clamp (closing `lead -> reproduced`):** execute live `createPortal` on vanilla Overworld (`minY=-64, maxY=319, logicalHeight=384`), vanilla Nether (`minY=0, maxY=255, logicalHeight=128`), and custom `logicalHeight` with entry at `y=200` (`min(255, 127)-9 = 118` vs wrong `246`), floor `max(minY+1, 70)`, hostable 20-write vs empty-air 44-write fallback platform, and `findClosestPortalPosition` radius (`128` Overworld vs `16` Nether).
4. **`NetherPortalBlock.getPortalDestination` & `TeleportTransition`:** live `8:1` (`DimensionType.getTeleportationScale`) scaling + `WorldBorder.clampToBounds`, relative position inside frame `[0..1]` along width/height, `X<->Z` axis-swap `yRot` rotation (`+90°`) and `Vec3` velocity rotation!

**Write in your run folder:**
- `rows.tsv`: `rule_id	phase	dim_or_axis	param_or_bounds	formula_or_result	constants_hex	source`
- `witnesses.tsv`: `case_id	tick	dim	in_pos_hex	in_vel_hex	in_yaw	out_pos_hex	out_vel_hex	out_yaw	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed test where frame size bounds, wait/decay ticks, `PortalForcer` fallback Y clamp in Nether (`y=118`), `8:1` coords, or axis-swapped exit velocity/yaw disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
