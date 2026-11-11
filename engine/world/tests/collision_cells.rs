//! Job 1.6: the snapshot answers collision queries from its cells' own models. Is that
//! the full scan, on every one of Julien's tracks?
use std::path::PathBuf;
use law::{Aabb, Block, BlockPos, CollisionContext, Input, PlayerState, Pose, Vec3, World, tick};
use world::WorldSnapshot;

/// The same cells through the trait's own scan of every cell.
struct Scan<'a>(&'a WorldSnapshot);
impl World for Scan<'_> {
    fn block(&self, p: BlockPos) -> &Block { self.0.block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.0.loaded(p) }
    fn has_fluid(&self) -> bool { self.0.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.0.has_inside_effects() }
    fn difficulty(&self) -> u8 { self.0.difficulty() }
    fn natural_regeneration(&self) -> bool { self.0.natural_regeneration() }
}
struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); self.0 >> 16 }
    fn unit(&mut self) -> f64 { (self.next() % (1 << 24)) as f64 / (1u64 << 24) as f64 }
}
fn planes(shape: law::shapes::ShapeView<'_>) -> [Vec<u64>; 3] {
    std::array::from_fn(|axis| (0..shape.shape.coords[axis].len()).map(|i| shape.coord(axis, i).to_bits()).collect())
}

#[test]
fn the_snapshot_answers_collisions_as_the_full_scan_does() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (mut queries, mut colliding, mut from_the_halo, mut suffocating, mut ticks, mut refused) = (0, 0, 0, 0, 0, 0);
    for (track, line) in include_str!("fixtures/tracks.txt").lines().enumerate() {
        let n: Vec<i32> = line.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect();
        let (min, max) = ([n[0], n[1], n[2]], [n[3], n[4], n[5]]);
        let snapshot = WorldSnapshot::load(root.join("tests/fixtures/world"), min, max).unwrap();
        let scan = Scan(&snapshot);
        let mut random = Random(track as u64);
        let mut point = |random: &mut Random| Vec3::new(min[0] as f64 + 3.0 + random.unit() * (max[0] - min[0] - 5) as f64,
            min[1] as f64 + 3.0 + random.unit() * (max[1] - min[1] - 6).max(1) as f64, min[2] as f64 + 3.0 + random.unit() * (max[2] - min[2] - 5) as f64);
        for trial in 0..2000 {
            let p = point(&mut random);
            // Bodies, thin slabs of space, and boxes resting exactly on cell faces.
            let bounds = match trial % 4 {
                0 => Aabb::player(p, Pose::Standing.height()),
                1 => Aabb::new(p.x, p.y, p.z, p.x + random.unit() * 1.5, p.y + random.unit() * 0.6, p.z + random.unit() * 1.5),
                2 => Aabb::player(Vec3::new(p.x.floor(), p.y.floor(), p.z.floor() + 0.3), Pose::Crouching.height()),
                _ => Aabb::player(Vec3::new(p.x, p.y.floor(), p.z), Pose::Standing.height()).deflate(1.0e-7),
            };
            let context = CollisionContext { feet_y: bounds.min.y, descending: random.next() % 2 == 0,
                fall_distance: (random.next() % 5) as f64, leather_boots: random.next() % 2 == 0 };
            let collect = |world: &dyn Fn(&mut dyn FnMut(law::shapes::ShapeView<'_>) -> bool) -> bool| {
                let mut found = Vec::new();
                assert!(world(&mut |s| { found.push(planes(s)); true }));
                found
            };
            let expected = collect(&|visit| scan.visit_collisions(bounds, context, visit));
            let actual = collect(&|visit| snapshot.visit_collisions(bounds, context, visit));
            assert_eq!(actual, expected, "{line} trial {trial} {bounds:?}");
            assert_eq!(snapshot.no_collision(bounds, context), scan.no_collision(bounds, context));
            let suffocates = scan.collides_with_suffocating_block(bounds, context);
            assert_eq!(snapshot.collides_with_suffocating_block(bounds, context), suffocates);
            queries += 1;
            colliding += !expected.is_empty() as u32;
            suffocating += suffocates as u32;
            // A collider whose planes start outside the cells the box itself touches came from the halo.
            from_the_halo += expected.iter().any(|planes| (0..3).any(|a| {
                let (lo, hi) = ([bounds.min.x, bounds.min.y, bounds.min.z][a].floor(), [bounds.max.x, bounds.max.y, bounds.max.z][a].floor());
                let first = f64::from_bits(planes[a][0]).floor();
                first < lo || first > hi
            })) as u32;
        }
        // Whole ticks: the same bits, until the run leaves the snapshot.
        for run in 0..24 {
            let p = point(&mut random);
            let mut fast = PlayerState { position: Vec3::new(p.x, p.y.floor() + 3.0 * random.unit(), p.z), ..PlayerState::default() };
            let mut slow = fast.clone();
            for _ in 0..40 {
                let keys = random.next();
                let input = Input { forward: keys & 1 != 0, left: keys & 2 != 0, jump: keys & 4 != 0, sneak: keys & 24 == 24, sprint: keys & 32 != 0,
                    yaw: run as f32 * 15.0, ..Input::default() };
                let step = |world: &dyn Fn(&mut PlayerState), state: &PlayerState| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut next = state.clone();
                    world(&mut next);
                    next
                })).ok();
                let stepped = step(&|s| tick(s, &scan, &input), &slow);
                // Leaving the snapshot is refused by both, at the same tick.
                let fast_stepped = step(&|s| tick(s, &snapshot, &input), &fast);
                assert_eq!(fast_stepped.is_some(), stepped.is_some(), "{line} run {run}: one world refused alone");
                let (Some(next), Some(fast_next)) = (stepped, fast_stepped) else { refused += 1; break };
                (slow, fast) = (next, fast_next);
                assert_eq!(fast.bits(), slow.bits(), "{line} run {run}");
                ticks += 1;
            }
        }
    }
    println!("{queries} box queries equal the full scan: {colliding} collide, {from_the_halo} with a collider from the halo, {suffocating} suffocate; {ticks} ticks bit-equal; {refused} runs leave the snapshot, refused by both at the same tick");
    assert!(colliding > 1000 && from_the_halo > 20 && suffocating > 100 && ticks > 2000 && refused > 0);
}

/// Powder snow's stored shape is empty: its collision comes from the context (feet above
/// it, falling, leather boots). The reach visit must never skip it as a block that cannot
/// collide; without `never_collides`'s powder-snow clause, nothing else here would notice.
#[test]
fn powder_snow_is_never_skipped_as_a_block_that_cannot_collide() {
    let snow: Vec<_> = world::model_table().iter().filter(|b| b.inside == law::Inside::PowderSnow).collect();
    assert!(!snow.is_empty(), "the table holds powder snow");
    for block in snow {
        let stored = [Some(&block.shape), block.above_shape.as_ref(), block.below_shape.as_ref(), block.sneak_shape.as_ref()];
        assert!(stored.iter().flatten().all(|shape| shape.is_empty()), "the case the clause guards: no stored shape");
        assert!(!block.never_collides(), "powder snow can collide through its context");
    }
}
