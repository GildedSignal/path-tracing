# How we work

## Who does what (routing detail: detail/clan.md)

| Who | For |
|---|---|
| the integrator (main Claude session) | specs and prompts, merges, the map, talking with him |
| Astra (`gpt-6.2-astra`, Codex) | the core and the hard builds; the Codex window is hers |
| Sol (`gpt-6.2-sol`, Codex) | strong builds beside the core |
| Terra (`gpt-6.1-terra`, Codex) | builds from a closed spec, the leftover window |
| Opus (native sub-agent) | red-teams of one-way doors, design, hard merges |
| Sonnet (Antigravity, or native) | judges GPT's and Google's work |
| Argon (Antigravity) | tooling builds, reading primary sources, linting the map against his words |
| Hydrogen (Antigravity) | builds, digests, premise checks |
| GLM (Z.ai, omp) | port code against the jar, from a prompt the integrator writes |

A smaller model never writes a prompt for a bigger one (S210). Launch: `scripts/launch.ps1 <job> <who> [-Worktree D:\path-tracing-wt\<name>] [-Base <ref>]`; a job is `jobs/<id>/prompt.md` in, `last.md` out. Meters: `scripts/meters.ps1`.

## A companion's job

A clear goal, the conditions, the gate (a flight in his game whenever the work reaches the game), and this map. How it gets there is its own (S237): it reads the whole repository, runs anything in its worktree, and writes a decision down instead of stopping on it.

**Closed, because it cannot be undone:** pushing; his world and server while he plays; secrets; git history; other branches; launching paid runs.

## The if-then graph (enforced by code)

| If | Then | Code |
|---|---|---|
| a session starts, resumes, clears or compacts | this map loads whole | `SessionStart` hook, `scripts/load-docs.sh` |
| a push | refused if it carries his private email or an Antigravity log | `.git/hooks/pre-push` |
| a meter passes 80% | it is reported | `scripts/meters.ps1 -WatchAt 80` |

Being built (the overhaul, `jobs/overhaul/plan.md`): the heuristics card on every message; the map appended to every launch and the judges' map question; the big-model launch guard; the job watcher that wakes the integrator; the scoreboard script; the commit checks on `map/` and `engine/`; net lines on every merge.

An edge that never fires or never catches anything is deleted. A new failure changes an edge, never adds prose.

## Rules no code enforces yet

- His turns go verbatim to `archive/turns/`; a turn that changes a belief changes this map the same day.
- When he asks to explain, explain and do not act (S179).
- After a judge passes: merge to `main`, print net lines, push the same hour.
- Disk: toolchain, builds and worktrees on D: (`PATH=D:\rust\cargo\bin`, `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`). PowerShell 5.1 garbles UTF-8: edit text with the Edit tool.
