# Double-Tap Sprint (`sprintWindow`) Port Specification

**Consumer:** step 3 of `docs\plan.md` (replaying Julien's recorded tracks through `law::tick`, starting with `jobs\hard-track\julien-428.jsonl`).  
**Jar truth:** `lab\results\local-20261116-1008-1-0-port-sprint-window\dossier.md` and `check.md` (grade: `reproduced`, Mojang 26.2 client jar sha1 `2dc72797acbc1b63fc16a11c4ac393605f453754`).

---

## 1. Premises

| Item | Location | Status | Notes |
|---|---|---|---|
| `Input` struct | `engine\law\src\state.rs:6-17` | **EXISTS** | Has `forward`, `back`, `left`, `right`, `jump`, `sneak`, `sprint`, `yaw`, `pitch`. |
| `PlayerState` struct | `engine\law\src\state.rs:52-121` | **EXISTS** | Has `input: Input` (`:55`), `sprinting: bool` (`:65`), `sprint_speed_modifier: bool` (`:67`), `shift_key_down: bool` (`:69`), `riding: bool` (`:114`), `using_item: bool` (`:118`), `use_item_can_sprint: Option<bool>` (`:120`). |
| `PlayerState.sprint_trigger_time` | `engine\law\src\state.rs:52-121` | **MISSING** | `LocalPlayer.sprintTriggerTime` (`protected int`, field `#1107` in `dossier.md:7`, `check.md:25`). |
| `PlayerState.sprint_window` | `engine\law\src\state.rs:52-121` | **MISSING** | `Options.sprintWindow` (`OptionInstance<Integer>`, `IntRange(0, 10)`, default `7`, `dossier.md:5`). |
| `PlayerState::default()` | `engine\law\src\state.rs:122-193` | **EXISTS** | Must initialize the new fields. |
| `PlayerState::bits()` / `from_bits()` | `engine\law\src\state.rs:200-250, 280-374` | **EXISTS** | Exhaustive destructuring (`:203-216`) requires encoding/decoding every `PlayerState` field. |
| `state_fields!` / `encode_as!` | `engine\law\src\state_reads.rs:29-95, 122-130` | **EXISTS / DIFFERS** | Exhaustive destructuring (`:23`) requires listing every `PlayerState` field. `encode_as!` (`:122-130`) supports `bool`, `i32`, `u32`, `u64`, `f32`, `f64`, etc., but has **no `u8` impl** yet (must add `u8` if new fields are `u8`). |
| `tick_movement` | `engine\law\src\movement.rs:118-214` | **DIFFERS** | Overwrites `state.shift_key_down` (`:149`) and `state.input` (`:150`) before calling `update_sprinting` (`:158`). Does not read previous tick's `forward` or `sneak` before `:150`. |
| `update_sprinting` | `engine\law\src\movement.rs:216-233` | **DIFFERS** | Ports `canStartSprinting` (except double-tap branch) + sprint-key start (`input.sprint`, `:220`) + `shouldStopSwimSprinting`/`shouldStopRunSprinting` (`:223-232`). Missing `sprint_trigger_time` decrement, reset, arm, and double-tap start (`LocalPlayer.aiStep:0-16, 343-425`). |
| `jar_sprint_latch_corpus` | `engine\law\src\movement.rs:1360-1385` | **EXISTS** | Calls `update_sprinting` directly (`:1382`); call site must match any signature change. |
| `set_sprinting` | `engine\law\src\attributes.rs:50-54`, `movement.rs:41-45` | **EXISTS** | Sets both `sprinting` and `sprint_speed_modifier` (matches `LivingEntity.setSprinting`, `dossier.md:32`). |
| `recording_header` | `engine\law\tests\replay.rs:361-367` | **DIFFERS** | Line `:366` asserts `sprintWindow == 0.0` (`"unsupported double-tap sprint window"`); does not return `sprintWindow` to `replay_text`. |
| `replay_text` | `engine\law\tests\replay.rs:369-416` | **DIFFERS** | Calls `recording_header(&object)` at `:379` and discards the value; does not thread `sprintWindow` into `state`. |
| `load_state` (`initialPlayer`) | `engine\law\tests\replay.rs:86-150` | **EXISTS / DIFFERS** | Line `:96` loads `state.input` from `initialPlayer.inputs` (which `PlayerSnapshot.java:25` captures at tick 0 `START`, not tick `-1` `ClientInput`; see Open Question 3). Does not load `sprintTriggerTime` (absent from `PlayerSnapshot.java`). |
| `engine\law\Cargo.toml` | `engine\law\Cargo.toml:1-16` | **EXISTS** | `law` has **zero** crate dependencies (`world` depends on `law`, `plan` depends on `law`, `words`, `world`). |
| `hardtrack` offline world | `engine\plan\tests\hard_track.rs:13-35` | **EXISTS** | `build() -> BoxWorld` builds `hardtrack` from `lab\tracks\hardtrack.build.txt` via `world::model`. No `.world.jsonl` exists in `lab\fixtures\` for `julien-428.jsonl`. |
| `Recorder.java` / `TickInput.java` | `bot\src\pathtracing\Recorder.java:148-173`, `TickInput.java:8-24` | **EXISTS** | `recordTrack` records raw keys, `isSprinting`, `sprintSpeedModifier`, and `settings(mc)` (including `sprintWindow`). `holdSettings` still refuses `sprintWindow != 0` for `replay` (`Recorder.java:212`) and `playPlan` (`Recorder.java:43`). |

---

## 2. The Exact Edits

### 2.1 New state fields and bit/read registration (`engine\law\src\state.rs`, `engine\law\src\state_reads.rs`)

**Jar source:**
- `Options.<init>` (`2296–2336`): `sprintWindow` is `OptionInstance<Integer>`, `IntRange(0, 10)` (`2315–2322`), default `7` (`2325`). Used verbatim as tick count (`LocalPlayer.aiStep:413-425`, `dossier.md:5`, `check.md:22`).
- `LocalPlayer.sprintTriggerTime`: `protected int` field `#1107`, read/written only in `LocalPlayer.aiStep` (`0–16`, `343–377`, `391–394`, `405–425`; `dossier.md:7-20`, `check.md:25`).

**Current lines (`engine\law\src\state.rs`):**
```rust
// state.rs:65-68
    pub sprinting: bool,
    /// MOVEMENT_SPEED modifier presence; entity-data packets update only `sprinting`.
    pub sprint_speed_modifier: bool,
```
```rust
// state.rs:137-139 (inside impl Default for PlayerState)
            sprinting: false,
            sprint_speed_modifier: false,
```
```rust
// state.rs:203-221 (inside PlayerState::bits)
        let PlayerState {
            ...
            sprinting, sprint_speed_modifier, ...
        } = self;
        let mut bits = vec![*t, ..., *no_jump_delay as u64, *ticks_frozen as u64];
```
```rust
// state.rs:292-294 (inside PlayerState::from_bits)
        let at = b.at; let n = b.word()?;
        s.food_tick_timer = u32::try_from(n).map_err(|_| Refusal { position: at, reason: "u32" })?;
        s.remaining_fire_ticks = b.i32()?; s.no_jump_delay = b.i32()?; s.ticks_frozen = b.i32()?;
```

**Must become:**
1. Add to `PlayerState` (`engine\law\src\state.rs:68`):
   ```rust
   pub sprint_trigger_time: u8,
   pub sprint_window: u8,
   ```
   (Using `u8` matches `dossier.md:60` and fits `IntRange(0, 10)`.)
2. Add to `PlayerState::default()` (`engine\law\src\state.rs:139`):
   ```rust
   sprint_trigger_time: 0,
   sprint_window: 0,
   ```
   (See Open Question 1 for `0` vs vanilla `7` in `Default`.)
3. Update `PlayerState::bits()` (`engine\law\src\state.rs:203-221`) to bind `sprint_trigger_time, sprint_window` and push `*sprint_trigger_time as u64, *sprint_window as u64` into `bits`, and update `PlayerState::from_bits()` (`engine\law\src\state.rs:294`) to decode both with `u8::try_from(n).map_err(|_| Refusal { position: at, reason: "u8" })?` in the matching positions.
4. Update `engine\law\src\state_reads.rs`:
   - Add `sprint_trigger_time,` and `sprint_window,` to `state_fields!` (`state_reads.rs:94`).
   - Add `u8: |v| [*v as u64],` to `encode_as!` (`state_reads.rs:123`).

---

### 2.2 Forward press edge & previous-tick caches (`engine\law\src\movement.rs`)

**Jar source (`LocalPlayer.aiStep`, `KeyboardInput.tick`, `ClientInput`):**
- `LocalPlayer.aiStep:56-85`: before `this.input.tick()` (`157-161`), `LocalPlayer` caches from the *previous* tick's `ClientInput`:
  - `67–77`: `cachedShift = this.input.keyPresses.shift()` (previous tick's sneak key);
  - `78–85`: `cachedForwardImpulse = this.input.hasForwardImpulse()` (`moveVector.y > 1.0E-5F`, `ClientInput:0-18`).
- `KeyboardInput.tick:0-130`: `calculateImpulse` produces `±1 / 0`; `Vec2.normalized()` (`< 1.0E-4F -> ZERO`) leaves `moveVector.y ∈ {0, 1, ≈0.7071}`, so `hasForwardImpulse()` is exact to `forward && !back` (`dossier.md:13`, `check.md:21`).
- In `engine\law\src\movement.rs:150`, `state.input` holds the previous tick's `Input` (initially `Input::default()`, matching `ClientInput`'s `Input.EMPTY` / `Vec2.ZERO`) right up until `write_state!(world, state.input) = *input;`.

**Current lines (`engine\law\src\movement.rs:149-158`):**
```rust
149:     write_state!(world, state.shift_key_down) = input.sneak;
150:     write_state!(world, state.input) = *input;
151:     if !read_state!(world, state.no_physics) {
...
157:     let slowly = read_state!(world, state.crouching) || (read_state!(world, state.pose) == Pose::Swimming && !read_state!(world, state.in_water));
158:     update_sprinting(state, world, input, slowly);
```

**Must become:**
Read `state.input` before line `150` overwrites it, guarded so that when `sprint_window == 0 && sprint_trigger_time == 0` (double-tap disabled and inactive), `StateField::input` is not spuriously marked read-before-write in `RecordingWorld`:
```rust
    let (prev_forward, prev_sneak) = if read_state!(world, state.sprint_window) > 0
        || read_state!(world, state.sprint_trigger_time) > 0
    {
        let prev = read_state!(world, state.input);
        (prev.forward && !prev.back, prev.sneak)
    } else {
        (false, false)
    };
    write_state!(world, state.shift_key_down) = input.sneak;
    write_state!(world, state.input) = *input;
    ...
    update_sprinting(state, world, input, slowly, prev_forward, prev_sneak);
```
(Justification for the guard: when `sprint_window == 0 && sprint_trigger_time == 0`, `sprint_trigger_time` remains `0` regardless of `prev_forward`/`prev_sneak` and double-tap never fires, so `state.input` has zero effect and existing `words` `StateReads` footprints stay unchanged.)

---

### 2.3 Countdown, reset (including anomaly 10's `isSlowDueToUsingItem`), arm, and double-tap start (`engine\law\src\movement.rs`)

**Jar source (`LocalPlayer.aiStep`, `dossier.md:11-32`, `check.md:19-20`):**
1. **Decrement (`0–16`):** `if (sprintTriggerTime > 0) --sprintTriggerTime;` (first thing in `aiStep`, before movement).
2. **Reset (`343–377`, anomaly 10 at `348`):**
   `if (cachedShift || isSlowDueToUsingItem() && !isPassenger() || input.keyPresses.backward()) { sprintTriggerTime = 0; }`
   where `isSlowDueToUsingItem()` (`1296–1330`) is `isUsingItem() && !useItem.getOrDefault(USE_EFFECTS, DEFAULT).canSprint()` and `isPassenger()` is `state.riding`.
3. **Predicate (`379–383`):** `if (canStartSprinting())` (`3263–3297`) — already matches `!sprinting && forward && possible && (!slowly || underwater) && !slow_using_item` in `movement.rs:220`.
4. **Double-tap branch (`386–425`):** inside `if (canStartSprinting())`:
   - `386–387`: `if (!cachedForwardImpulse)` (i.e. `!prev_forward`):
     - `391–399`: `if (sprintTriggerTime > 0)` → `setSprinting(true)`
     - `405–425`: `else` → `sprintTriggerTime = options.sprintWindow().get()`
5. **Sprint-key branch (`428–443`):** inside `if (canStartSprinting())`, *outside* `if (!cachedForwardImpulse)`:
   - `if (input.keyPresses.sprint()) setSprinting(true);`
6. **Stop branch (`446–487`):** unchanged (`movement.rs:223-232`).

**Current lines (`engine\law\src\movement.rs:216-233`):**
```rust
216: // LocalPlayer.aiStep; LocalPlayer.canStartSprinting; LocalPlayer.shouldStopRunSprinting; LocalPlayer.shouldStopSwimSprinting
217: fn update_sprinting(state: &mut PlayerState, world: &impl World, input: &Input, slowly: bool) {
218:     let forward = input.forward && !input.back;
219:     let possible = read_state!(world, state.food_level) > 6 && (read_state!(world, state.flying) || !read_state!(world, state.in_water) || read_state!(world, state.underwater));
220:     if !read_state!(world, state.sprinting) && forward && possible && (!slowly || read_state!(world, state.underwater)) && input.sprint && !(read_state!(world, state.using_item) && !read_state!(world, state.use_item_can_sprint).expect("used item canSprint")) {
221:         set_sprinting(state, world, true);
222:     }
223:     if read_state!(world, state.sprinting) {
...
233: }
```

**Must become:**
Extract `is_slow_due_to_using_item(state, world)` (shared by `aiStep:348` and `canStartSprinting:3263`), and update `update_sprinting`:
```rust
// LocalPlayer.isSlowDueToUsingItem (bytecode 1296..1330)
fn is_slow_due_to_using_item(state: &PlayerState, world: &impl World) -> bool {
    read_state!(world, state.using_item)
        && !read_state!(world, state.use_item_can_sprint).expect("used item canSprint")
}

// LocalPlayer.aiStep (0..16, 343..487); canStartSprinting; shouldStopRunSprinting; shouldStopSwimSprinting
fn update_sprinting(
    state: &mut PlayerState,
    world: &impl World,
    input: &Input,
    slowly: bool,
    prev_forward: bool,
    prev_sneak: bool,
) {
    if read_state!(world, state.sprint_trigger_time) > 0 {
        modify_state!(world, state.sprint_trigger_time) -= 1;
    }
    let slow_using_item = is_slow_due_to_using_item(state, world);
    if read_state!(world, state.sprint_trigger_time) > 0
        && (prev_sneak || (slow_using_item && !read_state!(world, state.riding)) || input.back)
    {
        write_state!(world, state.sprint_trigger_time) = 0;
    }
    let forward = input.forward && !input.back;
    let possible = read_state!(world, state.food_level) > 6
        && (read_state!(world, state.flying)
            || !read_state!(world, state.in_water)
            || read_state!(world, state.underwater));
    let can_start = !read_state!(world, state.sprinting)
        && forward
        && possible
        && (!slowly || read_state!(world, state.underwater))
        && !slow_using_item;
    if can_start {
        if !prev_forward {
            if read_state!(world, state.sprint_trigger_time) > 0 {
                set_sprinting(state, world, true);
            } else {
                let window = read_state!(world, state.sprint_window);
                if window > 0 {
                    write_state!(world, state.sprint_trigger_time) = window;
                }
            }
        }
        if input.sprint {
            set_sprinting(state, world, true);
        }
    }
    if read_state!(world, state.sprinting) {
        let stop = if read_state!(world, state.swimming) {
            read_state!(world, state.food_level) <= 6
                || !read_state!(world, state.in_water)
                || (!forward && !read_state!(world, state.on_ground) && !input.sneak)
        } else {
            !possible
                || !forward
                || (read_state!(world, state.horizontal_collision)
                    && !read_state!(world, state.minor_horizontal_collision))
        };
        if stop {
            set_sprinting(state, world, false);
        }
    }
}
```
And update `jar_sprint_latch_corpus` (`engine\law\src\movement.rs:1382`) to pass `false, false` for `prev_forward, prev_sneak` (`state.sprint_window` is `0`).

---

### 2.4 `sprintWindow` from recording header & what `replay.rs` stops refusing (`engine\law\tests\replay.rs`)

**Jar source (`Options.<init>` `2296–2336`):**
`sprintWindow` is an integer in `0..=10` (`OptionInstance.IntRange(0, 10)`).

**Current lines (`engine\law\tests\replay.rs:361-367, 378-381`):**
```rust
361: fn recording_header(object: &Object<'_>) {
362:     assert_eq!(object.string("type"), "recording");
363:     for option in ["autoJump", "toggleSprint", "toggleCrouch"] {
364:         assert!(!object.boolean(option), "unsupported recorder option {option}");
365:     }
366:     assert_eq!(object.number("sprintWindow"), 0.0, "unsupported double-tap sprint window");
367: }
```
```rust
378:             if line_index == 0 && object.string("type") == "recording" {
379:                 recording_header(&object);
380:                 continue;
381:             }
```

**What `replay.rs` stops refusing & must become:**
- `recording_header` **stops refusing `sprintWindow` values in `1..=10`** (still refusing `autoJump == true`, `toggleSprint == true`, `toggleCrouch == true`, and out-of-range / non-integer `sprintWindow` outside `0..=10`):
  ```rust
  fn recording_header(object: &Object<'_>) -> u8 {
      assert_eq!(object.string("type"), "recording");
      for option in ["autoJump", "toggleSprint", "toggleCrouch"] {
          assert!(!object.boolean(option), "unsupported recorder option {option}");
      }
      let window = object.number("sprintWindow");
      assert!(
          window.is_finite() && window.fract() == 0.0 && (0.0..=10.0).contains(&window),
          "unsupported double-tap sprint window {window}"
      );
      window as u8
  }
  ```
- In `replay_text` (`engine\law\tests\replay.rs:378-381`):
  ```rust
  if line_index == 0 && object.string("type") == "recording" {
      state.sprint_window = recording_header(&object);
      continue;
  }
  ```
- Update `recorder_header` (`engine\law\tests\replay.rs:500-502`) to assert `0`, `7`, and `10` return `0`, `7`, `10`, and add a panic test for `sprintWindow` `-1`, `11`, and `1.5` (`"unsupported double-tap sprint window"`).

*(Note: `bot\src\pathtracing\TickInput.java:17-24` `holdSettings` still throws on `sprintWindow != 0` for `Bot` `replay`/`playPlan`; `Recorder.recordTrack` at `Recorder.java:151` already uses `TickInput.settings`, so no Java edit is required to replay Julien's recordings.)*

---

## 3. The Proof Test

### 3.1 Full `julien-428.jsonl` bit-exact replay (`engine\plan\tests\hard_track.rs`)

Because `engine\law` has no dependency on `world` (`engine\law\Cargo.toml:1-16`), and `engine\plan\tests\hard_track.rs:13-35` already constructs the exact `BoxWorld` for `hardtrack` via `world::model` and `lab\tracks\hardtrack.build.txt`, the cleanest place for the full `julien-428.jsonl` proof test is `engine\plan\tests\hard_track.rs` (reusing `#[path = "../../law/tests/support/json.rs"] mod json;` — or see Open Question 2 if it must sit in `replay.rs` via an exported `.world.jsonl`).

- **Test name:** `julien_428_replay` in `engine\plan\tests\hard_track.rs`
- **Inputs:**
  - World: `build()` (`hard_track.rs:13-35`, built from `lab\tracks\hardtrack.build.txt`, `difficulty = 0`).
  - Recording: `include_str!("../../../jobs/hard-track/julien-428.jsonl")` (line 1 header with `"sprintWindow":7` and `"initialPlayer"`, lines `2..=430` for ticks `t = 0..=428`, line `431` end trailer `{"type":"end","ticks":429,"complete":true}`).
  - Initial `PlayerState`: parsed from `header.initialPlayer`, with `state.sprint_window = 7` from header and `state.sprint_trigger_time = 0`.
  - Per-tick inputs: `row.object("inputs")` (`forward`, `back`, `left`, `right`, `jump`, `sneak`, `sprint`, `yaw`, `pitch`) fed directly to `law::tick(&mut state, &world, &input)`. **Do not overwrite `state.sprinting` or `state.sprint_speed_modifier` from `startSprinting`/`startSprintSpeedModifier` between ticks** — assert them instead, so the law proves continuity across all 429 ticks!
- **Assertions (every tick `t = 0..=428`):**
  1. Pre-tick: `state.sprinting == row.boolean("startSprinting")` and `state.sprint_speed_modifier == row.boolean("startSprintSpeedModifier")`.
  2. Post-tick sprint: `state.sprinting == row.boolean("isSprinting")` and `state.sprint_speed_modifier == row.boolean("sprintSpeedModifier")`.
  3. Post-tick bit-exact state: `position.{x,y,z}.to_bits()` == `xBits`/`yBits`/`zBits`, `velocity.{x,y,z}.to_bits()` == `xBits`/`yBits`/`zBits`, `yaw.to_bits()` == `yRotBits`, `pitch.to_bits()` == `xRotBits`, `fall_distance.to_bits()` == `fallDistanceBits`, `on_ground`, `horizontal_collision`, `vertical_collision`, `shift_key_down`, `in_water`, `on_climbable`, `pose`.
  4. End trailer: `ticks == 429`, `complete == true`.

### 3.2 Double-tap positive tests and negative control (`engine\law\tests\replay.rs`)

Since `julien-428.jsonl` holds `W` across `t = 6..266` and starts sprinting via `inputs.sprint` at `t = 267` and `t = 419` (`dossier.md:51`, `check.md:48`), `engine\law\tests\replay.rs` must also test the double-tap path directly via `replay_text` (or `tick`):

- **Test name:** `double_tap_sprint_window_and_negative_control` in `engine\law\tests\replay.rs`
- **Inputs & assertions:**
  1. **Positive pattern (`sprintWindow = 7`, `r = 1..=5` vs `r = 6`):** on `stationary_fixture()`, `forward = true` at `t = 0`, `forward = false` for `r` ticks, `forward = true` at `t = r + 1` (with `sprint = false` throughout):
     - For `r ∈ 1..=5`, `state.sprinting` becomes `true` at `t = r + 1` (matching `dossier.md:43`: `start@2..6`).
     - For `r = 6`, `state.sprinting` stays `false` at `t = 7` (and `sprint_trigger_time` re-arms to `7`).
  2. **Boundary windows (`1` and `10`):** `sprintWindow = 1` never starts on `r = 0..=2`; `sprintWindow = 10` starts on `r = 1..=8` and not on `r = 9`.
  3. **Reset gates (`cachedShift`, `isSlowDueToUsingItem`, `input.back`):** arming at `t = 0` (`sprintWindow = 7`), then asserting no sprint at `t = 2` if `t = 1` had `sneak = true` (`cachedShift` fires one tick later), OR `t = 2` has `back = true`, OR `t = 2` has `using_item = true, use_item_can_sprint = Some(false)` (anomaly 10).
  4. **Negative control (`sprintWindow = 0`):** run the exact double-tap pattern (`forward = true` at `t = 0`, `forward = false` at `t = 1`, `forward = true` at `t = 2`, `sprint = false`) through `replay_text` with header `"sprintWindow":0`: assert `state.sprinting == false` and `state.sprint_trigger_time == 0` on every tick.

---

## 4. His Runs

`docs\HANDOFF.md:22` notes "51 tracks raced with recorded inputs"; `jobs\flight-corpus\report.md:3-7, 176-187` gives the complete 73-track audit of `lab\tracks\runs\*.jsonl` and `recordings\`:

| Group (count) | Tracks | Recorded `.jsonl`? | Replays after this change? | Reason if still blocked |
|---|---|---:|---|---|
| **Hard track (1)** | `hardtrack` (run 3, `T=428`) | Yes (`track-hardtrack-Brain_Forge-1.jsonl`, `sprintWindow: 7`) | **Yes** (law & world ready) | World built by `hardtrack.build.txt`; 0 diffs across all 429 ticks (`dossier.md:49`). *(Runs 1–2 have no `.jsonl`: `recordingNote`.)* |
| **Diversity practice (10)** | `practice1..10` | Yes (`sprintWindow: 7`, survival, `itemsUsed: {}`) | **Law-unblocked** *(needs world export)* | `flight-corpus\report.md:48-57`: no `.world.jsonl` fixture exported yet (`exact blocks unknown`); once exported, replays unless an unknown block hits an unmodeled hook. |
| **Infinity Tower (36)** | `infinity_tower_1..36` | Yes (`sprintWindow: 7`, survival, `itemsUsed: {}`) | **Law-unblocked** *(needs world export)* | `flight-corpus\report.md:63-104`: no `.world.jsonl` fixture exported yet (`exact blocks unknown`); `infinity_tower_36` has `egg: 1` in inventory with `itemsUsed: {}`. |
| **Parkour Race (1)** | `parkour_race` | Yes (`track-parkour_race-Brain_Forge-1.jsonl`, `sprintWindow: 7`) | **No** | `flight-corpus\report.md:109, 165, 182`; `recordings\track-parkour_race-Brain_Forge-1.jsonl:10497-10510`: toggles **Creative/Survival** and includes **Creative flight** (`movement.rs:133-135` refuses `abilities.flying`), with no `"anomaly"` / gamemode-switch support, plus no `.world.jsonl` export. |
| **Old 18 tracks (18)** | `amertystcoral`, `ametwoodbox`, `birchparkourmedium`, `butoonpressureplates`, `carpetwater`, `doorcorridor`, `doorparkourblue`, `fencemaze`, `fences`, `fencestraightbridge`, `flowers`, `grassjumpeasy`, `mangrovejumpeasy`, `mud`, `redfakemaze`, `stonesimple`, `stonestaircase`, `trapdoorparkour` | **No `Brain_Forge` `.jsonl`** *(8 have `PathBot` `.jsonl` at `sprintWindow: 0`)* | **No** | All `Brain_Forge` runs carry `"recordingNote":"java.lang.IllegalStateException: Requires hold-style input: {\"toggleSprint\":false,\"toggleCrouch\":false,\"autoJump\":false,\"sprintWindow\":7}"` (`flight-corpus\report.md:178-181`, `lab\tracks\runs\ametwoodbox.jsonl:1-2`, `butoonpressureplates.jsonl:1`). Additionally, 6 require unported block place/interact/door/trapdoor (`ametwoodbox`, `mud`, `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `amertystcoral`) and `butoonpressureplates` has unmodeled pressure-plate hooks (`flight-corpus\report.md:158-163`). |
| **Bot-only straight lines (3)** | `straight100`, `straight300`, `straight800` | `PathBot` `.jsonl` only (`sprintWindow: 0`) | *(Already `sprintWindow: 0`)* | 0 `Brain_Forge` runs (`flight-corpus\report.md:6, 186`). |
| **Unfinished / 0 runs (4)** | `snow`, `grass`, `infinity_tower_full`, `frosted_ice` | **No** | **No** | 0 completed runs, no recordings (`flight-corpus\report.md:6, 187`); `frosted_ice` also names bubble columns, jump boost, and dynamic water/ice (`flight-corpus\report.md:110, 169-174`). |

**Summary across the 48 `Brain_Forge` `.jsonl` recordings (+3 `PathBot` straight tracks = 51):**
- **47 / 48** `Brain_Forge` `.jsonl` recordings (`hardtrack` run 3 + `practice1..10` + `infinity_tower_1..36`) are unblocked by this port (`hardtrack` immediately with `hardtrack.build.txt`; the 46 others once their world snapshots are exported and assuming no unmodeled block hooks).
- **1 / 48** (`parkour_race`) still cannot replay due to Creative flight / gamemode switches.
- **18** older `Brain_Forge` tracks still have no `.jsonl` recording at all (`recordingNote`).

---

## 5. Open Questions for the Integrator

1. **Default value of `PlayerState::default().sprint_window` (`0` vs `7`):**
   - *Options:* (a) `0` (double-tap disabled unless set from header or caller); (b) `7` (vanilla `Options.<init>` default at offset `2325`).
   - *Recommendation:* **`0`**. Keeping `0` in `PlayerState::default()` preserves exact behavior for all existing `words`/`plan` searches, `jar_sprint_latch_corpus`, and headerless `replay` tests, while `replay_text` sets `state.sprint_window` from the recording header (`0..=10`).
2. **Where `julien_428_replay` lives (`engine\plan\tests\hard_track.rs` vs `engine\law\tests\replay.rs`):**
   - *Options:* (a) `julien_428_replay` in `engine\plan\tests\hard_track.rs` using existing `build()` + `world::model`, plus double-tap & negative-control tests in `engine\law\tests\replay.rs`; (b) export `hardtrack` to a committed `.world.jsonl` under `engine\law\tests\` so `replay_text` runs `julien-428.jsonl` inside `replay.rs`.
   - *Recommendation:* **(a)**. `engine\law` has no dependency on `world`, and `hard_track.rs` already owns `hardtrack.build.txt` → `BoxWorld`.
3. **`load_state` (`engine\law\tests\replay.rs:96`) reading `initialPlayer.inputs` as tick `-1`'s `ClientInput`:**
   - *Facts:* `PlayerSnapshot.java:25` captures `initialPlayer.inputs` from `TickInput.capture(mc)` at `START_CLIENT_TICK` of `t = 0` (the keys *for* tick `0`, not tick `-1`'s `LocalPlayer.input`), and `initialPlayer` does not capture `sprintTriggerTime`. On `julien-428.jsonl` this is harmless (`isSprinting` is already `true` at `t = 0`, and `W` is held from `t = 0..266`), and `practice2..5` / `infinity_tower_1,36` start with non-zero velocity on A. However, if a future recording starts stationary (`forward = false` at `t = -1`) and double-taps starting at `t = 0`, seeding `state.input` from `initialPlayer.inputs` (`t = 0`'s `forward = true`) would hide the `t = 0` press edge.
   - *Options:* (a) Keep `replay.rs:96` as-is for now; (b) change `load_state` to leave `state.input = Input::default()` (or later have `PlayerSnapshot.java` record `prevInput` and `sprintTriggerTime` separately).
   - *Recommendation:* **(a) for this port**, noting the limitation if a stationary-start `t = 0` double-tap recording is later added.
4. **Updating `TickInput.holdSettings` (`bot\src\pathtracing\TickInput.java:17-24`)?**
   - *Options:* (a) Leave `holdSettings` refusing `sprintWindow != 0` on the bot client; (b) allow `sprintWindow` on the bot too.
   - *Recommendation:* **(a)** — out of scope for the law/replay port; the bot client should stay hold-style (`sprintWindow == 0`).
