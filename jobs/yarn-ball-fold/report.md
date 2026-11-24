# Yarn-ball fold (Opus, 24 November): report

`wt/port-fold` `43f43730` → `28f9cbcb`, 1 engine commit. **STATUS: BLOCKED** — the three slices' premise does not hold at `43f43730`: the duplication they would fold was already folded by earlier passes, and what remains is unique, jar-witnessed rule logic or the frozen kernels. The 1,500-line bar is not reachable without deleting witness rows, deleting witnessed rules, or counting reformatting. One honest whole-table move shipped; no micro-trims.

## What shipped

| Commit | What | Law code lines (raw) | After rustfmt |
|---|---|---|---|
| `28f9cbcb` | elytra: the 252 rows where HotSpot's `cos` differs from fdlibm move from a Rust array to `src/dcos.bin` (input bits, output bits, LE, sorted), read by `include_bytes!` like the sine table | 19,047 → 18,796 (−251) | 23,504 → 23,259 (−245) |

Per file: `elytra.rs` 678 → 427 code lines (`perl scripts/law-lines.pl`); no other law file changed. `engine-lines.sh` (rustfmt now installed on D:, so it is formatting-neutral): engine code 35,988 → 35,743, tests 19,445 → 19,445.

**Decades.** Whole law: log10(19,047 / 18,796) = 0.006. Group 14 (elytra, `table.md`): 626 → 375 = 0.22 decades, now under the table's "220 + table as data" estimate plus the fdlibm kernel. Targeted groups 2, 7, 12, 18 and the ground rows: 0.

## Why the slices are blocked (each claim checked by a command this session)

1. **Slice 1, bodies and kicks: no per-class loops are left to fold.** `pearl::pearl_tick` runs `basis::body::THROWABLE.gravity/drag`; `projectiles::arrow_free_tick` runs `ARROW`; `falling::flight_tick` runs `FALLING_BLOCK.tick`; minecart runs `MINECART`; the wind charge is the identity row. The kick sites are different formulas: `living_knockback` is v/2 − dir·k with a 0.4 y cap and an RNG retry; `cause_extra_knockback` is ×0.6 on the attacker with the server-player restore; the explosion is dir·(1−d)·exposure·k; firework, riptide, punch and fishing pull each differ again. One `v' = a·v + b·dir + cap` row family would replace about 30 lines, while the direction and condition code stays at every call site. The rest of `projectiles`, `pearl`, `falling` and `explosion` is distinct Mojang methods (bow curve, crossbow quaternion fan, fishing cast, arrow damage and crit, pearl use and raytrace, explosion exposure, falling landing gate) pinned by BOW_*, XBOW_*, FISH_*, ARROW_*, PEARL_*, WC_*, EXP_* and `falling.tsv`. `sim.rs` calls the falling and pearl APIs (`LandingContext`, `pearl_tick`), so changing their shape means editing the frozen `sim.rs`.
2. **Slice 2, clock and ledger: the ledger is already one pipeline.** `core::window` + `core::mitigate` + `Hit` is the only hurt path; `effects::hurt` calls them. `core.rs` has no air law, so `breath::tick_air` is not a duplicate: deleting `breath.rs` means moving its 26 lines, not removing them. The only near-twin is `effects::eat` / `items::eat` (5 lines).
3. **Slice 3, ground rows: friction, speed and jump are already rows** (`world/data/models.tsv`, loaded by `basis/cell.rs`). There are no hard-coded ground switches to delete. GLM's `gen-ground-rows` checked against the law's models with a scratch probe (`law::basis::cell::model`, not committed): **1,196 blocks, 668 bit-equal on all three factors, 0 different; 528 need state properties and were not checked.**
4. **Where the law's mass is** (caller census over every `fn` in `engine/law/src`, code lines): reached from live code 5.6K; private helpers 6.0K; reached only by `tests/jar.rs` 1,887 (unique rules, each pinned by rows); reached only by in-file `#[cfg(test)]` modules 668 (placement, boat seat and dismount, elytra glide gates and wear, item merge and drop, tool rules — unique rules, none a twin of a live path); dead with no caller at all 9. The bar needs 1,500 from this pool without deleting a row.
5. **Reformatting is not progress.** Under rustfmt the law is 23,504 lines against 19,047 raw. A dense rewrite of `projectiles`/`pearl`/`falling`/`explosion` would drop roughly half their raw lines and close to nothing after rustfmt. I measured both and did not ship it.

