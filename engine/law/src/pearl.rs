//! Ender pearl launch, flight, raytrace, and landing laws.
//!
//! `ThrowableItemProjectile.<init>` spawns at `(owner.x, eyeY - 0.1f, owner.z)`,
//! `EnderpearlItem.use` launches via `Projectile.shootFromRotation(0, 1.5f, 1.0f)`
//! without ticking on the spawn tick, `ItemStack.consume(1)` skips creative, and
//! `UseCooldown` applies `(int)(1.0f * 20.0f) = 20` ticks.
//! `ThrowableProjectile.tick` applies gravity (`-0.03`) before drag
//! (`0.8f32 as f64` in water, `0.99f32 as f64` in air), raytraces with
//! `ClipContext.Block.COLLIDER` and `ClipContext.Fluid.NONE` plus entity sweep,
//! sets `pos` onto the hit point, and `ThrownEnderpearl.onHit` teleports the
//! owner to `oldPosition()` (start-of-impact-tick position) with no clamp or
//! chunk check, then calls `resetFallDistance()` before `hurtServer(5.0f)`.
//! At DataVersion 5023, both landing branches zero the owner's velocity.

use crate::{interact, Aabb, CollisionContext, Vec3, World};

pub const PEARL_SHOOT_POWER: f32 = 1.5;
pub const PEARL_INACCURACY: f32 = 1.0;
pub const PEARL_PITCH_OFFSET: f32 = 0.0;
pub const PEARL_SPAWN_EYE_DROP: f64 = 0.1f32 as f64;
pub const PEARL_COOLDOWN_SECONDS: f32 = 1.0;
pub const PEARL_COOLDOWN_TICKS: i32 = (PEARL_COOLDOWN_SECONDS * 20.0f32) as i32;

pub const PEARL_DAMAGE: f32 = 5.0;
pub const PEARL_WIDTH: f32 = 0.25;
pub const PEARL_HEIGHT: f32 = 0.25;

/// `ThrowableItemProjectile.<init> @30-36`:
/// `(owner.x, owner.getEyeY() - 0.10000000149011612d, owner.z)`.
pub fn spawn_origin(owner_pos: Vec3, eye_height: f32) -> Vec3 {
    Vec3::new(
        owner_pos.x,
        interact::get_eye_position(owner_pos, eye_height).y - PEARL_SPAWN_EYE_DROP,
        owner_pos.z,
    )
}

/// `ItemCooldowns.isOnCooldown @16-28` / `ItemCooldowns.tick @71-110`: active while `endTime > tickCount`.
pub fn is_on_cooldown(end_time: Option<i32>, tick_count: i32) -> bool {
    end_time.is_some_and(|end| end > tick_count)
}

