# Parkour Spiral, flown by hand on 26.3: what the bot's session teaches

Julien drove PathBot on Parkour Spiral 3.0.5 (its own world on the 26.3 rig, `scripts/map.ps1`) with the stick, from 15:37 on 2026-11-27. Source: the 26.3 bot client's log (`D:\path-tracing-rigs\26.3\clients\bot\logs\latest.log`) and RCON block tests on the served world. His words: "it is stuck thinking but also it has trouble with the slime jumps but sometimes it works somehow but rarely, also long distances don't work", then "even the gpu not helping for such short things means we're very wrong".

## What happened to 199 gotos

| Outcome | Gotos | Share |
|---|---|---|
| refused, `NeedsJoints` ("defect uncrossed") | 77 | 39 % |
| thought for at least 5 s first (longest 120 s) | 29 | 15 % |
| no reply: executed, or superseded by his next click (the log cannot tell) | 89 | 45 % |
| already there | 4 | 2 % |

13 replans on "moved by motion". 172 refusals in all, repeats counted.

- **The refused targets are near:** lower bound p50 10 ticks, max 38, so a few blocks.
- **The learning they cost:** learn ticks per refusal p50 97,811, p90 181,202, max 272,404, so about 1–3 s of thinking each.
- **What the defects are:** at the 40 distinct first-exception cells, air 30, slabs 6, a pressure plate 1. Slime sits at or under 1. The defects are the gaps of the jumps.

## The Fermi floor of one jump

A 3-block parkour jump is decided by the takeoff (the sprint-jump gait, steady after a few ticks), the aim (a handful matter) and the jump tick on the approach (about 10). That is about 10³ candidates of 10–20 ticks each: brute force is about 10⁴ law ticks, about 10 ms. With the gait as a stored generator, crossing a gap is a lookup of its arc against the gap: about 1 µs, no search.

Measured: about 10⁵ learn ticks per jump, and then failure. That is 10× a brute-force budget spent without finding what brute force over the right variables finds. The search explores raw ticks over cells and programs, not the jump's variables. Speed cannot fix that. The GPU is not even on this path: it serves the server mind's batches (`engine/mind`), not the bot's planner, and a faster wrong search only fails sooner.

## What it teaches

1. **A gap on uniform ground is the gait's arc, not a defect to learn.** The representation is wrong where every parkour jump becomes a cold learning problem. Spiral is a few arcs against slightly different gaps.
2. **Nothing learned transfers between near-identical jumps.** This matches BlockStep warm against cold: 65 against 68 solved, thinking 56.6 against 51.6 s, flat across tasks (`D:\path-tracing-evidence\blockstep-20261127`). The word keys are too exact, and the store is in memory only: a restart forgets it.
3. **Long gotos multiply cold learns, and one refusal refuses the whole route.** The detour is the cure: route around what is not crossed yet, or move to the least T-to-go and learn while moving (`map/now.md`).
4. **Slime works rarely because only luck finds it.** The law has the bounce, bit-exact (`law/src/basis/body.rs` `restitute`), and word signatures carry it (`word_store.rs`). The field prices every cell as a walking jump, so height gained by a rebound is invisible to it, and only raw learning can find the fall, landing and rebound sequence. A slime cell needs its answer to "fall onto" in the field, from the law, as water needs its stroke.
5. **Not checked:** whether the 13 "moved by motion" replans are slime landings, where a law-against-server bounce difference would make the server correct the velocity. Capturing one bounce from both sides settles it.

## The order this implies

The loop's shape stands; the joint learner is the bottleneck. In order:
1. Gaps as the gait's arcs (and slime and water as field entries from the law).
2. The detour.
3. Words that generalize and a store that persists: his learning session.

Each makes the next smaller.
