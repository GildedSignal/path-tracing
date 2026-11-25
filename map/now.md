# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Flat ground, section spine & real-time hot path on `main` (`Load-bearing`, flown):** `plan-track --mode solve` writes exact tick scripts verified by `law::try_tick` across 16 / 16 unclipped bare-legs tracks (`782.2 ms` median total first-solve time; `flowers` 48 vs 48, `stonesimple` 85 vs 86, `straight800` 2,248 in 140 µs). `TopoField` and raw ticks read the `16³` `StateId` section spine directly (`World::uniform()` support proofs, `ObservedWorld` missing-cell guard, exact cycle skipping in `events.rs`). `tests/real_time.rs` (60 samples each): standing median `11.25 µs` (p90 `16.70 µs`, max `18.40 µs`), moving median `15.15 µs` (p90 `18.80 µs`, max `22.30 µs`), repeat median `5.90 µs` (p90 `8.50 µs`, max `9.70 µs`), `0` allocated bytes and `0` law ticks across all 180 warm calls ([section-spine report](../jobs/plan-spine-simplify/report.md)).
- **Kit unfolding Slices 1–2 on `main` (`Flying` offline; live wire builds, unflown):**
  - Core architecture ([kit.rs](../engine/plan/src/kit.rs), [defect.rs](../engine/plan/src/defect.rs), [certify.rs](../engine/plan/src/certify.rs), [word_store.rs](../engine/plan/src/word_store.rs), [solve.rs](../engine/plan/src/solve.rs), [source.rs](../engine/plan/src/source.rs)): `StateId(u16)` corridors, factored `< 1 MB` `KitView` (`StateStatic` + `LegsProjection` + `BlockBreakClasses` + law-probed `VerbEffect`s via `WorldDelta + RecordingWorld`, sharing `Arc` handles across `64 -> 63` stack decrements and clearing `VERB_PLACE` at `1 -> 0`), admissible `KitCone`, double-entry `ObligationLedger` + backward `shadow_price`, causal obligation-exit windows, `NoRegen` + `WorldDelta` restoration. One landing rule for every edit (`KitOp::lands_at`): clicks land before their step's first tick, breaks land before their last. A block places against any face of a non-replaceable, non-hand-used neighbour with a shape (`place_faces`, matching vanilla); airborne below a higher goal, the cell under feet is offered (the pillar).
  - Kit words & generalization: clicks travel as frame-relative `AimRule::Click(KitTemplate)` (`(fwd, up, right)` in `approach_frame`, verb, restore flag, goal-offset yaw; zero world coordinates or block names). Certified kit routes promote as one compound motif indexed at their first defect so bare-legs proposals stay isolated; Phase A instantiates kit motifs against the live `WorldDelta` + ledger before learning.
  - Live wire ([kit_wire.rs](../engine/native/src/kit_wire.rs), [KitAction.java](../bot/src/pathtracing/KitAction.java), [ResidentRunner.java](../bot/src/pathtracing/ResidentRunner.java)): `resident::Source` retains canonical `StateId`; `meta[7..]` (`PlayerSnapshot.context`'s dimension, mode, and 41 SNBT slots) builds `Kit::Carried` (including Nether `DIM_WATER_EVAPORATES | DIM_FAST_LAVA`); no-edit plans emit byte-identical `WORD_PLAN = 1`, while edited plans emit `KIT_PLAN = 4` (10-word per-edit action records executed via `useItemOn` / `useItem` / `startDestroyBlock` + `continueDestroyBlock` with self-edit echo tolerance). Item drops are unmodeled, so restores require matching blocks in the hotbar.
  - Spartan verification ([kit_unfold.rs](../engine/plan/tests/kit_unfold.rs) `6/6`, [Slice 2 report](../jobs/kit-unfold-slice-2/report.md)): zero tax on mixed ground (`bare_T=60`, `kit_T=60`, serial median `3.10 µs`), `10,000` kits in `356,196 B`, `KitCone` KT16 `0/500` violations, `R-lego-1..4` (`45`/`57`, step-up `60`, mid-air `50` vs stop-and-place `159`, wall `38`, door `24`, water clutch `32`), Nether powder snow `36`; door corridor (`8,191 -> 0` learn ticks, `T=24`) and stone wall breach (`9,548 -> 0` learn ticks, `T=38`) generalize to `(+25, 0, +17)` + `90°` rotation on one `GeneratorStore`, with `edit()` invalidating only inside footprints; verified `WorldDelta` + `law::try_tick` track replays: `doorcorridor` `32` restored (his `99`), `amertystcoral` `33` restored (his `63`), `ametwoodbox` `45` restored (his `124`, with axe), `mud` `47` unrestored (his `97`; restored stays `NeedsJoints` because the pillar base exceeds `4.5` reach from the walkway).
- **Recursive `Word` / `WordStore` defect crossing on `main` (`Flying` offline):**
  - Core architecture: coordinate-free `Word`, rotation-invariant block-name-free `DefectSig`, raw exploration on section-backed terrain with `TopoField`, Phase A proposal before Phase B search, and single-winner `RecordingWorld` certification (`NoRegen` + footprint-scoped `edit()`).
  - Bare-legs ladder: **16 / 16 unclipped tracks pass** (`grassjumpeasy` 38 vs 50, `stonestaircase` 99 vs 113, `fencestraightbridge` 46 vs 47, `mangrovejumpeasy` 53 vs 45, `fences` 111 vs 89, `fencemaze` 68 vs 66, `redfakemaze` 61 vs 71, `carpetwater` 95 vs 85, `butoonpressureplates` 79 vs 71, `trapdoorparkour` 122 vs 184, `doorparkourblue` 86 vs 126); `generalize.rs` `5/5` (11-track shared store `0.95 s`). Without a kit, five tracks return `NeedsJoints` (`birchparkourmedium` clipped by `tracks.txt` bounds, plus the four kit tracks above).
- **Bot mod wire on `main`:** [resident.rs](../engine/native/src/resident.rs) and `ResidentRunner.java` handle `WORD_PLAN` and `KIT_PLAN` with a 4-word corridor hull, `store.words` block-edit invalidation (`edit()` kinds 1/2/3/4), `first_uncovered` chunk growth, and `pathtracing_native` `9/9`.
- **26.2 Law & `16³` `StateId` world spine on `main` (`Flying` offline):**
  - Full 26.2 law (`core.rs`, `face.rs`, `sim.rs`, fluids, mining, placement, redstone, doors, boats, portals, pistons, combat) verified across 9,117+ jar witnesses, replays, and `WorldDelta` rollback (`checkpoint` / `rollback`, ref-counted capability hooks), backed by canonical `StateId(u16)` `16³` sections with `4³` summary masks (`World::uniform`).
  - **Companions:** GLM 5.4 (`jobs/glm-chain.txt`) running `gen-water-swim-climb-rows` and `gen-ground-tags-rewitness`; Mistral Vibe (`jobs/mistral-chain.txt`) finished 5 live jar-truth prompts; Codex Sol (`yarn-cold-fold`) completed Slice 1 on `wt/yarn-cold-fold` (full 26.2 tables + 3 boundary fixes) and runs Slices 2+3 in `D:\path-tracing-wt\yarn-cold-fold-solo`.
  - **ES 1B-bot probe ([es-1b-probe](../jobs/es-1b-probe/report.md)):** passes 5/5 checks and 3/3 mutants offline (`10⁹` bots / `10⁶` scars in `4.28 MB`, `172 µs` edit split, `129.8 µs` max observer frame, `0.418 ms/tick` 500-bot `Battle`).

## Next, in order

1. **Live flight verification** of bare-legs crossings and the Slice 2 kit wire (`doorcorridor`, `ametwoodbox`, door/wall motifs) in Julien's game, plus unclipping `birchparkourmedium` bounds when needed.
2. **Cold-law fold (`yarn-cold-fold` Slices 2+3)** and item drops (restores picking up drops after 10t), client 5-tick break cooldown in the law, and `mud` restored.
3. **Far map & ES-native `10⁹` bots (`scales.md`, `server.md`):** shared goals, `Wheel`, `(max, +)` building, and melee singularity handoff ([pvp-strategy-2026-11-22.txt](../archive/research/pvp-strategy-2026-11-22.txt)).

## Derived component state (ES rule)

| Component | Derived state | Scope |
|---|---|---|
| Core `solve`, `Word` / `WordStore`, `TopoField` 16³ spine, `NoRegen` `certify` | `Flying` | `main`, `16/16` bare-legs tracks + `generalize` green |
| Kit unfolding Slices 1–2 (`kit.rs`, `defect`, `certify`, `word_store`, `kit_wire.rs`, `KitAction`) | `Flying` offline | `main`, `kit_unfold` `6/6`, native `9/9`, jar builds |
| 26.2 law + `16³` `StateId` world spine + `WorldDelta` | `Flying` | `main`, jar witnesses + replays green |
| Live JNI resident channel (`resident.rs` + `ResidentRunner.java`) | `Load-bearing` (flat) / `Flying` (words) | `main` |
| Parked `wt/flight-ladder-sets` | `Sketch` (unmerged) | parked unmerged; core recursive `Word` lives on `main` |
| Live kit flights, far map, ES `10⁹` bots | `Sketch` | awaits live game judgment on `KIT_PLAN` and `yarn-cold-fold` |

## Branches and open notes

- Kept until needed: `wt/yarn-cold-fold` (`D:\path-tracing-wt\yarn-cold-fold-solo`), `wt/port-fold` (`origin/wt/port-fold`), `wt/flat-ground` (`lab/fixtures/flat-ground-replay/`), `wt/server-mind`, `wt/build-4d`, `wt/track-face`, `wt/hard-track-2`, `wt/l0-veto`, `wt/build-preview`, `wt/hearth-small`.
- Untrimmed region files `r.0.0.mca` / `r.0.-1.mca` stay uncommitted on `main` (`*.bak-20261114` backups beside them).
