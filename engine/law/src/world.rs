use std::collections::BTreeMap;
use crate::{
    math,
    Aabb,
    PlayerState,
    Vec3,
    VoxelShape
};
pub type BlockPos = [i32; 3];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FluidKind {
    #[default] Empty,
    Water,
    Lava
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Fluid {
    pub kind: FluidKind,
    /// Surface height for snapshots that already supply the evaluated value.
    pub height: f32,
    /// Current for snapshots that already supply the evaluated vector.
    pub flow: Vec3,
    /// Vanilla amount for a raw water or lava state, in 1..=8.
    pub raw_amount: Option<u8>,
    /// Vanilla FALLING property. A raw current needs both this and raw_amount.
    pub raw_falling: Option<bool>,
}

// Direction.Plane.HORIZONTAL
const HORIZONTAL: [[i32; 2]; 4] = [[0, -1], [1, 0], [0, 1], [-1, 0]];

impl Fluid {
    // FlowingFluid.getHeight; FlowingFluid.hasSameAbove; FlowingFluid.getOwnHeight
    pub fn surface_height(self, world: &(impl World + ?Sized), [x, y, z]: BlockPos) -> f32 {
        if self.raw_amount.is_none() {
            return self.height;
        }
        let own_height = self.own_height([x, y, z]);
        if raw_fluid_block(world, [x, y + 1, z]).fluid.kind == self.kind {
            1.0
        } else {
            own_height
        }
    }

    fn own_height(self, position: BlockPos) -> f32 {
        match self.raw_amount {
            Some(amount @ 1..=8) => amount as f32 / 9.0,
            _ => panic!("raw fluid current or height needs a valid amount at {position:?}"),
        }
    }

    // FlowingFluid.getFlow; FlowingFluid.affectsFlow; FlowingFluid.isSolidFace
    pub fn current(self, world: &(impl World + ?Sized), [x, y, z]: BlockPos) -> Vec3 {
        let position = [x, y, z];
        if self.raw_amount.is_none() && self.raw_falling.is_none() {
            return self.flow;
        }
        let falling = self.raw_falling.unwrap_or_else(|| {
            panic!("raw fluid current needs FALLING at {position:?}")
        });
        let height = self.own_height(position);
        let mut velocity = Vec3::default();
        for [dx, dz] in HORIZONTAL {
            let side = [x + dx, y, z + dz];
            let neighbor = raw_fluid_block(world, side);
            let difference = if neighbor.fluid.kind == self.kind {
                height - neighbor.fluid.own_height(side)
            } else if neighbor.fluid.kind == FluidKind::Empty && !neighbor.flow_properties(side).blocks_motion {
                let below = [side[0], y - 1, side[2]];
                let fluid = raw_fluid_block(world, below).fluid;
                if fluid.kind != self.kind {
                    continue;
                }
                height - (fluid.own_height(below) - 0.8888889f32)
            } else {
                continue;
            };
            if difference != 0.0 {
                velocity.x += (dx as f32 * difference) as f64;
                velocity.z += (dz as f32 * difference) as f64;
            }
        }
        if falling {
            for (direction, [dx, dz]) in HORIZONTAL.into_iter().enumerate() {
                let wall = |p| {
                    let block = raw_fluid_block(world, p);
                    block.fluid.kind != self.kind && block.flow_properties(p).solid_faces[direction]
                };
                if wall([x + dx, y, z + dz])
                    || (!cfg!(feature = "mutant-fluid-wall-above") && wall([x + dx, y + 1, z + dz]))
                {
                    velocity = velocity.normalize().plus(Vec3::new(0.0, -6.0, 0.0));
                    break;
                }
            }
        }
        velocity.normalize()
    }
}

/// Block predicates used by FlowingFluid.getFlow.
#[derive(Clone, Copy, Debug, Default)]
pub struct FluidFlowBlock {
    /// Vanilla BlockState.blocksMotion.
    pub blocks_motion: bool,
    /// isFaceSturdy with IceBlock excluded, in north, east, south, west order.
    pub solid_faces: [bool; 4],
}

fn raw_fluid_block(world: &(impl World + ?Sized), position: BlockPos) -> &Block {
    world.raw_fluid_block(position).unwrap_or_else(|| {
        panic!("raw fluid neighborhood is unavailable at {position:?}")
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Inside {
    #[default] None,
    Cobweb,
    PowderSnow,
    Honey,
    Unsupported(String)
}

/// A damage request emitted by a grounded contact hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactDamage {
    HotFloor,
}

impl ContactDamage {
    pub fn raw_amount(self) -> f32 {
        match self {
            Self::HotFloor => 1.0,
        }
    }

    pub(crate) fn exhaustion(self) -> f32 {
        match self {
            Self::HotFloor => 0.1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum GroundedContact {
    #[default]
    None,
    DamageUnlessCareful(ContactDamage),
}

#[derive(Clone, Debug)]
pub struct Block {
    pub shape: VoxelShape,
    pub inside_shape: Option<VoxelShape>,
    pub water_block: bool,
    pub above_shape: Option<VoxelShape>,
    pub below_shape: Option<VoxelShape>,
    pub sneak_shape: Option<VoxelShape>,
    pub friction: f32,
    pub speed_factor: f32,
    pub jump_factor: f32,
    pub climbable: bool,
    pub scaffolding: bool,
    pub ladder_facing: Option<u8>,
    pub open_trapdoor_facing: Option<u8>,
    pub fluid: Fluid,
    pub fluid_flow: Option<FluidFlowBlock>,
    pub inside: Inside,
    pub grounded_contact: GroundedContact,
    pub stuck_multiplier: Option<Vec3>,
    pub unsupported_fall_on: Option<String>,
    pub fall_on_multiplier: f32,
    pub bed: bool,
    pub suffocating: bool,
    pub bounce: f32,
    pub suppresses_bounce: bool,
    pub slime: bool,
    pub fence: bool,
    pub wall_or_gate: bool,
    pub resets_fall_distance: bool,
    pub end_portal_or_gateway: bool,
    pub air: bool,
}
impl Default for Block {
    // BlockBehaviour.Properties; Block
    fn default() -> Self {
        Self {
            shape: VoxelShape::default(),
            inside_shape: None,
            water_block: false,
            above_shape: None,
            below_shape: None,
            sneak_shape: None,
            friction: 0.6,
            speed_factor: 1.0,
            jump_factor: 1.0,
            climbable: false,
            scaffolding: false,
            ladder_facing: None,
            open_trapdoor_facing: None,
            fluid: Fluid::default(),
            fluid_flow: None,
            inside: Inside::None,
            grounded_contact: GroundedContact::None,
            stuck_multiplier: None,
            unsupported_fall_on: None,
            fall_on_multiplier: 1.0,
            bed: false,
            suffocating: false,
            bounce: 0.0,
            suppresses_bounce: false,
            slime: false,
            fence: false,
            wall_or_gate: false,
            resets_fall_distance: false,
            end_portal_or_gateway: false,
            air: true
        }
    }
}
impl Block {
    fn flow_properties(&self, position: BlockPos) -> FluidFlowBlock {
        self.fluid_flow.unwrap_or_else(|| {
            panic!("raw fluid current needs block predicates at {position:?}")
        })
    }

    // Shapes.create
    pub fn boxes(boxes: &[Aabb]) -> Self {
        Self {
            shape: VoxelShape::boxes(boxes),
            air: false,
            ..Self::default()
        }
    }

    // Shapes.block
    pub fn full() -> Self {
        Self {
            suffocating: true,
            ..Self::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)])
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionContext {
    pub feet_y: f64,
    pub descending: bool,
    pub fall_distance: f64,
    pub leather_boots: bool
}
impl CollisionContext {
    // EntityCollisionContext.<init>
    pub fn of(state: &PlayerState) -> Self {
        Self {
            feet_y: state.bounding_box().min.y,
            descending: state.shift_key_down,
            fall_distance: state.fall_distance,
            leather_boots: state.leather_boots
        }
    }
}
pub trait World {
    fn block(&self, position: BlockPos) -> &Block;

    /// Raw fluid queries need known cells, including known air. Worlds with
    /// incomplete coverage inside loaded chunks must return None at unknown cells.
    fn raw_fluid_block(&self, position: BlockPos) -> Option<&Block> {
        self.loaded(position).then(|| self.block(position))
    }

    // CollisionGetter.collidesWithSuffocatingBlock; BlockCollisions.computeNext
    fn collides_with_suffocating_block(&self, bounds: Aabb, context: CollisionContext) -> bool {
        let lo = block_pos(bounds.min.minus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        let hi = block_pos(bounds.max.plus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        for z in lo[2]-1..=hi[2]+1 {
            for y in lo[1]-1..=hi[1]+1 {
                for x in lo[0]-1..=hi[0]+1 {
                    let p = [x, y, z];
                    if !self.loaded(p) || !self.block(p).suffocating {
                        continue;
                    }
                    let shape = self.collision_shape(p, context);
                    let offset = Vec3::new(x as f64, y as f64, z as f64);
                    let intersects = if shape.is_block() {
                        bounds.intersects(Aabb { min: offset, max: offset.plus(Vec3::new(1.0, 1.0, 1.0)) })
                    } else {
                        shape.moved(offset).intersects(bounds)
                    };
                    if intersects {
                        return true;
                    }
                }
            }
        }
        false
    }

    // BlockGetter.clip; ClipContext.Block.FALLDAMAGE_RESETTING; ClipContext.Fluid.WATER; VoxelShape.clip
    fn fall_reset_clip(&self, from: Vec3, to: Vec3) -> bool {
        let movement = to.minus(from);
        if movement.length_sqr() < 1.0e-7 {
            return false;
        }
        let inside_point = from.plus(movement.scale(0.001));
        crate::blocks::traverse_blocks(from, to, |p| {
            let block = self.block(p);
            let height = if block.resets_fall_distance || block.end_portal_or_gateway {
                1.0
            } else if block.fluid.kind == FluidKind::Water {
                block.fluid.surface_height(self, p) as f64
            } else {
                return false;
            };
            if height < 1.0e-7 {
                return false;
            }
            let bounds = Aabb::new(p[0] as f64, p[1] as f64, p[2] as f64,
                p[0] as f64+1.0, p[1] as f64+height, p[2] as f64+1.0);
            bounds.contains(inside_point) || bounds.clip(from, to).is_some()
        })
    }

    // Level.hasChunkAt
    fn loaded(&self, _position: BlockPos) -> bool {
        true
    }

    // Level.getMinY
    fn min_y(&self) -> i32 {
        -64
    }

    // EnvironmentAttributes.FAST_LAVA
    fn fast_lava(&self) -> bool {
        false
    }

    // Level.getDifficulty
    fn difficulty(&self) -> u8 {
        2
    }

    // GameRules.NATURAL_HEALTH_REGENERATION
    fn natural_regeneration(&self) -> bool {
        true
    }

    // Level.getEntityCollisions; WorldBorder.getCollisionShape
    fn extra_collisions(&self, _bounds: Aabb, _context: CollisionContext) -> Vec<VoxelShape> {
        Vec::new()
    }

    // BlockState.getCollisionShape; PowderSnowBlock.getCollisionShape
    fn collision_shape(&self, position: BlockPos, context: CollisionContext) -> &VoxelShape {
        let block = self.block(position);
        let above = context.feet_y > position[1] as f64 + 1.0 - 1.0e-5f32 as f64;
        if block.inside == Inside::PowderSnow {
            if context.fall_distance > 2.5 {
                static SHAPE: std::sync::OnceLock<VoxelShape> = std::sync::OnceLock::new();
                return SHAPE.get_or_init(|| VoxelShape::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 0.9f32 as f64, 1.0)]));
            }
            if context.leather_boots && above && !context.descending {
                static SHAPE: std::sync::OnceLock<VoxelShape> = std::sync::OnceLock::new();
                return SHAPE.get_or_init(|| Block::full().shape);
            }
            static SHAPE: std::sync::OnceLock<VoxelShape> = std::sync::OnceLock::new();
            return SHAPE.get_or_init(VoxelShape::default);
        }
        if above && context.descending {
            if let Some(state) = &block.sneak_shape {
                return state;
            }
        }
        if let Some(state) = if above {
            &block.above_shape
        } else {
            &block.below_shape
        }
        {
            return state;
        }
        &block.shape
    }

    // Entity.collectCollidersIgnoringWorldBorder; BlockCollisions.computeNext
    fn collisions(&self, bounds: Aabb, context: CollisionContext) -> Vec<VoxelShape> {
        let mut result = self.extra_collisions(bounds, context);
        let lo = block_pos(bounds.min.minus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        let hi = block_pos(bounds.max.plus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        for z in lo[2]-1..=hi[2]+1 {
            for y in lo[1]-1..=hi[1]+1 {
                for x in lo[0]-1..=hi[0]+1 {
                    let pos = [x, y, z];
                    if !self.loaded(pos) {
                        continue;
                    }
                    let local = self.collision_shape(pos, context);
                    if local.is_empty() {
                        continue;
                    }
                    let offset = Vec3::new(x as f64, y as f64, z as f64);
                    if local.is_block() {
                        if bounds.intersects(Aabb {
                            min:offset, max:offset.plus(Vec3::new(1.0, 1.0, 1.0))
                        }) {
                            result.push(local.moved(offset));
                        }
                    } else {
                        let shape = local.moved(offset);
                        if shape.intersects(bounds) {
                            result.push(shape);
                        }
                    }
                }
            }
        }
        result
    }

    // CollisionGetter.noCollision
    fn no_collision(&self, bounds: Aabb, context: CollisionContext) -> bool {
        self.collisions(bounds, context).is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct BoxWorld {
    pub blocks: BTreeMap<BlockPos, Block>,
    pub regions: Vec<(BlockPos, BlockPos, Block)>,
    pub minimum_y: i32,
    pub fast_lava: bool,
    pub difficulty: u8,
    pub natural_regeneration: bool,
    pub unloaded_chunks: Vec<[i32; 2]>,
    pub extra_shapes: Vec<VoxelShape>,
    air: Block,
}
impl Default for BoxWorld {
    // Level.getMinY; Level.getDifficulty; GameRules.NATURAL_HEALTH_REGENERATION
    fn default() -> Self {
        Self {
            blocks:BTreeMap::new(),
            regions:Vec::new(),
            minimum_y:-64,
            fast_lava:false,
            difficulty:2,
            natural_regeneration:true,
            unloaded_chunks:Vec::new(),
            extra_shapes:Vec::new(),
            air:Block { fluid_flow: Some(FluidFlowBlock::default()), ..Block::default() }
        }
    }
}
impl BoxWorld {
    // BlockGetter.getBlockState
    pub fn set(&mut self, pos: BlockPos, block: Block) {
        self.blocks.insert(pos, block);
    }

    // BlockGetter.getBlockState
    pub fn fill(&mut self, min: BlockPos, max: BlockPos, block: Block) {
        self.regions.push((min, max, block));
    }
}
impl World for BoxWorld {
    // Level.hasChunkAt
    fn loaded(&self, p: BlockPos) -> bool {
        !self.unloaded_chunks.contains(&[p[0]>>4, p[2]>>4])
    }

    // Level.getMinY
    fn min_y(&self) -> i32 {
        self.minimum_y
    }

    // EnvironmentAttributes.FAST_LAVA
    fn fast_lava(&self) -> bool {
        self.fast_lava
    }

    // Level.getDifficulty
    fn difficulty(&self) -> u8 {
        self.difficulty
    }

    // GameRules.NATURAL_HEALTH_REGENERATION
    fn natural_regeneration(&self) -> bool {
        self.natural_regeneration
    }

    // Level.getEntityCollisions; WorldBorder.getCollisionShape
    fn extra_collisions(&self, bounds: Aabb, _context: CollisionContext) -> Vec<VoxelShape> {
        self.extra_shapes.iter().filter(|state|state.intersects(bounds)).cloned().collect()
    }

    // BlockGetter.getBlockState
    fn block(&self, pos: BlockPos) -> &Block {
        if let Some(block) = self.blocks.get(&pos) {
            return block;
        }
        for (min, max, block) in self.regions.iter().rev() {
            if (0..3).all(|a| pos[a] >= min[a] && pos[a] <= max[a]) {
                return block;
            }
        }
        &self.air
    }
}

// BlockPos.containing
pub fn block_pos(p: Vec3) -> BlockPos {
    [math::floor(p.x), math::floor(p.y), math::floor(p.z)]
}
