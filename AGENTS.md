# Agents

You work on Julien's Minecraft pathfinding project. **The truth is `map/`**: the goal, the principles, the architecture, his vocabulary, how we work, and what is in flight. It is loaded whole at every session start and compaction; detail is one link away in `map/detail/`; history is in `archive/` and git.

- **The map leads, the code catches up.** When code teaches something new (a measured number, a new mechanism), the map changes in the same commit. Never write history into `map/`: rewrite it in the present tense.
- **The game is the judge.** Only tests the game judges are written; every step ends in a flight he can use.
- **Fermi first; question the requirement, delete, then simplify; one mechanism; store the generator; precision only where a decision can flip** (`map/principles.md`).
- **His words go verbatim** to `archive/turns/`; speak in his vocabulary (`map/vocabulary.md`). When he asks to explain, explain and do not act.
- **Closed, because it cannot be undone:** pushing (the integrator only); his world and server while he plays; secrets, passwords, tokens, game jars and his private email never enter git; git history; other people's branches; launching paid runs without the integrator.
- **Everything else is yours to decide.** Read the whole repository, run anything in your worktree, and write a decision down instead of stopping on it.
- **Enforced by code** (the if-then graph, `map/work.md`): the map's load, the push guard, the meter alarm; more being built (`jobs/overhaul/plan.md`).
- **Text has a cost:** `map/` stays under 40 KB; pointers, not copies.
