# Generator rows: what each ground does to the walking body

**Goal:** one parameter row per block class that changes a walking or jumping player's motion in 26.2, each backed by bit-exact jar witnesses of the player on it.

The planner's words live in velocity space: on ground each horizontal axis is `v ← (v + input(yaw)) × friction × drag` (`map/architecture.md` section 2). Today only stone-like ground is flown; his tracks cross grass, flowers, mud, snow, frosted ice, carpet over water, soul sand and more (`lab/tracks/*.json`). This job does not port Rust. It extracts each ground's parameters from the jar so a ground is a row, not a code path ("one mechanism").

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**From the bytecode** (class, method, offsets), for every block whose properties or overrides touch player motion: `friction` (slipperiness), `speedFactor`, `jumpFactor`; `makeStuckInBlock` multipliers (cobweb, sweet berry bush, powder snow); `stepOn`/`entityInside`/`updateEntityMovementAfterFallOn` effects (slime bounce, honey slide cap, soul sand/soul soil with and without Soul Speed, magma, bubble columns); which block position supplies each factor (the block below the feet `getBlockPosBelowThatAffectsMyMovement`, the block at the feet, or both, and the offset used). Group blocks that give identical answers into one class ("a flower is air").

**Witnesses:** run the jar's own player movement code (a tiny Java harness against the jar, no server if you can avoid it) on a flat floor of each class: sprint from rest for 30 ticks and sprint-jump from rest for 30 ticks, yaw 0 and yaw 45°, writing bit-exact position, velocity and on-ground per tick.

**Write in your run folder:**
- `rows.tsv`: `class	blocks	friction_hex	speed_factor_hex	jump_factor_hex	stuck_multiplier	factor_source_pos	scars	source`.
- `witnesses.tsv`: `class	program	yaw_bits	tick	x	y	z	vx	vy	vz	on_ground` (floats as hex bits).
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both tables from the jar, and `wiki.md`: a short `lab/wiki/` note on which grounds share a class and which scars separate the rest.

**Falsifier:** any witness tick where `v ← (v + input) × friction × drag` with the row's factors (plus its listed scars) gives different bits than the jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
