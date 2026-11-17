# Run Joints Report (in progress)

## 1. The cutting rule (draft from schema & docs)

From `docs\representation.md` (§10–11), `docs\vocabulary.md` (§4), and the recording JSONL schema observed in `jobs\hard-track\julien-428.jsonl` and `recordings\track-practice1-Brain_Forge-11.jsonl`:

- **Recording structure:**
  - Row 1 (`type: "recording"`): header + `initialPlayer` snapshot (state before `t=0`).
  - Tick rows (`t = 0, 1, ...`): per-tick state (`position`, `velocity`, `onGround`, `horizontalCollision`, `verticalCollision`, `isSprinting`, `isShiftKeyDown`, `pose`, `isInWater`, `onClimbable`, `inputs`, etc.).
  - Final row (`type: "end"`): `ticks` (number of tick rows) and `complete`.

- **Joint definition (to finalize against all recordings):**
  A joint is a tick `t` (comparing row `t` against `initialPlayer` at `t=0`, and row `t` against row `t-1` for `t > 0`) where a law event or held-word key changes:
  1. Takeoff (`onGround`: `true -> false`)
  2. Landing (`onGround`: `false -> true`)
  3. Horizontal-collision change (`horizontalCollision` flips)
  4. Climb starting or ending (`onClimbable` flips)
  5. Pose change (`pose` changes)
  6. Sprint change (`isSprinting` flips)
  7. Key change starting a new held word (discrete `inputs` keys: `forward`, `back`, `left`, `right`, `jump`, `sneak`, `sprint`)
  (Checking across all recordings whether `verticalCollision` on ceiling, `isInWater`, etc. also occur.)

## Findings so far (files read)
- `lab\tracks\runs\hardtrack.jsonl`: 3 runs (`T = 469, 434, 428`). Runs 1 and 2 have `recordingNote` (`Requires hold-style input... sprintWindow: 7`), no recording file. Run 3 (`T = 428`) has `recordings/track-hardtrack-Brain_Forge-1.jsonl` (identical or corresponding to `jobs/hard-track/julien-428.jsonl`).
- `lab\tracks\runs\practice1.jsonl`: 1 run (`T = 8`), `recordings/track-practice1-Brain_Forge-11.jsonl` (11 lines: header, `t=0..8`, end trailer). `A = [-16, 0, -47]`, `B = [-18, 1, -51]` (`bFace = up`).
- `lab\tracks\runs\practice2.jsonl`: 4 runs (`T = 13, 28, 13, 27`).
- `lab\tracks\runs\practice4.jsonl`: 3 runs (`T = 25, 20, 23`).

### Progress update: run index scan & Diversity practices (`practice1..10`)
- Scanned all 69 `lab\tracks\runs\*.jsonl` files and all 77 files in `recordings\`:
  - 48 tracks have `Brain_Forge` `.jsonl` recordings (`hardtrack` run 3, `practice1..10`, `infinity_tower_1..36`, `parkour_race`). All 76 non-`parkour_race` recordings parse cleanly with header + trailer (`end`).
  - `recordings\track-hardtrack-Brain_Forge-1.jsonl` is SHA-256 identical to `jobs\hard-track\julien-428.jsonl`.
  - 18 old tracks (plus runs 1–2 of `hardtrack`) have `Brain_Forge` runs with `recordingNote` and no `.jsonl` recording. 7 tracks (`straight100`, `straight300`, `straight800`, `snow`, `grass`, `infinity_tower_full`, `frosted_ice`) have 0 `Brain_Forge` runs (`straight*` have `PathBot` runs only).
  - Multi-run recorded tracks (for §4): `practice2` (4 runs), `practice4` (3 runs), `practice5` (4 runs), `practice7` (2 runs), `practice9` (7 runs), `infinity_tower_7` (2 runs: `698, 161`), `infinity_tower_21` (2 runs: `3869, 158`).
- Preliminary Diversity practice findings (§3):
  - Direct air/neo jumps over `y=0` floor: `practice5` (`T=11` vs `T=13` floor route `-2`, saving 2t), `practice7` (`T=11` vs `T=28` floor-recovery `-1`), `practice9` (`T=11` straight gap vs `T=13` floor route `-2,-4,-10`, saving 2t).
  - Runs that used `y=0` floor even in best run: `practice2` (`T=13` drops to `y=0` `t=0..4`, then jumps around wall `x=-11.70` `t=5..13`), `practice4` (`T=20` drops to `y=0`, hits wall `x=-3.70`, jumps up `t=18..20`).
  - Single-run practice tracks (no floor route in his runs): `practice1` (`T=8`), `practice3` (`T=11`), `practice6` (`T=11`), `practice8` (`T=8`), `practice10` (`T=11`).
