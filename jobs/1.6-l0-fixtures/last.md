Implemented. Changes are uncommitted. Validation was offline.

```powershell
.\lab\gates\record-fixtures.ps1 -Jar .\bot\build\pathtracing.jar
```

### Changes

- Added `WorldExporter.java` and Bot command routing for client-level exports through `.tmp`, with overwrite refusal, context shapes, fluid data and world metadata.
- Added `PlayerSnapshot.java`, Recorder header capture before input zero, three read-only state-access mixins and their configuration.
- Added `courses.ps1` with all 13 L0 courses, enclosed by floors, walls and roofs on separated patches.
- Added `fixture-rig.ps1` using 1.2e’s bounded transport and process ownership handling.
- Added `record-fixtures.ps1` with course readiness checks, seeded replay, visited-box export, server sampling, rejection checks, staged publication and reports.
- Added three offline test scripts.
- Documented usage, derivations and assumptions in `bot/recorder.md` and `jobs/1.6-l0-fixtures/last.md`.

### Validation

Passed the bot build, 27 driver stub cases, all course structural checks, PowerShell 5.1 parsing/ASCII checks and the hand-written block check through a compiled copy of the Rust JSON parser.

### Class derivations

- `ScaffoldingBlock` supplies `scaffolding`.
- `SlimeBlock` supplies the slime landing/step hook.
- Water-bearing `LiquidBlock` and `BubbleColumnBlock` supply `waterBlock`.
- `WebBlock`, `PowderSnowBlock` and `HoneyBlock` select their respective `inside` hooks. Others select `none`.
- `HayBlock` and `HoneyBlock` supply landing multiplier `0.2F`, `BedBlock` supplies `0.5F`, and the default hook supplies `1.0F`.
- `LadderBlock` and open `TrapDoorBlock` supply their facing property as vanilla 3D direction IDs.

Bounce, material factors, tags, shapes and fluid values come from game getters.

### Assumptions and integration limits

- The configured rig is the intended superflat. Reserved course patches are replaced. The bot has operator permission and command feedback.
- Export corners are normalized. Visited bounds include initial and post-tick feet positions, floored and expanded by three blocks.
- The player uses a supported vanilla profile, terrain stays movement-equivalent during replay, and entities do not interfere. Resources, equipment and effects carry over.
- Initial server resources are sampled before teleport/settling. Health, food and saturation must match the settled client. Exhaustion is assumed constant across that interval.
- `serverEnd` queries begin immediately after observing completion, but remain sequential and tick-unaligned. Resource exactness is provisional.
- Seeds use `Seed + courseIndex*Runs + run-1`, preserving sequences under course selection.
- Holding-scaffolding shapes are retained as extra metadata that Rust currently ignores.
- Bubble columns were omitted because the frozen schema lacks their acceleration hook.
- The current `wt/law` replay reader needs `end`-trailer support before consuming these recordings. Live export delivery and terrain/mutant coverage await the integrator’s run.