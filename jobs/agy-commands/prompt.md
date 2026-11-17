# Let Antigravity's models run commands: fix the launcher, prove it with the probe

**Julien (S210):** Hydrogen "can't run code" was our mistake, not the model's. The integrator found the cause: our launcher grants no command, and the sandbox refuses all commands because of a path form we add. With commands, Hydrogen and Argon can build and test on their own Google meters, which keeps the Codex window for Astra (S209: "the goal is to maximize astra more").

**Read:**
- `scripts\gemini.ps1`, all of it, above all the permission block from "In headless mode there is no approval prompt" to the `--add-dir` arguments;
- `scripts\launch.ps1` (how Antigravity jobs are started and slotted) and `scripts\test-antigravity.ps1` (the probe `launch.ps1` requires before any Antigravity job);
- `docs\clan.md` section 5c, rules 2 and 6, and the Antigravity traps paragraph;
- **the evidence:** `jobs\build-river-city\agy.log`, `report.md` and `settings.json`. Commands failed with "sandbox configuration error (`readonly /path-tracing-evidence/three-builds/: non-absolute file path`)".

## What is known, and what to establish
1. **Known:** `gemini.ps1` allows only `read_file` and `write_file`, plus `command(git)` and `command(cargo)` when `-Write` is given. For every directory it also grants driveless forms (`/path-tracing-evidence/...`) and `file:///` forms, because the agent's reads were denied without them (S129). The sandbox apparently builds read-only mounts from the read rules and rejects the driveless ones as non-absolute, which disables command execution altogether.
2. **Establish by experiment,** with the cheapest probes (one short Hydrogen run each, a trivial task):
   - which permission forms the sandbox accepts;
   - whether reads still succeed without the driveless forms in each form an agent uses (the S129 matrix: drive letter, lower case, leading slash, `file:///`, `%20`);
   - how `command(...)` rules are matched: name only, or with arguments;
   - whether a command runs in the job's worktree.

   Record each probe's input, output and cost.

## Build
1. **A `-Commands` mode** for `gemini.ps1` and `launch.ps1`. Default off; the read-only jobs stay exactly as they are.
   - When on, it allows a named list of commands: `cargo`, `rustc`, `java` and `javac` from the rig's JDK, `git` (read-only subcommands: `status`, `diff`, `log`, `show`), and `powershell -File` on scripts inside the job's own worktree.
   - It writes only into the job's folder and its worktree, a GPT-style worktree on D: created the same way `launch.ps1` creates Terra's.
   - The read rules keep working.
2. **The probe extended** (`scripts\test-antigravity.ps1`): a Hydrogen run in `-Commands` mode compiles and runs a ten-line Rust program in a scratch worktree, and writes its output into its report. `launch.ps1` keeps refusing Antigravity jobs until the probe passes for the current scripts.
3. **Docs:** fix rule 6 of `docs\clan.md` section 5c and the traps paragraph to say what is true now.

## Must not touch
`engine\`, `bot\`, `server\`, `clients\`, his tracks and runs. Never run `gpt.ps1` or `follow-up.ps1`. You may run `launch.ps1 ... -DryRun`, and `scripts\test-antigravity.ps1`, which spends a little of the Gemini meter. Read `scripts\meters.ps1` before and after.

**`jobs\agy-commands\last.md`:**
- the root cause, confirmed or corrected;
- the probe table;
- the files changed;
- the meter cost;
- how a job is launched with commands.

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
