# The player's hot state, and the jump to the next cause

**Goal:** the core holds the player in a compact hot state (each fact once, what a plain move touches, the wire unchanged at the boundary) and gains its event face, state + held input → the next cause and the state there, byte-identical to stepping the core tick by tick, so a word jumps from cause to cause.

Builder: Opus (native sub-agent of the integrator). Worktree `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (at `f24a6317` or later). Read first: `map/` (goal, principles, architecture, now, vocabulary), `jobs/hot-core-rebuild/prompt.md` and `report.md` (the last pass, its method, which still holds), `jobs/law-grouping-audit/table.md` (section 5: the `PlayerState` byte split), `jobs/event-forms/report.md` and `map/detail/engine.md` (the event forms, the binade rule), all in `C:\Users\PASCAL\Desktop\path tracing`. Commit on `wt/port-fold` in small commits; never push, never touch `main` or other branches, his game or server.

## His words (verbatim, 23 November)

"Minecraft shall be seen as a causal chain engine which simplifies a lot, ticks are not really needed at all they are a by-product of a way of thinking not of behvavior, behavior is king." — "The goal is to have to do less and less work each pass we'll make is aimed at improving scalability and scaling laws, making each subsequent things easier, easier test less lines for tests and so on." — "it should ask itself if the thing it is looking is actually useful or not for vanilla behavior and think with the constraints simple as that the how it was made before doesn't matter."

## The method (as in the hot-core pass, which worked)

- **Behaviour is king; the law is a causal chain.** A cause fires its effect; between causes a generator carries the state. A tick is derived only where behaviour is observed per tick.
- **The core is the oracle, frozen while you build.** Build beside it, judge against it bit for bit, then switch and delete what the switch made dead in one commit.
- **Think from the constraints, not from what is there:** the witnesses, the exactness tier, T, the cache line, the information floor. The old code says what, never how.
- **Useful for vanilla behaviour, or gone,** and a drop is checked, not declared: only when no recording, jar row or `real_time` line reaches it, listed with that evidence.
- **Spartan on duplication:** every fact held once, every value computed once, one path per shape.
- **The table is emergent:** groups and estimates change when the constraints demand; write each change and why.
- **Tests never name an internal helper;** a witness is state in → state out at a face. Witness rows never fall.
- **A disagreement is data:** logged as a reproducible row, explained, never dropped.

## 1. The hot state

- **Each fact once.** Yaw and pitch live in the input, not beside it; `movement_this_tick` is the step's output, not state; what is recomputed every tick is not stored. What a plain move touches is hot (the table measured ~96 B of 296 B); lukewarm and cold fields sit apart, reached only when a cause needs them.
- **The wire is a boundary, not the layout.** The mod's channel (`Recorder.java`, the `wire!` order in `state.rs`) and the planner's codec keep their bytes: they encode and decode the state, they do not dictate it. Byte-identical wire words on every recording are part of the gate; the mod is not touched.
- Report the hot state's size, its cache lines, and the bench.

## 2. The event face

- **State + held input → the next cause and the state there.** A cause is what changes the law's branch: a contact or collision, a cell boundary that changes friction, speed or a block effect, a clock threshold, a flag machine's switch, a binade edge of the float recurrence. Between causes the generator carries the state; the face jumps there in one call.
- **Exactness by construction, not by approximation:** inside a binade the drag recurrence is an exact integer map (the event forms; `engine/plan/src/events.rs` holds the judged P2 kernel). Never a linear jet, a nearest-cell table or a re-rounded state. Where no exact form exists yet, the face steps the core and says so.
- **The measure of the face is the share of recorded ticks it covers by jumping** (over the 31 gated recordings), not its correctness alone: stepping everything is always correct and worth nothing. Stepping is the exception: every stepped stretch kind is listed with its tick count and the reason no exact form exists for it.
- **Judged against the tick face:** for every held-input stretch in the 31 gated recordings and in extremes (long slides, falls through many binades, every block effect the hot layer knows), jump(state, input) equals stepping the core to the same cause, bit for bit, with the same footprints. Disagreements go to `jobs/hot-state-event-face/anomalies.tsv` in the main checkout, each explained.
- **One mechanism:** the planner's event maps (`events.rs`) and the core's face must become one. Astra changes `plan` on `main` tonight, so do not edit `engine/plan/` in this pass: build the face in `law` with `events.rs` as a second oracle, and list what in `plan` it replaces, for the integrator to switch after merging `main`. So this pass grows `engine/`: report it plainly, never as progress. **Cap:** the face comes in smaller than the 575 lines of `events.rs` it replaces, so the switch is a net deletion; over it, stop and say why the representation is wrong.
- Report the stretch lengths the face jumps (distribution over the recordings: n, median, p90, max) and its speed against stepping.

## Conditions

- Every commit green: `cargo test -p law --release --lib --test jar --test replay` (both `replay` and `tracks` run on the untracked `lab/fixtures/flat-ground-replay/`, `lab/track-world/` and `recordings/`), the planner's `real_time` byte-identical to `f24a6317`, the workspace builds.
- Each commit message prints code and test lines over the whole of `engine/` before → after, in decades (`scripts/engine-lines.sh`), and the bench ratio against `f24a6317` with its spread (`scripts/bench-judge.sh`). The commit-msg hook wants a `map/` change or `Map: unchanged, because …`, and a `Bought:` line when `engine/` grows. End each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Toolchain on D: (`export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo`), cargo from `engine/`. Edit with the Edit/Write tools. Do not edit `map/`. Commit often: the 5-hour window may end the run.

## Output

Your final message is the report: commits (hash, one line), code and test lines before → after and in decades, the hot state's size and cache lines, the wire check, the causes the face knows, the share of recorded ticks it covers by jumping, the stretches it jumps (distribution) and each kind it still steps with its reason, the face's lines against the 575 of `events.rs`, its speed against stepping, the anomalies and how each was explained, what in `plan` it replaces, the bench, what is left, and the lines the map should change.
