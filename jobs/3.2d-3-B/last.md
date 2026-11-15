# 3.2d-3 B / R1 — gated

Consumer: slice C's label-keyed generator store and the existing footprint, pattern and proposal consumers. Base: `cf42d31592303e9648984637b342801bb4a7b7f1`. No commit, push or live run was made.

## Changed files

- `engine/law/src/codec.rs`: one byte-comparing interner for typed ClassId/LabelId; role projections, fine-ID/kit projection tables, exact canonical answers, Unknown verbs, existing mining-progress query and framed labels.
- `engine/law/src/codec_state.rs`: the existing exhaustive state-answer projection moved into the codec.
- `engine/law/src/codec_planes.rs`: AllZero/AllOne/Words leaf planes, complete/loaded masks, masked SSE2 queries on x86_64, source-projection builder and individual-bit updates.
- `engine/law/src/codec_proof.rs`: private question-bound proofs, exact state/context/version binding, source coverage, timed native read certificates and direct reverse dependency indices.
- `engine/law/src/lib.rs`: codec export.
- `engine/law/src/footprint.rs`: common answer references, exhaustive query list and world-read draining that preserves the segment's read-before-write state mask.
- `engine/law/src/pattern.rs`: cell/environment/start IDs from the codec; exact-byte cell/presence wrappers; typed state questions; unknown query bits refuse. The shared block-field encoder remains here to preserve the original mutation sites.
- `engine/law/src/mining.rs`: expose the existing progress query; its arithmetic is unchanged.
- `engine/words/src/class.rs`: thin state-answer wrapper and comparison-question signature.
- `engine/words/src/table.rs`: proposal classes and world patterns share the table's interner.
- `engine/words/src/segment.rs`: exact support comparisons, timed native read sets and certificate adapter.
- `engine/words/src/gait.rs`: tag recurrence start questions distinctly from exact/comparison questions; no recurrence or resource arithmetic changed.
- `engine/plan/src/reach.rs`: replace digest-only premise equality with exact canonical answers; unsupported answers cannot establish equality. Search logic is unchanged.
- `engine/law/tests/codec.rs`: forced collisions, labels/kit/Unknown/mining distinctions, local presence, exact edits, coverage/state/version/question/owner refusals, environment timing and both un-evented read witnesses.
- `engine/words/tests/codec.rs`: forced collisions through proposal classes and the timed segment certificate adapter.
- `engine/world/tests/codec.rs`: actual registry air/button/plate/flower equivalence and local cobweb presence.
- `jobs/3.2d-3-B/verify.ps1`: reproducible offline gate, input hashes, outputs, invariant/refusal record, mutant names read from the unchanged pinned call sites, source-restoration checks.
- `jobs/3.2d-3-B/last.md`: this handoff.

The two region replacements and their `.bak-20261114` files were already present at entry; this job did not edit them. Their replacement hashes are in the run record. `bot/`, state.rs, movement.rs, flight.rs, plan/bin and native crates have no job delta. The unchanged footprint gate temporarily mutates movement.rs and restores its original bytes.

## Gate commands and outputs

From this worktree: `& .\jobs\3.2d-3-B\verify.ps1`.
The script sets `CARGO_TARGET_DIR=D:\path-tracing-target\r1-codec`, runs `cargo test --offline` from `engine/`, then runs the unchanged `jobs/1.2-footprints/verify.ps1`.

Final output:
```text
Evidence: D:\path-tracing-evidence\r1-codec-991db1497425453cb576337bbf779524
workspace PASS: passed=200 ignored=18
footprints PASS: caught=13/13, sources restored
```
`run.json` records the exact code/schema/fixture hashes and all mutant names; `workspace.log` includes both passing compile-fail proof tests; `footprints.log` records each CAUGHT result and the restored/randomness audits. The footprint gate's own per-mutant logs are retained under its generated evidence directory. The workspace's pre-existing ignored tests remain ignored; no codec gate was skipped. R1 synthetic worlds use Peaceful; registry-only checks do not depend on difficulty. No live clock or native replay claim is made.

The first qualification run also passed (evidence `r1-codec-41b8e9ba66f34b44a8f75bcd65577fb3`); the final run follows direct reverse-index and state-question refinements. During fixture construction, the wall-release positive control initially still hit an upper wall, and the sneak positive control initially still overlapped its original support. The final fixtures remove those witness mistakes and assert changed native movement before checking invalidation; no law arithmetic or acceptance condition was loosened.

## Slice C interface and boundaries

1. Use the owning `Interner`, exposed by `PatternTable::codec_mut` or `ProposalTable::codec_mut`, for source projections, footprints and labels. IDs are local to that owner, not persistent fingerprints. Projection tables and read-certificate stores reject a different owner.
2. `ProjectionTable` pins the source registry identity and kit query signature, caches answers by **fine** ID/query context, and holds no terrain. An inventory change selects another projection table over the same source. A model-table ID is not a fine block-state ID. Full Break/Place remain `Projection::Unknown`, retaining registry/fine-state/kit/verb identity; only the already-ported mining progress subquery is Known.
3. Build `Label { law_schema, body_kit, support, medium }` and call `Interner::label`. The resulting `LabelId` is C's outer store key. Preserve tuple order and roles; yaw is a word argument. Finer kit labels can reference the same legs kernel.
4. The four `LegsRole` masks and only their specified omissions are implemented. `pattern::law_digest` now returns `Canonical`, whose equality compares bytes. `pattern::cell_digest` likewise returns an exact answer, not u128. `fingerprint()` is informational. Exact source-query answers retain raw fluid-neighbor predicates: an empty neighbor's flow predicates can affect a different cell's current. The test pins this distinction from the legs projection; there is one field encoder with question signatures, not another geometry implementation.
5. `SectionRoles::from_classes` derives leaf bits from the selected projection without copying source terrain. `query` checks only requested words and refuses missing complete/loaded bits before returning exceptions. Loaded evidence must come from the actual qualified source; disk coverage does not grant a loaded lease.
6. Native segment preparation retains `Segment::timed_reads`. `certify_reads`/`ReadCertificates::capture` qualifies that native read tube once, including candidate-empty cells, geometry halos, environment/schedule reads, wall release and sneak back-off. `answer` and `validate(proof, expected_certificate, binding)` reuse it without tick/cell scans. The proof is specifically ReadsHold; it cannot be used as TubeEmpty or as a movement/dispatch proof.
7. Publish source edits and environment events before consuming a proof, under the same mutable certificate ownership boundary. `edit(position, tick_interval)` and `environment_event(tick_interval)` visit directly indexed recorded entries at their original ticks. Same-content/unrelated edits preserve proofs; changed answers invalidate only dependents; lost coverage is Unknown. `SnapshotSource` is only an explicitly qualified immutable adapter; the resident source must implement `Source` with its actual coverage and time semantics.
8. A new Block field, StateField or Query kind requires an exhaustive schema assignment. Presence/class equality proposes reuse only; it does not close world-hook/restoration obligations or create an unported capability. Source capture and prepared generator integration remain A/C's consumers of these interfaces.
