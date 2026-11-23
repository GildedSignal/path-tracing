# Port compress, pass 1 (Opus, 23 November): report

`wt/port-fold` `b605c9b` → `996a652`, 7 commits, all green on `cargo test -p law --release --lib --test jar --test replay` (lib 100, jar 18 over 9,117 TSV lines, replay 29 + 5 ignored; rerun by the integrator). No witness edited. `engine/law/src`: 31,351 → 31,393 lines (+1,501 folded from the dossiers, −1,459 deleted); ~7.7K of it are in-file unit tests (the dossiers' witnesses), ~19K code.

| Commit | What |
|---|---|
| `25c25b7` | fold the three GLM dossiers (`14522x` runs): status-effect ticks (`effects.rs`), shield/crit/sweep/mace (`combat.rs`), item entity and inventory (`items.rs`) |
| `20fe8fa` | clean: delete law items nothing reaches (−1,250) |
| `22febd3` | clean: one `Direction`, one `JavaRandom`, one view vector, one `wrapDegrees`, one clamp (−148) |
| `299a657` | `BoatQuery` folded into the one `World` read side (−119) |
| `e793446` | generator `affine_body` (`body.rs`): rows `FALLING_BLOCK`, `THROWABLE`, `ARROW`, `ITEM` (+33) |
| `0b9fbe4` | `MINECART` row (−3) |
| `996a652` | effect damage through the one `actually_hurt` (jar's `f - max(f - absorption, 0)` rounding), one `addExhaustion` (−8) |

Bench median ns/tick, before → after: walk 990.8 → 978.3, sprint-jump 631.3 → 628.2, ceiling 910.6 → 902.3, bumpy 961.7 → 951.0. `perf_core` unchanged within noise.

## Left

- **World:** six edit-side traits (`PistonWorld`, `RedstoneWorld`, `BucketWorld`, `TripwireWorld`, `PortalWorld`, `Rails`) with four bridge views in `sim.rs`; `SimWorld` holds the world twice (`SimCell` map plus `Block` kept in step by `sync_block`, ~600 lines); three wrappers repeat all 24 `World` methods (`ContactRecorder`, `RecordingWorld`, `WorldOverlay`).
- **Generators:** boat and player travel not on `affine_body`; no clock generator; no `cell_rewrite` + calendar; `aabb_sweep` exists twice (`shapes::collide_with_shapes`, `movement::collide_in_world`, the player's hot path). `ray_dda` is one function (`blocks::traverse_blocks`).
- **`PlayerState` split:** not started (~100 fields listed in three places). The effect book `Effects` sits beside `PlayerState` (its `absorption` duplicates the state's), so `WitherRose` still refuses.
- The 10–15K target also needs the in-file witnesses moved to data rows.
- The `replay` test is ignored: no recorder fixtures in `lab/fixtures` on this branch (`wt/flat-ground` holds 15 MB of them, untracked).
