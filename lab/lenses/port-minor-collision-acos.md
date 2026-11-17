# Port delta: LocalPlayer.isHorizontalCollisionMinor acos domain and NaN behavior

Use the staged Mojang 26.2 client jar in `D:/path-tracing-deps/data` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Anchor to `engine/law/src/movement.rs:803-819` (`minor_collision`) and `jobs/port-backlog/report.md` #35 (`monotone-tick`).

**Consumer:** batch `P1a` (`jobs/port-backlog/report.md`), `engine/law/src/movement.rs:818`, and `monotone-tick` (where `dot / sqrt(a * b) > 1.0` produced `acos` NaN on 14/255,150 ticks).

**From the 26.2 client bytecode**, state with class, method and offsets:
- `LocalPlayer.isHorizontalCollisionMinor` (or its 26.2 method name, to locate): every arithmetic step from `Vec2`/`Vec3` input rotation (`Mth.sin`/`Mth.cos` `(F)F` vs `(D)F`, `0.017453292f`), `1.0E-5F` horizontal squared-length guards (`lengthSquared()`), `dot` product, `Math.sqrt(a * b)`, `Math.acos(...)`, and the `0.13962633907794952` (`8°` in radians) comparison (`dcmpg`/`dcmpl`);
- critically verify whether Mojang clamps `dot / Math.sqrt(a * b)` to `[-1.0, 1.0]` before `Math.acos`, or passes the unclamped quotient to `Math.acos` so `NaN < 0.13962633907794952` evaluates to `false` (stopping sprint, matching unpatched `movement.rs:818`!).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LocalPlayer.isHorizontalCollisionMinor` and calling the jar method directly on inputs where `dot / Math.sqrt(a * b) > 1.0` in IEEE-754 double precision, printing raw argument bits, `Math.acos` result, and boolean return (`true` vs `false`). Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any claim that Mojang clamps (or does not clamp) `acos` that disagrees with the bytecode and direct jar call on a `> 1.0` quotient input.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
