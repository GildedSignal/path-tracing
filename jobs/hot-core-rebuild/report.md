# The hot player core (Opus, 23 November): report

`wt/port-fold` `38a49808` → `f24a6317`, 3 commits. Rerun by the integrator: workspace builds; lib 95 (the twins' in-file tests went with them), jar 18, replay 31 + 4 ignored; the recording replays `replay` and `tracks` pass; `plan`'s `real_time` passes (Opus: its 364 lines byte-identical to `38a49808`).

| Commit | What |
|---|---|
| `0b75bf6a` | his track recordings join the replay gate: 18 of 91 byte-identical against the region fixture (`tests/data/track_gate.txt`); `scripts/engine-lines.sh` |
| `97ab9a05` | `core.rs` beside the oracle; a differential test: 151,002 states × 5 causes = 755,010 pairs, 0 anomalies |
| `f24a6317` | `try_tick` runs the core; `movement.rs` deleted whole, `player.rs`'s hot half and its test-only twins deleted |

**Lines** (unformatted: no `rustfmt` on D:): `engine/` code 25,446 → 24,146 (0.023 decades), tests 11,902 → 11,748; the player's hot files 2,315 → 1,017 code lines (0.36 decades). `core.rs` is 746 code lines; no group over 2× the table.

**The gate now holds the game:** 13 flat-ground recordings plus 18 of his 91 track recordings. 51 are out because their chunks are not in the fixture (infinity_tower 2–4 and 8–36, hardtrack, parkour_race, practice 1–6, easy_dropper 6–8); 22 because they were recorded on worlds the fixture predates (the body falls through air at ticks 0–14). The double-tap sprint window is now a law input. The track world is an untracked `lab/track-world/` (~7 MB, main's full regions) and `recordings/` an untracked junction; without them `tracks` prints a skip.

**The differential test:** states from every tick of the 31 recordings, 9 extremes around each, 30 hazard blocks × 4 bodies × 30 ticks; it compared state bits, effects and a hash of every footprint and hook read. It reached 78 of the oracle's 99 hot functions, every one on the tick path. One anomaly class (a missing footprint call in the core), fixed before the switch: `anomalies.tsv`.

**One hurt pipeline:** a `Hit` row (amount, fire, armour, effects, fall, exhaustion) replaces three paths. **Dropped as unreachable by any recording, row or `real_time` line:** the client-level collector path, the riding sprint and swim rows (now checked as refusals), `DamageScaling::Always`, the 1e-5 impulse boundary, the unit-input travel twin. No TSV row edited or lost.

**Bench** against `38a49808` (10 rounds): walk 0.993, sprint-jump 0.984, ceiling 0.969, bumpy 0.999.

## Not done

- **The face is per tick** (state + input → effect, the causes ordered inside it); the event face, a jump to the next cause, is not built.
- **The hot state is still 296 B:** the planner's codec and the mod's wire share its layout. Yaw and pitch still sit beside the input's copies; `movement_this_tick` is still stored.
- Tests that name internal helpers remain: six in `core.rs`, ~60 helper paths in `jar.rs`.
- The speed fold allocates when an effect is present.
- 73 of his recordings need their world: the missing chunks exported, or a world per recording.
