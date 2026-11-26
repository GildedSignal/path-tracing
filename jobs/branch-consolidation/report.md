# Branch consolidation: build-4d, build-preview, track-face, server-mind onto main

Four local commits on `wt/branch-consolidation` over `036a52c2`, not pushed. Nothing under `engine/law/**`, `archive/turns/inbox.md` or `r.0.*.mca` is committed (`git diff --name-only main HEAD` holds none of them).

| Commit | Slice | Lines (all files) | engine/ + bot/ net |
|---|---|---|---|
| `2874f9c5` | 1, `engine/build` from `src/build-4d` `087b18ba` | +2,063 −1 | +2,062 |
| `1d8d9df9` | 2, `tools/preview` from `src/build-preview` tip `0cd32293` only | +596 −0 | 0 |
| `58a0afa3` | 3, B's clicked face from `src/track-face` `35a40137` | +248 −82 | +166 |
| `09dd12aa` | 4, `engine/mind` + `server-mod/` from `src/server-mind` `a802923b`, and the map | see commit | +5,945 |

## How each slice meets main

**1. Building (`engine/build`).** The crate kept its own copies of the state table; on main it reads the law's:
- `StateId` is `law::basis::cell::StateId`, interned through `cell::lookup`. Deleted: the local index type, the `defaults.txt` parser and the `world::BlockState` round trip.
- `Direction` is `law::neighbor::Direction`.
- An item is its `BlockId`. `ItemId` and four dead fields are deleted.
- Reach is `law::interact::is_within_block_interaction_range`, and the ray walk is `law::basis::ray::cells` (`law::blocks` no longer exists on main).
- The crate depends on `law` only.

Verdicts are unchanged against the source commit: `plan-build` prints the same table, and the pagoda gives 4,411 nodes, 87 scaffold cells, 24 roots, 77 layers, 649 max width and 0 residual obligations.

**2. Preview.** Only files from the tip commit: `Preview.java`, the two scripts, the view presets, the two kept PNGs, `timings.csv` and the job's `last.md`. One fix: `wrapper-run.txt` is written with `[IO.File]::WriteAllLines` into the resolved output folder. Before, it used `Set-Content` and joined an absolute `-Output` onto the root a second time.

**3. Face touch.** Re-applied onto main's spine rather than merged; `reach.rs` and `planner.rs` no longer exist on main.
- **The rule:** closed on the face's plane, open on both tangent axes, no epsilon. A selected face is the only face that counts; `None` means any of the six. Rust and Java share it, and both check the same 13 vectors (`bot/tests/fixtures/face-touch.txt`).
- **`Goal { cell, face }`:** `Face` is the law's `Direction`, so there is no second enum. `Goal::of(cell)` replaces every existing literal at the same line count.
- **Region:** `zone()` is the cell, or the selected face's square, and `gap`, `lower_bound`, `center` and `axis_sides` follow it. `axis_sides` stays the closed per-axis overlap that the straight gait bisects. It is a monotone necessary condition of a touch, so the bisection stays sound.
- **Wire:** `Track` carries `b_face` (from the definition's `bFace` or a seventh bounds field). `RequestGoal` takes an optional fourth goal word, the face's 3D-data value. `Bot.goToTrack` sends B and its face to the Rust planner.
- **Deleted:** the branch's straight-line driver helper `TrackDetector.aim()` and the 3-argument `touchesB` overload, both dead on main.
- **Java side:** main's `TrackDetector` still inflated B by `1e-3` while the Rust side had none. Both now use the one exact rule.

**4. Server mind.**
- **Palettes:** `WorldMirror` and the `.mca` loader hold `StateId` palettes instead of `&'static Block` pointers. `World::block` reads `StateId::block_at`, so offset models shift per cell. `state_id()` replaces a `model_id` that recovered a table index by pointer arithmetic.
- **Edits:** a delta is an edit only when the model changes (`same_physics`), as before.
- **Fence test:** main's planner now certifies a detour where the branch expected `NeedsJoints`. The test asserts the mirror holds the fence's collision and that the fenced plan costs more than open ground.
- **His world:** the disk-world test read `D:\path-tracing-lab\server\world` by default. It now reads a world only when `PATHTRACING_MIND_TEST_WORLD` names one.
- **Not taken:** `oracles/server-probe/` and the branch's map edits stay on `src/server-mind`. `.gitignore` anchors `/server/` so that `server-mod/src/pathtracing/server/` is tracked.

## Measured (this machine, this session)

| Gate | Result |
|---|---|
| `cargo test --offline -p build` | 6 passed; pagoda DAG 539 ms (release `plan-build`) |
| `javac` Preview.java (JDK 25) | exit 0, 17 classes |
| `preview.ps1 -Fixture` | PASS (lantern 15, neighbours 14, wall 0, 12 sizes, 19 emitters, mutant 72 px), 6.7 s wall |
| `preview.ps1` pagoda | 11 views, Java 2.20 s, wall 3.05 s, no missing textures |
| `-p plan --lib` | 5 passed |
| `--test face_touch` | 2 passed |
| `--test generalize` | 5 passed |
| `--test hard_track` | 5 passed |
| `--test kit_unfold --release` | 6 passed; e2 median 6.95 µs (main, same session: 6.95 µs) |
| `--test kit_unfold` (debug) | 5 passed, 1 failed: the e2 25 µs median ceiling (branch 1.09 ms, main 1.12 ms; it fails on main too) |
| `--test real_time --release`, n=60 per series | standing median 13.75 µs (p90 21.1, max 26.8); moving 13.95 µs (p90 18.2, max 21.3); repeat 5.10 µs (p90 5.2, max 11.1); 181 warm calls 0 B allocated, 0 law ticks. Main back-to-back: 13.15 / 14.20 / 4.90 µs |
| `-p pathtracing_native` | 7 + 2 passed |
| `-p mind` | 14 passed |
| `--workspace --no-fail-fast` (debug) | 252 passed, 1 failed (the debug kit_unfold ceiling above), 11 ignored; world 7 + 7 + 7 = 21/21, law 95 + 18 + 31, deviation 33 |
| `bot/build.ps1` | built `pathtracing.jar` |
| `bot/tests/test-native.ps1` | PASS |
| `server-mod/build.ps1`, `build-client.ps1` | both compile: 70,418 B and 37,033 B jars |
| TrackTests (headless, direct) | the face vectors, the top-then-west timer and the face roundtrip pass |
| `map/*.md` | 36,566 bytes (cap 40,000); the hooks pass |

## Not done, and why

- **No game flight.** None of the four capabilities has flown in his game: a face track, `KIT_PLAN` with a face, the mind with the server mod. `verify-stage2.ps1` and `verify-stage3.ps1` start servers and are not run here.
- **`bot/tests/test-tracks.ps1` is red on main** for two reasons that predate this work:
  - `detector()` expects the clock origin from before main's `lastOnA` rule ("Start tick is zero").
  - `TrackModeTests.transactions` fails "Finished run is recorded before restoring".

  Both fail identically on main's own sources. The face checks run before them and pass. The script also writes `jobs/track-ux/track-checks.json` with `Set-Content`, so I ran its steps directly instead.
- **`engine/mind/src/disk_world.rs` is a second region/NBT decoder** beside `engine/world`'s. It stays until `world` offers section palettes; adding that API would mean touching `engine/world`.
- **`engine/build`'s survival rules are its own** (gravity, water basin, hanging blocks). `rules.tsv`'s `survive_rule` covers only part of them.
- **The hearth half of `check-preview.ps1`** needs `jobs/build-hearth/place.txt`, which is generated and gitignored.
