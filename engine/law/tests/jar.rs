use law::{Aabb, Vec3, VoxelShape, basis::sweep::shapes as collide_with_shapes};

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
        law::basis::ray::cells_along(Vec3::new(values[0], values[1], values[2]), Vec3::new(values[3], values[4], values[5]), 
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
        law::basis::ray::cells(from, to, |p| { actual.push(p); false });
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
        let mut state = law::PlayerState { sprinting: header[1] == "true",
            sprint_speed_modifier: header[1] == "true", profile: law::Profile { movement_speed_base: from_hex(header[0]), ..(*(law::PlayerState::default()).profile).clone() }.into(), ..law::PlayerState::default() };
        state.profile.speed_modifiers = modifiers.split(';').filter(|s| !s.is_empty()).map(|entry| {
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
    assert_eq!((-law::basis::body::FALLING_BLOCK.gravity * law::basis::body::FALLING_BLOCK.air_drag as f64).to_bits(), 0xbfa41205c28f5c29);
    assert_ne!((-law::basis::body::FALLING_BLOCK.gravity * 0.98f64).to_bits(), 0xbfa41205c28f5c29);
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
    use law::basis::cell::{CellState, StateId, WorldMut, DIRECTION, FACE};
    use law::interact::{AttachFace, BlockHitResult, BlockId, ClipFluidMode, Direction};
    use law::{sim::SimWorld, world_hooks::*};
    use law::{BlockPos, BoxWorld, FluidKind, PlayerState};
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

    let block_name = BlockId::name;
    let dir_lower = |d: Direction| match d {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    };

    let fmt_bucket_events = |evs: &[CellEvent]| -> String {
        let mut parts = Vec::new();
        for ev in evs {
            match *ev {
                CellEvent::DestroyBlock { pos: [x, y, z], drop } => {
                    parts.push(format!("DESTROY({x},{y},{z},{drop})"));
                }
                CellEvent::SetBlock { pos: [x, y, z], state: cell, flags } => {
                    let name = cell.block().name();
                    let tag = if let Some(wl) = cell.get("waterlogged") {
                        let facing = if name == "ladder" { format!(",fac={}", cell.get("facing").unwrap()) } else { String::new() };
                        format!("wl={wl}{facing}")
                    } else if let Some(level) = cell.get("level") {
                        format!("lvl={level}")
                    } else {
                        String::new()
                    };
                    parts.push(format!("SET({x},{y},{z},{name},{flags},{tag})"));
                }
                CellEvent::ScheduleFluidTick { pos: [x, y, z], fluid, delay } => {
                    let fnm = match fluid { BucketContent::Water => "water", BucketContent::Lava => "lava", BucketContent::Empty => "empty" };
                    parts.push(format!("SCHED_FLUID({x},{y},{z},{fnm},{delay})"));
                }
                CellEvent::PlaySound { pos: [x, y, z], sound } => {
                    let sn = match sound {
                        BucketSound::EmptyWater => "item.bucket.empty",
                        BucketSound::EmptyLava => "item.bucket.empty_lava",
                        BucketSound::FireExtinguish => "block.fire.extinguish",
                        BucketSound::PickupWater => "item.bucket.fill",
                        BucketSound::PickupLava => "item.bucket.fill_lava",
                    };
                    parts.push(format!("SOUND({x},{y},{z},{sn})"));
                }
                CellEvent::GameEvent { pos: [x, y, z], event } => {
                    let en = match event {
                        BucketGameEvent::FluidPlace => "minecraft:fluid_place",
                        BucketGameEvent::FluidPickup => "minecraft:fluid_pickup",
                    };
                    parts.push(format!("GAME_EVENT({x},{y},{z},{en})"));
                }
                CellEvent::EvaporateSmokeParticles { count, .. } => {
                    for _ in 0..count { parts.push("PARTICLE".to_string()); }
                }
                other => panic!("unexpected bucket event: {other:?}"),
            }
        }
        parts.join(";")
    };

    let fmt_plate_events_id = |id: BlockId, evs: &[CellEvent]| -> String {
        let bname = block_name(id);
        evs.iter().map(|ev| match *ev {
            CellEvent::SetBlock { pos: [x, y, z], flags, state } => {
                let signal = state.get("power").map_or_else(
                    || if state.get("powered") == Some("true") { 15 } else { 0 },
                    |power| power.parse::<u8>().unwrap(),
                );
                let tag = if id == BlockId::of("light_weighted_pressure_plate") || id == BlockId::of("heavy_weighted_pressure_plate") {
                    format!("sig={signal}")
                } else {
                    format!("pow={}", signal > 0)
                };
                format!("SET({x},{y},{z},{bname},{flags},{tag})")
            }
            CellEvent::UpdateNeighborsAt { pos: [x, y, z], source_id } => {
                format!("NEIGHBOR({x},{y},{z},{})", block_name(source_id))
            }
            CellEvent::SensorFeedback { pos: [x, y, z], activated } => {
                let snd = match (id.name(), activated) {
                    ("oak_pressure_plate", true) => "block.wooden_pressure_plate.click_on",
                    ("oak_pressure_plate", false) => "block.wooden_pressure_plate.click_off",
                    ("stone_pressure_plate", true) => "block.stone_pressure_plate.click_on",
                    ("stone_pressure_plate", false) => "block.stone_pressure_plate.click_off",
                    (_, true) => "block.metal_pressure_plate.click_on",
                    (_, false) => "block.metal_pressure_plate.click_off",
                };
                format!("SOUND({x},{y},{z},{snd});GAME_EVENT({x},{y},{z},{})", if activated { "block_activate" } else { "block_deactivate" })
            }
            CellEvent::ScheduleTick { pos: [x, y, z], block_id, delay } => {
                format!("SCHED({x},{y},{z},{},{delay})", block_name(block_id))
            }
            other => panic!("unexpected sensor event: {other:?}"),
        }).collect::<Vec<_>>().join(";")
    };

    let fmt_button_events = |id: BlockId, facing: Direction, evs: &[CellEvent]| -> String {
        let bname = block_name(id);
        let fstr = dir_lower(facing);
        evs.iter().map(|ev| match *ev {
            CellEvent::SetBlock { pos: [x, y, z], flags, state } => {
                format!("SET({x},{y},{z},{bname},{flags},pow={},fac={fstr})", state.get("powered").unwrap())
            }
            CellEvent::UpdateNeighborsAt { pos: [x, y, z], source_id } => {
                format!("NEIGHBOR({x},{y},{z},{})", block_name(source_id))
            }
            CellEvent::ScheduleTick { pos: [x, y, z], block_id, delay } => {
                format!("SCHED({x},{y},{z},{},{delay})", block_name(block_id))
            }
            CellEvent::SensorFeedback { pos: [x, y, z], activated } => {
                let snd = match (id == BlockId::of("oak_button"), activated) {
                    (true, true) => "block.wooden_button.click_on",
                    (true, false) => "block.wooden_button.click_off",
                    (false, true) => "block.stone_button.click_on",
                    (false, false) => "block.stone_button.click_off",
                };
                format!("SOUND({x},{y},{z},{snd});GAME_EVENT({x},{y},{z},{})", if activated { "block_activate" } else { "block_deactivate" })
            }
            other => panic!("unexpected sensor event: {other:?}"),
        }).collect::<Vec<_>>().join(";")
    };

    let fmt_tripwire_events = |evs: &[CellEvent]| -> String {
        let mut parts = Vec::new();
        for ev in evs {
            match *ev {
                CellEvent::SetBlock { pos: [x, y, z], flags, state } => {
                    let name = state.block().name();
                    let powered = state.get("powered").unwrap();
                    let attached = state.get("attached").unwrap();
                    let extra = if name == "tripwire" {
                        format!("dis={}", state.get("disarmed").unwrap())
                    } else {
                        format!("fac={}", state.get("facing").unwrap())
                    };
                    parts.push(format!("SET({x},{y},{z},{name},{flags},pow={powered},att={attached},{extra})"));
                }

                CellEvent::ScheduleTick { pos: [x, y, z], block_id, delay } => {
                    parts.push(format!("SCHED({x},{y},{z},{},{delay})", block_name(block_id)));
                }
                CellEvent::UpdateNeighborsAt { pos: [x, y, z], source_id } => {
                    parts.push(format!("NEIGHBOR({x},{y},{z},{})", block_name(source_id)));
                }
                CellEvent::HookFeedback { pos: [x, y, z], feedback } => {
                    let (snd, ge) = match feedback {
                        TripwireHookFeedback::ClickOn => ("block.tripwire.click_on", "block_activate"),
                        TripwireHookFeedback::ClickOff => ("block.tripwire.click_off", "block_deactivate"),
                        TripwireHookFeedback::Attach => ("block.tripwire.attach", "block_attach"),
                        TripwireHookFeedback::Detach => ("block.tripwire.detach", "block_detach"),
                    };
                    parts.push(format!("SOUND({x},{y},{z},{snd})"));
                    parts.push(format!("GAME_EVENT({x},{y},{z},{ge})"));
                }
                CellEvent::GameEventShear { pos: [x, y, z] } => {
                    parts.push(format!("GAME_EVENT({x},{y},{z},shear)"));
                }
                other => panic!("unexpected tripwire event: {other:?}"),
            }
        }
        if parts.is_empty() { "NONE".to_string() } else { parts.join(";") }
    };

    struct MapTripwireWorld {
        cells: HashMap<BlockPos, StateId>,
        scheduled: Vec<(BlockPos, BlockId)>,
    }
    impl MapTripwireWorld {
        fn new() -> Self { Self { cells: HashMap::new(), scheduled: Vec::new() } }
        fn put_wire(&mut self, pos: BlockPos, powered: bool, attached: bool, disarmed: bool) {
            self.cells.insert(pos, BlockId::of("tripwire").default_state().with_bool("powered", powered).with_bool("attached", attached).with_bool("disarmed", disarmed));
        }
        fn put_hook(&mut self, pos: BlockPos, facing: Direction, powered: bool, attached: bool) {
            self.cells.insert(pos, BlockId::of("tripwire_hook").default_state().with("facing", DIRECTION[facing as usize]).with_bool("powered", powered).with_bool("attached", attached));
        }
        fn cell(&self, pos: BlockPos) -> CellState { WorldMut::cell(self, pos).kind() }
    }
    impl WorldMut for MapTripwireWorld {
        fn cell(&self, pos: BlockPos) -> StateId {
            self.cells.get(&pos).copied().unwrap_or(StateId::AIR)
        }
        fn set(&mut self, pos: BlockPos, cell: StateId) {
            if cell.is_air() { self.cells.remove(&pos); } else { self.cells.insert(pos, cell); }
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
    let base = BoxWorld::default();
    let mut plates = SimWorld::new(&base);
    let pos = [0, 64, 0];
    let button_state = |block, face: AttachFace, facing: Direction, powered| {
        StateId::of(block, &[("face", FACE[face as usize]), ("facing", DIRECTION[facing as usize])])
            .with_bool("powered", powered)
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
                let base = law::BoxWorld::default();
                let mut w = law::sim::SimWorld::new(&base);
                let target = [0, 65, 0];
                let state = |name, properties: &[(&str, &str)]| StateId::of(BlockId::of(name), properties);
                match f[1] {
                    "air_above_stone" => {
                        w.set_cell_raw([0, 64, 0], state("stone", &[]));
                        let hit = BlockHitResult::Hit {
                            pos: [0, 64, 0], face: Direction::Up, location: Vec3::new(0.5, 65.0, 0.5), inside: false,
                        };
                        let (res, ev) = w.use_bucket(BucketStack::new(BucketContent::Water, 1), hit, true, true, false, false, false, true, false);
                        assert_eq!(res, BucketUseResult::Success(FilledResult {
                            hand_stack: BucketStack::new(BucketContent::Empty, 1),
                            added_to_inventory: None,
                            dropped: None,
                        }));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "pickup_water_0" => {
                        w.set_cell_raw(target, state("water", &[]));
                        let (picked, ev) = w.pickup_bucket(target);
                        let picked = picked.unwrap();
                        assert_eq!(picked.0, BucketStack::new(BucketContent::Water, 1));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "pickup_water_3" => {
                        w.set_cell_raw(target, state("water", &[("level", "3")]));
                        let (picked, ev) = w.pickup_bucket(target);
                        assert!(picked.is_none());
                        assert!(ev.is_empty());
                    }
                    "waterlog_slab" => {
                        w.set_cell_raw([0, 64, 0], state("stone", &[]));
                        w.set_cell_raw(target, state("oak_slab", &[("type", "bottom"), ("waterlogged", "false")]));
                        let hit = BlockHitResult::Hit {
                            pos: [0, 65, 0], face: Direction::Up, location: Vec3::new(0.5, 64.5, 0.5), inside: false,
                        };
                        let (res, ev) = w.use_bucket(BucketStack::new(BucketContent::Water, 1), hit, true, true, false, false, false, true, false);
                        assert!(matches!(res, BucketUseResult::Success(_)));
                        assert_eq!(w.cell(target).get("waterlogged"), Some("true"));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "pickup_slab" => {
                        w.set_cell_raw(target, state("oak_slab", &[("type", "bottom"), ("waterlogged", "true")]));
                        let (picked, ev) = w.pickup_bucket(target);
                        let picked = picked.unwrap();
                        assert_eq!(picked.0, BucketStack::new(BucketContent::Water, 1));
                        assert_eq!(w.cell(target).get("waterlogged"), Some("false"));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "waterlog_already" => {
                        w.set_cell_raw(target, state("oak_slab", &[("type", "bottom"), ("waterlogged", "true")]));
                        let (ok, ev) = w.empty_bucket(target, BucketContent::Water, Some(([0, 65, 0], Direction::Up)), false);
                        assert!(ok);
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "double_slab_redirect" => {
                        w.set_cell_raw(target, state("oak_slab", &[("type", "double")]));
                        let hit = BlockHitResult::Hit {
                            pos: [0, 65, 0], face: Direction::Up, location: Vec3::new(0.5, 66.0, 0.5), inside: false,
                        };
                        let (res, ev) = w.use_bucket(BucketStack::new(BucketContent::Water, 1), hit, true, true, false, false, false, true, false);
                        assert!(matches!(res, BucketUseResult::Success(_)));
                        assert_eq!(w.cell([0, 66, 0]), state("water", &[]));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "sneak_redirect" => {
                        w.set_cell_raw(target, state("oak_slab", &[("type", "bottom"), ("waterlogged", "false")]));
                        let (ok, ev) = w.empty_bucket(target, BucketContent::Water, Some(([0, 65, 0], Direction::Up)), true);
                        assert!(ok);
                        assert_eq!(w.cell(target).get("waterlogged"), Some("false"));
                        assert_eq!(w.cell([0, 66, 0]), state("water", &[]));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[5]));
                    }
                    "pickup_unsurvivable" => {
                        w.set_cell_raw(target, state("ladder", &[("facing", "north"), ("waterlogged", "true")]));
                        let (picked, ev) = w.pickup_bucket(target);
                        let picked = picked.unwrap();
                        assert_eq!(picked.0, BucketStack::new(BucketContent::Water, 1));
                        assert!(w.cell(target).is_air());
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "evaporate_water" => {
                        w.water_evaporates = true;
                        let (ok, ev) = w.empty_bucket(target, BucketContent::Water, None, false);
                        assert!(ok);
                        assert!(w.cell(target).is_air());
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "evaporate_lava" => {
                        w.water_evaporates = true;
                        let (ok, ev) = w.empty_bucket(target, BucketContent::Lava, None, false);
                        assert!(ok);
                        assert_eq!(w.cell(target), state("lava", &[]));
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[4]));
                    }
                    "existing_source" => {
                        w.set_cell_raw(target, state("water", &[]));
                        let (ok, ev) = w.empty_bucket(target, BucketContent::Water, None, false);
                        assert!(ok);
                        assert_eq!(fmt_bucket_events(&ev), filter_jar_bucket_log(f[3]));
                    }
                    "solid_null_hit" => {
                        w.set_cell_raw(target, state("stone", &[]));
                        let (ok, ev) = w.empty_bucket(target, BucketContent::Water, None, false);
                        assert!(!ok);
                        assert!(ev.is_empty());
                    }
                    "lava_on_slab" => {
                        w.set_cell_raw(target, state("oak_slab", &[("type", "bottom"), ("waterlogged", "false")]));
                        let hit = BlockHitResult::Hit {
                            pos: [0, 65, 0], face: Direction::Up, location: Vec3::new(0.5, 65.5, 0.5), inside: false,
                        };
                        let (res, ev) = w.use_bucket(BucketStack::new(BucketContent::Lava, 1), hit, true, true, false, false, false, true, false);
                        assert!(matches!(res, BucketUseResult::Success(_)));
                        assert_eq!(w.cell(target).get("waterlogged"), Some("false"));
                        assert_eq!(w.cell([0, 66, 0]), state("lava", &[]));
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
                let id = if f[1] == "15" { BlockId::of("light_weighted_pressure_plate") } else { BlockId::of("heavy_weighted_pressure_plate") };
                let state = id.default_state();
                let CellState::PressurePlate(plate) = state.kind() else { unreachable!() };
                assert_eq!(plate.pressed_time(), f[4].parse::<u32>().unwrap());
                let count: usize = f[2].parse().unwrap();
                let bodies = vec![TriggerEntity {
                    bbox: Aabb::new(0.2, 64.0, 0.2, 0.8, 64.5, 0.8),
                    is_living: false, is_spectator: false, is_ignoring_block_triggers: false,
                }; count];
                let mut world = SimWorld::new(&base);
                world.set_cell_raw(pos, state);
                world.sensor_input(pos, SensorInput::Contact { bodies: &bodies, arrows: &[], client: false });
                assert_eq!(world.get_signal(pos, Direction::Up, true), f[3].parse::<u8>().unwrap(), "{raw}");
            }
            "PLATE_TRACE" => {
                let body = |is_living, is_spectator, is_ignoring_block_triggers, bbox| TriggerEntity {
                    bbox, is_living, is_spectator, is_ignoring_block_triggers,
                };
                let standard = Aabb::new(0.2, 64.0, 0.2, 0.8, 64.5, 0.8);
                let player = || body(true, false, false, Aabb::player(Vec3::new(0.5, 64.0625, 0.5), 1.8));
                let (id, reset, scheduled, bodies, signal, trace) = match f[1] {
                    "wood_press" => (BlockId::of("oak_pressure_plate"), true, false, vec![player()], f[2].parse().unwrap(), f[4]),
                    "wood_reentry" => (BlockId::of("oak_pressure_plate"), false, false, vec![player()], f[2].parse().unwrap(), ""),
                    "wood_held_tick" => (BlockId::of("oak_pressure_plate"), false, true, vec![player()], f[2].parse().unwrap(), f[3]),
                    "wood_release_tick" => (BlockId::of("oak_pressure_plate"), false, true, vec![], f[2].parse().unwrap(), f[3]),
                    "wood_unpowered_tick" => (BlockId::of("oak_pressure_plate"), false, true, vec![], f[2].parse().unwrap(), ""),
                    "wood_spectator" => (BlockId::of("oak_pressure_plate"), true, false, vec![body(false, true, false, standard)], 0, ""),
                    "wood_ignoring" => (BlockId::of("oak_pressure_plate"), true, false, vec![body(false, false, true, standard)], 0, ""),
                    "wood_hover_exact_025" | "wood_hover_0249" => {
                        let inside = f[1] == "wood_hover_0249";
                        let y = if inside { 64.249 } else { 64.25 };
                        (BlockId::of("oak_pressure_plate"), true, false,
                            vec![body(false, false, false, Aabb::new(0.2, y, 0.2, 0.8, 64.8, 0.8))],
                            if inside { 15 } else { 0 }, if inside { f[3] } else { "" })
                    }
                    "stone_item" => (BlockId::of("stone_pressure_plate"), true, false, vec![body(false, false, false, standard)], 0, ""),
                    "stone_mob" => (BlockId::of("stone_pressure_plate"), true, false,
                        vec![body(true, false, false, Aabb::new(0.2, 64.0, 0.2, 0.8, 65.5, 0.8))], 15, f[3]),
                    "gold_3" => (BlockId::of("light_weighted_pressure_plate"), true, false, vec![body(false, false, false, standard); 3], 3, f[3]),
                    "iron_11" => (BlockId::of("heavy_weighted_pressure_plate"), true, false, vec![body(false, false, false, standard); 11], 2, f[3]),
                    other => panic!("unexpected plate trace {other}"),
                };
                if reset {
                    plates.set_cell_raw(pos, id.default_state());
                }
                let input = if scheduled {
                    SensorInput::Scheduled { bodies: &bodies, arrows: &[] }
                } else {
                    SensorInput::Contact { bodies: &bodies, arrows: &[], client: false }
                };
                let (_, events) = plates.sensor_input(pos, input);
                assert_eq!(plates.get_signal(pos, Direction::Up, true), signal, "{raw}");
                assert_eq!(fmt_plate_events_id(id, &events), trace, "{raw}");
            }
            "PLATE_SIGNALS" => {
                let mut world = SimWorld::new(&base);
                world.set_cell_raw(pos, BlockId::of("oak_pressure_plate").default_state().with_bool("powered", true));
                assert_eq!(world.get_signal(pos, Direction::Up, true), 15);
                let got: Vec<String> = Direction::ALL.iter().map(|d|
                    format!("{d:?}={}", world.get_direct_signal(pos, *d, true)).to_ascii_uppercase()).collect();
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
                let CellState::Button(button) = button_state(BlockId::of("oak_button"), face, facing, powered).kind() else { unreachable!() };
                assert_box_bits(button.shape(), parse_box(&f[4..10]), &format!("BUTTON_SHAPE {face:?} {facing:?} {powered}"));
            }
            "BUTTON_TRACE" => {
                let stone = BlockId::of("stone_button");
                let oak = BlockId::of("oak_button");
                let normal = Aabb::new(0.4, 64.4, 0.88, 0.6, 64.6, 0.99);
                let tip = Aabb::new(0.4, 64.4, 0.88, 0.6, 64.6, 0.93);
                let back = Aabb::new(0.4, 64.4, 0.94, 0.6, 64.6, 0.99);
                let (id, powered, active, result, arrows, trace) = match f[1] {
                    "stone_use_unpowered" => (stone, false, true, Some(ButtonUseResult::Success), None, f[3]),
                    "stone_use_powered" => (stone, true, true, Some(ButtonUseResult::Consume), None, ""),
                    "stone_tick_unpowered" => (stone, false, false, None, None, ""),
                    "stone_tick_release" => (stone, true, false, None, None, f[2]),
                    "oak_arrow_press" => (oak, false, true, None, Some(normal), f[2]),
                    "oak_arrow_hold" => (oak, true, true, None, Some(normal), f[2]),
                    "oak_arrow_tip_release" => (oak, true, false, None, Some(tip), f[2]),
                    "oak_arrow_gone_release" => (oak, true, false, None, None, f[2]),
                    "oak_entity_inside_powered" => (oak, true, true, None, Some(back), ""),
                    "stone_entity_inside_arrow" => (stone, false, false, None, Some(back), ""),
                    other => panic!("unexpected button trace {other}"),
                };
                let mut world = SimWorld::new(&base);
                world.set_cell_raw(pos, button_state(id, AttachFace::Wall, Direction::North, powered));
                let arrows = arrows.as_slice();
                let input = if result.is_some() {
                    SensorInput::Use
                } else if matches!(f[1], "oak_arrow_press" | "oak_entity_inside_powered" | "stone_entity_inside_arrow") {
                    SensorInput::Contact { bodies: &[], arrows, client: false }
                } else {
                    SensorInput::Scheduled { bodies: &[], arrows }
                };
                let (used, events) = world.sensor_input(pos, input);
                assert_eq!(used, result, "{raw}");
                assert_eq!(world.cell(pos).get("powered"), Some(if active { "true" } else { "false" }), "{raw}");
                assert_eq!(fmt_button_events(id, Direction::North, &events), trace, "{raw}");
            }
            "BUTTON_GEOM" => {
                let facing = parse_dir(f[2]);
                let mut world = SimWorld::new(&base);
                world.set_cell_raw(pos, button_state(BlockId::of("stone_button"), parse_face(f[1]), facing, false));
                let (_, events) = world.sensor_input(pos, SensorInput::Use);
                let got: Vec<String> = Direction::ALL.iter().map(|d|
                    format!("{d:?}={}", world.get_direct_signal(pos, *d, true)).to_ascii_uppercase()).collect();
                assert_eq!(got.join(","), f[3]);
                assert_eq!(fmt_button_events(BlockId::of("stone_button"), facing, &events), f[4]);
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
                        seq_tw.scheduled.push((w1, BlockId::of("tripwire"))); // delay-0 echo is now pending!
                    }
                    "ew_echo_gate_noop" => {
                        tripwire_entity_inside(&mut seq_tw, w1, false, seq_tw_player, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_echo_tick" => {
                        seq_tw.scheduled.retain(|k| *k != (w1, BlockId::of("tripwire")));
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
                        let old_w2 = TripwireWireState { block_id: BlockId::of("tripwire"), powered: false, attached: true, disarmed: false };
                        w.put_hook(he, Direction::West, false, true);
                        tripwire_after_removal(&mut w, w2, old_w2, false, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_break_disarmed" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook(hw, Direction::East, false, true);
                        w.put_wire(w1, false, true, false);
                        let old_w2 = TripwireWireState { block_id: BlockId::of("tripwire"), powered: false, attached: true, disarmed: true };
                        w.put_hook(he, Direction::West, false, true);
                        tripwire_after_removal(&mut w, w2, old_w2, false, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[2]));
                    }
                    "ew_break_piston_noop" => {
                        let mut w = MapTripwireWorld::new();
                        let old_w2 = TripwireWireState { block_id: BlockId::of("tripwire"), powered: false, attached: true, disarmed: false };
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
                        let CellState::TripwireHook(hs) = w.cell([0, 64, 0]) else { unreachable!() };
                        tripwire_hook_calculate_state(&mut w, [0, 64, 0], hs, false, true, -1, None, &mut ev);
                        assert_eq!(w.cell([0, 64, 0]), CellState::TripwireHook(TripwireHookState { block_id: BlockId::of("tripwire_hook"), facing: Direction::East, powered: false, attached: false }));
                        assert_eq!(w.cell([1, 64, 0]), CellState::TripwireHook(TripwireHookState { block_id: BlockId::of("tripwire_hook"), facing: Direction::West, powered: false, attached: false }));
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "adj_fardist_1_from_unatt" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, false, false);
                        w.put_hook([1, 64, 0], Direction::West, false, false);
                        let CellState::TripwireHook(hs) = w.cell([0, 64, 0]) else { unreachable!() };
                        tripwire_hook_calculate_state(&mut w, [0, 64, 0], hs, false, true, -1, None, &mut ev);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    "span40_wire40_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, false, true);
                        for i in 1..=40 { w.put_wire([i, 64, 0], false, true, false); }
                        w.put_hook([41, 64, 0], Direction::West, false, true);
                        tripwire_entity_inside(&mut w, [40, 64, 0], false, ent(40.5, 0.5), &mut ev);
                        assert_eq!(w.cell([0, 64, 0]), CellState::TripwireHook(TripwireHookState { block_id: BlockId::of("tripwire_hook"), facing: Direction::East, powered: true, attached: true }));
                        assert_eq!(w.cell([41, 64, 0]), CellState::TripwireHook(TripwireHookState { block_id: BlockId::of("tripwire_hook"), facing: Direction::West, powered: true, attached: true }));
                        let set_sched: Vec<_> = ev.iter().copied().filter(|e| matches!(e, CellEvent::SetBlock { .. } | CellEvent::ScheduleTick { .. })).collect();
                        assert_eq!(fmt_tripwire_events(&set_sched), f[3]);
                    }
                    "span41_wire41_enter" => {
                        let mut w = MapTripwireWorld::new();
                        w.put_hook([0, 64, 0], Direction::East, false, true);
                        for i in 1..=41 { w.put_wire([i, 64, 0], false, true, false); }
                        w.put_hook([42, 64, 0], Direction::West, false, true);
                        tripwire_entity_inside(&mut w, [41, 64, 0], false, ent(41.5, 0.5), &mut ev);
                        assert_eq!(w.cell([0, 64, 0]), CellState::TripwireHook(TripwireHookState { block_id: BlockId::of("tripwire_hook"), facing: Direction::East, powered: false, attached: false }));
                        assert_eq!(w.cell([42, 64, 0]), CellState::TripwireHook(TripwireHookState { block_id: BlockId::of("tripwire_hook"), facing: Direction::West, powered: false, attached: true }));
                        assert_eq!(w.cell([1, 64, 0]), CellState::TripwireWire(TripwireWireState { block_id: BlockId::of("tripwire"), powered: false, attached: true, disarmed: false }));
                        let non_nbr: Vec<_> = ev.iter().copied().filter(|e| !matches!(e, CellEvent::UpdateNeighborsAt { .. })).collect();
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
                        assert_eq!(w.cell([0, 64, 2]), CellState::Empty);
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
                        let CellState::TripwireHook(hs) = w.cell([0, 64, 0]) else { unreachable!() };
                        tripwire_hook_calculate_state(&mut w, [0, 64, 0], hs, false, true, -1, None, &mut ev);
                        assert_eq!(w.cell([0, 64, 2]), CellState::Empty);
                        assert_eq!(fmt_tripwire_events(&ev), filter_jar_tw_log(f[3]));
                    }
                    other => panic!("unexpected tripwire boundary {other}"),
                }
            }
            "TRIPWIRE_HOOK_SIGNAL" => {
                let h = TripwireHookState { block_id: BlockId::of("tripwire_hook"), facing: Direction::East, powered: true, attached: true };
                assert_eq!(h.own_signal(), 15);
                let got: Vec<String> = Direction::ALL.iter().map(|d| format!("{d:?}={}", h.direct_signal(*d)).to_ascii_uppercase()).collect();
                assert_eq!(got.join(","), f[1]);
            }
            "BUBBLE" => {
                use law::basis::body::bubble_column_contact;
                let block = BlockId::of(&f[1].to_ascii_lowercase()).default_state().model_block();
                let open_top = f[2] == "true";
                assert_eq!(block.shape.is_empty() && block.fluid.kind == FluidKind::Empty, open_top);
                let mut world = BoxWorld::default();
                world.set([0, 65, 0], block.clone());
                let mut state = PlayerState {
                    velocity: Vec3::new(0.0, d64(f[5]), 0.0),
                    fall_distance: d64(f[7]),
                    ..PlayerState::default()
                };
                bubble_column_contact(&mut state, &world, [0, 64, 0], &(f[3] == "true"), f[4] == "true");
                assert_eq!(state.velocity.y.to_bits(), h64(f[6]), "bubble vy line {line_idx}: {raw}");
                assert_eq!(state.fall_distance.to_bits(), h64(f[8]), "bubble fall line {line_idx}: {raw}");
            }
            "BUBBLE_CLIMB" => {
                use law::basis::body::bubble_column_contact;
                let mut world = BoxWorld::default();
                world.set([0, 65, 0], BlockId::of("water").default_state().model_block().clone());
                let mut state = PlayerState::default();
                for (t, hex) in f[1..].iter().enumerate() {
                    state.fall_distance = 1.0;
                    bubble_column_contact(&mut state, &world, [0, 64, 0], &false, true);
                    assert_eq!(state.fall_distance.to_bits(), 0.0f64.to_bits());
                    assert_eq!(state.velocity.y.to_bits(), h64(hex), "climb step {t}");
                }
            }
            other => panic!("unexpected world_hooks row kind: {other}"),
        }
    }
    assert_eq!(row_count, 294, "expected all 294 jar witness rows executed");
}

#[test]
fn jar_vehicles_and_wormholes() {
    use law::{boat, interact, pearl, portal, Block, BlockPos, BoxWorld, Fluid, FluidFlowBlock, FluidKind, World};

    let h64 = |s: &str| u64::from_str_radix(s, 16).unwrap();
    let d64 = |s: &str| f64::from_bits(h64(s));
    let h32 = |s: &str| u32::from_str_radix(s, 16).unwrap();
    let f32b = |s: &str| f32::from_bits(h32(s));

    let water_block = |amount: u8| Block {
        air: false,
        fluid: Fluid {
            kind: FluidKind::Water,
            raw_amount: Some(amount),
            raw_falling: Some(false),
            ..Fluid::default()
        },
        fluid_flow: Some(FluidFlowBlock::default()),
        ..Block::default()
    };

    // World for BOAT_CLIP (matching VehicleWormholeWitness.emitBoatPlaceAndBreak)
    let mut clip_world = BoxWorld::default();
    clip_world.set([2, 69, 2], Block::full());
    clip_world.set([2, 70, 2], water_block(8));
    clip_world.set([2, 69, 3], Block::full());
    clip_world.set([2, 70, 3], water_block(8));
    clip_world.set([2, 70, 4], Block::full());
    clip_world.set([2, 71, 4], Block::full());

    let mut fw = BoxWorld::default();
    let stone = Block {
        friction: f32::from_bits(0x3f19_999a),
        ..Block::full()
    };
    let blue_ice = Block {
        friction: f32::from_bits(0x3f7d_2f1b),
        ..Block::full()
    };
    fw.fill([90, 64, 90], [99, 64, 110], stone);
    fw.fill([100, 64, 90], [110, 64, 110], blue_ice);
    fw.set(
        [200, 65, 200],
        Block { lily_pad: true, ..Block::boxes(&[Aabb::new(0.0625, 0.0, 0.0625, 0.9375, 0.09375, 0.9375)]) },
    );

    // Sea world for BOAT_SNAP (water at y <= 61)
    struct SeaWorld {
        top_y: i32,
        water: Block,
        air: Block,
        ceiling_block: Option<Aabb>,
    }
    impl World for SeaWorld {
        fn block(&self, pos: BlockPos) -> &Block {
            if pos[1] <= self.top_y {
                &self.water
            } else {
                &self.air
            }
        }
        fn visit_collision_cells(
            &self,
            _bounds: Aabb,
            _lo: BlockPos,
            _hi: BlockPos,
            _visit: impl FnMut(BlockPos) -> bool,
        ) -> bool {
            true
        }
        fn extra_collisions(
            &self,
            bounds: Aabb,
            _ctx: law::CollisionContext,
        ) -> Vec<VoxelShape> {
            match self.ceiling_block {
                Some(b) if b.intersects(bounds) => vec![VoxelShape::boxes(&[b])],
                _ => Vec::new(),
            }
        }
        fn visit_extra_collisions(
            &self,
            bounds: Aabb,
            _ctx: law::CollisionContext,
            mut visit: impl FnMut(law::shapes::ShapeView<'_>) -> bool,
        ) -> bool {
            if let Some(b) = self.ceiling_block {
                if b.intersects(bounds) {
                    let s = VoxelShape::boxes(&[b]);
                    return visit(s.view());
                }
            }
            true
        }
    }

    let parse_boat_status = |s: &str| match s {
        "IN_WATER" => boat::BoatStatus::InWater,
        "UNDER_WATER" => boat::BoatStatus::UnderWater,
        "UNDER_FLOWING_WATER" => boat::BoatStatus::UnderFlowingWater,
        "ON_LAND" => boat::BoatStatus::OnLand,
        "IN_AIR" => boat::BoatStatus::InAir,
        other => panic!("unexpected boat status: {other}"),
    };
    let parse_axis = |s: &str| match s {
        "X" => portal::PortalAxis::X,
        "Z" => portal::PortalAxis::Z,
        other => panic!("unexpected portal axis: {other}"),
    };

    let mut surv_hurt = boat::BoatHurtState::default();
    let mut row_count = 0usize;

    for (line, row) in include_str!("data/vehicles_wormholes.tsv").lines().enumerate() {
        if row.trim().is_empty() {
            continue;
        }
        row_count += 1;
        let p: Vec<_> = row.split_whitespace().collect();
        match p[0] {
            "BOAT_DIM" => {
                assert_eq!(boat::BOAT_WIDTH.to_bits(), h32(p[1]), "line {line}");
                assert_eq!(boat::BOAT_HEIGHT.to_bits(), h32(p[2]), "line {line}");
            }
            "BOAT_CLIP" => {
                let mode = match p[1] {
                    "ANY" => interact::ClipFluidMode::Any,
                    "NONE" => interact::ClipFluidMode::None,
                    other => panic!("unexpected clip mode {other}"),
                };
                let player_pos = Vec3::new(d64(p[2]), d64(p[3]), d64(p[4]));
                let eye_h = f32b(p[5]);
                let yaw = f32b(p[6]);
                let pitch = f32b(p[7]);
                let range = d64(p[8]);
                let eye = interact::get_eye_position(player_pos, eye_h);
                let hit = boat::boat_pov_hit(&clip_world, eye, yaw, pitch, range, mode);
                assert_eq!(hit.is_hit(), p[9] == "BLOCK", "BOAT_CLIP kind line {line}");
                let loc = hit.location();
                assert_eq!(loc.x.to_bits(), h64(p[10]), "BOAT_CLIP hx line {line}");
                assert_eq!(loc.y.to_bits(), h64(p[11]), "BOAT_CLIP hy line {line}");
                assert_eq!(loc.z.to_bits(), h64(p[12]), "BOAT_CLIP hz line {line}");
                if mode == interact::ClipFluidMode::Any {
                    let player_bb = Aabb::player(player_pos, 1.8);
                    let placed = boat::place_boat(
                        &clip_world,
                        player_pos,
                        player_bb,
                        eye_h,
                        yaw,
                        pitch,
                        range,
                        false,
                        false,
                        &[],
                    );
                    match p[9] {
                        "MISS" => assert_eq!(placed, boat::BoatPlaceResult::Pass, "line {line}"),
                        "BLOCK" if p[13] == "1" => assert_eq!(
                            placed,
                            boat::BoatPlaceResult::Success {
                                pos: loc,
                                yaw,
                                aabb: boat::boat_aabb(loc),
                                spawned_server: true,
                                consumed: true,
                            },
                            "line {line}"
                        ),
                        "BLOCK" => assert_eq!(placed, boat::BoatPlaceResult::Fail, "line {line}"),
                        _ => unreachable!(),
                    }
                }
            }
            "BOAT_PICK" => {
                // Eye enclosed in inflated pickable box -> BoatItem.use returns Pass
                let eye = Vec3::new(2.5, 73.62, 2.5);
                let enclosing = Aabb::new(2.0, 72.0, 2.0, 3.0, 73.5, 3.0);
                assert_eq!(enclosing.contains(eye), p[1] == "1");
                assert_eq!(enclosing.deflate(-0.2).contains(eye), p[2] == "1");
                let player_pos = Vec3::new(2.5, 72.0, 2.5);
                let res = boat::place_boat(
                    &clip_world,
                    player_pos,
                    Aabb::player(player_pos, 1.8),
                    1.62,
                    0.0,
                    90.0,
                    20.0,
                    false,
                    false,
                    &[boat::BoatPickTarget {
                        aabb: enclosing,
                        pick_radius: 0.2,
                        is_pickable: true,
                    }],
                );
                assert_eq!(res, boat::BoatPlaceResult::Pass, "BOAT_PICK line {line}");
            }
            "BOAT_HURT" => {
                let creative = p[1] == "1";
                let amt = f32b(p[2]);
                let mut h = if creative {
                    boat::BoatHurtState::default()
                } else {
                    surv_hurt
                };
                assert_eq!(h.hurt_time, p[3].parse::<i32>().unwrap(), "line {line}");
                assert_eq!(h.damage.to_bits(), h32(p[4]), "line {line}");
                assert_eq!(h.hurt_dir, p[5].parse::<i32>().unwrap(), "line {line}");
                let (ret, outcome) = h.hurt_server(amt, creative, false, false, true);
                assert_eq!(ret, p[6] == "1", "line {line}");
                assert_eq!(h.removed, p[7] == "1", "line {line}");
                assert_eq!(
                    matches!(outcome, boat::BoatHurtOutcome::Destroyed { .. }),
                    p[8] == "1",
                    "line {line}"
                );
                assert_eq!(
                    matches!(outcome, boat::BoatHurtOutcome::Discarded),
                    p[9] == "1",
                    "line {line}"
                );
                assert_eq!(h.hurt_time, p[10].parse::<i32>().unwrap(), "line {line}");
                assert_eq!(h.damage.to_bits(), h32(p[11]), "line {line}");
                assert_eq!(h.hurt_dir, p[12].parse::<i32>().unwrap(), "line {line}");
                if !creative {
                    surv_hurt = h;
                }
            }
            "BOAT_HURT_TICK" => {
                let mut h = boat::BoatHurtState {
                    damage: f32b(p[1]),
                    hurt_time: p[2].parse().unwrap(),
                    ..boat::BoatHurtState::default()
                };
                h.tick_decay();
                assert_eq!(h.damage.to_bits(), h32(p[3]), "BOAT_HURT_TICK dmg line {line}");
                assert_eq!(h.hurt_time, p[4].parse::<i32>().unwrap(), "BOAT_HURT_TICK ht line {line}");
            }
            "BOAT_STATUS" => {
                let pos = Vec3::new(d64(p[2]), d64(p[3]), d64(p[4]));
                let fric = boat::get_ground_friction(&fw, pos);
                assert_eq!(fric.to_bits(), h32(p[5]), "BOAT_STATUS fric {} line {line}", p[1]);
                let mut body = boat::BoatBody {
                    boat: boat::Boat { position: pos, ..Default::default() },
                    water_level: -f64::MAX,
                    ..Default::default()
                };
                let st = boat::get_status(&mut body, &fw);
                assert_eq!(st, parse_boat_status(p[6]), "BOAT_STATUS st {} line {line}", p[1]);
                assert_eq!(body.water_level.to_bits(), h64(p[7]), "BOAT_STATUS wl {} line {line}", p[1]);
            }
            "BOAT_SNAP" => {
                let cid = p[1];
                let old_status = parse_boat_status(p[2]);
                let clear = p[3] == "1";
                let y_in = d64(p[4]);
                let vx_in = d64(p[5]);
                let vy_in = d64(p[6]);
                let vz_in = d64(p[7]);
                let last_yd_in = d64(p[8]);
                let wla_expected = h32(p[9]);
                let flowing = cid.contains("FLOWING");
                let sea = SeaWorld {
                    top_y: 61,
                    water: water_block(if flowing { 1 } else { 8 }),
                    air: Block::default(),
                    ceiling_block: (!clear).then(|| Aabb::new(0.0, 61.5, 0.0, 1.0, 62.0, 1.0)),
                };
                let mut body = boat::BoatBody {
                    boat: boat::Boat {
                        position: Vec3::new(0.5, y_in, 0.5),
                        velocity: Vec3::new(vx_in, vy_in, vz_in),
                        ..Default::default()
                    },
                    status: Some(old_status),
                    last_yd: last_yd_in,
                    ..Default::default()
                };
                if old_status == boat::BoatStatus::InAir {
                    let entry = boat::water_entry(
                        true,
                        false,
                        false,
                        f32::from_bits(wla_expected),
                        boat::BOAT_HEIGHT,
                        clear,
                    );
                    if clear {
                        assert_eq!(entry, Some(boat::WaterEntry::Snap(d64(p[10]))), "{cid}");
                    } else {
                        assert_eq!(entry, Some(boat::WaterEntry::Blocked), "{cid}");
                    }
                }
                boat::tick_body(&mut body, &sea, boat::Controls::default(), true, false, false);
                assert_eq!(body.boat.position.y.to_bits(), h64(p[10]), "SNAP {cid} y");
                assert_eq!(body.boat.velocity.x.to_bits(), h64(p[11]), "SNAP {cid} vx");
                assert_eq!(body.boat.velocity.y.to_bits(), h64(p[12]), "SNAP {cid} vy");
                assert_eq!(body.boat.velocity.z.to_bits(), h64(p[13]), "SNAP {cid} vz");
                assert_eq!(body.last_yd.to_bits(), h64(p[14]), "SNAP {cid} last_yd");
                assert_eq!(body.water_level.to_bits(), h64(p[15]), "SNAP {cid} water_level");
                assert_eq!(body.status, Some(parse_boat_status(p[16])), "SNAP {cid} status");
            }
            "BOAT_PADDLE" => {
                let input = boat::Controls {
                    left: p[1] == "1",
                    right: p[2] == "1",
                    forward: p[3] == "1",
                    backward: p[4] == "1",
                };
                let mut b = boat::Boat {
                    yaw: f32b(p[5]),
                    delta_rotation: f32b(p[6]),
                    velocity: Vec3::new(d64(p[7]), 0.0, d64(p[8])),
                    ..Default::default()
                };
                let (pl, pr) = boat::control_boat(&mut b, input);
                assert_eq!(b.yaw.to_bits(), h32(p[9]), "PADDLE yaw line {line}");
                assert_eq!(b.delta_rotation.to_bits(), h32(p[10]), "PADDLE dr line {line}");
                assert_eq!(b.velocity.x.to_bits(), h64(p[11]), "PADDLE vx line {line}");
                assert_eq!(b.velocity.y.to_bits(), h64(p[12]), "PADDLE vy line {line}");
                assert_eq!(b.velocity.z.to_bits(), h64(p[13]), "PADDLE vz line {line}");
                assert_eq!(pl, p[14] == "1", "PADDLE left line {line}");
                assert_eq!(pr, p[15] == "1", "PADDLE right line {line}");
            }
            "PEARL_LAUNCH" => {
                let cid = p[1];
                let pos = Vec3::new(d64(p[2]), d64(p[3]), d64(p[4]));
                let eye_h = f32b(p[5]);
                let x_rot = f32b(p[6]);
                let y_rot = f32b(p[7]);
                let km = Vec3::new(d64(p[8]), d64(p[9]), d64(p[10]));
                let on_ground = p[11] == "1";
                let draws = [
                    d64(p[12]), d64(p[13]), d64(p[14]), d64(p[15]), d64(p[16]), d64(p[17]),
                ];
                let res = pearl::use_ender_pearl(&pearl::PearlUseInput {
                    is_spectator: false,
                    cooldown_end_tick: None,
                    tick_count: 100,
                    stack_count: 5,
                    has_infinite_materials: false,
                    owner_pos: pos,
                    owner_eye_height: eye_h,
                    x_rot,
                    y_rot,
                    known_movement: km,
                    on_ground,
                    draws,
                });
                let pearl::PearlUseResult::Thrown {
                    state,
                    y_rot: py_rot,
                    x_rot: px_rot,
                    remaining_stack,
                    cooldown_end_tick,
                } = res else {
                    panic!("expected pearl thrown for {cid}");
                };
                assert_eq!(remaining_stack, 4);
                assert_eq!(cooldown_end_tick, 120);
                assert_eq!(state.pos.x.to_bits(), h64(p[18]), "{cid} sx");
                assert_eq!(state.pos.y.to_bits(), h64(p[19]), "{cid} sy");
                assert_eq!(state.pos.z.to_bits(), h64(p[20]), "{cid} sz");
                assert_eq!(state.old_pos, state.pos);
                assert_eq!(state.tick_count, 0);
                assert_eq!(state.vel.x.to_bits(), h64(p[21]), "{cid} vx");
                assert_eq!(state.vel.y.to_bits(), h64(p[22]), "{cid} vy");
                assert_eq!(state.vel.z.to_bits(), h64(p[23]), "{cid} vz");
                assert_eq!(py_rot.to_bits(), h32(p[24]), "{cid} pearl yRot");
                assert_eq!(px_rot.to_bits(), h32(p[25]), "{cid} pearl xRot");
            }
            "PEARL_COOLDOWN" => {
                let cd_ticks: i32 = p[1].parse().unwrap();
                let elapsed: i32 = p[2].parse().unwrap();
                let expected_on = p[3] == "1";
                assert_eq!(cd_ticks, pearl::PEARL_COOLDOWN_TICKS);
                let end = pearl::add_cooldown_end_tick(0, cd_ticks);
                assert_eq!(pearl::is_on_cooldown(Some(end), elapsed), expected_on, "elapsed {elapsed}");
            }
            "CONSUME" => {
                let surv_res = pearl::use_ender_pearl(&pearl::PearlUseInput {
                    is_spectator: false,
                    cooldown_end_tick: None,
                    tick_count: 0,
                    stack_count: 5,
                    has_infinite_materials: false,
                    owner_pos: Vec3::default(),
                    owner_eye_height: 1.62,
                    x_rot: 0.0,
                    y_rot: 0.0,
                    known_movement: Vec3::default(),
                    on_ground: true,
                    draws: [0.0; 6],
                });
                let crea_res = pearl::use_ender_pearl(&pearl::PearlUseInput {
                    is_spectator: false,
                    cooldown_end_tick: None,
                    tick_count: 0,
                    stack_count: 5,
                    has_infinite_materials: true,
                    owner_pos: Vec3::default(),
                    owner_eye_height: 1.62,
                    x_rot: 0.0,
                    y_rot: 0.0,
                    known_movement: Vec3::default(),
                    on_ground: true,
                    draws: [0.0; 6],
                });
                let pearl::PearlUseResult::Thrown { remaining_stack: s_rem, .. } = surv_res else { panic!() };
                let pearl::PearlUseResult::Thrown { remaining_stack: c_rem, .. } = crea_res else { panic!() };
                assert_eq!(s_rem, p[1].parse::<i32>().unwrap());
                assert_eq!(c_rem, p[2].parse::<i32>().unwrap());
            }
            "PORTAL_SCALE" => {
                let from_s: f64 = p[1].parse().unwrap();
                let to_s: f64 = p[2].parse().unwrap();
                assert_eq!(portal::teleportation_scale(from_s, to_s).to_bits(), h64(p[3]));
            }
            "PORTAL_BORDER" => {
                let pos = Vec3::new(d64(p[1]), d64(p[2]), d64(p[3]));
                let wb = portal::WorldBorderBounds {
                    min_x: d64(p[4]),
                    max_x: d64(p[5]),
                    min_z: d64(p[6]),
                    max_z: d64(p[7]),
                };
                let got = wb.clamp_vec_to_bounds(pos);
                let exp = [p[8].parse().unwrap(), p[9].parse().unwrap(), p[10].parse().unwrap()];
                assert_eq!(got, exp, "PORTAL_BORDER line {line}");
            }
            "PORTAL_TRANSIT" => {
                let cid = p[1];
                let entry_axis = parse_axis(p[2]);
                let entry_rect = portal::FoundRectangle {
                    min_corner: [p[3].parse().unwrap(), p[4].parse().unwrap(), p[5].parse().unwrap()],
                    axis1_size: p[6].parse().unwrap(),
                    axis2_size: p[7].parse().unwrap(),
                };
                let exit_axis = parse_axis(p[8]);
                let exit_rect = portal::FoundRectangle {
                    min_corner: [p[9].parse().unwrap(), p[10].parse().unwrap(), p[11].parse().unwrap()],
                    axis1_size: p[12].parse().unwrap(),
                    axis2_size: p[13].parse().unwrap(),
                };
                let entry_pos = Vec3::new(d64(p[14]), d64(p[15]), d64(p[16]));
                let entry_vel = Vec3::new(d64(p[17]), d64(p[18]), d64(p[19]));
                let yaw = f32b(p[20]);
                let pitch = f32b(p[21]);
                let rel = portal::relative_position(entry_rect, entry_axis, entry_pos, 0.6, 1.8);
                assert_eq!(rel.x.to_bits(), h64(p[22]), "{cid} rel.x");
                assert_eq!(rel.y.to_bits(), h64(p[23]), "{cid} rel.y");
                assert_eq!(rel.z.to_bits(), h64(p[24]), "{cid} rel.z");
                let exit_pos = portal::exit_position(exit_rect, exit_axis, rel, 0.6, 1.8);
                assert_eq!(exit_pos.x.to_bits(), h64(p[25]), "{cid} exit.x");
                assert_eq!(exit_pos.y.to_bits(), h64(p[26]), "{cid} exit.y");
                assert_eq!(exit_pos.z.to_bits(), h64(p[27]), "{cid} exit.z");
                let tr = portal::transit(
                    portal::PortalTransit { position: entry_pos, velocity: entry_vel, yaw, pitch },
                    entry_axis,
                    exit_axis,
                    exit_pos,
                );
                assert_eq!(tr.position.x.to_bits(), h64(p[28]), "{cid} abs.x");
                assert_eq!(tr.position.y.to_bits(), h64(p[29]), "{cid} abs.y");
                assert_eq!(tr.position.z.to_bits(), h64(p[30]), "{cid} abs.z");
                assert_eq!(tr.velocity.x.to_bits(), h64(p[31]), "{cid} abs.vx");
                assert_eq!(tr.velocity.y.to_bits(), h64(p[32]), "{cid} abs.vy");
                assert_eq!(tr.velocity.z.to_bits(), h64(p[33]), "{cid} abs.vz");
                assert_eq!(tr.yaw.to_bits(), h32(p[34]), "{cid} abs.yRot");
                assert_eq!(tr.pitch.to_bits(), h32(p[35]), "{cid} abs.xRot");
            }
            "PORTAL_PROC" => {
                let p_delay: i32 = p[1].parse().unwrap();
                let b_delay: i32 = p[2].parse().unwrap();
                let e_delay: i32 = p[3].parse().unwrap();
                let trig_tick: i32 = p[4].parse().unwrap();
                let pt_trig: i32 = p[5].parse().unwrap();
                let pt_decay: i32 = p[6].parse().unwrap();
                let exp_flag = p[7] == "1";

                assert_eq!(portal::dimension_changing_delay(true, false), p_delay);
                assert_eq!(portal::PORTAL_COOLDOWN_VEHICLE, b_delay);
                assert_eq!(portal::dimension_changing_delay(false, false), e_delay);

                let mut state = portal::PortalProcessState::default();
                let mut fired_at = -1;
                for t in 1..=85 {
                    state.set_inside_portal([0, 64, 0], true, p_delay);
                    match state.handle_portal(true, portal::NETHER_PORTAL_DEFAULT_DELAY_TICKS, p_delay, false) {
                        portal::PortalHandleOutcome::NullDestinationLockedOut => {
                            fired_at = t;
                            break;
                        }
                        portal::PortalHandleOutcome::Counting { .. } => {}
                        other => panic!("unexpected outcome at t={t}: {other:?}"),
                    }
                }
                assert_eq!(fired_at, trig_tick);
                assert_eq!(state.portal_time, pt_trig);
                assert_eq!(state.portal_cooldown, p_delay);
                assert!(state.has_processor, "null destination keeps portalProcessor");
                let dec = state.handle_portal(true, portal::NETHER_PORTAL_DEFAULT_DELAY_TICKS, p_delay, false);
                assert_eq!(
                    dec,
                    portal::PortalHandleOutcome::Decayed {
                        portal_time: pt_decay,
                        expired: exp_flag,
                    }
                );
            }
            other => panic!("unexpected row kind: {other}"),
        }
    }
    assert_eq!(row_count, 65);
}

#[test]
fn jar_movement_wired() {
    use law::{
        apply_server_resource_event, calculate_fall_power, player, server_freeze_step, tick,
        try_tick, Block, BoxWorld, Fluid, FluidKind, Input, Inside, PlayerState, Pose,
        ServerResourceEvent, VoxelShape, Aabb,
    };
    let h64 = |s: &str| f64::from_bits(u64::from_str_radix(s, 16).unwrap());
    let h32 = |s: &str| f32::from_bits(u32::from_str_radix(s, 16).unwrap());
    let bit = |s: &str| match s { "1" => true, "0" => false, _ => panic!("bad bool {s}") };

    let mut count = 0usize;
    for (idx, row) in include_str!("data/movement_wired.tsv").lines().enumerate() {
        if row.trim().is_empty() { continue; }
        count += 1;
        let c: Vec<&str> = row.split('\t').collect();
        match c[0] {
            "ABILITY_SWITCH" => {
                let mut s = PlayerState { profile: law::Profile { flying_speed: 0.08, ..Default::default() }.into(), ..PlayerState::default() };
                s.set_game_mode(player::GameType::by_id(c[1].parse().unwrap()));
                s.flying = bit(c[2]);
                s.set_game_mode(player::GameType::by_id(c[3].parse().unwrap()));
                assert_eq!(
                    (s.profile.invulnerable, s.flying, s.profile.mayfly, s.profile.instabuild, s.profile.may_build, s.profile.flying_speed.to_bits()),
                    (bit(c[4]), bit(c[5]), bit(c[6]), bit(c[7]), bit(c[8]), h32(c[9]).to_bits()),
                    "row {idx}: {row}"
                );
            }
            "FLY_SPEED" => {
                let actual = law::core::air_speed(&PlayerState { flying: bit(c[1]), sprinting: bit(c[3]),
                    profile: law::Profile { riding: bit(c[2]), flying_speed: h32(c[4]), ..Default::default() }.into(), ..PlayerState::default() });
                assert_eq!(actual.to_bits(), h32(c[5]).to_bits(), "row {idx}: {row}");
            }
            "FLY_TRAVEL" => {
                let dir: i32 = c[1].parse().unwrap();
                let fspeed = h32(c[2]);
                let init_vy = h64(c[3]);
                let sprinting = bit(c[4]);
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    velocity: Vec3::new(0.0, init_vy, 0.0),
                    flying: true,
                    sprinting,
                    sprint_speed_modifier: sprinting, profile: law::Profile { game_mode: player::GameType::Creative, mayfly: true, invulnerable: true, instabuild: true, flying_speed: fspeed, ..Default::default() }.into(), ..PlayerState::default() };
                let inp = Input {
                    forward: true,
                    sprint: sprinting,
                    jump: dir > 0,
                    sneak: dir < 0,
                    ..Input::default()
                };
                tick(&mut s, &BoxWorld::default(), &inp);
                assert_eq!(
                    (s.velocity.x.to_bits(), s.velocity.y.to_bits(), s.velocity.z.to_bits()),
                    (h64(c[6]).to_bits(), h64(c[7]).to_bits(), h64(c[8]).to_bits()),
                    "row {idx}: {row}"
                );
            }
            "EFFECT_SPEED" => {
                let sp: i32 = c[1].parse().unwrap();
                let sl: i32 = c[2].parse().unwrap();
                let sprinting = bit(c[3]);
                let mut s = PlayerState::default();
                s.set_speed_effect((sp >= 0).then_some(sp as u8));
                s.set_slowness_effect((sl >= 0).then_some(sl as u8));
                s.set_sprinting(sprinting);
                assert_eq!(s.movement_speed_value().to_bits(), h64(c[4]).to_bits(), "row {idx}: {row}");
                let mut ts = s.clone();
                ts.effects = sp >= 0 || sl >= 0;
                ts.position = Vec3::new(0.5, 64.0, 0.5);
                let inp = Input { forward: true, sprint: sprinting, ..Input::default() };
                try_tick(&mut ts, &BoxWorld::default(), &inp).expect("modeled effects must not refuse");
                assert_eq!(ts.speed.to_bits(), h32(c[5]).to_bits(), "row {idx}: {row}");
                if sp < 0 && sl < 0 {
                    ts.effects = true;
                    assert!(try_tick(&mut ts, &BoxWorld::default(), &inp).is_err(), "unmodeled effect flag must refuse");
                }
            }
            "EFFECT_SPEED_ORD" => {
                let mut s = PlayerState::default();
                s.set_speed_effect(Some(1));
                s.set_slowness_effect(Some(0));
                s.set_sprinting(true);
                assert_eq!(s.movement_speed_value().to_bits(), h64(c[2]).to_bits(), "row {idx}: {row}");
            }
            "EFFECT_JUMP" => {
                let jb: i32 = c[1].parse().unwrap();
                let mut s = PlayerState {
                    position: Vec3::new(0.5, 64.0, 0.5),
                    on_ground: true,
                    yaw: 35.0,
                    effects: jb >= 0,
                    ..PlayerState::default()
                };
                s.set_jump_boost((jb >= 0).then_some(jb as u8));
                s.set_sprinting(true);
                assert_eq!(s.jump_boost_power().to_bits(), h32(c[2]).to_bits(), "row {idx} boost");
                assert_eq!(s.profile.safe_fall_distance.to_bits(), h64(c[4]).to_bits(), "row {idx} safe_fall");
                assert_eq!(calculate_fall_power(&s, 5.5).to_bits(), h64(c[5]).to_bits(), "row {idx} fall_power");
                // Confirm law::tick executes jump_from_ground + travel_air without effects refusal.
                tick(&mut s, &BoxWorld::default(), &Input { jump: true, forward: true, sprint: true, yaw: 35.0, ..Input::default() });
                let vertical = 0.98f32 as f64;
                assert_eq!(s.velocity.y.to_bits(), ((h64(c[7]) - 0.08) * vertical).to_bits(), "row {idx} jump vy");
            }
            "CRAWL_POSE" => {
                let case_id: i32 = c[1].parse().unwrap();
                let expected_pose = match c[2] {
                    "0" => Pose::Standing,
                    "1" => Pose::Crouching,
                    "2" => Pose::Swimming,
                    o => panic!("bad pose {o}"),
                };
                let mut w = BoxWorld::default();
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5), profile: law::Profile { gravity: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                let mut inp = Input::default();
                match case_id {
                    0 => {}
                    1 => inp.sneak = true,
                    2 => { inp.sneak = true; s.flying = true; s.profile.mayfly = true; }
                    3 => { w.set([0, 65, 0], Block::full()); s.position.y = 63.4; }
                    4 => { w.set([0, 65, 0], Block::full()); }
                    5 => { w.set([0, 65, 0], Block::full()); s.set_game_mode(player::GameType::Spectator); }
                    _ => panic!("bad case_id {case_id}"),
                }
                tick(&mut s, &w, &inp);
                assert_eq!(s.pose, expected_pose, "row {idx}: {row}");
            }
            "EXHAUSTION" => {
                let case_id: i32 = c[1].parse().unwrap();
                if case_id == 2 {
                    // Riding is guarded by LivingEntity.travelRidden refusal while ServerPlayer checkMovementStatistics guards riding.
                    continue;
                }
                let mut w = BoxWorld::default();
                w.set([0, 63, 0], Block::full());
                w.set([1, 63, 0], Block::full());
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    on_ground: true, profile: law::Profile { gravity: 0.0, movement_speed_base: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                s.set_sprinting(true);
                if case_id == 1 {
                    s.profile.invulnerable = true;
                }
                if case_id == 3 {
                    for x in 0..=1 {
                        for y in 64..=65 {
                            w.set([x, y, 0], Block {
                                fluid: Fluid { kind: FluidKind::Water, height: 1.0, ..Fluid::default() },
                                ..Block::default()
                            });
                        }
                    }
                    s.on_ground = false;
                    s.flying = true;
                    s.profile.mayfly = true;
                    s.velocity = Vec3::new(0.28, 0.12, 0.0);
                } else {
                    s.velocity = Vec3::new(0.28, -0.08, 0.0);
                }
                let inp = if case_id == 3 { Input::default() } else { Input { forward: true, ..Input::default() } };
                tick(&mut s, &w, &inp);
                assert_eq!(s.exhaustion.to_bits(), h32(c[2]).to_bits(), "row {idx}: {row}");
            }
            "REGEN_GUARD" => {
                let mut w = BoxWorld::default();
                w.difficulty = 0;
                w.natural_regeneration = true;
                let mut s = PlayerState {
                    health: h32(c[1]),
                    food_level: 15,
                    saturation: 1.0,
                    ..PlayerState::default()
                };
                apply_server_resource_event(&mut s, &w, ServerResourceEvent::Regeneration(c[2].parse().unwrap()));
                assert_eq!(
                    (s.health.to_bits(), s.food_level, s.saturation.to_bits()),
                    (h32(c[3]).to_bits(), c[4].parse::<i32>().unwrap(), h32(c[5]).to_bits()),
                    "row {idx}: {row}"
                );
            }
            "BUBBLE_COLUMN" => {
                let drag = bit(c[1]);
                let above_kind: i32 = c[2].parse().unwrap();
                let intersects = bit(c[3]);
                let flying = bit(c[4]);
                let init_vy = h64(c[5]);
                let mut w = BoxWorld::default();
                let mut col_block = Block {
                    air: false,
                    inside: Inside::BubbleColumn { drag },
                    ..Block::default()
                };
                if !intersects {
                    // Far corner outside player's AABB [0.2..0.8] while in block [0,64,0].
                    col_block.inside_shape = Some(VoxelShape::boxes(&[Aabb::new(0.9, 0.0, 0.9, 1.0, 1.0, 1.0)]));
                }
                w.set([0, 64, 0], col_block);
                if above_kind == 1 {
                    w.set([0, 65, 0], Block {
                        air: false,
                        inside_shape: Some(VoxelShape::default()),
                        fluid: Fluid { kind: FluidKind::Water, height: 1.0, ..Fluid::default() },
                        ..Block::default()
                    });
                } else if above_kind == 2 {
                    w.set([0, 65, 0], Block::boxes(&[Aabb::new(0.0, 0.9, 0.0, 1.0, 1.0, 1.0)]));
                }
                let mut s = PlayerState { position: Vec3::new(0.5, if above_kind == 2 && init_vy > 0.0 { 63.5 } else { 64.0 }, 0.5),
                    pose: Pose::Swimming,
                    velocity: Vec3::new(0.0, init_vy, 0.0),
                    flying,
                    fall_distance: 5.0, profile: law::Profile { gravity: 0.0, air_drag_modifier: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                tick(&mut s, &w, &Input::default());
                let expected_vy = if flying { h64(c[6]) * 0.6 } else { h64(c[6]) };
                let pre_fall = if flying { 0.0 } else { h64(c[7]) };
                let after_move_fall = if above_kind == 1 && init_vy >= 1.0 {
                    0.0 // move >= 1.0 crosses Water at [0,65,0] in Entity.move -> fall_reset_ray
                } else if init_vy < 0.0 {
                    pre_fall - (init_vy as f32) as f64
                } else {
                    pre_fall
                };
                let expected_fall = if !flying && h64(c[7]) == 0.0 { 0.0 } else { after_move_fall };
                assert_eq!(s.velocity.y.to_bits(), expected_vy.to_bits(), "row {idx} vy: {row}");
                assert_eq!(s.fall_distance.to_bits(), expected_fall.to_bits(), "row {idx} fall: {row}");
            }
            "FROST_MOD" => {
                let tf: i32 = c[1].parse().unwrap();
                let feet_non_air = bit(c[2]);
                let has_mod = bit(c[3]);
                let mod_amount = h64(c[4]);
                let (_, frost, _) = server_freeze_step(tf, true, true, feet_non_air, 1);
                assert_eq!(frost.is_some(), has_mod, "row {idx} has_mod: {row}");
                assert_eq!(frost.unwrap_or(0.0).to_bits(), mod_amount.to_bits(), "row {idx} frost: {row}");
            }
            "COLLECTOR_FREEZE" => {
                let steps: i32 = c[1].parse().unwrap();
                let init_frozen: i32 = c[2].parse().unwrap();
                let spectator = bit(c[3]);
                let mut w = BoxWorld::default();
                for dy in 0..steps {
                    w.set([0, 64 + dy, 0], Block { air: false, inside: Inside::PowderSnow, ..Block::default() });
                }
                let mut s = PlayerState { position: Vec3::new(0.5, if steps == 3 { 66.7 } else { 64.0 }, 0.5),
                    velocity: Vec3::new(0.0, if steps == 3 { -2.5 } else { 0.0 }, 0.0),
                    ticks_frozen: init_frozen,
                    remaining_fire_ticks: 40,
                    flying: spectator, profile: law::Profile { gravity: 0.0, air_drag_modifier: 0.0, game_mode: if spectator { player::GameType::Spectator } else { player::GameType::Survival }, mayfly: spectator, invulnerable: spectator, ..Default::default() }.into(), ..PlayerState::default() };
                tick(&mut s, &w, &Input::default());
                assert_eq!(s.in_powder_snow, bit(c[4]), "row {idx} inSnow: {row}");
                assert_eq!(s.ticks_frozen, c[5].parse::<i32>().unwrap(), "row {idx} tf: {row}");
                assert_eq!(s.remaining_fire_ticks, c[6].parse::<i32>().unwrap(), "row {idx} fire: {row}");
            }
            "COLLECTOR_LAVA" => {
                let with_water = bit(c[1]);
                let with_fire_res = bit(c[2]);
                let invuln = bit(c[3]);
                let mut w = BoxWorld::default();
                w.set([0, 64, 0], Block {
                    air: false,
                    inside_shape: Some(VoxelShape::default()),
                    fluid: Fluid { kind: FluidKind::Lava, height: 1.0, ..Fluid::default() },
                    ..Block::default()
                });
                if with_water {
                    w.set([1, 64, 0], Block {
                        air: false,
                        inside_shape: Some(VoxelShape::default()),
                        fluid: Fluid { kind: FluidKind::Water, height: 1.0, ..Fluid::default() },
                        ..Block::default()
                    });
                }
                let mut s = PlayerState { // X=0.8 makes player bounding box [0.5..1.1] intersect both [0,64,0] (lava) and [1,64,0] (water) at step 0.
                    position: Vec3::new(if with_water { 0.8 } else { 0.5 }, 64.0, 0.5),
                    ticks_frozen: 50,
                    effects: with_fire_res, profile: law::Profile { invulnerable: invuln, fire_resistance: with_fire_res, ..Default::default() }.into(), ..PlayerState::default() };
                tick(&mut s, &w, &Input::default());
                assert_eq!(s.health.to_bits(), h32(c[4]).to_bits(), "row {idx} health: {row}");
                assert_eq!(s.ticks_frozen, c[5].parse::<i32>().unwrap(), "row {idx} tf: {row}");
                assert_eq!(s.remaining_fire_ticks, c[6].parse::<i32>().unwrap(), "row {idx} fire: {row}");
                assert_eq!(s.hurt_cooldown.remaining_ticks, c[7].parse::<i32>().unwrap(), "row {idx} invulnTime: {row}");
                assert_eq!(s.exhaustion.to_bits(), h32(c[8]).to_bits(), "row {idx} ex: {row}");
            }
            "FREEZE_STEP" => {
                let tf: i32 = c[1].parse().unwrap();
                let in_snow = bit(c[2]);
                let spectator = bit(c[3]);
                let tc: i32 = c[4].parse().unwrap();
                let mut w = BoxWorld::default();
                w.set([0, 63, 0], Block::full());
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    on_ground: true,
                    ticks_frozen: tf,
                    flying: spectator, profile: law::Profile { gravity: 0.0, game_mode: if spectator { player::GameType::Spectator } else { player::GameType::Survival }, mayfly: spectator, invulnerable: spectator, ..Default::default() }.into(), ..PlayerState::default() };
                if in_snow {
                    w.set([0, 64, 0], Block { air: false, inside: Inside::PowderSnow, ..Block::default() });
                    tick(&mut s, &w, &Input::default());
                }
                let (frozen, frost, damage) = server_freeze_step(s.ticks_frozen, in_snow && !spectator, !spectator, true, tc);
                assert_eq!(frozen, c[5].parse::<i32>().unwrap(), "row {idx} tf: {row}");
                assert_eq!(frost.is_some(), bit(c[6]), "row {idx} has_mod: {row}");
                assert_eq!(frost.unwrap_or(0.0).to_bits(), h64(c[7]).to_bits(), "row {idx} frost: {row}");
                assert_eq!(damage.unwrap_or(0.0).to_bits(), h32(c[8]).to_bits(), "row {idx} dmg: {row}");
            }
            other => panic!("unknown tag {other}"),
        }
    }
    assert_eq!(count, 270);
}

#[test]
fn jar_world_items_vehicles() {
    use law::block_hooks::{
        BambooLeaves, HookContext, HookEntityKind, NonPlayerHookBlock, ShulkerAnimationStatus,
        ShulkerBoxEntityState, ShulkerBreakOutcome,
        WorldHookAction, aabb_contract, bamboo_outline_aabb,
        detector_rail_search_bb, evaluate_entity_inside, evaluate_step_on, get_seed,
        get_xz_offset, hopper_suck_aabb, shulker_break_outcome,
        shulker_can_open, shulker_can_open_box, shulker_collision_aabb,
        shulker_entity_push_vector, shulker_open_support_aabb, shulker_placement_facing,
        turtle_egg_can_destroy,
    };
    use law::basis::cell::StateId;
    use law::interact::{BlockId, Direction};
    use law::items::{
        can_consume, consume_ticks, eat, eat_food, eat_with_modifier, push, riptide_push,
        saturation_by_modifier, scale_effect_duration, spin_attack_hull, start_using_item,
        stop_using_item, trident_spin_attack_strength, use_item_tick, SpinAttack, SpinTouch,
        COOKED_BEEF, DRIED_KELP, ENCHANTED_GOLDEN_APPLE, GOLDEN_APPLE, GOLDEN_CARROT,
        GROUNDED_HOP, HONEY_BOTTLE, POTION, SPIN_ATTACK_POSE_EYE_HEIGHT, SPIN_ATTACK_POSE_HEIGHT,
        SPIN_ATTACK_POSE_WIDTH,
    };
    use law::minecart::{CartConditions, Minecart, RailInfo, RailShape, Rails};
    use law::{Block, BlockPos, BoxWorld};
    use std::collections::{HashMap, HashSet};

    #[derive(Default)]
    struct TestRails {
        rails: HashMap<BlockPos, RailInfo>,
        conductors: HashSet<BlockPos>,
    }
    impl TestRails {
        fn add_rail(&mut self, pos: BlockPos, shape: RailShape, powered: bool, is_powered: bool) {
            self.rails.insert(pos, RailInfo::new(shape, powered, is_powered));
        }
    }
    impl Rails for TestRails {
        fn rail_at(&self, pos: BlockPos) -> Option<RailInfo> {
            self.rails.get(&pos).copied()
        }
        fn rails_tag_below(&self, pos: BlockPos) -> bool {
            self.rails.contains_key(&[pos[0], pos[1] - 1, pos[2]])
        }
        fn is_redstone_conductor(&self, pos: BlockPos) -> bool {
            self.conductors.contains(&pos)
        }
    }

    let h64 = |s: &str| f64::from_bits(u64::from_str_radix(s, 16).unwrap());
    let h32 = |s: &str| f32::from_bits(u32::from_str_radix(s, 16).unwrap());
    let bit = |s: &str| s == "1";
    let parse_box = |c: &[&str]| {
        Aabb::new(h64(c[0]), h64(c[1]), h64(c[2]), h64(c[3]), h64(c[4]), h64(c[5]))
    };
    let assert_box = |actual: Aabb, expected: Aabb, ctx: &str| {
        assert_eq!(actual.min.x.to_bits(), expected.min.x.to_bits(), "{ctx} min.x");
        assert_eq!(actual.min.y.to_bits(), expected.min.y.to_bits(), "{ctx} min.y");
        assert_eq!(actual.min.z.to_bits(), expected.min.z.to_bits(), "{ctx} min.z");
        assert_eq!(actual.max.x.to_bits(), expected.max.x.to_bits(), "{ctx} max.x");
        assert_eq!(actual.max.y.to_bits(), expected.max.y.to_bits(), "{ctx} max.y");
        assert_eq!(actual.max.z.to_bits(), expected.max.z.to_bits(), "{ctx} max.z");
    };
    let parse_shape = |s: &str| match s {
        "NORTH_SOUTH" => RailShape::NorthSouth,
        "EAST_WEST" => RailShape::EastWest,
        "ASCENDING_EAST" => RailShape::AscendingEast,
        "ASCENDING_WEST" => RailShape::AscendingWest,
        "ASCENDING_NORTH" => RailShape::AscendingNorth,
        "ASCENDING_SOUTH" => RailShape::AscendingSouth,
        "SOUTH_EAST" => RailShape::SouthEast,
        "SOUTH_WEST" => RailShape::SouthWest,
        "NORTH_WEST" => RailShape::NorthWest,
        "NORTH_EAST" => RailShape::NorthEast,
        other => panic!("unknown rail shape {other}"),
    };
    let parse_dir = |s: &str| match s {
        "DOWN" => Direction::Down,
        "UP" => Direction::Up,
        "NORTH" => Direction::North,
        "SOUTH" => Direction::South,
        "WEST" => Direction::West,
        "EAST" => Direction::East,
        other => panic!("unknown direction {other}"),
    };
    let lookup_item = |s: &str| match s {
        "golden_carrot" => &GOLDEN_CARROT,
        "cooked_beef" => &COOKED_BEEF,
        "golden_apple" => &GOLDEN_APPLE,
        "enchanted_golden_apple" => &ENCHANTED_GOLDEN_APPLE,
        "dried_kelp" => &DRIED_KELP,
        "honey_bottle" => &HONEY_BOTTLE,
        "potion" => &POTION,
        other => panic!("unknown item {other}"),
    };

    let build_cart_scenario = |name: &str| -> (Minecart, TestRails, CartConditions, bool) {
        let mut rails = TestRails::default();
        let mut cond = CartConditions::default();
        let mut clamp_floor = false;
        let cart = match name {
            "S1_BRAKE" => {
                for x in 2..=22 {
                    rails.add_rail([x, 0, 2], RailShape::EastWest, false, true);
                }
                Minecart {
                    position: Vec3::new(2.5, 0.0625, 2.5),
                    velocity: Vec3::new(0.2, 0.0, 0.0),
                }
            }
            "S2_BOOST" => {
                for x in 2..=22 {
                    rails.add_rail([x, 0, 2], RailShape::EastWest, true, true);
                }
                Minecart {
                    position: Vec3::new(2.5, 0.0625, 2.5),
                    velocity: Vec3::new(0.05, 0.0, 0.0),
                }
            }
            "S3_CLIMB_EAST" => {
                for x in 0..=4 {
                    rails.add_rail([x, 0, 8], RailShape::EastWest, false, false);
                }
                rails.add_rail([5, 0, 8], RailShape::AscendingEast, false, true);
                for x in 6..=14 {
                    rails.add_rail([x, 1, 8], RailShape::EastWest, false, false);
                }
                Minecart {
                    position: Vec3::new(0.5, 0.0625, 8.5),
                    velocity: Vec3::new(0.3, 0.0, 0.0),
                }
            }
            "S4_DESCEND_EAST" => {
                for x in 0..=4 {
                    rails.add_rail([x, 0, 8], RailShape::EastWest, false, false);
                }
                rails.add_rail([5, 0, 8], RailShape::AscendingEast, false, false);
                for x in 6..=8 {
                    rails.add_rail([x, 1, 8], RailShape::EastWest, false, false);
                }
                Minecart {
                    position: Vec3::new(6.2, 1.0625, 8.5),
                    velocity: Vec3::new(-0.3, 0.0, 0.0),
                }
            }
            "S4_DESCEND_WEST" => {
                for x in 2..=4 {
                    rails.add_rail([x, 1, 8], RailShape::EastWest, false, false);
                }
                rails.add_rail([5, 0, 8], RailShape::AscendingWest, false, false);
                for x in 6..=10 {
                    rails.add_rail([x, 0, 8], RailShape::EastWest, false, false);
                }
                Minecart {
                    position: Vec3::new(4.8, 1.0625, 8.5),
                    velocity: Vec3::new(0.3, 0.0, 0.0),
                }
            }
            "S4_DESCEND_NORTH" => {
                for z in 2..=4 {
                    rails.add_rail([8, 1, z], RailShape::NorthSouth, false, false);
                }
                rails.add_rail([8, 0, 5], RailShape::AscendingNorth, false, false);
                for z in 6..=10 {
                    rails.add_rail([8, 0, z], RailShape::NorthSouth, false, false);
                }
                Minecart {
                    position: Vec3::new(8.5, 1.0625, 4.8),
                    velocity: Vec3::new(0.0, 0.0, 0.3),
                }
            }
            "S4_DESCEND_SOUTH" => {
                for z in 0..=4 {
                    rails.add_rail([8, 0, z], RailShape::NorthSouth, false, false);
                }
                rails.add_rail([8, 0, 5], RailShape::AscendingSouth, false, false);
                for z in 6..=8 {
                    rails.add_rail([8, 1, z], RailShape::NorthSouth, false, false);
                }
                Minecart {
                    position: Vec3::new(8.5, 1.0625, 6.2),
                    velocity: Vec3::new(0.0, 0.0, -0.3),
                }
            }
            "S5_CURVE_SE_IN" => {
                for x in 21..=23 {
                    rails.add_rail([x, 0, 19], RailShape::EastWest, false, false);
                }
                rails.add_rail([20, 0, 19], RailShape::SouthEast, false, false);
                for z in 20..=23 {
                    rails.add_rail([20, 0, z], RailShape::NorthSouth, false, false);
                }
                Minecart {
                    position: Vec3::new(21.2, 0.0625, 19.5),
                    velocity: Vec3::new(-0.35, 0.0, 0.0),
                }
            }
            "S5_CURVE_SW" => {
                for x in 17..=19 {
                    rails.add_rail([x, 0, 19], RailShape::EastWest, false, false);
                }
                rails.add_rail([20, 0, 19], RailShape::SouthWest, false, false);
                for z in 20..=23 {
                    rails.add_rail([20, 0, z], RailShape::NorthSouth, false, false);
                }
                Minecart {
                    position: Vec3::new(19.8, 0.0625, 19.5),
                    velocity: Vec3::new(0.35, 0.0, 0.0),
                }
            }
            "S5_CURVE_NW" => {
                for x in 17..=19 {
                    rails.add_rail([x, 0, 19], RailShape::EastWest, false, false);
                }
                rails.add_rail([20, 0, 19], RailShape::NorthWest, false, false);
                for z in 15..=18 {
                    rails.add_rail([20, 0, z], RailShape::NorthSouth, false, false);
                }
                Minecart {
                    position: Vec3::new(19.8, 0.0625, 19.5),
                    velocity: Vec3::new(0.35, 0.0, 0.0),
                }
            }
            "S5_CURVE_NE" => {
                for x in 21..=23 {
                    rails.add_rail([x, 0, 19], RailShape::EastWest, false, false);
                }
                rails.add_rail([20, 0, 19], RailShape::NorthEast, false, false);
                for z in 15..=18 {
                    rails.add_rail([20, 0, z], RailShape::NorthSouth, false, false);
                }
                Minecart {
                    position: Vec3::new(21.2, 0.0625, 19.5),
                    velocity: Vec3::new(-0.35, 0.0, 0.0),
                }
            }
            "S6_OFF_GROUND" => {
                cond.on_ground = true;
                clamp_floor = true;
                Minecart {
                    position: Vec3::new(10.06, 0.0, 30.5),
                    velocity: Vec3::new(0.04, 0.0, 0.0),
                }
            }
            "S6_OFF_AIR" => Minecart {
                position: Vec3::new(10.06, 10.0, 30.5),
                velocity: Vec3::new(0.3, 0.0, 0.0),
            },
            "S7_OCCUPIED" => {
                cond.occupied = true;
                for x in 2..=22 {
                    rails.add_rail([x, 0, 40], RailShape::EastWest, true, true);
                }
                Minecart {
                    position: Vec3::new(2.5, 0.0625, 40.5),
                    velocity: Vec3::new(0.05, 0.0, 0.0),
                }
            }
            "S8_WALL_SOUTH" => {
                for z in 46..=50 {
                    rails.add_rail([5, 0, z], RailShape::NorthSouth, true, true);
                }
                rails.conductors.insert([5, 0, 51]);
                Minecart {
                    position: Vec3::new(5.5, 0.0625, 50.5),
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                }
            }
            "S8_WALL_WEST" => {
                for x in 5..=9 {
                    rails.add_rail([x, 0, 50], RailShape::EastWest, true, true);
                }
                rails.conductors.insert([4, 0, 50]);
                Minecart {
                    position: Vec3::new(5.5, 0.0625, 50.5),
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                }
            }
            "S9_WATER_FLAT" => {
                cond.in_water = true;
                for x in 0..=10 {
                    rails.add_rail([x, 0, 60], RailShape::EastWest, false, false);
                }
                Minecart {
                    position: Vec3::new(2.5, 0.0625, 60.5),
                    velocity: Vec3::new(0.3, 0.0, 0.0),
                }
            }
            "S9_WATER_SLOPE" => {
                cond.in_water = true;
                rails.add_rail([4, 0, 62], RailShape::EastWest, false, false);
                rails.add_rail([5, 0, 62], RailShape::AscendingEast, false, false);
                rails.add_rail([6, 1, 62], RailShape::EastWest, false, false);
                Minecart {
                    position: Vec3::new(5.5, 0.0625, 62.5),
                    velocity: Vec3::new(0.15, 0.0, 0.0),
                }
            }
            other => panic!("unknown cart scenario {other}"),
        };
        (cart, rails, cond, clamp_floor)
    };

    let mut active_cart: Option<(String, Minecart, TestRails, CartConditions, bool)> = None;
    let mut shulker_anim_open = ShulkerBoxEntityState {
        status: ShulkerAnimationStatus::Opening,
        progress: 0.0,
        progress_old: 0.0,
    };
    let mut shulker_anim_close = ShulkerBoxEntityState {
        status: ShulkerAnimationStatus::Closing,
        progress: 1.0,
        progress_old: 1.0,
    };

    let mut count = 0usize;
    for (idx, row) in include_str!("data/world_items_vehicles.tsv").lines().enumerate() {
        count += 1;
        let c: Vec<_> = row.split('\t').collect();
        match c[0] {
            "MC_GET_POS" => {
                let shape = parse_shape(c[1]);
                let mut rails = TestRails::default();
                rails.add_rail([0, 64, 0], shape, false, false);
                let cart = Minecart::default();
                let got = cart.get_pos(&rails, h64(c[2]), h64(c[3]), h64(c[4]));
                assert_eq!(got.is_some(), bit(c[6]), "row {idx}: {row}");
                if let Some(p) = got {
                    assert_eq!(p.x.to_bits(), h64(c[7]).to_bits(), "row {idx} rx: {row}");
                    assert_eq!(p.y.to_bits(), h64(c[8]).to_bits(), "row {idx} ry: {row}");
                    assert_eq!(p.z.to_bits(), h64(c[9]).to_bits(), "row {idx} rz: {row}");
                }
            }
            "MC_GET_POS_OFFS" => {
                let shape = parse_shape(c[1]);
                let mut rails = TestRails::default();
                rails.add_rail([0, 64, 0], shape, false, false);
                let cart = Minecart::default();
                let got = cart.get_pos_offs(&rails, h64(c[2]), h64(c[3]), h64(c[4]), h64(c[5]));
                assert_eq!(got.is_some(), bit(c[6]), "row {idx}: {row}");
                if let Some(p) = got {
                    assert_eq!(p.x.to_bits(), h64(c[7]).to_bits(), "row {idx} rx: {row}");
                    assert_eq!(p.y.to_bits(), h64(c[8]).to_bits(), "row {idx} ry: {row}");
                    assert_eq!(p.z.to_bits(), h64(c[9]).to_bits(), "row {idx} rz: {row}");
                }
            }
            "MC_TICK" => {
                let scen = c[1];
                let step: usize = c[2].parse().unwrap();
                if step == 1 || active_cart.as_ref().map(|a| a.0.as_str()) != Some(scen) {
                    let (cart, rails, cond, clamp_floor) = build_cart_scenario(scen);
                    active_cart = Some((scen.to_string(), cart, rails, cond, clamp_floor));
                }
                let (_, cart, rails, cond, clamp_floor) = active_cart.as_mut().unwrap();
                cart.tick(rails, *cond);
                if *clamp_floor {
                    // Entity.move resolves floor collision at y=0.0 before next tick.
                    cart.position.y = 0.0;
                    cart.velocity.y = 0.0;
                }
                assert_eq!(cart.position.x.to_bits(), h64(c[3]).to_bits(), "row {idx} px: {row}");
                assert_eq!(cart.position.y.to_bits(), h64(c[4]).to_bits(), "row {idx} py: {row}");
                assert_eq!(cart.position.z.to_bits(), h64(c[5]).to_bits(), "row {idx} pz: {row}");
                assert_eq!(cart.velocity.x.to_bits(), h64(c[6]).to_bits(), "row {idx} vx: {row}");
                assert_eq!(cart.velocity.y.to_bits(), h64(c[7]).to_bits(), "row {idx} vy: {row}");
                assert_eq!(cart.velocity.z.to_bits(), h64(c[8]).to_bits(), "row {idx} vz: {row}");
            }
            "MC_STEER" => {
                let mut rails = TestRails::default();
                for x in 2..=10 {
                    rails.add_rail([x, 0, 70], RailShape::EastWest, false, true);
                }
                let mut cart = Minecart {
                    position: Vec3::new(5.5, 0.0625, 70.5),
                    velocity: Vec3::default(),
                };
                let intent = Vec3::new(h64(c[1]), h64(c[2]), h64(c[3]));
                cart.tick(
                    &rails,
                    CartConditions {
                        occupied: true,
                        passenger_intent: Some(intent),
                        ..CartConditions::default()
                    },
                );
                assert_eq!(cart.position.x.to_bits(), h64(c[4]).to_bits(), "row {idx} px: {row}");
                assert_eq!(cart.position.y.to_bits(), h64(c[5]).to_bits(), "row {idx} py: {row}");
                assert_eq!(cart.position.z.to_bits(), h64(c[6]).to_bits(), "row {idx} pz: {row}");
                assert_eq!(cart.velocity.x.to_bits(), h64(c[7]).to_bits(), "row {idx} vx: {row}");
                assert_eq!(cart.velocity.y.to_bits(), h64(c[8]).to_bits(), "row {idx} vy: {row}");
                assert_eq!(cart.velocity.z.to_bits(), h64(c[9]).to_bits(), "row {idx} vz: {row}");
            }
            "ITEM_PROPS" => {
                let item = lookup_item(c[1]);
                assert_eq!(item.consume_ticks, c[2].parse::<i32>().unwrap(), "row {idx}: {row}");
                assert_eq!(item.has_food, bit(c[3]), "row {idx}: {row}");
                assert_eq!(item.nutrition, c[4].parse::<i32>().unwrap(), "row {idx}: {row}");
                assert_eq!(item.saturation.to_bits(), h32(c[5]).to_bits(), "row {idx}: {row}");
                assert_eq!(item.can_always_eat, bit(c[6]), "row {idx}: {row}");
            }
            "ITEM_CONSUME_TICKS" => {
                assert_eq!(consume_ticks(h32(c[1])), c[2].parse::<i32>().unwrap(), "row {idx}: {row}");
            }
            "ITEM_SAT_MOD" => {
                let n: i32 = c[1].parse().unwrap();
                let sat = saturation_by_modifier(n, h32(c[2]));
                assert_eq!(sat.to_bits(), h32(c[3]).to_bits(), "row {idx}: {row}");
            }
            "ITEM_CAN_CONSUME" => {
                let item = lookup_item(c[1]);
                let fl: i32 = c[2].parse().unwrap();
                let invuln = bit(c[3]);
                assert_eq!(can_consume(item, fl, invuln), bit(c[4]), "row {idx}: {row}");
            }
            "ITEM_EAT_FOOD" => {
                let mut fl: i32 = c[1].parse().unwrap();
                let mut sat = h32(c[2]);
                let nut: i32 = c[3].parse().unwrap();
                let baked_sat = h32(c[4]);
                eat_food(&mut fl, &mut sat, nut, baked_sat);
                assert_eq!(fl, c[5].parse::<i32>().unwrap(), "row {idx} fl: {row}");
                assert_eq!(sat.to_bits(), h32(c[6]).to_bits(), "row {idx} sat: {row}");
            }
            "ITEM_EAT_MOD" => {
                let mut fl: i32 = c[1].parse().unwrap();
                let mut sat = h32(c[2]);
                let nut: i32 = c[3].parse().unwrap();
                let mod_f = h32(c[4]);
                eat_with_modifier(&mut fl, &mut sat, nut, mod_f);
                assert_eq!(fl, c[5].parse::<i32>().unwrap(), "row {idx} fl: {row}");
                assert_eq!(sat.to_bits(), h32(c[6]).to_bits(), "row {idx} sat: {row}");
            }
            "ITEM_COUNTDOWN" => {
                let item = lookup_item(c[1]);
                let mut fl: i32 = c[2].parse().unwrap();
                let mut sat = h32(c[3]);
                let run_ticks: i32 = c[4].parse().unwrap();
                let abort_at: i32 = c[5].parse().unwrap();
                let mut rem = start_using_item(item);
                for t in 1..=run_ticks {
                    if rem > 0 && use_item_tick(&mut rem, false, item.use_on_release) {
                        eat(&mut fl, &mut sat, item);
                    }
                    if t == abort_at {
                        stop_using_item(&mut rem);
                    }
                }
                assert_eq!(rem, c[6].parse::<i32>().unwrap(), "row {idx} rem: {row}");
                assert_eq!(fl, c[7].parse::<i32>().unwrap(), "row {idx} fl: {row}");
                assert_eq!(sat.to_bits(), h32(c[8]).to_bits(), "row {idx} sat: {row}");
            }
            "ITEM_POTION_SCALE" => {
                let dur: i32 = c[1].parse().unwrap();
                let scale = h32(c[2]);
                assert_eq!(scale_effect_duration(dur, scale), c[3].parse::<i32>().unwrap(), "row {idx}: {row}");
            }
            "TRIDENT_STRENGTH" => {
                let lvl: i32 = c[1].parse().unwrap();
                assert_eq!(trident_spin_attack_strength(lvl).to_bits(), h32(c[2]).to_bits(), "row {idx}: {row}");
            }
            "TRIDENT_LAUNCH" => {
                let imp = riptide_push(h32(c[1]), h32(c[2]), h32(c[3]));
                let v0 = Vec3::new(h64(c[4]), h64(c[5]), h64(c[6]));
                assert_eq!(imp.x.to_bits(), h64(c[7]).to_bits(), "row {idx} dx: {row}");
                assert_eq!(imp.y.to_bits(), h64(c[8]).to_bits(), "row {idx} dy: {row}");
                assert_eq!(imp.z.to_bits(), h64(c[9]).to_bits(), "row {idx} dz: {row}");
                let res = push(v0, imp);
                assert_eq!(res.x.to_bits(), h64(c[10]).to_bits(), "row {idx} px: {row}");
                assert_eq!(res.y.to_bits(), h64(c[11]).to_bits(), "row {idx} py: {row}");
                assert_eq!(res.z.to_bits(), h64(c[12]).to_bits(), "row {idx} pz: {row}");
            }
            "TRIDENT_HOP_POSE" => {
                assert_eq!(GROUNDED_HOP.y.to_bits(), h64(c[1]).to_bits(), "row {idx} hop");
                assert_eq!(SPIN_ATTACK_POSE_WIDTH.to_bits(), h32(c[2]).to_bits(), "row {idx} w");
                assert_eq!(SPIN_ATTACK_POSE_HEIGHT.to_bits(), h32(c[3]).to_bits(), "row {idx} h");
                assert_eq!(SPIN_ATTACK_POSE_EYE_HEIGHT.to_bits(), h32(c[4]).to_bits(), "row {idx} eye");
            }
            "TRIDENT_SPIN_CHECK" => {
                let old_box = Aabb::new(0.0, 64.0, 0.0, 0.6, 64.6, 0.6);
                let new_box = Aabb::new(1.2, 64.0, 0.0, 1.8, 64.6, 0.6);
                let hull = spin_attack_hull(old_box, new_box);
                assert_box(hull, parse_box(&c[7..13]), &format!("row {idx}"));
                let (horiz, touch) = match c[1] {
                    "NONE_CLEAR" => (false, SpinTouch::None),
                    "NONE_HORIZ" => (true, SpinTouch::None),
                    "NON_LIVING_HORIZ" => (true, SpinTouch::NonLivingOnly),
                    "LIVING_HIT" => (false, SpinTouch::Living),
                    other => panic!("unknown spin mode {other}"),
                };
                let mut spin = SpinAttack::default();
                spin.start();
                let mut vel = Vec3::new(-1.59, 1.59, 0.001);
                let _ = spin.check_auto_spin_attack(&mut vel, horiz, touch);
                assert_eq!(spin.remaining_ticks, c[2].parse::<i32>().unwrap(), "row {idx} ticks: {row}");
                assert_eq!(spin.flag_active, bit(c[3]), "row {idx} flag: {row}");
                assert_eq!(vel.x.to_bits(), h64(c[4]).to_bits(), "row {idx} vx: {row}");
                assert_eq!(vel.y.to_bits(), h64(c[5]).to_bits(), "row {idx} vy: {row}");
                assert_eq!(vel.z.to_bits(), h64(c[6]).to_bits(), "row {idx} vz: {row}");
            }
            "OFFSET_SEED" => {
                let x: i32 = c[1].parse().unwrap();
                let y: i32 = c[2].parse().unwrap();
                let z: i32 = c[3].parse().unwrap();
                let exp = u64::from_str_radix(c[4], 16).unwrap() as i64;
                assert_eq!(get_seed(x, y, z), exp, "row {idx}: {row}");
            }
            "OFFSET_BLOCK" => {
                let x: i32 = c[4].parse().unwrap();
                let z: i32 = c[5].parse().unwrap();
                let max_h = h32(c[6]);
                let off = get_xz_offset(x, z, max_h);
                assert_eq!(off.x.to_bits(), h64(c[7]).to_bits(), "row {idx} ox: {row}");
                assert_eq!(off.y.to_bits(), h64(c[8]).to_bits(), "row {idx} oy: {row}");
                assert_eq!(off.z.to_bits(), h64(c[9]).to_bits(), "row {idx} oz: {row}");
                let expected = parse_box(&c[10..16]);
                let state = StateId::of(BlockId::of(c[1]), &[
                    ("thickness", &c[2].to_ascii_lowercase()),
                    ("vertical_direction", &c[3].to_ascii_lowercase()),
                ]);
                let v = state.block_at([x, 0, z]).shape.view();
                let actual = Aabb::new(v.coord(0, 0), v.coord(1, 0), v.coord(2, 0),
                    v.coord(0, 1), v.coord(1, 1), v.coord(2, 1));
                assert_box(actual, expected, &format!("row {idx}"));
            }
            "OFFSET_BAMBOO_OUTLINE" => {
                let lv = match c[1] {
                    "NONE" => BambooLeaves::None,
                    "SMALL" => BambooLeaves::Small,
                    "LARGE" => BambooLeaves::Large,
                    other => panic!("unknown leaves {other}"),
                };
                let x: i32 = c[2].parse().unwrap();
                let z: i32 = c[3].parse().unwrap();
                assert_box(bamboo_outline_aabb([x, 0, z], lv), parse_box(&c[4..10]), &format!("row {idx}"));
            }
            "SHULKER_CONTRACT" => {
                let unit = Aabb::new(-0.5, 0.0, -0.5, 0.5, 1.0, 0.5);
                let got = aabb_contract(unit, h64(c[1]), h64(c[2]), h64(c[3]));
                assert_box(got, parse_box(&c[4..10]), &format!("row {idx}"));
            }
            "SHULKER_SHAPE" => {
                let dir = parse_dir(c[1]);
                let be = match c[2] {
                    "NULL_BE" => None,
                    "CLOSED" => Some(ShulkerBoxEntityState {
                        status: ShulkerAnimationStatus::Closed,
                        progress: h32(c[3]),
                        progress_old: h32(c[4]),
                    }),
                    "OPENED" => Some(ShulkerBoxEntityState {
                        status: ShulkerAnimationStatus::Opened,
                        progress: h32(c[3]),
                        progress_old: h32(c[4]),
                    }),
                    other => panic!("unknown shulker be {other}"),
                };
                assert_box(shulker_collision_aabb(dir, be), parse_box(&c[5..11]), &format!("row {idx}"));
            }
            "SHULKER_SUPPORT" => {
                let dir = parse_dir(c[1]);
                assert_box(shulker_open_support_aabb(dir, bit(c[2])), parse_box(&c[3..9]), &format!("row {idx}"));
            }
            "SHULKER_PUSH" => {
                let dir = parse_dir(c[1]);
                let mut be = ShulkerBoxEntityState {
                    status: ShulkerAnimationStatus::Opening,
                    progress: 0.0,
                    progress_old: 0.0,
                };
                let tick_res = be.tick_animation([0, 0, 0], dir);
                let sweep = tick_res.move_collided_sweep.unwrap();
                let delta = shulker_entity_push_vector(dir, sweep);
                assert_eq!(delta.x.to_bits(), h64(c[3]).to_bits(), "row {idx} dx: {row}");
                assert_eq!(delta.y.to_bits(), h64(c[4]).to_bits(), "row {idx} dy: {row}");
                assert_eq!(delta.z.to_bits(), h64(c[5]).to_bits(), "row {idx} dz: {row}");
            }
            "SHULKER_ANIM" => {
                let state = if c[1] == "OPEN" {
                    &mut shulker_anim_open
                } else {
                    &mut shulker_anim_close
                };
                let res = state.tick_animation([0, 0, 0], Direction::Up);
                let exp_status = match c[3] {
                    "CLOSED" => ShulkerAnimationStatus::Closed,
                    "OPENING" => ShulkerAnimationStatus::Opening,
                    "OPENED" => ShulkerAnimationStatus::Opened,
                    "CLOSING" => ShulkerAnimationStatus::Closing,
                    other => panic!("unknown anim status {other}"),
                };
                assert_eq!(state.status, exp_status, "row {idx} status: {row}");
                assert_eq!(state.progress_old.to_bits(), h32(c[4]).to_bits(), "row {idx} po: {row}");
                assert_eq!(state.progress.to_bits(), h32(c[5]).to_bits(), "row {idx} p: {row}");
                assert_eq!(res.do_neighbor_updates, bit(c[6]), "row {idx} neigh: {row}");
                assert_box(
                    shulker_collision_aabb(Direction::Up, Some(*state)),
                    parse_box(&c[7..13]),
                    &format!("row {idx}"),
                );
            }
            "SHULKER_CAN_OPEN" => {
                let mut w = BoxWorld::default();
                let mut status = ShulkerAnimationStatus::Closed;
                let mut ents = Vec::new();
                match c[1] {
                    "air_above" => {}
                    "stone_above" => w.set([0, 1, 0], Block::full()),
                    "bottom_slab_above" => w.set([0, 1, 0], Block::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)])),
                    "top_slab_above" => w.set([0, 1, 0], Block::boxes(&[Aabb::new(0.0, 0.5, 0.0, 1.0, 1.0, 1.0)])),
                    "entity_in_cell" => ents.push(Aabb::new(0.2, 1.1, 0.2, 0.8, 1.9, 0.8)),
                    "entity_top_plane" => ents.push(Aabb::new(0.2, 1.5, 0.2, 0.8, 1.9, 0.8)),
                    "opening_unconditional" => {
                        w.set([0, 1, 0], Block::full());
                        status = ShulkerAnimationStatus::Opening;
                    }
                    other => panic!("unknown can_open {other}"),
                }
                assert_eq!(shulker_can_open(&w, [0, 0, 0], Direction::Up, status, &ents), bit(c[2]), "row {idx}: {row}");
                assert_box(shulker_can_open_box([0, 0, 0], Direction::Up), parse_box(&c[3..9]), &format!("row {idx} box"));
            }
            "SHULKER_PLACE" => {
                assert_eq!(shulker_placement_facing(parse_dir(c[1])), parse_dir(c[2]), "row {idx}: {row}");
            }
            "SHULKER_BREAK" => {
                let creative = bit(c[1]);
                let empty = bit(c[2]);
                let spawns: usize = c[3].parse().unwrap();
                let exp = if creative && !empty {
                    ShulkerBreakOutcome::CreativeDropWithComponents
                } else if creative && empty {
                    ShulkerBreakOutcome::CreativeEmptyNoDrop
                } else {
                    ShulkerBreakOutcome::NormalDrop
                };
                assert_eq!(shulker_break_outcome(false, creative, empty), exp, "row {idx}: {row}");
                assert_eq!(spawns, usize::from(exp == ShulkerBreakOutcome::CreativeDropWithComponents), "row {idx} spawns: {row}");
            }
            "HOOK_PLAYER_AUDIT" => {
                assert_eq!(c[1].parse::<i32>().unwrap(), 5238, "row {idx}");
                assert_eq!(c[2].parse::<i32>().unwrap(), 0, "row {idx} player must never mutate");
            }
            "HOOK_HOPPER_SUCK" => {
                let pos = [c[1].parse().unwrap(), c[2].parse().unwrap(), c[3].parse().unwrap()];
                assert_box(hopper_suck_aabb(pos), parse_box(&c[4..10]), &format!("row {idx}"));
            }
            "HOOK_DETECTOR_BB" => {
                let pos = [c[1].parse().unwrap(), c[2].parse().unwrap(), c[3].parse().unwrap()];
                assert_box(detector_rail_search_bb(pos), parse_box(&c[4..10]), &format!("row {idx}"));
            }
            "HOOK_DETECTOR_STEP" => {
                let powered0 = bit(c[1]);
                let has_cart = bit(c[2]);
                let ev = evaluate_entity_inside(
                    NonPlayerHookBlock::DetectorRail {
                        block_id: BlockId(1),
                        powered: powered0,
                        can_survive: true,
                    },
                    HookEntityKind::Player,
                    HookContext { detector_has_minecart: has_cart, ..HookContext::default() },
                );
                assert!(!ev.player_mutated);
                match ev.action {
                    WorldHookAction::NoOp => {
                        assert!(powered0, "row {idx}");
                        assert_eq!(bit(c[3]), powered0, "row {idx}");
                        assert_eq!(c[4].parse::<i32>().unwrap(), 0, "row {idx}");
                        assert_eq!(c[5].parse::<u32>().unwrap(), 0, "row {idx}");
                        assert_eq!(c[6].parse::<i32>().unwrap(), 0, "row {idx}");
                    }
                    WorldHookAction::DetectorRailCheck(res) => {
                        assert_eq!(res.new_powered, bit(c[3]), "row {idx} powered1: {row}");
                        assert_eq!(res.set_block_flags.unwrap_or(0), c[4].parse::<i32>().unwrap(), "row {idx} flags: {row}");
                        assert_eq!(res.schedule_tick_delay.unwrap_or(0), c[5].parse::<u32>().unwrap(), "row {idx} sched: {row}");
                        assert_eq!(usize::from(res.update_comparator_output), c[6].parse::<usize>().unwrap(), "row {idx} comp: {row}");
                    }
                    other => panic!("unexpected detector action {other:?}"),
                }
            }
            "HOOK_REDSTONE_ORE" => {
                let ev = evaluate_step_on(
                    NonPlayerHookBlock::RedStoneOre { block_id: BlockId(2), lit: false },
                    HookEntityKind::Player,
                    HookContext { stepping_carefully: bit(c[1]), ..HookContext::default() },
                );
                assert!(!ev.player_mutated);
                assert_eq!(matches!(ev.action, WorldHookAction::SetRedstoneOreLit { .. }), bit(c[2]), "row {idx}: {row}");
            }
            "HOOK_LILY_PAD" => {
                let ent = if bit(c[1]) { HookEntityKind::Boat } else { HookEntityKind::Player };
                let ev = evaluate_entity_inside(NonPlayerHookBlock::LilyPad { block_id: BlockId(3) }, ent, HookContext::default());
                assert!(!ev.player_mutated);
                assert_eq!(matches!(ev.action, WorldHookAction::DestroyBlock { drop: true }), bit(c[2]), "row {idx}: {row}");
            }
            "HOOK_FROGSPAWN" => {
                let ent = if bit(c[1]) { HookEntityKind::FallingBlock } else { HookEntityKind::Player };
                let ev = evaluate_entity_inside(NonPlayerHookBlock::Frogspawn { block_id: BlockId(4) }, ent, HookContext::default());
                assert!(!ev.player_mutated);
                assert_eq!(matches!(ev.action, WorldHookAction::DestroyBlock { drop: false }), bit(c[2]), "row {idx}: {row}");
            }
            "HOOK_EYEBLOSSOM" => {
                let (attracts, ent) = match c[1].parse::<i32>().unwrap() {
                    0 => (true, HookEntityKind::Bee { has_poison: false }),
                    1 => (true, HookEntityKind::Player),
                    2 => (false, HookEntityKind::Bee { has_poison: false }),
                    other => panic!("unknown eyeblossom kind {other}"),
                };
                let ev = evaluate_entity_inside(
                    NonPlayerHookBlock::Eyeblossom { block_id: BlockId(5), attracts_bees: attracts },
                    ent,
                    HookContext::default(),
                );
                assert!(!ev.player_mutated);
                assert_eq!(matches!(ev.action, WorldHookAction::PoisonBee), bit(c[2]), "row {idx}: {row}");
            }
            "HOOK_TURTLE_CAN" => {
                let ent = match c[1] {
                    "player" | "player_nogrief" => HookEntityKind::Player,
                    "turtle" => HookEntityKind::Turtle,
                    "bat" => HookEntityKind::Bat,
                    "zombie" | "zombie_nogrief" => HookEntityKind::Zombie,
                    other => panic!("unknown turtle caller {other}"),
                };
                assert_eq!(turtle_egg_can_destroy(ent, bit(c[2])), bit(c[3]), "row {idx}: {row}");
            }
            "HOOK_CROP_RAVAGER" => {
                let blk = if c[1] == "pitcher_crop" {
                    NonPlayerHookBlock::PitcherCrop { block_id: BlockId(6) }
                } else {
                    NonPlayerHookBlock::Crop { block_id: BlockId(7) }
                };
                let ev = evaluate_entity_inside(
                    blk,
                    HookEntityKind::Ravager,
                    HookContext { mob_griefing: bit(c[2]), ..HookContext::default() },
                );
                assert!(!ev.player_mutated);
                assert_eq!(matches!(ev.action, WorldHookAction::DestroyBlock { drop: true }), bit(c[3]), "row {idx}: {row}");
            }
            "HOOK_SCULK_SENSOR" => {
                let ent = match c[1] {
                    "player" => HookEntityKind::Player,
                    "warden" => HookEntityKind::Warden,
                    "zombie" => HookEntityKind::Zombie,
                    other => panic!("unknown sculk sensor entity {other}"),
                };
                let ev = evaluate_step_on(
                    NonPlayerHookBlock::SculkSensor {
                        block_id: BlockId(8),
                        phase_inactive: true,
                        has_block_entity: true,
                        can_receive_step: true,
                    },
                    ent,
                    HookContext::default(),
                );
                assert!(!ev.player_mutated);
                assert_eq!(matches!(ev.action, WorldHookAction::SculkScheduleVibration), bit(c[2]), "row {idx}: {row}");
            }
            "HOOK_SCULK_SHRIEKER" => {
                let ent = if c[1] == "player" { HookEntityKind::Player } else { HookEntityKind::Zombie };
                let ev = evaluate_step_on(
                    NonPlayerHookBlock::SculkShrieker {
                        block_id: BlockId(9),
                        shrieking: false,
                        can_respond: true,
                        has_block_entity: true,
                    },
                    ent,
                    HookContext::default(),
                );
                assert!(!ev.player_mutated);
                assert_eq!(matches!(ev.action, WorldHookAction::SculkShriek { schedule_delay: 90, .. }), bit(c[2]), "row {idx}: {row}");
            }
            other => panic!("unknown tag {other}"),
        }
    }
    assert_eq!(count, 467);
}

#[test]
fn jar_player_elytra_damage() {
    use law::*;
    let text = include_str!("data/player_elytra_damage.tsv");
    let h32 = |s: &str| f32::from_bits(u32::from_str_radix(s, 16).unwrap());
    let h64 = |s: &str| f64::from_bits(u64::from_str_radix(s, 16).unwrap());
    let bit = |s: &str| match s { "1" => true, "0" => false, o => panic!("bit {o}") };

    let mut traj_vel: Option<(i32, Vec3)> = None;
    let mut count = 0usize;

    for (idx, line) in text.lines().enumerate() {
        let row = line.trim();
        if row.is_empty() || row.starts_with('#') { continue; }
        count += 1;
        let c: Vec<&str> = row.split('\t').collect();
        match c[0] {
            "ELYTRA_DCOS" => {
                let x = h64(c[1]);
                let expected = h64(c[2]);
                assert_eq!(elytra::jvm_dcos(x).to_bits(), expected.to_bits(), "row {idx} dcos: {row}");
            }
            "ELYTRA_TRAJ" => {
                let id: i32 = c[1].parse().unwrap();
                let tick_i: i32 = c[2].parse().unwrap();
                let gravity = h64(c[3]);
                let yaw = h32(c[4]);
                let pitch = h32(c[5]);
                let look = law::math::calculate_view_vector(yaw, pitch);
                assert_eq!(
                    (look.x.to_bits(), look.y.to_bits(), look.z.to_bits()),
                    (h64(c[6]).to_bits(), h64(c[7]).to_bits(), h64(c[8]).to_bits()),
                    "row {idx} look: {row}"
                );
                let v0 = match (id, tick_i) {
                    (_, t) if t > 0 && traj_vel.is_some_and(|(prev_id, _)| prev_id == id) => traj_vel.unwrap().1,
                    (1, 0) => Vec3::new(0.3f32 as f64, -0.2f32 as f64, 0.1f32 as f64),
                    (2, 0) => Vec3::new(0.5f32 as f64, -0.9f32 as f64, 0.0f32 as f64),
                    (3, 0) => Vec3::new(1.2f32 as f64, -0.05f32 as f64, 0.0f32 as f64),
                    (4, 0) => Vec3::new(0.2f32 as f64, -1.1f32 as f64, 0.05f32 as f64),
                    (5, 0) => Vec3::new(0.2f32 as f64, -0.05f32 as f64, 0.0f32 as f64),
                    (6, 0) => Vec3::new(0.01f32 as f64, -0.9f32 as f64, 0.0f32 as f64),
                    (7, 0) => Vec3::new(0.0f32 as f64, -2.0f32 as f64, 0.0f32 as f64),
                    (8, 0) => Vec3::new(1.0f32 as f64, -0.2f32 as f64, 1.0f32 as f64),
                    _ => unreachable!(),
                };
                let updated = elytra::update_fall_flying_movement(v0, look, pitch, gravity);
                assert_eq!(
                    (updated.x.to_bits(), updated.y.to_bits(), updated.z.to_bits()),
                    (h64(c[9]).to_bits(), h64(c[10]).to_bits(), h64(c[11]).to_bits()),
                    "row {idx} update_fall_flying_movement: {row}"
                );
                if v0.horizontal_sqr() >= 9.0e-6 && v0.y.abs() >= 0.003 {
                    let mut s = PlayerState { position: Vec3::new(0.5, 1000.0, 0.5),
                        velocity: v0,
                        yaw,
                        pitch,
                        pose: Pose::FallFlying,
                        fall_flying: true,
                        effects: gravity == 0.01, profile: law::Profile { gravity, slow_falling: gravity == 0.01, ..Default::default() }.into(), ..PlayerState::default() };
                    tick(&mut s, &BoxWorld::default(), &Input { yaw, pitch, ..Input::default() });
                    assert_eq!(s.pose, Pose::FallFlying, "row {idx} pose: {row}");
                    assert_eq!(
                        (s.velocity.x.to_bits(), s.velocity.y.to_bits(), s.velocity.z.to_bits()),
                        (h64(c[9]).to_bits(), h64(c[10]).to_bits(), h64(c[11]).to_bits()),
                        "row {idx} tick vel: {row}"
                    );
                }
                traj_vel = Some((id, updated));
            }
            "ELYTRA_WALL" => {
                let pre_h = h64(c[1]);
                let post_h = h64(c[2]);
                let hit = bit(c[3]);
                let amt = h32(c[4]);
                let got = elytra::fly_into_wall_damage(true, pre_h, post_h);
                assert_eq!(got.is_some(), hit, "row {idx} wall hit: {row}");
                assert_eq!(got.unwrap_or(0.0).to_bits(), amt.to_bits(), "row {idx} wall amt: {row}");
            }
            "ELYTRA_FALL_ACC" => {
                let vy = h64(c[1]);
                let fd = h64(c[2]);
                assert_eq!(elytra::check_fall_distance_accumulation(vy, fd).to_bits(), h64(c[3]).to_bits(), "row {idx} fall acc: {row}");
            }
            "EFFECT_LEV" => {
                let amp: u8 = c[1].parse().unwrap();
                let steps: i32 = c[2].parse().unwrap();
                let mut s = PlayerState {
                    position: Vec3::new(0.5, 64.0, 0.5),
                    fall_distance: 5.0,
                    effects: true,
                    ..PlayerState::default()
                };
                s.set_levitation(Some(amp));
                for _ in 0..steps {
                    tick(&mut s, &BoxWorld::default(), &Input::default());
                }
                assert_eq!(s.velocity.y.to_bits(), h64(c[3]).to_bits(), "row {idx} lev vy: {row}");
                assert_eq!(s.fall_distance, 0.0, "row {idx} lev fd: {row}");
            }
            "EFFECT_SLOWFALL_G" => {
                let vy = h64(c[1]);
                let expected = h64(c[2]);
                let g = if vy <= 0.0 { 0.08f64.min(0.01) } else { 0.08 };
                assert_eq!(g.to_bits(), expected.to_bits(), "row {idx} sf g: {row}");
            }
            "EFFECT_SLOWFALL_AIR" => {
                let vy0 = h64(c[1]);
                let steps: i32 = c[2].parse().unwrap();
                let mut s = PlayerState {
                    position: Vec3::new(0.5, 100.0, 0.5),
                    velocity: Vec3::new(0.0, vy0, 0.0),
                    fall_distance: 4.0,
                    effects: true,
                    ..PlayerState::default()
                };
                s.set_slow_falling(true);
                for _ in 0..steps {
                    let prev_vy = s.velocity.y;
                    tick(&mut s, &BoxWorld::default(), &Input::default());
                    // aiStep @537-558 resets fall_distance to 0 before travel(), so move() only holds this tick's downward dy.
                    let expected_fd = if prev_vy < 0.0 { (-(prev_vy as f32)) as f64 } else { 0.0 };
                    assert_eq!(s.fall_distance.to_bits(), expected_fd.to_bits(), "row {idx} sf air fd: {row}");
                }
                assert_eq!(s.velocity.y.to_bits(), h64(c[3]).to_bits(), "row {idx} sf air vy: {row}");
            }
            "EFFECT_SLOWFALL_CLIMB" => {
                let mut w = BoxWorld::default();
                w.set([0, 63, 0], Block { air: false, climbable: true, ..Block::default() });
                w.set([0, 64, 0], Block { air: false, climbable: true, ..Block::default() });
                w.set([1, 64, 0], Block::full());
                let mut s = PlayerState {
                    position: Vec3::new(0.7, 64.0, 0.5),
                    velocity: Vec3::new(0.2, -0.2, 0.0),
                    effects: true,
                    ..PlayerState::default()
                };
                s.set_slow_falling(true);
                tick(&mut s, &w, &Input::default());
                assert_eq!(s.velocity.y.to_bits(), h64(c[1]).to_bits(), "row {idx} sf climb vy: {row}");
            }
            "EFFECT_DOLPHIN" => {
                let dg = bit(c[1]);
                let sprint = bit(c[2]);
                let on_ground = bit(c[3]);
                let eff_val = h32(c[4]);
                let mut w = BoxWorld::default();
                let water_block = Block {
                    air: false,
                    inside_shape: Some(VoxelShape::default()),
                    fluid: Fluid { kind: FluidKind::Water, height: if sprint { 1.0 } else { 0.9 }, ..Fluid::default() },
                    ..Block::default()
                };
                w.set([0, 64, 0], water_block.clone());
                if sprint {
                    w.set([0, 65, 0], water_block);
                }
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    velocity: Vec3::new(0.5, -0.1, 0.0),
                    on_ground,
                    in_water: true,
                    underwater: sprint,
                    eye_in_water: sprint,
                    water_height: if sprint { 1.0 } else { 0.9 },
                    effects: dg, profile: law::Profile { water_movement_efficiency: eff_val, ..Default::default() }.into(), ..PlayerState::default() };
                s.set_sprinting(sprint);
                s.set_dolphins_grace(dg);
                tick(&mut s, &w, &Input { forward: sprint, sprint, ..Input::default() });
                assert_eq!(s.sprinting, sprint, "row {idx} sprint preserved: {row}");
                assert_eq!(s.velocity.x.to_bits(), h64(c[5]).to_bits(), "row {idx} dolphin vx: {row}");
                if !sprint {
                    assert_eq!(
                        (s.velocity.y.to_bits(), s.velocity.z.to_bits()),
                        (h64(c[6]).to_bits(), h64(c[7]).to_bits()),
                        "row {idx} dolphin vy/vz: {row}"
                    );
                }
            }
            "EFFECT_SLOWFALL_FLUID" => {
                let kind = c[1];
                let mut w = BoxWorld::default();
                let mut s = PlayerState {
                    position: Vec3::new(0.5, 64.0, 0.5),
                    velocity: Vec3::new(0.0, -0.01, 0.0),
                    effects: true,
                    ..PlayerState::default()
                };
                s.set_slow_falling(true);
                match kind {
                    "water" => {
                        w.set([0, 64, 0], Block {
                            air: false,
                            inside_shape: Some(VoxelShape::default()),
                            fluid: Fluid { kind: FluidKind::Water, height: 0.9, ..Fluid::default() },
                            ..Block::default()
                        });
                        s.in_water = true;
                        s.water_height = 0.9;
                    }
                    "lava" => {
                        w.set([0, 64, 0], Block {
                            air: false,
                            inside_shape: Some(VoxelShape::default()),
                            fluid: Fluid { kind: FluidKind::Lava, height: 0.25, ..Fluid::default() },
                            ..Block::default()
                        });
                        s.in_lava = true;
                        s.lava_height = 0.25;
                    }
                    other => panic!("fluid kind {other}"),
                }
                tick(&mut s, &w, &Input::default());
                assert_eq!(s.velocity.y.to_bits(), h64(c[2]).to_bits(), "row {idx} sf fluid: {row}");
            }
            "EFFECT_WEAVING" => {
                let weaving = bit(c[1]);
                let mut w = BoxWorld::default();
                w.set([0, 64, 0], Block { air: false, inside: Inside::Cobweb, ..Block::default() });
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    effects: weaving, profile: law::Profile { gravity: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                s.set_weaving(weaving);
                tick(&mut s, &w, &Input::default());
                assert_eq!(
                    (s.stuck_speed_multiplier.x.to_bits(), s.stuck_speed_multiplier.y.to_bits(), s.stuck_speed_multiplier.z.to_bits()),
                    (h64(c[2]).to_bits(), h64(c[3]).to_bits(), h64(c[4]).to_bits()),
                    "row {idx} weaving: {row}"
                );
            }
            "ENCH_MOV" => {
                let lvl: u8 = c[1].parse().unwrap();
                let mut s = PlayerState::default();
                s.set_movement_enchantments(lvl, lvl, lvl);
                s.apply_soul_speed_location(lvl, true);
                let spd = PlayerState::soul_speed_modifier_for(lvl).unwrap();
                assert_eq!(spd.amount.to_bits(), h64(c[2]).to_bits(), "row {idx} ss mod: {row}");
                assert_eq!(s.movement_speed_value().to_bits(), h64(c[3]).to_bits(), "row {idx} ss total: {row}");
                // Check soul sand 0.4 slowdown cancelled via movement_efficiency = 1.0 during real tick
                let mut w = BoxWorld::default();
                w.set([0, 63, 0], Block { speed_factor: 0.4, ..Block::full() });
                s.position = Vec3::new(0.5, 64.0, 0.5);
                s.on_ground = true;
                s.velocity = Vec3::new(0.2, 0.0, 0.0);
                let mut s_plain = PlayerState { position: s.position, on_ground: true, velocity: s.velocity, ..PlayerState::default() };
                tick(&mut s, &w, &Input::default());
                tick(&mut s_plain, &w, &Input::default());
                assert_eq!(h32(c[4]).to_bits(), 1.0f32.to_bits());
                assert!(s.velocity.x > s_plain.velocity.x);
                assert_eq!(s.profile.water_movement_efficiency.to_bits(), h32(c[5]).to_bits(), "row {idx} ds: {row}");
                assert_eq!(s.profile.sneaking_speed.to_bits(), h32(c[6]).to_bits(), "row {idx} sneak: {row}");
            }
            "ENCH_EPF" => {
                assert_eq!(PlayerState::protection_epf_for(4).to_bits(), h32(c[1]).to_bits(), "row {idx} prot: {row}");
                assert_eq!(PlayerState::feather_falling_epf_for(4).to_bits(), h32(c[2]).to_bits(), "row {idx} fef: {row}");
            }
            "MITIGATION" => {
                let name = c[1];
                let allowed = bit(c[3]);
                let exp_hp = h32(c[4]);
                let exp_abs = h32(c[5]);
                let exp_exh = h32(c[6]);
                let (raw, is_fire, bypass_armor, is_fall, exh, arm, tgh, res, prot, fef, abs0, fres) = match name {
                    "fall_kit" => (10.0f32, false, true, true, 0.0f32, 20.0f32, 8.0f32, None, 16.0f32, 12.0f32, 4.0f32, false),
                    "fall_kit_no_abs" => (10.0, false, true, true, 0.0, 20.0, 8.0, None, 16.0, 12.0, 0.0, false),
                    "pearl_kit" => (5.0, false, true, true, 0.0, 20.0, 8.0, None, 16.0, 12.0, 4.0, false),
                    "lava_kit" => (4.0, true, false, false, 0.1, 20.0, 8.0, None, 16.0, 12.0, 4.0, false),
                    "lava_bare_abs0" => (4.0, true, false, false, 0.1, 20.0, 8.0, None, 16.0, 0.0, 0.0, false),
                    "fall_res0" => (10.0, false, true, true, 0.0, 0.0, 0.0, Some(0), 0.0, 0.0, 0.0, false),
                    "fall_res1" => (10.0, false, true, true, 0.0, 0.0, 0.0, Some(1), 0.0, 0.0, 0.0, false),
                    "fall_res2" => (10.0, false, true, true, 0.0, 0.0, 0.0, Some(2), 0.0, 0.0, 0.0, false),
                    "fall_res3" => (10.0, false, true, true, 0.0, 0.0, 0.0, Some(3), 0.0, 0.0, 0.0, false),
                    "fall_res4" => (10.0, false, true, true, 0.0, 0.0, 0.0, Some(4), 0.0, 0.0, 0.0, false),
                    "lava_armor_res1_epf16" => (4.0, true, false, false, 0.1, 20.0, 8.0, Some(1), 16.0, 0.0, 0.0, false),
                    "on_fire_res0" => (1.0, true, true, false, 0.0, 20.0, 8.0, Some(0), 16.0, 0.0, 0.0, false),
                    "in_fire_armor_epf" => (2.0, true, false, false, 0.1, 20.0, 8.0, None, 16.0, 0.0, 0.0, false),
                    "campfire_armor_epf" => (2.0, true, false, false, 0.1, 20.0, 8.0, None, 16.0, 0.0, 0.0, false),
                    "berry_armor" => (1.0, false, false, false, 0.1, 20.0, 8.0, None, 0.0, 0.0, 0.0, false),
                    "cactus_armor_res0" => (1.0, false, false, false, 0.1, 20.0, 8.0, Some(0), 0.0, 0.0, 0.0, false),
                    "stalagmite_kit" => (15.0, false, true, true, 0.0, 20.0, 8.0, Some(0), 16.0, 12.0, 4.0, false),
                    "fly_into_wall_kit" => (6.5, false, true, false, 0.0, 20.0, 8.0, Some(0), 16.0, 12.0, 0.0, false),
                    "lava_fireres" => (4.0, true, false, false, 0.1, 0.0, 0.0, None, 0.0, 0.0, 0.0, true),
                    "onfire_fireres" => (1.0, true, true, false, 0.0, 0.0, 0.0, None, 0.0, 0.0, 0.0, true),
                    o => panic!("unknown mit case {o}"),
                };
                let mut ent = PlayerState { absorption: abs0, profile: law::Profile { armor: arm, armor_toughness: tgh, resistance: res,
                    protection_epf: prot, feather_falling_epf: fef, fire_resistance: fres, ..Default::default() }.into(), ..PlayerState::default() };
                let ok = law::core::hurt(&mut ent, law::core::Hit { amount: raw, fire: is_fire, armor: !bypass_armor, effects: true, fall: is_fall, exhaustion: exh });
                assert_eq!(ok, allowed, "row {idx} allowed: {row}");
                assert_eq!(
                    (ent.health.to_bits(), ent.absorption.to_bits(), ent.exhaustion.to_bits()),
                    (exp_hp.to_bits(), exp_abs.to_bits(), exp_exh.to_bits()),
                    "row {idx} mit {name}: {row}"
                );
                // Verify PlayerState wired through UnmodifiedContactDamage / tick_with_contact on contact variants
                let contact_req = match name {
                    "on_fire_res0" | "onfire_fireres" => Some(ContactDamage::OnFire),
                    "in_fire_armor_epf" => Some(ContactDamage::InFire { soul: true }),
                    "campfire_armor_epf" => Some(ContactDamage::Campfire { soul: true }),
                    "berry_armor" => Some(ContactDamage::SweetBerryBush),
                    "cactus_armor_res0" => Some(ContactDamage::Cactus),
                    _ => None,
                };
                if let Some(req) = contact_req {
                    let mut ps = PlayerState { absorption: abs0,
                        effects: res.is_some() || fres, profile: law::Profile { armor: arm, armor_toughness: tgh, resistance: res, protection_epf: prot, feather_falling_epf: fef, fire_resistance: fres, ..Default::default() }.into(), ..PlayerState::default() };
                    apply_server_resource_event(&mut ps, &BoxWorld::default(), ServerResourceEvent::UnmodifiedContactDamage(req));
                    assert_eq!(
                        (ps.health.to_bits(), ps.absorption.to_bits(), ps.exhaustion.to_bits()),
                        (exp_hp.to_bits(), exp_abs.to_bits(), exp_exh.to_bits()),
                        "row {idx} ServerResourceEvent {name}: {row}"
                    );
                }
            }
            "FIRE_BASETICK" => {
                let init: i32 = c[1].parse().unwrap();
                let exp_ticks: i32 = c[2].parse().unwrap();
                let exp_src = c[3];
                let mut s = PlayerState { remaining_fire_ticks: init, ..PlayerState::default() };
                let contact = tick_with_contact(&mut s, &BoxWorld::default(), &Input::default());
                assert_eq!(s.remaining_fire_ticks, exp_ticks, "row {idx} fire ticks: {row}");
                assert_eq!(contact == Some(ContactDamage::OnFire), exp_src == "onFire", "row {idx} onFire: {row}");
            }
            "FIRE_CLEAR" => {
                let init: i32 = c[1].parse().unwrap();
                let exp: i32 = c[2].parse().unwrap();
                assert_eq!(player::clear_fire_ticks(init), exp, "row {idx} clearFire: {row}");
            }
            "FIRE_IGNITE" => {
                let init: i32 = c[1].parse().unwrap();
                let exp: i32 = c[2].parse().unwrap();
                assert_eq!(player::base_fire_ignite_ticks(init, false, false), exp, "row {idx} fireIgnite: {row}");
            }
            "FIRE_INSIDE" => {
                let soul = bit(c[1]);
                let exp_ticks: i32 = c[2].parse().unwrap();
                let exp_dmg = h32(c[4]);
                let mut w = BoxWorld::default();
                w.set([0, 64, 0], Block { air: false, inside: Inside::Fire { soul }, ..Block::default() });
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    remaining_fire_ticks: -1, profile: law::Profile { gravity: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                let contact = tick_with_contact(&mut s, &w, &Input::default()).unwrap();
                assert_eq!(s.remaining_fire_ticks, exp_ticks, "row {idx} fire inside ticks: {row}");
                assert_eq!(contact, ContactDamage::InFire { soul }, "row {idx} fire inside req: {row}");
                assert_eq!(contact.raw_amount().to_bits(), exp_dmg.to_bits(), "row {idx} fire inside dmg: {row}");
            }
            "CAMPFIRE_INSIDE" => {
                let soul = bit(c[1]);
                let lit = bit(c[2]);
                let exp_ticks: i32 = c[3].parse().unwrap();
                let exp_src = c[4];
                let exp_dmg = h32(c[5]);
                let mut w = BoxWorld::default();
                w.set([0, 64, 0], Block {
                    air: false,
                    inside: if lit { Inside::Campfire { soul } } else { Inside::None },
                    ..Block::default()
                });
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    remaining_fire_ticks: -20, profile: law::Profile { gravity: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                let contact = tick_with_contact(&mut s, &w, &Input::default());
                assert_eq!(s.remaining_fire_ticks, exp_ticks, "row {idx} campfire ticks: {row}");
                if exp_src == "none" {
                    assert_eq!(contact, None, "row {idx} unlit campfire: {row}");
                } else {
                    let req = contact.unwrap();
                    assert_eq!(req, ContactDamage::Campfire { soul }, "row {idx} campfire req: {row}");
                    assert_eq!(req.raw_amount().to_bits(), exp_dmg.to_bits(), "row {idx} campfire dmg: {row}");
                }
            }
            "FIRE_TAIL" => {
                let before: i32 = c[1].parse().unwrap();
                let is_rain = bit(c[2]);
                let exp: i32 = c[3].parse().unwrap();
                let after = if is_rain { player::clear_fire_ticks(before) } else { before };
                assert_eq!(player::apply_effects_fire_tail(after, before, true, false, true), exp, "row {idx} fire tail: {row}");
            }
            "HURT_CACTUS" => {
                let exp_dmg = h32(c[2]);
                let mut w = BoxWorld::default();
                w.set([0, 64, 0], Block {
                    shape: VoxelShape::boxes(&[Aabb::new(0.0625, 0.0, 0.0625, 0.9375, 0.9375, 0.9375)]),
                    inside_shape: None,
                    inside: Inside::Cactus,
                    air: false,
                    ..Block::default()
                });
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5), profile: law::Profile { gravity: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                let req = tick_with_contact(&mut s, &w, &Input::default()).unwrap();
                assert_eq!(req, ContactDamage::Cactus, "row {idx} cactus: {row}");
                assert_eq!(req.raw_amount().to_bits(), exp_dmg.to_bits(), "row {idx} cactus dmg: {row}");
            }
            "HURT_BERRY" => {
                let age: i32 = c[1].parse().unwrap();
                let dx = h64(c[2]);
                let dz = h64(c[3]);
                let exp_sx = h64(c[4]);
                let exp_sy = h64(c[5]);
                let exp_sz = h64(c[6]);
                let exp_src = c[7];
                let exp_dmg = h32(c[8]);
                let mut w = BoxWorld::default();
                w.set([0, 64, 0], Block {
                    air: false,
                    inside: Inside::BerryBush { hurt: age > 0 },
                    ..Block::default()
                });
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5),
                    velocity: Vec3::new(dx, 0.0, dz), profile: law::Profile { gravity: 0.0, ..Default::default() }.into(), ..PlayerState::default() };
                let contact = tick_with_contact(&mut s, &w, &Input::default());
                assert_eq!(
                    (s.stuck_speed_multiplier.x.to_bits(), s.stuck_speed_multiplier.y.to_bits(), s.stuck_speed_multiplier.z.to_bits()),
                    (exp_sx.to_bits(), exp_sy.to_bits(), exp_sz.to_bits()),
                    "row {idx} berry stuck: {row}"
                );
                if exp_src == "none" {
                    assert_eq!(contact, None, "row {idx} berry no hurt: {row}");
                } else {
                    let req = contact.unwrap();
                    assert_eq!(req, ContactDamage::SweetBerryBush, "row {idx} berry req: {row}");
                    assert_eq!(req.raw_amount().to_bits(), exp_dmg.to_bits(), "row {idx} berry dmg: {row}");
                }
            }
            "FALL_FARMLAND" => {
                let w = h32(c[1]);
                let h = h32(c[2]);
                let fd = h64(c[3]);
                let rf = h32(c[4]);
                let exp_trampled = bit(c[5]);
                let exp_calc_dmg: i32 = c[6].parse().unwrap();
                let exp_hp = h32(c[7]);
                assert_eq!(player::farmland_trample_threshold(w, h, fd, rf), exp_trampled, "row {idx} farmland trample: {row}");
                let s = PlayerState::default();
                let calc_dmg = calculate_fall_damage(&s, fd, 1.0);
                assert_eq!(calc_dmg, exp_calc_dmg, "row {idx} farmland calc dmg: {row}");
                let mut ent = PlayerState::default();
                if calc_dmg > 0 { law::core::hurt(&mut ent, law::core::Hit { amount: calc_dmg as f32, ..law::core::Hit::FALL }); }
                assert_eq!(ent.health.to_bits(), exp_hp.to_bits(), "row {idx} farmland hp: {row}");
            }
            "FALL_DRIPSTONE" | "FALL_DRIPSTONE_HP" => {
                // The landing priced through the block's own row: pointed_dripstone[thickness, vertical_direction].
                let block = world::model(&world::BlockState { name: "minecraft:pointed_dripstone".into(), properties: vec![
                    ("thickness".into(), c[1].into()), ("vertical_direction".into(), c[2].into()), ("waterlogged".into(), "false".into())] }).unwrap();
                let (d, m) = law::core::fall_on(block, 5.0);
                if c[0] == "FALL_DRIPSTONE" {
                    let (exp_dist, exp_mult, exp_stal) = (h64(c[3]), h32(c[4]), c[5] == "stalagmite");
                    assert_eq!((d.to_bits(), m.to_bits(), m == 2.0), (exp_dist.to_bits(), exp_mult.to_bits(), exp_stal), "row {idx} dripstone: {row}");
                } else {
                    let calc_dmg = calculate_fall_damage(&PlayerState::default(), d, m).max(0);
                    let mut ent = PlayerState::default();
                    if calc_dmg > 0 { law::core::hurt(&mut ent, law::core::Hit { amount: calc_dmg as f32, ..law::core::Hit::FALL }); }
                    assert_eq!(ent.health.to_bits(), h32(c[4]).to_bits(), "row {idx} dripstone hp: {row}");
                }
            }
            other => panic!("unknown tag {other}"),
        }
    }
    assert_eq!(count, 280);
}

#[test]
fn jar_cauldron_and_redstone() {
    use law::basis::cell::{BlockId, StateId};
    use law::{block_hooks::*, player, redstone::*, tick, try_tick, BoxWorld, Block, Input, Inside, PlayerState, Vec3, VoxelShape};

    let h64 = |s: &str| -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) };
    let bit = |s: &str| -> bool {
        match s {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => panic!("bad bit {s}"),
        }
    };
    let parse_shape = |raw: &str| -> VoxelShape {
        let boxes: Vec<Aabb> = raw
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|b| {
                let c: Vec<f64> = b.split(',').map(&h64).collect();
                assert_eq!(c.len(), 6);
                Aabb::new(c[0], c[1], c[2], c[3], c[4], c[5])
            })
            .collect();
        VoxelShape::boxes(&boxes)
    };
    let assert_shape_eq = |actual: &VoxelShape, expected: &VoxelShape, ctx: &str| {
        for axis in 0..3 {
            let a_bits: Vec<u64> = actual.coords[axis].iter().map(|v| v.to_bits()).collect();
            let e_bits: Vec<u64> = expected.coords[axis].iter().map(|v| v.to_bits()).collect();
            assert_eq!(a_bits, e_bits, "{ctx} axis {axis}");
        }
        let n = |a: usize| expected.coords[a].len().saturating_sub(1);
        for x in 0..n(0) {
            for y in 0..n(1) {
                for z in 0..n(2) {
                    assert_eq!(actual.is_full([x, y, z]), expected.is_full([x, y, z]), "{ctx} cell [{x},{y},{z}]");
                }
            }
        }
    };
    let parse_sched = |s: &str| -> Option<(i32, TickPriority)> {
        if s == "none" || s == "-1" {
            return None;
        }
        let (d, p) = s.split_once(':').unwrap();
        let prio = match p.parse::<i32>().unwrap() {
            -3 => TickPriority::ExtremelyHigh,
            -2 => TickPriority::VeryHigh,
            -1 => TickPriority::High,
            0 => TickPriority::Normal,
            other => panic!("unexpected priority {other}"),
        };
        Some((d.parse().unwrap(), prio))
    };
    let parse_tilt = |s: &str| -> BigDripleafTilt {
        match s {
            "none" => BigDripleafTilt::None,
            "unstable" => BigDripleafTilt::Unstable,
            "partial" => BigDripleafTilt::Partial,
            "full" => BigDripleafTilt::Full,
            other => panic!("unexpected tilt {other}"),
        }
    };

    let mut count = 0usize;
    for (idx, row) in include_str!("data/cauldron_redstone.tsv").lines().enumerate() {
        if row.is_empty() {
            continue;
        }
        count += 1;
        let c: Vec<&str> = row.split('\t').collect();
        match c[0] {
            "CAULDRON_HEIGHT" => {
                let name = c[1];
                let lvl: u8 = c[2].parse().unwrap();
                let exp = h64(c[3]);
                let got = if name == "lava_cauldron" {
                    LAVA_CAULDRON_CONTENT_HEIGHT
                } else {
                    layered_cauldron_content_height(lvl)
                };
                assert_eq!(got.to_bits(), exp.to_bits(), "row {idx}: {row}");
            }
            "CAULDRON_SHAPE" => {
                let expected = parse_shape(c[3]);
                let state = StateId::of(BlockId::of(c[1]), &[("level", c[2])]);
                assert_shape_eq(state.model_block().inside_shape.as_ref().unwrap(), &expected, row);
            }
            "CAULDRON_INSIDE" => {
                let kind = if c[1] == "powder_snow_cauldron" {
                    CauldronKind::PowderSnow
                } else {
                    CauldronKind::Water
                };
                let lvl: u8 = c[2].parse().unwrap();
                let on_fire = bit(c[3]);
                let may_interact = bit(c[4]);
                assert_eq!(c[5], "EXTINGUISH", "row {idx}: {row}");
                let exp_block = c[6];
                let exp_lvl: u8 = c[7].parse().unwrap();
                let expected = match exp_block {
                    "cauldron" => CauldronFireInsideResult::EmptyCauldron,
                    "water_cauldron" if on_fire && may_interact => {
                        CauldronFireInsideResult::SetWaterLevel { level: exp_lvl }
                    }
                    _ => CauldronFireInsideResult::NoChange,
                };
                assert_eq!(
                    layered_cauldron_fire_inside(true, on_fire, may_interact, kind, lvl),
                    expected,
                    "row {idx}: {row}"
                );
                let mut w = BoxWorld::default();
                w.set(
                    [0, 64, 0],
                    StateId::of(BlockId::of(c[1]), &[("level", c[2])]).model_block().clone(),
                );
                let mut s = PlayerState {
                    position: Vec3::new(0.5, 64.25, 0.5),
                    remaining_fire_ticks: if on_fire { 100 } else { -5 },
                    ..PlayerState::default()
                };
                assert_eq!(player::clear_fire_ticks(s.remaining_fire_ticks), if on_fire { 0 } else { -5 }, "row {idx} clearFire: {row}");
                tick(&mut s, &w, &Input::default());
                assert_eq!(s.remaining_fire_ticks, if on_fire { 0 } else { -20 }, "row {idx} extinguish: {row}");
                assert_eq!(s.ticks_frozen, 0, "row {idx} no freeze: {row}");
            }
            "LAVA_CAULDRON_INSIDE" => {
                assert_eq!(c[1], "apply:CLEAR_FREEZE;apply:LAVA_IGNITE;after:LAVA_IGNITE");
                // Verify player movement execution matches LavaFluid collector semantics on lava_cauldron shape.
                let mut w = BoxWorld::default();
                w.set(
                    [0, 64, 0],
                    BlockId::of("lava_cauldron").default_state().model_block().clone(),
                );
                let mut s = PlayerState {
                    position: Vec3::new(0.5, 64.25, 0.5),
                    ticks_frozen: 50,
                    remaining_fire_ticks: 10,
                    ..PlayerState::default()
                };
                tick(&mut s, &w, &Input::default());
                assert_eq!(s.ticks_frozen, 0);
                assert_eq!(s.remaining_fire_ticks, 300);
                assert_eq!(s.health.to_bits(), 16.0f32.to_bits());

                // Longer fire duration is never shortened by LavaIgnite.
                let mut s_hot = PlayerState { position: Vec3::new(0.5, 64.25, 0.5),
                    remaining_fire_ticks: 500,
                    effects: true, profile: law::Profile { fire_resistance: true, ..Default::default() }.into(), ..PlayerState::default() };
                tick(&mut s_hot, &w, &Input::default());
                // Entity.baseTick decrements 500 -> 499; LavaIgnite never shortens a longer fire.
                assert_eq!(s_hot.remaining_fire_ticks, 499);
                assert_eq!(s_hot.health.to_bits(), 20.0f32.to_bits());
            }
            "BIG_DRIPLEAF_META" => {
                let tilt = parse_tilt(c[1]);
                let exp_delay: i32 = c[2].parse().unwrap();
                let exp_vib = bit(c[3]);
                let expected_shape = parse_shape(c[4]);
                assert_eq!(tilt.delay_until_next_state(), exp_delay, "row {idx}: {row}");
                assert_eq!(tilt.causes_vibration(), exp_vib, "row {idx}: {row}");
                let state = StateId::of(BlockId::of("big_dripleaf"), &[("tilt", c[1])]);
                assert_shape_eq(&state.model_block().shape, &expected_shape, row);
            }
            "BIG_DRIPLEAF_INSIDE" => {
                let by: i32 = c[1].parse().unwrap();
                let y = h64(c[2]);
                let on_ground = bit(c[3]);
                let has_sig = bit(c[4]);
                let is_client = bit(c[5]);
                let exp_tilt = parse_tilt(c[6]);
                let exp_sched = parse_sched(c[7]);
                let got = big_dripleaf_entity_inside(is_client, BigDripleafTilt::None, has_sig, on_ground, y, by);
                if exp_tilt == BigDripleafTilt::None {
                    assert_eq!(got, None, "row {idx}: {row}");
                } else {
                    let tr = got.expect("expected tilt transition");
                    assert_eq!(tr.tilt, exp_tilt, "row {idx}: {row}");
                    assert_eq!(tr.schedule_delay, exp_sched.map(|(d, _)| d), "row {idx}: {row}");
                }
            }
            "BIG_DRIPLEAF_TICK" => {
                let tilt = parse_tilt(c[1]);
                let has_sig = bit(c[2]);
                let exp_tilt = parse_tilt(c[3]);
                let exp_delay = parse_sched(c[4]).map(|(d, _)| d);
                let tr = big_dripleaf_tick(tilt, has_sig);
                assert_eq!(tr.tilt, exp_tilt, "row {idx}: {row}");
                assert_eq!(tr.schedule_delay, exp_delay, "row {idx}: {row}");
            }
            "WITHER_ROSE_EFFECT" => {
                assert_eq!(c[1].parse::<i32>().unwrap(), WITHER_ROSE_EFFECT_DURATION_TICKS);
                assert_eq!(c[2].parse::<i32>().unwrap(), WITHER_ROSE_EFFECT_AMPLIFIER);
            }
            "WITHER_ROSE_GATE" => {
                let diff: u8 = c[1].parse().unwrap();
                let invuln = bit(c[2]);
                let exp_applied = bit(c[3]);
                assert_eq!(
                    wither_rose_applies_effect(true, diff, true, invuln),
                    exp_applied,
                    "row {idx}: {row}"
                );
                let mut w = BoxWorld::default();
                w.difficulty = diff;
                w.set(
                    [0, 64, 0],
                    Block {
                        air: false,
                        inside: Inside::WitherRose,
                        inside_shape: Some(VoxelShape::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)])),
                        ..Block::default()
                    },
                );
                let mut s = PlayerState { position: Vec3::new(0.5, 64.0, 0.5), profile: law::Profile { invulnerable: invuln, ..Default::default() }.into(), ..PlayerState::default() };
                try_tick(&mut s, &w, &Input::default()).expect("a wither rose is a row of the effect book");
                let book: Vec<_> = s.effect_book.iter().map(|e| (e.kind, e.duration, e.amplifier)).collect();
                let wither = vec![(law::EffectKind::Wither, WITHER_ROSE_EFFECT_DURATION_TICKS, WITHER_ROSE_EFFECT_AMPLIFIER)];
                assert_eq!(book == wither, exp_applied, "row {idx} wither tick gate: {row}");
            }
            "WITHER_EFFECT_TICK" => {
                let dur: i32 = c[1].parse().unwrap();
                let amp: i32 = c[2].parse().unwrap();
                let exp = bit(c[3]);
                assert_eq!(player::wither_should_apply_effect_tick(dur, amp), exp, "row {idx}: {row}");
            }
            "PORTAL" => {
                let name = c[1];
                let exp_tt: i32 = c[2].parse().unwrap();
                assert_eq!(exp_tt, END_PORTAL_TRANSITION_TICKS, "row {idx}: {row}");
                let exp_shape = parse_shape(c[3]);
                let got_shape = if name == "end_portal" {
                    VoxelShape::boxes(&[end_portal_inside_aabb()])
                } else {
                    Block::full().shape
                };
                assert_shape_eq(&got_shape, &exp_shape, row);
                if name == "end_portal" {
                    // Feet above y=0.75 do not touch end_portal's [6/16..12/16] trigger volume.
                    let mut w = BoxWorld::default();
                    w.set(
                        [0, 64, 0],
                        Block {
                            air: false,
                            inside: Inside::EndPortal,
                            inside_shape: Some(got_shape),
                            end_portal_or_gateway: true,
                            ..Block::default()
                        },
                    );
                    let mut high = PlayerState {
                        position: Vec3::new(0.5, 64.8, 0.5),
                        ..PlayerState::default()
                    };
                    assert!(try_tick(&mut high, &w, &Input::default()).is_ok());
                    let mut in_portal = PlayerState {
                        position: Vec3::new(0.5, 64.5, 0.5),
                        ..PlayerState::default()
                    };
                    assert!(try_tick(&mut in_portal, &w, &Input::default()).is_err());
                }
            }
            "TICK_PRIORITY" => {
                let exp: i32 = c[2].parse().unwrap();
                let p = match c[1] {
                    "EXTREMELY_HIGH" => TickPriority::ExtremelyHigh,
                    "VERY_HIGH" => TickPriority::VeryHigh,
                    "HIGH" => TickPriority::High,
                    "NORMAL" => TickPriority::Normal,
                    other => panic!("unexpected priority {other}"),
                };
                assert_eq!(tick_priority_value(p), exp, "row {idx}: {row}");
            }
            "REPEATER_DELAY" => {
                let setting: i32 = c[1].parse().unwrap();
                let exp: i32 = c[2].parse().unwrap();
                assert_eq!(repeater_game_delay(setting), exp, "row {idx}: {row}");
            }
            "REPEATER_NEIGHBOR" => {
                let locked = bit(c[1]);
                let powered = bit(c[2]);
                let should_turn_on = bit(c[3]);
                let will_tick = bit(c[4]);
                let prioritize = feeds_into_diode(bit(c[5]), bit(c[5]));
                let exp = parse_sched(c[6]);
                assert_eq!(
                    diode_neighbor_schedule(repeater_game_delay(2), locked, powered, should_turn_on, will_tick, prioritize),
                    exp,
                    "row {idx}: {row}"
                );
            }
            "REPEATER_TICK" => {
                let locked = bit(c[1]);
                let powered = bit(c[2]);
                let should_turn_on = bit(c[3]);
                let exp_powered = bit(c[4]);
                let exp_updated = bit(c[5]);
                let exp_held = parse_sched(c[6]);
                let expected = if locked {
                    DiodeTick::IgnoredLocked
                } else {
                    match (powered, should_turn_on, exp_powered, exp_updated, exp_held) {
                        (true, false, false, true, None) => DiodeTick::TurnOff,
                        (true, true, true, false, None) => DiodeTick::NoChange,
                        (false, _, true, true, held) => DiodeTick::TurnOn { held_off_after: held },
                        other => panic!("unexpected repeater tick expectation {other:?} in {row}"),
                    }
                };
                assert_eq!(
                    diode_tick(locked, powered, should_turn_on, repeater_game_delay(3)),
                    expected,
                    "row {idx}: {row}"
                );
            }
            "OBSERVER_ARMS" => {
                let obs_side = bit(c[1]);
                let powered = bit(c[2]);
                let has_sched = bit(c[3]);
                let cli = bit(c[4]);
                let exp = parse_sched(c[5]);
                assert_eq!(observer_arms(obs_side, powered, has_sched, cli), exp, "row {idx}: {row}");
            }
            "OBSERVER_TICK" => {
                let powered = bit(c[1]);
                let exp_powered = bit(c[2]);
                let exp_sched = parse_sched(c[3]);
                let expected = match (powered, exp_powered, exp_sched) {
                    (true, false, None) => ObserverTick::PowerOff,
                    (false, true, Some((2, TickPriority::Normal))) => ObserverTick::PowerOn { off_after: 2 },
                    other => panic!("unexpected observer tick {other:?} in {row}"),
                };
                assert_eq!(observer_tick(powered), expected, "row {idx}: {row}");
            }
            "DISPENSER_NEIGHBOR" => {
                let triggered = bit(c[1]);
                let sig_here = bit(c[2]);
                let sig_above = bit(c[3]);
                let exp_trig = bit(c[4]);
                let exp_upd = bit(c[5]);
                let exp_sched = parse_sched(c[6]);
                let expected = match (exp_trig, exp_upd, exp_sched) {
                    (true, true, Some((4, TickPriority::Normal))) => DispenserEdge::Arm { dispense_after: 4 },
                    (false, true, None) => DispenserEdge::Disarm,
                    (_, false, None) => DispenserEdge::NoChange,
                    other => panic!("unexpected dispenser edge {other:?} in {row}"),
                };
                assert_eq!(
                    dispenser_neighbor_changed(triggered, sig_here, sig_above),
                    expected,
                    "row {idx}: {row}"
                );
            }
            "OFFSET_BAMBOO" => {
                let x: i32 = c[1].parse().unwrap();
                let z: i32 = c[2].parse().unwrap();
                let expected = parse_shape(c[3]);
                let state = BlockId::of("bamboo").default_state();
                assert_shape_eq(&state.block_at([x, 64, z]).shape, &expected, row);
            }
            "OFFSET_SPELEOTHEM" => {
                let x: i32 = c[1].parse().unwrap();
                let z: i32 = c[2].parse().unwrap();
                let expected = parse_shape(c[5]);
                let state = StateId::of(BlockId::of("pointed_dripstone"), &[
                    ("thickness", c[3]), ("vertical_direction", c[4]),
                ]);
                assert_shape_eq(&state.block_at([x, 64, z]).shape, &expected, row);
            }
            other => panic!("unknown tag {other}"),
        }
    }
    assert_eq!(count, 326);
}

