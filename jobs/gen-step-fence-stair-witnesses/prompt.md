# Jar witnesses: the player against steps, fences and stairs

**Goal:** bit-exact jar witnesses of a moving player meeting a step, a slab, stairs, a fence, a wall and a carpet in 26.2, so the planner's joints at these defects are checked against Mojang.

The next flights cross steps, fences and stairs (`map/now.md`: they are refused today as `NeedsJoints`). A joint is where one word may end and another begin, at a face (`map/architecture.md` section 2). Its correctness rests on the collision and step-up law: `Entity.collide`, `collideBoundingBox`, step-up with `maxUpStep` 0.6, the order of axes, and each block's collision shape. This job does not port Rust; it produces the witnesses the law and the planner are judged by.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Cases** (each a tiny Java harness on the jar's own classes, no server if you can avoid it): a player walking, sprinting and sprint-jumping into (a) a full block step (1.0, refused by step-up), (b) a bottom slab (0.5, stepped), (c) stairs from the low side, the high side and the side, in all four facings and the inner/outer corner shapes, (d) a fence and a wall (1.5 tall collision), approached square and at 45°, including the gap between a fence post and an unconnected neighbour, (e) carpet, snow layers 1–8, and a trapdoor open and closed. Starts at 3 distances and 3 speeds per case, yaws on the law's sine table (`Mth.sin` index), 40 ticks each.

**From the bytecode**, state with class, method and offsets: the step-up algorithm's order (which candidate heights are tried, how the best is chosen), the horizontal collision flag and `minorHorizontalCollision`, and each shape's boxes for the blocks above.

**Write in your run folder:**
- `witnesses.tsv`: `case	block_state	facing	program	yaw_index	start_x	start_y	start_z	start_vx	start_vy	start_vz	tick	x	y	z	vx	vy	vz	on_ground	horizontal_collision` (floats as hex bits).
- `shapes.tsv`: `block_state	box_index	minX	minY	minZ	maxX	maxY	maxZ` (hex bits) for every state used.
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both from the jar, and `wiki.md`: a short `lab/wiki/` note on the step-up rule and which of these blocks a sprint-jump clears.

**Falsifier:** any row the existing law (`engine/law/src/movement.rs` on `main`) reproduces with different bits; list those rows first in the dossier, they are the most important data.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