/// `ItemCooldowns.addCooldown @145-165`: replaces any active entry with `tickCount + durationTicks`.
pub fn add_cooldown_end_tick(tick_count: i32, duration_ticks: i32) -> i32 {
    tick_count + duration_ticks
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PearlUseInput {
    pub is_spectator: bool,
    pub cooldown_end_tick: Option<i32>,
    pub tick_count: i32,
    pub stack_count: i32,
    pub has_infinite_materials: bool,
    pub owner_pos: Vec3,
    pub owner_eye_height: f32,
    pub x_rot: f32,
    pub y_rot: f32,
    pub known_movement: Vec3,
    pub on_ground: bool,
    pub draws: [f64; 6],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PearlUseResult {
    Pass,
    Thrown {
        state: PearlState,
        y_rot: f32,
        x_rot: f32,
        remaining_stack: i32,
        cooldown_end_tick: i32,
    },
}

/// `ServerGamePacketListenerImpl.handleUseItem` (wrap degrees) ->
/// `ServerPlayerGameMode.useItem` (spectator / cooldown gate) ->
/// `EnderpearlItem.use` -> `Projectile.spawnProjectileFromRotation` ->
/// `ItemStack.consume(1, player)` -> `UseCooldown.apply`.
pub fn use_ender_pearl(input: &PearlUseInput) -> PearlUseResult {
    if input.is_spectator
        || input.stack_count <= 0
        || is_on_cooldown(input.cooldown_end_tick, input.tick_count)
    {
        return PearlUseResult::Pass;
    }
    let pos = spawn_origin(input.owner_pos, input.owner_eye_height);
    let shot = crate::basis::body::launch(crate::math::wrap_degrees(input.x_rot), crate::math::wrap_degrees(input.y_rot),
        PEARL_PITCH_OFFSET, PEARL_SHOOT_POWER, PEARL_INACCURACY, input.draws, input.known_movement, input.on_ground);
    let (vel, y_rot, x_rot) = (shot.velocity, shot.y_rot, shot.x_rot);
    let remaining_stack = if input.has_infinite_materials {
        input.stack_count
    } else {
        input.stack_count - 1
    };
    PearlUseResult::Thrown {
        state: PearlState::new(pos, vel),
        y_rot,
        x_rot,
        remaining_stack,
        cooldown_end_tick: add_cooldown_end_tick(input.tick_count, PEARL_COOLDOWN_TICKS),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PearlState {
    pub pos: Vec3,
    pub old_pos: Vec3,
    pub vel: Vec3,
    pub tick_count: i32,
    pub fall_distance: f64,
    pub in_water: bool,
}

impl PearlState {
    pub fn new(pos: Vec3, vel: Vec3) -> Self {
        Self {
            pos,
            old_pos: pos,
            vel,
            tick_count: 0,
            fall_distance: 0.0,
            in_water: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PearlEntityTarget {
    pub id: u64,
    pub aabb: Aabb,
    pub can_hit: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PearlHit {
    Miss,
    Block { location: Vec3 },
    Entity { id: u64, location: Vec3 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PearlLanding {
    pub teleport_pos: Vec3,
    pub reset_fall_distance: bool,
    pub reset_momentum: bool,
    pub reset_impulse_context: bool,
    pub damage: f32,
}

impl PearlLanding {
    pub fn owner_velocity(self, before: Vec3) -> Vec3 {
        if self.reset_momentum { Vec3::default() } else { before }
    }
}

/// `EntityDimensions.makeBoundingBox` for `EntityTypes.ENDER_PEARL` (0.25 x 0.25).
pub fn pearl_aabb(pos: Vec3) -> Aabb {
    Aabb::from_bottom(pos, PEARL_WIDTH, PEARL_HEIGHT)
}

/// `ProjectileUtil.computeMargin` (`@0-19`).
pub fn compute_margin(tick_count: i32) -> f32 {
    0.0f32.max(0.3f32.min((tick_count - 2) as f32 / 20.0f32))
}

/// `ProjectileUtil.getHitResultOnMoveVector` (`@0-32`) + `getHitResult` (`@0-85`):
/// `ClipContext.Block.COLLIDER`, `ClipContext.Fluid.NONE`, entity sweep
/// `bb.expandTowards(delta).inflate(1.0)` with `compute_margin`. Entity hit wins.
pub fn pearl_raytrace(
    world: &impl World,
    pearl: &PearlState,
    entities: &[PearlEntityTarget],
) -> PearlHit {
    trace_motion(world, pearl.pos, pearl.vel, pearl.tick_count, entities).0
}

fn trace_motion(
    world: &impl World,
    from: Vec3,
    delta: Vec3,
    tick_count: i32,
    entities: &[PearlEntityTarget],
) -> (PearlHit, Vec3) {
    let mut to = from.plus(delta);
    let ctx = CollisionContext::empty(from.y);

    let hit = crate::basis::ray::clip(from, to, crate::basis::ray::ClipFluidMode::None, |pos| (world.collision_shape(pos, ctx), None, 0.0, false));
    let block_hit = hit.is_hit().then(|| hit.location());
    let result = match block_hit {
        Some(location) => {
            to = location;
            PearlHit::Block { location }
        }
        None => PearlHit::Miss,
    };

    // ProjectileUtil.getHitResult @54-81: bb.expandTowards(delta).inflate(1.0)
    let sweep = pearl_aabb(from).expand(delta).deflate(-1.0);
    let margin = compute_margin(tick_count) as f64;
    let mut best_dist = f64::MAX;
    let mut best_entity = None;
    for target in entities {
        if !target.can_hit || !sweep.intersects(target.aabb) {
            continue;
        }
        let inflated = target.aabb.deflate(-margin);
        if let Some(loc) = inflated.clip_or_inside(from, to) {
            let dist = from.minus(loc).length_sqr();
            if dist < best_dist {
                best_dist = dist;
                best_entity = Some((target.id, loc));
            }
        }
    }
    if let Some((id, location)) = best_entity {
        return (PearlHit::Entity { id, location }, location);
    }
    (result, to)
}

/// One server frame (`ServerLevel.tickNonPassenger` + `ThrowableProjectile.tick` +
/// `ThrownEnderpearl.onHit`):
/// records `old_pos`, increments `tick_count`, applies gravity (`-0.03`) before drag
/// (`0.8f32` in water / `0.99f32` in air), raytraces, sets `pos` onto hit or endpoint,
/// and on impact with a valid owner returns `PearlLanding` targeting `old_pos`
/// (`reset_fall_distance` strictly before `5.0f` damage; `pearl.fall_distance` remains `0.0`).
pub fn pearl_tick(
    world: &impl World,
    pearl: &mut PearlState,
    entities: &[PearlEntityTarget],
    owner_teleportable: bool,
) -> Option<PearlLanding> {
    pearl.old_pos = pearl.pos;
    pearl.tick_count += 1;
    use crate::basis::body::{Medium, THROWABLE};
    let (hit, endpoint) = THROWABLE.advance(&mut pearl.vel, Medium::Steady(pearl.in_water), |delta| {
        trace_motion(world, pearl.pos, delta, pearl.tick_count, entities)
    });
    pearl.pos = endpoint;

    if hit != PearlHit::Miss && owner_teleportable {
        Some(PearlLanding {
            teleport_pos: pearl.old_pos,
            reset_fall_distance: true,
            reset_momentum: crate::basis::cell::DATA_VERSION == 5023,
            reset_impulse_context: true,
            damage: PEARL_DAMAGE,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, BoxWorld, Fluid, FluidFlowBlock, FluidKind};

    #[test]
    fn pearl_air_and_water_drag_gravity_order_witnesses() {
        let world = BoxWorld::default();
        let v0 = Vec3::new(0.4, 0.5, -0.2);
        let p0 = Vec3::new(0.5, 70.0, 0.5);

        // Air 8 ticks (evidence.txt:138-153) + 20 ticks continuity (check-adv-out.txt:3)
        let mut air = PearlState::new(p0, v0);
        let air_expected = [
            (
                [0x3fd9_5810_6666_6667, 0x3fdd_c779_ab85_1eb8, 0xbfc9_5810_6666_6667],
                [0x3fec_ac08_3333_3334, 0x4051_9dc7_79ab_851f, 0x3fd3_53f7_cccc_cccc],
            ),
            (
                [0x3fd9_172e_f8ca_819a, 0x3fdb_94a2_c30c_aa32, 0xbfc9_172e_f8ca_819a],
                [0x3ff4_9bcf_d7cc_3a00, 0x4051_b95c_1c6e_91c9, 0x3fbb_2181_419e_2ffc],
            ),
            (
                [0x3fd8_d6f3_a347_1210, 0x3fd9_676c_b8b6_3d6a, 0xbfc8_d6f3_a347_1210],
                [0x3ffa_d18c_c09d_fe84, 0x4051_d2c3_8927_4806, 0xbfb6_8c66_04ef_f424],
            ),
            (
                [0x3fd8_975c_bca8_c08b, 0x3fd7_3fc9_23e3_af79, 0xbfc8_975c_bca8_c08b],
                [0x4000_7bb1_f7e4_1753, 0x4051_ea03_524b_2bb5, 0xbfd1_eec7_df90_5d4e],
            ),
            (
                [0x3fd8_5868_9ffc_b8fa, 0x3fd5_1da9_c0d9_b1be, 0xbfc8_5868_9ffc_b8fa],
                [0x4003_86bf_0be3_ae72, 0x4051_ff20_fc0c_0567, 0xbfde_1afc_2f8e_b9cb],
            ),
            (
                [0x3fd8_1a15_ac85_c7de, 0x3fd3_0100_7061_622a, 0xbfc8_1a15_ac85_c7de],
                [0x4006_8a01_c174_676e, 0x4052_1221_fc7c_66c9, 0xbfe5_1403_82e8_cedd],
            ),
            (
                [0x3fd7_dc62_45b1_9394, 0x3fd0_e9bf_376a_cf55, 0xbfc7_dc62_45b1_9394],
                [0x4009_858e_0a2a_99e0, 0x4052_230b_bbb3_d198, 0xbfeb_0b1c_1455_33c2],
            ),
            (
                [0x3fd7_9f4c_d30d_f130, 0x3fcd_afb0_7d60_d7b5, 0xbfc7_9f4c_d30d_f130],
                [0x400c_7977_a48c_5806, 0x4052_31e3_93f2_8204, 0xbff0_7977_a48c_5807],
            ),
        ];
        for (i, (v_exp, p_exp)) in air_expected.iter().enumerate() {
            assert!(pearl_tick(&world, &mut air, &[], true).is_none());
            assert_eq!([air.vel.x.to_bits(), air.vel.y.to_bits(), air.vel.z.to_bits()], *v_exp, "air t{} v", i + 1);
            assert_eq!([air.pos.x.to_bits(), air.pos.y.to_bits(), air.pos.z.to_bits()], *p_exp, "air t{} pos", i + 1);
        }
        for _ in 9..=20 {
            assert!(pearl_tick(&world, &mut air, &[], true).is_none());
        }
        assert_eq!(air.tick_count, 20);

        // Enters water at t4 (evidence.txt:155-170)
        let mut wet = PearlState::new(p0, v0);
        let wet_expected = [
            (
                [0x3fd3_df29_5463_d8c7, 0x3fd2_c986_197d_8450, 0xbfc3_df29_5463_d8c7],
                [0x3fff_c957_15b6_f4b6, 0x4051_e58d_0f40_c58a, 0xbfcf_255c_56db_d2d9],
            ),
            (
                [0x3fcf_cb75_5bc5_d1c7, 0x3fca_fcce_64c7_8438, 0xbfbf_cb75_5bc5_d1c7],
                [0x4001_e162_e097_d777, 0x4051_f30b_7673_294c, 0xbfd7_858b_825f_5dde],
            ),
            (
                [0x3fc9_6f91_1c93_bf4b, 0x3fc2_849c_f0e6_a789, 0xbfb9_6f91_1c93_bf4b],
                [0x4003_785b_f261_136c, 0x4051_fc4d_c4eb_9ca0, 0xbfdd_e16f_c984_4db1],
            ),
            (
                [0x3fc4_5940_e8bf_b60f, 0x3fb7_7c1d_f1cf_0d65, 0xbfb4_5940_e8bf_b60f],
                [0x4004_bdf0_00ed_0ecd, 0x4052_022c_cc68_1063, 0xbfe1_7be0_01da_1d9a],
            ),
            (
                [0x3fc0_4767_24ab_383b, 0x3fa9_49a8_c49b_a688, 0xbfb0_4767_24ab_383b],
                [0x4005_c266_7337_c251, 0x4052_0556_0180_a3d8, 0xbfe3_84cc_e66f_84a1],
            ),
        ];
        for t in 1..=8 {
            if t == 4 {
                wet.in_water = true;
            }
            assert!(pearl_tick(&world, &mut wet, &[], true).is_none());
            if t >= 4 {
                let (v_exp, p_exp) = wet_expected[t - 4];
                assert_eq!([wet.vel.x.to_bits(), wet.vel.y.to_bits(), wet.vel.z.to_bits()], v_exp, "wet t{t} v");
                assert_eq!([wet.pos.x.to_bits(), wet.pos.y.to_bits(), wet.pos.z.to_bits()], p_exp, "wet t{t} pos");
            }
        }
    }

    #[test]
    fn pearl_margin_clip_modes_and_landing_witnesses() {
        // computeMargin(tickCount) (evidence.txt:191-200, check-adv-out.txt:7)
        let expected_margins = [0.0f32, 0.0, 0.0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.3, 0.3, 0.3, 0.3];
        for (t, &exp) in expected_margins.iter().enumerate() {
            assert_eq!(compute_margin(t as i32).to_bits(), exp.to_bits(), "t={t}");
        }
        assert_eq!(compute_margin(1000).to_bits(), 0.3f32.to_bits());

        // Clip-mode corridor (evidence.txt:186-189): grass(empty collider) x=2, water x=3, stone x=4
        let mut corridor = BoxWorld::default();
        corridor.fill([-4, 63, -4], [8, 63, 4], Block::full());
        corridor.set([2, 65, 0], Block { air: false, ..Block::default() });
        corridor.set(
            [3, 65, 0],
            Block {
                air: false,
                fluid: Fluid {
                    kind: FluidKind::Water,
                    raw_amount: Some(8),
                    raw_falling: Some(false),
                    ..Fluid::default()
                },
                fluid_flow: Some(FluidFlowBlock::default()),
                ..Block::default()
            },
        );
        corridor.fill([4, 64, 0], [4, 66, 0], Block::full());
        let probe = PearlState::new(Vec3::new(0.5, 65.5, 0.5), Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(
            pearl_raytrace(&corridor, &probe, &[]),
            PearlHit::Block {
                location: Vec3::new(
                    f64::from_bits(0x4010_0000_0000_0000),
                    f64::from_bits(0x4050_6000_0000_0000),
                    f64::from_bits(0x3fe0_0000_0000_0000),
                )
            }
        );

        // Entity hit wins over block hit + margin sweep
        let entity = PearlEntityTarget {
            id: 7,
            aabb: Aabb::new(1.5, 65.0, 0.0, 2.5, 66.0, 1.0),
            can_hit: true,
        };
        assert_eq!(
            pearl_raytrace(&corridor, &probe, &[entity]),
            PearlHit::Entity { id: 7, location: Vec3::new(1.5, 65.5, 0.5) }
        );

        // trace2-landing (evidence.txt:175-185): stone top face y=64, start (0.5, 65.5, 0.5), vy=-1.0
        let mut floor = BoxWorld::default();
        floor.fill([-4, 63, -4], [4, 63, 4], Block::full());
        let mut p2 = PearlState::new(Vec3::new(0.5, 65.5, 0.5), Vec3::new(0.0, -1.0, 0.0));
        assert!(pearl_tick(&floor, &mut p2, &[], true).is_none());
        assert_eq!(p2.pos.y.to_bits(), 0x4050_1ebd_3c2b_851f); // evidence.txt:177
        assert_eq!(p2.fall_distance.to_bits(), 0x0000_0000_0000_0000); // evidence.txt:178
        let landing = pearl_tick(&floor, &mut p2, &[], true).expect("lands on t2");
        assert_eq!(p2.pos.y.to_bits(), 0x4050_0000_0000_0000); // face y=64.0 (evidence.txt:180)
        assert_eq!(landing.teleport_pos.y.to_bits(), 0x4050_1ebd_3c2b_851f); // old_pos (evidence.txt:184)
        assert_ne!(landing.teleport_pos, p2.pos);
        assert!(landing.reset_fall_distance);
        assert_eq!(landing.damage, 5.0);
        assert_eq!(p2.fall_distance.to_bits(), 0);
        let version = crate::basis::cell::DATA_VERSION.to_string();
        let expected = include_str!("../tests/data/pearl_momentum.tsv").lines()
            .find_map(|line| line.split_once('\t').filter(|(id, _)| *id == version).map(|(_, bits)| bits))
            .expect("jar momentum witness for the selected DataVersion");
        let velocity = landing.owner_velocity(Vec3::new(0.25, -0.125, 0.5));
        assert_eq!([velocity.x.to_bits(), velocity.y.to_bits(), velocity.z.to_bits()].as_slice(),
            expected.split('\t').map(|field| u64::from_str_radix(field, 16).unwrap()).collect::<Vec<_>>());

        // fastDrop v=-3 (check-adv-out.txt:8-9)
        let mut fast = PearlState::new(Vec3::new(0.5, 70.5, 0.5), Vec3::new(0.0, -3.0, 0.0));
        assert!(pearl_tick(&floor, &mut fast, &[], true).is_none());
        assert!(pearl_tick(&floor, &mut fast, &[], true).is_none());
        let fast_land = pearl_tick(&floor, &mut fast, &[], true).expect("lands on t3");
        assert_eq!(fast.pos.y, 64.0);
        assert_eq!(fast_land.teleport_pos, fast.old_pos);
        assert_eq!((fast_land.teleport_pos.y - fast.pos.y).to_bits(), 0x3fe0_0758_f6d2_ed80); // 0x1.00758f6d2ed8p-1, check-adv-out.txt:9

        // wallHit v=+5 (check-adv-out.txt:10-11)
        let mut wall = PearlState::new(Vec3::new(0.5, 65.5, 0.5), Vec3::new(5.0, 0.0, 0.0));
        let wall_land = pearl_tick(&corridor, &mut wall, &[], true).expect("hits wall on t1");
        assert_eq!(wall.pos.x, 4.0);
        assert_eq!(wall.pos.y.to_bits(), 0x4050_5ea7_ef9d_b22d); // 0x1.05ea7ef9db22dp6
        assert_eq!(wall_land.teleport_pos, Vec3::new(0.5, 65.5, 0.5));
        assert_eq!((wall_land.teleport_pos.y - wall.pos.y).to_bits(), 0x3f95_8106_24dd_3000); // 0x1.5810624dd3p-6
    }
}
