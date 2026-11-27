//! Shared living server-side pathfinding mind driving up to 10^4 bots over shared memory.
mod disk_world;
mod pool;
mod protocol;
mod reserve;
mod shm;
mod world_mirror;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use law::{PlayerState, Vec3};
use pool::{
    Observer, PlanStatusOut, RenderBotOut, SolveSummary, SwarmConfig, SwarmPosture, TierChangeOut,
    TickCmdOut, VisualOut, WorkerPool,
};
use protocol::{
    BotSyncPayload, MODE_MATERIALIZED, MSG_BLOCK_DELTA, MSG_BOT_GOAL, MSG_BOT_PLAN_STATUS,
    MSG_BOT_REMOVE, MSG_BOT_SELF_SYNC, MSG_BOT_SPAWN, MSG_BOT_STOP, MSG_BOT_TICK_CMD,
    MSG_BOT_TIER_CHANGE, MSG_CHUNK_TICKET, MSG_CHUNK_UNLOAD, MSG_OBSERVERS, MSG_RENDER_FRAME,
    MSG_RENDER_STREAM, MSG_SECTION_AIR, MSG_SECTION_RLE, MSG_SWARM_CLEAR, MSG_SWARM_CONFIG,
    MSG_SWARM_GOAL_BATCH, MSG_SWARM_PLAN_SUMMARY, MSG_SWARM_POSTURE, MSG_SWARM_SPAWN_BATCH,
    MSG_SWARM_STOP_ALL, MSG_TICK_BEGIN, MSG_TICK_END, MSG_VISUAL_BATCH, MSG_WORLD_RESET,
    MSG_WORLD_RULES, Reader, Writer,
};
use shm::Shm;
use world_mirror::{FxHashMap, WorldMirror};

#[derive(Default)]
struct RenderStreamState {
    enabled: bool,
    force_full: bool,
    dirty: bool,
}

#[derive(Default)]
struct TickTotals {
    promotions: u32,
    demotions: u32,
    hooked_promotions: u32,
}

