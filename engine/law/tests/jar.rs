use law::{
    Aabb, 
    Vec3, 
    VoxelShape, 
    shapes::collide_with_shapes
};

#[test]
fn jar_collision_corpus(){
    for (line, row) in include_str!("data/shapes.tsv").lines().enumerate(){
        let v:Vec<_>=row.split_whitespace().map(|s|f64::from_bits(u64::from_str_radix(s, 16).unwrap())).collect();
        assert_eq!(v.len(), 31);
        let body=Aabb::new(v[0], v[1], v[2], v[3], v[4], v[5]);
        let boxes:Vec<_>=v[9..27].chunks(6).map(|a|Aabb::new(a[0], a[1], a[2], a[3], a[4], a[5])).collect();
        assert_eq!(VoxelShape::boxes(&boxes).intersects(body), v[30]==1.0, "jar overlap case {line}");
        let actual=collide_with_shapes(Vec3::new(v[6], v[7], v[8]), body, &[VoxelShape::boxes(&boxes)]);
        for (a, e) in [actual.x, actual.y, actual.z].into_iter().zip(&v[27..30]){
            assert_eq!(a.to_bits(), e.to_bits(), "jar shape case {line}: {actual:?}");
        }
    }
}

#[test]
fn jar_intersection_corpus() {
    for (line, row) in include_str!("data/intersections.tsv").lines().enumerate() {
        let (values, visits)=row.split_once('|').unwrap();
        let values: Vec<_>=values.split_whitespace().map(|v|f64::from_bits(u64::from_str_radix(v, 16).unwrap())).collect();
        assert_eq!(values.len(), 12);
        let expected: Vec<[i32; 4]>=visits.split(';').filter(|s|!s.is_empty()).map(|s| {
            let p:Vec<i32>=s.split(',').map(|v|v.parse().unwrap()).collect();
            p.try_into().unwrap()
        }).collect();
        let mut actual=Vec::new();
        law::blocks::for_each_intersected(Vec3::new(values[0], values[1], values[2]), Vec3::new(values[3], values[4], values[5]), 
        Aabb::new(values[6], values[7], values[8], values[9], values[10], values[11]), |p, step| {
            actual.push([p[0], p[1], p[2], step]);
            true
        });
        assert_eq!(actual, expected, "jar block intersection case {line}");
    }
}

#[test]
fn jar_epsilon_boundaries() {
    for row in include_str!("data/epsilons.tsv").lines() {
        let fields: Vec<_> = row.split_whitespace().collect();
        let value = f64::from_bits(u64::from_str_radix(fields[0], 16).unwrap());
        assert_eq!(law::math::equal(0.0, value), fields[1] == "true", "equal {value:?}");
        let expected = u64::from_str_radix(fields[2], 16).unwrap();
        assert_eq!(Vec3::new(value, 0.0, 0.0).normalize().x.to_bits(), expected, "normalize {value:?}");
    }
}

#[test]
fn jar_ray_traversal_and_clipping() {
    use law::{Block, BoxWorld, Fluid, FluidKind, World};
    for (line, row) in include_str!("data/rays.tsv").lines().enumerate() {
        let (values, visits) = row.split_once('|').unwrap();
        let fields: Vec<_> = values.split_whitespace().collect();
        let coordinates: Vec<_> = fields[..6].iter().map(|v| f64::from_bits(u64::from_str_radix(v, 16).unwrap())).collect();
        let from = Vec3::new(coordinates[0], coordinates[1], coordinates[2]);
        let to = Vec3::new(coordinates[3], coordinates[4], coordinates[5]);
        let target = [fields[6].parse().unwrap(), fields[7].parse().unwrap(), fields[8].parse().unwrap()];
        let height = f32::from_bits(u32::from_str_radix(fields[9], 16).unwrap());
        let expected: Vec<[i32; 3]> = visits.split(';').filter(|s| !s.is_empty()).map(|s| {
            s.split(',').map(|v| v.parse().unwrap()).collect::<Vec<_>>().try_into().unwrap()
        }).collect();
        let mut actual = Vec::new();
        law::blocks::traverse_blocks(from, to, |p| { actual.push(p); false });
        assert_eq!(actual, expected, "jar ray visits {line}");
        let mut world = BoxWorld::default();
        world.set(target, Block {
            air: false, fluid: Fluid { kind: FluidKind::Water, height, ..Fluid::default() }, ..Block::default()
        });
        assert_eq!(world.fall_reset_clip(from, to), fields[10] == "true", "jar ray hit {line}");
    }
}

#[test]
fn jar_landing_hooks() {
    use law::{Block, BoxWorld, Input, PlayerState};
    for row in include_str!("data/landings.tsv").lines() {
        let fields: Vec<_> = row.split_whitespace().collect();
        let distance: f64 = fields[1].parse().unwrap();
        let effective_distance = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
        let multiplier = f32::from_bits(u32::from_str_radix(fields[3], 16).unwrap());
        let mut block = Block::full();
        block.bed = fields[0] == "BedBlock";
        block.fall_on_multiplier = if block.bed { 1.0 } else { 0.2 };
        let mut world = BoxWorld::default();
        world.fill([-1, -1, -1], [1, -1, 1], block);
        let mut state = PlayerState {
            on_ground: true,
            fall_distance: distance,
            velocity: Vec3::new(0.0, -0.08*0.98f32 as f64, 0.0),
            health: 100.0,
            ..PlayerState::default()
        };
        assert_eq!(effective_distance, if fields[0] == "BedBlock" { distance*0.5 } else { distance });
        assert_eq!(multiplier, if fields[0] == "BedBlock" { 1.0 } else { 0.2 });
        let expected: i32 = fields[4].parse::<i32>().unwrap().max(0);
        law::tick(&mut state, &world, &Input::default());
        assert_eq!(100.0-state.health, expected as f32, "{row}");
    }
}
#[test]
fn jar_movement_speed_attribute_order() {
    for (line, row) in include_str!("data/attributes.tsv").lines().enumerate() {
        let (header, modifiers) = row.split_once('|').unwrap();
        let header: Vec<_> = header.split_whitespace().collect();
        let from_hex = |s| f64::from_bits(u64::from_str_radix(s, 16).unwrap());
        let mut state = law::PlayerState {
            movement_speed_base: from_hex(header[0]),
            sprinting: header[1] == "true",
            sprint_speed_modifier: header[1] == "true",
            ..law::PlayerState::default()
        };
        state.speed_modifiers = modifiers.split(';').filter(|s| !s.is_empty()).map(|entry| {
            let fields: Vec<_> = entry.split_whitespace().collect();
            law::SpeedModifier {
                id: fields[0].to_owned(),
                amount: from_hex(fields[1]),
                operation: match fields[2] {
                    "0" => law::SpeedOperation::AddValue,
                    "1" => law::SpeedOperation::AddMultipliedBase,
                    "2" => law::SpeedOperation::AddMultipliedTotal,
                    _ => panic!("attribute operation"),
                },
            }
        }).collect();
        assert_eq!(state.movement_speed_value().to_bits(), from_hex(header[2]).to_bits(),
            "jar attribute case {line}");
    }
}

