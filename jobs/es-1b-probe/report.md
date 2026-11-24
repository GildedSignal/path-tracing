# ES 1-Billion-Bot Feasibility Probe (`jobs/es-1b-probe`)

## 1. Verdict

All 5 Spartan checks **PASS** under `cargo run --release -- --all` (exit code `0`), and all 3 planted mutants (`per-bot-index`, `no-reabsorb`, `dirty-observer`) **FAIL** cleanly with exit code `1` on their targeted gates.

| Check | Question Answered | Outcome | Key Measured Result |
| :--- | :--- | :--- | :--- |
| Check 1 | `10^9` bots across 1,000 LIVE ES `Program` streams (`100,000` scars) vs LIVE `law::try_tick` | **PASS** | `1,000` stratified samples, `0` bit-exact mismatches |
| Check 2 | Single block edit shock across `N = 10^3 -> 10^9` + arrow crossing + undisturbed `WorldGait` | **PASS** | Log-log median slope `= 0.0038` (`< 0.05`); `wake stream/bot=1, arrow=1, wg=1`; `0` Wheel wake-ups over `100k` ticks |
| Check 3 | Sub-32 MB L3 residency + `seal()` scar reabsorption (`k = 10^6 -> 500k` on `10^9` bots) | **PASS** | Fixed `k=100k`: `0.703 MB -> 0.886 MB` (`1.261x`); `k=10^6`: `4.282 MB -> 2.371 MB` after 50% rejoin (`0.0045 -> 0.0025 B/bot`) |
| Check 4 | Pure observer 192-block flyover across `10M`-bot highway (`500` frames) | **PASS** | Frame extract `p99 = 104.300 us`, `max = 129.800 us` (both `< 1000 us`); `digest_equal = true`, `bit_mismatches = 0` |
| Check 5 | Lazy Tier-0 -> Dense `Battle` (`500` bots, real `law::entity_contact::push_pair`) -> Form-P dissolution | **PASS** | `36,174` push pairs; `0.418 ms/tick` median (`0.640 ms` max); `oracle_mismatches = 0`, `dissolved_mismatches = 0`, `inventory = 32000/32000` |
| Mutants | `per-bot-index` (Check 2), `no-reabsorb` (Check 3), `dirty-observer` (Check 4) | **3 / 3 CAUGHT** | All 3 failed their target gates and exited with code `1` |

---

## 2. Live Wiring & Environment

- **Toolchain / Target:** `rustc 1.99.1`, `cargo 1.99.1`, `--release` build via `CARGO_TARGET_DIR = D:\path-tracing-target\es-1b-probe`.
- **Live project crates linked via `Cargo.toml`:**
  - `law` (`engine/law`) — `BoxWorld`, `PlayerState`, `try_tick`, `tick_with_collision`, `entity_contact::push_pair`, `Vec3`, `BlockPos`.
  - `plan` (`engine/plan`) — `plan::events::Map` (`advance`, `first_cross`).
- **Live ES sources compiled unmodified via `#[path = "..."]` in `src/main.rs`:**
  - `D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/bits.rs`
  - `D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/codec.rs` (`Enc`, `MAXB`, `MAXC`)
  - `D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/shelf.rs`
  - `D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/program.rs` (`Program`, `Prog`, `Ship`)
  - `D:/path-tracing-deps/emergent-space-engine/crates/universe/src/calendar.rs` (`Wheel`)
  - Minimal local shim `src/belt.rs` stubbing unused `BeltColumn` / `Belt::table_bytes()` / `base()` so `program.rs` builds without modifying ES.
