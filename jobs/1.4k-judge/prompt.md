# Judge job 1.4k: the footprint gate's thirteenth mutant

You are the **judge** (AGENTS.md: Sonnet judges GPT's work, fresh context). Read only. The spec is `C:\Users\PASCAL\Desktop\path tracing\jobs\1.4k-inside-cell-mutant\prompt.md`; the builder's report is `jobs\1.4k-inside-cell-mutant\last.md`; the worktree is `D:\path-tracing-wt\inside-cell` (uncommitted). This touches a **gate** (`jobs/1.2-footprints/verify.ps1`), so the bar is: it only strengthens, and it tests what it claims.

The builder could not find two exported scaffolding states that differ **only** in the inside-cell shape (the two candidates also differ in the sneak shape), so it built a synthetic cell: the capped model with the uncapped model's inside-cell shape substituted. Decide:
1. Is the synthetic cell a fair test of the key (it asserts the other four contexts are equal, the tick's result differs, and the digests differ), or does it test something no real block can reach? Does that matter for a key test?
2. Does the new mutant drop exactly `inside_cell` and nothing else, and does it run the new test rather than an unrelated one?
3. Does anything in the diff weaken an existing mutant, test or gate?

Verdict PASS, PASS WITH FINDINGS, or FAIL, each finding with file and line. No invented findings. Under 30 lines.

## The diff, verbatim

```diff --git a/engine/world/tests/blocks.rs b/engine/world/tests/blocks.rs
index 6844d4f..573d117 100644
--- a/engine/world/tests/blocks.rs
+++ b/engine/world/tests/blocks.rs
@@ -24,6 +24,33 @@ fn inside_cell_collision_matches_jar_bottom_cap() {
     }
 }
 
+#[test]
+fn inside_cell_shape_is_keyed_in_recorded_footprint() {
+    use law::footprint::{Query, RecordingWorld};
+    let cap = model(&state("scaffolding[bottom=true,distance=1,waterlogged=false]")).unwrap();
+    let uncapped = model(&state("scaffolding[bottom=false,distance=1,waterlogged=false]")).unwrap();
+    let mut open = cap.clone();
+    open.inside_cell_shape = uncapped.inside_cell_shape.clone();
+    for (a, b) in [(&cap.shape, &open.shape), (cap.above_shape.as_ref().unwrap(), open.above_shape.as_ref().unwrap()),
+        (cap.below_shape.as_ref().unwrap(), open.below_shape.as_ref().unwrap()), (cap.sneak_shape.as_ref().unwrap(), open.sneak_shape.as_ref().unwrap())] {
+        assert_eq!(a.coords, b.coords);
+    }
+    assert_ne!(cap.inside_cell_shape.as_ref().unwrap().coords, open.inside_cell_shape.as_ref().unwrap().coords);
+    let mut world = BoxWorld::default();
+    world.set([0,0,0], cap.clone());
+    let initial = PlayerState { position: Vec3::new(0.5, 0.2, 0.5), velocity: Vec3::new(0.0,-0.2,0.0), ..Default::default() };
+    let input = Input { sneak: true, ..Default::default() };
+    let mut capped = initial.clone();
+    let record = RecordingWorld::new(&world).tick(&mut capped, &input);
+    let mask = record.footprint.reads[&[0,0,0]];
+    assert_ne!(mask & Query::Shape as u8, 0);
+    world.set([0,0,0], open.clone());
+    let mut uncapped = initial;
+    law::tick(&mut uncapped, &world, &input);
+    assert_ne!(capped.position.y.to_bits(), uncapped.position.y.to_bits());
+    assert_ne!(law::pattern::cell_digest(cap, true, mask), law::pattern::cell_digest(&open, true, mask));
+}
+
 #[test]
 fn descending_climbable_caps_motion_and_bottom_plate_stops_it() {
     let mut world = BoxWorld::default();
diff --git a/jobs/1.2-footprints/verify.ps1 b/jobs/1.2-footprints/verify.ps1
index 27fa8c3..aaef912 100644
--- a/jobs/1.2-footprints/verify.ps1
+++ b/jobs/1.2-footprints/verify.ps1
@@ -58,6 +58,7 @@ try {
     $answers = @('test', '--offline', '-p', 'law', '--test', 'footprints', 'edits_that_keep_a_cells_answers_cannot_alter_tick')
     Mutant 'fluid-unkeyed' 'law/src/pattern.rs' 'groups & (FLUID | HOOKS) != 0' 'groups & HOOKS != 0' $answers
     Mutant 'sneak-unkeyed' 'law/src/pattern.rs' 'for shape in [shape, above, below, sneak, inside_cell] {' 'let _ = sneak; for shape in [shape, above, below, inside_cell] {' $answers
+    Mutant 'inside-cell-unkeyed' 'law/src/pattern.rs' 'for shape in [shape, above, below, sneak, inside_cell] {' 'let _ = inside_cell; for shape in [shape, above, below, sneak] {' @('test', '--offline', '-p', 'world', '--test', 'blocks', 'inside_cell_shape_is_keyed_in_recorded_footprint')
     # A shift that leaves a binade must not be called exact.
     Mutant 'scope-unbounded' 'law/src/pattern.rs' '(0..3).all(|a| shift[a] == 0 ||' '(0..3).all(|a| true ||' @('test', '--offline', '-p', 'law', '--test', 'pattern', 'shifts_inside_the_binades_replay_bit_exact')
     # The fluid check's loaded reads at absolute chunk corners: a shifted key no longer sees them.
```n