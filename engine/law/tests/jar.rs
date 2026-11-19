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