#[test]
fn jar_projectiles_and_redstone() {
    use law::neighbor::Direction;
    use law::projectiles::*;
    use law::redstone::{comparator, copper_bulb, daylight, target, torch, TickPriority};
    use law::{Axis, Vec3};

    let h32 = |s: &str| u32::from_str_radix(s, 16).unwrap();
    let f32b = |s: &str| f32::from_bits(h32(s));
    let h64 = |s: &str| u64::from_str_radix(s, 16).unwrap();
    let d64 = |s: &str| f64::from_bits(h64(s));
    let vec3_bits = |actual: Vec3, cols: &[&str], label: &str| {
        assert_eq!(actual.x.to_bits(), h64(cols[0]), "{label} x");
        assert_eq!(actual.y.to_bits(), h64(cols[1]), "{label} y");
        assert_eq!(actual.z.to_bits(), h64(cols[2]), "{label} z");
    };

    let zero_draws = [0.5f64; 6];
    let mut arrow_air: Option<ArrowState> = None;
    let mut arrow_water: Option<ArrowState> = None;
    let mut throw_air: Option<(Vec3, Vec3)> = None;
    let mut throw_water: Option<(Vec3, Vec3)> = None;
    let mut torch_toggles = torch::Toggles::default();
    let mut torch_lit = true;

    let mut count = 0usize;
    for (line, row) in include_str!("data/projectiles_redstone.tsv").lines().enumerate() {
        if row.is_empty() {
            continue;
        }
        count += 1;
        let c: Vec<&str> = row.split('\t').collect();
        match c[0] {
            "BOW_CURVE" => {
                let t: i32 = c[1].parse().unwrap();
                let power = bow_power_for_time(t);
                let fires = bow_releases(power);
                let speed = bow_launch_speed(power);
                let crit = bow_is_crit(power);
                assert_eq!(power.to_bits(), h32(c[2]), "row {line} power");
                assert_eq!(fires, c[3] == "1", "row {line} fires");
                assert_eq!(speed.to_bits(), h32(c[4]), "row {line} speed");
                assert_eq!(crit, c[5] == "1", "row {line} crit");
                if fires {
                    let rel = bow_release(t).unwrap();
                    assert_eq!(rel.power.to_bits(), h32(c[2]), "row {line} rel power");
                    assert_eq!(rel.speed.to_bits(), h32(c[4]), "row {line} rel speed");
                    assert_eq!(rel.crit, crit, "row {line} rel crit");
                } else {
                    assert!(bow_release(t).is_none(), "row {line} no release");
                }
            }
            "BOW_LAUNCH" => {
                let t: i32 = c[1].parse().unwrap();
                let pitch = f32b(c[2]);
                let yaw = f32b(c[3]);
                let speed = bow_release(t).unwrap().speed;
                let known = Vec3::new(0.12, -0.0784, -0.08);
                let still = law::basis::body::launch(pitch, yaw + 0.0, 0.0, speed, BOW_INACCURACY, zero_draws, Vec3::default(), true);
                vec3_bits(still.velocity, &c[4..7], &format!("row {line} still"));
                assert_eq!(still.y_rot.to_bits(), h32(c[7]), "row {line} y_rot");
                assert_eq!(still.x_rot.to_bits(), h32(c[8]), "row {line} x_rot");

                let ground = law::basis::body::launch(pitch, yaw + 0.0, 0.0, speed, BOW_INACCURACY, zero_draws, known, true);
                vec3_bits(ground.velocity, &c[9..12], &format!("row {line} ground"));
                assert_eq!(ground.y_rot.to_bits(), h32(c[7]), "row {line} ground y_rot");
                assert_eq!(ground.x_rot.to_bits(), h32(c[8]), "row {line} ground x_rot");

                let air = law::basis::body::launch(pitch, yaw + 0.0, 0.0, speed, BOW_INACCURACY, zero_draws, known, false);
                vec3_bits(air.velocity, &c[12..15], &format!("row {line} air"));
                assert_eq!(air.y_rot.to_bits(), h32(c[7]), "row {line} air y_rot");
                assert_eq!(air.x_rot.to_bits(), h32(c[8]), "row {line} air x_rot");
            }
            "BOW_SPREAD" => {
                let draws = [0.8, 0.2, 0.3, 0.7, 0.75, 0.25];
                let launch = law::basis::body::launch(-30.5, 17.0, 0.0, 3.0, BOW_INACCURACY, draws, Vec3::default(), true);
                vec3_bits(launch.velocity, &c[1..4], &format!("row {line} spread"));
                assert_eq!(launch.y_rot.to_bits(), h32(c[4]), "row {line} spread y_rot");
                assert_eq!(launch.x_rot.to_bits(), h32(c[5]), "row {line} spread x_rot");
            }
            "XBOW_CHARGE" => {
                let qc: u8 = c[1].parse().unwrap();
                assert_eq!(crossbow_charging_time_seconds(qc).to_bits(), h32(c[2]), "row {line} secs");
                assert_eq!(crossbow_charge_duration(qc), c[3].parse::<i32>().unwrap(), "row {line} ticks");
            }
            "XBOW_POWER" => {
                assert_eq!(crossbow_shooting_power(false).to_bits(), h32(c[1]), "row {line} arrow");
                assert_eq!(crossbow_shooting_power(true).to_bits(), h32(c[2]), "row {line} firework");
            }
            "XBOW_FAN" => {
                let n: usize = c[1].parse().unwrap();
                let idx: usize = c[2].parse().unwrap();
                let spread = f32b(c[3]);
                let angle = projectile_fan_angle(idx, n, spread);
                assert_eq!(angle.to_bits(), h32(c[4]), "row {line} fan");
                if n == 3 {
                    assert_eq!(crossbow_multishot_angles(spread)[idx].to_bits(), h32(c[4]));
                }
            }
            "XBOW_MULTISHOT" => {
                let angle = f32b(c[1]);
                let pitch = f32b(c[2]);
                let yaw = f32b(c[3]);
                let speed = f32b(c[4]);
                let launch = crossbow_launch(pitch, yaw, angle, speed, 0.0, zero_draws);
                vec3_bits(launch.velocity, &c[5..8], &format!("row {line} xbow"));
                assert_eq!(launch.y_rot.to_bits(), h32(c[8]), "row {line} xbow y_rot");
                assert_eq!(launch.x_rot.to_bits(), h32(c[9]), "row {line} xbow x_rot");
            }
            "ARROW_TRAJ" => {
                let water = c[1] == "1";
                let tick: i32 = c[2].parse().unwrap();
                let slot = if water { &mut arrow_water } else { &mut arrow_air };
                if tick == 1 {
                    let init = law::basis::body::launch(-30.5, 17.0, 0.0, 3.0, 0.0, zero_draws, Vec3::default(), true);
                    *slot = Some(ArrowState {
                        pos: Vec3::new(0.5, 65.62, -0.5),
                        vel: init.velocity,
                        ..ArrowState::default()
                    });
                }
                let a = slot.as_mut().unwrap();
                assert!(arrow_free_tick(a, water, water));
                vec3_bits(a.pos, &c[3..6], &format!("row {line} arrow pos"));
                vec3_bits(a.vel, &c[6..9], &format!("row {line} arrow vel"));
            }
            "ARROW_TRANSITION" => {
                let water_before = c[1] == "1";
                let water_after = c[2] == "1";
                let init = law::basis::body::launch(-30.5, 17.0, 0.0, 3.0, 0.0, zero_draws, Vec3::default(), true);
                let mut a = ArrowState {
                    pos: Vec3::new(0.0, 64.5, 0.0),
                    vel: init.velocity,
                    ..ArrowState::default()
                };
                assert!(arrow_free_tick(&mut a, water_before, water_after));
                vec3_bits(a.pos, &c[3..6], &format!("row {line} trans pos"));
                vec3_bits(a.vel, &c[6..9], &format!("row {line} trans vel"));
            }
            "THROW_TRAJ" => {
                let water = c[1] == "1";
                let tick: i32 = c[2].parse().unwrap();
                let slot = if water { &mut throw_water } else { &mut throw_air };
                if tick == 1 {
                    let init = law::basis::body::launch(-30.5, 17.0, 0.0, SNOWBALL_SHOOT_POWER, 0.0, zero_draws, Vec3::default(), true);
                    *slot = Some((Vec3::new(0.5, 65.52, -0.5), init.velocity));
                }
                let (p, v) = slot.as_mut().unwrap();
                law::basis::body::THROWABLE.tick(p, v, water);
                vec3_bits(*p, &c[3..6], &format!("row {line} throw pos"));
                vec3_bits(*v, &c[6..9], &format!("row {line} throw vel"));
            }
            "ARROW_STICK" => {
                let mut a = ArrowState {
                    pos: Vec3::new(d64(c[1]), d64(c[2]), d64(c[3])),
                    vel: Vec3::new(d64(c[4]), d64(c[5]), d64(c[6])),
                    crit: true,
                    pierce_level: 3,
                    ..ArrowState::default()
                };
                arrow_hit_block(&mut a);
                vec3_bits(a.pos, &c[7..10], &format!("row {line} stick pos"));
                assert_eq!(a.vel, Vec3::default());
                assert!(a.in_ground);
                assert_eq!(a.shake_time, ARROW_SHAKE_TIME);
                assert!(!a.crit);
                assert_eq!(a.pierce_level, 0);
            }
            "ARROW_DMG" => {
                let vel = Vec3::new(d64(c[1]), d64(c[2]), d64(c[3]));
                let pwr: u8 = c[4].parse().unwrap();
                let base = arrow_power_base_damage(ARROW_BASE_DAMAGE, pwr);
                assert_eq!(base.to_bits(), h64(c[5]), "row {line} base");
                let dmg = arrow_damage(vel, base);
                assert_eq!(dmg, c[6].parse::<i32>().unwrap(), "row {line} dmg");
                let max_roll = arrow_crit_bonus_bound(dmg) - 1;
                assert_eq!(arrow_crit_damage(dmg, max_roll), c[7].parse::<i32>().unwrap(), "row {line} crit");
            }
            "ARROW_PUNCH" => {
                let punch: u8 = c[1].parse().unwrap();
                let resist = d64(c[2]);
                let vel = Vec3::new(d64(c[3]), d64(c[4]), d64(c[5]));
                let kb = arrow_punch_knockback(punch);
                let pushed = arrow_punch_push(vel, kb, resist);
                assert_eq!(pushed.is_some(), c[6] == "1", "row {line} pushes");
                vec3_bits(pushed.unwrap_or_default(), &c[7..10], &format!("row {line} punch"));
            }
            "FISH_CAST" => {
                let pitch = f32b(c[1]);
                let yaw = f32b(c[2]);
                let draws = [d64(c[3]), d64(c[4]), d64(c[5])];
                let (spawn, vel) = fishing_cast(pitch, yaw, Vec3::new(10.0, 64.0, -5.0), 65.62, draws);
                vec3_bits(spawn, &c[6..9], &format!("row {line} fish spawn"));
                vec3_bits(vel, &c[9..12], &format!("row {line} fish vel"));
            }
            "FISH_PULL" => {
                let owner = Vec3::new(d64(c[1]), d64(c[2]), d64(c[3]));
                let hook = Vec3::new(d64(c[4]), d64(c[5]), d64(c[6]));
                let tgt_vel = Vec3::new(d64(c[7]), d64(c[8]), d64(c[9]));
                let imp = fishing_pull_impulse(owner, hook);
                let next = fishing_pull_velocity(owner, hook, tgt_vel);
                vec3_bits(imp, &c[10..13], &format!("row {line} fish imp"));
                vec3_bits(next, &c[13..16], &format!("row {line} fish next"));
            }
            "TORCH_CONST" => {
                assert_eq!(torch::TOGGLE_DELAY, c[1].parse::<i32>().unwrap());
                assert_eq!(torch::RECENT_TOGGLE_TIMER, c[2].parse::<i64>().unwrap());
                assert_eq!(torch::MAX_RECENT_TOGGLES, c[3].parse::<i32>().unwrap());
                assert_eq!(torch::RESTART_DELAY, c[4].parse::<i32>().unwrap());
                assert_eq!(TickPriority::Normal as i32, c[5].parse::<i32>().unwrap());
            }
            "TORCH_SIG" => {
                let lit = c[2] == "1";
                let dir = match c[3] {
                    "down" => Direction::Down,
                    "up" => Direction::Up,
                    "north" => Direction::North,
                    "south" => Direction::South,
                    "west" => Direction::West,
                    "east" => Direction::East,
                    other => panic!("bad dir {other}"),
                };
                let exp_sig: i32 = c[4].parse().unwrap();
                let exp_dir: i32 = c[5].parse().unwrap();
                if c[1] == "standing" {
                    assert_eq!(torch::standing_get_signal(lit, dir), exp_sig, "row {line} sig");
                    assert_eq!(torch::get_direct_signal(lit, None, dir), exp_dir, "row {line} dir");
                } else {
                    assert_eq!(torch::wall_get_signal(lit, Direction::North, dir), exp_sig, "row {line} sig");
                    assert_eq!(torch::get_direct_signal(lit, Some(Direction::North), dir), exp_dir, "row {line} dir");
                }
            }
            "TORCH_STEP" => {
                let game_time: i64 = c[1].parse().unwrap();
                let expect_lit_before = c[2] == "1";
                let has_sig = c[3] == "1";
                assert_eq!(torch_lit, expect_lit_before, "row {line} pre lit");
                let res = torch::tick(torch_lit, has_sig, &mut torch_toggles, game_time);
                torch_lit = res.lit;
                assert_eq!(res.lit, c[4] == "1", "row {line} post lit");
                assert_eq!(res.burnt_out, c[5] == "1", "row {line} burnt");
                let exp_restart: i32 = c[6].parse().unwrap();
                assert_eq!(
                    res.restart_after,
                    if exp_restart >= 0 { Some((exp_restart, TickPriority::Normal)) } else { None },
                    "row {line} restart"
                );
                assert_eq!(torch_toggles.off_times.len(), c[7].parse::<usize>().unwrap(), "row {line} len");
            }
            "COMP_PRIO" => {
                assert_eq!(comparator::DELAY, c[1].parse::<i32>().unwrap());
                assert_eq!(comparator::priority(true) as i32, c[2].parse::<i32>().unwrap());
                assert_eq!(comparator::priority(false) as i32, c[3].parse::<i32>().unwrap());
            }
            "COMP_MATH" => {
                let mode = if c[1] == "COMPARE" { comparator::Mode::Compare } else { comparator::Mode::Subtract };
                let input: i32 = c[2].parse().unwrap();
                let side: i32 = c[3].parse().unwrap();
                assert_eq!(comparator::calculate_output_signal(input, side, mode), c[4].parse::<i32>().unwrap(), "row {line} out");
                assert_eq!(comparator::should_turn_on(input, side, mode), c[5] == "1", "row {line} on");
            }
            "COMP_CONTAINER" => {
                let size: usize = c[1].parse().unwrap();
                let slots: Vec<(i32, i32)> = if c[2] == "0" {
                    Vec::new()
                } else {
                    c[2].split(';').map(|pair| {
                        let (cnt, max) = pair.split_once(':').unwrap();
                        (cnt.parse().unwrap(), max.parse().unwrap())
                    }).collect()
                };
                assert_eq!(comparator::container_analog_from_slots(size, &slots), c[3].parse::<i32>().unwrap(), "row {line} container");
            }
            "COMP_READ" => {
                let behind_sig: i32 = c[1].parse().unwrap();
                let behind_analog = if c[2].parse::<i32>().unwrap() >= 0 { Some(c[2].parse().unwrap()) } else { None };
                let behind_cond = c[3] == "1";
                let far_frame = if c[4].parse::<i32>().unwrap() >= 0 { Some(c[4].parse().unwrap()) } else { None };
                let far_analog = if c[5].parse::<i32>().unwrap() >= 0 { Some(c[5].parse().unwrap()) } else { None };
                assert_eq!(
                    comparator::input_signal(behind_sig, behind_analog, behind_cond, far_frame, far_analog),
                    c[6].parse::<i32>().unwrap(),
                    "row {line} read"
                );
            }
            "BULB" => {
                let lit = c[1] == "1";
                let pow = c[2] == "1";
                let sig = c[3] == "1";
                let changed = c[4] == "1";
                let next_lit = c[5] == "1";
                let next_pow = c[6] == "1";
                let analog: i32 = c[7].parse().unwrap();
                let flipped = copper_bulb::check_and_flip(lit, pow, sig);
                assert_eq!(flipped, if changed { Some((next_lit, next_pow)) } else { None }, "row {line} bulb");
                assert_eq!(copper_bulb::analog_output(next_lit), analog, "row {line} bulb analog");
            }
            "TARGET_CONST" => {
                assert_eq!(target::ACTIVATION_TICKS_ARROWS, c[1].parse::<i32>().unwrap());
                assert_eq!(target::ACTIVATION_TICKS_OTHER, c[2].parse::<i32>().unwrap());
            }
            "TARGET_STR" => {
                let axis = match c[1] {
                    "x" => Axis::X,
                    "y" => Axis::Y,
                    "z" => Axis::Z,
                    other => panic!("bad axis {other}"),
                };
                let got = target::redstone_strength(d64(c[2]), d64(c[3]), d64(c[4]), axis);
                assert_eq!(got, c[5].parse::<i32>().unwrap(), "row {line} target");
            }
            "DAYLIGHT" => {
                let sky: i32 = c[1].parse().unwrap();
                let deg = f32b(c[2]);
                let inv = c[3] == "1";
                let got = daylight::update_signal_strength(sky, deg, inv);
                assert_eq!(got, c[4].parse::<i32>().unwrap(), "row {line} daylight");
            }
            other => panic!("unknown tag {other}"),
        }
    }
    assert_eq!(count, 547);
}

