# Law basis, pass 1 (Opus, 23 November): report

`wt/port-fold` `996a652` → `c392524`, 4 commits, each green on `cargo test -p law --release --lib --test jar --test replay` (lib 100, jar 18 over 9,117 rows, replay 29 + 5 ignored; rerun by the integrator). No witness edited. Code lines (non-blank, non-comment, outside `#[cfg(test)]`) in `engine/law/src`: 22,485 → 21,863; total 31,424 → 30,846.

| Commit | What |
|---|---|
| `781a4df` | `basis/sweep.rs` (one axis-by-axis AABB sweep, step-up as the same sweep lifted; player, boat, sim bodies, jar shape corpus) and `basis/ray.rs` (one DDA, one clip); `blocks.rs`, `contact.rs` deleted |
| `ad09a38` | `body`: one `Entity.move` tail (setPos rule, `Mth.equal` flags, restitution); rows gain vertical drag and `LIVING`; one `friction_drag` |
| `44ce58c` | `clock`: `Clock { step, at, reset }`; rows `FOOD_SATURATED`, `FOOD_FED`, `AIR` |
| `c392524` | `body`: one launch kernel; every projectile item is a row (power, inaccuracy, roll) |

| Generator | Code | Total | Budget |
|---|---|---|---|
| `body` | 117 | 186 | ~300 |
| `sweep` | 138 | 168 | ~300 |
| `ray` | 166 | 201 | ~150 |
| `clock` | 14 | 25 | ~150 |
| `cell` | — | — | ~400, not written |
| `contact` | — | — | ~200, not written |

Bench median ns/tick, base → `ad09a38`: walk 982 → 973, sprint-jump 630 → 640, ceiling 903 → 903, bumpy 950 → 958. The bench moves ±5% with code layout alone.

Matched to the jar on edges no witness reaches (each worth a jar row): the boat's x/z collision flags use `Mth.equal`; the sim's falling block restitutes; the pearl's inside-block clip returns the 0.001 nudge.

## Left

- **`PlayerState`:** 533 `read_state!`/`write_state!`/`modify_state!` calls (419 in `movement.rs`), every field access instrumented for footprints. The compact hot state with footprints per group comes before the player can be rows; the effect book goes in (lifts `WitherRose`).
- **`cell`:** `sim.rs` (4,346 code lines) keeps a private 38-block palette and holds the world twice (`SimCell` and `Block`). `cell` belongs on the world crate's census table behind one `WorldMut` with reversible deltas; with `world_hooks`, `block_hooks`, `interact`, `place`, `neighbor`, `redstone`, `fluid`, `piston` that is ~10K code lines.
- **`contact`:** entity push in `entity_contact.rs`; inside-a-cell effects an enum match in `movement.rs::inside_block`.
- Fire, freeze, hurt cooldown, portal, pickup delay onto `clock`; the arrow's drag as a named scar; the projectile hit test into `ray`; in-file witnesses (~7.7K lines) into TSV rows.
