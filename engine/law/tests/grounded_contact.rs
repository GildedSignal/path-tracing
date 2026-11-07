use law::{
    Block, BoxWorld, ContactDamage, GroundedContact, HurtCooldown, Input, PlayerState, Pose,
    ServerResourceEvent, Vec3, apply_grounded_step, apply_server_resource_event, tick,
    tick_with_contact,
};
use std::{env, fs, path::Path};

const HOT_FLOOR: GroundedContact = GroundedContact::DamageUnlessCareful(ContactDamage::HotFloor);
const PAD: [i32; 3] = [4112, -61, 4100];

fn normal_world() -> BoxWorld {
    let mut world = BoxWorld::default();
    world.natural_regeneration = false;
    world
}

fn bits(field: &str) -> u32 {
    u32::from_str_radix(field, 16).unwrap()
}

fn assert_resources(state: &PlayerState, health: &str, cooldown: &str, at: &str) {
    assert_eq!(state.health.to_bits(), bits(health), "{at} health bits");
    assert_eq!(
        state.hurt_cooldown.remaining_ticks,
        cooldown.parse::<i32>().unwrap(),
        "{at} cooldown"
    );
}

fn replay_standing(path: &Path, case: &str, block: &str, effect: GroundedContact) {
    let trace = fs::read_to_string(path.join(case).join("server").join(format!("{case}.tsv")))
        .expect("read frozen contact trace");
    let mut lines = trace.lines();
    assert_eq!(lines.next().unwrap(), format!("contact-v1\t{case}\tPathBot"));
    let mut rows = lines.enumerate().map(|(index, row)| {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1, "{case} sequence");
        fields
    }).peekable();

    let mut world = normal_world();
    world.set(PAD, Block { grounded_contact: effect, ..Block::full() });
    let predecessor = rows.next().unwrap();
    assert_eq!((predecessor[1], predecessor[14]), ("P", "minecraft:stone"));
    let position = |row: &[&str]| {
        Vec3::new(row[4].parse().unwrap(), row[5].parse().unwrap(), row[6].parse().unwrap())
    };
    let mut server = PlayerState {
        position: position(&predecessor),
        velocity: Vec3::new(0.0, -0.08 * 0.98f32 as f64, 0.0),
        health: f32::from_bits(bits(predecessor[28])),
        hurt_cooldown: HurtCooldown {
            remaining_ticks: predecessor[29].parse().unwrap(),
            last_raw_hit: 0.0,
        },
        on_ground: true,
        main_supporting_block: Some(PAD),
        ..PlayerState::default()
    };
    let mut local = server.clone();
    let previous_world_tick: i64 = predecessor[2].parse().unwrap();
    let previous_entity_tick: i32 = predecessor[3].parse().unwrap();

    for tick_index in 0..32 {
        let world_tick = (previous_world_tick + i64::from(tick_index) + 1).to_string();
        let entity_tick = previous_entity_tick + tick_index + 1;
        let clock = entity_tick.to_string();
        let at = format!("{case} tick {tick_index}");
        let phase = |state: &PlayerState, tag, row: &[&str]| {
            assert_eq!(row.len(), 30, "{at} {tag} row width");
            assert_eq!(&row[1..4], &[tag, world_tick.as_str(), clock.as_str()], "{at} phase");
            assert_eq!(row[14], block, "{at} {tag} block");
            assert_resources(state, row[28], row[29], &format!("{at} {tag}"));
        };

        apply_server_resource_event(&mut server, &world, ServerResourceEvent::HurtCooldownTick);
        phase(&server, "B", &rows.next().unwrap());
        apply_server_resource_event(&mut server, &world, ServerResourceEvent::Regeneration(entity_tick));
        let dispatch = rows.next().unwrap();
        phase(&server, "C", &dispatch);

        let request = tick_with_contact(&mut local, &world, &Input::default());
        assert_eq!(local.position, position(&dispatch), "{at} position");
        assert_eq!(local.on_ground, dispatch[7].parse::<bool>().unwrap(), "{at} onGround");
        assert_eq!((local.pose, dispatch[8]), (Pose::Standing, "STANDING"), "{at} pose");
        assert_eq!(local.shift_key_down, dispatch[9].parse::<bool>().unwrap(), "{at} careful");
        assert_eq!((local.main_supporting_block, dispatch[12], dispatch[13]),
            (Some(PAD), "true", "true"), "{at} support");
        assert_eq!(local.health.to_bits(), bits(predecessor[28]), "{at} local health");
        assert_eq!(local.hurt_cooldown.remaining_ticks, 0, "{at} local cooldown");

        let damage = rows.next_if(|row| row[1] == "R");
        assert_eq!(request.is_some(), damage.is_some(), "{at} request presence");
        if let (Some(request), Some(row)) = (request, damage) {
            assert_eq!(row.len(), 12, "{at} request row width");
            assert_eq!(&row[2..5], &[world_tick.as_str(), clock.as_str(), "true"], "{at} request phase");
            let source = match request { ContactDamage::HotFloor => "minecraft:hot_floor" };
            assert_eq!(source, row[5], "{at} request source");
            assert_eq!(request.raw_amount().to_bits(), bits(row[6]), "{at} raw amount");
            assert_resources(&server, row[8], row[10], &format!("{at} request entry"));
            let before = server.health;
            apply_server_resource_event(&mut server, &world, ServerResourceEvent::UnmodifiedContactDamage(request));
            assert_resources(&server, row[9], row[11], &format!("{at} request return"));
            assert_eq!(server.health < before, row[7].parse::<bool>().unwrap(), "{at} acceptance");
        }
        apply_server_resource_event(&mut server, &world, ServerResourceEvent::FoodTick);
        phase(&server, "E", &rows.next().unwrap());
    }
    let seal = rows.next().unwrap();
    assert_eq!(&seal[1..3], &["Z", "32"], "{case} seal");
    assert!(rows.next().is_none(), "{case} extra events after seal");
}