- **Zero edits to `engine/`, `bot/`, `map/`, `D:\path-tracing-wt\`, or `D:\path-tracing-deps\emergent-space-engine\`.**

---

## 3. Raw Probe Measurements (`es-1b-probe --all`, exit code `0`)

All percentile distributions report every sample without outlier trimming (`n, min, median, p90, p99, max`).

### Calibration (`src/gait.rs`)
- Flat stone `26.2` sprint-jump calibrated from `law::tick_with_collision` (`BoxWorld::default()` with `difficulty = 0`):
  - `settle = 264 ticks`
  - `period = 12`
  - `total_dz = 4.276060`
  - `footprint_reads = 208 cells`
  - Verified bit-exact across 5,000 post-settle ticks against `law::try_tick` (`0` mismatches).

### Check 1 — Bit-Exact Oracle (`10^9` Bots)
- `streams = 1000`, `total_bots = 1000000000`, `sealed_scars = 100000` (`100` per stream)
- `samples = 1000` (stratified across streams, base runs, and scars, cross-checked against fresh `law::try_tick` rollouts)
- `mismatches = 0 => PASS`

### Check 2 — `O(1)` Space-Time Block-Edit Shock + Arrow + `WorldGait`
- `WorldGait` (4-tick clock in ES `Wheel` over `100,000` ticks undisturbed): `wheel wake-ups = 0 (expected 0)`
- Shock test per `N` (single block placed at `[4, 0, 200]` at `t_star`, stream holds `1,000` scars + `1` unsealed `hot` scar, flying arrow launched at `t_star - 5`, plus `WorldGait(0)`; per-trial measures `notify_edit` + `stream.seal()` + `first_cross()` + `stream.split()`):

| Ladder `N` | Index Bytes | Wake (`stream/bot`, `arrow`, `wg`) | `n` | `min` (us) | `median` (us) | `p90` (us) | `p99` (us) | `max` (us) |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| `1,000` | `11,431 B` | `1, 1, 1` | `100` | `151.800` | `163.200` | `173.000` | `202.700` | `204.500` |
| `100,000` | `11,431 B` | `1, 1, 1` | `100` | `153.500` | `159.000` | `172.100` | `193.500` | `193.700` |
| `10,000,000` | `11,431 B` | `1, 1, 1` | `100` | `169.600` | `172.000` | `177.400` | `233.300` | `245.700` |
| `1,000,000,000` | `11,431 B` | `1, 1, 1` | `100` | `165.100` | `172.100` | `180.200` | `193.900` | `198.200` |

- `log-log latency slope across ladder = 0.0038` (gate: `|slope| < 0.05` and `stream wakeups == 1`) `=> PASS`

### Check 3 — Scars, `seal()` Reabsorption, and `< 32 MB` L3 Residency
- Fixed scar budget `k = 100,000` (`1,000` streams):
  - `N = 1,000,000`: `0.703 MB` (`7.37 B/scar`, `0.737012 B/bot`)
  - `N = 10,000,000`: `0.772 MB` (`8.09 B/scar`, `0.080901 B/bot`)
  - `N = 100,000,000`: `0.894 MB` (`9.37 B/scar`, `0.009370 B/bot`)
  - `N = 1,000,000,000`: `0.886 MB` (`9.29 B/scar`, `0.000929 B/bot`)
  - Ratio `1B / 1M = 1.261x` across `1,000x` bot growth.
- Reabsorption at `N = 10^9`, `k = 1,000,000`:
  - Before 50% rejoin `seal()`: `4.282 MB` (`4.49 B/scar`, `0.0045 B/bot`)
  - After 50% rejoin `seal()`: `scars 1000000 -> 500000`, `bytes 4.282 MB -> 2.371 MB`
  - Per-stream `seal()` latency (`n = 1000`): `min = 74.500 us`, `median = 75.800 us`, `p90 = 77.000 us`, `p99 = 86.200 us`, `max = 120.000 us`
- `under_32MB = true`, `reabsorbed = true => PASS`

### Check 4 — Pure Observer Light-Cone Flyover (`10M`-Bot Highway, `500` Frames)
- Window: `192`-block frustum sweeping across `10,000,000` bots at `20` frames/sec (`dt = 1` tick/frame across `500` frames).
- Visible bots per frame (`n = 500`): `min = 5372.000`, `median = 5390.000`, `p90 = 5390.000`, `p99 = 5400.000`, `max = 5400.000`.
- Frame extraction latency (`n = 500`, gate `p99 < 1000 us` AND worst `max < 1000 us`):
  - `min = 37.600 us`
  - `median = 48.300 us`
  - `p90 = 55.300 us`
  - `p99 = 104.300 us`
  - `max = 129.800 us`
- Observer isolation: `bit_mismatches = 0`, `digest_equal = true => PASS`

### Check 5 — Dense Pit Promotion & Dissolution (`500` Bots, Real `law::entity_contact::push_pair`)
- `500` bots staggered across `50` Form-P streams, promoted from lazy Tier-0 to Tier-2 `Battle` at `promote_tick = 23`, stepped together for `60` dense ticks, then dissolved back to outgoing Tier-0 `Program` + `Ef` scars and compared against a brute-force reference from `t = 0`.
- `push_pairs_applied = 36174` (strict canonical `a < b` order).
- Dense `Battle` step (`n = 60`): `min = 0.364 ms/tick`, `median = 0.418 ms/tick`, `p90 = 0.497 ms/tick`, `p99 = 0.547 ms/tick`, `max = 0.640 ms/tick`.
- Working-set memory round-trip: `battle_bytes_during = 192000 B -> after_dissolve = 0 B`, `dissolved_FormP = 41500 B`.
- Exactness & conservation: `oracle_mismatches = 0`, `dissolved_mismatches = 0`, `inventory = 32000/32000 => PASS`.

---

## 4. Planted Mutants (`3 / 3` Caught)

| Mutant | Broken Assumption | Exit Code | Measured Failure |
| :--- | :--- | :---: | :--- |
| `--mutant per-bot-index` | Indexes per bot instead of per stream | `1` | Check 2 **FAIL**: at `N = 100,000`, `index = 5,254,151 B`, `wake stream/bot = 50,001`, log-log slope `= 0.1072` (also caught at `N = 1,000`: `501` wakes) |
| `--mutant no-reabsorb` | Writes zero-delta scars instead of matching base during `seal()` | `1` | Check 3 **FAIL**: `scars 1000000 -> 1000000`, `bytes 4.282 MB -> 3.694 MB`, `reabsorbed = false` |
| `--mutant dirty-observer` | Mutates stream/bot `hot` state during frame extraction | `1` | Check 4 **FAIL**: `digest_equal = false` (pre/post highway digest mismatch) |

---

## 5. ES Integration Findings & Friction Discovered

These are concrete facts observed directly in ES and `engine/law` source files while getting the probe running cleanly:

1. **`Enc::default()` panics on `n >= 4` (`crates/shipmem/src/codec.rs:62-72, 278-284, 336`):**
   - `Enc` derives `Default` (leaving `idx: Vec::new()`), whereas `Enc::new(chord)` allocates `idx: vec![[0; MAXB]; MAXC]`. Calling `Program::seal_from(..., &mut Enc::default())` panics with `index out of bounds: the len is 0 but the index is 0` on `n >= 4`. Calling `Enc::new(false)` resolves it.
   - **Actionable ES cleanup:** remove `#[derive(Default)]` on `Enc` or implement `Default` via `Self::new(false)`.