#[test]
fn bucket_respects_jar_water_and_lava_replaceability() {
    use law::{BoxWorld, basis::cell::BlockId, sim::SimWorld, world_hooks::BucketContent};

    let base = BoxWorld::default();
    let mut sim = SimWorld::new(&base);
    let target = [0, 65, 0];
    // The 26.2 Fluid overload returns false for each of these states.
    for name in ["stone", "granite", "dirt"] {
        let state = BlockId::of(name).default_state();
        sim.set_cell_raw(target, state);
        for content in [BucketContent::Water, BucketContent::Lava] {
            assert!(!sim.empty_bucket_at(content, target, None, false), "{name}: {content:?}");
            assert_eq!(sim.cell(target), state);
        }
    }
    assert!(sim.empty_bucket_at(BucketContent::Water, [1, 65, 0], None, false));
    assert_eq!(sim.cell([1, 65, 0]), BlockId::of("water").default_state());
}

#[test]
fn piston_tick_refuses_jar_unbreakable_bedrock() {
    use law::{BoxWorld, basis::cell::{BlockId, StateId}, sim::SimWorld};

    let base = BoxWorld::default();
    let stone = BlockId::of("stone").default_state();
    let bedrock = BlockId::of("bedrock").default_state();
    assert_eq!(bedrock.rules().destroy_speed.to_bits(), 0xbf80_0000);
    let lever = StateId::of(BlockId::of("lever"), &[("face", "floor"), ("facing", "east")]);
    let piston = StateId::of(BlockId::of("piston"), &[("facing", "east")]);
    for (payload, extends) in [(bedrock, false), (stone, true)] {
        let mut sim = SimWorld::new(&base);
        sim.set_cell_raw([0, 63, 0], stone);
        sim.set_cell_raw([0, 64, 0], lever);
        sim.set_cell_raw([1, 64, 0], piston);
        sim.set_cell_raw([2, 64, 0], payload);
        assert!(sim.pull_lever([0, 64, 0]).0);
        for _ in 0..8 {
            sim.step_tick();
        }
        assert_eq!(sim.cell([1, 64, 0]), piston.with_bool("extended", extends));
        if extends {
            assert_eq!(sim.cell([2, 64, 0]).block().name(), "piston_head");
            assert_eq!(sim.cell([3, 64, 0]), stone);
        } else {
            assert_eq!(sim.cell([2, 64, 0]), bedrock);
            assert_eq!(sim.cell([3, 64, 0]), StateId::AIR);
        }
    }
}

