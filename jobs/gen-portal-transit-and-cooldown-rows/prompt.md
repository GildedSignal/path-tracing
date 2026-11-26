# GLM 5.4 live verification: 26.2 Nether/End portal cooldown, lighting, and dimension transit timing

**Goal:** Execute live 26.2 `NetherPortalBlock.entityInside`, `EndPortalBlock.entityInside`, `Entity.handlePortal`, `Player.getPortalWaitTime`, `PortalShape.findPortalShape`, and post-transit cooldown tick-by-tick on `server-26.2.jar`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Live scenes to execute:**
1. Standing vs creative `getPortalWaitTime` (`80` ticks survival vs `1` tick creative) inside lit `nether_portal` (`X`/`Z` axis) recording `portalProcess` ticks until transit + post-transit `portalCooldown` (`300` ticks).
2. `EndPortalBlock` zero-tick transit + `PortalShape` min/max obsidian frame bounds (`2x3`..`21x21`) and flint-and-steel ignition check.

**Write in your run folder:**
- `rows.tsv`, `witnesses.tsv`, `dossier.md` (`Claim: ...`), `reproduce.sh`, `wiki.md`.
- Write only in your run folder; no game launch, network, or `work.patch`.
