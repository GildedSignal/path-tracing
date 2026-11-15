# 1.3e: freeze damage and the step-based inside collector, so L0 keeps powder-snow run 7

**Consumer:** L0's live Full gate. On 14 November it kept 94 runs and refused one: powder-snow run 7, seed 95, 420 ticks. The replay met a server event it does not model: damage of type `minecraft:freeze`, amount 1.0, no source entity. Your branch `wt/l0-veto` (worktree `D:\path-tracing-wt\l0-veto`, commit `9a76a7c`, job 1.3d) already made the freeze counter increment once per sweep step. This job adds what the jar does next.

**The evidence:** GLM's checked dossiers, each with `dossier.md`, `check.md` and a `reproduce.sh`, all under `C:\Users\PASCAL\Desktop\path tracing\lab\results\`:
- `local-20261114-1633-1-0-port-freeze-damage`: when freeze damage fires, and every movement field it writes;
- `local-20261114-1628-1-0-port-freeze-step-fluid` and `local-20261115-1143-1-0-port-freeze-step-fluid`: the step-based collector's flushing;
- `local-20261114-1107-1-0-l0-freeze-tick-111`: the tick-111 second writer;
- `local-20261112-1251-1-2-port-block-laws` and `local-20261113-2350-1-0-port-block-laws`: cobweb WEAVING, and the inside blocks the law lacks.

Where an author and its checker disagree, the checker wins. The conflicts are listed in `C:\Users\PASCAL\Desktop\path tracing\jobs\port-backlog\report.md`, section 4, item 4.

## Decisions already made

1. **Port, from those dossiers, into `engine/law/src/movement.rs`** (and `world.rs` only where a dossier names it):
   - the step-based collector's step-run set-dedup flush, and the default `Shapes.block()` step advance on non-air cells, in `inside_segment`;
   - the per-flush freeze +1, the server's −2 decay, and the frost speed modifier `-0.05*(min(tf,140)/140)` (server-driven per engine.md: recorded and read as input where the server owns it);
   - the freeze damage period, threshold and amount, and the `(0.9, 1.5, 0.9)` stuck multiplier.

   Each line cites its jar class and offset in a comment, as the existing law does.
2. **Server-owned outcomes** (the damage event, health, hurt cooldowns) go through the law's existing `ServerResourceEvent` path, as the recorder's damage rows require. Do not simulate what engine.md classes as server-driven.
3. **Cobweb WEAVING:** refuse it explicitly if a state can carry it (Unknown, never a silent wrong answer). Model it only if a dossier gives every constant.
4. **Nothing else from the port backlog.** Elytra, boats, pushing and mining are other batches.

## Gate (offline; nothing live)

Someone else uses this machine, so run no client and no server. Use `D:\path-tracing-evidence\l0-veto-20261114*` for the recorded runs.
- The recorded powder-snow run 7 (seed 95) replays through the law with the freeze damage row matched bit for bit, through all 420 ticks.
- Every run L0 kept on 14 November still replays.
- `cargo test --offline` from `engine/` passes.
- `jobs\1.2-footprints\verify.ps1` catches all mutants.
- A mutant of the damage period or the decay goes red.

**Done means** the gate passes, and `jobs\1.3e-freeze-damage\last.md` lists every file changed, the jar offsets used, and the replay outputs. The integrator then runs L0 live on a free machine.

No new crates, no push, no tolerance or epsilon, no per-run special case.