#[test]
fn bucket_preserves_single_slab_type_and_redirects_double_slab() {
    use law::{BoxWorld, basis::cell::{BlockId, StateId}, neighbor::Direction, sim::SimWorld, world_hooks::BucketContent};

    let base = BoxWorld::default();
    let target = [0, 65, 0];
    let above = Direction::Up.relative(target);
    let water = BlockId::of("water").default_state();
    // world_hooks.tsv records the in-place and redirected bucket writes.
    for kind in ["bottom", "top", "double"] {
        let mut sim = SimWorld::new(&base);
        let slab = StateId::of(BlockId::of("oak_slab"), &[("type", kind), ("waterlogged", "false")]);
        sim.set_cell_raw(target, slab);
        assert!(sim.empty_bucket_at(BucketContent::Water, target, Some((target, Direction::Up)), false));
        if kind == "double" {
            assert_eq!(sim.cell(target), slab);
            assert_eq!(sim.cell(above), water);
        } else {
            assert_eq!(sim.cell(target), slab.with_bool("waterlogged", true));
            assert_eq!(sim.cell(above), StateId::AIR);
        }
    }
}

#[test]
fn jar_world_sim() {
    use law::boat::{self, BoatStatus, Controls as BoatControls};
    use law::fluid::{FlowKind, FluidCell};
    use law::footprint::RecordingWorld;
    use law::basis::cell::StateId;
    use law::neighbor::{Direction, update_flags};
    use law::interact::BlockId;
    use law::sim::{
        CellState, FallingKind, SimLivingEntity, SimWorld, WorldDelta,
    };
    use law::world_hooks::BucketContent;
    use law::{BlockPos, BoxWorld, PlayerState, Vec3, World};

    fn parse_h64(s: &str) -> u64 {
        u64::from_str_radix(s, 16).unwrap_or_else(|e| panic!("bad h64 {s:?}: {e}"))
    }
    fn parse_h32(s: &str) -> u32 {
        u32::from_str_radix(s, 16).unwrap_or_else(|e| panic!("bad h32 {s:?}: {e}"))
    }
    fn dir_name(d: Direction) -> &'static str {
        match d {
            Direction::Down => "down",
            Direction::Up => "up",
            Direction::North => "north",
            Direction::South => "south",
            Direction::West => "west",
            Direction::East => "east",
        }
    }
    fn sim_block_tag<W: World>(sim: &SimWorld<'_, W>, pos: BlockPos) -> String {
        let cell = sim.cell(pos);
        match cell.kind() {
            CellState::Empty => "air".to_owned(),
            CellState::Plain => match cell.block().name() {
                name @ ("stone" | "cobblestone" | "obsidian" | "torch") => name.to_owned(),
                "slime_block" => "slime".to_owned(),
                "honey_block" => "honey".to_owned(),
                _ => "solid".to_owned(),
            },
            CellState::Fluid { kind, cell: fc } => {
                let name = match kind {
                    FlowKind::Water => "water",
                    FlowKind::Lava { .. } => "lava",
                };
                format!("{name}:{}:{}", u8::from(fc.source), fc.amount)
            }
            CellState::Falling(FallingKind::Sand) => "sand".to_owned(),
            CellState::Falling(FallingKind::ConcretePowder) => "concrete_powder".to_owned(),
            CellState::Falling(FallingKind::Anvil { .. }) => match cell.block().name() {
                name @ ("anvil" | "chipped_anvil") => name.to_owned(),
                _ => "damaged_anvil".to_owned(),
            },
            CellState::MovingPiston { facing, .. } => {
                format!("moving_piston:{}", dir_name(facing))
            }
            CellState::PistonHead { facing, .. } => format!("piston_head:{}", dir_name(facing)),
            CellState::Piston { sticky, extended, .. } => {
                let prefix = if sticky { "sticky_piston" } else { "piston" };
                format!("{prefix}:{}", u8::from(extended))
            }
            CellState::Door(d) => format!("iron_door:{}:{}", u8::from(d.open), u8::from(d.powered)),
            CellState::TrapDoor {
                open,
                powered,
                waterlogged,
                ..
            } => format!(
                "oak_trapdoor:{}:{}:{}",
                u8::from(open),
                u8::from(powered),
                u8::from(waterlogged)
            ),
            CellState::RedstoneWire { power } => format!("wire:{power}"),
            CellState::Lever(l) => format!("lever:{}", u8::from(l.powered)),
            _ => "other".to_owned(),
        }
    }
    fn boat_status_name(s: Option<BoatStatus>) -> &'static str {
        match s {
            Some(BoatStatus::InWater) => "IN_WATER",
            Some(BoatStatus::UnderWater) => "UNDER_WATER",
            Some(BoatStatus::UnderFlowingWater) => "UNDER_FLOWING_WATER",
            Some(BoatStatus::OnLand) => "ON_LAND",
            Some(BoatStatus::InAir) => "IN_AIR",
            None => "NONE",
        }
    }
    fn pig_entity(x: f64, y: f64, z: f64) -> SimLivingEntity {
        SimLivingEntity {
            position: Vec3::new(x, y, z),
            width: 0.9,
            height: 0.9,
            eye_height: 0.765,
            health: 10.0,
            ..SimLivingEntity::default()
        }
    }
    fn floor_stone<W: World>(sim: &mut SimWorld<'_, W>, x0: i32, z0: i32, x1: i32, z1: i32, y: i32) {
        let stone = BlockId::of("stone").default_state();
        for x in x0..=x1 {
            for z in z0..=z1 {
                sim.set_cell_raw([x, y, z], stone.clone());
            }
        }
    }

    let raw = include_str!("data/world_sim.tsv");
    let rows: Vec<Vec<&str>> = raw
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| l.split('\t').collect())
        .collect();
    assert_eq!(rows.len(), 63, "expected 63 witness rows in world_sim.tsv");

    let base = BoxWorld::default();

    // -------------------------------------------------------------------------
    // WorldDelta: footprint invalidation, and undo gives back the base's certificates
    // -------------------------------------------------------------------------
    {
        let state = |name: &str, props: &[(&str, &str)]| law::basis::cell::StateId::of(BlockId::of(name), props);
        let mut delta = WorldDelta::new(&base);
        delta.set([5, 64, 5], state("stone", &[]));
        let rec = RecordingWorld::new(&delta);
        let mut player = PlayerState {
            position: Vec3::new(5.5, 65.0, 5.5),
            ..PlayerState::default()
        };
        law::tick(&mut player, &rec, &law::Input::default());
        let fp = rec.footprint();
        let mark = delta.checkpoint();
        assert!(!delta.invalidates(mark, &fp));
        delta.set([5, 64, 5], state("water", &[("level", "0")]));
        assert!(delta.invalidates(mark, &fp) && delta.has_fluid());
        assert!(delta.edited_since(mark).eq([[5, 64, 5]]));
        delta.rollback(mark);
        assert!(!delta.has_fluid());
        assert_eq!(delta.state_id([5, 64, 5]), Some(state("stone", &[])));
        assert_eq!(delta.model_id([5, 64, 5]), state("stone", &[]).model());
        delta.rollback(0);
        assert_eq!(delta.state_id([5, 64, 5]), None);

        struct UniformBase(law::Block);
        impl World for UniformBase {
            fn block(&self, _: BlockPos) -> &law::Block { &self.0 }
            fn uniform(&self, _: BlockPos, _: BlockPos) -> Option<&law::Block> { Some(&self.0) }
            fn loaded(&self, _: BlockPos) -> bool { true }
        }
        let ubase = UniformBase(law::Block::default());
        let mut udelta = WorldDelta::new(&ubase);
        assert!(udelta.uniform([5, 64, 5], [5, 64, 5]).is_some());
        udelta.set([5, 64, 5], state("stone", &[]));
        let umark = udelta.checkpoint();
        udelta.set([20, 64, 20], state("stone", &[]));
        assert!(udelta.uniform([5, 64, 5], [5, 64, 5]).is_none());
        assert!(udelta.uniform([20, 64, 20], [20, 64, 20]).is_none());
        udelta.rollback(umark);
        assert!(udelta.uniform([20, 64, 20], [20, 64, 20]).is_some());
        assert!(udelta.uniform([5, 64, 5], [5, 64, 5]).is_none());
        udelta.rollback(0);
        assert!(udelta.uniform([5, 64, 5], [5, 64, 5]).is_some());
    }

    // -------------------------------------------------------------------------
    // SCENARIO A: Lever -> Redstone Wire -> Iron Door + Waterlogged Trapdoor
    // -------------------------------------------------------------------------
    {
        let a_rows: Vec<_> = rows.iter().filter(|r| r[0] == "A_CASCADE").collect();
        assert_eq!(a_rows.len(), 9);
        let mut sim = SimWorld::new(&base);
        floor_stone(&mut sim, -2, -2, 13, 5, 64);
        let lever_pos = [0, 65, 0];
        sim.set_cell_raw(
            lever_pos,
            StateId::of(BlockId::of("lever"), &[("face", "floor"), ("facing", "east"), ("powered", "false")]),
        );
        for i in 1..=8 {
            sim.set_cell_raw([i, 65, 0], BlockId::of("redstone_wire").default_state());
        }
        let branch_wire = [2, 65, 1];
        sim.set_cell_raw(branch_wire, BlockId::of("redstone_wire").default_state());
        let door_low = [9, 65, 0];
        let door_high = [9, 66, 0];
        sim.set_cell_raw(
            door_low,
            StateId::of(BlockId::of("iron_door"), &[("facing", "north"), ("half", "lower"), ("hinge", "left"), ("open", "false"), ("powered", "false")]),
        );
        sim.set_cell_raw(
            door_high,
            StateId::of(BlockId::of("iron_door"), &[("facing", "north"), ("half", "upper"), ("hinge", "left"), ("open", "false"), ("powered", "false")]),
        );
        let trap_pos = [2, 65, 2];
        sim.set_cell_raw(
            trap_pos,
            StateId::of(BlockId::of("oak_trapdoor"), &[("facing", "north"), ("half", "bottom"), ("open", "false"), ("powered", "false"), ("waterlogged", "true")]),
        );
        sim.set_cell_raw([1, 65, 2], BlockId::of("stone").default_state());
        sim.set_cell_raw([2, 65, 3], BlockId::of("stone").default_state());

        let check_a = |sim: &SimWorld<'_, BoxWorld>, row: &[&str]| {
            assert_eq!(sim_block_tag(sim, lever_pos), row[3], "row {row:?} lever");
            assert_eq!(sim_block_tag(sim, [1, 65, 0]), row[4], "row {row:?} wire1");
            assert_eq!(sim_block_tag(sim, [8, 65, 0]), row[5], "row {row:?} wire8");
            assert_eq!(sim_block_tag(sim, branch_wire), row[6], "row {row:?} branch_wire");
            assert_eq!(sim_block_tag(sim, door_low), row[7], "row {row:?} door_low");
            assert_eq!(sim_block_tag(sim, door_high), row[8], "row {row:?} door_high");
            assert_eq!(sim_block_tag(sim, trap_pos), row[9], "row {row:?} trap");
            assert_eq!(sim_block_tag(sim, [3, 65, 2]), row[10], "row {row:?} east_water");
        };

        let (powered, _) = sim.pull_lever(lever_pos);
        assert!(powered);
        check_a(&sim, a_rows[0]);
        for row in &a_rows[1..=7] {
            sim.step_tick();
            check_a(&sim, row);
        }
        let (powered, _) = sim.pull_lever(lever_pos);
        assert!(!powered);
        check_a(&sim, a_rows[8]);
    }

    // -------------------------------------------------------------------------
    // SCENARIO B: Bucket -> Fluid Spread, Hole Attraction, 2-Source, Lava Contact
    // -------------------------------------------------------------------------
    {
        let b_rows: Vec<_> = rows.iter().filter(|r| r[0] == "B_FLUID").collect();
        assert_eq!(b_rows.len(), 7);
        let mut sim = SimWorld::new(&base);
        floor_stone(&mut sim, 62, -2, 72, 16, 63);
        floor_stone(&mut sim, 62, -2, 72, 16, 64);
        let stone = BlockId::of("stone").default_state();
        for x in 64..=70 {
            sim.set_cell_raw([x, 65, -1], stone.clone());
            sim.set_cell_raw([x, 65, 1], stone.clone());
        }
        sim.set_cell_raw([64, 65, 0], stone.clone());
        sim.set_cell_raw([70, 65, 0], stone.clone());
        sim.set_cell_raw([68, 64, 0], StateId::AIR);

        for x in 64..=68 {
            sim.set_cell_raw([x, 65, 3], stone.clone());
            sim.set_cell_raw([x, 65, 5], stone.clone());
        }
        sim.set_cell_raw([64, 65, 4], stone.clone());
        sim.set_cell_raw([68, 65, 4], stone.clone());

        for x in 64..=68 {
            sim.set_cell_raw([x, 65, 7], stone.clone());
            sim.set_cell_raw([x, 65, 9], stone.clone());
        }
        sim.set_cell_raw([64, 65, 8], stone.clone());
        sim.set_cell_raw([67, 65, 8], stone.clone());
        sim.set_cell_raw([66, 65, 8], StateId::liquid("lava", FluidCell::WATER_SOURCE));

        for x in 64..=68 {
            sim.set_cell_raw([x, 65, 10], stone.clone());
            sim.set_cell_raw([x, 65, 12], stone.clone());
            sim.set_cell_raw([x, 65, 13], stone.clone());
            sim.set_cell_raw([x, 65, 15], stone.clone());
            sim.set_cell_raw([x, 66, 13], stone.clone());
            sim.set_cell_raw([x, 66, 15], stone.clone());
        }
        sim.set_cell_raw([64, 65, 11], stone.clone());
        sim.set_cell_raw([68, 65, 11], stone.clone());
        sim.set_cell_raw([64, 65, 14], stone.clone());
        sim.set_cell_raw([66, 65, 14], stone.clone());
        sim.set_cell_raw([64, 66, 14], stone.clone());
        sim.set_cell_raw([66, 66, 14], stone.clone());

        assert!(sim.empty_bucket_at(BucketContent::Water, [65, 65, 0], None, false));
        assert!(sim.empty_bucket_at(BucketContent::Water, [65, 65, 4], None, false));
        assert!(sim.empty_bucket_at(BucketContent::Water, [67, 65, 4], None, false));
        assert!(sim.empty_bucket_at(BucketContent::Water, [65, 65, 8], None, false));
        assert!(sim.empty_bucket_at(BucketContent::Water, [65, 65, 11], None, false));
        assert!(sim.empty_bucket_at(BucketContent::Lava, [67, 65, 11], None, false));
        assert!(sim.empty_bucket_at(BucketContent::Water, [65, 65, 14], None, false));
        assert!(sim.empty_bucket_at(BucketContent::Lava, [65, 66, 14], None, false));

        for t in 0..=35usize {
            if t > 0 {
                sim.step_tick();
            }
            if let Some(row) = b_rows.iter().find(|r| r[1].parse::<usize>().unwrap() == t) {
                assert_eq!(sim_block_tag(&sim, [66, 65, 0]), row[2], "B t={t} (66,65,0)");
                assert_eq!(sim_block_tag(&sim, [67, 65, 0]), row[3], "B t={t} (67,65,0)");
                assert_eq!(sim_block_tag(&sim, [68, 65, 0]), row[4], "B t={t} (68,65,0)");
                assert_eq!(sim_block_tag(&sim, [68, 64, 0]), row[5], "B t={t} (68,64,0)");
                assert_eq!(sim_block_tag(&sim, [69, 65, 0]), row[6], "B t={t} (69,65,0)");
                assert_eq!(sim_block_tag(&sim, [66, 65, 4]), row[7], "B t={t} (66,65,4)");
                assert_eq!(sim_block_tag(&sim, [66, 65, 8]), row[8], "B t={t} (66,65,8)");
                assert_eq!(sim_block_tag(&sim, [66, 65, 11]), row[9], "B t={t} (66,65,11)");
                assert_eq!(sim_block_tag(&sim, [65, 65, 14]), row[10], "B t={t} (65,65,14)");
            }
        }
    }

    // -------------------------------------------------------------------------
    // SCENARIO C: Pistons (extend/retract + MovingPiston + Slime bounce + Honey drag)
    // -------------------------------------------------------------------------
    {
        let c_rows: Vec<_> = rows.iter().filter(|r| r[0] == "C_PISTON").collect();
        assert_eq!(c_rows.len(), 11);
        let mut sim = SimWorld::new(&base);
        floor_stone(&mut sim, 126, -2, 134, 10, 63);
        floor_stone(&mut sim, 126, -2, 134, 10, 64);

        sim.set_cell_raw([128, 64, 4], StateId::of(BlockId::of("sticky_piston"), &[("facing", "up"), ("extended", "false")]));
        sim.set_cell_raw([128, 65, 4], BlockId::of("slime_block").default_state());
        let c1_lever = [127, 65, 4];
        sim.set_cell_raw(c1_lever, StateId::of(BlockId::of("lever"), &[("face", "floor"), ("facing", "east"), ("powered", "false")]));
        let pig_c1 = sim.spawn_living(pig_entity(128.5, 66.0, 4.5));

        sim.set_cell_raw([128, 65, 0], StateId::of(BlockId::of("sticky_piston"), &[("facing", "east"), ("extended", "false")]));
        sim.set_cell_raw([129, 65, 0], BlockId::of("honey_block").default_state());
        sim.set_cell_raw([129, 65, 1], BlockId::of("slime_block").default_state());
        sim.set_cell_raw([130, 65, 0], BlockId::of("stone").default_state());
        let c2_lever = [127, 65, 0];
        sim.set_cell_raw(c2_lever, StateId::of(BlockId::of("lever"), &[("face", "floor"), ("facing", "east"), ("powered", "false")]));
        let pig_c2 = sim.spawn_living(pig_entity(129.5, 66.0, 0.5));

        sim.pull_lever(c1_lever);
        sim.pull_lever(c2_lever);
        for t in 1..=11usize {
            if t == 7 {
                sim.pull_lever(c1_lever);
                sim.pull_lever(c2_lever);
            }
            sim.step_tick();
            let row = c_rows[t - 1];
            assert_eq!(sim_block_tag(&sim, [128, 65, 4]), row[3], "C t={t} (128,65,4)");
            assert_eq!(sim_block_tag(&sim, [128, 66, 4]), row[4], "C t={t} (128,66,4)");
            assert_eq!(sim_block_tag(&sim, [129, 65, 0]), row[5], "C t={t} (129,65,0)");
            assert_eq!(sim_block_tag(&sim, [130, 65, 0]), row[6], "C t={t} (130,65,0)");
            assert_eq!(sim_block_tag(&sim, [131, 65, 0]), row[7], "C t={t} (131,65,0)");
            assert_eq!(sim_block_tag(&sim, [129, 65, 1]), row[8], "C t={t} (129,65,1)");
            let e1 = &sim.living_entities[pig_c1];
            assert_eq!(e1.position.x.to_bits(), parse_h64(row[9]), "C t={t} pig1 x");
            assert_eq!(e1.position.y.to_bits(), parse_h64(row[10]), "C t={t} pig1 y");
            assert_eq!(e1.position.z.to_bits(), parse_h64(row[11]), "C t={t} pig1 z");
            assert_eq!(e1.velocity.x.to_bits(), parse_h64(row[12]), "C t={t} pig1 vx");
            assert_eq!(e1.velocity.y.to_bits(), parse_h64(row[13]), "C t={t} pig1 vy");
            assert_eq!(e1.velocity.z.to_bits(), parse_h64(row[14]), "C t={t} pig1 vz");
            let e2 = &sim.living_entities[pig_c2];
            assert_eq!(e2.position.x.to_bits(), parse_h64(row[15]), "C t={t} pig2 x");
            assert_eq!(e2.position.y.to_bits(), parse_h64(row[16]), "C t={t} pig2 y");
            assert_eq!(e2.position.z.to_bits(), parse_h64(row[17]), "C t={t} pig2 z");
            assert_eq!(e2.velocity.x.to_bits(), parse_h64(row[18]), "C t={t} pig2 vx");
            assert_eq!(e2.velocity.y.to_bits(), parse_h64(row[19]), "C t={t} pig2 vy");
            assert_eq!(e2.velocity.z.to_bits(), parse_h64(row[20]), "C t={t} pig2 vz");
        }
    }

    // -------------------------------------------------------------------------
    // SCENARIO D: Falling Sand / Anvil / Torch Refuse / Concrete Powder Solidify
    // -------------------------------------------------------------------------
    {
        let d_rows: Vec<_> = rows.iter().filter(|r| r[0] == "D_FALL").collect();
        assert_eq!(d_rows.len(), 22);
        let mut sim = SimWorld::new(&base);
        sim.set_rng_seed(0x9e37_79b9_7f4a_7c15);
        floor_stone(&mut sim, 190, -2, 206, 2, 64);

        let d1_sup = [192, 69, 0];
        let d1_sand = [192, 70, 0];
        sim.set_cell_raw(d1_sup, BlockId::of("stone").default_state());
        sim.set_cell_raw(d1_sand, BlockId::of("sand").default_state());

        let d2_sup = [196, 68, 0];
        let d2_anvil = [196, 69, 0];
        sim.set_cell_raw(d2_sup, BlockId::of("stone").default_state());
        sim.set_cell_raw(d2_anvil, StateId::of(BlockId::of("anvil"), &[("facing", "north")]));
        let pig_d2 = sim.spawn_living(pig_entity(196.5, 65.0, 0.5));

        let d3_torch = [200, 65, 0];
        let d3_sup = [200, 67, 0];
        let d3_sand = [200, 68, 0];
        sim.set_cell_raw(d3_torch, BlockId::of("torch").default_state());
        sim.set_cell_raw(d3_sup, BlockId::of("stone").default_state());
        sim.set_cell_raw(d3_sand, BlockId::of("sand").default_state());

        let d4_water = [204, 65, 0];
        sim.set_cell_raw([203, 65, 0], BlockId::of("stone").default_state());
        sim.set_cell_raw([205, 65, 0], BlockId::of("stone").default_state());
        sim.set_cell_raw([204, 65, -1], BlockId::of("stone").default_state());
        sim.set_cell_raw([204, 65, 1], BlockId::of("stone").default_state());
        sim.set_cell_raw(d4_water, StateId::liquid("water", FluidCell::WATER_SOURCE));
        let d4_sup = [204, 67, 0];
        let d4_powder = [204, 68, 0];
        sim.set_cell_raw(d4_sup, BlockId::of("stone").default_state());
        sim.set_cell_raw(d4_powder, BlockId::of("white_concrete_powder").default_state());

        let air = StateId::AIR;
        sim.set_cell_with_flags(d1_sup, air.clone(), update_flags::ALL);
        sim.set_cell_with_flags(d2_sup, air.clone(), update_flags::ALL);
        sim.set_cell_with_flags(d3_sup, air.clone(), update_flags::ALL);
        sim.set_cell_with_flags(d4_sup, air, update_flags::ALL);

        for t in 1..=22usize {
            sim.step_tick();
            let row = d_rows[t - 1];
            let fb_sand = sim
                .falling_blocks
                .iter()
                .find(|fb| !fb.removed && (fb.pos.x - 192.5).abs() < 0.1);
            let fb_anvil = sim
                .falling_blocks
                .iter()
                .find(|fb| !fb.removed && (fb.pos.x - 196.5).abs() < 0.1);

            if row[2] == "LANDED" {
                assert!(fb_sand.is_none(), "D t={t} expected sand landed");
            } else {
                let fb = fb_sand.unwrap_or_else(|| panic!("D t={t} expected sand airborne"));
                assert_eq!(fb.pos.y.to_bits(), parse_h64(row[2]), "D t={t} sand y");
                assert_eq!(fb.vel.y.to_bits(), parse_h64(row[3]), "D t={t} sand vy");
                assert_eq!(fb.time.to_string(), row[4], "D t={t} sand time");
            }
            assert_eq!(sim_block_tag(&sim, [192, 65, 0]), row[5], "D t={t} sand block");
            if row[6] == "LANDED" {
                assert!(fb_anvil.is_none(), "D t={t} expected anvil landed");
            } else {
                let fb = fb_anvil.unwrap_or_else(|| panic!("D t={t} expected anvil airborne"));
                assert_eq!(fb.pos.y.to_bits(), parse_h64(row[6]), "D t={t} anvil y");
                assert_eq!(fb.vel.y.to_bits(), parse_h64(row[7]), "D t={t} anvil vy");
            }
            assert_eq!(
                sim.living_entities[pig_d2].health.to_bits(),
                parse_h32(row[8]),
                "D t={t} pigD2 hp"
            );
            let anvil_tag = sim_block_tag(&sim, [196, 65, 0]);
            if row[9] == "anvil" {
                assert!(
                    anvil_tag == "anvil" || anvil_tag == "chipped_anvil",
                    "D t={t} anvil block {anvil_tag}"
                );
            } else {
                assert_eq!(anvil_tag, row[9], "D t={t} anvil block");
            }
            assert_eq!(sim_block_tag(&sim, d3_torch), row[10], "D t={t} torch");
            let d4_tag = sim_block_tag(&sim, d4_water);
            if row[11] == "white_concrete" {
                assert_eq!(d4_tag, "stone", "D t={t} concrete solidified");
            } else {
                assert_eq!(d4_tag, row[11], "D t={t} concrete water");
            }
        }
    }

    // -------------------------------------------------------------------------
    // SCENARIO E: Boat Water-Snap + Paddle + Living Entity Crowd Push
    // -------------------------------------------------------------------------
    {
        let e1_rows: Vec<_> = rows.iter().filter(|r| r[0] == "E_BOAT_CROWD").collect();
        let e2_rows: Vec<_> = rows.iter().filter(|r| r[0] == "E_PADDLE").collect();
        assert_eq!(e1_rows.len(), 8);
        assert_eq!(e2_rows.len(), 6);

        let mut sim = SimWorld::new(&base);
        floor_stone(&mut sim, 253, -3, 275, 5, 63);
        floor_stone(&mut sim, 253, -3, 275, 5, 64);
        for x in 254..=260 {
            for z in -2..=2 {
                sim.set_cell_raw([x, 64, z], StateId::liquid("water", FluidCell::WATER_SOURCE));
            }
        }

        let b_idx = sim.spawn_boat(Vec3::new(256.5, 65.8, 0.5), 0.0);
        sim.boats[b_idx].body.boat.velocity = Vec3::new(0.15, 0.0, 0.25);

        let p0 = sim.spawn_living(pig_entity(270.5, 65.0, 0.5));
        let p1 = sim.spawn_living(pig_entity(270.6, 65.0, 0.5));
        let p2 = sim.spawn_living(pig_entity(270.5, 65.0, 0.65));
        let p3 = sim.spawn_living(pig_entity(270.4, 65.0, 0.4));

        for t in 1..=8usize {
            sim.step_tick();
            let row = e1_rows[t - 1];
            let b = &sim.boats[b_idx].body;
            assert_eq!(b.boat.position.x.to_bits(), parse_h64(row[2]), "E1 t={t} bx");
            assert_eq!(b.boat.position.y.to_bits(), parse_h64(row[3]), "E1 t={t} by");
            assert_eq!(b.boat.position.z.to_bits(), parse_h64(row[4]), "E1 t={t} bz");
            assert_eq!(b.boat.velocity.x.to_bits(), parse_h64(row[5]), "E1 t={t} bvx");
            assert_eq!(b.boat.velocity.y.to_bits(), parse_h64(row[6]), "E1 t={t} bvy");
            assert_eq!(b.boat.velocity.z.to_bits(), parse_h64(row[7]), "E1 t={t} bvz");
            assert_eq!(b.boat.yaw.to_bits(), parse_h32(row[8]), "E1 t={t} byaw");
            let mut b_status_probe = b.clone();
            let cur_status = Some(boat::get_status(&mut b_status_probe, sim.world()));
            assert_eq!(boat_status_name(cur_status), row[9], "E1 t={t} status");
            assert_eq!(sim.living_entities[p0].position.x.to_bits(), parse_h64(row[10]), "E1 t={t} p0x");
            assert_eq!(sim.living_entities[p0].position.z.to_bits(), parse_h64(row[11]), "E1 t={t} p0z");
            assert_eq!(sim.living_entities[p1].position.x.to_bits(), parse_h64(row[12]), "E1 t={t} p1x");
            assert_eq!(sim.living_entities[p1].position.z.to_bits(), parse_h64(row[13]), "E1 t={t} p1z");
            assert_eq!(sim.living_entities[p2].position.x.to_bits(), parse_h64(row[14]), "E1 t={t} p2x");
            assert_eq!(sim.living_entities[p2].position.z.to_bits(), parse_h64(row[15]), "E1 t={t} p2z");
            assert_eq!(sim.living_entities[p3].position.x.to_bits(), parse_h64(row[16]), "E1 t={t} p3x");
            assert_eq!(sim.living_entities[p3].position.z.to_bits(), parse_h64(row[17]), "E1 t={t} p3z");
        }

        sim.boats[b_idx].removed = true;
        let pb_idx = sim.spawn_boat(Vec3::new(257.5, 64.9, 0.5), 30.0);
        sim.boats[pb_idx].controls = BoatControls {
            left: true,
            right: false,
            forward: true,
            backward: false,
        };
        sim.boats[pb_idx].player_controlling = true;
        sim.boats[pb_idx].apply_controls = true;
        sim.boats[pb_idx].server_side = false;

        for t in 1..=6usize {
            sim.step_tick();
            let row = e2_rows[t - 1];
            let b = &sim.boats[pb_idx].body;
            assert_eq!(b.boat.position.x.to_bits(), parse_h64(row[2]), "E2 t={t} bx");
            assert_eq!(b.boat.position.y.to_bits(), parse_h64(row[3]), "E2 t={t} by");
            assert_eq!(b.boat.position.z.to_bits(), parse_h64(row[4]), "E2 t={t} bz");
            assert_eq!(b.boat.velocity.x.to_bits(), parse_h64(row[5]), "E2 t={t} bvx");
            assert_eq!(b.boat.velocity.y.to_bits(), parse_h64(row[6]), "E2 t={t} bvy");
            assert_eq!(b.boat.velocity.z.to_bits(), parse_h64(row[7]), "E2 t={t} bvz");
            assert_eq!(b.boat.yaw.to_bits(), parse_h32(row[8]), "E2 t={t} byaw");
            assert_eq!(boat_status_name(b.status), row[9], "E2 t={t} status");
        }
    }
}

