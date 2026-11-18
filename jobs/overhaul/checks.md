# The overhaul's checks: every issue raised on 18 November, its fix, and how to tell it worked

Fill the last column at each review. "Before" is the measured state on 18 November.

| # | Issue (his words) | Fix | How to check it worked | Before (18 Nov) | Review |
|---|---|---|---|---|---|
| 1 | Docs not read, then said to be read (S231: "why did you had to reread the docs") | `map/` loaded whole by the `SessionStart` hook; ends with "END OF LIVING DOCS" | after a new session or compaction, ask "what does the map say about X" without any Read; the answer cites the map correctly | 5 of 11 docs read; "docs are read" claimed | |
| 2 | Docs too long and too detailed (S236) | `map/` capped at 40 KB by the pre-commit hook; detail one link away | `cat map/*.md \| wc -c` | 308 KB loaded → 22 KB | |
| 3 | Stale lines and history in the docs (S236, S239: AGENTS.md named Fable) | present tense only, checked on commit; AGENTS.md rewritten | `grep -E "[0-9]+ November\|superseded\|previously" map/*.md map/detail/*.md` returns nothing; no false fact found in a spot check | AGENTS.md named Fable; 64 dates in PRINCIPLES | |
| 4 | "Update the doc" done as appending | the map is rewritten, not patched; `engine/` commits must touch `map/` or say why | count commits touching `engine/` without `map/` and without a `Map: unchanged` line: must be 0 | no check existed | |
| 5 | Rules forgotten in a long context (S237) | the 10-line card on every message (`UserPromptSubmit` hook) | the card appears with each of his messages; count violations of its 10 lines he points out per week | — | |
| 6 | Job ends missed; he had to wake me (S239) | a watcher on every launch wakes the integrator (`PostToolUse` hook) | every job end is reported without him asking | missed repeatedly; worked live for Hydrogen at 16:20 | |
| 7 | Prompts for big runs badly framed (S230: "show me in chat the prompt") | Astra, Sol, Opus and Argon builds refused without `jobs/<id>/shown.txt` | no big-model launch without a prompt he saw | Astra's warm-floor spec cost a Codex window in 45 min | |
| 8 | Specs and judges checked against the last spec, not the docs (post-mortems 16 and 18 Nov) | goal, principles and architecture lead every prompt; judges get "does this obey the map?" | judge reports carry a map-conformance verdict; count FAILs on map grounds | judges passed the per-yaw store | |
| 9 | The new core never flew in his game (S239: "always test in game that's the only kind of test that matters") | `plan-track --mode solve` exports the plan the mod plays (Astra, part 0); every step's gate is a flight | flights of `straight100/300/800`, `stonesimple`, ten flat goals: clock T = predicted T, no deviation | never flown | |
| 10 | Tests as big as the code; tests the game does not judge (S235) | only game-judged tests kept | test lines ÷ code lines; any new test must be judged by the game | 8,796 test lines ≈ code → 1,861 | |
| 11 | Flat ground relearns its gait per command (S230, S232) | `jobs/flat-ground`: the gait stored once, the yaw put in | `scripts/scoreboard.ps1`: per new command against the 10 µs floor; law ticks prepared per command = 0; refusals while moving = 0 | 61 ms standing, 57 ms moving (3.8 orders), 22 of 60 refused | |
| 12 | Wrong framing and made-up numbers (S223 "warm", S225) | every progress number comes from `scripts/scoreboard.ps1`, beside its floor | no number reported that the scoreboard did not print | the 57 µs "warm" figure reported for days | |
| 13 | Busy work: wiki, lenses, digests (S234, S235) | removed; every meter works on the next useful work, the bottleneck first (S241) | no companion idle while it has useful work; no wiki or lens library back | 169 wiki pages, 165 lenses, a 19-lens queue | |
| 14 | Prevention instead of fixing (S235: "cheaper to fix than to prevent") | GLM checker removed; no frozen internal gates | GLM cost per result halves; deltas found in flights get fixed | author + checker per GLM run | |
| 15 | Companions too restricted and over-monitored (S237) | contracts: read anything, run anything in the worktree, decide and keep going; only the irreversible closed | fewer BLOCKED stops on open questions; less integrator time per job | Terra stopped at step one; reading was restricted | |
| 16 | Code never shrinks; the cleanup trigger was lost (S239) | net lines print on every commit; growth in `engine/` or `bot/` needs `Bought:` | net lines per week; the old A\* planner and dead `words` code deleted | no trigger; A\* planner still in `plan` | |
| 17 | Rarely merged or pushed (S239: "push more and more often") | judge pass → merge → push the same hour | days between a passing judge and `main` | the new core never on `main` → merged 18 Nov | |
| 18 | Dead things kept (S239) | archived: worktrees, branches, the wiki, lenses | `git worktree list \| wc -l`; branches under `wt/` | 62 worktrees, 58 branches → 10 and 10 | |
| 19 | His private email pushed to the public repo (18 Nov) | history rewritten; `pre-push` refuses the email and Antigravity logs | the hook refuses a test push containing them | leaked in 6 logs, then erased | |
| 20 | Usage blindness (S168, S238) | `/meters` opens the companion meters; the native plan shows in the app | he can read every meter without asking | asked repeatedly | |
| 21 | My names and jargon, not his (S179, S222) | `map/vocabulary.md`; card line 9 | count his "made up names" or "explain" corrections | frequent | |
| 22 | Acting when he asked to explain (S179) | card line 9 | count of turns where he had to stop an action | repeated | |
| 23 | Duplicate runs offered as options (S221) | memory "say duplicate plainly" | no duplicate run launched | one on 17 Nov | |
| 24 | A stopping habit: reporting instead of finishing approved work (S240: "why did you not do it?") | — (no mechanism yet) | approved items left undone at a turn's end, unblocked | 5 items left undone on 18 Nov | |
| 25 | Wrong mind for the job (S174, S175) | routing in `map/work.md`; a smaller model never writes for a bigger one | count of mis-routed jobs he flags | several | |
| 26 | Mods built before checking what the app allows (18 Nov, the meters "under Context") | check the app's slots before building | — | built twice in the wrong place | |
