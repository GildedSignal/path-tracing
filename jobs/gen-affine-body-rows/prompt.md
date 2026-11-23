# Generator rows: every body's drag-and-gravity recurrence

**Goal:** one parameter row per moving body class of 26.2 for the shared `affine_body` generator, each row backed by bit-exact jar trajectories.

The law is being compressed onto a few shared generators (`map/now.md`, step 2). `affine_body` is the recurrence every body runs between contacts: per axis `v ← (v + input) × drag (+ gravity, before or after the drag)`. This job does not port Rust. It extracts the parameters and the irregular bits ("scars") from the jar so the generator can be checked against Mojang.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Bodies:** player (walking air, in water, in lava, levitation, slow falling), item entity, experience orb, falling block, primed TNT, boat (in air, on water, on ice), minecart (off rail), arrow, trident, snowball/egg/ender pearl (`ThrowableProjectile`), fishing bobber, wind charge, firework rocket (unboosted).

**Per body, from the bytecode** (class, method, bytecode offsets): gravity constant and its type (float or double); horizontal and vertical drag in air, water and lava; whether gravity applies before or after drag, and before or after the move; the rest threshold (a velocity zeroed below some value); and every scar — anything that is not the plain recurrence (for example a `0.98` applied only on ground, a water buoyancy term, an inertia that depends on block friction). A scar is one row with its method and offset, never prose.

**Witnesses:** run the jar's own classes (a tiny Java harness against the jar on the classpath, no server start if you can avoid it) for 40 ticks from 3 start velocities per body and regime, and write the bit-exact position and velocity per tick as hex.

**Write in your run folder:**
- `rows.tsv`: `body	regime	gravity_hex	gravity_type	drag_h_hex	drag_v_hex	order	rest_threshold_hex	scars	source` (one line per body and regime; `source` = class.method@offset).
- `witnesses.tsv`: `body	regime	start_vx	start_vy	start_vz	tick	x	y	z	vx	vy	vz` (all floats as hex bits).
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both tables from the jar, and `wiki.md`: a short note for `lab/wiki/` naming the bodies that share one row and the scars that keep the others apart.

**Falsifier:** any witness tick where the plain recurrence with the row's parameters (plus its listed scars) gives different bits than the jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