#[test]
fn jar_mounts_breath_void() {
    use law::breath::{tick_air, BreathEffects};
    use law::math::JavaRandom;
    use law::mounts::{
        boost_factor, execute_riders_jump_camel, execute_riders_jump_horse,
        player_jump_pending_scale, ridden_input_happy_ghast, ridden_input_horse,
        ridden_speed, tick_ridden_happy_ghast, MountState,
    };
    use law::player::death::{
        below_world, respawn, respawn_anchor_charge, respawn_anchor_find_and_use,
        respawn_anchor_use, BedRule, RespawnAnchorUse, BAD_RESPAWN_EXPLOSION_RADIUS,
        BED_BOUNCE_RESTITUTION, BED_FALL_DISTANCE_MULTIPLIER, FELL_OUT_OF_WORLD,
        RESPAWN_ANCHOR_MAX_CHARGES, RESPAWN_ANCHOR_MIN_CHARGES,
    };
    use law::portal::{
        create_portal, find_closest_portal_position, nether_exit, spiral_around, DimensionHeightBounds,
        PortalAxis, PortalBlock, PortalWorld, WorldBorderBounds,
    };
    use law::{apply_server_resource_event, BlockPos, BoxWorld, PlayerState, ServerResourceEvent};

    let h32 = |s: &str| u32::from_str_radix(s, 16).unwrap();
    let h64 = |s: &str| u64::from_str_radix(s, 16).unwrap();
    let p_pos = |s: &str| -> BlockPos {
        let v: Vec<i32> = s.split(',').map(|x| x.parse().unwrap()).collect();
        [v[0], v[1], v[2]]
    };

    #[derive(Clone, Copy)]
    enum TerrainKind {
        Flat,
        Bumpy,
        Solid,
    }
    struct TestPortalWorld {
        kind: TerrainKind,
        dims: DimensionHeightBounds,
        border: WorldBorderBounds,
        writes: usize,
        hash: u64,
    }
    impl TestPortalWorld {
        fn ground(&self, x: i32, z: i32) -> i32 {
            match self.kind {
                TerrainKind::Flat => 64,
                TerrainKind::Bumpy => 60 + (x * 7 + z * 13).rem_euclid(8),
                TerrainKind::Solid => 318,
            }
        }
    }
    impl PortalWorld for TestPortalWorld {
        fn dims(&self) -> DimensionHeightBounds { self.dims }
        fn border(&self) -> WorldBorderBounds { self.border }
        fn height(&self, x: i32, z: i32) -> i32 { self.ground(x, z) }
        fn within_border(&self, _pos: BlockPos) -> bool { true }
        fn replaceable(&self, pos: BlockPos) -> bool {
            match self.kind {
                TerrainKind::Solid => false,
                _ => pos[1] >= self.ground(pos[0], pos[2]),
            }
        }
        fn solid(&self, pos: BlockPos) -> bool {
            match self.kind {
                TerrainKind::Solid => true,
                _ => pos[1] < self.ground(pos[0], pos[2]),
            }
        }
        fn has_horizontal_axis(&self, _pos: BlockPos) -> bool { true }
        fn set_block(&mut self, pos: BlockPos, block: PortalBlock, flag: i32) {
            self.writes += 1;
            let b = match block {
                PortalBlock::Obsidian => "OBSIDIAN",
                PortalBlock::Air => "AIR",
                PortalBlock::Portal(PortalAxis::X) => "PORTAL_X",
                PortalBlock::Portal(PortalAxis::Z) => "PORTAL_Z",
            };
            let line = format!("{flag}|{b}|{}|{}|{}\n", pos[0], pos[1], pos[2]);
            for byte in line.as_bytes() {
                self.hash ^= *byte as u64;
                self.hash = self.hash.wrapping_mul(0x100000001b3);
            }
        }
    }

    let version = law::basis::cell::DATA_VERSION.to_string();
    for raw in include_str!("../../world/tests/fixtures/jar-boundaries.tsv").lines() {
        let f: Vec<_> = raw.split('\t').collect();
        if f[0] != version || f[1] != "portal_missing" { continue; }
        let spectator = f[2].parse().unwrap();
        let attempted_creation = f[3] == "1";
        let origin = [0, 64, 0];
        let mut terrain = TestPortalWorld {
            kind: TerrainKind::Flat,
            dims: DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
            border: WorldBorderBounds::default_mc(),
            writes: 0,
            hash: 0xcbf29ce484222325,
        };
        assert_eq!(nether_exit(&mut terrain, None, origin, PortalAxis::X, spectator).is_some(), attempted_creation);
        assert_eq!(terrain.writes != 0, attempted_creation);
    }

    let mut rows = 0usize;
    for (idx, raw) in include_str!("data/mounts_breath_void.tsv").lines().enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        rows += 1;
        let f: Vec<_> = raw.split('\t').collect();
        match f[0] {
            "jump_scale" => {
                let c: i32 = f[1].parse().unwrap();
                assert_eq!(player_jump_pending_scale(c).to_bits(), h32(f[2]), "row {idx} jump_scale");
            }
            "boost" => {
                let t: i32 = f[1].parse().unwrap();
                let total: i32 = f[2].parse().unwrap();
                assert_eq!(boost_factor(true, t, total).to_bits(), h32(f[3]), "row {idx} boost");
            }
            "horse_input" => {
                let m = MountState::horse();
                let xxa = f32::from_bits(h32(f[1]));
                let zza = f32::from_bits(h32(f[2]));
                let v = ridden_input_horse(&m, xxa, zza);
                assert_eq!(v.x.to_bits(), h64(f[3]), "row {idx} horse_input x");
                assert_eq!(v.z.to_bits(), h64(f[4]), "row {idx} horse_input z");
            }
            "horse_jump" => {
                let c: i32 = f[1].parse().unwrap();
                let yaw = f32::from_bits(h32(f[2]));
                let scale = player_jump_pending_scale(c);
                let mut m_fwd = MountState::horse();
                m_fwd.yaw = yaw;
                execute_riders_jump_horse(&mut m_fwd, scale, 1.0);
                assert_eq!(m_fwd.velocity.x.to_bits(), h64(f[3]), "row {idx} horse_jump fx");
                assert_eq!(m_fwd.velocity.y.to_bits(), h64(f[4]), "row {idx} horse_jump fy");
                assert_eq!(m_fwd.velocity.z.to_bits(), h64(f[5]), "row {idx} horse_jump fz");
                let mut m_still = MountState::horse();
                m_still.yaw = yaw;
                execute_riders_jump_horse(&mut m_still, scale, 0.0);
                assert_eq!(m_still.velocity.y.to_bits(), h64(f[6]), "row {idx} horse_jump ny");
            }
            "pig_speed" => {
                let mut m = MountState::pig();
                m.boosting = true;
                m.boost_time = f[1].parse().unwrap();
                m.boost_time_total = 140;
                assert_eq!(ridden_speed(&m, false, false).to_bits(), h32(f[2]), "row {idx} pig_speed");
            }
            "strider_speed" => {
                let mut m = MountState::strider();
                m.boosting = true;
                m.boost_time = f[1].parse().unwrap();
                m.boost_time_total = 140;
                let suff = f[2] == "1";
                assert_eq!(ridden_speed(&m, false, suff).to_bits(), h32(f[3]), "row {idx} strider_speed");
            }
            "horse_speed" => {
                let m = MountState::horse();
                assert_eq!(ridden_speed(&m, false, false).to_bits(), h32(f[1]), "row {idx} horse_speed");
            }
            "camel_speed" => {
                let mut m = MountState::camel();
                let sp = f[1] == "1";
                m.dash_cooldown = f[2].parse().unwrap();
                assert_eq!(ridden_speed(&m, sp, false).to_bits(), h32(f[3]), "row {idx} camel_speed");
                assert_eq!(m.speed_attribute.to_bits(), h64(f[4]), "row {idx} camel attr");
            }
            "camel_dash" => {
                let c: i32 = f[1].parse().unwrap();
                let yaw = f32::from_bits(h32(f[2]));
                let mut m = MountState::camel();
                m.yaw = yaw;
                execute_riders_jump_camel(&mut m, player_jump_pending_scale(c));
                assert_eq!(m.velocity.x.to_bits(), h64(f[3]), "row {idx} camel_dash x");
                assert_eq!(m.velocity.y.to_bits(), h64(f[4]), "row {idx} camel_dash y");
                assert_eq!(m.velocity.z.to_bits(), h64(f[5]), "row {idx} camel_dash z");
                assert_eq!(m.dash_cooldown, 55);
            }
            "ghast_in" => {
                let pitch = f32::from_bits(h32(f[1]));
                let zza = f32::from_bits(h32(f[2]));
                let jmp = f[3] == "1";
                let m = MountState::happy_ghast();
                let v = ridden_input_happy_ghast(0.5, zza, pitch, jmp, m.speed_attribute);
                assert_eq!(v.x.to_bits(), h64(f[4]), "row {idx} ghast_in x");
                assert_eq!(v.y.to_bits(), h64(f[5]), "row {idx} ghast_in y");
                assert_eq!(v.z.to_bits(), h64(f[6]), "row {idx} ghast_in z");
            }
            "ghast_yaw" => {
                let mut m = MountState::happy_ghast();
                m.yaw = f32::from_bits(h32(f[1]));
                let tgt = f32::from_bits(h32(f[2]));
                tick_ridden_happy_ghast(&mut m, tgt, 0.0);
                assert_eq!(m.yaw.to_bits(), h32(f[3]), "row {idx} ghast_yaw");
            }
            "spiral" => {
                let ctr = [f[1].parse().unwrap(), f[2].parse().unwrap(), f[3].parse().unwrap()];
                let pts = spiral_around(ctr, 16);
                assert_eq!(pts.len(), f[4].parse::<usize>().unwrap(), "row {idx} spiral count");
                let mut h: u64 = 0xcbf29ce484222325;
                for p in pts {
                    let s = format!("{}|{}|{}\n", p[0], p[1], p[2]);
                    for b in s.as_bytes() {
                        h ^= *b as u64;
                        h = h.wrapping_mul(0x100000001b3);
                    }
                }
                assert_eq!(h, h64(f[5]), "row {idx} spiral hash");
            }
            "poi" => {
                let is_nether = f[1] == "1";
                let origin = [f[2].parse().unwrap(), f[3].parse().unwrap(), f[4].parse().unwrap()];
                let pois: Vec<BlockPos> = f[5].split(';').map(p_pos).collect();
                let expected = if f[6] == "none" { None } else { Some(p_pos(f[6])) };
                let w = TestPortalWorld {
                    kind: TerrainKind::Flat,
                    dims: DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
                    border: WorldBorderBounds::default_mc(),
                    writes: 0,
                    hash: 0xcbf29ce484222325,
                };
                assert_eq!(find_closest_portal_position(&w, origin, is_nether, &pois), expected, "row {idx} poi");
            }
            "portal" => {
                let kind = match f[1] {
                    "flat" => TerrainKind::Flat,
                    "bumpy" => TerrainKind::Bumpy,
                    "solid" => TerrainKind::Solid,
                    other => panic!("bad terrain {other}"),
                };
                let dims = DimensionHeightBounds {
                    min_y: f[2].parse().unwrap(),
                    max_y: f[3].parse().unwrap(),
                    logical_height: f[4].parse().unwrap(),
                };
                let axis = match f[5] {
                    "X" => PortalAxis::X,
                    "Z" => PortalAxis::Z,
                    other => panic!("bad axis {other}"),
                };
                let origin = [f[6].parse().unwrap(), f[7].parse().unwrap(), f[8].parse().unwrap()];
                let expected = if f[9] == "none" { None } else { Some(p_pos(f[9])) };
                let mut w = TestPortalWorld {
                    kind,
                    dims,
                    border: WorldBorderBounds::default_mc(),
                    writes: 0,
                    hash: 0xcbf29ce484222325,
                };
                let got = create_portal(&mut w, origin, axis).map(|r| r.min_corner);
                assert_eq!(got, expected, "row {idx} portal base");
                assert_eq!(w.writes, f[10].parse::<usize>().unwrap(), "row {idx} portal writes");
                assert_eq!(w.hash, h64(f[11]), "row {idx} portal hash");
            }
            "air" => {
                let seed: i64 = f[1].parse().unwrap();
                let mut air: i32 = f[2].parse().unwrap();
                let mode: i32 = f[3].parse().unwrap();
                let bonus = f64::from_bits(h64(f[4]));
                let ticks: u32 = f[5].parse().unwrap();
                let hit_ticks: Vec<u32> = f[6].split(',').filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect();
                let final_air: i32 = f[7].parse().unwrap();
                let traj = h64(f[8]);
                let effects = BreathEffects {
                    water_breathing: mode == 2,
                    conduit_power: false,
                    breath_of_the_nautilus: mode == 3,
                };
                let bubble = mode == 1;
                let invuln = mode == 4;
                let mut rng = JavaRandom::new(seed);
                let mut h: u64 = 0xcbf29ce484222325;
                let mut got_hits = Vec::new();
                for t in 1..=ticks {
                    let (a, hit) = tick_air(air, t <= 600, bubble, &effects, bonus, invuln, false, &mut rng);
                    air = a;
                    if hit.is_some() {
                        got_hits.push(t);
                    }
                    h ^= air as u32 as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
                assert_eq!(got_hits, hit_ticks, "row {idx} air hits");
                assert_eq!(air, final_air, "row {idx} final air");
                assert_eq!(h, traj, "row {idx} air traj");
            }
            "void" => {
                let min_y: i32 = f[1].parse().unwrap();
                let y = f64::from_bits(h64(f[2]));
                assert_eq!(below_world(y, min_y), f[3] == "1", "row {idx} void");
            }
            "bed_const" => {
                assert_eq!(BED_BOUNCE_RESTITUTION.to_bits(), h32(f[1]));
                assert_eq!(BED_FALL_DISTANCE_MULTIPLIER.to_bits(), h64(f[2]));
                assert_eq!(BAD_RESPAWN_EXPLOSION_RADIUS.to_bits(), h32(f[3]));
                assert_eq!(RESPAWN_ANCHOR_MIN_CHARGES, f[4].parse::<u8>().unwrap());
                assert_eq!(RESPAWN_ANCHOR_MAX_CHARGES, f[5].parse::<u8>().unwrap());
                assert!(!BedRule::CAN_SLEEP_WHEN_DARK.explodes);
                assert!(BedRule::CAN_SLEEP_WHEN_DARK.can_sleep.test(true));
                assert!(!BedRule::CAN_SLEEP_WHEN_DARK.can_sleep.test(false));
                assert!(BedRule::EXPLODES.explodes);
                assert_eq!(respawn_anchor_charge(3), Some(4));
                assert_eq!(respawn_anchor_charge(4), None);
                assert_eq!(respawn_anchor_use(0, false), RespawnAnchorUse::Pass);
                assert_eq!(respawn_anchor_use(1, true), RespawnAnchorUse::SetSpawn);
                assert_eq!(respawn_anchor_use(1, false), RespawnAnchorUse::Explode { radius: 5.0, fire: true });
                assert_eq!(respawn_anchor_find_and_use(2, false, true, false), Some(1));
                assert_eq!(respawn_anchor_find_and_use(0, false, true, false), None);
                assert_eq!(respawn_anchor_find_and_use(0, true, true, false), Some(0));
            }
            other => panic!("unexpected row tag {other}"),
        }
    }
    assert_eq!(rows, 593, "expected all 593 jar witness rows");

    // Void 10-tick cadence (FellOutOfWorld + HurtCooldownTick) and respawn reset.
    let world = BoxWorld::default();
    let mut state = PlayerState { profile: law::Profile { invulnerable: true, ..Default::default() }.into(), ..Default::default() };
    apply_server_resource_event(&mut state, &world, ServerResourceEvent::FellOutOfWorld);
    assert_eq!(state.health.to_bits(), (20.0f32 - FELL_OUT_OF_WORLD).to_bits());
    for _ in 0..9 {
        apply_server_resource_event(&mut state, &world, ServerResourceEvent::HurtCooldownTick);
        apply_server_resource_event(&mut state, &world, ServerResourceEvent::FellOutOfWorld);
        assert_eq!(state.health.to_bits(), 16.0f32.to_bits());
    }
    apply_server_resource_event(&mut state, &world, ServerResourceEvent::HurtCooldownTick);
    apply_server_resource_event(&mut state, &world, ServerResourceEvent::FellOutOfWorld);
    assert_eq!(state.health.to_bits(), 12.0f32.to_bits());
    respawn(&mut state, &world);
    assert_eq!((state.health.to_bits(), state.food_level, state.saturation.to_bits(), state.exhaustion.to_bits()),
        (20.0f32.to_bits(), 20, 5.0f32.to_bits(), 0.0f32.to_bits()));
    assert_eq!((state.hurt_cooldown.remaining_ticks, state.hurt_cooldown.last_raw_hit.to_bits()), (0, 0.0f32.to_bits()));
}
