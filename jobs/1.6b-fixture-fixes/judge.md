**Verdict: ready with fixes.**

Commit `684946e` (`wt/fixtures`). Nothing here makes a recorded movement fixture wrong today, and the mixin only observes. But the `corner` course as built cannot kill the axis-tie mutant it exists for, and the end-of-run server read will in practice never be compared. Fix findings 1 to 3 before the gate is frozen against these fixtures.

Checked: `bot\build.ps1` builds. The offline tests pass: `test-courses`, `test-server-state`, `test-fixture-json`, and `test-record-fixtures` (34 cases). `test-law-reader.ps1` now FAILS (finding 5). No game or server was run. The Java mixin and exporter have no offline test, so those verdicts come from reading them against the 26.2 bytecode and `javap` of the client jar.

## Findings, most severe first

### 1. The `corner` course cannot tell the two axis orders apart (not fit for purpose; the spec asked for it)
`lab/gates/courses.ps1:88-92`, `lab/gates/record-fixtures.ps1:38-46`.

- **The tie is real.** The input does give |dx| = |dz| bit for bit.
  - At yaw 0, `Mth.sin(0)` is `SIN[0]` = 0 and `Mth.cos(0)` is `SIN[16384]` = 1.0. I read both entries in `jar-sin.bin`.
  - `getInputVector` then gives x' = vx*1 - vz*0 = vx and z' = vz*1 + vx*0 = vz.
  - forward+left makes vx = vz. All later ops are symmetric (friction, 0.98, `Vec3.add`).
  - The start is (5776.5, 4100.5), the walls are at x = 5780 and z = 4104, and x - z = 1676 is a multiple of the ulp inside one binade. So both gaps are bit-equal and both axes first touch on the same tick.
- **The geometry makes the tie irrelevant.** The corner is an inside (concave) corner: two long walls meeting at (20, 8).
  - `collideWithShapes` clips the axes one after another.
  - The x-wall spans z 1..8 and the z-wall spans x 1..20. Neither wall ends, so clipping x first never changes what blocks z, and the reverse holds too.
  - So X-first and Z-first return the same vector. A mutant that swaps the order, or flips `<` to `<=`, produces identical positions and is NOT caught.
- **The witness predicate is vacuous for order.** `Axis-TieCollision` only requires a tie plus any difference between request and result. On this corner it is true, and it would also be true on a geometry where order cannot matter. The report line "collision exists: True" therefore proves nothing about killing the mutant.

Failure scenario: the red-team plants "axis tie order" at the gate's freeze. Every `corner` fixture passes unchanged, and L0 reports the mutant alive or, worse, that coverage exists.

