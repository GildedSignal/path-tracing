Added the frozen-counter controls in [replay.rs](D:/path-tracing-wt/l0-veto/engine/law/tests/replay.rs:421):

- `freeze_sync_refuses_tick_111_post_moved_to_pre` refuses the resulting END mismatch.
- `freeze_sync_refuses_tick_111_post_moved_to_between` refuses the START mismatch.
- `freeze_sync_refuses_tick_112_between_moved_to_pre` refuses the START mismatch.
- `replay_text_freeze_sync_checks_recorded_value_and_start` passes the recorded text and refuses edited values at both START and END.
- `replay_text_legacy_fixture_without_freeze_sync` confirms an older fixture still replays its recorded counter.

`cargo test --offline` passed from `engine/`. The recorder-fixture replay was ignored because no recordings are present; no live run was performed.