#[test]
#[ignore = "requires the integrator's frozen contact-v1 TSVs in CONTACT_EVIDENCE"]
fn standing_contact_trace() {
    let evidence = env::var_os("CONTACT_EVIDENCE").expect("set CONTACT_EVIDENCE");
    let path = Path::new(&evidence);
    replay_standing(path, "magma-standing", "minecraft:magma_block", HOT_FLOOR);
    replay_standing(path, "netherrack-standing", "minecraft:netherrack", GroundedContact::None);
}

#[test]
fn grounded_step_uses_legacy_support_and_shift_flag() {
    let mut world = normal_world();
    world.set([0, -1, 0], Block { grounded_contact: HOT_FLOOR, ..Block::full() });
    let mut state = PlayerState {
        position: Vec3::new(1.0625, 0.0, 0.5),
        main_supporting_block: Some([0, -2, 0]),
        on_ground: true,
        pose: Pose::Crouching,
        ..PlayerState::default()
    };
    assert_eq!(apply_grounded_step(&mut state, &world), Some(ContactDamage::HotFloor));
    state.pose = Pose::Standing;
    state.shift_key_down = true;
    assert_eq!(apply_grounded_step(&mut state, &world), None);
    state.shift_key_down = false;
    state.on_ground = false;
    assert_eq!(apply_grounded_step(&mut state, &world), None);
    state.on_ground = true;
    state.no_physics = true;
    assert_eq!(apply_grounded_step(&mut state, &world), None);
    state.no_physics = false;
    state.main_supporting_block = None;
    assert_eq!(apply_grounded_step(&mut state, &world), None);
}

#[test]
fn cooldown_refuses_equal_hits_and_applies_only_the_excess_of_a_stronger_hit() {
    let world = normal_world();
    let mut state = PlayerState {
        hurt_cooldown: HurtCooldown { remaining_ticks: 11, last_raw_hit: 0.5 },
        ..PlayerState::default()
    };
    let hit = || ServerResourceEvent::UnmodifiedContactDamage(ContactDamage::HotFloor);
    apply_server_resource_event(&mut state, &world, hit());
    assert_eq!(state.health.to_bits(), 19.5f32.to_bits());
    assert_eq!(state.hurt_cooldown, HurtCooldown { remaining_ticks: 11, last_raw_hit: 1.0 });
    assert_eq!(state.exhaustion.to_bits(), 0.1f32.to_bits());

    apply_server_resource_event(&mut state, &world, hit());
    assert_eq!(state.health.to_bits(), 19.5f32.to_bits());
    assert_eq!(state.hurt_cooldown, HurtCooldown { remaining_ticks: 11, last_raw_hit: 1.0 });
    assert_eq!(state.exhaustion.to_bits(), 0.1f32.to_bits());

    apply_server_resource_event(&mut state, &world, ServerResourceEvent::HurtCooldownTick);
    assert_eq!(state.hurt_cooldown.remaining_ticks, 10);
    apply_server_resource_event(&mut state, &world, hit());
    assert_eq!(state.health.to_bits(), 18.5f32.to_bits());
    assert_eq!(state.hurt_cooldown, HurtCooldown { remaining_ticks: 20, last_raw_hit: 1.0 });
    assert_eq!(state.exhaustion.to_bits(), 0.2f32.to_bits());
}

#[test]
fn local_step_and_observed_damage_keep_their_own_boundaries() {
    let mut world = normal_world();
    world.set([0, -1, 0], Block { grounded_contact: HOT_FLOOR, ..Block::full() });
    let mut state = PlayerState {
        position: Vec3::new(0.5, 0.0, 0.5),
        velocity: Vec3::new(0.0, -0.08 * 0.98f32 as f64, 0.0),
        on_ground: true,
        hurt_cooldown: HurtCooldown { remaining_ticks: 19, last_raw_hit: 1.0 },
        ..PlayerState::default()
    };
    let cooldown = state.hurt_cooldown;
    tick(&mut state, &world, &Input::default());
    assert_eq!(state.health.to_bits(), 20.0f32.to_bits());
    apply_server_resource_event(&mut state, &world, ServerResourceEvent::Damage(1.0));
    assert_eq!(state.health.to_bits(), 19.0f32.to_bits());
    assert_eq!(state.exhaustion.to_bits(), 0.0f32.to_bits());
    assert_eq!(state.hurt_cooldown, cooldown);
}
