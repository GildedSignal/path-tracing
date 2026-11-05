Stopped under your instruction to ask when scope is ambiguous.

**Should L0 freeze auto-jump off and `sprintWindow` to zero, or should this job include those option-dependent `LocalPlayer` behaviours?** Bytecode confirms both can generate movement without the corresponding key being pressed. The lab copies pack settings, and recordings omit them.

Completed: removed all dependencies and extended the braking probe. Threshold ticks measured:

| Initial gait | Release | Reverse 1 | 2 | 3 | 4 | 5 | 6 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Stone walk | 8 | **5** | 8 | 10 | 11 | 12 | 13 |
| Stone sprint | 8 | **6** | 8 | 10 | 11 | 12 | 13 |
| Ice walk | 38 | 36 | 35 | 33 | 30 | 25 | **7** |
| Ice sprint | 40 | 39 | 38 | 36 | 35 | 32 | **27** |

These are Java recurrence results, not L0 evidence. Every reverse protocol ends with release.

No Rust movement functions are ported yet; no `unimplemented!` branches exist. Cargo tests and mutant builds remain unrun. Changes are uncommitted.

Details and reproduction commands: [engine/README.md](/D:/path-tracing-wt/law/engine/README.md).