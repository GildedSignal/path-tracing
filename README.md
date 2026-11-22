# Path Tracing

> *"Make Minecraft's best path tracing algo in the world, which means we'll have to go first principles — it should be extremely fast to run and get to the point in the shortest amount of time T. We're building a cathedral here."*

**A first-principles Rust physics & planning engine for Minecraft 26.2.**  
Not a heuristic A\* block-walker: a bit-exact reproduction of Mojang's game (`0.6–0.95 µs/tick` per core) compressed through symmetry, events, and equivalence classes so a million-block run thinks in microseconds, chaotic 3D obstacle courses play effortlessly, tools edit and restore the world through **one mechanism**, and **10,000+ bots** act as real players from **one mind** inside the server.

- **One currency — `T`:** server ticks elapsed from the click's server tick to arrival with the world restored and kit recovered. Thinking time, lag, tool use, and cleanup all count in `T`.
- **Start with [`map/`](map/):** [`goal.md`](map/goal.md) · [`principles.md`](map/principles.md) · [`architecture.md`](map/architecture.md) · [`vocabulary.md`](map/vocabulary.md) · [`work.md`](map/work.md) · [`now.md`](map/now.md)

---

## Ambition & Scope

The long-term target is an AI body and swarm substrate across every dimension of Minecraft:
- **Microsecond acting at unbounded span:** solve uniform terrain in microseconds regardless of distance; solve intricate chained 3D mechanics without per-terrain special cases.
- **Any tool, any terrain edit:** placing/breaking blocks, water buckets, ender pearls, Nether portals, boats, elytra, mounts, pistons, and redstone pulled by **T** alone, with exact cleanup obligations (**tolls with a debt**).
- **10,000+ bots & cooperative 4D building:** one shared mind in the server (borrowing from [Emergent Space](https://github.com/GildedSignal/emergent-space-engine)), coupled only where footprints intersect, lazy between events, scheduling blueprints in `(max, +)` over `(min, +)`.
- **Adversarial & survival resilience:** opposing players modeled honestly at the **two-regime singularity boundary** ([`representation.md`](map/detail/representation.md#L188-L191), long-horizon combat notebook in [`pvp-strategy-2026-11-22.txt`](archive/research/pvp-strategy-2026-11-22.txt)).

---

## Why Existing Minecraft Pathfinders Hit a Wall

| Approach | How it works | Where it breaks |
|---|---|---|
| **Grid A\*** (Baritone, Mineflayer, Azalea) | Throws away velocity and hitboxes; plans over integer blocks with ~20 hand-coded moves and guessed penalty weights. | Blind to momentum (misses head-hitters at `9.734 m/s`, ice tunnels at `19.345 m/s`), fakes tool costs, needs custom search code per mechanic, computes `O(distance)`. |
| **Raw tick A\*** (Tungsten) | Runs A\* directly over real tick physics (`(pos, vel)`). | Solves tight parkour, then blows up exponentially with distance, tool edits, world mutation, and bot count inside JVM world state. |
| **This engine** | Reproduces 26.2 bytecode bit-exact in Rust, then folds spacetime down to feedback **words** and equivalence classes. | Deliberate upfront physics & representation investment before scaling to complex kits and swarms. |

---

## Architecture & Philosophy

1. **The law as composable generators, not Mojang's class tree** ([`engine.md`](map/detail/engine.md))  
   Minecraft's classes mostly instantiate ~5–6 physical patterns (`affine_body`, `aabb_sweep`, `ray_dda`, resource/cooldown clocks, `cell_rewrite` + class-major event calendar) plus parameter tables and thin Mojang quirks (*scars*), leaving genuinely distinct math (`elytra`, `piston`, `PortalForcer`) as separate kernels. Arithmetic copies Java IEEE-754 casts, operation order, and the `65,536`-entry sine table bit-for-bit (never fused multiply-add). Each generator keeps two faces: `k = 1` tick (oracle & tight contacts) and `k = Δt` to the next exit event (time-skipping **word**).

2. **The space under a kit is a crystal with defects** ([`representation.md`](map/detail/representation.md))  
   Four folds collapse the raw `state × world × tick` space:
   - **Translational symmetry:** uniform ground drops position; travel becomes `(word, phase, count)`.
   - **Dissipation:** drag erases velocity history beyond momentum's memory (~263–268 ticks on sprint-jump).
   - **Events:** motion integrates between contacts instead of stepping tick-by-tick.
   - **Kit classes:** 32,366 block states collapse to their answers under the active **kit** (*"a flower is air"*).  
   Compute tracks **defects and surprises**, never path length.

3. **One loop & one mechanism (`solve`)** ([`architecture.md`](map/architecture.md))  
   Bracket **T** via the kit's light cone -> extend the largest certified word -> cut at the first exception -> compose backward reachable **sets** in `(min, +)` -> move the instant the opening bracket closes. Thinking runs while the body executes certified words. A carried tool (`bucket`, `block`, `pearl`, `boat`, `elytra`) is simply a law input — never bespoke search code.

4. **Two regimes at the contact singularity**  
   Outside melee reach, the crystal stays pure and wins by choosing *where* and *when* contact happens; inside melee reach, an autonomous adversary branches across cooldown ticks, handing off across the singularity boundary onto the `k = 1` tick kernel while keeping `Word` certificates strictly pure (`found / forbidden / searched / refused`) and any future learned evaluator strictly advisory.

---

## Engineering Ethic & Principles ([`principles.md`](map/principles.md))

- **The game is the judge:** no fake tests. Ported physics are verified bit-exact against Mojang's actual `server-26.2.jar` (`tests/jar.rs`), and nothing merges to `main` until flown on a real 26.2 server with game-logic mods (Lithium, C2ME) stripped.
- **Fermi first, then measure tails:** every metric is reported beside its physical floor using distributions (`p50`, `p90`, `p99`, worst case), never averages alone.
- **Question, delete, simplify:** every branch starts by cutting dead weight (`clean:`). Shrinking the law keeps hot loops in L1/L3 cache and within one agent's context window; any deletion that leaves jar witnesses, replays, and flights identical is proven sound.
- **SR-71 precision & zero made-up numbers:** bit-exact where a single bit flips outcomes, certified widths elsewhere; irreversible ruin is a hard barrier, not a tuned constant; unmodeled mechanics `refuse!` explicitly instead of guessing.
- **Derived state, never declared** ([`work.md`](map/work.md)): `Sketch` (branch / jar tests) -> `Flying` (wired and flown on `main`) -> `Load-bearing` -> `Frozen`. Every flight records *expected vs. seen*; anomalies are treated as the highest-value signal.

---

## Measured Status ([`now.md`](map/now.md))

- **Flown on `main` (`Load-bearing`):**
  - **Movement law:** `0.6–0.95 µs/tick` on one 7800X3D core (`~5×10⁴×` real time; Fermi floor `~0.1–0.3 µs`).
  - **Exact-tick live flights (`plan-track --mode solve`):** `straight100` `288/288`, `straight300` `845/845`, `straight800` `2,248/2,248`, `stonesimple` `85/85` (human best `86`).
  - **Warm command latency (`real_time.rs`, 60 commands each):** median `13.5 µs` standing / `12.7 µs` moving (first command on a ground `119 µs`; floor `~1–10 µs`).
  - **Resident stick mod (`engine/native` + `bot/`):** flies open ground up to `46` blocks; `NeedsJoints` on fences/stairs, air-lag inspection, and direct word execution in the client are the active bottlenecks on `main`.
- **Port branch `wt/port-fold` (`Sketch`, kept off `main` until compressed & flown):**
  - `32,252 / 32,366` 26.2 block states modeled (`99.65%`; remaining `114` dynamic shulker/moving-piston states already covered in law code), `31,351` Rust lines across `42` files in `engine/law/src/`, verified by `9,117` bit-exact `server-26.2.jar` rows across `18` TSVs (`tests/jar.rs`).
- **Active order of work:** close the 3 remaining local GLM dossiers onto `wt/port-fold` -> compress `wt/port-fold` onto the dual-face generator basis -> finish the 21-track & direct-runner pass on `main` -> wire compressed generators & fly tools/edits slice by slice -> scale to `10⁴` lazy bots and `(max, +)` building.

---

## Repository Layout

| Path | Purpose |
|---|---|
| [`map/`](map/) & [`map/detail/`](map/detail/) | Living single source of truth (`AGENTS.md` + `map/*.md` stay `< 40 KB`, loaded whole on startup/compaction). |
| `engine/law` | Tick physics & world-mutation laws, shapes, sine table (`sin.bin`), jar witness suite (`tests/jar.rs`), recorder replay (`tests/replay.rs`). |
| `engine/world` | Region loader, `server-26.2.jar` block census & property extractor (`ModelTable.java`), snapshot grid. |
| `engine/words` | Feedback programs, law-comparison equivalence classes, canonical gait discovery. |
| `engine/plan` | `solve`, joints & sets, bounds/events, `plan-track`, deviation comparator. |
| `engine/native` | Resident JNI planner channel inside the Fabric bot process. |
| `bot/` | Fabric mod for both clients (stick input, direct runner, track clock, state/input recorder). |
| `scripts/` & `lab/` | Launchers, game runners, meters, scoreboard, GLM jar extraction (`lab/glm.md`). |
| `archive/` | Verbatim turn logs (`archive/turns/inbox.md`), postmortems, research notebooks. |

---

## Running Locally

Toolchains and build caches live on `D:` (`D:\rust\cargo`, `D:\path-tracing-target`), and clients reuse your `%LOCALAPPDATA%\ImmersiveMinecraft\data` Java 25 / Minecraft 26.2 / Fabric install without altering the launcher. Performance mods from your pack stay except Lithium and C2ME, which are stripped from the bot client and the local `127.0.0.1` server so Mojang's own logic runs unaltered.

- **`Play.cmd`** — starts the local 26.2 server and your client (`Brain_Forge`).
- **`Play with bot.cmd`** (or `.\play.ps1`) — starts server, `PathBot` client, and your client (`scripts\setup.ps1` runs on first use).
- **`Stop server.cmd`** — saves and stops cleanly (`scripts\rcon.ps1` sends server commands; `bot\build.ps1` compiles the mod with `javac`).
- **In-game stick controls:** right-click a block with a stick to send `PathBot`; sneak + right-click to stop (`/msg PathBot goto <x> <y> <z>`, `come`, `stop`).
- **Tracks & scoreboard:** `/track` creates/times courses, `/map` switches dimensions, `scripts\play-track.ps1 -Track <name> -Mode solve` flies a solved track, `scripts\flight-scoreboard.ps1` scores `T` vs. predictions and human times.
- **Verification (headless jar & planner suite, inside `engine/`):** `cargo test --offline -p law`, `-p world --lib --test world`, `-p plan --test hard_track --test real_time`.
