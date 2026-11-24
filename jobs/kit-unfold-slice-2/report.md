# Kit Unfolding Slice 2: report

Branch `wt/kit-unfold-slice-2` from `537a70ac`, three local commits (2A wire, 2B words, 2C tracks + map). Nothing pushed. Nothing flown: the server and his game were off-limits, so every number below is offline.

## What changed

**2A: live `KitSpec`, `StateId` and `KitOp` on the wire** (`engine/native`, `bot`, one rule in `plan`)
- `resident.rs` `Source` keeps each palette entry's canonical `StateId`, and derives the block from it with `block_at(p)`, as `WorldSnapshot` does. So the fact is held once, and `Source::state_id` gives `GeneratorSuperflatSource::from_world` the live states.
- `kit_wire.rs` reads `PlayerSnapshot.context`'s own body (`meta[7..]`: dimension, mode and `TrackInventoryCodec`'s 41 SNBT slots, hotbar at list index 31..39, feet at 3, chest at 1) into `Kit::Carried(KitSpec)`. It uses a small JSON/SNBT scanner because the workspace has no dependencies. Decisions:
  - Game modes:
    - **survival** gets the full kit.
    - **adventure** and **creative** keep only the hand (`interact`). Creative's instant breaks and endless stacks are not modelled, so it does not get the edit verbs.
    - **spectator**, or any body that cannot be read (for example the existing `[123, 125]` test body), is `BareLegs`.
  - Tools:
    - Tool tiers come from the hotbar.
    - Gold is counted as tier 1 and copper as tier 2. Both are lower than their real speed, so a planned break never ends before the real one.
    - One efficiency level and one silk flag stand for all tools: the smallest efficiency, and silk only if every tool has it.
  - Placeable blocks: the largest hotbar stack of a non-falling, full, sturdy, non-fluid, non-interactive block.
  - Counted items: water and powder-snow buckets, pearls, boats, elytra, and boots (depth strider, soul speed, leather).
  - The Nether sets `DIM_WATER_EVAPORATES | DIM_FAST_LAVA`. `restore_required` is always true.
- **One landing rule** (`KitOp::lands_at`), used by `certify`, `step_kit_node`, the native replay and the test replay:
  - A click lands before its step's first tick.
  - A break lands before its *last* tick. The client calls `startDestroyBlock` + `continueDestroyBlock` on the first tick, which adds the first progress, and the `n`-th call destroys the block before that tick's movement. Slice 1 landed breaks after the step, one tick late.
  - No Slice 1 number moved.
- **Wire layout.** A plan without edits stays `WORD_PLAN = 1`, word for word. A test proves this: the survival kit on flat ground sends exactly the bare answer. A plan with edits is `KIT_PLAN = 4`:

  ```
  [4, version, issue, entered, now, revision, t, count, n, bits[n], inputs[count × 3],
   a, actions[a × 10], hx0, hz0, hx1, hz1]
  action = [tick, slot 0..8, opcode, x, y, z, face 0..5 (D,U,N,S,W,E), aim_yaw_bits, aim_pitch_bits, hold_ticks]
  opcode: 1 Interact, 2 Place, 3 BreakHold, 4 BucketEmpty, 5 BucketPickup
  ```

  - `(x, y, z)` is the edited cell.
  - Place and BucketEmpty click the neighbour `cell − normal(face)`; the other opcodes click the cell itself.
  - The look aims at the clicked face's centre from the eye, as `Entity.lookAt` computes it, at the action's tick.
  - On a break's ticks the input stream's movement look is that same aim with no keys pressed. The look does not change the physics when no key is pressed.
  - The slot is chosen per action from the hotbar:
    - Interact uses the first empty hotbar slot, so a door opens even while sneaking.
    - Break uses the best tool for the block (`kit::tool_for`), or the hand.
    - Place uses the slot holding that block item.
    - A bucket pickup uses the slot the bucket was emptied from.
  - A restore that needs a block not in the hotbar is refused with its reason: item drops are not modelled.
- **Java (`KitAction`, `ResidentRunner`, `Recorder`)**
  - Each record is played at the start of its tick, after the movement input. The bot selects the slot and turns its look to the aim. A click calls `useItemOn`, then `useItem` if the block lets the click pass (as vanilla `startUseItem` does), swings the arm, and turns the look back so the movement keeps its yaw. A break calls `startDestroyBlock` + `continueDestroyBlock` on its first tick and `continueDestroyBlock` after.
  - If the block still stands on the break's last tick, that tick is held and replayed, up to 8 times. The cause is the client's 5-tick destroy delay after a previous break, or float accumulation. The extra ticks count in T.
  - `cancelPlan` calls `stopDestroyBlock`.
