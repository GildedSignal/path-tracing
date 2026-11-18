//! Shared living server-side pathfinding mind driving all bots over shared memory.
mod protocol;
mod shm;
mod world_mirror;

use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use law::{BlockPos, PlayerState, Vec3, World, codec::Interner};
use plan::{Goal, Kit, PlanOutcome, source::GeneratorSuperflatSource, store::GeneratorStore};
use protocol::{
    BotSyncPayload, FLAG_ARRIVED, FLAG_HORIZ_COL, FLAG_ON_GROUND, FLAG_SETTLED, FLAG_START_SPRINT,
    FLAG_STOP_SPRINT, MSG_BLOCK_DELTA, MSG_BOT_GOAL, MSG_BOT_PLAN_STATUS, MSG_BOT_REMOVE,
    MSG_BOT_SELF_SYNC, MSG_BOT_SPAWN, MSG_BOT_STOP, MSG_BOT_TICK_CMD, MSG_CHUNK_UNLOAD,
    MSG_SECTION_AIR, MSG_SECTION_RLE, MSG_TICK_BEGIN, MSG_TICK_END, MSG_WORLD_RESET,
    MSG_WORLD_RULES, PLAN_STATUS_ALREADY_THERE, PLAN_STATUS_NEEDS_JOINTS, PLAN_STATUS_READY,
    PLAN_STATUS_REFUSED, Reader, Writer,
};
use shm::Shm;
use words::Arguments;
use world_mirror::WorldMirror;

enum BotPhase {
    Idle,
    Executing {
        steps: Vec<(u16, Arguments, u32)>,
        idx: usize,
        elapsed: u32,
        goal_air: BlockPos,
        corridor: (BlockPos, BlockPos),
    },
    Settling,
}

struct BotSlot {
    state: PlayerState,
    phase: BotPhase,
    pending_replan: Option<BlockPos>,
}

impl BotSlot {
    fn from_sync(p: &BotSyncPayload) -> Self {
        Self {
            state: state_from_sync(p),
            phase: BotPhase::Idle,
            pending_replan: None,
        }
    }

    fn apply_sync(&mut self, p: &BotSyncPayload, reason: u32) {
        self.state = state_from_sync(p);
        if reason != 0 {
            self.phase = BotPhase::Idle;
            self.pending_replan = None;
        }
    }
}

fn state_from_sync(p: &BotSyncPayload) -> PlayerState {
    PlayerState {
        position: Vec3::new(p.x, p.y, p.z),
        velocity: Vec3::new(p.vx, p.vy, p.vz),
        yaw: p.yaw,
        pitch: p.pitch,
        on_ground: p.on_ground,
        sprinting: p.sprinting,
        sprint_speed_modifier: p.sprinting,
        health: p.health,
        saturation: p.saturation,
        exhaustion: p.exhaustion,
        food_level: p.food_level,
        food_tick_timer: p.food_tick_timer,
        hurt_cooldown: law::HurtCooldown {
            remaining_ticks: p.hurt_rem,
            last_raw_hit: p.last_hurt,
        },
        ..PlayerState::default()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn parse_shm_arg() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--shm"
            && let Some(p) = args.next()
        {
            return PathBuf::from(p);
        }
    }
    PathBuf::from(r"D:\path-tracing-run\server-mind.shm")
}

fn send_plan_status(
    shm: &Shm,
    w: &mut Writer,
    bot_id: u32,
    status: u32,
    predicted_ticks: u32,
    solve_ns: u64,
    msg: &str,
) {
    w.clear();
    w.u32(bot_id);
    w.u32(status);
    w.u32(predicted_ticks);
    w.u64(solve_ns);
    w.str(msg);
    let _ = shm.push_e2s(MSG_BOT_PLAN_STATUS, &w.buf);
}

fn send_tick_cmd(
    shm: &Shm,
    w: &mut Writer,
    bot_id: u32,
    keys: u32,
    flags: u32,
    state: &PlayerState,
) {
    w.clear();
    w.u32(bot_id);
    w.u32(keys);
    w.u32(flags);
    w.f64(state.position.x);
    w.f64(state.position.y);
    w.f64(state.position.z);
    w.f32(state.yaw);
    w.f32(state.pitch);
    let _ = shm.push_e2s(MSG_BOT_TICK_CMD, &w.buf);
}

