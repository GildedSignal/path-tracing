The witness implementation is **not complete**. I found that `Surprise` already retains predicted and actual state bits, but its footprint does not retain the world contents needed for an independent replay. Footprints can also include environment reads and external collision results, which the replay format must account for.

No files were changed. The baseline `cargo test --offline` was still running when last checked; the JSONL round trip, writer-cost measurement, and 12-of-12 verification gate were not completed. Net change: 0 lines.

STATUS: BLOCKED: I could not complete the self-contained witness serialization and replay in this session.