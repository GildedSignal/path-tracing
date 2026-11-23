# Law basis, pass 2 (Opus, 23 November): report

`wt/port-fold` `c392524` → `d068260a`, 6 commits, each green on the gate (lib 101, jar 18, replay 29 + 5 ignored; rerun by the integrator). No witness row edited; one jar harness assertion changed (below). The planner's `real_time` T, `law_ticks` and refusals are byte-identical to `c392524` over 367 lines.

| Commit | What |
|---|---|
| `5c04fdc2` | `scripts/bench-judge.sh`: both sides built from archived trees, functions 64 B aligned, A B B A on one pinned core; `scripts/law-lines.pl` counts code lines |
| `6c8d6883` | footprints per step (`World::state_footprint(read, written)`); the 533 per-field macros gone |
| `04393844` | one declaration (`player_state!`), one `wire!` list for `bits`/`from_bits`; hand-written `Clone`, `state_fields!`, `Encode` deleted |
| `930ef155` | the effect book inside `PlayerState`; `WitherRose` is a row (WITHER 40/0), not a refusal |
| `dd8f4d8e` | hot state plus `Shared<Profile>` (41 fields no tick writes) |
| `d068260a` | `clock::COOLDOWN`: nine "−1 a tick while positive" counters as one row |

**Bench:** identical code on both sides reads 1.000 ± 0.3%. At `d068260a`, 12 rounds, ratio to base: walk 1.008, sprint-jump 1.015, ceiling 1.010, bumpy 1.012 (base 966 / 627 / 893 / 934 ns). Edits that should compile the same move it 0.995–1.020: layout, not work. Not faster yet.

**Code lines** in `engine/law/src`: 21,863 → 21,403 (total 30,846 → 30,400). Generators: body 117, sweep 137, ray 166, clock 17 (budgets 300 / 300 / 150 / 150); `cell`, `contact` not written. No old file deleted.

**Hot state:** 296 B plus a 152 B shared profile (was 416 B). The ~24–32 B target does not hold for the player: position and velocity in f64 are 48 B alone.

**Harness change (accepted by the integrator):** the 8 `WITHER_ROSE_GATE` rows asserted "applied ⇒ the tick refuses"; now "applied ⇔ the book holds exactly WITHER 40/0 after the tick". The rows' expected column is unchanged.

**GLM lanes:** all four ended "incomplete, no dossier": Z.ai answered 429/529 (rate limit, overload) through the 90 minutes with four in parallel. Nothing used.

## Left

- The player's travel as `LIVING` rows (input, sprint, jump, swim, climb, powder snow, fluids); fire, freeze, exhaustion as clock rows.
- `cell` keyed on the world crate's model table (`models.tsv`/`blocks.tsv`; `census.tsv` lists only buckets); `sim.rs` (4,324 code lines) still doubles the world.
- `contact`; the three pass-1 witness rows (no Java on PATH for the builder).
- Side effects of the wire's effect tail: one malformed encoding refuses at 137 ("missing word") instead of 136 ("extra word").
