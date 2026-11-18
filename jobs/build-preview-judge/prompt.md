# Judge: the preview renderer

**Goal:** a verdict on branch `build-preview`: merge it, merge part of it, or archive it, judged by the map and by what the game can judge.

You judge in a fresh context; you did not build this. Look for what is wrong, not for what is right.

## What it is

- Branch `build-preview`, worktree `D:/path-tracing-wt/build-preview`. Its spec and its builder's report: `jobs/build-preview/prompt.md`, `jobs/build-preview/last.md`.
- What it was for: rendered views of a build from its command list, textures read from the local client jar and never committed, for building in passes (S205).
- `main` has moved on since it was built: 143 commits behind, and it conflicts with main. The map (above, and `map/`) is what is believed now; the spec may be older than it.
- The diff to judge: only commit `0cd3229` (the renderer; the branch's other 74 commits are old integrator commits from before the history rewrite), `git diff 0cd3229^ 0cd3229`, saved for you in `jobs/build-preview-judge/diff.patch` (data files such as recordings and region files left out; their names are in `jobs/build-preview-judge/diff-stat.txt`).

## Questions, in order

1. Does it obey the map: one mechanism, the generator stored once, precision only where a decision can flip, nothing kept that the game does not judge?
2. Is it still needed? Name what on `main` now does the same job, or what step of `map/now.md` consumes it. No consumer is a reason to archive.
3. Is it right? Read the code against its spec and the law (`engine/law`, Mojang's code where the spec quotes it). Each finding: file:line, what is wrong, the concrete case, the fix.
4. What does a flight in his game have to show before it merges (track, expected T or behaviour)?

## Output

Your final message: the verdict on the first line, `MERGE`, `MERGE PART: <which files>`, or `ARCHIVE`, with one sentence why; then the four answers; findings ranked by severity.
