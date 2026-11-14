**Job 3.2g is incomplete.** I added `PlayerState::from_bits` in [state.rs](/D:/path-tracing-wt/from-bits/engine/law/src/state.rs), with position-bearing refusals and raw float-bit decoding. Focused tests cover NaN payloads, optional fields and invalid encodings. The full `cargo test --offline` workspace suite passed; its existing fixture-dependent and release-measurement tests remained ignored.

The witness JSONL format and loader, collection of states from existing tests and archived fixtures, and witness-writing cost measurement remain to be done. The current table records predicted exit and tick count, but does not retain predicted end bits; it also holds an interned pattern key rather than a reconstructible world snapshot.

The footprints gate did not reach its mutation checks: it was denied access to `D:\path-tracing-deps\footprints-verification\workspace.log`. No 12/12 result is claimed.

The uncommitted change is **154 added lines, 0 removed**, including [the decoder tests](/D:/path-tracing-wt/from-bits/engine/law/tests/from_bits.rs).