#[test]
fn jar_falling_block_flight() {
    use law::falling;
    for (line, row) in include_str!("data/falling.tsv").lines().enumerate() {
        let fields: Vec<_> = row.split_whitespace().collect();
        let tick: u32 = fields[1].parse().unwrap();
        let time: i32 = fields[2].parse().unwrap();
        let _y_bits = u64::from_str_radix(fields[3], 16).unwrap();
        let _vy_bits = u64::from_str_radix(fields[4], 16).unwrap();
        assert_eq!(tick as i32, time, "jar falling case {line}: time++ per tick");
    }
    let spawn = falling::spawn(1000, 1000, 1000);
    assert_eq!(spawn, (1000.5, 1000.0, 1000.5));
    for _block in ["SAND", "ANVIL"] {
        let (mut y, mut vy) = (1000.0, 0.0);
        for (line, row) in include_str!("data/falling.tsv").lines().enumerate() {
            let fields: Vec<_> = row.split_whitespace().collect();
            if fields[0] != _block { continue; }
            let tick: i32 = fields[2].parse().unwrap();
            let expected_y = f64::from_bits(u64::from_str_radix(fields[3], 16).unwrap());
            let expected_vy = f64::from_bits(u64::from_str_radix(fields[4], 16).unwrap());
            (y, vy) = falling::flight_tick(y, vy);
            assert_eq!(y.to_bits(), expected_y.to_bits(), "jar falling y {_block} case {line}");
            assert_eq!(vy.to_bits(), expected_vy.to_bits(), "jar falling vy {_block} case {line}");
            assert!(!falling::discards(tick, 1000, -64, 320), "jar falling discard {_block} case {line}");
        }
    }
    assert_eq!((-falling::GRAVITY * falling::AIR_DRAG).to_bits(), 0xbfa41205c28f5c29);
    assert_ne!((-falling::GRAVITY * 0.98f64).to_bits(), 0xbfa41205c28f5c29);
    assert!(falling::discards(101, -65, -64, 320));
    assert!(falling::discards(101, 321, -64, 320));
    assert!(falling::discards(601, 100, -64, 320));
    assert!(!falling::discards(600, 100, -64, 320));
    assert!(!falling::discards(101, 100, -64, 320));
    assert!(!falling::discards(100, -65, -64, 320));
    assert_eq!(falling::fall_distance_cells(0.5), 0);
    assert_eq!(falling::fall_distance_cells(2.0), 1);
    assert_eq!(falling::fall_distance_cells(2.5), 2);
    assert_eq!(falling::stalactite_hurt(100, 97), (6.0, 40));
    assert_eq!(falling::fall_damage(5, falling::ANVIL_PER_DISTANCE, falling::ANVIL_MAX_DAMAGE), 10.0);
    assert_eq!(falling::fall_damage(100, falling::ANVIL_PER_DISTANCE, falling::ANVIL_MAX_DAMAGE), 40.0);
    assert_eq!(falling::stalactite_hurt(100, 90), (11.0, 40));
    assert!(falling::anvil_degrades(3, 0.19));
    assert!(!falling::anvil_degrades(3, 0.21));
}

#[test]
fn jar_combat_and_explosion_witness() {
    use law::{combat, explosion};
    let h64 = |s: &str| u64::from_str_radix(s, 16).unwrap();
    let d64 = |s: &str| f64::from_bits(h64(s));
    let h32 = |s: &str| u32::from_str_radix(s, 16).unwrap();
    let f32b = |s: &str| f32::from_bits(h32(s));

    for (line, row) in include_str!("data/combat_explosion.tsv").lines().enumerate() {
        let p: Vec<_> = row.split_whitespace().collect();
        match p[0] {
            "KB" => {
                let on_ground = p[2] == "1";
                let kb_res = d64(p[3]);
                let vel = Vec3::new(d64(p[4]), d64(p[5]), d64(p[6]));
                let strength = d64(p[7]);
                let (x, z) = (d64(p[8]), d64(p[9]));
                let expected = Vec3::new(d64(p[10]), d64(p[11]), d64(p[12]));
                let actual = combat::living_knockback(vel, on_ground, strength, x, z, kb_res, || 0.5)
                    .unwrap_or(vel);
                assert_eq!(actual.x.to_bits(), expected.x.to_bits(), "KB x line {line}");
                assert_eq!(actual.y.to_bits(), expected.y.to_bits(), "KB y line {line}");
                assert_eq!(actual.z.to_bits(), expected.z.to_bits(), "KB z line {line}");
            }
            "EXTRA_LIVING" | "EXTRA_BOAT" => {
                let is_living = p[0] == "EXTRA_LIVING";
                let yaw = f32b(p[1]);
                let extra = f32b(p[2]);
                let tgt_vel = if is_living {
                    Vec3::new(0.05, -0.0784, -0.05)
                } else {
                    Vec3::new(0.1, 0.0, -0.1)
                };
                let res = combat::cause_extra_knockback(
                    Vec3::new(0.2, 0.0, 0.3),
                    true,
                    yaw,
                    tgt_vel,
                    true,
                    is_living,
                    0.0,
                    false,
                    extra,
                    || 0.5,
                );
                assert_eq!(res.attacker_sprinting, p[3] == "1", "{row}");
                assert_eq!(res.attacker_velocity.x.to_bits(), h64(p[4]), "{row}");
                assert_eq!(res.attacker_velocity.y.to_bits(), h64(p[5]), "{row}");
                assert_eq!(res.attacker_velocity.z.to_bits(), h64(p[6]), "{row}");
                assert_eq!(res.target_packet_velocity.x.to_bits(), h64(p[7]), "{row}");
                assert_eq!(res.target_packet_velocity.y.to_bits(), h64(p[8]), "{row}");
                assert_eq!(res.target_packet_velocity.z.to_bits(), h64(p[9]), "{row}");
            }
            "EXTRA_ZERO" => {
                let res = combat::cause_extra_knockback(
                    Vec3::new(0.2, 0.0, 0.3),
                    true,
                    f32b(p[1]),
                    Vec3::new(0.05, 0.0, 0.0),
                    true,
                    true,
                    0.0,
                    false,
                    0.0,
                    || 0.5,
                );
                assert_eq!(res.attacker_sprinting, p[2] == "1", "{row}");
                assert_eq!(res.attacker_velocity.x.to_bits(), h64(p[3]), "{row}");
                assert_eq!(res.attacker_velocity.y.to_bits(), h64(p[4]), "{row}");
                assert_eq!(res.attacker_velocity.z.to_bits(), h64(p[5]), "{row}");
                assert_eq!(res.target_packet_velocity.x.to_bits(), h64(p[6]), "{row}");
                assert_eq!(res.target_packet_velocity.y.to_bits(), h64(p[7]), "{row}");
                assert_eq!(res.target_packet_velocity.z.to_bits(), h64(p[8]), "{row}");
            }
            "ATK_SCALE" => {
                let t: i32 = p[1].parse().unwrap();
                let s = combat::attack_strength_scale(t, 0.5, 4.0);
                let dmg_factor = combat::base_damage_scale_factor(s);
                assert_eq!(s.to_bits(), h32(p[2]), "ATK_SCALE {t}");
                assert_eq!(dmg_factor.to_bits(), h32(p[3]), "ATK_DMG_FACTOR {t}");
            }
            "WC_SHOOT" => {
                let jumping = p[1] == "1";
                let known = Vec3::new(0.1, if jumping { 0.42 } else { -0.0784 }, -0.05);
                let player_pos = Vec3::new(0.5, 64.0, 0.5);
                let eye_y = 64.0 + 1.62f32 as f64;
                let init_pos = explosion::wind_charge_spawn_pos(player_pos, eye_y);
                let init_vel = explosion::shoot_from_rotation(
                    90.0, 0.0, 0.0, explosion::WIND_CHARGE_SHOOT_POWER, Vec3::default(), known, !jumping,
                );
                let wc_box = explosion::wind_charge_aabb(init_pos);
                let after = explosion::wind_charge_free_tick(init_pos, init_vel, 5, 319);
                assert_eq!(init_pos.x.to_bits(), h64(p[2]), "wc init x");
                assert_eq!(init_pos.y.to_bits(), h64(p[3]), "wc init y");
                assert_eq!(init_pos.z.to_bits(), h64(p[4]), "wc init z");
                assert_eq!(init_vel.x.to_bits(), h64(p[5]), "wc vel x");
                assert_eq!(init_vel.y.to_bits(), h64(p[6]), "wc vel y");
                assert_eq!(init_vel.z.to_bits(), h64(p[7]), "wc vel z");
                assert_eq!(wc_box.min.y.to_bits(), h64(p[8]), "wc box minY");
                assert_eq!(wc_box.max.y.to_bits(), h64(p[9]), "wc box maxY");
                assert_eq!(after.position.y.to_bits(), h64(p[10]), "wc tick1 y");
                assert_eq!(after.velocity.x.to_bits(), h64(p[11]), "wc tick1 vx");
                assert_eq!(after.velocity.y.to_bits(), h64(p[12]), "wc tick1 vy");
                assert_eq!(after.velocity.z.to_bits(), h64(p[13]), "wc tick1 vz");
            }
            "EXP_WC" => {
                let p_pos = Vec3::new(0.5, 64.0, 0.5);
                let tgt = explosion::ExplosionTarget {
                    feet_pos: p_pos,
                    eye_y: 64.0 + 1.62f32 as f64,
                    bb: Aabb::player(p_pos, 1.8),
                    is_living: true,
                    is_player: true,
                    is_spectator: false,
                    is_creative_flying: false,
                    is_flying_ability: false,
                    is_primed_tnt: false,
                    ignore_explosion: false,
                    explosion_knockback_resistance: 0.0,
                };
                let seen = explosion::seen_percent(Vec3::new(0.5, 64.25, 0.5), tgt.bb, |_, _| true);
                assert_eq!(seen.to_bits(), h32(p[1]), "EXP_WC seen");
                let hit = explosion::explosion_entity_effect(
                    explosion::ExplosionProfile::wind_charge(),
                    Vec3::new(0.5, 64.25, 0.5),
                    tgt,
                    |_, _| true,
                )
                .unwrap();
                assert_eq!(hit.damage, 0.0);
                assert_eq!(hit.knockback_impulse.x.to_bits(), h64(p[3]), "EXP_WC kb x");
                assert_eq!(hit.knockback_impulse.y.to_bits(), h64(p[4]), "EXP_WC kb y");
                assert_eq!(hit.knockback_impulse.z.to_bits(), h64(p[5]), "EXP_WC kb z");
            }
            "EXP_TNT" => {
                let p_pos = Vec3::new(0.5, 64.0, 0.5);
                let tgt = explosion::ExplosionTarget {
                    feet_pos: p_pos,
                    eye_y: 64.0 + 1.62f32 as f64,
                    bb: Aabb::player(p_pos, 1.8),
                    is_living: true,
                    is_player: true,
                    is_spectator: false,
                    is_creative_flying: false,
                    is_flying_ability: false,
                    is_primed_tnt: false,
                    ignore_explosion: false,
                    explosion_knockback_resistance: 0.0,
                };
                let hit = explosion::explosion_entity_effect(
                    explosion::ExplosionProfile::tnt(),
                    Vec3::new(2.0, 64.0, 0.5),
                    tgt,
                    |_, _| true,
                )
                .unwrap();
                assert_eq!(hit.damage.to_bits(), h32(p[2]), "EXP_TNT damage");
            }
            other => panic!("unexpected row kind: {other}"),
        }
    }
}

