# Antigravity runs commands: the launcher fixed, measured (17 November, by the integrator)

Terra's run (Codex, 20:17) stopped `BLOCKED`: the Codex sandbox denies Antigravity's sign-in and network, so no live probe could start. Julien: "Yes, run them" (the integrator ran the probes outside that sandbox).

## Root cause, confirmed and corrected

The builds' "cannot run commands" was our launcher (S210), in two parts:
1. **Path forms.** Under agy's terminal sandbox, read rules become read-only mounts and write rules writable ones. Any form without a drive letter (`/C:/...`, `file:///c:/...`) is rejected: "sandbox configuration error: readonly /C:/Users/PASCAL/Desktop/path tracing/: non-absolute file path", and no command runs (probes H1, H3).
2. **Rule matching.** Without the sandbox a command runs "unsandboxed" and needs an allow rule matching its whole command line: `command(cargo)` does not allow `cargo --version`, `command(cargo run)` does not allow `cargo run --quiet`, `command(cargo *)` and `command(cargo:*)` match nothing (E1–E5, F1–F5). Only an exact line passes (F1). An allow-list cannot serve a build.

Also measured: only a write rule **without** a trailing slash makes a folder writable for commands (O2; `D:/x/` stays read-only, O1; `/**` forms break the sandbox, O3). Cargo creates a missing target folder through a temporary sibling in its parent, so the target folder must exist before the run.

## Probe table (`jobs/_probe-cmd/<name>`, `scripts/probe-agy-commands.ps1`, Hydrogen low)

| Probe | Sandbox | Path forms | Rule | Command | Result |
|---|---|---|---|---|---|
| A | no | all (today's) | `command(cargo)`, `command(git)` | `cargo --version` | denied (unsandboxed, no exact rule) |
| C | no | drive | `cargo`, `cargo --version`, `git` | `cargo --version`; `git --version` | ran; denied |
| E1–E5 | no | drive / no-driveless / all | `command(cargo)` or `command(git)` | `cargo --version`, `cargo run --quiet`, `git --version` | all denied |
| F1 | no | drive | `command(cargo --version)` | `cargo --version` | ran: cargo 1.99.1 |
| F2–F5 | no | drive / all | `cargo run`, `cargo *`, `cargo:*` | `cargo run --quiet`, `cargo --version` | denied |
| H1 | yes | all | `command(*)` | `cargo run --quiet` | sandbox error: `/C:/...` non-absolute |
| H3 | yes | no driveless, `file:///` kept | `command(*)` | `cargo run --quiet` | sandbox error: `file:///c:/...` non-absolute |
| H2, I1, K1 | yes | drive | `command(*)` | `cargo run --quiet` | ran, in the job folder, not the worktree |
| H4 | yes | drive | `command(*)` | write to `D:\path-tracing-evidence` | refused |
| J1, J2, L1, M1, N1 | yes | drive | `command(*)`, `write_file(D:/x/)` | build in the worktree | refused: the worktree is read-only |
| O1 | yes | drive | `write_file(D:\x\)` | write in the worktree | refused |
| O2 | yes | drive | `write_file(D:/x)` (no slash) | write in the worktree | **written** |
| O3 | yes | drive | `write_file(D:/x/**)` | write | sandbox error |
| `test-antigravity.ps1`, first run | yes | drive, through `gemini.ps1 -Commands` | — | `cargo run` with `--target-dir` | refused at the target's temporary sibling |
| `test-antigravity.ps1`, second run | yes | same, target pre-created | — | `cargo run`; a write outside | **`probe ok <nonce>: sum = 55`**; the outside write refused; reads by all six forms still pass |

Cost: about 30 small runs, Gemini 5-hour 21% → 24%, week 43% → 45% (meters.ps1).

## Files changed

- `scripts/gemini.ps1`: `-Commands` (needs `-Write <worktree>`): `--sandbox`, `command(*)`, drive-letter read forms only, writable mounts (no trailing slash) for the worktree, the job folder, `D:\path-tracing-target\agy-<job>` (created before the run) and `D:\rust`; the prompt's contract tells the model where it may write and to `cd` into the worktree. Read-only jobs are unchanged.
- `scripts/launch.ps1`: `-Commands -Worktree D:\path-tracing-wt\<id>` for Argon and Hydrogen, making the worktree like a GPT one.
- `scripts/test-antigravity.ps1`: a live commands probe (build and run a ten-line Rust program with a nonce; a write outside must be refused); stamped pass on 17 November.
- `scripts/probe-agy-commands.ps1`: the single-variable probe used above.
- `docs/clan.md`: section 5c rule 6 and the Antigravity traps.

## How a job is launched with commands

`.\scripts\launch.ps1 <id> hydrogen -Commands -Worktree D:\path-tracing-wt\<id>` (or `argon`). The job builds with `--target-dir D:\path-tracing-target\agy-<id>` and writes only in its worktree and its job folder. The integrator commits its work with `Built-by:`.

STATUS: DONE
