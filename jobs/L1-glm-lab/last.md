Implemented L1. Changes are uncommitted.

- [lab/setup.sh](D:/path-tracing-wt/glm-lab/lab/setup.sh): pinned installer; checksum mismatch preserves any existing executable.
- [lab/run.sh](D:/path-tracing-wt/glm-lab/lab/run.sh) and [lab/checker.md](D:/path-tracing-wt/glm-lab/lab/checker.md): author/checker phases, bounded sessions, patch capture and restoration, grades, and `DONE`.
- [glm-lab.yml](D:/path-tracing-wt/glm-lab/.github/workflows/glm-lab.yml), [lab/actions.sh](D:/path-tracing-wt/glm-lab/lab/actions.sh), and [lab/scrub.sh](D:/path-tracing-wt/glm-lab/lab/scrub.sh): dispatch-only execution, parallel worktrees and Cargo targets, quota logging, secret/file scrubbing, summaries, and one 14-day artifact.
- [lenses/README.md](D:/path-tracing-wt/glm-lab/lab/lenses/README.md), [results/README.md](D:/path-tracing-wt/glm-lab/lab/results/README.md), and [.gitignore](D:/path-tracing-wt/glm-lab/.gitignore): ownership and artifact instructions; results ignored except README. No lenses added.
- [test-harness.sh](D:/path-tracing-wt/glm-lab/lab/tests/test-harness.sh) and [check-workflow.cjs](D:/path-tracing-wt/glm-lab/lab/tests/check-workflow.cjs): repeatable offline checks.

Passed: `bash -n`, YAML parsing and structural controls, stub end-to-end runs, binary/ignored-file patch restoration, timeouts, mixed parallel failures, grading, key scrubbing, size boundaries, and mocked installer checks.

Assumptions:

- Checker budget is half the author budget, rounded up.
- Author defaults to 60 minutes, limited to 1–180; job timeout is 300 minutes.
- Runs require fresh, clean, detached worktrees; dossiers begin with `Claim:`.
- Five MB means 5 MiB.
- Quota lookup failure is logged but does not prevent runs.

Untested until the first real run: the release download and its supplied hash, actual GLM authentication/CLI behavior, GitHub secret wiring, and artifact upload/download.