- **Self-edit tolerance.** `ResidentRunner.sourceChanged` lets a block echo inside the corridor through only when an action already issued (`action.tick ≤ planTick − 1`) edits that same cell. For Interact the door's other half (y ± 1) also counts. Every other edit in the corridor still stops the flight. `test-native.ps1` covers this.

**2B: coordinate-free kit words** (`word_store`, `defect` Phase A and the kit sections, `solve` promotion)
- A click is stored as `AimRule::Click(KitTemplate)` inside the existing word. The template holds:
  - the verb;
  - the cell as `(fwd, up, right)` from the body's cell in `approach_frame`;
  - a restore flag (the after-state then comes from the open obligation);
  - a goal-offset yaw.

  It holds no world coordinate and no block name. Words record whether they contain a click. Only a carried kit is offered such words; bare legs and Phase B's local proposals never see them.
- A crossing with clicks is promoted as **one motif**, a whole compound indexed alone at its first step's defect. An edit only means something together with its restore. A first try promoted kit routes step by step, and that filled Phase B's local proposals at other kit sites: R-lego-2 ran out of budget by 27 ticks. Keeping the motif whole leaves every bare-legs number byte-identical (`generalize`).
- `expand_kit_ops` now records an observation for each kit step, so the obs line stays aligned with the steps (in Slice 1 it was not). Edits, ledger and counts are booked through one helper (`book`).
- Phase A now runs for kits too (`exec_kit_chain`), on the node's delta. Movement atomics stop on the same kit events Phase B learned them with. Click atomics are made concrete against the live cell with `KitView` verbs, `can_place_at`, `BreakProjection` and the `ObligationLedger`.
- Ownership: the `defect.rs` header, `ArenaNode`, `exec_atomic_raw`, `exec_word_chain` and Phase B are untouched. The new types are named by path inside my functions.

**2C: survival tracks** (`kit.rs`, `expand_kit_ops`)
- **Placement law fix.** `can_place_at` used to require a *sturdy* neighbour face. Vanilla places against any face of a block that has a shape and is neither replaceable nor used by the hand: you can place on mud's top face, for example. The shared `kit::place_faces` now holds that rule, and the wire uses it to choose the clicked face (visible faces first, then the nearest).
- **The pillar.** When the bot is airborne and the goal's standing level is above its feet cell, the cell under its feet is always offered as a placement (jump, place below, land). This is how both of his mud runs climb. Before, only the two cells the topo field ranked nearest the goal were offered.
- The test's "full survival kit" carries a silk diamond pickaxe, axe and shovel. Before, it had only a pickaxe, so each oak log took about 60 ticks and `ametwoodbox` ran out of budget.

## Verification (raw, after the last edit)

