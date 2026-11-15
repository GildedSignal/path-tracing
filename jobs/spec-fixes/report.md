# `3.2d-3` Spec Fixes & P1–P4 Job Specs (Post-Slice A & B Merge on `wt/3.2d-3`)

**Consumer:** Terra's next jobs on `D:\path-tracing-wt\3.2d-3` (`wt/3.2d-3`): P1 (T1), then P2–P4, then slices C–K.  
**Basis:** `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-3\spec.md` (141 lines), `C:\Users\PASCAL\Desktop\path tracing\jobs\spec-premises\report.md` (35 audited claims: 10 `EXISTS`, 17 `DIFFERS`, 8 `MISSING`), `wt/3.2d-3` code after merging Slice A (`eb3632a`) and Slice B (`564ace6`), their judge verdicts, and `D:\path-tracing-wt\plain\engine\plan\src\plain.rs`.

---

## 1. Status of the 25 `DIFFERS` & `MISSING` Claims on `wt/3.2d-3`

Summary after Slices A and B:
- **Satisfied by A or B:** **9** (rows 3, 6*, 7, 10, 11, 14, 18, 20, 29; *row 6 has one minor stale CLI message noted below)
- **Partially satisfied / still open for later slices:** **8** (rows 5, 8, 13, 15, 17, 19, 28, 31)
- **Still differs / still missing:** **8** (rows 2, 12, 22, 24, 25, 26, 27, 34)
- **Moot:** **0**

### Audit Resolution Table

