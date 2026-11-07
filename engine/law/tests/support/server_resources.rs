use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Clock {
    world: i64,
    player: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Resources([u32; 5]);

impl Resources {
    const FROZEN_START: Self = Self([0x41a00000, 20, 0x41a00000, 0x40799996, 0]);

    fn read(fields: &[&str]) -> Self {
        let [health, food, saturation, exhaustion, timer] = fields else {
            panic!("expected five raw resource fields");
        };
        Self([bits(health), food.parse().unwrap(), bits(saturation), bits(exhaustion), timer.parse().unwrap()])
    }

    fn initial_state(self) -> PlayerState {
        let [health, food, saturation, exhaustion, timer] = self.0;
        PlayerState {
            health: f32::from_bits(health),
            food_level: food as i32,
            saturation: f32::from_bits(saturation),
            exhaustion: f32::from_bits(exhaustion),
            food_tick_timer: timer,
            ..PlayerState::default()
        }
    }

    fn observed(state: &PlayerState) -> Self {
        Self([state.health.to_bits(), state.food_level as u32, state.saturation.to_bits(),
            state.exhaustion.to_bits(), state.food_tick_timer])
    }

    fn compare(self, expected: Self, point: &str, clock: Clock) -> Result<(), String> {
        let names = ["health", "food", "saturation", "exhaustion", "food timer"];
        for ((name, actual), expected) in names.into_iter().zip(self.0).zip(expected.0) {
            if actual != expected {
                return Err(format!("{point} {clock:?}: {name}, predicted {actual:08x}, observed {expected:08x}"));
            }
        }
        Ok(())
    }
}

fn bits(value: &str) -> u32 {
    assert_eq!(value.len(), 8, "expected raw f32 bits");
    u32::from_str_radix(value, 16).unwrap()
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Start(Resources),
    Charge(u32),
    Damage(u32),
    Regen,
    Food,
    Depleted(Resources),
    Refill,
    AfterRegen(Resources),
    AfterFood(Resources),
    End(Resources),
}

impl Kind {
    fn input(self, player_tick: i32) -> Option<ServerResourceEvent> {
        Some(match self {
            Self::Charge(raw) => ServerResourceEvent::Exhaustion(f32::from_bits(raw)),
            Self::Damage(raw) => ServerResourceEvent::Damage(f32::from_bits(raw)),
            Self::Regen => ServerResourceEvent::Regeneration(player_tick),
            Self::Food => ServerResourceEvent::FoodTick,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct Row {
    clock: Clock,
    kind: Kind,
}

fn parse_trace(text: &str, run: &str) -> Vec<Row> {
    parse_timeline(text, &format!("resource-v1,{run}"), false)
}

fn parse_timeline(text: &str, header: &str, corpus: bool) -> Vec<Row> {
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(header), "resource witness version or identity");
    let rows: Vec<_> = lines.enumerate().map(|(order, line)| {
        let fields: Vec<_> = line.split(',').collect();
        let [sequence, world, player, kind, payload @ ..] = fields.as_slice() else {
            panic!("invalid trace row {line}");
        };
        assert_eq!(sequence.parse::<usize>().unwrap(), order, "trace order at row {line}");
        let clock = Clock { world: world.parse().unwrap(), player: player.parse().unwrap() };
        let kind = match (*kind, payload) {
            ("start", fields) if order == 0 => Kind::Start(Resources::read(fields)),
            ("charge", [raw]) => Kind::Charge(bits(raw)),
            ("damage", [raw]) => Kind::Damage(bits(raw)),
            ("regen", []) => Kind::Regen,
            ("food", []) => Kind::Food,
            ("depleted", fields) if !corpus => Kind::Depleted(Resources::read(fields)),
            ("refill", []) if !corpus => Kind::Refill,
            ("after-regen", fields) if corpus => Kind::AfterRegen(Resources::read(fields)),
            ("after-food", fields) if corpus => Kind::AfterFood(Resources::read(fields)),
            ("end", fields) => Kind::End(Resources::read(fields)),
            _ => panic!("invalid trace row {line}"),
        };
        Row { clock, kind }
    }).collect();
    assert!(matches!(rows.first().map(|r| r.kind), Some(Kind::Start(_))), "missing start");
    assert!(matches!(rows.last().map(|r| r.kind), Some(Kind::End(_))), "missing end");
    let mut previous = rows[0].clock;
    let mut last_food = previous;
    let mut waiting_for_food = None;
    for (index, row) in rows.iter().enumerate().skip(1) {
        assert!(row.clock.world >= previous.world && row.clock.player >= previous.player,
            "trace clock went backwards at event {index}");
        previous = row.clock;
        match row.kind {
            Kind::Regen => {
                assert!(waiting_for_food.is_none(), "missing FoodData tick before event {index}");
                assert_eq!(row.clock, Clock { world: last_food.world + 1, player: last_food.player + 1 },
                    "missing or repeated server tick at event {index}");
                waiting_for_food = Some(row.clock);
                if corpus {
                    assert!(matches!(rows.get(index + 1), Some(Row { clock, kind: Kind::AfterRegen(_) }) if *clock == row.clock),
                        "missing after-regen checkpoint at event {index}: {row:?}");
                }
            }
            Kind::Food => {
                assert_eq!(waiting_for_food.take(), Some(row.clock), "FoodData without matching regeneration at event {index}");
                last_food = row.clock;
                if corpus {
                    assert!(matches!(rows.get(index + 1), Some(Row { clock, kind: Kind::AfterFood(_) }) if *clock == row.clock),
                        "missing after-food checkpoint at event {index}: {row:?}");
                }
            }
            Kind::AfterRegen(_) => assert!(matches!(rows[index - 1].kind, Kind::Regen), "extra after-regen at event {index}"),
            Kind::AfterFood(_) => assert!(matches!(rows[index - 1].kind, Kind::Food), "extra after-food at event {index}"),
            Kind::Depleted(_) => assert!(matches!(rows[index - 1].kind, Kind::Food) && row.clock == last_food,
                "depletion must immediately follow FoodData"),
            Kind::Refill => assert_eq!(waiting_for_food, Some(row.clock), "refill outside regeneration"),
            Kind::End(_) => {
                assert_eq!(index + 1, rows.len(), "early end");
                assert_eq!(row.clock, last_food, "missing final player tick");
                assert!(waiting_for_food.is_none(), "end inside server tick");
            }
            _ => {}
        }
    }
    rows
}

#[derive(Debug)]
struct Checkpoints {
    depleted: Option<(Clock, Resources)>,
    refill: Option<Clock>,
    end: (Clock, Resources),
}

fn replay_trace(rows: &[Row], world: &BoxWorld, phase_shift: i32) -> Checkpoints {
    let Kind::Start(initial) = rows[0].kind else { unreachable!() };
    let mut state = initial.initial_state();
    let mut depleted = None;
    let mut refill = None;
    for row in rows {
        let Some(input) = row.kind.input(row.clock.player + phase_shift) else { continue };
        let (food, saturation) = (state.food_level, state.saturation);
        apply_server_resource_event(&mut state, world, input);
        match row.kind {
            Kind::Food if depleted.is_none() && (state.food_level < food || state.saturation < saturation) => {
                depleted = Some((row.clock, Resources::observed(&state)));
            }
            Kind::Regen if depleted.is_some() && refill.is_none() && state.saturation > saturation => {
                refill = Some(row.clock);
            }
            _ => {}
        }
    }
    Checkpoints { depleted, refill, end: (rows.last().unwrap().clock, Resources::observed(&state)) }
}

fn compare(actual: &Checkpoints, rows: &[Row]) -> Result<(), String> {
    let mut saw_depletion = false;
    let mut saw_refill = false;
    for row in rows {
        match row.kind {
            Kind::Depleted(expected) => {
                let Some((clock, resources)) = actual.depleted else {
                    return Err(format!("first depleted tick {:?}: replay never depleted", row.clock));
                };
                if saw_depletion || clock != row.clock {
                    return Err(format!("first depleted tick: predicted {clock:?}, observed {:?}", row.clock));
                }
                resources.compare(expected, "first depleted tick", clock)?;
                saw_depletion = true;
            }
            Kind::Refill => {
                if saw_refill || actual.refill != Some(row.clock) {
                    return Err(format!("first refill: predicted {:?}, observed {:?}", actual.refill, row.clock));
                }
                saw_refill = true;
            }
            Kind::End(expected) => {
                if saw_depletion != actual.depleted.is_some() || saw_refill != actual.refill.is_some() {
                    return Err(format!("unobserved checkpoint: {actual:?}"));
                }
                actual.end.1.compare(expected, "end", row.clock)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(dead_code)] // Used by the standalone 1.6f verifier.
pub(super) fn verify_landing_resources(
    text: &str, run: &str, world: &BoxWorld, damage: Option<(i64, i32, u32)>,
) {
    let rows = parse_trace(text, run);
    let observed_damage: Vec<_> = rows.iter().filter_map(|row| match row.kind {
        Kind::Damage(bits) => Some((row.clock.world, row.clock.player, bits)),
        _ => None,
    }).collect();
    assert_eq!(observed_damage.as_slice(), damage.as_slice(), "{run}: server damage event order");
    let predicted = replay_trace(&rows, world, 0);
    compare(&predicted, &rows).unwrap_or_else(|failure| panic!("{run}: {failure}"));
    println!("{run}: server resource checkpoints {predicted:?}");
}

#[allow(dead_code)] // Used by the standalone terrain verifier.
pub(super) fn verify_corpus_resources(
    text: &str, run: &str, identity: (&str, &str), world: &BoxWorld, client_start: &PlayerState,
    damage: &[(i64, i32, u32)], fences: [(i64, i32); 2],
) {
    let (recipient, entity) = identity;
    let rows = parse_timeline(text, &format!("resource-v2,{run},{recipient},{entity}"), true);
    let first = rows[0];
    let last = rows.last().unwrap();
    assert_eq!([(first.clock.world, first.clock.player), (last.clock.world, last.clock.player)], fences,
        "{run}: resource and motion fences disagree");
    assert!(last.clock.player > first.clock.player, "{run}: empty server resource interval");
    let Kind::Start(initial) = first.kind else { unreachable!() };
    let mut state = initial.initial_state();
    assert_eq!((state.health.to_bits(), state.food_level, state.saturation.to_bits(), state.food_tick_timer),
        (20.0f32.to_bits(), 20, 20.0f32.to_bits(), 0), "{run}: unsupported server start resources");
    assert!((0.0..=40.0).contains(&state.exhaustion), "{run}: invalid initial exhaustion");
    assert_eq!((state.health.to_bits(), state.food_level), (client_start.health.to_bits(), client_start.food_level),
        "{run}: initial client health or food differs from server");
    let mut expected_damage = damage.iter();
    let mut checkpoints = 0;
    for (index, row) in rows.iter().enumerate().skip(1) {
        if let Kind::Charge(raw) | Kind::Damage(raw) = row.kind {
            let value = f32::from_bits(raw);
            assert!(value.is_finite() && value >= 0.0, "{run}: invalid amount at event {index}: {row:?}");
        }
        if let Kind::Damage(raw) = row.kind {
            assert_eq!(expected_damage.next(), Some(&(row.clock.world, row.clock.player, raw)),
                "{run}: unproven server damage at event {index}: {row:?}");
        }
        if let Some(input) = row.kind.input(row.clock.player) {
            apply_server_resource_event(&mut state, world, input);
        } else if let Kind::AfterRegen(expected) | Kind::AfterFood(expected) | Kind::End(expected) = row.kind {
            Resources::observed(&state).compare(expected, &format!("event {index}"), row.clock)
                .unwrap_or_else(|failure| panic!("{run}: {failure}"));
            checkpoints += 1;
        }
        assert!(state.health > 0.0 && state.food_level > 6,
            "{run}: unsupported resource eligibility at event {index}: {row:?}");
    }
    assert_eq!(expected_damage.next(), None, "{run}: witnessed landing damage absent from resource timeline");
    println!("{run}: {checkpoints} ordered server resource checkpoints, {} events, sealed {:?}", rows.len(), Resources::observed(&state));
}

fn check_controls(rows: &[Row], world: &BoxWorld) {
    let wrong_phase = compare(&replay_trace(rows, world, 1), rows).expect_err("one-tick phase shift survived");
    assert!(wrong_phase.starts_with("first refill"), "{wrong_phase}");
    println!("Phase control failed at {wrong_phase}");

    let depletion = rows.iter().position(|r| matches!(r.kind, Kind::Depleted(_))).unwrap();
    let refill = rows.iter().position(|r| matches!(r.kind, Kind::Refill)).unwrap();
    let charge = rows[..depletion].iter().rposition(|r| matches!(r.kind, Kind::Charge(_))).unwrap();
    let mut moved = rows.to_vec();
    let mut input = moved.remove(charge);
    input.clock = rows[refill].clock;
    moved.insert(refill, input);
    let wrong_order = compare(&replay_trace(&moved, world, 0), rows).expect_err("delayed charge survived");
    assert!(wrong_order.starts_with("first depleted tick"), "{wrong_order}");
    println!("Charge-order control failed at {wrong_order}");
}

fn run_case(variable: &str, difficulty: u8) -> PlayerState {
    let recording = PathBuf::from(std::env::var_os(variable).expect(variable));
    let run = recording.file_stem().unwrap().to_str().unwrap();
    let trace_path = recording.with_extension("resources.csv");
    let trace_text = fs::read_to_string(&trace_path).unwrap();
    let rows = parse_trace(&trace_text, run);
    let fixture = fs::read_to_string(recording.with_extension("world.jsonl")).unwrap();
    let (world, mut movement, server_end) = parse_world(&fixture);
    let start = movement.clone();
    assert_eq!(world.difficulty, difficulty, "{run} difficulty");
    assert!(world.natural_regeneration, "{run} natural regeneration disabled");
    assert_eq!(Resources::observed(&start), Resources::FROZEN_START, "{run} fixture start");
    let Kind::Start(initial) = rows[0].kind else { unreachable!() };
    assert_eq!(initial, Resources::FROZEN_START, "{run} authoritative start");
    assert_eq!(start.position, Vec3::new(4112.5, -60.0, 4100.5), "{run} start position");
    assert!(start.on_ground && start.velocity.horizontal_sqr() == 0.0, "{run} start at rest");

    let text = fs::read_to_string(&recording).unwrap();
    let (ticks, _) = replay_text(&text, &world, &mut movement);
    assert_eq!(ticks, 100, "{run} frozen recording length");
    for (tick, line) in text.lines().skip(1).take(100).enumerate() {
        let row = Object::parse(line);
        let expected = Input { jump: [0, 20, 40].contains(&tick), ..Input::default() };
        assert_eq!(input(&row.object("inputs")), expected, "{run} input {tick}");
    }
    let server_end = server_end.expect("missing legacy serverEnd");
    let server_end = Object::parse(&server_end);
    assert!(server_end.boolean("atRest"), "{run} end at rest");
    println!("{run}: {ticks} bit-exact movement ticks");

    let predicted = replay_trace(&rows, &world, 0);
    compare(&predicted, &rows).unwrap_or_else(|failure| panic!("{run}: {failure}. Trace: {}", trace_path.display()));
    assert_server(&predicted.end.1.initial_state(), &server_end);
    assert!(predicted.depleted.is_some(), "{run} no distinguishing depletion");
    assert_eq!(predicted.refill.is_some(), difficulty == 0, "{run} distinguishing refill");
    let inputs: Vec<_> = rows.iter().enumerate()
        .filter(|(_, row)| matches!(row.kind, Kind::Charge(_) | Kind::Damage(_))).collect();
    assert_eq!(inputs.len(), 3, "{run} expected three jumps, observed {inputs:?}");
    for (index, row) in inputs {
        assert!(matches!(row.kind, Kind::Charge(raw) if raw == 0.05f32.to_bits()),
            "{run} unexpected input at event {index}: {row:?}");
    }
    println!("{run}: {predicted:?}");
    if difficulty == 0 {
        check_controls(&rows, &world);
    }
    start
}

#[test]
#[ignore = "Integrator supplies the frozen 1.5h recordings from disposable rigs"]
fn authoritative_resource_pair() {
    let peaceful = run_case("L0_RESOURCE_PEACEFUL", 0);
    let normal = run_case("L0_RESOURCE_NORMAL", 2);
    // The fixture player state excludes the independent client and server clocks.
    assert_eq!(format!("{peaceful:?}"), format!("{normal:?}"), "pair initial movement states");
}

#[test]
fn authoritative_resource_controls() {
    // Synthetic checkpoints pin the gate logic independently of live availability.
    let mut trace = String::from("resource-v1,offline\n0,100,10,start,41a00000,20,41a00000,40799996,0\n");
    let mut order = 1;
    for player in 11..=40 {
        let world = player + 90;
        let mut add = |data: &str| {
            trace.push_str(&format!("{order},{world},{player},{data}\n"));
            order += 1;
        };
        add("regen");
        if player == 20 { add("refill"); }
        if player == 11 {
            for _ in 0..3 { add("charge,3d4ccccd"); }
        }
        add("food");
        if player == 11 { add("depleted,41a00000,20,41980000,3d4ccc00,0"); }
        if player == 40 { add("end,41a00000,20,41a00000,3d4ccc00,0"); }
    }
    let mut world = BoxWorld::default();
    world.difficulty = 0;
    world.natural_regeneration = true;
    let rows = parse_trace(&trace, "offline");
    compare(&replay_trace(&rows, &world, 0), &rows).unwrap();
    check_controls(&rows, &world);
}

#[test]
fn authoritative_damage_between_regeneration_and_food() {
    let trace = concat!(
        "resource-v1,damage\n",
        "0,109,19,start,41a00000,20,41a00000,00000000,0\n",
        "1,110,20,regen\n",
        "2,110,20,charge,3e800000\n",
        "3,110,20,damage,3f800000\n",
        "4,110,20,food\n",
        "5,110,20,end,41980000,20,41a00000,3e800000,1\n",
    );
    let mut world = BoxWorld::default();
    world.difficulty = 0;
    world.natural_regeneration = true;
    let rows = parse_trace(trace, "damage");
    compare(&replay_trace(&rows, &world, 0), &rows).unwrap();
}

#[test]
fn corpus_refuses_missing_events_and_checkpoints() {
    let header = "resource-v2,run,player,7";
    let events = [
        "109,19,start,41a00000,20,41a00000,00000000,0",
        "110,20,regen",
        "110,20,after-regen,41a00000,20,41a00000,00000000,0",
        "110,20,charge,3d4ccccd",
        "110,20,food",
        "110,20,after-food,41a00000,20,41a00000,3d4ccccd,0",
        "110,20,end,41a00000,20,41a00000,3d4ccccd,0",
    ];
    let verify = |omit| {
        let mut text = format!("{header}\n");
        for (index, event) in events.iter().enumerate().filter(|(index, _)| Some(*index) != omit).enumerate() {
            text.push_str(&format!("{index},{}\n", event.1));
        }
        let mut world = BoxWorld::default();
        world.difficulty = 2;
        world.natural_regeneration = true;
        let start = PlayerState { health: 20.0, food_level: 20, ..PlayerState::default() };
        verify_corpus_resources(&text, "run", ("player", "7"), &world, &start, &[], [(109, 19), (110, 20)]);
    };
    verify(None);
    for (omit, diagnostic) in [(3, "event 4 Clock"), (5, "missing after-food checkpoint")] {
        let error = std::panic::catch_unwind(|| verify(Some(omit))).unwrap_err();
        assert!(error.downcast_ref::<String>().unwrap().contains(diagnostic));
    }
}
