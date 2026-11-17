# Continue: finish the track fixes

The server is now **stopped** (checked 17 November, 15:30). Run `scripts\common.ps1`'s `Test-Port` again yourself before the spawn audit, and refuse if it is open.

**On the rig's `maps.json` being outside your writable roots:** don't write it. Read the rig's `maps.json` and its map dimensions' region files read-only from `C:\Users\PASCAL\Desktop\path tracing`. Write the full corrected file, with the same schema and every map, to your worktree as `jobs\track-ux\maps.json`. The integrator compares it and copies it into the rig. List each map's old and new spawn in `last.md`, and note any map whose column has no standable cell, for Julien.

**Finish what your last message left open:**
1. End-to-end headless coverage for the recovery path: a stale saved session (from a crash, a disconnect or an unfinished exit) is recovered by itself, and the track then arms with no `/track restore`. Also cover a failed kit transaction, which must not leave the track stuck.
2. The arming time **from stepping on A to "Armed"**, over at least 20 events: track selection, detector arming and recovery included, not only the kit transaction. Name each wait, and give before and after distributions. Review the restoration's remaining client-tick transition by the same rule: remove it unless it is a server confirmation.
3. The action bar ("Arming…", "Armed: go", "Not armed: <reason>") and the chat line when he leaves A unarmed, each covered by a test.
4. The command wiring for autocomplete, covered by a test beside the prefix-filter check.

The rest of the original prompt (`jobs\track-ux\prompt.md`) still holds, "Must not touch" included. Done means everything in its "Done means", with `maps.json` delivered as above. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