| Audit Row | Spec Line & Claim | Status on `wt/3.2d-3` | Evidence on `wt/3.2d-3` | Unblocks |
|---|---|---|---|---|
| **2** (`DIFFERS`) | L4: `wt/plain-path is unmerged... Read plain.rs` | **Still differs** | `plain.rs` lives at `D:\path-tracing-wt\plain\engine\plan\src\plain.rs:1-404` (`wt/plain`, not `wt/plain-path`). `D:\path-tracing-wt\3.2d-3\engine\plan\src\lib.rs:1-18` has no `plain` module. | P1, C, E, J |
| **3** (`MISSING`) | L10: `Reuse the clock/comparator native module` | **Satisfied by A** (native JNI+QPC); comparator extraction remains F (row 27) | `D:\path-tracing-wt\3.2d-3\engine\native\Cargo.toml:1-13` (`cdylib`+`rlib`), `engine/native/src/lib.rs:9-132` (JNI + `qpc()`). | A (done), F, K |
| **5** (`DIFFERS`) | L11, L24, L91, L115: `only the existing F CPU worker` / `existing F` | **Partially satisfied by A** | A added background worker `"pathtracing-F"` (`engine/native/src/resident.rs:125`), but `Planner` (`engine/plan/src/search.rs:115-170`) is still synchronous inside that thread, and unreceived cells panic into `catch_unwind` (`resident.rs:40-49, 136-137`) instead of typed `Unknown`. | F, H |
| **6** (`DIFFERS`) | L12, L108: `existing bit schema/recorders` & `from_bits` inverse | **Satisfied by A** (with 2 notes) | `engine/law/src/state.rs:284-292` (`from_observed_bits`), `bot/src/pathtracing/StateBits.java:9-68` (`bits`, `known`, `provenance`), `PlayerSnapshot.java:16-55`, `engine/native/src/lib.rs:38-44, 58-63`. Stale CLI string remains at `engine/plan/src/bin/plan-track.rs:23`; remote live `exhaustion`/`foodTickTimer`/`natural_regeneration` remain unknown until K (`PlayerSnapshot.java:51-52`, `ResidentSource.java:22-23`). | P1, P4, F, J, K |
| **7** (`DIFFERS`) | L14: `Near data comes from the existing client packet hooks as new-section/palette and block deltas` | **Satisfied by A** | `bot/src/pathtracing/mixin/ClientPacketListenerMixin.java:118-140` hooks `handleBlockUpdate`, `handleChunkBlocksUpdate`, `handleLevelChunkWithLight`, `handleForgetLevelChunk` into `ResidentSource.{block,chunk,unload}` (`ResidentSource.java:30-71`) -> `PublishSourceDelta` (`resident.rs:76-114`). (Client-thread string cost carried by C/K, §4.) | A (done), G |
| **8** (`DIFFERS`) | L14: `Retain one versioned lazy source adapter, not a dense snapshot rebuilt per goal` | **Partially satisfied by A; open for B-`Source`/far adapter** | `resident::Source` (`engine/native/src/resident.rs:17-52`) is versioned (`session`, `revision`) and lazy per received section, **but** it implements only `law::World`, not `law::codec::proof::Source` (`codec_proof.rs:12-15`) with tick/coverage semantics, and no far snapshot/generator adapter is wired yet. `WorldSnapshot` (`snapshot.rs`) & `WorldReader` (`region.rs`) unchanged. | C, E, G |
| **10** (`MISSING`) | L17: `existing same-host QPC clock contract and actual server tick-start samples` | **Satisfied by A** | `engine/native/src/lib.rs:9-16, 113-132` (`qpc`/`clock`), `bot/src/pathtracing/HopCapture.java:15-89`, `HopServerMixin.java:11-17` (`server-tick-start`, `server-resources`), `HopInputMixin.java:12-21`, `HopRconMixin.java:10-14`, `jobs/3.2d-3/check-hop-clock.ps1`. | A (done), K |
| **11** (`DIFFERS`) | L17: `Tag real input sequences on the existing connection` | **Satisfied by A** (at connection packet layer) | `HopCapture.packet` (`HopCapture.java:33-46`) tags every connection packet `pair:seq:type` without marker packets and correlates `applied(packet)` (`:47-51`) via `HopInputMixin.java:12-21`. `TickInput.java:6-7` itself remains 9 fields (sufficient since `HopCapture` tags the `Packet<?>` on `Connection`). | A (done), F, K |
| **12** (`DIFFERS`) | L19, L112: `remove the plain remainder/script try_tick loops ... Poisoning the legacy script writer` | **Still differs** | `plain.rs` (`settle`, `Goal::touched` loop, `script_start`/`script_row`) is only on unmerged `wt/plain`. On `wt/3.2d-3`, runtime/legacy loops are `flight::write_script` (`flight.rs:59-79`), `walk::certify` (`walk.rs:75-111`), `polish::slide`, and A's temporary `legacy_sentence_adapter` (`resident.rs:195-204`). | E, F, J |
| **13** (`DIFFERS`) | L20: `No EPSILON*16, half-ulp allowance or fed resource state authorizes motion` | **Partially differs** | `half_ulp` (`plain.rs:332`) is not on `wt/3.2d-3`, **but** `words::gait::fed` (`engine/words/src/gait.rs:45, 72-77`) and `search::fed` (`engine/plan/src/search.rs:132-134`) still exist on `wt/3.2d-3`. | P1, P4, C, D, E, J |
| **14** (`DIFFERS`) | L28: `replaces independent encoding/equality in footprints, pattern keys, words::class/ProposalTable and law_digest` | **Satisfied by B** | `codec::Interner` (`codec.rs:180-255`), `Footprint::answers` (`footprint.rs:51-64`), `PatternTable` + `law_digest`/`cell_digest` returning `Canonical` (`pattern.rs:72-84, 156-294`), `words::class` (`class.rs:23-25`), `ProposalTable` (`table.rs:143-163`). | B (done), C, J |
| **15** (`DIFFERS`) | L29: `Keep fine block-state identity plus registry hash in the source, not only the 518-model legs projection` | **Partially satisfied by A+B** | A retains fine block-state strings in `ResidentSource`/`resident.rs:26`; B adds `FineId(u32)` & `ProjectionTable` (`codec.rs:137-170`). **Still missing:** registry hash in `ResidentSource`/world join delta/`BlockDumper.java`, compact `FineId` assignment (instead of per-block string sorting), and fine IDs in `WorldSnapshot` (`snapshot.rs:1-50`). | C, E, F, K |
| **17** (`MISSING`) | L32, L109: `existing verb's hardness/tool-time/harvest and drop-generator ... place replaceability/face/legality` | **Scoped by B; full Break/Place unported** | B implements `UnportedVerb::{Break, Place}` returning `Projection::Unknown` (`codec.rs:113-125, 218-220`) and `Interner::mining_progress` using `law::mining::MiningSample` (`codec.rs:223-232`, `mining.rs:12-27`). Harvest/drop/silk-touch/place tables do not exist in `engine/` or `BlockDumper.java:76-77`. | C–K |
| **18** (`MISSING`) | L37, L137: `Section role planes remain the existing leaf presence interface: AllZero/AllOne/Words` | **Satisfied by B** (standalone) | `SectionRoles` + `Plane::{AllZero, AllOne, Words}` + `complete`/`loaded` masks + SSE2 `masked_pair` (`engine/law/src/codec_planes.rs:7-93`). Not yet connected to `resident::Source` (C/E wire it). | C, E, R3 |
| **19** (`DIFFERS`) | L38, L115: `using the existing arc/read certificate on its qualified domain; reuse without tick scans` | **Partially satisfied by B** | B built `ReadCertificates`/`ReadsHold` (`codec_proof.rs:95-183`) and `Segment::certify_reads` (`segment.rs:52-57`). **Still missing:** `arith::free_tick` (`arith.rs:32-55`) and `walk::certify` (`walk.rs:75-111`) still scan ticks/cells; `TubeEmpty` / 4D arc emptiness certificate over `SectionRoles` is not built yet. | C, H |
| **20** (`MISSING`) | L38: `Edits notify its reverse footprint index` | **Satisfied by B** | `ReadCertificates` maintains `cells` & `environment` reverse indices (`codec_proof.rs:99-100`) and `edit`/`environment_event` (`:150-182`). | B (done), G |
| **22** (`DIFFERS`) | L47: `coverage of the law's yaw-index domain` | **Still differs** | `words::program::Arguments::yaw_index: u8` has 64 bins (`program.rs:85, 97`), while `law::math::{sin, cos}` indexes 65536 `sin.bin` bins (`math.rs:1-17`); `Arguments::refined_yaw` (`program.rs:86`) carries arbitrary `f32` yaws as used by `plain.rs:180-182`. | P3, C, D |
| **24** (`DIFFERS`) | L63, L72: `law's existing difficulty/body/resource guard... Its food restriction is void on Peaceful by that same law` | **Still differs** | `movement.rs:1255, 1276` (`server_regeneration`, `food_tick`) check `world.difficulty()`, **but** `update_sprinting` (`movement.rs:219, 225`) and `codec_state.rs:17` check `food_level > 6` unconditionally without checking `difficulty`; `law::tick` never calls `server_regeneration`. | P4, D, E, F |
| **25** (`DIFFERS`) | L69: `reproduce cargo test --offline -p plan --test plain straight` | **Still differs** | `plain.rs` & `tests/plain.rs` live only on `D:\path-tracing-wt\plain\engine` (`wt/plain`), not `wt/3.2d-3`. `zones_prereqs` does not exist yet on `wt/3.2d-3`. | P1 |
| **26** (`DIFFERS`) | L78: `The mod executes the existing word predicates/keys using Minecraft's own state/functions` | **Still differs** | `bot/src/pathtracing/` has no Java `Program`/`Predicate` executor (`ResidentRunner.java:77-84` uses `legacy_sentence_adapter`); `words::recurrence` (`gait.rs:55-64`) still encodes all `PlayerState::bits()` except `t`, `position`, and relative `main_supporting_block`, not read-before-write motor bits. | C, F |
| **27** (`DIFFERS`) | L79: `Extract compare_event from plan::deviation...` | **Still differs** | `engine/plan/src/deviation.rs:1-80` was untouched by A/B; only offline JSONL `compare` (`:35-37`) and pre-1.4b `at` offset shim (`:58`) exist. | F |
| **28** (`DIFFERS`) | L80, L113: `versioned ownership manifest... echoed sprint/pose/swim/frozen... food packet` | **Partially satisfied by A** | A added `ClientboundSetHealthPacket` hook (`ClientPacketListenerMixin.java:36-37`), `HopCapture.resources` (`HopCapture.java:60-72`), and `provenance` mask (`StateBits.java:10-13`). **Still missing:** versioned `StateField` ownership manifest (`deviation.rs:30` still uses `NEVER`), and remote clients still lack server `exhaustion`/`food_tick_timer`/`natural_regeneration` (`PlayerSnapshot.java:51-52`). | F, K |
| **29** (`MISSING`) | L88: `Spine remains Answer<Q>=Yes/No(private question-bound proof)\|Unknown(refusal)` | **Satisfied by B** | `engine/law/src/codec_proof.rs:60-86` (`Answer<Q>`, private `Proof<Q>`, `ReadsHold`, `TubeEmpty`, `compile_fail` doctests). (`TubeEmpty` producer deferred to C/H.) | B (done), H |
| **31** (`MISSING`) | L108, L119, L125–131: `verify.ps1`, `ObserverOnly`, `frozen baseline bundle` | **Partially satisfied by A** | A created `jobs/3.2d-3/verify.ps1:1-66`, `build-observer.ps1`, `observe-baseline.ps1`, `freeze-baseline.ps1`, `freeze-inputs.ps1`, `check-hop-clock.ps1`, `jobs/3.2d-3/observer/*.java`. **Still open:** `Prerequisites` manifests (written by P1–P4), `-Case T4Food` (P4), live `CaptureBaseline` execution (`verify.ps1:17-20` throws until K pairs observer-only with frozen baseline), and live gates (K). | P1–P4, C–K |
| **34** (`MISSING`) | L132: `Use jobs/3.2d-3-oracle's separately produced other-family executable/manifest` | **Still missing** | `jobs/3.2d-3-oracle` does not exist on `wt/3.2d-3` or `main`. | K |

