# Job 1.5d: the replay reader meets the real recordings

On `wt/law` (`0f6c901`), `engine/law/tests/replay.rs` must read the recordings the recorder (merged on `main`) and the fixture recorder (`wt/fixtures`) actually write. Make exactly these changes. Do not commit; the integrator commits. Cargo settings as in job 1.5b.

1. **The `end` trailer.** Recordings end with `{"type":"end","ticks":N,"complete":true|false}`. Accept it as the last line only. Fail the fixture if it is missing, if `complete` is false, or if N differs from the number of tick rows. Any other line with a `type` key after the first line is an error.
2. **`serverEnd`** may carry the number of server ticks between the last recorded tick and the read (the field's name and meaning are in `git show wt/fixtures:bot/recorder.md` once job 1.6b lands; until then, accept a field `ticksAfterEnd`). Compare the server resources only when it is 0, and print that they were skipped otherwise. Never pass silently.
3. **New world fields** from the exporter: `bed`, `suffocating`, `endPortalOrGateway`, `inside: "unmodeled"`, `ticksFrozen` in the `player` line. Read them. A block marked unmodeled that the player touches fails the replay with its name.
4. **Powder snow.** If not already ported, port the `tryAddFrost` `MOVEMENT_SPEED` modifier (−0.05 × percentFrozen while `ticksFrozen > 0`) and its decay, seeded from the `player` line's `ticksFrozen`.

Done means `cargo test` passes in `engine/` (replay ignored without fixtures), the four mutant builds pass, and a synthetic fixture for each of items 1 to 3 is tested. The final message lists every change.
