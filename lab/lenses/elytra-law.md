# Lens: the elytra law, transcribed (jar reader)

**Why it matters.** Rung R-brach in `docs/ladder.md`: once the walking law passes, the elytra law is ported, and the bot must find the time-optimal glide (a brachistochrone) without new search code. The port must be bit-exact, and `docs/engine.md` notes that elytra uses `Math.cos`, which HotSpot may implement differently from `StrictMath`.

**Getting the jar:** as in the `hook-catalogue` lens (the official manifest, hash checked, `javap`; nothing from the jar committed).

**What to produce.**
- `transcript.md`: for the code path a player follows while fall-flying (start from `LivingEntity.travel` and follow the fall-flying branch, including how it is entered and left), every arithmetic operation in order: operands, precision, each `f2d`/`d2f`/`i2f`, each constant with its exact value, each call to `Math`, `Mth` or `StrictMath`, with class, method and offset. Pseudo-code is fine; pasting bytecode is not.
- `reproduce.sh`: regenerates the list of offsets and constants from the jar mechanically, so the checker can diff it against `transcript.md`.
- `dossier.md`: the claim (one sentence), which `Math` calls appear, and for each whether Java guarantees its result (StrictMath semantics) or leaves it to the platform, with the Java SE specification section that says so.