#[test]
fn server_unloaded_chunk_applies_gravity_not_minus_0_1() {
    use law::{BoxWorld, Input, PlayerState};
    let mut world = BoxWorld::default();
    world.unloaded_chunks.push([0, 0]);
    let mut state = PlayerState {
        position: Vec3::new(0.5, 64.0, 0.5),
        velocity: Vec3::default(),
        on_ground: false,
        ..PlayerState::default()
    };
    law::tick(&mut state, &world, &Input::default());
    let expected_vy = (-0.08f64) * (0.98f32 as f64);
    assert_eq!(state.velocity.y.to_bits(), expected_vy.to_bits());
}

#[test]
fn jar_world_hooks() {
    use law::interact::{AttachFace, BlockHitResult, BlockId, ClipFluidMode, Direction};
    use law::world_hooks::*;
    use law::{Block, BlockPos, BoxWorld, Fluid, FluidKind};
    use std::collections::HashMap;

    let h64 = |s: &str| u64::from_str_radix(s, 16).unwrap();
    let d64 = |s: &str| f64::from_bits(h64(s));
    let parse_box = |p: &[&str]| {
        Aabb::new(d64(p[0]), d64(p[1]), d64(p[2]), d64(p[3]), d64(p[4]), d64(p[5]))
    };
    let assert_box_bits = |actual: Aabb, expected: Aabb, label: &str| {
        assert_eq!(actual.min.x.to_bits(), expected.min.x.to_bits(), "{label} min.x");
        assert_eq!(actual.min.y.to_bits(), expected.min.y.to_bits(), "{label} min.y");
        assert_eq!(actual.min.z.to_bits(), expected.min.z.to_bits(), "{label} min.z");
        assert_eq!(actual.max.x.to_bits(), expected.max.x.to_bits(), "{label} max.x");
        assert_eq!(actual.max.y.to_bits(), expected.max.y.to_bits(), "{label} max.y");
        assert_eq!(actual.max.z.to_bits(), expected.max.z.to_bits(), "{label} max.z");
    };
    let parse_face = |s: &str| match s {
        "FLOOR" => AttachFace::Floor,
        "WALL" => AttachFace::Wall,
        "CEILING" => AttachFace::Ceiling,
        _ => panic!("bad face: {s}"),
    };
    let parse_dir = |s: &str| match s {
        "DOWN" => Direction::Down,
        "UP" => Direction::Up,
        "NORTH" => Direction::North,
        "SOUTH" => Direction::South,
        "WEST" => Direction::West,
        "EAST" => Direction::East,
        _ => panic!("bad dir: {s}"),
    };

    const AIR_ID: BlockId = BlockId(0);
    const STONE_ID: BlockId = BlockId(1);
    const WATER_ID: BlockId = BlockId(2);
    const LAVA_ID: BlockId = BlockId(3);
    const SLAB_ID: BlockId = BlockId(4);
    const LADDER_ID: BlockId = BlockId(5);
    const OAK_PLATE_ID: BlockId = BlockId(10);
    const STONE_PLATE_ID: BlockId = BlockId(11);
    const GOLD_PLATE_ID: BlockId = BlockId(12);
    const IRON_PLATE_ID: BlockId = BlockId(13);
    const STONE_BTN_ID: BlockId = BlockId(20);
    const OAK_BTN_ID: BlockId = BlockId(21);
    const WIRE_ID: BlockId = BlockId(30);
    const HOOK_ID: BlockId = BlockId(31);

    struct MapBucketWorld {
        cells: HashMap<BlockPos, BucketCell>,
        evap: bool,
    }
    impl MapBucketWorld {
        fn new(evap: bool) -> Self {
            Self { cells: HashMap::new(), evap }
        }
    }
    impl BucketWorld for MapBucketWorld {
        fn air_id(&self) -> BlockId { AIR_ID }
        fn water_block_id(&self) -> BlockId { WATER_ID }
        fn lava_block_id(&self) -> BlockId { LAVA_ID }
        fn cell(&self, pos: BlockPos) -> BucketCell {
            self.cells.get(&pos).copied().unwrap_or_else(|| BucketCell::air(AIR_ID))
        }
        fn set_cell(&mut self, pos: BlockPos, cell: BucketCell) {
            if cell.is_air { self.cells.remove(&pos); } else { self.cells.insert(pos, cell); }
        }
        fn water_evaporates(&self, _pos: BlockPos) -> bool { self.evap }
    }

    let block_name = |id: BlockId| -> &'static str {
        match id {
            AIR_ID => "air",
            STONE_ID => "stone",
            WATER_ID => "water",
            LAVA_ID => "lava",
            SLAB_ID => "oak_slab",
            LADDER_ID => "ladder",
            OAK_PLATE_ID => "oak_pressure_plate",
            STONE_PLATE_ID => "stone_pressure_plate",
            GOLD_PLATE_ID => "light_weighted_pressure_plate",
            IRON_PLATE_ID => "heavy_weighted_pressure_plate",
            STONE_BTN_ID => "stone_button",
            OAK_BTN_ID => "oak_button",
            WIRE_ID => "tripwire",
            HOOK_ID => "tripwire_hook",
            _ => panic!("unknown block id {:?}", id),
        }
    };
    let dir_lower = |d: Direction| match d {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    };

    let fmt_bucket_events = |evs: &[BucketEvent]| -> String {
        let mut parts = Vec::new();
        for ev in evs {
            match *ev {
                BucketEvent::DestroyBlock { pos: [x, y, z], drop } => {
                    parts.push(format!("DESTROY({x},{y},{z},{drop})"));
                }
                BucketEvent::SetBlock { pos: [x, y, z], cell, flags } => {
                    let tag = if let Some(wl) = cell.waterlogged {
                        let extra = if cell.block_id == LADDER_ID { ",fac=north" } else { "" };
                        format!("wl={wl}{extra}")
                    } else if let Some((_, lvl)) = cell.liquid_block {
                        format!("lvl={lvl}")
                    } else {
                        String::new()
                    };
                    parts.push(format!("SET({x},{y},{z},{},{flags},{tag})", block_name(cell.block_id)));
                }
                BucketEvent::ScheduleFluidTick { pos: [x, y, z], fluid, delay } => {
                    let fnm = match fluid { BucketContent::Water => "water", BucketContent::Lava => "lava", BucketContent::Empty => "empty" };
                    parts.push(format!("SCHED_FLUID({x},{y},{z},{fnm},{delay})"));
                }
                BucketEvent::PlaySound { pos: [x, y, z], sound } => {
                    let sn = match sound {
                        BucketSound::EmptyWater => "item.bucket.empty",
                        BucketSound::EmptyLava => "item.bucket.empty_lava",
                        BucketSound::FireExtinguish => "block.fire.extinguish",
                        BucketSound::PickupWater => "item.bucket.fill",
                        BucketSound::PickupLava => "item.bucket.fill_lava",
                    };
                    parts.push(format!("SOUND({x},{y},{z},{sn})"));
                }
                BucketEvent::GameEvent { pos: [x, y, z], event } => {
                    let en = match event {
                        BucketGameEvent::FluidPlace => "minecraft:fluid_place",
                        BucketGameEvent::FluidPickup => "minecraft:fluid_pickup",
                    };
                    parts.push(format!("GAME_EVENT({x},{y},{z},{en})"));
                }
                BucketEvent::EvaporateSmokeParticles { count, .. } => {
                    for _ in 0..count { parts.push("PARTICLE".to_string()); }
                }
            }
        }
        parts.join(";")
    };

    let fmt_plate_events_id = |id: BlockId, evs: &[PlateEvent]| -> String {
        let bname = block_name(id);
        evs.iter().map(|ev| match *ev {
            PlateEvent::SetSignal { pos: [x, y, z], flags, signal } => {
                let tag = if id == GOLD_PLATE_ID || id == IRON_PLATE_ID {
                    format!("sig={signal}")
                } else {
                    format!("pow={}", signal > 0)
                };
                format!("SET({x},{y},{z},{bname},{flags},{tag})")
            }
            PlateEvent::UpdateNeighborsAt { pos: [x, y, z], source_id } => {
                format!("NEIGHBOR({x},{y},{z},{})", block_name(source_id))
            }
            PlateEvent::PlaySound { pos: [x, y, z], activated } => {
                let snd = match (id, activated) {
                    (OAK_PLATE_ID, true) => "block.wooden_pressure_plate.click_on",
                    (OAK_PLATE_ID, false) => "block.wooden_pressure_plate.click_off",
                    (STONE_PLATE_ID, true) => "block.stone_pressure_plate.click_on",
                    (STONE_PLATE_ID, false) => "block.stone_pressure_plate.click_off",
                    (_, true) => "block.metal_pressure_plate.click_on",
                    (_, false) => "block.metal_pressure_plate.click_off",
                };
                format!("SOUND({x},{y},{z},{snd})")
            }
            PlateEvent::GameEvent { pos: [x, y, z], activated } => {
                format!("GAME_EVENT({x},{y},{z},{})", if activated { "block_activate" } else { "block_deactivate" })
            }
            PlateEvent::ScheduleTick { pos: [x, y, z], block_id, delay } => {
                format!("SCHED({x},{y},{z},{},{delay})", block_name(block_id))
            }
        }).collect::<Vec<_>>().join(";")
    };

    let fmt_button_events = |id: BlockId, facing: Direction, evs: &[ButtonEvent]| -> String {
        let bname = block_name(id);
        let fstr = dir_lower(facing);
        evs.iter().map(|ev| match *ev {
            ButtonEvent::SetPowered { pos: [x, y, z], flags, powered } => {
                format!("SET({x},{y},{z},{bname},{flags},pow={powered},fac={fstr})")
            }
            ButtonEvent::UpdateNeighborsAt { pos: [x, y, z], source_id } => {
                format!("NEIGHBOR({x},{y},{z},{})", block_name(source_id))
            }
            ButtonEvent::ScheduleTick { pos: [x, y, z], block_id, delay } => {
                format!("SCHED({x},{y},{z},{},{delay})", block_name(block_id))
            }
            ButtonEvent::PlaySound { pos: [x, y, z], activated } => {
                let snd = match (id == OAK_BTN_ID, activated) {
                    (true, true) => "block.wooden_button.click_on",
                    (true, false) => "block.wooden_button.click_off",
                    (false, true) => "block.stone_button.click_on",
                    (false, false) => "block.stone_button.click_off",
                };
                format!("SOUND({x},{y},{z},{snd})")
            }
            ButtonEvent::GameEvent { pos: [x, y, z], activated } => {
                format!("GAME_EVENT({x},{y},{z},{})", if activated { "block_activate" } else { "block_deactivate" })
            }
        }).collect::<Vec<_>>().join(";")
    };

    let fmt_tripwire_events = |evs: &[TripwireEvent]| -> String {
        let mut parts = Vec::new();
        for ev in evs {
            match *ev {
                TripwireEvent::SetWire { pos: [x, y, z], flags, wire } => {
                    parts.push(format!("SET({x},{y},{z},tripwire,{flags},pow={},att={},dis={})",
                        wire.powered, wire.attached, wire.disarmed));
                }
                TripwireEvent::SetHook { pos: [x, y, z], flags, hook } => {
                    parts.push(format!("SET({x},{y},{z},tripwire_hook,{flags},pow={},att={},fac={})",
                        hook.powered, hook.attached, dir_lower(hook.facing)));
                }
                TripwireEvent::ScheduleTick { pos: [x, y, z], block_id, delay } => {
                    parts.push(format!("SCHED({x},{y},{z},{},{delay})", block_name(block_id)));
                }
                TripwireEvent::UpdateNeighborsAt { pos: [x, y, z], source_id } => {
                    parts.push(format!("NEIGHBOR({x},{y},{z},{})", block_name(source_id)));
                }
                TripwireEvent::HookFeedback { pos: [x, y, z], feedback } => {
                    let (snd, ge) = match feedback {
                        TripwireHookFeedback::ClickOn => ("block.tripwire.click_on", "block_activate"),
                        TripwireHookFeedback::ClickOff => ("block.tripwire.click_off", "block_deactivate"),
                        TripwireHookFeedback::Attach => ("block.tripwire.attach", "block_attach"),
                        TripwireHookFeedback::Detach => ("block.tripwire.detach", "block_detach"),
                    };
                    parts.push(format!("SOUND({x},{y},{z},{snd})"));
                    parts.push(format!("GAME_EVENT({x},{y},{z},{ge})"));
                }
                TripwireEvent::GameEventShear { pos: [x, y, z] } => {
                    parts.push(format!("GAME_EVENT({x},{y},{z},shear)"));
                }
            }
        }
        if parts.is_empty() { "NONE".to_string() } else { parts.join(";") }
    };

    struct MapTripwireWorld {
        cells: HashMap<BlockPos, TripwireCell>,
        scheduled: Vec<(BlockPos, BlockId)>,
    }
    impl MapTripwireWorld {
        fn new() -> Self { Self { cells: HashMap::new(), scheduled: Vec::new() } }
        fn put_wire(&mut self, pos: BlockPos, powered: bool, attached: bool, disarmed: bool) {
            self.cells.insert(pos, TripwireCell::Wire(TripwireWireState {
                block_id: WIRE_ID, powered, attached, disarmed,
            }));
        }
        fn put_hook(&mut self, pos: BlockPos, facing: Direction, powered: bool, attached: bool) {
            self.cells.insert(pos, TripwireCell::Hook(TripwireHookState {
                block_id: HOOK_ID, facing, powered, attached,
            }));
        }
    }
    impl TripwireWorld for MapTripwireWorld {
        fn wire_block_id(&self) -> BlockId { WIRE_ID }
        fn hook_block_id(&self) -> BlockId { HOOK_ID }
        fn cell(&self, pos: BlockPos) -> TripwireCell {
            self.cells.get(&pos).copied().unwrap_or(TripwireCell::Empty)
        }
        fn set_cell(&mut self, pos: BlockPos, cell: TripwireCell) {
            if cell == TripwireCell::Empty { self.cells.remove(&pos); } else { self.cells.insert(pos, cell); }
        }
        fn has_scheduled_tick(&self, pos: BlockPos, block_id: BlockId) -> bool {
            self.scheduled.contains(&(pos, block_id))
        }
    }

    let filter_jar_bucket_log = |raw: &str| -> String {
        raw.split(';')
            .filter(|s| !s.is_empty() && !s.starts_with("DESTROY_AIR_NOOP(") && !s.starts_with("SET_SAME("))
            .collect::<Vec<_>>()
            .join(";")
    };
    let filter_jar_tw_log = |raw: &str| -> String {
        let kept: Vec<_> = raw.split(';')
            .filter(|s| !s.is_empty() && !s.starts_with("LEVEL_EVENT("))
            .collect();
        if kept.is_empty() { "NONE".to_string() } else { kept.join(";") }
    };

    // Mutable states tracked across sequential plate/button/tripwire trace lines:
    let mut seq_plate = PressurePlateState {
        block_id: OAK_PLATE_ID,
        kind: PressurePlateKind::Binary { mobs_only: false },
        signal: 0,
    };
    let mut seq_tw = MapTripwireWorld::new();
    let mut seq_tw_player = TriggerEntity {
        bbox: Aabb::player(Vec3::new(1.5, 64.0, 0.5), 1.8),
        is_living: true,
        is_spectator: false,
        is_ignoring_block_triggers: false,
    };

    let mut row_count = 0usize;
    for (line_idx, raw) in include_str!("data/world_hooks.tsv").lines().enumerate() {
        row_count += 1;
        let f: Vec<_> = raw.split_whitespace().collect();
        match f[0] {
            "BUCKET_CONST" => {
                assert_eq!(BucketContent::Empty.fluid_context(), ClipFluidMode::SourceOnly);
                assert_eq!(BucketContent::Water.fluid_context(), ClipFluidMode::None);
                assert_eq!(BucketContent::Lava.fluid_context(), ClipFluidMode::None);
                assert_eq!(WATER_TICK_DELAY, f[4].parse::<u32>().unwrap());
                assert_eq!(f[10], "true");
                assert_eq!(f[11], "false");
                assert_eq!(f[12], "false");
                let empty_surv = bucket_empty_success_item(BucketStack::new(BucketContent::Water, 1), false);
                assert_eq!(empty_surv, BucketStack::new(BucketContent::Empty, 1));
                let empty_cre = bucket_empty_success_item(BucketStack::new(BucketContent::Water, 1), true);
                assert_eq!(empty_cre, BucketStack::new(BucketContent::Water, 1));
                let fr1 = create_filled_result(
                    BucketStack::new(BucketContent::Water, 1),
                    BucketStack::new(BucketContent::Empty, 1),
                    false, false, true,
                );
                assert_eq!(fr1.hand_stack, BucketStack::new(BucketContent::Empty, 1));
                let fr2 = create_filled_result(
                    BucketStack::new(BucketContent::Empty, 2),
                    BucketStack::new(BucketContent::Water, 1),
                    false, false, true,
                );
                assert_eq!(fr2.hand_stack, BucketStack::new(BucketContent::Empty, 1));
                assert_eq!(fr2.added_to_inventory, Some(BucketStack::new(BucketContent::Water, 1)));
                let fr_cre = create_filled_result(
                    BucketStack::new(BucketContent::Empty, 1),
                    BucketStack::new(BucketContent::Water, 1),
                    true, true, true,
                );
                assert_eq!(fr_cre.hand_stack, BucketStack::new(BucketContent::Empty, 1));
                assert_eq!(fr_cre.added_to_inventory, None);
            }
            "BUCKET_CASE" => {
                let mut ev = Vec::new();
                match f[1] {
                    "air_above_stone" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 64, 0], BucketCell::solid(STONE_ID));
                        let hit = BlockHitResult::Hit {
                            pos: [0, 64, 0], face: Direction::Up, location: Vec3::new(0.5, 65.0, 0.5), inside: false,
                        };
                        let res = bucket_use(&mut w, BucketStack::new(BucketContent::Water, 1), hit, true, true, false, false, false, true, false, &mut ev);
                        assert_eq!(res, BucketUseResult::Success(FilledResult {
                            hand_stack: BucketStack::new(BucketContent::Empty, 1),
                            added_to_inventory: None,
                            dropped: None,
                        }));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "pickup_water_0" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::liquid_source(WATER_ID, BucketContent::Water));
                        let picked = bucket_pickup_block(&mut w, [0, 65, 0], &mut ev).unwrap();
                        assert_eq!(picked.0, BucketStack::new(BucketContent::Water, 1));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "pickup_water_3" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::liquid_flowing(WATER_ID, BucketContent::Water, 3));
                        assert!(bucket_pickup_block(&mut w, [0, 65, 0], &mut ev).is_none());
                        assert!(ev.is_empty());
                    }
                    "waterlog_slab" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 64, 0], BucketCell::solid(STONE_ID));
                        w.set_cell([0, 65, 0], BucketCell::waterloggable(SLAB_ID, false, true));
                        let hit = BlockHitResult::Hit {
                            pos: [0, 65, 0], face: Direction::Up, location: Vec3::new(0.5, 64.5, 0.5), inside: false,
                        };
                        let res = bucket_use(&mut w, BucketStack::new(BucketContent::Water, 1), hit, true, true, false, false, false, true, false, &mut ev);
                        assert!(matches!(res, BucketUseResult::Success(_)));
                        assert_eq!(w.cell([0, 65, 0]).waterlogged, Some(true));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "pickup_slab" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::waterloggable(SLAB_ID, true, true));
                        let picked = bucket_pickup_block(&mut w, [0, 65, 0], &mut ev).unwrap();
                        assert_eq!(picked.0, BucketStack::new(BucketContent::Water, 1));
                        assert_eq!(w.cell([0, 65, 0]).waterlogged, Some(false));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "waterlog_already" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::waterloggable(SLAB_ID, true, true));
                        let ok = bucket_empty_contents(&mut w, BucketContent::Water, [0, 65, 0], Some(([0, 65, 0], Direction::Up)), false, false, &mut ev);
                        assert!(ok);
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "double_slab_redirect" => {
                        let mut w = MapBucketWorld::new(false);
                        let mut dbl = BucketCell::waterloggable(SLAB_ID, false, true);
                        dbl.accepts_water_container = false;
                        w.set_cell([0, 65, 0], dbl);
                        let hit = BlockHitResult::Hit {
                            pos: [0, 65, 0], face: Direction::Up, location: Vec3::new(0.5, 66.0, 0.5), inside: false,
                        };
                        let res = bucket_use(&mut w, BucketStack::new(BucketContent::Water, 1), hit, true, true, false, false, false, true, false, &mut ev);
                        assert!(matches!(res, BucketUseResult::Success(_)));
                        assert_eq!(w.cell([0, 66, 0]).liquid_block, Some((BucketContent::Water, 0)));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "sneak_redirect" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::waterloggable(SLAB_ID, false, true));
                        let ok = bucket_empty_contents(&mut w, BucketContent::Water, [0, 65, 0], Some(([0, 65, 0], Direction::Up)), true, false, &mut ev);
                        assert!(ok);
                        assert_eq!(w.cell([0, 65, 0]).waterlogged, Some(false));
                        assert_eq!(w.cell([0, 66, 0]).liquid_block, Some((BucketContent::Water, 0)));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[5]));
                    }
                    "pickup_unsurvivable" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::waterloggable(LADDER_ID, true, false));
                        let picked = bucket_pickup_block(&mut w, [0, 65, 0], &mut ev).unwrap();
                        assert_eq!(picked.0, BucketStack::new(BucketContent::Water, 1));
                        assert!(w.cell([0, 65, 0]).is_air);
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "evaporate_water" => {
                        let mut w = MapBucketWorld::new(true);
                        let ok = bucket_empty_contents(&mut w, BucketContent::Water, [0, 65, 0], None, false, false, &mut ev);
                        assert!(ok);
                        assert!(w.cell([0, 65, 0]).is_air);
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "evaporate_lava" => {
                        let mut w = MapBucketWorld::new(true);
                        let ok = bucket_empty_contents(&mut w, BucketContent::Lava, [0, 65, 0], None, false, false, &mut ev);
                        assert!(ok);
                        assert_eq!(w.cell([0, 65, 0]).liquid_block, Some((BucketContent::Lava, 0)));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "existing_source" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::liquid_source(WATER_ID, BucketContent::Water));
                        let ok = bucket_empty_contents(&mut w, BucketContent::Water, [0, 65, 0], None, false, false, &mut ev);
                        assert!(ok);
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[3]));
                    }
                    "solid_null_hit" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::solid(STONE_ID));
                        let ok = bucket_empty_contents(&mut w, BucketContent::Water, [0, 65, 0], None, false, false, &mut ev);
                        assert!(!ok);
                        assert!(ev.is_empty());
                    }
                    "lava_on_slab" => {
                        let mut w = MapBucketWorld::new(false);
                        w.set_cell([0, 65, 0], BucketCell::waterloggable(SLAB_ID, false, true));
                        let hit = BlockHitResult::Hit {
                            pos: [0, 65, 0], face: Direction::Up, location: Vec3::new(0.5, 65.5, 0.5), inside: false,
                        };
                        let res = bucket_use(&mut w, BucketStack::new(BucketContent::Lava, 1), hit, true, true, false, false, false, true, false, &mut ev);
                        assert!(matches!(res, BucketUseResult::Success(_)));
                        assert_eq!(w.cell([0, 65, 0]).waterlogged, Some(false));
                        assert_eq!(w.cell([0, 66, 0]).liquid_block, Some((BucketContent::Lava, 0)));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[5]));
                    }
                    other => panic!("unexpected bucket case {other}"),
                }
            }
            "PLATE_BOX" => {
                let exp = parse_box(&f[2..8]);
                let actual = match f[1] {
                    "TOUCH" => PRESSURE_PLATE_TOUCH_AABB,
                    "SHAPE" => PRESSURE_PLATE_SHAPE,
                    "SHAPE_PRESSED" => PRESSURE_PLATE_SHAPE_PRESSED,
                    other => panic!("bad plate box {other}"),
                };
                assert_box_bits(actual, exp, f[1]);
            }
            "PLATE_WEIGHTED" => {
                let max_w: i32 = f[1].parse().unwrap();
                let cnt: i32 = f[2].parse().unwrap();
                let exp_sig: u8 = f[3].parse().unwrap();
                let exp_time: u32 = f[4].parse().unwrap();
                let plate = PressurePlateState {
                    block_id: if max_w == 15 { GOLD_PLATE_ID } else { IRON_PLATE_ID },
                    kind: PressurePlateKind::Weighted { max_weight: max_w },
                    signal: 0,
                };
                assert_eq!(weighted_or_binary_signal(plate.kind, cnt), exp_sig, "weighted max={max_w} cnt={cnt}");
                assert_eq!(plate.pressed_time(), exp_time);
            }
            "PLATE_TRACE" => {
                let pos = [0, 64, 0];
                let mut ev = Vec::new();
                let std_box = Aabb::new(0.2, 64.0, 0.2, 0.8, 64.5, 0.8);
                let ent = |is_living, is_spectator, is_ignoring_block_triggers, bbox| TriggerEntity {
                    bbox, is_living, is_spectator, is_ignoring_block_triggers,
                };
                match f[1] {
                    "wood_press" => {
                        seq_plate = PressurePlateState { block_id: OAK_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: false }, signal: 0 };
                        let e = ent(true, false, false, Aabb::player(Vec3::new(0.5, 64.0625, 0.5), 1.8));
                        pressure_plate_entity_inside(&mut seq_plate, pos, false, &[e], &mut ev);
                        assert_eq!(seq_plate.signal, f[2].parse::<u8>().unwrap());
                        assert_eq!(fmt_plate_events_id(seq_plate.block_id, &ev), f[4]);
                    }
                    "wood_reentry" => {
                        let e = ent(true, false, false, Aabb::player(Vec3::new(0.5, 64.0625, 0.5), 1.8));
                        pressure_plate_entity_inside(&mut seq_plate, pos, false, &[e], &mut ev);
                        assert_eq!(seq_plate.signal, f[2].parse::<u8>().unwrap());
                        assert!(ev.is_empty());
                    }
                    "wood_held_tick" => {
                        let e = ent(true, false, false, Aabb::player(Vec3::new(0.5, 64.0625, 0.5), 1.8));
                        pressure_plate_tick(&mut seq_plate, pos, &[e], &mut ev);
                        assert_eq!(seq_plate.signal, f[2].parse::<u8>().unwrap());
                        assert_eq!(fmt_plate_events_id(seq_plate.block_id, &ev), f[3]);
                    }
                    "wood_release_tick" => {
                        pressure_plate_tick(&mut seq_plate, pos, &[], &mut ev);
                        assert_eq!(seq_plate.signal, f[2].parse::<u8>().unwrap());
                        assert_eq!(fmt_plate_events_id(seq_plate.block_id, &ev), f[3]);
                    }
                    "wood_unpowered_tick" => {
                        pressure_plate_tick(&mut seq_plate, pos, &[], &mut ev);
                        assert_eq!(seq_plate.signal, f[2].parse::<u8>().unwrap());
                        assert!(ev.is_empty());
                    }
                    "wood_spectator" => {
                        let mut p = PressurePlateState { block_id: OAK_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: false }, signal: 0 };
                        pressure_plate_entity_inside(&mut p, pos, false, &[ent(false, true, false, std_box)], &mut ev);
                        assert_eq!(p.signal, 0);
                        assert!(ev.is_empty());
                    }
                    "wood_ignoring" => {
                        let mut p = PressurePlateState { block_id: OAK_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: false }, signal: 0 };
                        pressure_plate_entity_inside(&mut p, pos, false, &[ent(false, false, true, std_box)], &mut ev);
                        assert_eq!(p.signal, 0);
                        assert!(ev.is_empty());
                    }
                    "wood_hover_exact_025" => {
                        let mut p = PressurePlateState { block_id: OAK_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: false }, signal: 0 };
                        pressure_plate_entity_inside(&mut p, pos, false, &[ent(false, false, false, Aabb::new(0.2, 64.25, 0.2, 0.8, 64.8, 0.8))], &mut ev);
                        assert_eq!(p.signal, 0);
                        assert!(ev.is_empty());
                    }
                    "wood_hover_0249" => {
                        let mut p = PressurePlateState { block_id: OAK_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: false }, signal: 0 };
                        pressure_plate_entity_inside(&mut p, pos, false, &[ent(false, false, false, Aabb::new(0.2, 64.249, 0.2, 0.8, 64.8, 0.8))], &mut ev);
                        assert_eq!(p.signal, 15);
                        assert_eq!(fmt_plate_events_id(p.block_id, &ev), f[3]);
                    }
                    "stone_item" => {
                        let mut p = PressurePlateState { block_id: STONE_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: true }, signal: 0 };
                        pressure_plate_entity_inside(&mut p, pos, false, &[ent(false, false, false, std_box)], &mut ev);
                        assert_eq!(p.signal, 0);
                        assert!(ev.is_empty());
                    }
                    "stone_mob" => {
                        let mut p = PressurePlateState { block_id: STONE_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: true }, signal: 0 };
                        pressure_plate_entity_inside(&mut p, pos, false, &[ent(true, false, false, Aabb::new(0.2, 64.0, 0.2, 0.8, 65.5, 0.8))], &mut ev);
                        assert_eq!(p.signal, 15);
                        assert_eq!(fmt_plate_events_id(p.block_id, &ev), f[3]);
                    }
                    "gold_3" => {
                        let mut p = PressurePlateState { block_id: GOLD_PLATE_ID, kind: PressurePlateKind::Weighted { max_weight: 15 }, signal: 0 };
                        let list = vec![ent(false, false, false, std_box); 3];
                        pressure_plate_entity_inside(&mut p, pos, false, &list, &mut ev);
                        assert_eq!(p.signal, 3);
                        assert_eq!(fmt_plate_events_id(p.block_id, &ev), f[3]);
                    }
                    "iron_11" => {
                        let mut p = PressurePlateState { block_id: IRON_PLATE_ID, kind: PressurePlateKind::Weighted { max_weight: 150 }, signal: 0 };
                        let list = vec![ent(false, false, false, std_box); 11];
                        pressure_plate_entity_inside(&mut p, pos, false, &list, &mut ev);
                        assert_eq!(p.signal, 2);
                        assert_eq!(fmt_plate_events_id(p.block_id, &ev), f[3]);
                    }
                    other => panic!("unexpected plate trace {other}"),
                }
            }
            "PLATE_SIGNALS" => {
                let p = PressurePlateState { block_id: OAK_PLATE_ID, kind: PressurePlateKind::Binary { mobs_only: false }, signal: 15 };
                assert_eq!(p.own_signal(), 15);
                let got: Vec<String> = Direction::VALUES.iter().map(|d| format!("{d:?}={}", p.direct_signal(*d)).to_ascii_uppercase()).collect();
                assert_eq!(got.join(","), f[2]);
            }
            "BUTTON_META" => {
                assert_eq!(f[1], "12");
                assert_eq!(f[2].parse::<u32>().unwrap(), BUTTON_STAY_WOOD);
                assert_eq!(f[3], "2");
                assert_eq!(f[4].parse::<u32>().unwrap(), BUTTON_STAY_STONE);
            }
            "BUTTON_SHAPE" => {
                let face = parse_face(f[1]);
                let facing = parse_dir(f[2]);
                let powered = f[3] == "true";
                let btn = ButtonState {
                    block_id: OAK_BTN_ID,
                    face,
                    facing,
                    powered,
                    ticks_to_stay_pressed: BUTTON_STAY_WOOD,
                    arrow_activatable: true,
                };
                assert_box_bits(btn.shape(), parse_box(&f[4..10]), &format!("BUTTON_SHAPE {face:?} {facing:?} {powered}"));
            }
            "BUTTON_TRACE" => {
                let pos = [0, 64, 0];
                let mut ev = Vec::new();
                let mut stone = ButtonState {
                    block_id: STONE_BTN_ID, face: AttachFace::Wall, facing: Direction::North, powered: false, ticks_to_stay_pressed: BUTTON_STAY_STONE, arrow_activatable: false,
                };
                let mut oak = ButtonState {
                    block_id: OAK_BTN_ID, face: AttachFace::Wall, facing: Direction::North, powered: false, ticks_to_stay_pressed: BUTTON_STAY_WOOD, arrow_activatable: true,
                };
                match f[1] {
                    "stone_use_unpowered" => {
                        let res = button_use_without_item(&mut stone, pos, &mut ev);
                        assert_eq!(res, ButtonUseResult::Success);
                        assert_eq!(fmt_button_events(STONE_BTN_ID, Direction::North, &ev), f[3]);
                    }
                    "stone_use_powered" => {
                        stone.powered = true;
                        let res = button_use_without_item(&mut stone, pos, &mut ev);
                        assert_eq!(res, ButtonUseResult::Consume);
                        assert!(ev.is_empty());
                    }
                    "stone_tick_unpowered" => {
                        button_tick(&mut stone, pos, &[], &mut ev);
                        assert!(ev.is_empty());
                    }
                    "stone_tick_release" => {
                        stone.powered = true;
                        button_tick(&mut stone, pos, &[], &mut ev);
                        assert!(!stone.powered);
                        assert_eq!(fmt_button_events(STONE_BTN_ID, Direction::North, &ev), f[2]);
                    }
                    "oak_arrow_press" => {
                        let arr = Aabb::new(0.4, 64.4, 0.88, 0.6, 64.6, 0.99);
                        button_entity_inside(&mut oak, pos, false, &[arr], &mut ev);
                        assert!(oak.powered);
                        assert_eq!(fmt_button_events(OAK_BTN_ID, Direction::North, &ev), f[2]);
                    }
                    "oak_arrow_hold" => {
                        oak.powered = true;
                        let arr = Aabb::new(0.4, 64.4, 0.88, 0.6, 64.6, 0.99);
                        button_check_pressed(&mut oak, pos, &[arr], &mut ev);
                        assert!(oak.powered);
                        assert_eq!(fmt_button_events(OAK_BTN_ID, Direction::North, &ev), f[2]);
                    }
                    "oak_arrow_tip_release" => {
                        oak.powered = true;
                        let tip = Aabb::new(0.4, 64.4, 0.88, 0.6, 64.6, 0.93);
                        button_check_pressed(&mut oak, pos, &[tip], &mut ev);
                        assert!(!oak.powered);
                        assert_eq!(fmt_button_events(OAK_BTN_ID, Direction::North, &ev), f[2]);
                    }
                    "oak_arrow_gone_release" => {
                        oak.powered = true;
                        button_check_pressed(&mut oak, pos, &[], &mut ev);
                        assert!(!oak.powered);
                        assert_eq!(fmt_button_events(OAK_BTN_ID, Direction::North, &ev), f[2]);
                    }
                    "oak_entity_inside_powered" => {
                        oak.powered = true;
                        let arr = Aabb::new(0.4, 64.4, 0.94, 0.6, 64.6, 0.99);
                        button_entity_inside(&mut oak, pos, false, &[arr], &mut ev);
                        assert!(ev.is_empty());
                    }
                    "stone_entity_inside_arrow" => {
                        let arr = Aabb::new(0.4, 64.4, 0.94, 0.6, 64.6, 0.99);
                        button_entity_inside(&mut stone, pos, false, &[arr], &mut ev);
                        assert!(ev.is_empty());
                    }
                    other => panic!("unexpected button trace {other}"),
                }
            }
            "BUTTON_GEOM" => {
                let face = parse_face(f[1]);
                let facing = parse_dir(f[2]);
                let mut btn = ButtonState {
                    block_id: STONE_BTN_ID, face, facing, powered: false, ticks_to_stay_pressed: BUTTON_STAY_STONE, arrow_activatable: false,
                };
                let mut ev = Vec::new();
                button_press(&mut btn, [0, 64, 0], &mut ev);
                let got_dirs: Vec<String> = Direction::VALUES.iter().map(|d| format!("{d:?}={}", btn.direct_signal(*d)).to_ascii_uppercase()).collect();
                assert_eq!(got_dirs.join(","), f[3]);
                assert_eq!(fmt_button_events(STONE_BTN_ID, facing, &ev), f[4]);
            }
            "TRIPWIRE_SHAPE" => {
                let exp = parse_box(&f[2..8]);
                let actual = match f[1] {
                    "WIRE_ATTACHED" => TRIPWIRE_SHAPE_ATTACHED,
                    "WIRE_UNATTACHED" => TRIPWIRE_SHAPE_UNATTACHED,
                    "HOOK_NORTH" => tripwire_hook_shape(Direction::North),
                    "HOOK_SOUTH" => tripwire_hook_shape(Direction::South),
                    "HOOK_WEST" => tripwire_hook_shape(Direction::West),
                    "HOOK_EAST" => tripwire_hook_shape(Direction::East),
                    other => panic!("bad tripwire shape {other}"),
                };
                assert_box_bits(actual, exp, f[1]);
            }
            "TRIPWIRE_TRACE" => {
                let hw = [0, 64, 0];
                let w1 = [1, 64, 0];
                let w2 = [2, 64, 0];
                let he = [3, 64, 0];
                let mut ev = Vec::new();
                match f[1] {
                    "ew_enter" => {
                        seq_tw = MapTripwireWorld::new();
                        seq_tw.put_hook(hw, Direction::East, false, true);
                        seq_tw.put_wire(w1, false, true, false);
                        seq_tw.put_wire(w2, false, true, false);
                        seq_tw.put_hook(he, Direction::West, false, true);
                        tripwire_entity_inside(&mut seq_tw, w1, false, seq_tw_player, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_reenter_powered" => {
                        tripwire_entity_inside(&mut seq_tw, w1, false, seq_tw_player, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_dwell_10" => {
                        // Hook tick + wire tick at +10
                        tripwire_hook_tick(&mut seq_tw, hw, &mut ev);
                        tripwire_tick(&mut seq_tw, w1, &[seq_tw_player], &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_leave_release" => {
                        seq_tw_player.bbox = Aabb::player(Vec3::new(50.5, 64.0, 50.5), 1.8);
                        tripwire_tick(&mut seq_tw, w1, &[seq_tw_player], &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                        seq_tw.scheduled.push((w1, WIRE_ID)); // delay-0 echo is now pending!
                    }
                    "ew_echo_gate_noop" => {
                        tripwire_entity_inside(&mut seq_tw, w1, false, seq_tw_player, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_echo_tick" => {
                        seq_tw.scheduled.retain(|k| *k != (w1, WIRE_ID));
                        tripwire_tick(&mut seq_tw, w1, &[seq_tw_player], &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_shears_disarm" => {
                        tripwire_player_will_destroy(&mut seq_tw, w1, false, true, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_disarmed_press" => {
                        seq_tw.scheduled.clear();
                        seq_tw_player.bbox = Aabb::player(Vec3::new(1.5, 64.0, 0.5), 1.8);
                        tripwire_entity_inside(&mut seq_tw, w1, false, seq_tw_player, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_break_armed" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook(hw, Direction::East, false, true);
                        w.put_wire(w1, false, true, false);
                        let old_w2 = TripwireWireState { block_id: WIRE_ID, powered: false, attached: true, disarmed: false };
                        w.put_hook(he, Direction::West, false, true);
                        tripwire_after_removal(&mut w, w2, old_w2, false, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_break_disarmed" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook(hw, Direction::East, false, true);
                        w.put_wire(w1, false, true, false);
                        let old_w2 = TripwireWireState { block_id: WIRE_ID, powered: false, attached: true, disarmed: true };
                        w.put_hook(he, Direction::West, false, true);
                        tripwire_after_removal(&mut w, w2, old_w2, false, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_break_piston_noop" => {
                        let mut w = MapTripwireWorld::new();
                        let old_w2 = TripwireWireState { block_id: WIRE_ID, powered: false, attached: true, disarmed: false };
                        tripwire_after_removal(&mut w, w2, old_w2, true, &mut ev);
                        assert!(ev.is_empty());
                    }
                    other => panic!("unexpected tripwire trace {other}"),
                }
            }
            "TRIPWIRE_BOUNDARY" => {
                let mut ev = Vec::new();
                let ent = |x: f64, z: f64| TriggerEntity {
                    bbox: Aabb::player(Vec3::new(x, 64.0, z), 1.8),
                    is_living: true,
                    is_spectator: false,
                    is_ignoring_block_triggers: false,
                };
                match f[1] {
                    "adj_fardist_1_from_att" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, true, true);
                        w.put_hook([1, 64, 0], Direction::West, true, true);
                        let TripwireCell::Hook(hs) = w.cell([0, 64, 0]) else { unreachable!() };
                        tripwire_hook_calculate_state(&mut w, [0, 64, 0], hs, false, true, -1, None, &mut ev);
                        assert_eq!(w.cell([0, 64, 0]), TripwireCell::Hook(TripwireHookState { block_id: HOOK_ID, facing: Direction::East, powered: false, attached: false }));
                        assert_eq!(w.cell([1, 64, 0]), TripwireCell::Hook(TripwireHookState { block_id: HOOK_ID, facing: Direction::West, powered: false, attached: false }));
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "adj_fardist_1_from_unatt" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, false, false);
                        w.put_hook([1, 64, 0], Direction::West, false, false);
                        let TripwireCell::Hook(hs) = w.cell([0, 64, 0]) else { unreachable!() };
                        tripwire_hook_calculate_state(&mut w, [0, 64, 0], hs, false, true, -1, None, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "span40_wire40_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, false, true);
                        for i in 1..=40 { w.put_wire([i, 64, 0], false, true, false); }
                        w.put_hook([41, 64, 0], Direction::West, false, true);
                        tripwire_entity_inside(&mut w, [40, 64, 0], false, ent(40.5, 0.5), &mut ev);
                        assert_eq!(w.cell([0, 64, 0]), TripwireCell::Hook(TripwireHookState { block_id: HOOK_ID, facing: Direction::East, powered: true, attached: true }));
                        assert_eq!(w.cell([41, 64, 0]), TripwireCell::Hook(TripwireHookState { block_id: HOOK_ID, facing: Direction::West, powered: true, attached: true }));
                        let set_sched: Vec<_> = ev.iter().copied().filter(|e| matches!(e, TripwireEvent::SetWire { .. } | TripwireEvent::SetHook { .. } | TripwireEvent::ScheduleTick { .. })).collect();
                        assert_eq!(fmt_tripwire_events(&set_sched), f[3]);
                    }
                    "span41_wire41_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, false, true);
                        for i in 1..=41 { w.put_wire([i, 64, 0], false, true, false); }
                        w.put_hook([42, 64, 0], Direction::West, false, true);
                        tripwire_entity_inside(&mut w, [41, 64, 0], false, ent(41.5, 0.5), &mut ev);
                        assert_eq!(w.cell([0, 64, 0]), TripwireCell::Hook(TripwireHookState { block_id: HOOK_ID, facing: Direction::East, powered: false, attached: false }));
                        assert_eq!(w.cell([42, 64, 0]), TripwireCell::Hook(TripwireHookState { block_id: HOOK_ID, facing: Direction::West, powered: false, attached: true }));
                        assert_eq!(w.cell([1, 64, 0]), TripwireCell::Wire(TripwireWireState { block_id: WIRE_ID, powered: false, attached: true, disarmed: false }));
                        let non_nbr: Vec<_> = ev.iter().copied().filter(|e| !matches!(e, TripwireEvent::UpdateNeighborsAt { .. })).collect();
                        assert_eq!(fmt_tripwire_events(&non_nbr), f[3]);
                    }
                    "span42_wire42_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, false, true);
                        for i in 1..=42 { w.put_wire([i, 64, 0], false, true, false); }
                        w.put_hook([43, 64, 0], Direction::West, false, true);
                        tripwire_entity_inside(&mut w, [42, 64, 0], false, ent(42.5, 0.5), &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "ns_clean_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::South, false, true);
                        w.put_wire([0, 64, 1], false, true, false);
                        w.put_wire([0, 64, 2], false, true, false);
                        w.put_hook([0, 64, 3], Direction::North, false, true);
                        tripwire_entity_inside(&mut w, [0, 64, 1], false, ent(0.5, 1.5), &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "ns_gap_south_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::South, false, true);
                        w.put_wire([0, 64, 1], false, true, false);
                        // [0, 64, 2] left Empty/Stone (non-wire gap)
                        w.put_wire([0, 64, 3], false, true, false);
                        w.put_hook([0, 64, 4], Direction::North, false, true);
                        tripwire_entity_inside(&mut w, [0, 64, 3], false, ent(0.5, 3.5), &mut ev);
                        assert_eq!(w.cell([0, 64, 2]), TripwireCell::Empty);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "ns_gap_north_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::South, false, true);
                        w.put_wire([0, 64, 1], false, true, false);
                        w.put_wire([0, 64, 3], false, true, false);
                        w.put_hook([0, 64, 4], Direction::North, false, true);
                        tripwire_entity_inside(&mut w, [0, 64, 1], false, ent(0.5, 1.5), &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "ns_gap_hook_calc" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::South, true, true);
                        w.put_wire([0, 64, 1], true, true, false);
                        w.put_wire([0, 64, 3], false, true, false);
                        w.put_hook([0, 64, 4], Direction::North, true, true);
                        let TripwireCell::Hook(hs) = w.cell([0, 64, 0]) else { unreachable!() };
                        tripwire_hook_calculate_state(&mut w, [0, 64, 0], hs, false, true, -1, None, &mut ev);
                        assert_eq!(w.cell([0, 64, 2]), TripwireCell::Empty);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    other => panic!("unexpected tripwire boundary {other}"),
                }
            }
            "TRIPWIRE_HOOK_SIGNAL" => {
                let h = TripwireHookState { block_id: HOOK_ID, facing: Direction::East, powered: true, attached: true };
                assert_eq!(h.own_signal(), 15);
                let got: Vec<String> = Direction::VALUES.iter().map(|d| format!("{d:?}={}", h.direct_signal(*d)).to_ascii_uppercase()).collect();
                assert_eq!(got.join(","), f[1]);
            }
            "BUBBLE" => {
                let above_kind = f[1];
                let open_top = f[2] == "true";
                let drag_down = f[3] == "true";
                let intersects = f[4] == "true";
                let vy_in = d64(f[5]);
                let exp_vy_out = h64(f[6]);
                let fall_in = d64(f[7]);
                let exp_fall_out = h64(f[8]);

                let step = bubble_column_entity_inside(vy_in, drag_down, open_top, intersects);
                assert_eq!(step.vy.to_bits(), exp_vy_out, "bubble vy line {line_idx}: {raw}");
                let actual_fall = if step.reset_fall_distance { 0.0 } else { fall_in };
                assert_eq!(actual_fall.to_bits(), exp_fall_out, "bubble fall line {line_idx}: {raw}");

                // Also verify World-backed above probe matches AIR / TORCH / STONE / WATER / BUBBLE_COLUMN:
                let mut bw = BoxWorld::default();
                let above_block = match above_kind {
                    "AIR" => Block::default(),
                    "TORCH" => Block { air: false, ..Block::default() },
                    "STONE" => Block::full(),
                    "WATER" | "BUBBLE_COLUMN" => Block {
                        air: false,
                        fluid: Fluid { kind: FluidKind::Water, raw_amount: Some(8), raw_falling: Some(false), ..Fluid::default() },
                        ..Block::default()
                    },
                    other => panic!("unknown above kind {other}"),
                };
                bw.set([0, 65, 0], above_block);
                let wstep = bubble_column_world_step(&bw, [0, 64, 0], vy_in, drag_down, intersects);
                assert_eq!(wstep.vy.to_bits(), exp_vy_out, "bubble world vy line {line_idx}");
                assert_eq!(wstep.reset_fall_distance, step.reset_fall_distance);
            }
            "BUBBLE_CLIMB" => {
                let mut vy = 0.0;
                for (t, hex) in f[1..].iter().enumerate() {
                    let step = bubble_column_entity_inside(vy, false, false, true);
                    vy = step.vy;
                    assert!(step.reset_fall_distance);
                    assert_eq!(vy.to_bits(), h64(hex), "climb step {t}");
                }
            }
            other => panic!("unexpected world_hooks row kind: {other}"),
        }
    }
    assert_eq!(row_count, 294, "expected all 294 jar witness rows executed");
}
