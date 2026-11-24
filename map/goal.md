# The goal

"make minecraft's best path tracing algo in the world, which means we'll have to go first principle, it should be extremely fast to run and get to the point in the shortest amount of time T" (S6). "this is meant to be world class, garbage in, garbage out." (X1)

Path tracing here means pathfinding: a bot reaches the point Julien clicks with the stick in the least time T. Then many bots — up to one billion (`10⁹`) resident on ES generators and scars, with live players in the observer's light cone sharing work on his server (S18, S35, S45): "we're building a cathedral here."

## T, the one currency

- T is server ticks from the click's server tick to arrival with everything restored. Thinking, lag and transport count (S124). A mission's T ends when its cleanup ends (S46).
- Health, food and the inventory are resources spent in T (S6). An irreversible loss is a barrier, never a price.
- T is judged against a bracket: a proven lower bound (the kit's light cone) and the best T found. It is reported as a distribution, never an average alone.
- His best time on a track is the minimum bar; beating him is the aim (S25). Only his time is used, never his path.

## The two extremes, held by one mechanism (S131, S161, S166)

- A million-block straight line on plain ground with bare legs, thought in microseconds in total for every new command (S225).
- A hard track: "many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that" (S166).
- Compute grows with what the terrain holds, never with its length: "the terrains complexity should be a mirror of compute's complexity" (S131).

## The scoreboard

| Row | Measured by |
|---|---|
| T against his time and the bracket | the track clock in his game, `scripts/flight-scoreboard.ps1` |
| thinking per new command (standing, moving), p50 and p99 | `engine/plan/tests/real_time.rs` |
| first move latency: never waits on thinking when the bracket is closed | the flight log |
| deviations: expected against seen, per thousand predictions | the deviation tool (`engine/plan/src/bin/deviation.rs`) |
| cost per bot and server ms per tick at N bots | later (step 9) |

Each row is shown beside its Fermi floor, with the gap in orders of magnitude.

## Floors (Fermi, inputs in `detail/representation.md`)

| Case | Floor | Measured | Gap |
|---|---|---|---|
| a new command on flat ground | ~1–10 µs | 13.5 µs standing, 12.7 µs moving (median); the first on a ground 119 µs | ~0.1 order |
| the same command asked again | ~1 µs | 4.5 µs | ~0.65 order |
| a click to the bot's first input, in his game | ~1 tick | 1.8–2.3 s (`plan-track` started per click, the world read from disk) | ~1.5 orders |
| a law tick | 0.1–0.3 µs | 0.6–0.95 µs | ~0.5 order |
| a new joint pattern | ~12 ms, once | — | — |
| 10⁴ bots deciding | ~0.03 core | — | — |
| 10⁹ bots (ES Form-P + 10⁶ scars, offline probe) | L3 resident, O(1) edit split, < 1 ms observer frame | 4.28 MB (0.0045 B/bot), 172 µs split, 129.8 µs max frame | offline; server flight pending |

## Where the work is

The order of work and its state: [now.md](now.md). The goal of every step: a flight in his game that he can use (S239: "always test in game that's the only kind of test that matters").
