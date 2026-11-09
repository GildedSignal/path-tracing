#[path = "support/footprints.rs"]
mod support;
use law::*;
use support::{FullScan, Random, state_bits};

struct SuffocatingOnly<'a> { world: &'a BoxWorld, air: Block }
impl World for SuffocatingOnly<'_> {
    fn block(&self, p: BlockPos) -> &Block {
        let block = self.world.block(p);
        if block.suffocating { block } else { &self.air }
    }
    fn loaded(&self, p: BlockPos) -> bool { self.world.loaded(p) }
}

fn planes(shape: law::shapes::ShapeView<'_>) -> [Vec<u64>; 3] {
    std::array::from_fn(|axis| (0..shape.shape.coords[axis].len()).map(|i| shape.coord(axis, i).to_bits()).collect())
}

#[test]
fn random_box_queries_equal_full_scan() {
    for seed in 0..32 {
        let mut random = Random(seed);
        let mut world = support::world(&mut random, seed);
        // Force overlap, later-region precedence, override precedence, and a
        // collider whose owning cell is outside the tight bounds in every seed.
        world.fill([0, 0, 0], [1, 1, 1], Block::full());
        world.fill([0, 0, 0], [0, 0, 0], Block::default());
        world.set([0, 0, 0], Block::full());
        world.set([1, 0, 0], Block { suffocating: true,
            ..Block::boxes(&[Aabb::new(-0.5, 0.0, 0.0, 1.0, 1.0, 1.0)]) });
        world.extra_shapes.push(VoxelShape::boxes(&[Aabb::new(-0.75, 0.0, -0.75, -0.5, 2.0, -0.5)]));
        let slow = FullScan(&world);
        let suffocating = SuffocatingOnly { world: &world, air: Block::default() };
        for trial in 0..12 {
            let bounds = if trial == 0 { Aabb::new(0.6, 0.1, 0.1, 0.9, 0.9, 0.9) } else {
                let p = Vec3::new(random.coordinate(), random.coordinate(), random.coordinate());
                Aabb::player(p, Pose::Standing.height())
            };
            let context = CollisionContext { feet_y: bounds.min.y, descending: random.int(2) == 0,
                fall_distance: random.int(5) as f64, leather_boots: random.int(2) == 0 };
            let expected: Vec<_> = slow.collisions(bounds, context).iter().map(|s| planes(s.view())).collect();
            let mut actual = Vec::new();
            assert!(world.visit_collisions(bounds, context, |s| { actual.push(planes(s)); true }));
            assert_eq!(actual, expected, "seed {seed} trial {trial}");
            assert_eq!(world.no_collision(bounds, context), expected.is_empty(), "seed {seed} trial {trial}");
            assert_eq!(world.collides_with_suffocating_block(bounds, context),
                !suffocating.collisions(bounds, context).is_empty(), "seed {seed} trial {trial}");
        }
    }
}

// Preserve the collision fast paths while overriding just the two flags.
struct ForcedHooks<'a>(&'a BoxWorld);
impl World for ForcedHooks<'_> {
    fn block(&self, p: BlockPos) -> &Block { self.0.block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.0.loaded(p) }
    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        self.0.collision_cell_bounds(bounds, lo, hi)
    }
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool {
        self.0.visit_collision_cells(bounds, lo, hi, visit)
    }
}

#[test]
fn tick_equals_forced_fluid_and_inside_hooks() {
    for seed in 0..48 {
        let mut random = Random(seed);
        let mut world = if seed%4 == 0 { BoxWorld::default() } else { support::world(&mut random, seed) };
        world.fill([-4, -1, -4], [4, -1, 4], Block::full());
        // Positive controls guarantee both true branches matter to the tick.
        if seed%4 == 1 { world.set([0, 0, 0], Block { inside: Inside::Cobweb, air: false, ..Block::default() }); }
        if seed%4 == 2 { world.set([0, 0, 0], Block { air: false,
            fluid: Fluid { kind: FluidKind::Water, height: 0.8, flow: Vec3::new(0.5, 0.0, 0.0), ..Fluid::default() }, ..Block::default() }); }
        let mut fast = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), on_ground: true, ..PlayerState::default() };
        let mut forced = fast.clone();
        for step in 0..8 {
            let input = random.input();
            tick(&mut fast, &world, &input);
            tick(&mut forced, &ForcedHooks(&world), &input);
            assert_eq!(state_bits(&fast), state_bits(&forced), "seed {seed} step {step}");
        }
    }
}
