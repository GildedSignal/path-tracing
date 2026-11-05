# Job L1: the GLM lab's harness on GitHub Actions

Read `lab/glm.md` first: it is the charter, and this job builds its machinery. GLM (Z.ai's glm-5.3, run through the `omp` command-line agent) runs on GitHub's machines (Julien, S21), never on his PC. The repository `GildedSignal/path-tracing` is public. ES built a similar lab; its files are in `jobs/L1-glm-lab/es-reference/` (`/` became `_` in the names). Reuse what fits and cut what this charter doesn't ask for: no wiki, no publishing to branches, no rung verdict files, no schedule.

Where something is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks. Do not commit; the integrator commits and pushes. You have no network: you cannot run the workflow. Test what you can locally (bash syntax with `bash -n`, the YAML's structure, the scripts against a fake `omp` that you write as a stub).

## What to build

1. **`lab/setup.sh`:** installs omp v18.16.0 for Linux x64 from `https://github.com/can1357/oh-my-pi/releases/download/v18.16.0/omp-linux-x64`, verified by SHA-256 `6498ed785bab00d2926e0387e0ec02362d6d45f1c3d0d1b57c1618b7f95c9264`, into `~/.local/bin/omp`. Refuse on a mismatch.
2. **`lab/run.sh <lens> <run-id> [author-minutes]`**, one run, two phases:
   - **Author:** one omp session (`omp -p --model=zai/glm-5.3 --thinking=high --approval-mode=yolo --no-session --max-time=<m>m @<prompt file>`). Its prompt is, in order: `docs/PRINCIPLES.md` from `## The goal` up to (not including) `## From Emergent Space`; `lab/glm.md`; `lab/lenses/<lens>.md`; and the run's paths. The author writes `lab/results/<run-id>/dossier.md` and `lab/results/<run-id>/reproduce.sh`, and may write anything else only inside that folder. Afterwards, any change outside `lab/results/<run-id>/` is saved as `work.patch` in the folder, then reverted.
   - **Checker:** a fresh omp session that never sees the author's session. Its prompt: the same PRINCIPLES excerpt, `lab/glm.md`, `lab/checker.md`, and the dossier. It runs `reproduce.sh` itself (applying `work.patch` first, if the dossier says it is needed), tries to break the claim, and writes `check.md`, whose first line is exactly one of `Grade: reproduced`, `Grade: lead`, or `Grade: nothing-found`, followed by the reasons.
   - It writes `lab/results/<run-id>/DONE`, one line: the run id, the lens, the grade, and the dossier's one-sentence claim.
   - Every limit is in minutes per phase, so a run can never hang the job.
3. **`lab/checker.md`:** the checker's instructions, written from `lab/glm.md` ("A GLM result counts only if a machine can check it") and ES's checker rules in `lab_README.md`. Keep it under 40 lines.
4. **`.github/workflows/glm-lab.yml`:**
   - Triggered only by `workflow_dispatch`, with inputs: `lenses` (space-separated, up to 3) and `minutes` (default 60).
   - **Never** `pull_request`, `pull_request_target`, `issue_comment` or `schedule`. This is a public repository: no event a stranger can cause may reach the secret.
   - One job on `ubuntu-latest`, `permissions: contents: read`, `actions/checkout` with `persist-credentials: false`. `ZAI_API_KEY` comes from secrets, only into the run steps' environment. Fail with a clear message if it is missing.
   - It runs the lenses in parallel, each in its own git worktree, with a separate `CARGO_TARGET_DIR`. It waits for all of them within the job's timeout.
   - Before uploading, it deletes every file that contains the key's value, and any file over 5 MB.
   - It uploads `lab/results/` as one artifact (14 days), and prints each `DONE` line to the job summary (`$GITHUB_STEP_SUMMARY`).
   - Logs the quota: `omp usage` before the runs, as ES did.
5. **`lab/lenses/README.md`:** two lines: one file per lens, written by the integrator, never by GLM. Don't write lenses yourself.
6. **`lab/results/README.md`:** results arrive as workflow artifacts, downloaded by the integrator with `gh run download`. Add `lab/results/*` to `.gitignore`, except that README.

## Done means

- `bash -n` passes on every script. The workflow is valid YAML, with only the triggers above, and the secret is used only where stated.
- A stub `omp` exercises `run.sh` end to end locally: both phases, the patch capture and revert, `DONE` and the key scrub.
- The final message lists every change and every assumption, and what remains untested until the first real run.
