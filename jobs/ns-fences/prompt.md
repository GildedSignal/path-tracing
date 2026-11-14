# Read job: why does the planner find no sentence on fences and redfakemaze?

You read code and evidence and explain a cause; you edit nothing but your report. Every claim cites file and line, or the evidence file and line.

**Why.** Julien built 18 tracks and raced them himself. The current planner (on main, `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\`) certifies a sentence on 8, flown live with 0 corrections. On five movement tracks it finds **no sentence**: the forward search runs out of its time budget after 100k–214k expansions and millions of replays. The next planner design (red-teamed, `jobs/3.2d-3-design/redteam-opus.md` finding 1) gives such "hard spans" a solver: the existing forward search scoped to one span, with time spent in every key, FIFO ties, cells compared by the law's answers, and the arc certificate instead of tick-by-tick arcs. Your diagnosis tells that design what these tracks actually need. Consumer: the 3.2d-3 build.

**Evidence, per track** (all under `C:\Users\PASCAL\Desktop\path tracing\`):
- `jobs/nosentence-evidence/<track>.txt`: the planner's failure line (time used, expansions, replays, inventions, the state nearest the goal by its order).
- `jobs/nosentence-evidence/<track>.blocks.txt`: every non-air block in the track's box, one per line, `x y z block_state`. The player usually stands at y = -60 on a -61 floor; the blocks say where not.
- `lab/tracks/<track>.json` (A and B), `lab/tracks/<track>.kit.json` (what the player carries), `lab/tracks/runs/<track>.jsonl` (Julien's runs and times).
- `engine/world/tests/fixtures/tracks.txt` (each track's box and his best times).
- The planner: `engine/plan/src/search.rs`, `arith.rs`, `reach.rs`, `flight.rs`; the words: `engine/words/src/` (`class.rs`, `program.rs` with the 54 programs, `segment.rs`, `table.rs`).
- Known planner faults, already diagnosed: `jobs/butoon-diagnosis/last.md` (a flag or pointer treated as a different cell; ties popping the last-made node; greedy order by distance alone) and `jobs/es-audit/last.md` (segments cut at every block crossing; cell-keyed backward map).

**Answer, per track, under 50 lines each:**
1. **The route Julien's time implies.** From the blocks, describe the course (gaps and their widths, heights, fences and their 1.5-block collision, ceilings, the maze's walls and openings) and the route that fits his time. Name each **hard span** on it: where the body needs something other than a plain sprint-jump (a jump at the gap limit, a fence to go around, a turn in a maze, a step-up, a neo or corner).
2. **Why the search fails there.** Using the failure line's nearest state and the code: is it (a) a move the 54 programs and 64 aims cannot produce at all (name it, with the law's numbers from the code); (b) a move they can produce but the order never expands within budget (why: the distance map's estimate, the greedy order, ties); (c) a cell the planner wrongly treats as blocked or different (as with the buttons); or (d) something else? Cite lines.
3. **What a span solver needs** to cross each hard span: the words (program, aim, timing) and what bounds the search there. Say whether the forward search scoped to the span would find it in the red-team's terms, or what it lacks.
4. What you could not read or verify, by name.
