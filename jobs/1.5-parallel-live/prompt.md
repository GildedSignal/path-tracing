# Job 1.5: live runs that rarely block, and several clients at once

**Why.** L0 became the lab's bottleneck on 9 November. It needs the real client (the judge), one client under one rig lock, and it's sensitive to CPU load, so GPT, GLM and builds pause while it records. Its loop also went through the live step: a missing recorded field meant re-recording. Julien (S61): "we'll have many clients either way", so learning what a client costs, and how many fit, is worth it in itself. The 100× testing goal and Phase 3's audits will run many.

**Starts after job 1.3 (L0) merges.** It touches the same recorder and rig scripts.

## Build

1. **Over-capture the recording.** Each live tick records everything the client could feed the law, not only today's comparison fields:
   - every shared flag and pose;
   - each attribute's base value and its modifiers (id, amount, operation);
   - effects, food, saturation and exhaustion;
   - the server packets applied that tick (entity data, motion, teleports), with their order;
   - the input as the client read it.
   
   The comparison fields and L0's pass line stay frozen. The goal is that a new divergence can be diagnosed and fixed offline against a saved recording, without re-recording. Show it by re-deriving the 9 November sprint-flag diagnosis offline from a new recording.
2. **Pin cores.** Live runs pin the server and each client to dedicated cores (processor affinity) at normal priority. `scripts/gpt.ps1`, `scripts/glm-local.ps1` and builds run below normal priority on the other cores. Measure it: a live L0 recording with a GLM batch and a release build running at the same time, compared with the quiet machine. Zero corrections, and identical recordings, are the pass. If that fails, report how many cores the live side needs.
3. **Several clients.** A rig can start k disposable copies (own folder, port and lock each), with each client recording a different terrain class. Find the largest k that keeps zero corrections on this machine (8 cores, 16 threads, 31 GB), and report per-client RAM, cores and the wall time per corpus against k = 1.

## Must not

- Weaken L0 or any frozen gate.
- Open Julien's world (`server\`).
- Write game files with PowerShell 5.1's `-Encoding UTF8`: its BOM resets `options.txt` (HANDOFF, Traps).

## Done means

The final message gives: the recording's new fields and its bytes per tick; the offline re-diagnosis; corrections with and without load, pinned; and k, RAM, cores and wall time per k.
