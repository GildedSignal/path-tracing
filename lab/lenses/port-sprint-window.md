# Port delta: double-tap sprint (the Sprint Window option)

Use the staged Mojang 26.2 client jar in the staged dependency root D:/path-tracing-deps/data and staged javap at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe. Sprinting is decided client-side, in LocalPlayer.

**Consumer:** Julien's recorded races (S149–S152). His client has `sprintWindow` 7, so the law's replay refuses his recordings at `engine/law/tests/replay.rs:366` ("unsupported double-tap sprint window"). That covers his 428-tick hard-track shortcut, `jobs/hard-track/julien-428.jsonl`, and 32 of his 44 track runs. The recordings hold per-tick raw keys (`forward`, `sprint`, ...), yaw, pitch, position bits and `isSprinting`. A port of the double-tap rule lets the law replay them.

**From the 26.2 client bytecode**, state with class, method and offsets:
- the `sprintWindow` option: its type, range and default, and how its value becomes a tick count;
- the timer LocalPlayer keeps for the first forward press (`sprintTriggerTime` or its 26.2 name): where it is set, decremented and read in `aiStep`, and the exact tick order relative to `Input.tick`, `KeyboardInput`, and the sprint key's own path;
- the full predicate that starts sprinting from a second forward press: the forward-impulse threshold, food, blindness, using an item, sneaking, swimming, flying and collision; and whether a value of 0 disables it;
- what stops sprinting, and whether a double-tap start differs from a sprint-key start in any later state (the server sees only the sprint command).

**Build a jar-backed reproduction:** drive a real LocalPlayer's input logic, or an exact transcription checked line by line against the bytecode. Print, for window values 0, 1, 7 and the maximum, the tick at which sprinting starts for these forward-key patterns: press, release r ticks, press again, for r = 0…window+1.

**Then check one of Julien's recordings:** apply the rule to the first 400 ticks of `C:/Users/PASCAL/Desktop/path tracing/jobs/hard-track/julien-428.jsonl`, using its recorded raw keys only. Print every tick where the predicted sprint state differs from the recorded `isSprinting`, with that tick's inputs.

**Falsifier:**
- a rule that needs any input not in the recording;
- or any tick where the prediction differs from `isSprinting`, unless the reproduction names the jar condition that explains it.

Leave the rig alone.