### Exact Smallest Spec Replacement Text for Still-Open Rows

- **Row 2 (L4) — unblocks P1, C, E, J:**  
  Replace `wt/plain-path is unmerged and judged FAIL, not a certified foundation. Read plain.rs hash ...` with:  
  > `D:/path-tracing-wt/plain` (`wt/plain`, `plain.rs` hash `589708E6...`, `movement.rs` hash `F68E7FB4...`) is unmerged and judged FAIL, not on `wt/3.2d-3`. Read it as a reference baseline only; do not depend on `plan::plain` from `wt/3.2d-3` until Slice E builds the new adapter.

- **Row 5 (L11, L24, L91, L115) — unblocks F, H:**  
  Replace `only the existing F CPU worker` / `existing F` with:  
  > only the resident `"pathtracing-F"` worker (`engine/native/src/resident.rs:125`) hosting synchronous `Planner` (`engine/plan/src/search.rs`); C/E/H replace panic-as-Unknown (`resident.rs:40-49, 136`) with typed `Unknown` refusals, and never run search on the client thread.

- **Row 6 (L12, L108) — unblocks P1, P4, F, J, K:**  
  Append to L12:  
  > `PlayerState::from_observed_bits` (`engine/law/src/state.rs:284-292`) and `StateBits.java` implement the shared bit/known/provenance schema; remote stock clients leave server `exhaustion`, `food_tick_timer`, and `natural_regeneration` Unknown (`PlayerSnapshot.java:51-52`, `ResidentSource.java:22-23`) until Slice K supplies authoritative server resource capture/sync. Update or retire `plan-track.rs:23`'s stale error text in J.

- **Row 8 (L14–15) — unblocks C, E, G:**  
  Replace `Retain one versioned lazy source adapter, not a dense snapshot rebuilt per goal` with:  
  > Extend A's versioned client-near `Source` (`engine/native/src/resident.rs:17-52`) to implement B's `law::codec::proof::Source` (`engine/law/src/codec_proof.rs:12-15`) with real tick/coverage/loaded semantics, plus an optional pinned immutable far snapshot or qualified generator+scars adapter (never use `SnapshotSource` for live mutable worlds).

- **Row 12 (L19, L112) — unblocks E, F, J:**  
  Replace L19's `remove the plain remainder/script try_tick loops and final x/z audit...` with:  
  > Build the new `plan::plain` path cleanly on `wt/3.2d-3` without importing `wt/plain`'s `settle`, `Goal::touched` loop, `script_start`/`script_row`, or final `x/z` audit; remove A's `legacy_sentence_adapter` (`resident.rs:195-204`) in F and retire `flight::write_script` (`flight.rs:59-79`), `walk::certify` (`walk.rs:75-111`), and `polish::slide` plain loops in E/J so poisoning `write_script` alters no runtime answer.

- **Row 13 (L20) — unblocks P1, P4, C, D, E, J:**  
  Append to L20:  
  > Retire `words::gait::fed` (`engine/words/src/gait.rs:45, 72-77`) and `search::fed` (`engine/plan/src/search.rs:132-134`); neither `half_ulp` (`wt/plain`) nor `fed` may authorize plain or store motion.

- **Row 15 (L29) — unblocks C, E, F, K:**  
  Replace `Keep fine block-state identity plus registry hash in the source, not only the 518-model legs projection` with:  
  > Pin a 26.2 block-state registry hash in `ResidentSource` / world-join delta (`resident.rs:79-85`), assign compact `FineId(u32)` (`codec.rs:137`) per canonical state once instead of sorting/building strings per block per chunk (`ResidentSource.java:37-42, 72-75`), and treat `WorldSnapshot` (`snapshot.rs`) solely as a 518-model offline fixture adapter.

- **Row 17 (L32, L109) — unblocks C–K:**  
  Replace L32 with:  
  > Only the already-ported `mining::MiningSample::progress` (`mining.rs:19-27`, `codec.rs:223-232`) is `Projection::Known`; `UnportedVerb::{Break, Place}` return `Projection::Unknown` retaining `(registry_hash, fine_state, kit)` (`codec.rs:113-125, 218-220`) until native break/harvest/drop/silk-touch and place legality/debt tables are ported in a later phase.

- **Row 19 (L38, L115) — unblocks C, H:**  
  Replace `using the existing arc/read certificate on its qualified domain; reuse without tick scans` with:  
  > using `codec::proof::ReadCertificates` (`codec_proof.rs:95-183`) for native read sets and building a 4D worldtube/arc `TubeEmpty` certificate over `SectionRoles` (`codec_planes.rs:33-72`) in C/H to replace `arith::free_tick`'s (`arith.rs:32-55`) per-tick cell scan on its qualified domain.