fn corridor_bounds(
    p: BlockPos,
    a: BlockPos,
    goal_solid: BlockPos,
    min_y: i32,
    max_y: i32,
) -> (BlockPos, BlockPos) {
    let cl = [
        p[0].min(goal_solid[0]) - 8,
        (a[1].min(goal_solid[1]) - 1).max(min_y),
        p[2].min(goal_solid[2]) - 8,
    ];
    let ch = [
        p[0].max(goal_solid[0]) + 8,
        (p[1].max(goal_solid[1]) + 8).min(max_y),
        p[2].max(goal_solid[2]) + 8,
    ];
    (cl, ch)
}

fn solve_bot(
    bot_id: u32,
    bot: &mut BotSlot,
    goal_air: BlockPos,
    world: &WorldMirror,
    store: &GeneratorStore,
    codec: &mut Interner,
    shm: &Shm,
    w: &mut Writer,
) {
    let goal_solid = [goal_air[0], goal_air[1] - 1, goal_air[2]];
    let p = [
        bot.state.position.x.floor() as i32,
        bot.state.position.y.floor() as i32,
        bot.state.position.z.floor() as i32,
    ];
    let support = bot
        .state
        .main_supporting_block
        .filter(|&c| world.loaded(c) && !world.is_air(c))
        .or_else(|| {
            (1..=8)
                .map(|d| [p[0], p[1] - d, p[2]])
                .find(|&c| world.loaded(c) && !world.is_air(c))
        });
    let Some(a) = support else {
        bot.phase = BotPhase::Idle;
        send_plan_status(
            shm,
            w,
            bot_id,
            PLAN_STATUS_REFUSED,
            0,
            0,
            "no ground under bot within 8 blocks",
        );
        return;
    };
    if bot.state.main_supporting_block.is_none() && bot.state.on_ground {
        bot.state.main_supporting_block = Some(a);
    }

    let (cl, ch) = corridor_bounds(p, a, goal_solid, world.min_loaded_y(), world.max_loaded_y());

    if let Err(why) = world.check_corridor_valid(cl, ch) {
        bot.phase = BotPhase::Idle;
        send_plan_status(shm, w, bot_id, PLAN_STATUS_REFUSED, 0, 0, &why);
        return;
    }

    let corridor = match GeneratorSuperflatSource::from_world(world, (cl, ch), a) {
        Ok(c) => c,
        Err(why) => {
            bot.phase = BotPhase::Idle;
            send_plan_status(shm, w, bot_id, PLAN_STATUS_REFUSED, 0, 0, &why);
            return;
        }
    };

    let t0 = Instant::now();
    let solved = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        plan::solve(
            &bot.state,
            Goal { cell: goal_solid },
            Kit::BareLegs,
            &corridor,
            store,
            codec,
            u64::MAX,
        )
    }));
    let solve_ns = t0.elapsed().as_nanos().min(u64::MAX as u128) as u64;

    let result = match solved {
        Ok(r) => r,
        Err(panic_payload) => {
            let msg = panic_payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic_payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "planner panic".to_owned());
            bot.phase = BotPhase::Idle;
            send_plan_status(shm, w, bot_id, PLAN_STATUS_REFUSED, 0, solve_ns, &msg);
            return;
        }
    };

    match result.outcome {
        PlanOutcome::Ready(found) => {
            if found.ticks == 0 || found.words.is_empty() {
                bot.phase = BotPhase::Idle;
                send_plan_status(
                    shm,
                    w,
                    bot_id,
                    PLAN_STATUS_ALREADY_THERE,
                    0,
                    solve_ns,
                    "Already there.",
                );
            } else {
                let steps: Vec<(u16, Arguments, u32)> = found
                    .words
                    .iter()
                    .map(|step| (step.program, step.args, step.ticks))
                    .collect();
                let ticks = found.ticks.min(u32::MAX as u64) as u32;
                bot.phase = BotPhase::Executing {
                    steps,
                    idx: 0,
                    elapsed: 0,
                    goal_air,
                    corridor: (cl, ch),
                };
                send_plan_status(shm, w, bot_id, PLAN_STATUS_READY, ticks, solve_ns, "");
            }
        }
        PlanOutcome::NeedsJoints { exception, .. } => {
            bot.phase = BotPhase::Idle;
            send_plan_status(
                shm,
                w,
                bot_id,
                PLAN_STATUS_NEEDS_JOINTS,
                0,
                solve_ns,
                &format!("NeedsJoints at {exception:?}"),
            );
        }
        PlanOutcome::Unknown(refusal) => {
            bot.phase = BotPhase::Idle;
            let why = result
                .explanation
                .unwrap_or_else(|| format!("{refusal:?}"));
            send_plan_status(shm, w, bot_id, PLAN_STATUS_REFUSED, 0, solve_ns, &why);
        }
    }
}