impl TickTotals {
    fn reset(&mut self) {
        *self = Self::default();
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

fn send_plan_status(shm: &Shm, w: &mut Writer, p: &PlanStatusOut) {
    w.clear();
    w.u32(p.bot_id);
    w.u32(p.status);
    w.u32(p.predicted_ticks);
    w.u64(p.solve_ns);
    w.str(&p.msg);
    let _ = shm.push_e2s(MSG_BOT_PLAN_STATUS, &w.buf);
}

fn send_tier_change(shm: &Shm, w: &mut Writer, t: &TierChangeOut) {
    w.clear();
    w.u32(t.bot_id);
    w.u32(t.bot_index);
    w.u8(t.new_tier);
    w.u32(t.flags);
    w.f64(t.state.position.x);
    w.f64(t.state.position.y);
    w.f64(t.state.position.z);
    w.f64(t.state.velocity.x);
    w.f64(t.state.velocity.y);
    w.f64(t.state.velocity.z);
    w.f32(t.state.yaw);
    w.f32(t.state.pitch);
    let _ = shm.push_e2s(MSG_BOT_TIER_CHANGE, &w.buf);
}

fn send_tick_cmd(shm: &Shm, w: &mut Writer, c: &TickCmdOut) {
    w.clear();
    w.u32(c.bot_id);
    w.u32(c.keys);
    w.u32(c.flags);
    w.f64(c.x);
    w.f64(c.y);
    w.f64(c.z);
    w.f32(c.yaw);
    w.f32(c.pitch);
    let _ = shm.push_e2s(MSG_BOT_TICK_CMD, &w.buf);
}

fn send_chunk_ticket(shm: &Shm, w: &mut Writer, cx: i32, cz: i32, level: u8) {
    w.clear();
    w.i32(cx);
    w.i32(cz);
    w.u8(level);
    let _ = shm.push_e2s(MSG_CHUNK_TICKET, &w.buf);
}

fn send_visual_batch(shm: &Shm, w: &mut Writer, visuals: &[VisualOut]) {
    const MAX_PER_FRAME: usize = 2048;
    for chunk in visuals.chunks(MAX_PER_FRAME) {
        w.clear();
        w.u32(chunk.len() as u32);
        for v in chunk {
            w.u32(v.bot_id);
            w.f64(v.x);
            w.f64(v.y);
            w.f64(v.z);
            w.f32(v.vx);
            w.f32(v.vy);
            w.f32(v.vz);
            w.f32(v.yaw);
            w.f32(v.pitch);
            w.u8(v.flags);
        }
        let _ = shm.push_e2s(MSG_VISUAL_BATCH, &w.buf);
    }
}

fn send_plan_summary(shm: &Shm, w: &mut Writer, sum: &mut SolveSummary, total: u32, batch_wall_ns: u64) {
    let (min_ns, med_ns, max_ns) = if sum.solve_ns_samples.is_empty() {
        (0, 0, 0)
    } else {
        sum.solve_ns_samples.sort_unstable();
        let n = sum.solve_ns_samples.len();
        (sum.solve_ns_samples[0], sum.solve_ns_samples[n / 2], sum.solve_ns_samples[n - 1])
    };
    w.clear();
    w.u32(total);
    w.u32(sum.ready);
    w.u32(sum.already);
    w.u32(sum.joints);
    w.u32(sum.refused);
    w.u64(min_ns);
    w.u64(med_ns);
    w.u64(max_ns);
    w.u64(batch_wall_ns);
    w.u32(sum.min_pred_t);
    w.u32(sum.max_pred_t);
    w.str(&sum.sample_msg);
    let _ = shm.push_e2s(MSG_SWARM_PLAN_SUMMARY, &w.buf);
}

const RENDER_FRAME_DELTA_FLAG: u32 = 0x8000_0000;

fn encode_render_frame(w: &mut Writer, tick: u32, bots: &[RenderBotOut], is_delta: bool) {
    w.clear();
    if bots.is_empty() {
        w.u32(tick);
        w.u32(if is_delta { RENDER_FRAME_DELTA_FLAG } else { 0 });
        w.f64(0.0);
        w.f64(0.0);
        w.f64(0.0);
        return;
    }
    let ox = bots[0].x.floor();
    let oy = bots[0].y.floor();
    let oz = bots[0].z.floor();
    let count_word = (bots.len() as u32) | if is_delta { RENDER_FRAME_DELTA_FLAG } else { 0 };
    w.u32(tick);
    w.u32(count_word);
    w.f64(ox);
    w.f64(oy);
    w.f64(oz);
    w.buf.reserve(bots.len() * 20);
    for b in bots {
        w.f32((b.x - ox) as f32);
        w.f32((b.y - oy) as f32);
        w.f32((b.z - oz) as f32);
        let yaw_q = ((b.yaw.rem_euclid(360.0) / 360.0) * 65536.0)
            .round()
            .clamp(0.0, 65535.0) as u16;
        let pitch_q = ((b.pitch.clamp(-90.0, 90.0) / 90.0) * 32767.0).round() as i16;
        w.u16(yaw_q);
        w.i16(pitch_q);
        w.u16(b.bot_index);
        w.u8(b.speed_q);
        w.u8(b.flags);
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_message(
    msg_type: u16,
    payload: &[u8],
    world: &Arc<RwLock<WorldMirror>>,
    pool: &mut WorkerPool,
    config: &mut SwarmConfig,
    posture: &mut SwarmPosture,
    observers: &mut Vec<Observer>,
    ticket_last_sent: &mut FxHashMap<([i32; 2], u8), u64>,
    totals: &mut TickTotals,
    render_stream: &mut RenderStreamState,
    shm: &Shm,
    w: &mut Writer,
) -> Result<(), String> {
    let mut r = Reader::new(payload);
    match msg_type {
        MSG_WORLD_RESET => {
            let session = r.u64()?;
            let difficulty = r.u8()?;
            let natural_regen = r.u8()? != 0;
            let world_dir = r.str()?;
            {
                let mut guard = world.write().unwrap();
                guard.reset(session, difficulty, natural_regen);
                if !world_dir.is_empty() {
                    guard.set_world_dir(Path::new(world_dir));
                }
            }
            pool.reset_stores();
            ticket_last_sent.clear();
            totals.reset();
            *posture = SwarmPosture::default();
            render_stream.dirty = true;
        }
        MSG_WORLD_RULES => {
            let difficulty = r.u8()?;
            let natural_regen = r.u8()? != 0;
            world.write().unwrap().set_rules(difficulty, natural_regen);
        }
        MSG_SECTION_AIR => {
            let cx = r.i32()?;
            let cy = r.i32()?;
            let cz = r.i32()?;
            world.write().unwrap().set_section_air(cx, cy, cz);
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
            world
                .write()
                .unwrap()
                .set_section_rle(cx, cy, cz, &names, &runs)?;
        }
        MSG_BLOCK_DELTA => {
            let x = r.i32()?;
            let y = r.i32()?;
            let z = r.i32()?;
            let state = r.str()?;
            let changed = world.write().unwrap().set_block_delta([x, y, z], state);
            if changed {
                pool.replan_in_corridor([x, y, z]);
            }
        }
        MSG_CHUNK_UNLOAD => {
            let cx = r.i32()?;
            let cz = r.i32()?;
            world.write().unwrap().unload_chunk(cx, cz);
        }
        MSG_BOT_SPAWN => {
            let sync = BotSyncPayload::read(&mut r)?;
            pool.spawn_one(sync.bot_id, MODE_MATERIALIZED, state_from_sync(&sync));
            render_stream.dirty = true;
        }
        MSG_BOT_REMOVE => {
            let bot_id = r.u32()?;
            pool.remove_one(bot_id);
            render_stream.dirty = true;
        }
        MSG_BOT_SELF_SYNC => {
            let sync = BotSyncPayload::read(&mut r)?;
            let reason = r.u32()?;
            pool.self_sync(sync.bot_id, state_from_sync(&sync), reason);
        }
        MSG_BOT_GOAL => {
            let bot_id = r.u32()?;
            let gx = r.i32()?;
            let gy = r.i32()?;
            let gz = r.i32()?;
            let _issue_qpc = r.u64()?;
            for p in pool.goal_one(bot_id, [gx, gy, gz]) {
                send_plan_status(shm, w, &p);
            }
        }
        MSG_BOT_STOP => {
            let bot_id = r.u32()?;
            pool.stop_one(bot_id);
        }
        MSG_SWARM_SPAWN_BATCH => {
            let count = r.u32()?;
            let start_bot_id = r.u32()?;
            let start_index = r.u32()?;
            let mode = r.u8()?;
            let cols = r.u32()?;
            let base_x = r.f64()?;
            let base_y = r.f64()?;
            let base_z = r.f64()?;
            let spacing_x = r.f64()?;
            let spacing_z = r.f64()?;
            pool.spawn_batch(
                count,
                start_bot_id,
                start_index,
                mode,
                cols,
                base_x,
                base_y,
                base_z,
                spacing_x,
                spacing_z,
            );
            render_stream.dirty = true;
        }
        MSG_SWARM_CLEAR => {
            pool.clear();
            ticket_last_sent.clear();
            totals.reset();
            *posture = SwarmPosture::default();
            render_stream.dirty = true;
        }
        MSG_SWARM_GOAL_BATCH => {
            let gx = r.i32()?;
            let gy = r.i32()?;
            let gz = r.i32()?;
            let formation_mode = r.u8()?;
            let param = r.i32()?;
            let wave_ticks = r.u16()?;
            let _issue_qpc = r.u64()?;
            *posture = SwarmPosture::default();
            let t0 = Instant::now();
            let mut sum = pool.goal_batch_ext(gx, gy, gz, formation_mode, param, wave_ticks);
            let wall_ns = t0.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            for p in &sum.statuses {
                send_plan_status(shm, w, p);
            }
            send_plan_summary(shm, w, &mut sum, pool.total_bots() as u32, wall_ns);
        }
        MSG_SWARM_STOP_ALL => {
            *posture = SwarmPosture::default();
            pool.stop_all();
        }
        MSG_SWARM_POSTURE => {
            let flags = r.u8()?;
            let tx = r.f64()?;
            let ty = r.f64()?;
            let tz = r.f64()?;
            *posture = SwarmPosture { flags, tx, ty, tz };
        }
        MSG_OBSERVERS => {
            let count = r.u32()? as usize;
            observers.clear();
            observers.reserve(count);
            for _ in 0..count {
                let _id = r.u32()?;
                let x = r.f64()?;
                let y = r.f64()?;
                let z = r.f64()?;
                let view_dist = r.f32()?;
                observers.push(Observer {
                    x,
                    y,
                    z,
                    view_dist,
                });
            }
        }
        MSG_SWARM_CONFIG => {
            config.max_visual = r.u32()?;
            config.max_materialized = r.u32()?;
            config.materialize_radius = r.f32()?;
            config.canary_count = r.u32()?;
            config.canary_period = r.u32()?;
        }
        MSG_RENDER_STREAM => {
            let enabled = r.u8()? != 0;
            let force_full = r.u8()? != 0;
            render_stream.enabled = enabled;
            if force_full {
                render_stream.force_full = true;
            }
        }
        MSG_TICK_BEGIN => {
            let server_tick = r.u64()?;
            let tick_t0 = Instant::now();
            let emit_full_render = render_stream.enabled && (render_stream.force_full || render_stream.dirty);
            let (outputs, tickets) = pool.step_tick(
                server_tick,
                *config,
                *posture,
                observers,
                ticket_last_sent,
                render_stream.enabled,
                emit_full_render,
            );

            for (coord, level) in tickets {
                send_chunk_ticket(shm, w, coord[0], coord[1], level);
            }
            for p in &outputs.plan_statuses {
                send_plan_status(shm, w, p);
            }
            for tc in &outputs.tier_changes {
                send_tier_change(shm, w, tc);
            }
            for cmd in &outputs.tick_cmds {
                send_tick_cmd(shm, w, cmd);
            }
            if !outputs.visuals.is_empty() {
                send_visual_batch(shm, w, &outputs.visuals);
            }
            if render_stream.enabled {
                let total_bots = pool.total_bots();
                if emit_full_render
                    || (!outputs.render_bots.is_empty() && outputs.render_bots.len() == total_bots)
                {
                    encode_render_frame(w, server_tick as u32, &outputs.render_bots, false);
                    let _ = shm.push_e2s(MSG_RENDER_FRAME, &w.buf);
                    render_stream.force_full = false;
                    render_stream.dirty = false;
                } else if !outputs.render_bots.is_empty() {
                    encode_render_frame(w, server_tick as u32, &outputs.render_bots, true);
                    let _ = shm.push_e2s(MSG_RENDER_FRAME, &w.buf);
                }
            }

            totals.promotions += outputs.stats.promotions;
            totals.demotions += outputs.stats.demotions;
            totals.hooked_promotions += outputs.stats.hooked_promotions;

            let (total_sections, disk_sections) = {
                let wg = world.read().unwrap();
                (wg.section_count() as u32, wg.disk_section_count() as u32)
            };
            let mind_tick_ns = tick_t0.elapsed().as_nanos().min(u64::MAX as u128) as u64;

            w.clear();
            w.u64(server_tick);
            w.u32(outputs.stats.active_bots);
            w.u32(total_sections);
            w.u32(disk_sections);
            w.u32(pool.total_bots() as u32);
            w.u32(outputs.stats.virt_count);
            w.u32(outputs.stats.vis_count);
            w.u32(outputs.stats.mat_count);
            w.u32(outputs.stats.arrived_count);
            w.u32(outputs.stats.settled_count);
            w.u32(outputs.stats.exact_t_count);
            w.u32(outputs.stats.min_act_t);
            w.u32(outputs.stats.max_act_t);
            w.u32(totals.promotions);
            w.u32(totals.demotions);
            w.u32(totals.hooked_promotions);
            w.u64(mind_tick_ns);
            let _ = shm.push_e2s(MSG_TICK_END, &w.buf);
            shm.set_engine_tick_seq(server_tick);
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

    let world = Arc::new(RwLock::new(WorldMirror::new()));
    let programs = Arc::new(words::tiny_programs());
    let mut pool = WorkerPool::new(world.clone(), programs);
    let mut config = SwarmConfig::default();
    let mut posture = SwarmPosture::default();
    let mut observers = Vec::new();
    let mut ticket_last_sent = FxHashMap::default();
    let mut totals = TickTotals::default();
    let mut render_stream = RenderStreamState::default();

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
            pool.clear();
            pool.reset_stores();
            world.write().unwrap().reset(0, 0, true);
            observers.clear();
            ticket_last_sent.clear();
            totals.reset();
            posture = SwarmPosture::default();
            render_stream = RenderStreamState::default();
        }

        let mut processed = false;
        while let Some(msg_type) = shm.pop_s2e(&mut scratch) {
            processed = true;
            if let Err(why) = handle_message(
                msg_type,
                &scratch,
                &world,
                &mut pool,
                &mut config,
                &mut posture,
                &mut observers,
                &mut ticket_last_sent,
                &mut totals,
                &mut render_stream,
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
    use law::{World, codec::Interner};
    use plan::{Goal, Kit, PlanOutcome, store::GeneratorStore};
    use protocol::{FORM_CITADEL, FORM_CORRIDOR, FORM_QUAD, FORM_WEDGE};

    fn build_flat_mirror(cx_range: std::ops::RangeInclusive<i32>, cz_range: std::ops::RangeInclusive<i32>) -> WorldMirror {
        let mut world = WorldMirror::new();
        world.reset(1, 0, true);
        let palette = ["minecraft:air", "minecraft:grass_block[snowy=false]"];
        let runs = [(768u16, 0u16), (256, 1), (3072, 0)];
        for cx in cx_range {
            for cz in cz_range.clone() {
                world.set_section_rle(cx, -4, cz, &palette, &runs).unwrap();
                world.set_section_air(cx, -3, cz);
            }
        }
        world
    }

    #[test]
    fn fence_delta_is_crossed_around() {
        let mut world = build_flat_mirror(-1..=3, -1..=1);
        let open = {
            let state = PlayerState { position: Vec3::new(0.5, -60.0, 0.5), on_ground: true, main_supporting_block: Some([0, -61, 0]), food_level: 20, ..PlayerState::default() };
            match plan::solve(&state, Goal::of([30, -61, 0]), Kit::BareLegs, &world, &GeneratorStore::default(), &mut Interner::default(), u64::MAX).outcome {
                PlanOutcome::Ready(p) => p.ticks,
                other => panic!("open ground: {other:?}"),
            }
        };
        assert!(world.set_block_delta(
            [20, -60, 0],
            "minecraft:oak_fence[east=false,north=false,south=false,waterlogged=false,west=false]",
        ));
        assert!(!world.block([20, -60, 0]).never_collides(), "the mirror holds the fence's collision");

        let state = PlayerState {
            position: Vec3::new(0.5, -60.0, 0.5),
            on_ground: true,
            main_supporting_block: Some([0, -61, 0]),
            food_level: 20,
            ..PlayerState::default()
        };
        let (cl, ch) = pool::corridor_bounds(
            [0, -60, 0],
            [0, -61, 0],
            [30, -61, 0],
            world.min_loaded_y(),
            world.max_loaded_y(),
        );
        world.check_corridor_valid(cl, ch).unwrap();
        let store = GeneratorStore::default();
        let mut codec = Interner::default();
        let res = plan::solve(
            &state,
            Goal::of([30, -61, 0]),
            Kit::BareLegs,
            &world,
            &store,
            &mut codec,
            u64::MAX,
        );
        // The planner learns the joint at the fence (Phase B) and certifies a detour.
        match res.outcome {
            PlanOutcome::Ready(p) => assert!(p.ticks > open, "fence T={} must exceed open-ground T={open}", p.ticks),
            other => panic!("expected a certified detour, got {other:?} ({:?})", res.explanation),
        }
    }

    #[test]
    fn world_mirror_hooked_pressure_plate_allows_solve_and_flags_hooked() {
        let mut world = build_flat_mirror(-1..=3, -1..=1);
        assert!(world.set_block_delta(
            [15, -60, 0],
            "minecraft:oak_pressure_plate[powered=false]",
        ));
        let state = PlayerState {
            position: Vec3::new(0.5, -60.0, 0.5),
            on_ground: true,
            main_supporting_block: Some([0, -61, 0]),
            food_level: 20,
            ..PlayerState::default()
        };
        let (cl, ch) = pool::corridor_bounds(
            [0, -60, 0],
            [0, -61, 0],
            [30, -61, 0],
            world.min_loaded_y(),
            world.max_loaded_y(),
        );
        let store = GeneratorStore::default();
        let mut codec = Interner::default();
        let res = plan::solve(
            &state,
            Goal::of([30, -61, 0]),
            Kit::BareLegs,
            &world,
            &store,
            &mut codec,
            u64::MAX,
        );
        assert!(matches!(res.outcome, PlanOutcome::Ready(_)), "{:?}", res.outcome);
        assert_eq!(world.corridor_has_hooked(cl, ch), Some([15, -60, 0]));
        assert!(!world.set_block_delta(
            [15, -60, 0],
            "minecraft:oak_pressure_plate[powered=true]",
        ));
    }

    #[test]
    fn disk_world_loads_unloaded_far_chunk() {
        // A real server world, read only when named: never his world by default.
        let Some(lab_world) = std::env::var_os("PATHTRACING_MIND_TEST_WORLD") else { return };
        let lab_world = Path::new(&lab_world);
        if !lab_world.is_dir() {
            return;
        }
        let mut world = WorldMirror::new();
        world.reset(1, 0, true);
        world.set_world_dir(lab_world);
        let loaded = world.ensure_chunks_from_disk(&[[350, 255]]);
        assert_eq!(loaded, 1);
        assert!(!world.is_server_loaded(350, 255));
        assert!(world.loaded([5600, -61, 4080]));
        assert!(!world.is_air([5600, -61, 4080]));
        assert!(world.is_air([5600, -60, 4080]));
    }

    #[test]
    fn worker_pool_steps_10000_virtual_bots_fast() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-14..=-1, -14..=-5)));
        let programs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::new(world, programs);

        pool.spawn_batch(
            10_000,
            1,
            0,
            protocol::MODE_VIRTUAL,
            100,
            -200.5,
            -60.0,
            -200.5,
            1.0,
            1.0,
        );
        assert_eq!(pool.total_bots(), 10_000);

        let solve_t0 = Instant::now();
        let sum = pool.goal_batch(-120, -60, -200, true);
        let solve_ms = solve_t0.elapsed().as_secs_f64() * 1e3;
        assert_eq!(sum.ready, 10_000, "ready={}, refused={}, sample={}", sum.ready, sum.refused, sum.sample_msg);
        assert_eq!(sum.refused, 0);

        let mut last_sent = FxHashMap::default();
        let mut tick_ms = Vec::new();
        for tick in 1..=5u64 {
            let t0 = Instant::now();
            let (out, _) = pool.step_tick(tick, SwarmConfig::default(), SwarmPosture::default(), &[], &mut last_sent, false, false);
            tick_ms.push(t0.elapsed().as_secs_f64() * 1e3);
            assert_eq!(out.stats.active_bots, 10_000);
            assert_eq!(out.stats.virt_count, 10_000);
        }
        eprintln!(
            "10k virtual test: solve_ms={:.2} tick_ms={:?}",
            solve_ms, tick_ms
        );
    }

    #[test]
    fn render_frame_packs_10000_bots_compactly() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-14..=3, -14..=-1)));
        let programs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::new(world, programs);
        pool.spawn_batch(
            10_000,
            1,
            0,
            protocol::MODE_VIRTUAL,
            100,
            -200.5,
            -60.0,
            -200.5,
            1.5,
            1.5,
        );
        let sum = pool.goal_batch(-120, -60, -200, true);
        assert_eq!(sum.ready, 10_000);

