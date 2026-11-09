use law::*;

pub struct Random(pub u64);
impl Random {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 16
    }
    pub fn int(&mut self, n: i32) -> i32 { (self.next() % n as u64) as i32 }
    pub fn coordinate(&mut self) -> f64 { (self.int(33)-16) as f64 / 8.0 }
    pub fn input(&mut self) -> Input {
        let keys = self.next();
        Input { forward: keys&1 != 0, back: keys&2 != 0, left: keys&4 != 0, right: keys&8 != 0,
            jump: keys&16 != 0, sneak: keys&32 != 0, sprint: keys&64 != 0,
            yaw: self.int(720) as f32-360.0, pitch: self.int(180) as f32-90.0 }
    }
}

pub fn block(random: &mut Random) -> Block {
    match random.int(7) {
        0 => Block::default(),
        1 => Block::full(),
        2 => Block { suffocating: true, ..Block::boxes(&[Aabb::new(-0.5, 0.0, -0.25, 1.5, 1.5, 1.25)]) },
        3 => Block::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)]),
        4 => Block { inside: Inside::Cobweb, air: false, ..Block::default() },
        5 => Block { air: false, fluid: Fluid { kind: FluidKind::Water, height: 0.8,
            flow: Vec3::new(0.25, 0.0, -0.5), ..Fluid::default() }, ..Block::default() },
        _ => Block { above_shape: Some(Block::full().shape), below_shape: Some(VoxelShape::default()),
            sneak_shape: Some(Block::full().shape), suffocating: true, ..Block::default() },
    }
}

pub fn world(random: &mut Random, seed: u64) -> BoxWorld {
    let mut world = BoxWorld::default();
    for _ in 0..4 {
        let lo = [random.int(5)-2, random.int(4)-2, random.int(5)-2];
        let hi = [lo[0]+random.int(3), lo[1]+random.int(2), lo[2]+random.int(3)];
        world.fill(lo, hi, block(random));
    }
    for _ in 0..12 {
        let p = [random.int(7)-3, random.int(5)-2, random.int(7)-3];
        world.set(p, block(random));
        if random.int(3) == 0 { world.set(p, Block::default()); }
    }
    if seed%3 == 0 { world.unloaded_chunks.push([-1, 0]); }
    world
}

/// Uses trait full scans and can force the two omission certificates true.
pub struct FullScan<'a>(pub &'a BoxWorld);
impl World for FullScan<'_> {
    fn block(&self, p: BlockPos) -> &Block { self.0.block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.0.loaded(p) }
    fn min_y(&self) -> i32 { self.0.min_y() }
    fn fast_lava(&self) -> bool { self.0.fast_lava() }
    fn difficulty(&self) -> u8 { self.0.difficulty() }
    fn natural_regeneration(&self) -> bool { self.0.natural_regeneration() }
    fn extra_collisions(&self, bounds: Aabb, context: CollisionContext) -> Vec<VoxelShape> {
        self.0.extra_collisions(bounds, context)
    }
}

pub fn state_bits(state: &PlayerState) -> Vec<u64> { state.bits() }