fn is_settled(state: &PlayerState) -> bool {
    state.on_ground
        && state.velocity.x == 0.0
        && state.velocity.z == 0.0
        && !state.sprinting
        && state.no_jump_delay == 0
}

fn input_keys(input: &law::Input) -> u32 {
    [
        input.forward,
        input.back,
        input.left,
        input.right,
        input.jump,
        input.sneak,
        input.sprint,
    ]
    .iter()
    .enumerate()
    .fold(0u32, |mask, (bit, &b)| mask | (u32::from(b) << bit))
}

fn step_all_bots(
    server_tick: u64,
    bots: &mut BTreeMap<u32, BotSlot>,
    world: &WorldMirror,
    store: &GeneratorStore,
    codec: &mut Interner,
    programs: &[words::Program],
    shm: &Shm,
    w: &mut Writer,
) {
    let replans: Vec<(u32, BlockPos)> = bots
        .iter_mut()
        .filter_map(|(&id, b)| b.pending_replan.take().map(|g| (id, g)))
        .collect();
    for (id, goal_air) in replans {
        if let Some(bot) = bots.get_mut(&id) {
            solve_bot(id, bot, goal_air, world, store, codec, shm, w);
        }
    }

    let mut active_bots = 0u32;
    for (&bot_id, bot) in bots.iter_mut() {
        let phase = std::mem::replace(&mut bot.phase, BotPhase::Idle);
        match phase {
            BotPhase::Idle => {}
            BotPhase::Executing {
                steps,
                mut idx,
                mut elapsed,
                goal_air,
                corridor,
            } => {
                active_bots += 1;
                let (prog, args, ticks) = steps[idx];
                let selected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    programs[prog as usize].select(world, &bot.state, elapsed, args)
                }));
                let input = match selected {
                    Ok(Some((_, inp))) => inp,
                    Ok(None) => {
                        send_plan_status(
                            shm,
                            w,
                            bot_id,
                            PLAN_STATUS_REFUSED,
                            0,
                            0,
                            "word program has no row",
                        );
                        continue;
                    }
                    Err(_) => {
                        send_plan_status(
                            shm,
                            w,
                            bot_id,
                            PLAN_STATUS_REFUSED,
                            0,
                            0,
                            "word program select panicked",
                        );
                        continue;
                    }
                };

                let prev_sprinting = bot.state.sprinting;
                let tick_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    law::try_tick(&mut bot.state, world, &input)
                }));
                match tick_res {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        send_plan_status(shm, w, bot_id, PLAN_STATUS_REFUSED, 0, 0, &e.0);
                        continue;
                    }
                    Err(_) => {
                        send_plan_status(
                            shm,
                            w,
                            bot_id,
                            PLAN_STATUS_REFUSED,
                            0,
                            0,
                            "law tick panicked",
                        );
                        continue;
                    }
                }

                elapsed += 1;
                if elapsed >= ticks {
                    idx += 1;
                    elapsed = 0;
                }
                let arrived = idx >= steps.len();
                let mut settled = false;
                if arrived {
                    if is_settled(&bot.state) {
                        settled = true;
                        bot.phase = BotPhase::Idle;
                    } else {
                        bot.phase = BotPhase::Settling;
                    }
                } else {
                    bot.phase = BotPhase::Executing {
                        steps,
                        idx,
                        elapsed,
                        goal_air,
                        corridor,
                    };
                }

                let mut flags = 0u32;
                if bot.state.on_ground {
                    flags |= FLAG_ON_GROUND;
                }
                if bot.state.horizontal_collision {
                    flags |= FLAG_HORIZ_COL;
                }
                if !prev_sprinting && bot.state.sprinting {
                    flags |= FLAG_START_SPRINT;
                } else if prev_sprinting && !bot.state.sprinting {
                    flags |= FLAG_STOP_SPRINT;
                }
                if arrived {
                    flags |= FLAG_ARRIVED;
                }
                if settled {
                    flags |= FLAG_SETTLED;
                }

                send_tick_cmd(shm, w, bot_id, input_keys(&input), flags, &bot.state);
            }
            BotPhase::Settling => {
                active_bots += 1;
                let prev_sprinting = bot.state.sprinting;
                let coast = law::Input {
                    yaw: bot.state.yaw,
                    pitch: bot.state.pitch,
                    ..Default::default()
                };
                let tick_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    law::try_tick(&mut bot.state, world, &coast)
                }));
                match tick_res {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        send_plan_status(shm, w, bot_id, PLAN_STATUS_REFUSED, 0, 0, &e.0);
                        continue;
                    }
                    Err(_) => {
                        send_plan_status(
                            shm,
                            w,
                            bot_id,
                            PLAN_STATUS_REFUSED,
                            0,
                            0,
                            "law settle panicked",
                        );
                        continue;
                    }
                }

                let settled = is_settled(&bot.state);
                if !settled {
                    bot.phase = BotPhase::Settling;
                }
                let mut flags = 0u32;
                if bot.state.on_ground {
                    flags |= FLAG_ON_GROUND;
                }
                if bot.state.horizontal_collision {
                    flags |= FLAG_HORIZ_COL;
                }
                if !prev_sprinting && bot.state.sprinting {
                    flags |= FLAG_START_SPRINT;
                } else if prev_sprinting && !bot.state.sprinting {
                    flags |= FLAG_STOP_SPRINT;
                }
                if settled {
                    flags |= FLAG_SETTLED;
                }
                send_tick_cmd(shm, w, bot_id, 0, flags, &bot.state);
            }
        }
    }

    w.clear();
    w.u64(server_tick);
    w.u32(active_bots);
    w.u32(world.section_count() as u32);
    let _ = shm.push_e2s(MSG_TICK_END, &w.buf);
    shm.set_engine_tick_seq(server_tick);
}