- **Row 22 (L47, L55, L71, L127) — unblocks P3, C, D:**  
  Replace `coverage of the law's yaw-index domain` (L47) and clarify L55/L71:  
  > The any-bearing native domain is the 65536-entry `law::math::{sin, cos}` `sin.bin` index `0..65536` (`engine/law/src/math.rs:1-17`, passed via `Arguments::refined_yaw` in `words/src/program.rs:86`, distinct from the 64-bin `Arguments::yaw_index: u8` subset used by `tiny_programs`). Ties select the lowest `0..65536` index.

- **Row 24 (L63, L72) — unblocks P4, D, E, F:**  
  Replace `Its food restriction is void on Peaceful by that same law` (L63) and `its food restriction stays void` (L72) with:  
  > Food drain is void on Peaceful by `food_tick` (`movement.rs:1276`: `else if world.difficulty() != 0`), while the unchanged sprint guard `food_level > 6` (`movement.rs:219, 225`; `codec_state.rs:17`) remains active in the law and evaluates actual observed bars (blocking sprinting if observed `food_level <= 6`, with no fabricated bars, no `fed` substitution, and no food-drain debit check).

- **Row 25 (L69) — unblocks P1:**  
  Replace L69's test command cell with the complete P1 job spec in §2 below (reference baseline on `wt/plain` read-only; new `zones_prereqs` target on `wt/3.2d-3`).

- **Row 26 (L78) — unblocks C, F:**  
  Replace `The mod executes the existing word predicates/keys using Minecraft's own state/functions` with:  
  > Slice F builds a Java `Program`/`Predicate` executor mirroring `words::Predicate` (`engine/words/src/program.rs:4-19`) over Minecraft's `LocalPlayer`/world state, and Slice C replaces `words::recurrence` (`engine/words/src/gait.rs:55-64`, currently all non-t/pos fields) with read-before-write motor bits + phase.

- **Row 27 (L79) — unblocks F:**  
  Append to L79:  
  > Extract `compare_event` in Slice F from `engine/plan/src/deviation.rs:1-80` and delete its pre-1.4b `at` offset shim (`deviation.rs:58`), matching `PlayerState::from_bits` / `StateField` word order directly.