**The exact pin:** the bar is reachable only by one of (a) deleting the in-file-witnessed rules no caller uses (668 lines: placement, boat seat, glide gates, item merge and drop, tool rules), (b) rewriting `sim.rs`'s calling side so the Java-shaped context structs and enums between it and the rules go (rule 3 forbids nibbling `sim.rs`), or (c) driving the jar-only rows through a world-level boundary (`sim` step or `WorldMut`) instead of per-method helpers. Option (c) is the `table.md` rebuild of groups 1, 4, 6 and 16 (~2.9K → ~1K), which is a cell-engine rebuild, not this fold. Each one is his call.

## Gates (run at `28f9cbcb`, `CARGO_TARGET_DIR=D:\path-tracing-target\job-yarn-ball-fold`)

| Gate | Result |
|---|---|
| `cargo test -p law --test jar` | 18 passed, 0 failed; no TSV or expected value touched |
| law lib | 95 passed |
| `cargo test -p law --test replay` | 32 passed, 4 ignored, 0 failed (279.9 s) |
| `cargo test --release -p plan --test real_time` | passed; its 364 CMD/END/REFUSED lines (timings stripped) byte-identical to `43f43730` |
| `cargo test --workspace --no-fail-fast` | 3 failures, none from this change: `native` `resident::tests::existing_f_uses_exact_observed_start_and_returns_memory_adapter` (the same failure at `43f43730`, rerun from an archive of that commit: the answer is `Unknown(Loaded)`); `plan` `deviation::solve_scripts_reach_the_saved_track_finish` (`plan::flight::track` cannot find a track world file, an I/O error before any law code runs); `world` `blocks::julien_tracks_load` (the fixture chunk anomaly the law-basis-3 report records). The rest pass: plan lib 5, `hard_track` 5 + 1 ignored, words 1, world 6 + 7, `state_schema` 2 |
| `bench-judge.sh c392524 12` | walk 1.001, sprint-jump **1.012**, ceiling 0.990, bumpy 1.001 |
| `bench-judge.sh 43f43730 12` | run 1: walk 0.999, sprint-jump 1.004, ceiling **1.006**, bumpy 1.002; run 2: walk 0.999, sprint-jump 1.000, ceiling 0.999, bumpy 0.998 |

**Bench reading.** The changed code (`elytra::jvm_dcos`) runs only while gliding, and no bench scenario glides. Medians, ns/tick (base, candidate): run 1 against `43f43730`: walk 968.1 / 967.1, sprint-jump 632.4 / 634.1, ceiling 886.7 / 890.5, bumpy 934.2 / 934.3; run 2: walk 968.8 / 967.8, sprint-jump 632.2 / 632.2, ceiling 886.4 / 885.1, bumpy 932.7 / 931.0. Against `c392524`: walk 967.2 / 968.9, sprint-jump 626.4 / 632.8, ceiling 894.9 / 886.9, bumpy 934.2 / 933.9. Run 1's ceiling 1.006 did not repeat. The sprint-jump 1.012 against `c392524` is already in `43f43730` itself (its sprint-jump median is 632.2–632.4 against `c392524`'s 626.4, a ratio of 1.009). I inferred that from the medians; I did not bench `43f43730` against `c392524` directly. So the `c392524` gate fails on sprint-jump for a reason that predates this job.

## Files

- `.scratch/` in the worktree (untracked): `callers.pl`, `spans.pl` (the caller census), `fmt-lines.sh` (law lines after rustfmt), the raw bench outputs.
- `rustfmt` was added to the D: toolchain (`rustup component add rustfmt`), so `scripts/engine-lines.sh` now counts formatted lines, as its header intends.
