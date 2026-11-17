# Port delta: PortalForcer.createPortal Y-ceiling, floor and empty return

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-portal-entry-and-create` (`local-20261117-1537-1-0`) already settled the 81-tick counter, `-4`/tick decay, 20-write hostable footprint, and 44-write fallback footprint, failing only on the fallback ceiling formula (`maxY - 9` vs `min(getMaxY(), getMinY() + getLogicalHeight() - 1) - 9`); do not re-ask the counter or footprints.

**Consumer:** `docs/plan.md` step 8 (`P4c`, Nether portal creation in Overworld, Nether, and reduced-`logicalHeight` dimensions), closing `port-portal-entry-and-create` (`lead -> reproduced`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `PortalForcer.createPortal`: exact offsets for `topUsable = Math.min(level.getMaxY(), level.getMinY() + level.getLogicalHeight() - 1)` (`@33-57`), the fallback floor `Math.max(level.getMinY() - (-1), 70)` (`@452-459`), the fallback ceiling `topUsable - 9` (`@464-469`), the empty-return gate when `ceiling < floor` (`@475-481`), and `Mth.clamp(pos.getY(), floor, ceiling)` (`@497-505`);
- `WorldBorder.clampToBounds` (to locate) applied to `pos - dir` right before the clamp (`@448-534`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `PortalForcer.createPortal` and `WorldBorder.clampToBounds`, and invoke `PortalForcer.createPortal` directly across: (1) Nether dimensions (`minY=0, maxY=255, logicalHeight=128`) at entry `y=200` (proving clamp to `118`, NOT `200`), (2) Overworld (`minY=-64, maxY=319, logicalHeight=384`) at `y=40` (`70`) and `y=999` (`310`), (3) `minY=0, maxY=256, logicalHeight=256` at `y=999` (`246`), and (4) `maxY=78` (`Optional.empty`, 0 writes). Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any fallback base Y coordinate or `Optional.empty` threshold that disagrees with `PortalForcer.createPortal` bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
