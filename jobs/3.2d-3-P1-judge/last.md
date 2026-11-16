VERDICT: PASS WITH FINDINGS

Judge: Opus 5.5, fresh context, 16 Nov. Job 3.2d-3-P1 (T1), commit 08ccc41 on `wt/3.2d-3-P1`, base `wt/3.2d-3`.
The premise holds on what was run: one prepared entry, memo hits, and an independent fresh `law::try_tick` replay from each
shifted start matches bit for bit. The gate passes. The hot-path counter cannot see a tick, though, and the key drops `position.y`.

## Findings (most severe first)

1. **Medium: the hot "zero ticks" assertion is true by construction, and a hot tick through the plain world goes unseen.**
   `t1.rs:114-126` is the hot window. Its code touches neither `counted` nor any `World`. `counted.ticks` only goes up
   where the test increments it by hand (`t1.rs:90`, `t1.rs:128`), so `hot_ticks` at `t1.rs:126/130` is always 0.
   The key derivation `label(&mut codec, &world, s)` (`t1.rs:111`) reads the floor column through the unwrapped
   `world`, outside both the counter and the clock. My mutations: M1, a hot `law::try_tick` through `&counted`, was CAUGHT
   by the reads counter (`hot world reads=798..884 hot_tick_calls=0`, which shows the ticks counter is decorative).
   M2, the same call through `&world`, SURVIVED (`test result: ok`).
   Smallest fix: move key derivation into the window, pass `&counted` to `label`, and assert the window's reads
   equal the label column's 4 reads. Drop the hand-incremented `ticks` field, or wrap the world handed to the hot code.
2. **Medium: `position.y` is left out of the recurrence key and is not translated when the entry is applied.** `motor()`
   skips `F::position` whole (`t1.rs:27`). The spec drops only x/z (report.md:133), and y is a read-before-write motor field.
   The applied state takes y from the prepared entry as an absolute value (`t1.rs:117-124` translate only t, x, z and the
   support block). A start on the same floor class at another height would hit the memo and come out with the wrong y.
   No start here tests that, because all nine share y. Fix: in `motor()`, push `s.position.y.to_bits()` when
   `F::position` is read. The alternative is to key on y relative to the support top and translate y on apply.
3. **Low: the nine starts are three, and only two of them differ from the preparation start.** All three tracks have
   the same A, `[300,-61,-480]` (`lab/tracks/straight{100,300,800}.json`), and the replay runs on the single `world` snapshot
   (`t1.rs:65`, `t1.rs:128`), so each track repeats the same starts. Start j=0 (t=0, x=300.5, full bars) is the
   preparation start itself (`t1.rs:83`), so those 3 comparisons check the entry against itself. t, x and resources
   change together for each j (`t1.rs:71-74`), and z never changes, although the spec asks for distinct "x/z" (report.md:132).
   M3, below, shows it: only start=1 and start=2 miss when t enters the key. Fix: shift z too, and at least one start
   should change only one of t, x/z or the bars. A track with a different A would also need its own `track.world`.
4. **Low: the negative control covers only `on_ground`** (`t1.rs:137-139`). The spec also names a changed floor `LabelId`
   (report.md:138). Fix: one lookup with a label derived from another floor class must miss.
5. **Low: values are hard-coded where the spec says to derive them.** `program_id = 30` (`t1.rs:79`) is right: in
   `words/src/program.rs:139-149` it is forward, sprint, jump always, sneak never. The yaw is a constant near 0°
   (`t1.rs:81`) and the yaw index is the literal `0u16` (`t1.rs:102,113`); neither comes from the track's bearing.
   Both sides of the key comparison use the same literals, so those slots compare trivially. `cycle_key`'s word indices
   8, 9 and 47-49 (`t1.rs:41-43`) match `state.rs:221-233` today but break silently if the layout changes. Fix: assert the
   program's name, compute the yaw index from `track.a`→`track.b`, and find the indices from the `bits()` schema.
6. **Info: the timing covers only `memo.get` + clone + translate.** Key derivation is excluded (`t1.rs:111-114`). My run:
   min 2000, median 2100, max 5900 ns, n=9, debug. The builder reports 4.9/5.2/5.7 µs. Both are debug builds over 9 samples of 3 distinct starts,
   so read the timing as an order of magnitude, not as a distribution.

What checks out: none of the forbidden paths changed. The diff touches only `engine/plan/Cargo.toml`,
`engine/plan/tests/zones_prereqs/*` and `jobs/3.2d-3-P1/*`. t2, t3 and t4 are empty. The reference replay is a fresh
`law::try_tick` from each observed start (`t1.rs:127-128`) and does not use the memo. Every listed state bit is compared:
x, y, z, vx, vy, vz, on_ground, sprinting, no_jump_delay, fall_distance, the shifted support and t (`t1.rs:133-135`).
The key leaves out t, x/z, the support block and the numeric resources, and includes the read fields plus the
food>6 and health>0 guards (`t1.rs:23-35`). The negative control is real: flipping `on_ground` changes the interned id.
Schema enforcement (A) is checked: native decodes, and recorded is refused with `unknown word` (`t1.rs:56-61`). The test
has no tolerances, epsilons or retries, and adds no search code.

## Commands run (CARGO_TARGET_DIR=D:\path-tracing-target\3.2d-3-P1-judge, Idle priority, offline)

- `git -C D:\path-tracing-wt\3.2d-3-P1 diff wt/3.2d-3..08ccc41 --stat`: 8 files, +231. No forbidden path.
- `cargo test --offline -p plan --test zones_prereqs t1_observed_transient -- --nocapture`: PASS. It printed
  `transient_ticks=263 cycle_period=12`, `prepared_entries=1 certificate_cells=23650 cycle_increment=(0.0,4.276060366589263)`,
  and `hot lookup+apply ns min=2000 median=2100 max=5900 n=9`.
- `cargo test --offline --workspace`: PASS. 49 suites, 210 passed, 0 failed, 19 ignored.
- M1, `law::try_tick` in the hot window via `&counted`: FAIL, as it should (9 failures on hot world reads; hot_tick_calls=0).
- M2, the same via `&world`: test PASSED, so the mutant survived (finding 1).
- M3, `out.push(s.t)` in `motor()`: FAIL, as it should (6 key mismatches, start=1 and start=2 on each track).
- Afterwards: `git checkout -- engine/plan/tests/zones_prereqs/t1.rs`. Line endings went back to LF, the blob equals
  HEAD (`f2252136`), and `git status` matches the start: only the two .mca files that were already modified and their .bak files.
- Not rerun: the footprint mutants (13/13) and the prerequisites verifier. This job does not touch their inputs.
