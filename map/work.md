# How we work

## Who does what (routing detail: detail/clan.md)

| Who | For |
|---|---|
| the integrator + Julien | core planner, representation, learning architecture, merges, the map |
| Opus (Claude headless / sub-agent) | structural ports and hard refactors (`wt/port-state-spine`), red-teams of one-way doors |
| Astra (`gpt-6.2-astra`, Codex) | hard spec-driven builds when Codex window is open |
| Sol (`gpt-6.2-sol`, Codex) | strong builds beside the core |
| Terra (`gpt-6.1-terra`, Codex) | builds from a closed spec, the leftover window |
| Sonnet (native sub-agent) | judging diffs, cleaning, digests, linting the map against his words |
| GLM 5.4 (Z.ai, omp) | parameter table rows, Mojang scars, and jar witnesses against the generator basis, plus compiled `lab/wiki/` pages |
| Vibe (Mistral, omp) | the same lane as GLM, a second family whose rows check GLM's; enters through the bake-off (detail/clan.md) |

Antigravity is his own tool; he pilots it. A smaller model never writes a prompt for a bigger one (S210). Launch: `scripts/launch.ps1 <job> terra|astra|sol [-Worktree D:\path-tracing-wt\<name>] [-Base <ref>]`; Opus and Sonnet are the integrator's native sub-agents; GLM runs through `scripts/glm-chain.ps1`. A job is `jobs/<id>/prompt.md` in, `last.md` out. Meters: `scripts/meters.ps1`.

## A companion's job

A clear goal, in one line under the prompt's title, the conditions, the gate (a flight in his game whenever the work reaches the game), and this map. How it gets there is its own (S237): it reads the whole repository, runs anything in its worktree, and writes a decision down instead of stopping on it.

**Closed, because it cannot be undone:** pushing; his world and server while he plays; secrets; git history; other branches; launching paid runs.

## The if-then graph (enforced by code)

| If | Then | Code |
|---|---|---|
| a session starts, resumes, clears or compacts | this map loads whole | `SessionStart` hook, `scripts/load-docs.sh` |
| he sends a message | the heuristics card is injected; his words go verbatim to `archive/turns/inbox.md` | `UserPromptSubmit` hook, `scripts/hooks/prompt.ps1` |
| any launch | the goal, principles and architecture lead the prompt; a judge gets the map question | `scripts/gpt.ps1`, `lab/run.sh` |
| any launch | refused unless the prompt opens with its title, then `**Goal:**` in one line | `PreToolUse` hook, `scripts/hooks/launch.ps1` |
| a native Opus sub-agent (or an Agent call with no model, which inherits Opus) | refused unless its prompt names `jobs/<id>/prompt.md` and `jobs/<id>/shown.txt` exists | `PreToolUse` hook on Agent, `scripts/hooks/opus.ps1` |
| a launch of Astra or Sol | refused unless `jobs/<id>/shown.txt` exists (he saw the prompt) | `PreToolUse` hook, `scripts/hooks/launch.ps1` |
| a launch succeeds | a watcher wakes the integrator when the job ends | `PostToolUse` hook (async, wakes the model) |
| a commit touches `map/` | 40 KB cap, no history wording, links resolve | `.git/hooks/pre-commit` (`scripts/install-hooks.sh`) |
| a commit touches `engine/` | it touches `map/` or carries `Map: unchanged, because …` | `.git/hooks/commit-msg` |
| a push | refused if it carries his private email or an Antigravity log | `.git/hooks/pre-push` |
| a meter passes 80% | it is reported | `scripts/meters.ps1 -WatchAt 80` |
| a number is reported as progress | it comes from the scoreboard, beside its floor | `scripts/scoreboard.ps1` |
| a commit changes code | its net lines print; growth in `engine/` or `bot/` needs a `Bought:` line | `.git/hooks/commit-msg` |

An edge that never fires or never catches anything is deleted. A new failure changes an edge, never adds prose.

## Rules no code enforces yet

- His turns go verbatim to `archive/turns/`; a turn that changes a belief changes this map the same day.
- When he asks to explain, explain and do not act (S179).
- After a judge passes: merge to `main`, print net lines, push the same hour.
- Disk: toolchain, builds and worktrees on D: (`PATH=D:\rust\cargo\bin`, `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`). PowerShell 5.1 garbles UTF-8: edit text with the Edit tool. Codex Windows `workspace-write` mounts `.git/` read-only even inside standalone D: clones, so builders leave verified edits in their clone and the integrator commits and runs `bench-judge.sh`.
- Component state is derived, never declared: `Sketch` (branch/jar only) -> `Flying` (wired and flown on `main`) -> `Load-bearing` -> `Frozen`. Feature branches open with a `clean:` commit removing dead code in their area.