| Gate | Result |
|---|---|
| `cargo test --offline -p pathtracing_native` | lib 7/7 (5 new: live meta → `KitSpec` + Nether flag, SNBT items, `Source::state_id` through `from_world`, flat ground with kit = `WORD_PLAN` byte-identical, 2-high wall → `KIT_PLAN` `T=30`, 2 actions, slots 0/1, break ticks still and aimed, every cell restored) + schema 2/2 |
| `bot\build.ps1` | `Built …\bot\build\pathtracing.jar`, exit 0 (javac prints its usual deprecation note) |
| `bot\tests\test-native.ps1` | `PASS: every field/mask Java -> Rust -> Java; every unknown refused; pinned recording preserved` (+ new `KitAction` decode and tolerance checks). The script now respects an existing `CARGO_TARGET_DIR`. Run with `-Evidence` in my target directory |
| `bot\tests\test-clock.ps1` | 448 cases passed |
| `bot\tests\test-full-tick-demand.ps1` | passes (`timer produced 24, executed 10`) |
| `bot\tests\test-tracks.ps1` | **fails on the baseline too**: run on stashed (baseline) bot sources, it stops on the same `AssertionError: Start tick is zero` |
| `bot\tests\test-resident-path.ps1` | **fails on the baseline too**: it greps `resident.rs` for `start: state.clone()`, which is absent at `537a70ac` |
| `kit_unfold` | **6/6**. Zero tax `bare_T=60 kit_T=60`, median 5.5–7.0 µs (≤ 25 µs). 10,000 kits in `356,196 B` (< 1 MB). KT16 0/500 violations |
| `generalize` | 5/5, every `XFER` line identical to the baseline |
| `hard_track` | 5/5 |
| `real_time` | 0 refused, 0 warm `law_ticks`. Medians in three runs: standing 15.6 / 13.75 / 14.6 µs, moving 15.85 / 13.7 / 13.65 µs, repeat 5.2 µs. The baseline `537a70ac`, built the same way, gives standing 11.8 / 14.05 µs, moving 13.0 / 13.1 µs, repeat 5.2 µs: no difference beyond noise. Warm `allocated_bytes=168` per command in every series, the same at the baseline, so it predates this job (the gate's "0 warm allocated bytes" does not hold there either) |
| `world` | 21/21 (7 + 0 + 7 + 7 + 0) |
| `law --lib` | 95/95 |

The track tests give the same results on the committed (trimmed) region files as on the untrimmed working copies. I swapped them in, ran, and restored them byte-for-byte.

**Translated + rotated generalization (`G-kit`, one shared `GeneratorStore`)**

| Motif | Site A | Site B (+25, 0, +17), turned 90° |
|---|---|---|
| oak door corridor (`interact_only`) | T=24, learn 8,191 | T=24, **learn 0**, 1 proposal hit, 2 edits, replayed and restored |
| 2-high stone wall breach (silk pickaxe) | T=38, learn 9,548 | T=38, **learn 0**, 1 proposal hit, 2 edits, replayed and restored |

For site B's certified route, `edit()` at a cell outside its footprint drops 0 routes, and the route is still served. A hole in the walked floor drops it, and it is no longer served.

**Tracks** (all replayed through `WorldDelta` + `law::try_tick`; restored ones checked cell by cell)

| Track | Result | His best |
|---|---|---|
| `doorcorridor` | T=32 restored (Slice 1, unchanged) | 99 |
| `amertystcoral` | **T=33 restored**, learn 9,459, 2 edits (trapdoor opened and closed); 31 unrestored | 63 |
| `ametwoodbox` | **T=45 restored**, learn 72,446, 4 edits: a cobblestone step, the top log broken with the axe, the step broken, the log placed back | 124 |
| `mud` | **T=47 unrestored**, learn 110,480, 8 edits | 97 |
| `mud` restored | `NeedsJoints` at the 117,600-tick cap | — |

Why restored mud does not close: the pillar's base under the walkway is out of reach (4.88 > 4.5) from the walkway. The restoring plan (breach the mud box, rebuild it, pillar outside at z = 78, jump, break the pillar from the walkway) is about 9 edits deep, and Phase B's budget and `TopoField` do not reach it. Both belong to `plan-spine-simplify`, so I did not touch them.

**Ticks that moved**
- R-lego-3 mid-air bridge: 48 → 50. Phase A now serves it from an R-lego-1 motif and takes its first hit.
- R-lego-2 step-up gap: 54 → 60, with learning 48,640 → 38,310. The pillar candidate reorders that search. Disabling only the pillar gives 54 again.

I kept the pillar because it closes `mud`, a real track: 47 against his 97.

## Net lines

`scripts/engine-lines.sh` (rustfmt):

| | code | tests |
|---|---|---|
| `537a70ac` | 36,560 | 21,090 |
| now | 37,439 | 21,585 |
| change | **+879** | **+495** |

`bot/`: `git diff --stat 537a70ac -- bot` gives 5 files, +118 / −9. `KitAction.java` is new (68 lines); the rest is in `ResidentRunner`, `Recorder`, `NativeChannelTests` and `test-native.ps1`. Spent:
- native: the wire reader, 317 lines with ~110 of tests;
- defect: Phase A for kits, the template, book and instantiate helpers;
- word_store: the template and the general `intern`.

Deleted: certify's and the test's duplicated `apply_op`, `step_kit_node`'s `apply_at_end` plumbing, and the candidates' hand-written count deltas.

## Not done, and the gaps found

- **No live flight.** The server and his world were closed to this job. The wire builds and every offline Java/JNI check passes, but nothing here has been judged by the game. The first flights should be `doorcorridor`, then `ametwoodbox` with an axe in the hotbar.
- **Item drops are not modelled.** A restore assumes the broken block's drop; in the game, the item entity waits 10 ticks before it can be picked up. The wire therefore refuses a restore when the hotbar lacks that block. `ametwoodbox`'s restore needs an `oak_log` in the hotbar, as his own run carried.
- **The client's 5-tick destroy delay** after a previous break is not yet a law input. The Java runner holds a break's last tick until the block falls (up to 8 ticks), so T stretches instead of the flight failing.
- **Placement edges.** Clicking a block with a use action (a chest or furnace face) as a placement support would open it; only doors and switches are excluded. A hidden face may be clicked (the vanilla server allows it), but visible faces are preferred.
- **Interact with a full hotbar while sneaking** uses the selected slot. If that slot holds an item, vanilla skips the door.
- **Phase A takes its first hit.** A reused motif is not re-optimized: R-lego-3 is 50, against 48 found by search.