        let mut last_sent = FxHashMap::default();
        let (out, _) = pool.step_tick(1, SwarmConfig::default(), SwarmPosture::default(), &[], &mut last_sent, true, true);
        assert_eq!(out.render_bots.len(), 10_000);

        let mut w = Writer::new();
        encode_render_frame(&mut w, 42, &out.render_bots, false);
        assert_eq!(w.buf.len(), 32 + 10_000 * 20);
        assert_eq!(u32::from_le_bytes(w.buf[0..4].try_into().unwrap()), 42);
        assert_eq!(u32::from_le_bytes(w.buf[4..8].try_into().unwrap()), 10_000);
    }

    #[test]
    fn lazy_virtual_matches_eager_and_midflight_promotion() {
        let make_state = |z: f64| PlayerState {
            position: Vec3::new(0.5, -60.0, z),
            on_ground: true,
            main_supporting_block: Some([0, -61, z.floor() as i32]),
            health: 20.0,
            saturation: 5.0,
            food_level: 20,
            ..PlayerState::default()
        };

        // (a) Eager MODE_MATERIALIZED across all ticks.
        let world_eager = Arc::new(RwLock::new(build_flat_mirror(-1..=6, -1..=1)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool_eager = WorkerPool::with_workers(1, world_eager, progs.clone());
        pool_eager.spawn_one(1, protocol::MODE_MATERIALIZED, make_state(0.5));
        let st_a = pool_eager.goal_one(1, [80, -60, 0]);
        assert_eq!(st_a[0].status, protocol::PLAN_STATUS_READY);
        let pred_t = st_a[0].predicted_ticks;
        let mut sent_a = FxHashMap::default();
        let mut final_a = None;
        for tick in 1..=(pred_t as u64 + 10) {
            let (out, _) = pool_eager.step_tick(tick, SwarmConfig::default(), SwarmPosture::default(), &[], &mut sent_a, false, false);
            if let Some(cmd) = out.tick_cmds.first() {
                if (cmd.flags & protocol::FLAG_SETTLED) != 0 {
                    final_a = Some((cmd.x, cmd.y, cmd.z, out.stats.min_act_t));
                    break;
                }
            }
        }
        let final_a = final_a.expect("eager bot settles");

        // (b) Lazy MODE_VIRTUAL across all 228+ ticks + settle.
        let world_lazy = Arc::new(RwLock::new(build_flat_mirror(-1..=6, -1..=1)));
        let mut pool_lazy = WorkerPool::with_workers(1, world_lazy, progs.clone());
        pool_lazy.spawn_one(1, protocol::MODE_VIRTUAL, make_state(0.5));
        let st_b = pool_lazy.goal_one(1, [80, -60, 0]);
        assert_eq!(st_b[0].predicted_ticks, pred_t);
        let mut sent_b = FxHashMap::default();
        let mut final_b = None;
        for tick in 1..=(pred_t as u64 + 10) {
            let (out, _) = pool_lazy.step_tick(tick, SwarmConfig::default(), SwarmPosture::default(), &[], &mut sent_b, true, false);
            if out.stats.settled_count == 1 {
                let rb = out.render_bots.first().expect("active virtual bot emitted");
                final_b = Some((rb.x, rb.y, rb.z, out.stats.min_act_t));
                break;
            }
        }
        let final_b = final_b.expect("lazy virtual bot settles");

        // (c) Lazy MODE_AUTO promoted mid-flight at tick 113 via an observer + stepped eagerly.
        let world_prom = Arc::new(RwLock::new(build_flat_mirror(-1..=6, -1..=1)));
        for cx in -1..=6 {
            for cz in -1..=1 {
                // Ensure server-loaded status doesn't depend on extra messages; mark loaded via section air isn't set,
                // wait: tier change emits VISUAL without server-loaded, or promote via MODE -> actually let's test Observer->Visual (still eager!).
                let _ = (cx, cz);
            }
        }
        let mut pool_prom = WorkerPool::with_workers(1, world_prom, progs);
        pool_prom.spawn_one(1, protocol::MODE_AUTO, make_state(0.5));
        let st_c = pool_prom.goal_one(1, [80, -60, 0]);
        assert_eq!(st_c[0].predicted_ticks, pred_t);
        let mut sent_c = FxHashMap::default();
        let cfg_prom = SwarmConfig {
            max_visual: 4,
            max_materialized: 0,
            materialize_radius: 0.0,
            canary_count: 0,
            canary_period: 100,
        };
        let mut final_c = None;
        let mut saw_midflight_promote = false;
        for tick in 1..=(pred_t as u64 + 10) {
            let obs = if tick >= 113 {
                vec![pool::Observer {
                    x: 40.0,
                    y: -60.0,
                    z: 0.5,
                    view_dist: 128.0,
                }]
            } else {
                Vec::new()
            };
            let (out, _) = pool_prom.step_tick(tick, cfg_prom, SwarmPosture::default(), &obs, &mut sent_c, true, false);
            if tick == 113 {
                assert!(
                    out.tier_changes.iter().any(|tc| tc.new_tier == protocol::TIER_VISUAL),
                    "expected mid-flight promotion to VISUAL at tick 113"
                );
                saw_midflight_promote = true;
            }
            if out.stats.settled_count == 1 {
                let rb = out.render_bots.first().expect("promoted bot emitted");
                final_c = Some((rb.x, rb.y, rb.z, out.stats.min_act_t));
                break;
            }
        }
        assert!(saw_midflight_promote);
        let final_c = final_c.expect("mid-flight promoted bot settles");

        assert_eq!(final_a.3, pred_t);
        assert_eq!(final_b.3, pred_t);
        assert_eq!(final_c.3, pred_t);
        assert!((final_a.0 - final_b.0).abs() < 1e-9, "x mismatch a={} b={}", final_a.0, final_b.0);
        assert!((final_a.1 - final_b.1).abs() < 1e-9, "y mismatch a={} b={}", final_a.1, final_b.1);
        assert!((final_a.2 - final_b.2).abs() < 1e-9, "z mismatch a={} b={}", final_a.2, final_b.2);
        assert!((final_a.0 - final_c.0).abs() < 1e-9, "x mismatch a={} c={}", final_a.0, final_c.0);
        assert!((final_a.1 - final_c.1).abs() < 1e-9, "y mismatch a={} c={}", final_a.1, final_c.1);
        assert!((final_a.2 - final_c.2).abs() < 1e-9, "z mismatch a={} c={}", final_a.2, final_c.2);
    }

    #[test]
    fn render_frame_delta_packs_only_changed_bot() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-1..=3, -1..=1)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(1, world, progs);
        pool.spawn_batch(4, 1, 0, protocol::MODE_AUTO, 2, 0.5, -60.0, 0.5, 50.0, 50.0);

        let cfg = SwarmConfig {
            max_visual: 4,
            max_materialized: 0,
            materialize_radius: 0.0,
            canary_count: 0,
            canary_period: 100,
        };
        let mut last_sent = FxHashMap::default();
        // Tick 1: initial full emit (simulate dirty).
        let (out1, _) = pool.step_tick(1, cfg, SwarmPosture::default(), &[], &mut last_sent, true, true);
        assert_eq!(out1.render_bots.len(), 4);
        let mut w = Writer::new();
        encode_render_frame(&mut w, 1, &out1.render_bots, false);
        assert_eq!(u32::from_le_bytes(w.buf[4..8].try_into().unwrap()), 4);

        // Tick 2: idle with no observer -> 0 bots emitted.
        let (out2, _) = pool.step_tick(2, cfg, SwarmPosture::default(), &[], &mut last_sent, true, false);
        assert!(out2.render_bots.is_empty());

        // Tick 3: observer sees only bot 1 -> tier change for 1 bot -> 1 entry delta (52 bytes, bit 31 set).
        let obs = [pool::Observer {
            x: 0.5,
            y: -60.0,
            z: 0.5,
            view_dist: 10.0,
        }];
        let (out3, _) = pool.step_tick(3, cfg, SwarmPosture::default(), &obs, &mut last_sent, true, false);
        assert_eq!(out3.render_bots.len(), 1);
        assert_eq!(out3.render_bots[0].bot_index, 0);
        encode_render_frame(&mut w, 3, &out3.render_bots, true);
        assert_eq!(w.buf.len(), 32 + 20);
        assert_eq!(
            u32::from_le_bytes(w.buf[4..8].try_into().unwrap()),
            1 | RENDER_FRAME_DELTA_FLAG
        );
    }

    #[test]
    fn airborne_goal_snaps_to_ground_and_arrives() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-1..=2, -1..=1)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(1, world, progs);
        let state = PlayerState {
            position: Vec3::new(0.5, -60.0, 0.5),
            on_ground: true,
            main_supporting_block: Some([0, -61, 0]),
            health: 20.0,
            saturation: 5.0,
            food_level: 20,
            ..PlayerState::default()
        };
        pool.spawn_one(1, protocol::MODE_VIRTUAL, state);

        let sum = pool.goal_batch(10, -47, 0, false);
        assert_eq!(sum.ready, 1, "ready={}, refused={}, sample={}", sum.ready, sum.refused, sum.sample_msg);
        assert_eq!(sum.refused, 0);

        let mut last_sent = FxHashMap::default();
        let cfg = SwarmConfig {
            canary_count: 0,
            ..SwarmConfig::default()
        };
        for tick in 1..=200u64 {
            let (out, _) = pool.step_tick(tick, cfg, SwarmPosture::default(), &[], &mut last_sent, true, false);
            if out.stats.settled_count == 1 {
                let rb = out.render_bots.first().expect("settled bot emitted");
                assert!((rb.y - (-60.0)).abs() < 1e-6, "expected y=-60, got {}", rb.y);
                assert_eq!(out.stats.arrived_count, 1);
                return;
            }
        }
        panic!("airborne-snapped bot did not settle within 200 ticks");
    }

    #[test]
    fn lazy_step_curve_matches_eager_every_tick() {
        let make_state = || PlayerState {
            position: Vec3::new(0.5, -60.0, 0.5),
            on_ground: true,
            main_supporting_block: Some([0, -61, 0]),
            health: 20.0,
            saturation: 5.0,
            food_level: 20,
            ..PlayerState::default()
        };
        let progs = Arc::new(words::tiny_programs());
        let mut pool_eager = WorkerPool::with_workers(
            1,
            Arc::new(RwLock::new(build_flat_mirror(-1..=6, -1..=1))),
            progs.clone(),
        );
        let mut pool_lazy = WorkerPool::with_workers(
            1,
            Arc::new(RwLock::new(build_flat_mirror(-1..=6, -1..=1))),
            progs,
        );
        pool_eager.spawn_one(1, protocol::MODE_MATERIALIZED, make_state());
        pool_lazy.spawn_one(1, protocol::MODE_VIRTUAL, make_state());

        let st_a = pool_eager.goal_one(1, [80, -60, 0]);
        let st_b = pool_lazy.goal_one(1, [80, -60, 0]);
        let pred_t = st_a[0].predicted_ticks;
        assert_eq!(pred_t, st_b[0].predicted_ticks);

        let cfg = SwarmConfig {
            canary_count: 0,
            ..SwarmConfig::default()
        };
        let mut sent_a = FxHashMap::default();
        let mut sent_b = FxHashMap::default();
        let mut checked = 0u32;
        for tick in 1..=(pred_t as u64 + 10) {
            let (oa, _) = pool_eager.step_tick(tick, cfg, SwarmPosture::default(), &[], &mut sent_a, false, false);
            let (ob, _) = pool_lazy.step_tick(tick, cfg, SwarmPosture::default(), &[], &mut sent_b, true, false);
            if let (Some(cmd), Some(rb)) = (oa.tick_cmds.first(), ob.render_bots.first()) {
                assert!(
                    (cmd.x - rb.x).abs() < 1e-4
                        && (cmd.y - rb.y).abs() < 1e-4
                        && (cmd.z - rb.z).abs() < 1e-4,
                    "tick {tick} curve mismatch eager=({},{},{}) lazy=({},{},{})",
                    cmd.x, cmd.y, cmd.z, rb.x, rb.y, rb.z
                );
                checked += 1;
                if ob.stats.settled_count == 1 {
                    assert_eq!(oa.stats.settled_count, 1);
                    assert!(checked >= pred_t);
                    return;
                }
            }
        }
        panic!("step curve test did not settle within predicted window");
    }

    #[test]
    fn imperial_formations_solve_cleanly() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-4..=10, -4..=10)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::new(world, progs);
        pool.spawn_batch(
            100,
            1,
            0,
            protocol::MODE_VIRTUAL,
            10,
            -10.5,
            -60.0,
            -10.5,
            1.5,
            1.5,
        );
        for (mode, param) in [
            (FORM_WEDGE, 0),
            (FORM_QUAD, 10),
            (FORM_CITADEL, 6),
            (FORM_CORRIDOR, 10),
        ] {
            let sum = pool.goal_batch_ext(20, -60, 0, mode, param, 0);
            assert_eq!(
                sum.ready + sum.already,
                100,
                "mode={mode} ready={} already={} refused={} sample={}",
                sum.ready,
                sum.already,
                sum.refused,
                sum.sample_msg
            );
            assert_eq!(sum.refused, 0);
            assert_eq!(sum.joints, 0);
        }
    }

    #[test]
    fn stop_midair_falls_and_settles_on_ground() {
        let make_state = |z: f64| PlayerState {
            position: Vec3::new(0.5, -60.0, z),
            on_ground: true,
            main_supporting_block: Some([0, -61, z.floor() as i32]),
            health: 20.0,
            saturation: 5.0,
            food_level: 20,
            ..PlayerState::default()
        };
        let world = Arc::new(RwLock::new(build_flat_mirror(-1..=6, -1..=1)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(1, world, progs);
        pool.spawn_one(1, protocol::MODE_VIRTUAL, make_state(0.5));
        pool.spawn_one(2, protocol::MODE_MATERIALIZED, make_state(2.5));
        assert_eq!(pool.goal_one(1, [80, -60, 0])[0].status, protocol::PLAN_STATUS_READY);
        assert_eq!(pool.goal_one(2, [80, -60, 2])[0].status, protocol::PLAN_STATUS_READY);

        let cfg = SwarmConfig {
            canary_count: 0,
            ..SwarmConfig::default()
        };
        let mut last_sent = FxHashMap::default();
        let mut tick = 0u64;
        let mut both_airborne = false;
        for _ in 0..20 {
            tick += 1;
            let (out, _) = pool.step_tick(tick, cfg, SwarmPosture::default(), &[], &mut last_sent, true, false);
            let virt_air = out.render_bots.iter().find(|r| r.bot_index == 0).is_some_and(|r| r.y > -59.5);
            let mat_air = out.tick_cmds.iter().find(|c| c.bot_id == 2).is_some_and(|c| c.y > -59.5);
            if virt_air && mat_air {
                both_airborne = true;
                break;
            }
        }
        assert!(both_airborne, "bots never became airborne");

        pool.stop_all();

        let mut saw_settling = false;
        let mut last_virt = None;
        let mut last_mat = None;
        for _ in 0..60 {
            tick += 1;
            let (out, _) = pool.step_tick(tick, cfg, SwarmPosture::default(), &[], &mut last_sent, true, false);
            if let Some(r) = out.render_bots.iter().find(|r| r.bot_index == 0) {
                last_virt = Some((r.y, (r.flags & 1) != 0));
            }
            if let Some(c) = out.tick_cmds.iter().find(|c| c.bot_id == 2) {
                last_mat = Some((c.y, (c.flags & protocol::FLAG_ON_GROUND) != 0));
            }
            if out.stats.active_bots > 0 {
                saw_settling = true;
            } else {
                assert!(saw_settling, "airborne bots froze without settling");
                let (vy, vg) = last_virt.expect("virtual settled pose");
                let (my, mg) = last_mat.expect("materialized settled pose");
                assert!((vy - (-60.0)).abs() < 1e-6 && vg, "virtual landed at y={vy}, ground={vg}");
                assert!((my - (-60.0)).abs() < 1e-6 && mg, "materialized landed at y={my}, ground={mg}");
                return;
            }
        }
        panic!("stopped airborne bots did not settle within 60 ticks");
    }

    #[test]
    fn remove_bot_drops_from_pool_and_knockback_self_sync_settles() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-1..=4, -1..=2)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(1, world, progs);
        let base_state = PlayerState {
            position: Vec3::new(0.5, -60.0, 0.5),
            on_ground: true,
            main_supporting_block: Some([0, -61, 0]),
            health: 20.0,
            saturation: 5.0,
            food_level: 20,
            ..PlayerState::default()
        };
        pool.spawn_one(1, protocol::MODE_MATERIALIZED, base_state.clone());
        pool.spawn_one(2, protocol::MODE_MATERIALIZED, PlayerState {
            position: Vec3::new(0.5, -60.0, 2.5),
            ..base_state
        });
        assert_eq!(pool.total_bots(), 2);

        // Simulate knockback synced from server (`reason = 2`).
        let kb_state = PlayerState {
            position: Vec3::new(0.5, -60.0, 2.5),
            velocity: Vec3::new(0.0, 0.36, 0.4),
            on_ground: false,
            health: 12.0,
            saturation: 5.0,
            food_level: 20,
            ..PlayerState::default()
        };
        pool.self_sync(2, kb_state, 2);

        // Remove bot 1 (slain).
        pool.remove_one(1);
        assert_eq!(pool.total_bots(), 1);

        let cfg = SwarmConfig {
            canary_count: 0,
            ..SwarmConfig::default()
        };
        let mut last_sent = FxHashMap::default();
        let mut saw_airborne = false;
        let mut landed_cmd = None;
        for tick in 1..=60u64 {
            let (out, _) = pool.step_tick(tick, cfg, SwarmPosture::default(), &[], &mut last_sent, true, tick == 1);
            if tick == 1 {
                assert_eq!(out.render_bots.len(), 1);
            } else {
                assert!(out.render_bots.len() <= 1);
            }
            if let Some(cmd) = out.tick_cmds.iter().find(|c| c.bot_id == 2) {
                if cmd.y > -59.8 {
                    saw_airborne = true;
                }
                if (cmd.flags & protocol::FLAG_SETTLED) != 0 {
                    landed_cmd = Some((cmd.y, (cmd.flags & protocol::FLAG_ON_GROUND) != 0));
                }
            } else if saw_airborne && out.stats.active_bots == 0 {
                let (my, mg) = landed_cmd.expect("settled tick_cmd");
                assert!((my - (-60.0)).abs() < 1e-6 && mg, "y={my}, ground={mg}");
                return;
            }
        }
        panic!("knockback self_sync bot did not arc and settle cleanly");
    }

    #[test]
    fn shared_goal_field_reused_across_workers_for_obstacle_batch() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-2..=3, -2..=2)));
        {
            let mut wg = world.write().unwrap();
            for z in -4..=4 {
                assert!(wg.set_block_delta([12, -60, z], "minecraft:stone"));
                assert!(wg.set_block_delta([12, -59, z], "minecraft:stone"));
            }
        }
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(4, world.clone(), progs);
        pool.spawn_batch(
            16,
            1,
            0,
            protocol::MODE_VIRTUAL,
            4,
            0.5,
            -60.0,
            -2.5,
            1.0,
            1.5,
        );

        let sum1 = pool.goal_batch(24, -60, 0, false);
        assert_eq!(sum1.ready, 16, "ready={}, refused={}, sample={}", sum1.ready, sum1.refused, sum1.sample_msg);
        assert_eq!(sum1.refused, 0);
        assert_eq!(sum1.joints, 0);
        assert!(sum1.shared_topo_builds >= 1, "expected shared TopoField prewarm");
        assert_eq!(sum1.worker_topo_builds, 0, "workers rebuilt TopoField despite shared cache: {}", sum1.worker_topo_builds);
        assert_eq!(sum1.worker_far_builds, 0, "workers rebuilt FarField despite shared cache: {}", sum1.worker_far_builds);

        // Open a center breach and verify section invalidation + shorter detour recompute cleanly.
        assert!(world.write().unwrap().set_block_delta([12, -60, 0], "minecraft:air"));
        assert!(world.write().unwrap().set_block_delta([12, -59, 0], "minecraft:air"));
        pool.replan_in_corridor([12, -60, 0]);
        pool.replan_in_corridor([12, -59, 0]);

        let sum2 = pool.goal_batch(24, -60, 0, false);
        assert_eq!(sum2.ready, 16, "replan ready={}, refused={}, sample={}", sum2.ready, sum2.refused, sum2.sample_msg);
        assert!(sum2.max_pred_t <= sum1.max_pred_t, "breached wall max_t={} exceeded walled max_t={}", sum2.max_pred_t, sum1.max_pred_t);
        assert_eq!(sum2.worker_topo_builds, 0);
    }

    #[test]
    fn four_d_reservations_separate_bottleneck_bridge_without_drift() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-1..=2, -2..=2)));
        {
            let mut wg = world.write().unwrap();
            for z in -15..=15 {
                if z == 0 {
                    continue;
                }
                assert!(wg.set_block_delta([10, -60, z], "minecraft:stone"));
                assert!(wg.set_block_delta([10, -59, z], "minecraft:stone"));
            }
        }
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(4, world, progs);
        pool.spawn_batch(
            8,
            1,
            0,
            protocol::MODE_MATERIALIZED,
            2,
            2.5,
            -60.0,
            -1.5,
            1.5,
            1.5,
        );

        let sum = pool.goal_batch(18, -60, 0, false);
        assert_eq!(sum.ready, 8, "ready={}, refused={}, sample={}", sum.ready, sum.refused, sum.sample_msg);
        assert_eq!(sum.refused, 0);
        assert!(sum.reserve_stats.staggered_bots >= 4, "expected bottleneck staggering, got {:?}", sum.reserve_stats);
        assert_eq!(sum.reserve_stats.residual_conflicts, 0, "residual conflicts: {:?}", sum.reserve_stats);

        let cfg = SwarmConfig {
            canary_count: 0,
            ..SwarmConfig::default()
        };
        let mut last_sent = FxHashMap::default();
        let max_ticks = u64::from(sum.max_pred_t) + u64::from(sum.reserve_stats.max_extra_delay) + 40;
        let mut all_settled = false;
        for tick in 1..=max_ticks {
            let (out, _) = pool.step_tick(tick, cfg, SwarmPosture::default(), &[], &mut last_sent, false, false);
            let mut doorway_cells: Vec<(u32, [i32; 3])> = Vec::new();
            for cmd in &out.tick_cmds {
                let cx = cmd.x.floor() as i32;
                let cy = cmd.y.floor() as i32;
                let cz = cmd.z.floor() as i32;
                if (9..=11).contains(&cx) && cz == 0 {
                    if let Some((other_id, _)) = doorway_cells.iter().find(|(_, c)| *c == [cx, cy, cz]) {
                        panic!(
                            "doorway collision at tick {tick}: bots {other_id} and {} both inside {:?}",
                            cmd.bot_id, [cx, cy, cz]
                        );
                    }
                    doorway_cells.push((cmd.bot_id, [cx, cy, cz]));
                }
            }
            if out.stats.settled_count == 8 {
                assert_eq!(out.stats.arrived_count, 8);
                assert_eq!(out.stats.exact_t_count, 8, "staggering must keep 100% exact law ticks");
                all_settled = true;
                break;
            }
        }
        assert!(all_settled, "not all 8 bottleneck bots settled within {max_ticks} ticks");
    }

    #[test]
    fn disjoint_open_formation_pays_zero_reservation_delay() {
        let world = Arc::new(RwLock::new(build_flat_mirror(-1..=5, -1..=2)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::new(world, progs);
        pool.spawn_batch(
            32,
            1,
            0,
            protocol::MODE_VIRTUAL,
            4,
            0.5,
            -60.0,
            0.5,
            2.0,
            2.0,
        );
        let sum = pool.goal_batch(40, -60, 0, true);
        assert_eq!(sum.ready, 32, "ready={}, refused={}, sample={}", sum.ready, sum.refused, sum.sample_msg);
        assert_eq!(sum.reserve_stats.checked_bots, 32);
        assert_eq!(sum.reserve_stats.staggered_bots, 0, "parallel open grid should have 0 conflicts: {:?}", sum.reserve_stats);
        assert_eq!(sum.reserve_stats.max_extra_delay, 0);
    }

    #[test]
    fn gpu_min_plus_matches_cpu_below_and_above_crossover() {
        use gpu::min_plus::{LegsCost, PackedTopoGrid, solve_min_plus_cpu};
        let legs = LegsCost::default();

        // Below crossover: 4 * (32 * 6 * 32) = 24,576 < 250,000 -> runs on CPU.
        let small = PackedTopoGrid::synthetic_arena(32, 6, 32);
        let small_goals = vec![
            (6 * small.dy + 1) * small.dx + 6,
            (24 * small.dy + 1) * small.dx + 24,
            (8 * small.dy + 1) * small.dx + 26,
            (26 * small.dy + 1) * small.dx + 8,
        ];
        assert!(small_goals.len() * small.vol() < 250_000);
        let (below_out, below_gpu) = WorkerPool::min_plus_batch(&small, &small_goals, legs, 256).unwrap();
        assert!(!below_gpu, "expected CPU below min_plus crossover");
        assert_eq!(below_out, solve_min_plus_cpu(&small, &small_goals, legs, 1));

        // Above crossover: 16 * (64 * 8 * 64) = 524,288 >= 250,000 -> runs on RX 7900 XTX GPU.
        let big = PackedTopoGrid::synthetic_arena(64, 8, 64);
        let big_goals: Vec<usize> = (0..16usize)
            .map(|g| {
                let x = 8 + (g * 7) % 48;
                let z = 8 + (g * 11) % 48;
                (z * big.dy + 1) * big.dx + x
            })
            .collect();
        assert!(big_goals.len() * big.vol() >= 250_000);
        let (above_out, above_gpu) = WorkerPool::min_plus_batch(&big, &big_goals, legs, 256).unwrap();
        if WorkerPool::gpu_device_name().is_some() {
            assert!(above_gpu, "expected GPU above min_plus crossover");
        }
        let cpu_ref = solve_min_plus_cpu(&big, &big_goals, legs, 1);
        let max_err = cpu_ref
            .iter()
            .zip(above_out.iter())
            .filter(|(a, b)| a.is_finite() || b.is_finite())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(max_err < 1e-3, "min_plus parity failed above crossover: max_err={max_err}");

        // WorldMirror packed grid reuse + invalidation + CPU/GPU parity.
        let mut mirror = build_flat_mirror(0..=3, 0..=3);
        for z in 4..=24 {
            if z != 14 {
                assert!(mirror.set_block_delta([16, -60, z], "minecraft:stone"));
                assert!(mirror.set_block_delta([16, -59, z], "minecraft:stone"));
            }
        }
        let world = Arc::new(RwLock::new(mirror));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(1, world.clone(), progs);
        let (bmin, bmax) = ([0, -61, 0], [47, -54, 47]); // 48 * 8 * 48 = 18,432 cells
        let g1 = pool.cached_topo_grid(bmin, bmax);
        let g2 = pool.cached_topo_grid(bmin, bmax);
        assert!(Arc::ptr_eq(&g1, &g2), "expected PackedTopoGrid reuse when provenance unchanged");

        let mirror_goals: Vec<usize> = (0..16usize)
            .map(|g| g1.idx([24 + (g as i32) % 16, -60, 4 + ((g * 3) as i32) % 36]).unwrap())
            .collect(); // 16 * 18,432 = 294,912 >= 250,000
        let (g_out, w_dist, w_gpu) = pool.solve_min_plus_world(bmin, bmax, &mirror_goals, legs, 256).unwrap();
        assert!(Arc::ptr_eq(&g1, &g_out));
        if WorkerPool::gpu_device_name().is_some() {
            assert!(w_gpu, "expected GPU on WorldMirror min_plus above crossover");
        }
        let w_cpu = solve_min_plus_cpu(&g1, &mirror_goals, legs, 1);
        let w_err = w_cpu
            .iter()
            .zip(w_dist.iter())
            .filter(|(a, b)| a.is_finite() || b.is_finite())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(w_err < 1e-3, "WorldMirror min_plus parity failed: w_err={w_err}");

        assert!(world.write().unwrap().set_block_delta([16, -60, 10], "minecraft:air"));
        let g3 = pool.cached_topo_grid(bmin, bmax);
        assert!(!Arc::ptr_eq(&g1, &g3), "expected PackedTopoGrid rebuild after block delta");
    }

    #[test]
    fn gpu_rollout_matches_cpu_below_and_above_crossover() {
        use gpu::rollout::{
            ROLLOUT_FLAG_FORWARD, ROLLOUT_FLAG_JUMP, ROLLOUT_FLAG_SPRINT, RolloutReq,
            SolidVoxelBox, run_rollouts_cpu,
        };
        let make_reqs = |count: usize, ticks: u32| -> Vec<RolloutReq> {
            (0..count)
                .map(|i| RolloutReq {
                    x: 8.5 + ((i % 32) as f64),
                    y: -60.0,
                    z: 8.5 + (((i / 32) % 32) as f64),
                    vx: 0.0,
                    vy: 0.0,
                    vz: 0.0,
                    yaw: ((i * 17) % 360) as f32,
                    flags: ROLLOUT_FLAG_FORWARD | ROLLOUT_FLAG_SPRINT | if i % 3 == 0 { ROLLOUT_FLAG_JUMP } else { 0 },
                    ticks,
                    on_ground: 1,
                })
                .collect()
        };

        let world = Arc::new(RwLock::new(build_flat_mirror(0..=3, 0..=3)));
        let progs = Arc::new(words::tiny_programs());
        let mut pool = WorkerPool::with_workers(1, world.clone(), progs);
        let (bmin, bmax) = ([0, -61, 0], [63, -54, 63]);
        let v1 = pool.cached_solid_voxels(bmin, bmax);
        let v2 = pool.cached_solid_voxels(bmin, bmax);
        assert!(Arc::ptr_eq(&v1, &v2), "expected SolidVoxelBox reuse when provenance unchanged");
        let voxels_flat = SolidVoxelBox::flat_floor([0, -61, 0], 64, 8, 64, 0);
        assert_eq!(v1.bits, voxels_flat.bits, "WorldMirror SolidVoxelBox must match flat_floor bits");

        // Below crossover: 1,024 * 64 = 65,536 < 1,000,000 -> CPU.
        let small_reqs = make_reqs(1_024, 64);
        let (small_out, small_gpu) = WorkerPool::rollout_batch(&small_reqs, &v1).unwrap();
        assert!(!small_gpu, "expected CPU below rollout crossover");
        let small_cpu = run_rollouts_cpu(&small_reqs, &v1, 1);
        let small_err = small_cpu
            .iter()
            .zip(small_out.iter())
            .map(|(a, b)| (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()))
            .fold(0.0f64, f64::max);
        assert!(small_err < 1e-12, "rollout below-crossover mismatch: {small_err}");

        // Above crossover: 32,768 * 64 = 2,097,152 >= 1,000,000 -> GPU.
        let big_reqs = make_reqs(32_768, 64);
        let (big_out, big_gpu) = WorkerPool::rollout_batch(&big_reqs, &v1).unwrap();
        if WorkerPool::gpu_device_name().is_some() {
            assert!(big_gpu, "expected GPU above rollout crossover");
        }
        let big_cpu = run_rollouts_cpu(&big_reqs, &v1, 4);
        let big_err = big_cpu
            .iter()
            .zip(big_out.iter())
            .map(|(a, b)| (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()))
            .fold(0.0f64, f64::max);
        assert!(big_err < 1e-10, "rollout above-crossover mismatch: {big_err}");

        // WorldMirror rollout reuse + invalidation parity.
        let (w_out, w_gpu) = pool.rollout_world(bmin, bmax, &big_reqs).unwrap();
        if WorkerPool::gpu_device_name().is_some() {
            assert!(w_gpu, "expected GPU on WorldMirror rollout above crossover");
        }
        let w_err = big_cpu
            .iter()
            .zip(w_out.iter())
            .map(|(a, b)| (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()))
            .fold(0.0f64, f64::max);
        assert!(w_err < 1e-10, "WorldMirror rollout mismatch: {w_err}");

        assert!(world.write().unwrap().set_block_delta([20, -60, 20], "minecraft:stone"));
        let v3 = pool.cached_solid_voxels(bmin, bmax);
        assert!(!Arc::ptr_eq(&v1, &v3), "expected SolidVoxelBox rebuild after block delta");
    }

    #[test]
    fn gpu_crowd_push_matches_cpu_below_and_above_crossover() {
        use gpu::crowd_push::{CrowdState, step_crowd_cpu};
        let small = CrowdState::dense_arena(2_048, 16.0);
        let mut s_auto = small.clone();
        let mut s_ref = small;
        let used_gpu = WorkerPool::dense_crowd_push_step(&mut s_auto, 0.0, 0.0, 16, 16, 1).unwrap();
        assert!(!used_gpu, "expected CPU below crowd crossover");
        step_crowd_cpu(&mut s_ref, 0.0, 0.0, 16, 16, 1, 1);
        let err_small = s_ref
            .vel_x
            .iter()
            .zip(s_auto.vel_x.iter())
            .chain(s_ref.vel_z.iter().zip(s_auto.vel_z.iter()))
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        assert!(err_small < 1e-12);

        let large = CrowdState::dense_arena(16_384, 32.0);
        let mut l_auto = large.clone();
        let mut l_ref = large;
        let used_gpu = WorkerPool::dense_crowd_push_step(&mut l_auto, 0.0, 0.0, 32, 32, 4).unwrap();
        if WorkerPool::gpu_device_name().is_some() {
            assert!(used_gpu, "expected GPU above crowd crossover");
        }
        step_crowd_cpu(&mut l_ref, 0.0, 0.0, 32, 32, 4, 4);
        let err_large = l_ref
            .pos_x
            .iter()
            .zip(l_auto.pos_x.iter())
            .chain(l_ref.pos_z.iter().zip(l_auto.pos_z.iter()))
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        assert!(err_large < 1e-6, "crowd push parity error: {err_large}");
    }
}