Smallest fix:
- Use an outer corner, i.e. a single stone block at local (20, y, 8) with nothing adjoining it. It is the same diagonal, since 20 - 8 = 16.5 - 4.5.
- On the tie tick, X-first leaves x free (the box's z range still ends below 8) and then clips z. Z-first does the opposite. The results differ.
- Make the witness require that exactly one of x and z is clipped on the tie tick. An inside corner clips both, so it fails the check.
- Confirm offline in the Rust law that swapping the order changes the position on that tick.

### 2. `serverEnd` will almost never have offset 0, so the end resource check is effectively skipped
`lab/gates/record-fixtures.ps1:267` (read), `:98-107` (offset); `scripts/server-state.ps1`.

- The read happens after `Bot` sees the "Saved" line, then spawns a fresh powershell (`Invoke-Script`), which runs `server-state.ps1`, which runs `rcon.ps1`.
- That costs about 0.5 s or more, so at least 10 server ticks, plus the bot's own idle ticks.
- The offset is arithmetically correct (modulo 2147483647, min and max from the bracket, null when they straddle). But the value it reports will be 10 or more on every run, so the reader skips every `serverEnd`.
- The tests only show offset 0 through a stub that returns the same game time.
- L0 requires "damage and exhaustion equal the server's". With this design the fixtures contribute zero checked server snapshots. The reports say "compare only when exactly zero" but never say this will be the norm.

Smallest fix, either:
- (a) Freeze the server tick at the last recorded tick. The bot is op, so the recorder can send `/tick freeze` when it finishes, the driver reads, then runs `/tick unfreeze`. Or
- (b) Let the reader replay `offset` extra zero-input ticks before comparing. The bot releases its inputs at the end, and the law supports zero-input ticks. This is only sound when the player is at rest on the ground.

Also have the report count and state how many `serverEnd` snapshots are comparable.

### 3. The initial `foodTickTimer`, exhaustion and regen state are assumed constant, and only partly checked
`lab/gates/record-fixtures.ps1:88-95` and `:255`.

- The start read is taken after the thaw loop, before `replay`. The client then runs the teleport and 5 settled ticks, at least about 25 ticks, before input zero.
- `foodTickTimer` only advances while the bot is hurt with food >= 18, which is exactly what edges, water (drowning) and fall runs produce.
- The driver compares only health, food and saturation (client against server). If the bot is hurt and its saturation is 0, a heal fires only every 80 ticks.
  - Roughly 70% of the time nothing visible changes in the window, but the timer moved about 25 ticks.
  - The run is accepted with a stale `foodTickTimer` in the player line, and the law's regen schedule is shifted.
- The abort and cleanup paths also leave the bot hurt for the next run.

Smallest fix: before each run, require server Health = 20 and `foodTickTimer` = 0, i.e. wait or reject. Otherwise read the snapshot a second time after the settle and require every resource field to be identical, including exhaustion bits and the timer.

### 4. The powder-snow note and fixtures rest on an unverified premise
`bot/recorder.md:259-263`.

- The −0.05 × percentFrozen modifier matches `tryAddFrost` in `LivingEntity` (line 1064 of the dump). But `removeFrost` and `tryAddFrost` run only inside the `ServerLevel` branch of `LivingEntity.tick` (offsets 667-730).
- The client sees the speed modifier only through attribute-update packets, and the thaw (−2/tick) only through entity-data sync. Increments happen locally through `InsideBlockEffectType.FREEZE`.
- So the recorded `speed` and `ticksFrozen` of a powder-snow run depend on packet timing. "The law must model it" is not enough: a one-tick delay between the server tick and the client's packet is unmodeled and would show up as a bit mismatch.
- The recorder logs no attribute or entity-data packets as anomalies.

Fix: capture the `ClientboundUpdateAttributes` and `SetEntityData` arrival ticks as per-tick fields or anomalies, or have the law take `speed` as an input on powder-snow ticks. At minimum, change the doc to say this is unverified rather than "the law must model it".

### 5. The reader patch is correct for `0f6c901` but is now dead
`jobs/1.6-l0-fixtures/law-reader.patch`, `lab/gates/tests/test-law-reader.ps1:12`.

- I checked it with `git apply --check` against `engine/law/tests/replay.rs` and `lab/fixtures/README.md` extracted from `0f6c901`. It applies cleanly.
- Content:
  - the trailer logic (`type=end`, `complete`, matching tick count, nothing after it) is correct;
  - `foodTickTimer` is compared only when present;
  - the offset gate is `!= Some("0")`, so null, missing and nonzero all skip. A null emitted by `ConvertTo-Json` is the string `null` in `Object`, which is handled;
  - `Object::parse` handles `collisionSteps` arrays and nested objects.
- But `wt/law` is now at `3f6d65c` (job 1.5d), which implements trailer, offset and `ticksFrozen` itself. The patch no longer applies there, and `test-law-reader.ps1` fails with "Cannot apply reader integration patch."
- The 1.5d reader does not compare `foodTickTimer`, so one thing from the patch is missing.

Fix: delete the patch and its test. Add a reader test that parses a recorded `collisionSteps` row and a `serverEnd` with the real field names. Port the `foodTickTimer` compare if you want it.

### 6. A `corner` run without a tie witness is kept and the driver still says PASS
`lab/gates/record-fixtures.ps1:277-280`, test case `corner-no-tie`.

The course exists only to produce the tie, yet a run with none is kept and the exit code is 0 (the test asserts exit 0). The spec says "reports whether it does", so this is literal.

Fix: the course should reject runs with no witness, or the overall result should be FAIL with a "no witness" note. Do this together with finding 1.

### 7. Test gaps (item 8)
- `Axis-TieCollision` is exercised by one tie row (positive) and by a recording with no `collisionSteps` (negative).
  - An always-false mutant is caught.
  - An always-true mutant is NOT: `corner-no-tie` has no rows at all, so every implementation returns false there.
  - Unequal magnitudes, sign bits, and equal request/result are never tested.
  - Fix: add a unit test of the predicate with rows for each of those cases.
- The mixin and exporter have no offline test. The hook-override walk and the `bed` / `fallOnMultiplier` values are verified here only by reading. Add a Java or script check that lists, for the class set the courses use, which blocks come out as `unmodeled`.

### 8. Minor
- `scripts/rcon.ps1` `Read-Packet` reads one reply packet per command. A `data get entity` reply over 4096 characters is split by the server. The result is a loud failure ("Truncated entity NBT"), not a silent one, but a bot with a large recipe book would fail every snapshot. The test covers truncation.
- Negative game-time offsets (client ahead of the server) come out as about 2^31 because of the modulo. Harmless (not zero) but confusing in the report.
- `collisionSteps` carries two full vectors with bits per tick, so each fixture is roughly twice as large. The repo is public. Consider storing bits only.
- `courses.ps1` `edges`: the first return step, `Fill 12 -61 1 20 -61 1`, is already floor, so the test assertion for it is trivial. The staircase has only 3 real rises from the floor surface, which matches the intent. The island width and the sneak percentage are as specified.
- Water: the rim is placed immediately after the source row (the next command). A flood is only possible if more than 5 ticks elapse between those two commands. I judge that unlikely but unproven.

## Item by item

1. **Items 1 to 8 of the spec.**
   - **1 (abort).** Done as written. No `rec stop` after an abort. Each cleanup step has its own try/catch (`Cleanup`). The stub is silent after an abort. The `abort-continue` test asserts "no `bot:rec stop`" and that the following runs and courses complete. The `cleanup-failure` test covers a failing cleanup. A cancel that fails still marks the run rejected through the `failed:` note.
   - **2 (water).** Done. Details under point 3 below.
   - **3 (one-tick read).** Done as specified. Findings 2 and 3 are about what the spec chose, not about deviations.
   - **4 (powder snow).** `ticksFrozen` is captured. The driver waits for server `TicksFrozen` = 0. The recorder settle also requires client `ticksFrozen` = 0. The doc is added. See finding 4.
   - **5 (corner).** The input and tie arithmetic are exact. The geometry is flawed (finding 1). The step-up equivalence statement is in the report, as asked.
   - **6 (edges).** Done: wider islands, a return stair, and `SneakPercent` = 90 for edges only. Other courses keep the default sequence because the extra random draw happens only when `SneakPercent` differs from 50.
   - **7 (exporter).** Done. Details under point 6 below.
   - **8 (minor).** Game mode is restored in `finally`, and the readiness export is deleted and never saved as an artifact.

2. **The mixin.** `EntityCollisionMixin` injects at `RETURN` of `Entity.collide(Vec3)`.
   - The method is private. There is exactly one call site in the dumps, in `Entity.move` after `maybeBackOffFromEdge`. `move` is the only caller, so the logged request is the post-sneak, post-stuck vector.
   - There is no `astore_1`, so the argument is never reassigned before a return.
   - It only reads `requested` and `getReturnValue()`. `Vec3` is immutable.
   - It adds no argument, return value, order or side effect to the game's path. The mixin config lists it as client-only, so the server is untouched.
   - The recorder filters on `Minecraft.getInstance().player`.
   - Risk: an exception inside `TraceJson` would escape into `collide`. That cannot happen with finite doubles.

3. **Water.** The tracer matches vanilla's `getSpread` (down first, slope search 4, falling water spreads at 7, sources spread sideways).
   - The nearest wet cell is at z ≥ 8, more than 3.5 blocks from the start. A rim of stone at z = 7, y = −61..−60 separates both the pool and the source row.
   - The negative control is real: it turns the rim's upper layer to air and demands the specific "fluid reaches within 3 blocks" failure. It only passes if the tracer floods the start area.

4. **Single-tick read.** `data get entity` is one command and is atomic. The NBT field names match 26.2 (`foodTickTimer`, `foodSaturationLevel`, `foodExhaustionLevel`, `Health`, `playerGameType`, `TicksFrozen`, which is omitted at 0). The offset arithmetic is right, but see finding 2.

5. **Corner arithmetic.** See finding 1.

6. **Exporter.** `javap` shows the override walk is right for the course blocks. Stairs, slabs, ladders, vines, trapdoors, ice, soul sand, scaffolding and liquids declare neither hook. The blocks that do declare one come out as expected:
   - Honey, powder snow, web, hay, slime and bed are in the modeled sets.
   - Pointed dripstone (`fallOn`), bubble column (`entityInside`), cactus and magma are not modeled, so `inside` = `unmodeled`. A block with a hook is never marked `none`.
   - Beds export `bed: true` and `fallOnMultiplier` 1.0.
   - `suffocating` comes from `state.isSuffocating(level, pos)`.
   - `endPortalOrGateway` is `instanceof EndPortalBlock || EndGatewayBlock`.
   - The `fallOn`-only unmodeled flag also makes the player-inside check unsupported for such blocks. That is conservative, not wrong.

7. **Reader patch.** Correct at its base, obsolete now (finding 5).

8. **Tests.** They run the real driver, with stubs only for the rig and the bot. See findings 6 and 7 for what they cannot catch.
