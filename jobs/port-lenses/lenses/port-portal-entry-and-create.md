# Port delta: Nether portal entry counter and PortalForcer.createPortal

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-wormhole-facts` already settled `80`/`0` wait, `10t` cooldown, `8:1` XZ scale, and `16`/`128` `findClosestPortalPosition`; do not re-ask those.

**Consumer:** `docs/plan.md` step 8 (wormholes: portal), batch `P4c` (`jobs/port-backlog/report.md`), and `port-wormhole-facts`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `NetherPortalBlock.entityInside` -> `Entity.setAsInsidePortal` / `PortalProcessor` / `Entity.handlePortal` (to locate): exact overlap trigger (`entityInside` shape), how `portalTime` increments and resets if the player steps out before `80` ticks, and in which tick phase (`Entity.baseTick` vs `aiStep`) the dimension transfer fires;
- `PortalForcer.createPortal` (to locate): when no exit portal exists in `findClosestPortalPosition`, exact search/fallback (`BlockPos` spiral & Y bounds, to locate), frame obsidian blocks, portal interior blocks, and obsidian/air platform cleared around the exit.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `NetherPortalBlock.entityInside`, `PortalProcessor` (to locate), `Entity.handlePortal`, and `PortalForcer.createPortal`, and print the exact portal-timer trace (enter, step out early, re-enter) and `createPortal` block-write footprint. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any portal counter reset rule, tick phase, or `createPortal` block footprint that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