fn handle_message(
    msg_type: u16,
    payload: &[u8],
    world: &mut WorldMirror,
    store: &mut GeneratorStore,
    codec: &mut Interner,
    bots: &mut BTreeMap<u32, BotSlot>,
    programs: &[words::Program],
    shm: &Shm,
    w: &mut Writer,
) -> Result<(), String> {
    let mut r = Reader::new(payload);
    match msg_type {
        MSG_WORLD_RESET => {
            let session = r.u64()?;
            let difficulty = r.u8()?;
            let natural_regen = r.u8()? != 0;
            world.reset(session, difficulty, natural_regen);
            *store = GeneratorStore::default();
            *codec = Interner::default();
        }
        MSG_WORLD_RULES => {
            let difficulty = r.u8()?;
            let natural_regen = r.u8()? != 0;
            world.set_rules(difficulty, natural_regen);
        }
        MSG_SECTION_AIR => {
            let cx = r.i32()?;
            let cy = r.i32()?;
            let cz = r.i32()?;
            world.set_section_air(cx, cy, cz);
        }
        MSG_SECTION_RLE => {
            let cx = r.i32()?;
            let cy = r.i32()?;
            let cz = r.i32()?;
            let palette_count = r.u16()? as usize;
            let run_count = r.u16()? as usize;
            let mut names = Vec::with_capacity(palette_count);
            for _ in 0..palette_count {
                names.push(r.str()?);
            }
            let mut runs = Vec::with_capacity(run_count);
            for _ in 0..run_count {
                let len = r.u16()?;
                let idx = r.u16()?;
                runs.push((len, idx));
            }
            world.set_section_rle(cx, cy, cz, &names, &runs)?;
        }
        MSG_BLOCK_DELTA => {
            let x = r.i32()?;
            let y = r.i32()?;
            let z = r.i32()?;
            let state = r.str()?;
            world.set_block_delta([x, y, z], state);
            for bot in bots.values_mut() {
                if let BotPhase::Executing {
                    goal_air,
                    corridor: (lo, hi),
                    ..
                } = &bot.phase
                    && x >= lo[0]
                    && x <= hi[0]
                    && y >= lo[1]
                    && y <= hi[1]
                    && z >= lo[2]
                    && z <= hi[2]
                {
                    bot.pending_replan = Some(*goal_air);
                }
            }
        }
        MSG_CHUNK_UNLOAD => {
            let cx = r.i32()?;
            let cz = r.i32()?;
            world.unload_chunk(cx, cz);
        }
        MSG_BOT_SPAWN => {
            let sync = BotSyncPayload::read(&mut r)?;
            bots.insert(sync.bot_id, BotSlot::from_sync(&sync));
        }
        MSG_BOT_REMOVE => {
            let bot_id = r.u32()?;
            bots.remove(&bot_id);
        }
        MSG_BOT_SELF_SYNC => {
            let sync = BotSyncPayload::read(&mut r)?;
            let reason = r.u32()?;
            if let Some(bot) = bots.get_mut(&sync.bot_id) {
                bot.apply_sync(&sync, reason);
            } else {
                bots.insert(sync.bot_id, BotSlot::from_sync(&sync));
            }
        }
        MSG_BOT_GOAL => {
            let bot_id = r.u32()?;
            let gx = r.i32()?;
            let gy = r.i32()?;
            let gz = r.i32()?;
            let _issue_qpc = r.u64()?;
            if let Some(bot) = bots.get_mut(&bot_id) {
                bot.pending_replan = None;
                solve_bot(bot_id, bot, [gx, gy, gz], world, store, codec, shm, w);
            } else {
                send_plan_status(shm, w, bot_id, PLAN_STATUS_REFUSED, 0, 0, "unknown bot");
            }
        }
        MSG_BOT_STOP => {
            let bot_id = r.u32()?;
            if let Some(bot) = bots.get_mut(&bot_id) {
                bot.phase = BotPhase::Idle;
                bot.pending_replan = None;
            }
        }
        MSG_TICK_BEGIN => {
            let server_tick = r.u64()?;
            step_all_bots(server_tick, bots, world, store, codec, programs, shm, w);
        }
        other => return Err(format!("unknown s2e message {other}")),
    }
    if r.remaining() != 0 {
        return Err(format!("trailing bytes in msg {msg_type}: {}", r.remaining()));
    }
    Ok(())
}

