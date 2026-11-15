# 3.2d-3 slice A (R0): the resident channel and the observed start

**The spec** is `D:\path-tracing-wt\sets\jobs\3.2d-3\last.md`. Read all of it. Your slice is section 7's row A and section 1 (R0). The design behind it is `C:\Users\PASCAL\Desktop\path tracing\jobs\zones-design\design.md`, with its red-team in `redteam-opus.md` in the same folder.

**Why:** Julien (S131): "the terrains complexity should be a mirror of compute's complexity here our path should take µs not tens of ms". Today every click starts a new `plan-track.exe` process. It re-parses the model table, re-simulates the bot's settle instead of reading its real state (`plan-track.rs:23`), and talks over RCON, chat, a file and a log polled every 50 ms.

## Decisions already made (do not reopen them)

1. **The channel is JNI, written by hand in Rust, with no external crates.** The workspace has none and builds `--offline`. The integrator tested this premise on 15 November:
   - a `cdylib` crate built with `cargo build --offline --release` (GNU toolchain) exported `extern "system" fn Java_Probe_answer`;
   - `System.load` of that DLL in the staged JDK (`D:\path-tracing-deps\data\java\java-runtime-epsilon`) returned 42;
   - the JVM warns that native access needs `--enable-native-access=ALL-UNNAMED`. Add that flag to the bot client's launch arguments where the rig's client launcher sets JVM arguments.

   Declare the JNI types and function-table offsets you use yourself, from the JNI specification.
2. **A new workspace member, `engine/native`** (`cdylib`, named `pathtracing_native`), holds the JNI entry points. It depends on `law`, `world`, `words` and `plan`.
   - The mod loads it with `System.load` from the environment variable `PATHTRACING_NATIVE` if set, else `<PATHTRACING_ROOT>\engine\target\release\pathtracing_native.dll`. A missing library is a clear chat refusal, never a fallback to the old process.
3. **The spec's "existing clock/comparator native module" does not exist** (no JNI anywhere in the repo). Build its clock half here:
   - QueryPerformanceCounter and QueryPerformanceFrequency, declared by hand through `extern "system"` from kernel32;
   - exposed to Java as one native method.
   - Every hop is stamped with it, per section 1.
4. **The entry points** are section 1's: `RequestGoal(observation, goal, issue_stamp, version)`, `PublishSourceDelta(delta)`, `ObserveEvent(event, observation)` and `Cancel(version)`. They return `Ready`, `WordPlan`, `Hard` or `Refusal`.

   In this slice, `RequestGoal` runs the **existing** planner (`plan::flight` / `search`) on one worker thread, from the observed state, and never on the client thread. It returns its sentence as a legacy adapter: the existing steps converted in memory to the inputs the mod's playback already consumes. There is no file and no JSONL. Slice F (R4) replaces this adapter with `WordPlan`; label it `legacy_sentence_adapter` so it is easy to remove.
5. **`PlayerState::from_bits`** is the exact inverse of the shared bit schema (`engine/law/src/state.rs`), with a known-field mask.
   - A missing required field refuses; it is never filled by a settle, a rest state, a fed clone or a default saturation.
   - The mod sends its observed bits through the channel.
6. **Source data.** The planner loads its model table and world source once per client world, at join, and keeps them resident.
   - Near data arrives as section and block deltas through the existing packet hooks (`ClientPacketListenerMixin.java`).
   - The region reader stays only as a pinned immutable far source, per section 1. Do not build R3's tree.
7. **The Java `StraightLine` planner is removed.** `Bot.java`'s `goto` and `come` go through the channel.
8. **Not in this slice:** the live baseline capture (`CaptureBaseline`, `ObserverOnly` live runs). Someone else is using this machine (S135). Write the capture code and the `verify.ps1` skeleton (`jobs\3.2d-3\verify.ps1 -Case Slice -Slice A`), run only offline checks, and launch no client, server or game.

## Gate (machine-checked, offline)

- Every `PlayerState` field and known-mask bit round-trips Java → Rust → Java, including a recorded state from `recordings/` (a Rust test plus a Java test in `bot/tests`).
- Missing data is refused.
- No settle, process start, log polling or script-file path remains on the click path. Show it with a grep-based test or a poisoned old path.
- The issuance stamp is taken before any per-command work.
- `cargo test --offline` from `engine/` passes.
- `jobs\1.2-footprints\verify.ps1` catches all its mutants.
- `bot\build.ps1` builds, and `bot\tests\test-tracks.ps1` passes.

## Do not touch

- `engine/law/src/{pattern.rs,footprint.rs}`, `engine/words/src/{class.rs,table.rs}`: slice B (the codec) is being built beside you.
- The law's arithmetic: `movement.rs` is pinned, and L0 is unchanged.

No new search code, no new crates, no push.

**Done means:** the gate passes, and `jobs\3.2d-3-A\last.md` lists every file changed, the gate's commands and outputs, and anything left for slices E, F and K.