- **Row 28 (L80, L113) — unblocks F, K:**  
  Append to L80:  
  > Slice F creates the versioned `StateField` ownership manifest in Rust (replacing `deviation.rs:30`'s `NEVER` list for event equality while keeping `provenance`); `ClientboundSetHealthPacket` is already hooked (`ClientPacketListenerMixin.java:36-37`), and Slice K supplies authoritative server `exhaustion`, `food_tick_timer`, and `natural_regeneration` rows on live runs.

- **Row 31 (L108, L119, L125–131) — unblocks P1–P4, C–K:**  
  Note on L119:  
  > A built the `jobs/3.2d-3/verify.ps1` skeleton (`verify.ps1:1-66`) and `observer-only.jar` (`build-observer.ps1`). Each premise job P1–P4 writes `$Evidence/<Test>/manifest.json` matching `verify.ps1:21-30`; P4 implements `-Case T4Food`; K implements `CaptureBaseline` (pairing `observer-only.jar` with the frozen baseline binary) and the remaining live cases.

- **Row 34 (L132) — unblocks K:**  
  Add to §7 / L132:  
  > Schedule `jobs/3.2d-3-oracle` (other-family executable and manifest) as a required dependency before Slice K starts.

---

## 2. Decision-Free Job Spec for P1 (T1) on `wt/3.2d-3`

- **Worktree & branch:** `D:\path-tracing-wt\3.2d-3` (branch `wt/3.2d-3`, after A+B merge). Reference-only read of `D:\path-tracing-wt\plain\engine` (`wt/plain`, unmerged; do NOT copy `plain.rs` or add `pub mod plain` to `engine/plan/src/lib.rs`).
- **Inputs (all on `wt/3.2d-3` unless marked reference):**
  1. `D:\path-tracing-wt\plain\engine\plan\tests\plain.rs` (`straight100`, `straight300`, `straight800`, read-only reference).
  2. `D:\path-tracing-wt\3.2d-3\engine\world\tests\fixtures\tracks.txt:19-21` (`straight100`, `straight300`, `straight800`) and `engine/world/tests/fixtures/world` (Peaceful `difficulty = 0`, mobs off).
  3. `D:\path-tracing-wt\3.2d-3\engine\law\src\state.rs:202-292` (`PlayerState::bits`, `PlayerState::from_observed_bits`), `bot/tests/fixtures/native-state.hex`, `bot/tests/fixtures/recorded-state.hex`.
  4. `D:\path-tracing-wt\3.2d-3\engine\law\src\codec.rs` (`Interner`, `LegsRole`, `Label`, `StateQuestion::Recurrence`), `engine/law/src/codec_proof.rs` (`ReadCertificates`, `SnapshotSource`, `Binding`), `engine/law/src/footprint.rs` (`RecordingWorld`), `engine/law/src/movement.rs` (`law::try_tick`).
  5. `D:\path-tracing-wt\3.2d-3\engine\words\src\program.rs` (`tiny_programs`, `Arguments`), `engine/plan/src/walk.rs:43-54` (`settle` used ONLY to seed valid ground-standing fixture coordinates before `from_observed_bits`), `engine/plan/src/flight.rs:12-40` (`track`), `engine/plan/src/goal.rs` (`Goal`).
- **Exact files to create:**
  - Create test harness `D:\path-tracing-wt\3.2d-3\engine\plan\tests\zones_prereqs\main.rs` containing `mod t1;` (and empty placeholder modules `mod t2;`, `mod t3;`, `mod t4;` in `t2.rs`, `t3.rs`, `t4.rs` so P2 and P4 can later run in parallel without editing the same file) plus `D:\path-tracing-wt\3.2d-3\engine\plan\tests\zones_prereqs\t1.rs`.
  - Register `[[test]]` in `D:\path-tracing-wt\3.2d-3\engine\plan\Cargo.toml` with `name = "zones_prereqs"` and `path = "tests/zones_prereqs/main.rs"` (preserving `cargo test --offline -p plan --test zones_prereqs <fn>`).
  - Create `D:\path-tracing-wt\3.2d-3\jobs\3.2d-3-P1\verify.ps1` and `last.md` (with `CARGO_TARGET_DIR=D:\path-tracing-target\3.2d-3-P1` and evidence under `D:\path-tracing-evidence\3.2d-3-P1`).
- **Exact test `t1_observed_transient` to write in `t1.rs`:**
  1. **Observed-start schema enforcement (A):** Decode `bot/tests/fixtures/native-state.hex` via `PlayerState::from_observed_bits` with all-true mask (must succeed). Decode `bot/tests/fixtures/recorded-state.hex` via `PlayerState::from_observed_bits` with its line-2 mask (must fail with `Refusal { reason: "unknown word", .. }`).
  2. **Fixture starts on Peaceful:** Load `straight100`, `straight300`, `straight800` via `plan::flight::track`. Generate settled starting geometry on each track's A cell, then construct multiple observed `PlayerState` starts through `PlayerState::bits()` + `PlayerState::from_observed_bits(&bits, &vec![true; bits.len()])` with: (a) distinct measured entry ticks `t` (e.g., `0, 77, 1543`), (b) distinct observed `x/z` positions inside the same binary64 binade on the stone floor (`300.5` vs `304.5` vs `315.25`), and (c) distinct valid Peaceful resource bars (`food_level = 20, saturation = 20.0, exhaustion = 0.0` vs `food_level = 17, saturation = 5.0, exhaustion = 0.25`, `health = 20.0`). **Never** call `words::gait::fed` or `search::fed`.
  3. **Recurrence key & `LabelId` preparation (B):** Derive the floor `LabelId` via `codec::Interner::label` from the column's 4 `LegsRole` `ClassId`s (`Support02`, `Support05`, `BodyFeet`, `Inside`) + `body_kit` + `codec::SCHEMA`. Key the prepared transient entry on `(LabelId, program_id, yaw_index, pitch_bits, motor_recurrence_id)` where `motor_recurrence_id` is interned via `codec::state_key(StateQuestion::Recurrence, &motor_words)` containing ONLY read-before-write motor fields + resource guard bits (`food_level > 6`, `health > 0.0`), excluding `t`, `x/z` position, relative `main_supporting_block`, and numeric resource values.
  4. **One-time native preparation & zero-tick hot application:** Wrap the `World` in a tick-counting wrapper (`AtomicUsize` incremented on every `World::block`/`read_state` inside `law::try_tick`). Prepare the transient for `straight100` (sprint-jump program `F S j a s n` at the track's bearing) ONCE off-command via `law::try_tick` + `RecordingWorld`, recording: `transient_ticks`, `cycle_period`, ending motor `PlayerState` template, cycle increments, and `ReadCertificates` capture. Then look up and apply the prepared transient for `straight300`, `straight800`, and shifted `(t, x, z, resources)` starts:
     - Assert **exact recurrence-key & `LabelId` equality** across those clicks/positions/ticks/resource bars (100% memo hit; `prepared_entries == 1`).
     - Assert **zero** `law::try_tick` calls during hot lookup and application (`hot_tick_calls == 0`).
     - Assert **bit-exact** motor state bits (`position.y.to_bits()`, `velocity.{x,y,z}.to_bits()`, `on_ground`, `sprinting`, `no_jump_delay`, `fall_distance`, `main_supporting_block` shifted by anchor `dx, dz`, `t == entry_t + transient_ticks`, and `position.{x,z}.to_bits()`) against a fresh reference `law::try_tick` replay from each observed start (using the same binade anchor).
     - Assert a negative control: changing a motor bit (`on_ground = false` or `sprinting = true`) or floor `LabelId` misses the memo and does not reuse the entry.
  5. **Baseline reference check (read-only):** Run `cargo test --offline -p plan --test plain straight -- --nocapture` in `D:\path-tracing-wt\plain\engine` and log its output as unmerged baseline evidence (not merged into `wt/3.2d-3`).
- **Gate:**
  - `cargo test --offline -p plan --test zones_prereqs t1_observed_transient -- --nocapture` (in `D:\path-tracing-wt\3.2d-3\engine`, PASS).
  - `cargo test --offline --workspace` (in `D:\path-tracing-wt\3.2d-3\engine`, PASS).
  - `& .\jobs\1.2-footprints\verify.ps1` (13/13 mutants CAUGHT, sources restored).
  - Write `D:\path-tracing-evidence\3.2d-3\T1\manifest.json` (`passed: true, difficulty: "Peaceful", mobs: "Off"`, file hashes) AND verify `.\jobs\3.2d-3\verify.ps1 -Case Prerequisites -Tests T1 -Difficulty Peaceful -Mobs Off -Evidence D:\path-tracing-evidence\3.2d-3` exits 0.
- **Must NOT touch:** `engine/law/src/*`, `engine/words/src/*`, `engine/world/src/*`, `engine/native/src/*`, `engine/plan/src/*`, `bot/*`, `D:\path-tracing-wt\plain\*`, and must launch no game/server/client.

---

## 3. Decision-Free Job Specs for P2, P3, and P4 (and Parallelism)

### Parallelism Schedule
- **Step 1 (serial):** **P1** runs first on `wt/3.2d-3` and creates `zones_prereqs/{main.rs, t1.rs, t2.rs, t3.rs, t4.rs}`. Merge P1 to `wt/3.2d-3`.
- **Step 2 (parallel):** **P2** (edits only `t2.rs` + `jobs/3.2d-3-P2/*`) and **P4** (edits only `t4.rs` + `jobs/3.2d-3/verify.ps1` `-Case T4Food` + `jobs/3.2d-3-P4/*`) have disjoint files and disjoint dependencies after P1; run in separate worktrees off `wt/3.2d-3` (`wt/3.2d-3-P2`, `wt/3.2d-3-P4`) and merge serially to `wt/3.2d-3`. *(P3's numerical single-yaw-miss witness can also run early in `t3.rs` in parallel if desired, or wait cleanly for Step 3.)*
- **Step 3 (serial after P2):** **P3** runs off `wt/3.2d-3` after P2 merges (edits only `t3.rs` + `jobs/3.2d-3-P3/*`), consuming P2's exact binary64 binade integer maps.

---

### P2 (T2): Binary64 Binade Integer Maps & Inverse Stop

- **Branch/worktree:** `wt/3.2d-3-P2` branched from `wt/3.2d-3` after P1.
- **Inputs:** P1's `t1.rs` sprint-jump cycle `movement_this_tick` `f64` operands (`[dx, dz]`) on flat stone (`straight100`), `engine/plan/src/goal.rs:13-40` (`Goal::zone`, `Goal::touched`), `law::try_tick` (`engine/law/src/movement.rs`).
- **Exact implementation & test `t2_binade` in `engine/plan/tests/zones_prereqs/t2.rs` (plus `jobs/3.2d-3-P2/derivation.md`):**
  1. **Exact binary64 map prototype (isolated in `t2.rs`):**
     - Represent position coordinate $p = (-1)^s m \cdot 2^{e-52}$ as exact `(sign, exp, mantissa_u64)` (handling normal, subnormal, and signed-zero `±0.0`).
     - Given cycle step operands $(d_0, \dots, d_{P-1})$ (exact `movement_this_tick` values before position addition, NOT $p_{\text{after}} - p_{\text{before}}$), write $d_j / U = N_j + r_j$ by exact integer/dyadic division where $U = 2^{e-52}$. Round-to-nearest-ties-to-even adds $N_j$ if $r_j < 1/2$, $N_j + 1$ if $r_j > 1/2$, and $N_j + ((m + N_j) \bmod 2)$ at exact $r_j = 1/2$.
     - Compose the finite parity graph over one period $P$ (state = parity bits of $m$ needed by all $r_j$ with exact $1/2$ ties, plus carry), find its prefix and cycle length, and advance $q$ cycles in $O(1)$ via quotient/remainder without a $q$-loop.
     - Compute the next binade-crossing (or sign-flip) event tick via exact integer ceil/floor division of the affine increment map (never stepping tick-by-tick toward the boundary), execute the exact boundary ticks, and continue in the next binade.
     - Invert `Goal::touched` (`[cell - 1e-3, cell + 1 + 1e-3]` intersected with player AABB `[p - 0.3, p + 0.3]`) through the integer map + stored vertical-phase eligibility to return exact first-touch tick and remainder without simulating the tail ticks.
  2. **Witness comparisons against streaming native `Plain` (`law::try_tick` on uniform stone column, `difficulty = 0`):**
     - Crossings from $x = 2^{19} - 100$ through $2^{21} + 100$ (spanning $2^{19}, 2^{20}, 2^{21}$ binade boundaries) at multiple arbitrary queried cycle counts $q$.
     - Negative coordinates ($-2^{20} - 100 \dots -2^{19} + 100$).
     - Zero & signed-zero crossings ($-50 \dots +50$, including `-0.0` vs `+0.0`).
     - Exact tie-parity witnesses ($r = 1/2$ with even vs odd $m + N$).
     - Boundary-neighbour witnesses (`±1` ULP around binade boundaries and `±1` ULP around `Goal::zone` `1e-3` slab edges).
     - Assert **bit-exact** `position.{x,y,z}.to_bits()` and ending motor state bits against the streaming native fold at every witness.
  3. **Poison test:** Write a deterministic `#[ignore]` or test-gated/logged sub-check (`t2_binade_poison_check` or in `jobs/3.2d-3-P2/verify.ps1`) that flips tie-to-even parity in a temporary copy/mode (or env flag `PATHTRACING_T2_POISON=1` used ONLY inside `zones_prereqs/t2.rs` during the P2 verification script), proves `t2_binade` FAILS red, then runs clean and proves PASS green. (No poison switch in production code.)
- **Gate:**
  - `cargo test --offline --release -p plan --test zones_prereqs t2_binade -- --nocapture` (PASS).
  - Poison run with `PATHTRACING_T2_POISON=1` fails (RED), unset passes (GREEN).
  - `cargo test --offline --workspace` + `& .\jobs\1.2-footprints\verify.ps1` (13/13 CAUGHT, restored).
  - Write `D:\path-tracing-evidence\3.2d-3\T2\manifest.json` (`passed: true, difficulty: "Peaceful", mobs: "Off"`, file hashes) and pass `verify.ps1 -Case Prerequisites -Tests T1,T2 -Difficulty Peaceful -Mobs Off -Evidence D:\path-tracing-evidence\3.2d-3`.
- **Must NOT touch:** `engine/law/src/*`, `engine/words/src/*`, `engine/world/src/*`, `engine/native/src/*`, `engine/plan/src/*`, `bot/*`, `t1.rs`, `t3.rs`, `t4.rs`.

---

### P3 (T3): Two-Yaw Lattice & Euclidean Floor-Sum Interval Construction

- **Branch/worktree:** `wt/3.2d-3-P3` branched from `wt/3.2d-3` after P2 merges.
- **Inputs:** P1's observed transient, P2's binary64 binade integer maps in `t2.rs` (imported via `super::t2`), `engine/law/src/math.rs:1-17` (65536 `sin.bin` entries), `engine/words/src/program.rs:84-103` (`Arguments { refined_yaw: Some(yaw), .. }`), `engine/plan/src/goal.rs:13-40`.
- **Exact test `t3_two_yaw` in `engine/plan/tests/zones_prereqs/t3.rs`:**
  1. **Single-yaw miss witness:** On a uniform stone column (`y = -61`, standing `y = -60.0`), start $A = (0.5, -60.0, 0.5)$, target $B = (1000000, -61, 330000)$. Enumerate all 65536 native `sin.bin` bin-center yaws (`index = 0..65536`, `yaw = (((index as f64 + 0.5) * TAU / 65536.0) / 0.017453292f32 as f64) as f32`); prove analytically / via P2's integer map that **zero** single indices intersect `B`'s `Goal::zone` (`[1000000 - 1e-3, 1000001 + 1e-3] x [330000 - 1e-3, 330001 + 1e-3]` grown by player half-width `0.3`).
  2. **Two-yaw bracket & integer lattice construction:**
     - Bracket requested bearing $(dx, dz)$ by exact integer cross-products of adjacent law-table direction vectors $D_i, D_{i+1}$ over the 65536-entry `sin.bin` domain (lowest native index on exact tie).
     - Account for start transient from observed start AND yaw-switch transient at index $i \to i+1$ from the carried motor state (prepared once per native index pair or derived from P1/P2), plus binade boundaries from P2.
     - For each event/binade regime, solve $lo \le E + q_1 D_i + q_2 D_{i+1} \le hi$ with $q_1, q_2 \ge 0$ using Euclidean quotient / floor-sum interval arithmetic over exact dyadic integers (no $q$-by-$q$ loop, no yaw sweep, no LP/SMT/graph solver), choosing minimum total ticks then canonical `(q1, q2)`.
  3. **Verification:**
     - Verify `B = (1000000, -61, 330000)` against a native `law::try_tick` witness replay in `t3_two_yaw` (reference witness only, never used inside the solver), asserting `Goal::touched` is true on the exact predicted tick and false before it.
     - Test axis-aligned cases (`(1000, -61, 0)`, `(0, -61, -1000)`), exact native-index ties (diagonal bearings), and all four quadrants / adjacent-direction pairs.
     - Record and assert operation counts at distances $10^2, 10^4, 10^6$ (must be bounded by event regimes and operand bits, **not** linear in distance or $q$).
- **Gate:**
  - `cargo test --offline --release -p plan --test zones_prereqs t3_two_yaw -- --nocapture` (PASS).
  - `cargo test --offline --workspace` + `& .\jobs\1.2-footprints\verify.ps1` (13/13 CAUGHT, restored).
  - Write `D:\path-tracing-evidence\3.2d-3\T3\manifest.json` (`passed: true, difficulty: "Peaceful", mobs: "Off"`, file hashes) and pass `verify.ps1 -Case Prerequisites -Tests T1,T2,T3 -Difficulty Peaceful -Mobs Off -Evidence D:\path-tracing-evidence\3.2d-3`.
- **Must NOT touch:** `engine/law/src/*`, `engine/words/src/*`, `engine/world/src/*`, `engine/native/src/*`, `engine/plan/src/*`, `bot/*`, `t1.rs`, `t2.rs`, `t4.rs`.

---

### P4 (T4): Peaceful Food Guard & Observed Resource Packet Re-Anchor

- **Branch/worktree:** `wt/3.2d-3-P4` branched from `wt/3.2d-3` after P1 merges (can run in parallel with P2).
- **Inputs:** `engine/law/src/movement.rs:217-233, 1253-1306`, `engine/law/src/codec_state.rs:17`, `engine/law/src/state.rs:284-292`, `bot/tests/fixtures/recorded-player.json`, `bot/src/pathtracing/mixin/ClientPacketListenerMixin.java:36-37`, `bot/src/pathtracing/HopCapture.java:60-72`, `jobs/3.2d-3/verify.ps1`.
- **Exact offline test `t4_food` in `engine/plan/tests/zones_prereqs/t4.rs`:**
  1. **Unchanged law vs store resource guard on both sides of `food_level > 6`:**
     - Construct native-valid Peaceful (`difficulty = 0`) observed starts via `from_observed_bits` with:
       (a) `food_level = 7, saturation = 0.0, exhaustion = 3.99, health = 20.0`,
       (b) `food_level = 6, saturation = 0.0, exhaustion = 0.0, health = 20.0`,
       (c) `food_level = 20, saturation = 5.0, exhaustion = 0.0, health = 20.0`.
     - Run sprint-jump `law::try_tick` over 200+ ticks on flat stone (`difficulty = 0`): assert that for (a) and (c), `exhaustion` wraps past `4.0` in `food_tick` (`movement.rs:1272-1278`) with **zero** `food_level` drain (`food_level` stays `7` / `20`), and `sprinting` stays allowed; for (b) (`food_level = 6`), `update_sprinting` (`movement.rs:219`) refuses to enter sprint (`sprinting == false`) and matches `codec_state::state_answers` (`codec_state.rs:17`) bit-for-bit.
  2. **Reject `fed` / default substitution:**
     - Perturb `food_level` (`6` vs `7` vs `20`), `health` (`0.0` vs `20.0`), `saturation` (`0.0` vs `20.0`), and `exhaustion` (`0.0` vs `3.99`); assert `PlayerState::from_observed_bits` rejects unknown resource words (`known[food_word] = false`), `state_answers` distinguishes `food_level = 6` from `7`, and no `fed` or invented food-drain switch/tick-loop permission check is added.
  3. **No saturation prediction claim:** Explicitly assert only observed/unchanged-guard equality; do NOT assume saturation stays constant across server `server_regeneration` ticks (anomaly 2).
- **Exact `-Case T4Food` live harness in `jobs/3.2d-3/verify.ps1`:**
  - Notice `verify.ps1:16, 31` currently throws on `-Live` and non-`A` cases, and `PlayerSnapshot.observed` (`PlayerSnapshot.java:51-52`) withholds `exhaustionBits`/`foodTickTimer` from `RequestGoal` until K.
  - Therefore P4 adds a dedicated `-Case T4Food` branch in `jobs/3.2d-3/verify.ps1` (taking the rig lock, launching hidden/background server + bot client on `D:\path-tracing-lab` under S135 with `difficulty=peaceful`, `doMobSpawning=false`), which:
    1. Issues server commands (e.g., `/effect` or food/saturation setup or Peaceful wait) that trigger `ClientboundSetHealthPacket` (`ClientPacketListenerMixin.java:36-37` -> `ResidentEvents.observe("health-food")` -> `NativeChannel.ObserveEvent` type 1).
    2. Captures `HopCapture` (`hops-server-*.tsv` `server-resources` + `hops-bot-*.tsv` `observe-health-food`, `observation-bits-health-food`, `observation-known-health-food`), verifies `difficulty == 0`, verifies packet application and updated `food_level`/`health`/`saturation` bits in `ObserveEvent`, and checks `t4_food` guard equivalence on the captured observed bars—without weakening `PlayerSnapshot.observed`'s refusal of `RequestGoal` when `exhaustion`/`foodTickTimer` are unknown.
- **Gate:**
  - `cargo test --offline --release -p plan --test zones_prereqs t4_food -- --nocapture` (PASS).
  - `.\jobs\3.2d-3\verify.ps1 -Case T4Food -Difficulty Peaceful -Mobs Off -Live` (PASS, evidence saved under `D:\path-tracing-evidence\3.2d-3\T4\`).
  - `cargo test --offline --workspace` + `& .\jobs\1.2-footprints\verify.ps1` (13/13 CAUGHT, restored).
  - Write `D:\path-tracing-evidence\3.2d-3\T4\manifest.json` (`passed: true, difficulty: "Peaceful", mobs: "Off"`, file hashes) and pass `verify.ps1 -Case Prerequisites -Tests T1,T2,T3,T4 -Difficulty Peaceful -Mobs Off -Evidence D:\path-tracing-evidence\3.2d-3`.
- **Must NOT touch:** `engine/law/src/*`, `engine/words/src/*`, `engine/world/src/*`, `engine/native/src/*`, `engine/plan/src/*`, `t1.rs`, `t2.rs`, `t3.rs`, and must NOT weaken `PlayerSnapshot.observed`'s unknown-resource refusal on `RequestGoal`.

---

## 4. Judge Findings Carried Forward to Later Slices

### Findings Explicitly Highlighted for Carrying

1. **A's unmeasured per-block chunk strings on the client thread (Judge A #2) -> Slice C / E / F + K:**
   - *Where:* `bot/src/pathtracing/ResidentSource.java:30-54, 72-75` builds & sorts property strings (`canonical(state)`) for 4096 blocks/section × ~24 sections (~100k strings/chunk) on the Minecraft client thread and allocates `ArrayList<Long>` + 4096-word index arrays across JNI; `ResidentRunner.java:24` / `PlayerSnapshot.java:34-55` also builds a full `JsonObject` per packet event.
   - *Slice ownership:* **C / E** pins the 26.2 registry hash at world join and moves chunk transmission to compact registry-indexed `FineId` integers (with string decoding/cache once per registry, not per block per chunk); **F** encodes observed `StateBits` directly without intermediate `JsonObject` allocation on events; **K** measures client-thread latency distribution (p50/p90/p99/max) against Fermi floors.

2. **A's playback cancelling on echoed packets (Judge A #3) -> Slice F (+ K live check):**
   - *Where:* `bot/src/pathtracing/ResidentRunner.java:21-35` unconditionally calls `cancel(mc); TickInput.release(mc);` whenever `executing` is true on *any* `ResidentEvents.observe(kind)` call—including `"entity-data"` (`ClientPacketListenerMixin.java:44-49`) and `"attributes"` (`:38-43`), which the server uses to echo sprint/pose.
   - *Slice ownership:* **F** replaces `legacy_sentence_adapter` and unconditional abort with the shared `deviation::compare_event` + versioned `StateField` ownership manifest (`spec.md:79-82`): echoed/server-driven fields update/re-anchor without cancelling valid continuation; **K** verifies on live Peaceful runs.

3. **B's byte clones in the hot path (Judge B #1) -> Slice C (+ H / J):**
   - *Where:* `engine/law/src/pattern.rs:289-308` removed `PatternTable`'s `(model_id, mask, loaded)` `by_model` cache; `Memo::bytes` clones a `Vec<u8>` on every `cell_id` lookup; `engine/plan/src/reach.rs` boxes canonical bytes per premise cell. Release `read_cost` measurements were `ignored`.
   - *Slice ownership:* **C** builds `GeneratorStore` + `ProjectionTable` + `SectionRoles` using `ClassId(u16)`/`FineId(u32)` without per-cell `Vec<u8>` clones on the hot plain path, and runs the release `read_cost` / pattern measurements; **H / J** eliminates unnecessary `Vec<u8>` clones in `PatternTable::cell_id` / `Memo` and `reach.rs` where `PatternTable` remains used for exceptional spans.

4. **The shared 16-bit `ClassId` space (Judge B #2) -> Slice C:**
   - *Where:* `engine/law/src/codec.rs:173, 208` caps `ClassId(u16)` at 65,535 (`TableFull` at `u16::MAX`), and `PatternTable::start_ids` (`pattern.rs:226`) + `ProposalTable::ids` (`words/src/table.rs:148`) now intern state classes into the *same* `Interner` as cell and environment answers (previously `HashMap<Box<[u64]>, u32>`).
   - *Slice ownership:* **C** decides whether state recurrence/proposal keys get their own ID type/space (e.g. `StateClassId(u32)` or separate table) or `ClassId` widens, so large proposal/store runs cannot hit `TableFull` at 65,535.

### Remaining Judge Findings Tracked for Completeness

- **Judge A #1** (live `goto`/`come` refuse until server `exhaustion`/`foodTickTimer`/`natural_regeneration` exist; `PlayerSnapshot.java:51-52`, `ResidentSource.java:22-23`): **Slice K** (and P4 for `-Case T4Food` only).
- **Judge A #4** (per-tick `ObserveEvent({0, version})` poll in `ResidentRunner.java:65`): **Slice F / G** (latch/callback wakeup).
- **Judge A #5** (`Source::block` / `natural_regeneration` panic-as-Unknown in `resident.rs:40-49, 136`): **Slice C / E** (typed `codec::proof::Source` refusals).
- **Judge A #6** (`HopRconMixin.java:13` `lambda$runCommand$0` fragility): **Slice K** (injection audit + live baseline capture).
- **Judge A #7** (`legacy_sentence_adapter` runs `law::try_tick` per tick in `resident.rs:195-204`): **Slice F** (deletes adapter).
- **Judge B #3** (`ClassId`/`LabelId` are interner-local; `Arc::ptr_eq` owner checks): **Slice C** (persist canonical bytes, never numeric IDs).
- **Judge B #4** (`SnapshotSource` ignores tick in `codec_proof.rs:19-44`): **Slice C / E** (resident source implements real tick/version semantics; never use `SnapshotSource` for live mutable worlds).
- **Judge B #5–6** (public `Interner::with_digest`/`digests()` test hooks, dead `pattern::fnv` wrapper at `pattern.rs:342-344`): **Slice J** (cleanup/consolidation).
