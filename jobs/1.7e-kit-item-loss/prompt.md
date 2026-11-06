# Job 1.7e: an item disappears from Julien's inventory after a track

Julien (S38): "bug, when doing a track it deletes sulfur spike from my inventory". The sulfur spike (`minecraft:sulfur_spike`, a 26.2 block item) is in his own inventory before A. After the track, when his inventory is restored, it is gone. His kits don't contain it. The session file is deleted after the restore, so no evidence was kept.

On `wt/tracks` (fast-forwarded to `main`). Do not commit; the integrator commits. You don't run the game. If the worktree has uncommitted changes from an interrupted run of this job, continue from them.

**Already checked by the integrator, don't redo:** stick removal compares the exact id `minecraft:stick`. The capture (`TrackInventoryCodec.server`) and the restore (`TrackHumanClient.apply`, creative slot packets to menu slots 5 to 45) use consistent slot numbers. The codec uses `getOrThrow`, so it doesn't skip an item silently.

1. **Find the cause** in the code and the 26.2 jar.
   - Read the server's handling of `ServerboundSetCreativeModeSlotPacket` (validation: creative mode, slot range, stack size, feature flags, the stream codec), the item's class and default components, and every step between entering track mode and the restore.
   - State the cause, with the jar method that drops the item, or the line in our code. If you find more than one candidate, test each one.
2. **Fix it** so every item survives a round trip, not just the sulfur spike. Never special-case one item (AGENTS.md: no patches). If some item truly can't be restored this way, use a mechanism that can restore it. If none exists, refuse to enter track mode while such an item is in the inventory, and say which item and why.
3. **Verify before deleting.** After a restore, read the server's view of the inventory and compare it slot by slot with the saved session. If anything differs, keep the session file, report the slot and the item in chat, and offer `/track restore` to retry.
4. **A test over every item:** for every item in the 26.2 registry (`BuiltInRegistries.ITEM`), a stack of count 1 and a full stack go through the save codec, the session JSON round trip, and the network encoding the creative packet uses (`ItemStack.OPTIONAL_STREAM_CODEC` or whatever the packet uses, with the registry). Each must come back equal (`ItemStack.matches`). List any failures.
5. Update `bot/tracks.md` for `/track restore` (at most 37 lines).

## Done means

- `bot\build.ps1` builds. The headless tests, including the all-items test, and `TrackMixinAudit` pass.
- The final message states the cause and lists every change.
