**PASS WITH FINDINGS**

I found no blocking issues. I did not regenerate `use-effects.tsv` (that would write into the worktree), and I did not run `jobs/1.2-footprints/verify.ps1` (12 of 12 mutants), so those two claims are unconfirmed.

**Does the ported rule equal Mojang's? PASS.** I disassembled the 26.2 client jar myself with `javap`.
- `modifyInput` runs in this order: 0.98 (offsets 12â€“15), then the item multiplier gated on `isUsingItem && !isPassenger` (19â€“41), then the sneak factor (42â€“63), then square-movement normalization (64â€“68). `modify_input` at `movement.rs:268-274` is in the same order (0.98, item, sneak), so there is no order swap. The `isPassenger` gate is covered because `riding` panics earlier at `movement.rs:114`.
- `itemUseSpeedMultiplier` (0â€“19) and `isSlowDueToUsingItem` (0â€“34) read `UseEffects` from the stack, as the port does.
- `canStartSprinting` (31â€“35) refuses sprint start when the predicate is true, which matches `movement.rs:207`.

**Is the item data the game's? PASS.**
- `engine/world/tools/use-effects.ps1` and `UseEffectsTable.java` regenerate `engine/world/data/use-effects.tsv` from the jar. They walk the same `DATA_COMPONENT_INITIALIZERS` path as the dossier's `UseEffectsProbe`.
- The jar's sha1 is `2dc72797â€¦3754`, which matches the dossier. The script guards sha256 `40896EE9â€¦`, the same guard `models.ps1` uses for the jar the law already reads.
- The table has 1,537 rows: 1,530 are `false 3e4ccccd` and 7 are `true 3f800000`. The seven are exactly the spears, which matches the dossier.
- The law never reads the table. Callers supply `use_item_speed_multiplier` and `use_item_can_sprint`, and `item_multiplier` and `update_sprinting` refuse (`.expect`) when they are `None`.

**Is there a patch? No.** I found no epsilon, clamp, retry or per-item branch. The seven spears are never named in code, only in the data.

**Is the witness real? PASS, with a caveat.**
- The builder's log (`log.jsonl`, item_20) shows a fields-less version of the test failing on the original panic: `movement.rs:124 not implemented: LocalPlayer.itemUseSpeedMultiplier`. It used the same assertion, `0x3fe9f7f1008b3824`.
- The committed test can't compile on `main` because the new fields don't exist there, so its red state there is a compile error. The signature change was necessary, and the assertion was not weakened.
- Green: the law reproduces the dossier's apple tick-20 z (`3fe9f7f1008b3824`) and iron spear z (`40103af6980f2207`). The world test also checks sprint start against the table.

**Sprint semantics. Confirmed from bytecode, not from the dossier.**
- Retention: `shouldStopRunSprinting` (0â€“43) does not read the predicate. `isSlowDueToUsingItem` has only two callers in `LocalPlayer` (`canStartSprinting:32` and `aiStep:348`).
- Start: a world test sets `sprinting` on, ticks with an apple, and asserts sprinting continues.

**Non-blocking findings**
1. `UseEffectsTable.java` has `if (fx == null) fx = UseEffects.DEFAULT;`. It is the jar's own constant, not ours, and it never fires (all 1,537 rows have explicit bindings). It should throw instead, so a future jar can't silently default.
2. `aiStep:348` also calls `isSlowDueToUsingItem` to reset `sprintTriggerTime` (the double-tap sprint window). The law has no such state (grep finds none), so nothing is wrong today. The spec's offset list omitted it, so the spec is at fault. Add it to HANDOFF as an unmodeled reader.
3. The report says "the dossier says retention does not read the predicate." The dossier says only that the start gate is "cited by bytecode only." The claim is true, but it is attributed to the wrong source.
4. Nothing in production calls `world::use_effects` yet. The data path exists and is tested, but the adapter that feeds the law is future work.
5. `main...wt/use-item` also shows three older docs commits (`ac2510b`, `fb5ae99`, `1973d94`) and `AGENTS.md`/docs edits. They are not part of `41972f8`. Check they are intended before merging.
6. The `Bought:` line is truthful but does not say that 93% of the growth is generated data.

**Net lines (`41972f8`): +1,649 / âˆ’10, so +1,639.**

| Part | Lines |
|---|---|
| Production code (`movement.rs`, `state.rs`, `state_reads.rs`, `class.rs`, `items.rs`, `lib.rs`) | +38 / âˆ’10 (net +28) |
| Tests (`use_item_witness.rs`, `use_effects.rs`) | +32 |
| Tooling (`UseEffectsTable.java`, `use-effects.ps1`) | +42 |
| Generated data (`use-effects.tsv`) | +1,537 |

**Gates.** `cargo test --offline` from `engine/` on `41972f8` exited 0 and every target reported `ok`. That includes the jar suite at 6 of 6, `use_item_witness` at 1 of 1 and `use_effects` at 1 of 1, with 0 failures. There were no link failures. An earlier run showed exit code 1 only because of my PowerShell pipe; the clean rerun exited 0.

**Merges as-is.** I'd fold finding 1 into the next job rather than hold the merge.