fn main() {
    let shm_path = parse_shm_arg();
    let shm = Shm::open(&shm_path).expect("open shared memory");
    let (mut last_server_epoch, engine_epoch) = shm.init_engine();
    shm.set_engine_heartbeat_ms(now_ms());

    let mut world = WorldMirror::new();
    let mut store = GeneratorStore::default();
    let mut codec = Interner::default();
    let programs = words::tiny_programs();
    let mut bots: BTreeMap<u32, BotSlot> = BTreeMap::new();

    eprintln!(
        "mind ready on {} (server_epoch={}, engine_epoch={})",
        shm_path.display(),
        last_server_epoch,
        engine_epoch
    );

    let mut scratch = Vec::with_capacity(65536);
    let mut writer = Writer::new();
    let mut last_activity = Instant::now();

    loop {
        shm.set_engine_heartbeat_ms(now_ms());
        let se = shm.server_epoch();
        if se != last_server_epoch {
            last_server_epoch = se;
            bots.clear();
            world.reset(0, 0, true);
            store = GeneratorStore::default();
            codec = Interner::default();
        }

        let mut processed = false;
        while let Some(msg_type) = shm.pop_s2e(&mut scratch) {
            processed = true;
            if let Err(why) = handle_message(
                msg_type,
                &scratch,
                &mut world,
                &mut store,
                &mut codec,
                &mut bots,
                &programs,
                &shm,
                &mut writer,
            ) {
                eprintln!("mind protocol error on msg {msg_type}: {why}");
            }
        }

        if processed {
            last_activity = Instant::now();
        } else if last_activity.elapsed() < Duration::from_millis(2) {
            std::thread::yield_now();
        } else {
            std::thread::sleep(Duration::from_micros(50));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fence_delta_triggers_needs_joints() {
        let mut world = WorldMirror::new();
        world.reset(1, 0, true);
        let palette = ["minecraft:air", "minecraft:grass_block[snowy=false]"];
        let runs = [(768u16, 0u16), (256, 1), (3072, 0)];
        for cx in -1..=3 {
            for cz in -1..=1 {
                world.set_section_rle(cx, -4, cz, &palette, &runs).unwrap();
            }
        }
        world.set_block_delta(
            [20, -60, 0],
            "minecraft:oak_fence[east=false,north=false,south=false,waterlogged=false,west=false]",
        );

        let state = PlayerState {
            position: Vec3::new(0.5, -60.0, 0.5),
            on_ground: true,
            main_supporting_block: Some([0, -61, 0]),
            food_level: 20,
            ..PlayerState::default()
        };
        let a = [0, -61, 0];
        let goal_solid = [30, -61, 0];
        let (cl, ch) = corridor_bounds(
            [0, -60, 0],
            a,
            goal_solid,
            world.min_loaded_y(),
            world.max_loaded_y(),
        );
        world.check_corridor_valid(cl, ch).unwrap();
        let corridor = GeneratorSuperflatSource::from_world(&world, (cl, ch), a).unwrap();
        let store = GeneratorStore::default();
        let mut codec = Interner::default();
        let res = plan::solve(
            &state,
            Goal { cell: goal_solid },
            Kit::BareLegs,
            &corridor,
            &store,
            &mut codec,
            u64::MAX,
        );
        match res.outcome {
            PlanOutcome::NeedsJoints { exception, .. } => assert_eq!(exception, [20, -60, 0]),
            other => panic!("expected NeedsJoints at [20, -60, 0], got {other:?} ({:?})", res.explanation),
        }
    }
}