2. **`Program::hot` truncates bot ordinals to `u32` (`crates/shipmem/src/program.rs:369, 463, 494`):**
   - `hot: FxMap<u32, Ship>`, `eval` queries `(i as u32)`, and `update` inserts `(i as u32, s)`.
   - Within `10^9` bots per stream (`<= u32::MAX`) this works without truncation, **but** any single stream exceeding `4,294,967,295` silently aliases indices inside `hot`. Keeping streams under `~10^6` bots (as Check 1/3 do) or widening `hot`'s key to `u64` eliminates the cliff.

3. **`Program::split()` ignores unsealed `hot` entries and scans `O(k)` over `ef` (`crates/shipmem/src/program.rs:645-682`):**
   - Callers MUST run `stream.seal(&mut enc)` before `split(s, &mut enc)` or any unsealed `hot` edits are dropped across the cut.
   - Inside `split()`, scars are partitioned by scanning `self.ef.cursor(0)` over all `k` scars (`O(k)` rather than EF `rank(s)` slicing). With `k = 1,000` scars per stream (Check 2), `seal() + split()` stays under `~246 us` across `N = 10^3 -> 10^9`, though huge `k` within one stream will grow linearly with `k`.

4. **`belt.rs` workspace coupling (`crates/shipmem/src/belt.rs:9`):**
   - `use emergent_space::Key;` couples `emergent-shipmem` to `emergent-space` even when `Prog::Belt` is unused. Gate-keeping `belt` or factoring out `Key` allows lightweight `shipmem + calendar` linkage cleanly.

5. **Probe / physics integration traps caught during bringup:**
   - **Floating-point subtraction across binades:** subtracting two world positions after large `Z` motion loses ULP alignment relative to `law::try_tick`. Using `tick_with_collision`'s unrounded `actual.z` per-tick deltas (`Map::new`) kept all 1,000 stratified 1B-bot samples bit-exact (`0` mismatches).
   - **Swept projectile registration deduplication (`src/index.rs:157-200`):** registering `[t0, t0 + ticks]` per swept tick without deduplicating touched cells registered `[4, 0, 200]` twice (`arrow=2` wake-ups); deduplicating visited cells (`HashSet<BlockPos>`) keeps one clean entry per `(cell, Arrow)` (`arrow=1`).
   - **Sprint-jump impulse spacing:** on `26.2` flat sprint-jump, the jump impulse tick covers `> 0.6` blocks, so `CADENCE = 5` leaves the right-split head inside `< 0.3` blocks of `z_wall`. Setting `CADENCE = 6` (~`2.14` blocks/bot spacing) gives clean geometric wall clearance.
   - **World setup:** `BoxWorld.air` is private in `engine/law/src/world.rs`; worlds must be built via `BoxWorld::default()`, `difficulty = 0`, and `fill(...)`.

---

## 6. Probe Crate Layout (`jobs/es-1b-probe/`)

- `Cargo.toml` — standalone probe package depending only on `law` and `plan`.
- `src/belt.rs` — minimal stub for unused `Prog::Belt`.
- `src/gait.rs` — live `law` calibration (`26.2` flat-stone sprint-jump), `Map` evaluation, 5,000-tick self-check.
- `src/index.rs` — 4D sparse `SpaceTimeIndex` (`16^3` section + `64`-bit `4x4x4` subcell mask), `WorldGait` (`clock_4` + ES `Wheel`), swept `register_arrow_crossing`.
- `src/dense.rs` — Check 5 `Battle` promotion, `law::entity_contact::push_pair` dense stepping, and dissolution back to `Program`.
- `src/main.rs` — ES module wiring, Checks 1–4, mutant harness (`--all`, `--mutant none|per-bot-index|no-reabsorb|dirty-observer`).

## 7. Honest Scope Boundaries

This offline probe validates ES memory compression (`Form-P` + `Ef` scars), `O(1)` edit splitting across a `10^3 -> 10^9` ladder, `Wheel` stillness for periodic `WorldGait`, observer frustum extraction under `1 ms` (`p99` and `max`), and exact promotion/dissolution around `law::entity_contact::push_pair`. It does **not** test Minecraft wire packet serialization, full-chunk `.mca` world decoding, 20 Hz live server tick scheduling, or heterogeneous multi-word/multi-kit `(word, count)` chains.
