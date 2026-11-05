Implemented. `.\bot\build.ps1` passes without warnings. Offline raw-bit serialization, input-validation, and server-state parser checks pass. Live gates remain unrun, as required.

Changed files:

- `bot/build.ps1`
- `bot/fabric.mod.json`
- `bot/pathtracing.mixins.json`
- `bot/recorder.md`
- `bot/src/pathtracing/Bot.java`
- `bot/src/pathtracing/Recorder.java`
- `bot/src/pathtracing/TickInput.java`
- `bot/src/pathtracing/TraceJson.java`
- `bot/src/pathtracing/BlockDumper.java`
- `bot/src/pathtracing/mixin/ClientPacketListenerMixin.java`
- `bot/src/pathtracing/mixin/BlockBehaviourAccess.java`
- `scripts/server-state.ps1`

Owner commands:

```text
/msg PathBot replay <name>
/msg PathBot rec start <name>
/msg PathBot rec stop
/msg PathBot dump blocks
```

`/msg PathBot stop` also cancels replay and clears the follow path.

File formats:

- `pathtracing/inputs/<name>.jsonl`: start-array header, then consecutive tick-input objects.
- `pathtracing/recordings/<name>.jsonl`: one tick-result object containing inputs, state, decimal/raw-bit pairs, and optional packet-anomaly arrays.
- `pathtracing/dumps/blocks.jsonl`: one object per block state, including properties, tags, grouped collision contexts/AABBs, and offset generators with eight samples.
- `.\scripts\server-state.ps1 PathBot`: prints one JSON object containing all seven requested server fields.

Full schemas and behavior are documented in [bot/recorder.md](D:/path-tracing-wt/recorder/bot/recorder.md).

Could not commit: Git denied access to `C:\Users\PASCAL\Desktop\path tracing\.git\worktrees\recorder\index.lock`. All changes remain in this worktree.