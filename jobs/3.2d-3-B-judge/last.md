# Judge verdict, 3.2d-3 slice B (R1), branch wt/3.2d-3-B, commit 564ace6

**PASS WITH FINDINGS.** No blocking constraint violation. Judged by Sonnet 5.5, fresh context, offline only (Idle priority, hidden).

## 1. Gates
- `cargo test --offline` from `engine/` (my own run, target judge-B): 200 passed, 18 ignored, 0 failed. This includes both `compile_fail` doctests on `Proof` (wrong-question cast, forged constructor). Matches the builder's 200/18.
- Footprint gate `jobs\1.2-footprints\verify.ps1`: not re-run by me, since it plants mutants in the worktree. The builder's retained log `D:\path-tracing-evidence\r1-codec-991db1497425453cb576337bbf779524\footprints.log` shows all 13 mutants CAUGHT and restored PASS, and `run.json` says PASS. `jobs/3.2d-3-B/verify.ps1` reads the mutant names from the pinned call sites by AST and requires each caught exactly once. Sound.
- **Scratch mutation (outside the tree):** I copied `engine/` to my scratchpad and made `Interner::find_in` take the first bucket entry without comparing bytes (same for labels). `cargo test -p law --lib` then fails `codec::tests::collision_compares_bytes` (15 passed, 1 failed). The collision tests use `Interner::with_digest(|_| 0)`, so the byte comparison does the work. Not tautological.
- Tests check the claimed things:
  - Forced collision in all four consumers: footprints, pattern keys and presence (`law/tests/codec.rs::forced_collision...`: `Footprint::answers`, `PatternTable` insert/lookup, `law_digest`), and proposal classes (`words/tests/codec.rs`).
  - Air, buttons, plates and a flower are equal under legs, through the real registry (`world/tests/codec.rs`).
  - A distant cobweb is harmless; an in-tube cobweb makes the certificate answer No (a real native `RecordingWorld::tick` footprint).
  - Verb Unknown retention, kit distinctions and label distinctions (`labels_kits...`).
  - Wrong state, version, question and foreign owner refuse; same-content, unrelated-tick and out-of-coverage edits behave exactly.
  - Both un-evented reads (wall release, sneak back-off) include a positive control that native movement changed.
  - Plane query refuses missing coverage and loaded bits.

## 2. Constraint violations: none blocking
- No tolerance, epsilon, retry, made-up number, new search code or new crate. `movement.rs` is untouched (not in the diff). `bot/`, `state.rs`, `flight.rs` and `plan/bin` are untouched. B also touches `gait.rs`, `segment.rs`, `reach.rs` and `mining.rs` (exposes the progress query). These are consumers the spec names; the mining change leaves the arithmetic alone.
- No fifth encoding: pattern.rs keeps the single field encoder (`encode`, `encode_shape`) and the codec wraps it. `class.rs` now delegates to `codec::state_answers` (the old body moved to `codec_state.rs`).
- No digest accepted without byte equality: `Interner` buckets by digest and compares bytes; `law_digest` and `cell_digest` return `Canonical` (PartialEq over bytes); `reach.rs` premises hold `Canonical`, and an unsupported answer (`None`) cannot establish a premise. `u128` remains only as `fingerprint()` and `digests()`, informational.
- Legs omissions are exactly section 2's (`legs_model`: world_hook always; air only when Inside::None and stuck_multiplier absent; fluid_flow only when fluid is empty). Role masks match.
- `unsafe` SSE2 intrinsics in `codec_planes.rs`: allowed by the spec ("single-thread SIMD").

## 3. Findings, ranked
1. **(Moderate) A likely hot-path performance regression, unmeasured.** `PatternTable::cell_id` used to cache by (model id, mask, loaded) (`by_model`). That is removed, and `Memo::bytes` now clones a `Vec<u8>` per cell lookup, call-local only. `reach.rs` premises changed from `u128` to boxed bytes per premise cell. The report makes no performance claim and the `read_cost` measurement tests are `ignored`. Before merge, run the release `read_cost` and pattern measurements (AGENTS: measure the build). Not blocking for correctness.
2. **(Moderate) Shared 16-bit class space.** `ClassId` is `u16` with `TableFull` at 65535. State classes (`start_ids`, `ProposalTable.ids`) now intern in the same `Interner` as cell and environment answers, where before start classes lived in an unbounded `HashMap<Box<[u64]>, u32>`. A long run or a large proposal table can now hit `TableFull` earlier. Slice C should decide whether state keys get their own id space.
3. **(Minor) Ids are interner-local**, not persistent fingerprints (the report says so). The owner checks (`Arc::ptr_eq`) on `ProjectionTable` and `ReadCertificates` refuse a foreign owner, which is correct; anything persisted across processes must use the bytes.
4. **(Minor) `SnapshotSource` ignores the tick when reading a cell.** `edit` re-reads the current content for a read recorded at an earlier tick. The doc says it is an explicitly qualified immutable adapter; the resident source must implement `Source` with real tick semantics. C or E must not use it as the live source.
5. **(Minor) `Interner::with_digest` and `digests()` are public test hooks.** Harmless, but they expose the bucket function.
6. **(Minor) Leftover `fnv` wrapper in pattern.rs** delegates to `codec::digest` and is probably dead. My build showed 5 compiler warnings; none was about the new code that I checked.

## 4. Spec departures
- Break and Place projections return `Unknown` retaining registry, fine-state, kit and verb identity; only `mining_progress` is Known. Prompt decision 3 and spec section 2. Sound.
- `ProjectionTable` caches by fine ID with the model passed in by the caller and holds no terrain. Sound. A model-table ID is not a fine ID, and the report states it.
- Section role planes are standalone (`SectionRoles::from_classes`), not yet fed by the resident source (A). Deferred to C/E, per the spec's ordering.
- The proof spine has one real question (`ReadsHold`); `TubeEmpty` exists only to prove non-interchangeability. The `Answer<Q>` shape is satisfied; the tube-empty claim itself is not built. Deferred.
- "Check the full 4D worldtube ONCE when made": `certify_reads` records native per-tick read sets (`timed_reads`), not a separate arc certificate. Sound for this slice.

## 5. Merge risk
Disjoint from A by file name (empty intersection). `git merge-tree --write-tree wt/3.2d-3-A wt/3.2d-3-B` is clean, and each is clean onto main 031cbaa. B changes `law::pattern` public signatures (`cell_digest` returns `Canonical`; `Start` gains `question`). A's native crate uses neither. The existing `law/tests/footprints.rs` and `world/tests/blocks.rs` compare `cell_digest` results with `!=` and still compile (my test run). The same pre-existing `.mca` fixture dirt is in the worktree; do not commit it.

## 6. Net lines
+1161/-165 over 20 files: about 650 are new codec sources, about 280 tests, about 110 gate and report. About 160 removed lines are the old `class.rs` body (moved to `codec_state.rs`) and the old digest plumbing. Bought: the one codec the spec requires, the forced-collision gate and the proof spine, closing plain-path judge finding 4. The commit needs a `Bought:` line.
