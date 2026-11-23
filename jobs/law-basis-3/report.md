# Law basis, pass 3 (Opus, 23 November): report

`wt/port-fold` `d068260a` → `38a49808`, 4 commits, each green on the gate (lib 101, jar 18, replay 29 + 5 ignored; rerun by the integrator, workspace builds). No witness row edited. Planner `real_time`: 364 CMD/END/REFUSED lines byte-identical to `d068260a`.

**Code lines** (`engine/law/src`): 21,403 → 20,185 (total 30,400 → 29,087): **0.025 decades**; 1.13 decades above the ~1,500 floor. No old file deleted.

| Commit | What | Lines | Decades |
|---|---|---|---|
| `0faab598` | dead code the compiler finds with the tests built in (`MiningClock`, `ServerTickPhase`, ~12 small items) | 21,403 → 21,169 | 0.005 |
| `5e60475f` | `basis/cell.rs`: the world crate's generated tables move into `law`; `BlockId` is a census row; `rules.tsv` (hardness, push reaction, conductor, signal, survival) for the 39 sim blocks; `sim.rs`'s palette, `CellProps`, 30 constructors and hand-built blocks go | 21,169 → 20,401 | 0.016 |
| `1e8d9d22` | `CellState`/`SimCell` into `cell`; one `WorldMut`; `RedstoneWorld` and five state types go | 20,401 → 20,234 | 0.004 |
| `38a49808` | tripwire on `WorldMut`; `TripwireWorld` goes | 20,234 → 20,185 | 0.001 |

Generators: cell 366 / 400, body 117 / 300, sweep 137 / 300, ray 166 / 150, clock 17 / 150; `contact` not written. Bench against `d068260a`: walk 0.980, sprint-jump 0.994, ceiling 1.002, bumpy 1.010, all inside the ±1.5% layout spread.

Harness assertions rewritten (rows and expected values unchanged): `jar_world_sim` (palette ids → `BlockId::of`), `interact` part 6 tests (map world of `SimCell`), `jar_world_hooks` (tripwire world on `WorldMut`). `rules.tsv`'s `copper_door` row (hardness 3) is not checked against the jar.

**What the pass found:** the mass is not dead code. 991 of 1,106 law functions are reached by a witness; the family files are witnessed rule logic written as Java-shaped code. Deleting them means re-expressing each rule as rows over `cell`/`WorldMut` and rewriting its harness (`jar_world_hooks` alone is 940 lines).

## Left

- `BucketWorld` (needs Slab/Ladder back in `CellState`), `PistonWorld`, `PortalWorld`, `Rails` onto `WorldMut`.
- A world that stores cell ids: `WorldOverlay`, `SimCell.block` and the fallback in `cell()` go; then each family's rule onto `CellState`, its file deleted.
- The player's travel as `LIVING` rows (`inside_block` is still the enum match); `contact`.
- The three pass-1 jar edges as witness rows.
- `world/tests/blocks.rs::julien_tracks_load` fails on a missing fixture chunk (18, −31), as at `d068260a`: the fixture `.mca` anomaly on `main`.
