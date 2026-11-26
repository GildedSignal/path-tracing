//! Sharded persistent worker pool owning per-worker `GeneratorStore`s and stepping bots in parallel.
use std::{
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, SyncSender},
    },
    thread::JoinHandle,
    time::Instant,
};

use law::{BlockPos, PlayerState, Vec3, World, codec::Interner};
use plan::{
    Goal, Kit, PlanOutcome, SharedFieldSnapshot,
    source::{CorridorResult, CorridorSource},
    store::GeneratorStore,
};
use words::Arguments;

use crate::{
    protocol::{
        FLAG_ARRIVED, FLAG_HORIZ_COL, FLAG_ON_GROUND, FLAG_SETTLED, FLAG_START_SPRINT,
        FLAG_STOP_SPRINT, FORM_CITADEL, FORM_CORRIDOR, FORM_GRID, FORM_POINT, FORM_QUAD,
        FORM_WEDGE, MODE_AUTO, MODE_MATERIALIZED, MODE_VIRTUAL, PLAN_STATUS_ALREADY_THERE,
        PLAN_STATUS_NEEDS_JOINTS, PLAN_STATUS_READY, PLAN_STATUS_REFUSED, POSTURE_SALUTE,
        POSTURE_STARE, TIER_FLAG_ON_GROUND, TIER_FLAG_SNEAKING, TIER_FLAG_SPRINTING,
        TIER_MATERIALIZED, TIER_VIRTUAL, TIER_VISUAL,
    },
    reserve::{BotFootprint, ReservationTable, ReserveStats, extract_footprint},
    world_mirror::{FxHashMap, FxHashSet, WorldMirror},
};

#[derive(Clone, Copy)]
pub struct Observer {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub view_dist: f32,
}

#[derive(Clone, Copy)]
pub struct SwarmConfig {
    pub max_visual: u32,
    pub max_materialized: u32,
    pub materialize_radius: f32,
    pub canary_count: u32,
    pub canary_period: u32,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            max_visual: 128,
            max_materialized: 32,
            materialize_radius: 16.0,
            canary_count: 8,
            canary_period: 100,
        }
    }
}

#[derive(Clone, Copy)]
pub struct StepPose {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub yaw: f32,
    pub speed: f32,
    pub on_ground: bool,
    pub sprinting: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct StepCurveKey {
    program: u16,
    ticks: u32,
    yaw_index: u8,
    refined_yaw_bits: u32,
    init_yaw_bits: u32,
    vx_bits: u64,
    vy_bits: u64,
    vz_bits: u64,
    on_ground: bool,
    sprinting: bool,
    no_jump_delay: u8,
}

#[derive(Clone, Copy, Default)]
pub struct SwarmPosture {
    pub flags: u8,
    pub tx: f64,
    pub ty: f64,
    pub tz: f64,
}

pub struct CertifiedStep {
    pub program: u16,
    pub args: Arguments,
    pub ticks: u32,
    pub start_pos: Vec3,
    pub end: plan::state_view::StateAt,
    pub curve: Option<Arc<Vec<StepPose>>>,
}

pub(crate) const JUMP_ARC_Y: [f64; 12] = [
    0.0,
    0.42,
    0.7532,
    1.001336,
    1.166109,
    1.249187,
    1.252203,
    1.176759,
    1.024424,
    0.796736,
    0.495201,
    0.121297,
];

pub enum BotPhase {
    Idle,
    Executing {
        steps: Vec<CertifiedStep>,
        idx: usize,
        elapsed: u32,
        lazy_ticks: u32,
        delay_ticks: u16,
        step_start_state: PlayerState,
        goal_air: BlockPos,
        corridor: (BlockPos, BlockPos),
    },
    Settling,
}

pub struct BotSlot {
    pub bot_id: u32,
    pub bot_index: u32,
    pub rel_x: f64,
    pub rel_z: f64,
    pub mode: u8,
    pub tier: u8,
    pub state: PlayerState,
    pub phase: BotPhase,
    pub pending_replan: Option<BlockPos>,
    pub predicted_ticks: u32,
    pub active_ticks: u32,
    pub arrived: bool,
    pub settled: bool,
    pub hooked_corridor: Option<(BlockPos, BlockPos)>,
    pub dirty_render: bool,
}

impl BotSlot {
    pub fn approx_pose(&self) -> (Vec3, f32, f32, f32, bool, bool) {
        if let BotPhase::Executing {
            steps,
            idx,
            elapsed,
            lazy_ticks,
            step_start_state,
            ..
        } = &self.phase
            && *lazy_ticks > 0
            && let Some(step) = steps.get(*idx)
        {
            if let Some(curve) = &step.curve
                && *elapsed > 0
                && let Some(p) = curve.get((*elapsed - 1) as usize)
            {
                return (
                    Vec3::new(
                        step.start_pos.x + f64::from(p.dx),
                        step.start_pos.y + f64::from(p.dy),
                        step.start_pos.z + f64::from(p.dz),
                    ),
                    p.yaw,
                    self.state.pitch,
                    p.speed,
                    p.on_ground,
                    p.sprinting,
                );
            }
            let ticks = step.ticks.max(1);
            let frac = f64::from(*elapsed) / f64::from(ticks);
            let dx = step.end.position.x - step.start_pos.x;
            let dz = step.end.position.z - step.start_pos.z;
            let x = step.start_pos.x + dx * frac;
            let z = step.start_pos.z + dz * frac;
            let base_y = step.start_pos.y + (step.end.position.y - step.start_pos.y) * frac;
            let (y, on_ground) = if step.program == plan::store::SPRINT_JUMP && step_start_state.on_ground {
                let phase = (*elapsed as usize) % JUMP_ARC_Y.len();
                (step.start_pos.y.min(step.end.position.y) + JUMP_ARC_Y[phase], phase == 0)
            } else {
                (base_y, self.state.on_ground)
            };
            let yaw = step.args.refined_yaw.unwrap_or_else(|| {
                if dx * dx + dz * dz > 1e-8 {
                    (-dx).atan2(dz).to_degrees() as f32
                } else {
                    self.state.yaw
                }
            });
            let speed = (dx.hypot(dz) / f64::from(ticks)) as f32;
            let sprinting = step.program != plan::store::WALK;
            return (Vec3::new(x, y, z), yaw, self.state.pitch, speed, on_ground, sprinting);
        }
        let speed = self.state.velocity.x.hypot(self.state.velocity.z) as f32;
        (
            self.state.position,
            self.state.yaw,
            self.state.pitch,
            speed,
            self.state.on_ground,
            self.state.sprinting,
        )
    }
}

pub struct PlanStatusOut {
    pub bot_id: u32,
    pub status: u32,
    pub predicted_ticks: u32,
    pub solve_ns: u64,
    pub msg: String,
}

pub struct TierChangeOut {
    pub bot_id: u32,
    pub bot_index: u32,
    pub new_tier: u8,
    pub flags: u32,
    pub state: PlayerState,
}

pub struct TickCmdOut {
    pub bot_id: u32,
    pub keys: u32,
    pub flags: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

pub struct VisualOut {
    pub bot_id: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
}

pub struct RenderBotOut {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub bot_index: u16,
    pub speed_q: u8,
    pub flags: u8,
}

pub struct TicketNeed {
    pub bot_id: u32,
    pub cx: i32,
    pub cz: i32,
    pub next_cx: i32,
    pub next_cz: i32,
    pub level: u8,
    pub hooked: bool,
    pub already_mat: bool,
}

pub struct CandidateOut {
    pub dist_sq: f64,
    pub bot_id: u32,
    pub within_mat_radius: bool,
    pub already_mat: bool,
    pub cx: i32,
    pub cz: i32,
    pub next_cx: i32,
    pub next_cz: i32,
}

#[derive(Default)]
pub struct WorkerStepStats {
    pub active_bots: u32,
    pub virt_count: u32,
    pub vis_count: u32,
    pub mat_count: u32,
    pub arrived_count: u32,
    pub settled_count: u32,
    pub exact_t_count: u32,
    pub min_act_t: u32,
    pub max_act_t: u32,
    pub promotions: u32,
    pub demotions: u32,
    pub hooked_promotions: u32,
}

pub struct SolveGoalSpec {
    pub gx: i32,
    pub gy: i32,
    pub gz: i32,
    pub formation_mode: u8,
    pub param: i32,
    pub wave_ticks: u16,
    pub total_bots: u32,
    pub cols: u32,
    pub spacing_x: f64,
    pub spacing_z: f64,
}

#[derive(Default)]
pub struct SolveSummary {
    pub ready: u32,
    pub already: u32,
    pub joints: u32,
    pub refused: u32,
    pub min_pred_t: u32,
    pub max_pred_t: u32,
    pub solve_ns_samples: Vec<u64>,
    pub sample_msg: String,
    pub statuses: Vec<PlanStatusOut>,
    pub reserve_stats: ReserveStats,
    pub shared_topo_builds: u64,
    pub shared_far_builds: u64,
    pub worker_topo_builds: u64,
    pub worker_far_builds: u64,
}

#[derive(Default)]
pub struct StepOutputs {
    pub plan_statuses: Vec<PlanStatusOut>,
    pub tier_changes: Vec<TierChangeOut>,
    pub tick_cmds: Vec<TickCmdOut>,
    pub visuals: Vec<VisualOut>,
    pub render_bots: Vec<RenderBotOut>,
    pub stats: WorkerStepStats,
}

struct SpawnBatchJob {
    count: u32,
    start_bot_id: u32,
    start_index: u32,
    mode: u8,
    cols: u32,
    base_x: f64,
    base_y: f64,
    base_z: f64,
    spacing_x: f64,
    spacing_z: f64,
}

struct GoalChunkInfo {
    chunks: Vec<[i32; 2]>,
    bot_min: Option<BlockPos>,
    bot_max: Option<BlockPos>,
    sample_state: Option<PlayerState>,
    goal_min: Option<BlockPos>,
    goal_max: Option<BlockPos>,
    unique_goal: Option<BlockPos>,
    multi_goal: bool,
}

enum WorkerTask {
    Clear,
    ResetStores,
    SpawnOne {
        bot_id: u32,
        bot_index: u32,
        mode: u8,
        state: PlayerState,
    },
    SpawnBatch {
        worker_idx: usize,
        worker_count: usize,
        job: Arc<SpawnBatchJob>,
    },
    RemoveOne {
        bot_id: u32,
    },
    SelfSync {
        bot_id: u32,
        state: PlayerState,
        reason: u32,
    },
    StopOne {
        bot_id: u32,
    },
    StopAll,
    MarkReplanAt {
        pos: BlockPos,
    },
    GoalOne {
        bot_id: u32,
        goal_air: BlockPos,
        emit_status: bool,
    },
    CollectGoalOneChunks {
        bot_id: u32,
        goal_air: BlockPos,
    },
    CollectGoalChunks {
        spec: Arc<SolveGoalSpec>,
    },
    SolveBatch {
        spec: Arc<SolveGoalSpec>,
        emit_statuses: bool,
        shared_fields: Option<Arc<SharedFieldSnapshot>>,
        collect_footprints: bool,
    },
    ApplyDelays {
        delays: Vec<(u32, u16)>,
    },
    CollectStepIntents {
        server_tick: u64,
        total_bots: usize,
        cfg: SwarmConfig,
        observers: Arc<Vec<Observer>>,
    },
    StepBots {
        server_tick: u64,
        total_bots: usize,
        cfg: SwarmConfig,
        posture: SwarmPosture,
        observers: Arc<Vec<Observer>>,
        mat_allowed: Arc<FxHashSet<u32>>,
        vis_allowed: Arc<FxHashSet<u32>>,
        promote_allowed: Arc<FxHashSet<u32>>,
        emit_statuses: bool,
        emit_render_bots: bool,
        force_full_render: bool,
    },
}

enum WorkerReply {
    Ack,
    GoalOneChunks(Vec<[i32; 2]>),
    GoalBatchChunks(GoalChunkInfo),
    SolveDone(SolveSummary, Vec<BotFootprint>),
    StepIntents {
        candidates: Vec<CandidateOut>,
        ticket_needs: Vec<TicketNeed>,
    },
    StepDone(StepOutputs),
}

struct WorkerHandle {
    tx: SyncSender<WorkerTask>,
    rx: Receiver<WorkerReply>,
    join: Option<JoinHandle<()>>,
}

struct WorkerState {
    bots: Vec<BotSlot>,
    by_id: FxHashMap<u32, usize>,
    store: GeneratorStore,
    codec: Interner,
    step_curves: FxHashMap<StepCurveKey, Arc<Vec<StepPose>>>,
    world: Arc<RwLock<WorldMirror>>,
    programs: Arc<Vec<words::Program>>,
}

impl WorkerState {
    fn add_bot(&mut self, slot: BotSlot) {
        let id = slot.bot_id;
        if let Some(&idx) = self.by_id.get(&id) {
            self.bots[idx] = slot;
        } else {
            let idx = self.bots.len();
            self.bots.push(slot);
            self.by_id.insert(id, idx);
        }
    }

    fn remove_bot(&mut self, bot_id: u32) -> bool {
        let Some(idx) = self.by_id.remove(&bot_id) else {
            return false;
        };
        self.bots.swap_remove(idx);
        if let Some(moved) = self.bots.get(idx) {
            self.by_id.insert(moved.bot_id, idx);
        }
        true
    }

    fn get_bot_mut(&mut self, bot_id: u32) -> Option<&mut BotSlot> {
        let idx = *self.by_id.get(&bot_id)?;
        Some(&mut self.bots[idx])
    }

    fn handle(&mut self, task: WorkerTask) -> WorkerReply {
        match task {
            WorkerTask::Clear => {
                self.bots.clear();
                self.by_id.clear();
                self.step_curves.clear();
                WorkerReply::Ack
            }
            WorkerTask::ResetStores => {
                self.store = GeneratorStore::default();
                self.codec = Interner::default();
                self.step_curves.clear();
                WorkerReply::Ack
            }
            WorkerTask::SpawnOne {
                bot_id,
                bot_index,
                mode,
                state,
            } => {
                let tier = if mode == MODE_MATERIALIZED {
                    TIER_MATERIALIZED
                } else {
                    TIER_VIRTUAL
                };
                self.add_bot(BotSlot {
                    bot_id,
                    bot_index,
                    rel_x: 0.0,
                    rel_z: 0.0,
                    mode,
                    tier,
                    state,
                    phase: BotPhase::Idle,
                    pending_replan: None,
                    predicted_ticks: 0,
                    active_ticks: 0,
                    arrived: false,
                    settled: false,
                    hooked_corridor: None,
                    dirty_render: true,
                });
                WorkerReply::Ack
            }
            WorkerTask::SpawnBatch {
                worker_idx,
                worker_count,
                job,
            } => {
                let mut spawned = Vec::new();
                {
                    let world = self.world.read().unwrap();
                    let tier = if job.mode == MODE_MATERIALIZED {
                        TIER_MATERIALIZED
                    } else {
                        TIER_VIRTUAL
                    };
                    let cols = job.cols.max(1);
                    for local_i in (worker_idx..job.count as usize).step_by(worker_count) {
                        let i = local_i as u32;
                        let rel_x = (i % cols) as f64 * job.spacing_x;
                        let rel_z = (i / cols) as f64 * job.spacing_z;
                        let x = job.base_x + rel_x;
                        let y = job.base_y;
                        let z = job.base_z + rel_z;
                        let bx = x.floor() as i32;
                        let by = y.floor() as i32;
                        let bz = z.floor() as i32;
                        let support = (1..=8)
                            .map(|d| [bx, by - d, bz])
                            .find(|&c| world.loaded(c) && !world.is_air(c));
                        let state = PlayerState {
                            position: Vec3::new(x, y, z),
                            velocity: Vec3::default(),
                            yaw: 0.0,
                            pitch: 0.0,
                            on_ground: true,
                            main_supporting_block: support,
                            health: 20.0,
                            saturation: 5.0,
                            food_level: 20,
                            ..PlayerState::default()
                        };
                        spawned.push(BotSlot {
                            bot_id: job.start_bot_id + i,
                            bot_index: job.start_index + i,
                            rel_x,
                            rel_z,
                            mode: job.mode,
                            tier,
                            state,
                            phase: BotPhase::Idle,
                            pending_replan: None,
                            predicted_ticks: 0,
                            active_ticks: 0,
                            arrived: false,
                            settled: false,
                            hooked_corridor: None,
                            dirty_render: true,
                        });
                    }
                }
                for slot in spawned {
                    self.add_bot(slot);
                }
                WorkerReply::Ack
            }
            WorkerTask::RemoveOne { bot_id } => {
                self.remove_bot(bot_id);
                WorkerReply::Ack
            }
            WorkerTask::SelfSync {
                bot_id,
                state,
                reason,
            } => {
                if let Some(bot) = self.get_bot_mut(bot_id) {
                    bot.state = state;
                    bot.dirty_render = true;
                    if reason != 0 {
                        bot.phase = if is_settled(&bot.state) {
                            BotPhase::Idle
                        } else {
                            BotPhase::Settling
                        };
                        bot.pending_replan = None;
                    }
                } else {
                    self.add_bot(BotSlot {
                        bot_id,
                        bot_index: bot_id.saturating_sub(1),
                        rel_x: 0.0,
                        rel_z: 0.0,
                        mode: MODE_MATERIALIZED,
                        tier: TIER_MATERIALIZED,
                        state,
                        phase: BotPhase::Idle,
                        pending_replan: None,
                        predicted_ticks: 0,
                        active_ticks: 0,
                        arrived: false,
                        settled: false,
                        hooked_corridor: None,
                        dirty_render: true,
                    });
                }
                WorkerReply::Ack
            }
            WorkerTask::StopOne { bot_id } => {
                let world = self.world.read().unwrap();
                if let Some(&idx) = self.by_id.get(&bot_id) {
                    let bot = &mut self.bots[idx];
                    materialize_lazy_state(bot, &world, &self.programs);
                    bot.phase = if is_settled(&bot.state) {
                        BotPhase::Idle
                    } else {
                        BotPhase::Settling
                    };
                    bot.pending_replan = None;
                    bot.dirty_render = true;
                }
                WorkerReply::Ack
            }
            WorkerTask::StopAll => {
                let world = self.world.read().unwrap();
                for bot in &mut self.bots {
                    materialize_lazy_state(bot, &world, &self.programs);
                    bot.phase = if is_settled(&bot.state) {
                        BotPhase::Idle
                    } else {
                        BotPhase::Settling
                    };
                    bot.pending_replan = None;
                    bot.dirty_render = true;
                }
                WorkerReply::Ack
            }
            WorkerTask::MarkReplanAt { pos: [x, y, z] } => {
                self.store.invalidate_far_section([
                    x.div_euclid(16),
                    y.div_euclid(16),
                    z.div_euclid(16),
                ]);
                for bot in &mut self.bots {
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
                WorkerReply::Ack
            }
            WorkerTask::GoalOne {
                bot_id,
                goal_air,
                emit_status,
            } => {
                let mut sum = SolveSummary::default();
                let world = self.world.read().unwrap();
                if let Some(&idx) = self.by_id.get(&bot_id) {
                    materialize_lazy_state(&mut self.bots[idx], &world, &self.programs);
                    let out = solve_slot(
                        &mut self.bots[idx],
                        goal_air,
                        0,
                        &world,
                        &self.store,
                        &mut self.codec,
                        &self.programs,
                        &mut self.step_curves,
                    );
                    record_solve(&mut sum, out, emit_status);
                } else if emit_status {
                    sum.statuses.push(PlanStatusOut {
                        bot_id,
                        status: PLAN_STATUS_REFUSED,
                        predicted_ticks: 0,
                        solve_ns: 0,
                        msg: "unknown bot".into(),
                    });
                }
                WorkerReply::SolveDone(sum, Vec::new())
            }
            WorkerTask::CollectGoalOneChunks { bot_id, goal_air } => {
                let mut chunks = FxHashSet::default();
                if let Some(bot) = self.bots.iter().find(|b| b.bot_id == bot_id) {
                    insert_corridor_chunks(&mut chunks, bot.approx_pose().0, goal_air);
                }
                WorkerReply::GoalOneChunks(chunks.into_iter().collect())
            }
            WorkerTask::CollectGoalChunks { spec } => {
                let mut chunks = FxHashSet::default();
                let mut bot_min: Option<BlockPos> = None;
                let mut bot_max: Option<BlockPos> = None;
                let mut sample_state: Option<PlayerState> = None;
                let mut goal_min: Option<BlockPos> = None;
                let mut goal_max: Option<BlockPos> = None;
                let mut unique_goal: Option<BlockPos> = None;
                let mut multi_goal = false;
                let expand_box = |lo: &mut Option<BlockPos>, hi: &mut Option<BlockPos>, p: BlockPos| {
                    *lo = Some(lo.map_or(p, |m| [m[0].min(p[0]), m[1].min(p[1]), m[2].min(p[2])]));
                    *hi = Some(hi.map_or(p, |m| [m[0].max(p[0]), m[1].max(p[1]), m[2].max(p[2])]));
                };
                for bot in &self.bots {
                    let pos = bot.approx_pose().0;
                    let bp = [
                        pos.x.floor() as i32,
                        pos.y.floor() as i32,
                        pos.z.floor() as i32,
                    ];
                    expand_box(&mut bot_min, &mut bot_max, bp);
                    if sample_state.is_none() {
                        sample_state = Some(bot.state.clone());
                    }
                    let goal_air = goal_for_bot(bot, &spec);
                    expand_box(&mut goal_min, &mut goal_max, goal_air);
                    if !multi_goal {
                        match unique_goal {
                            None => unique_goal = Some(goal_air),
                            Some(g) if g != goal_air => multi_goal = true,
                            _ => {}
                        }
                    }
                    insert_corridor_chunks(&mut chunks, pos, goal_air);
                }
                WorkerReply::GoalBatchChunks(GoalChunkInfo {
                    chunks: chunks.into_iter().collect(),
                    bot_min,
                    bot_max,
                    sample_state,
                    goal_min,
                    goal_max,
                    unique_goal,
                    multi_goal,
                })
            }
            WorkerTask::SolveBatch {
                spec,
                emit_statuses,
                shared_fields,
                collect_footprints,
            } => {
                if let Some(snap) = shared_fields.as_deref() {
                    self.store.import_shared_fields(snap);
                }
                let topo_before = self.store.topo_cache_builds();
                let far_before = self.store.far_field_builds();
                let mut sum = SolveSummary::default();
                sum.solve_ns_samples.reserve(self.bots.len());
                let mut footprints = if collect_footprints {
                    Vec::with_capacity(self.bots.len())
                } else {
                    Vec::new()
                };
                let world = self.world.read().unwrap();
                let cols = spec.cols.max(1);
                let rows = spec.total_bots.max(1).div_ceil(cols).max(1);
                for bot in &mut self.bots {
                    bot.pending_replan = None;
                    materialize_lazy_state(bot, &world, &self.programs);
                    let goal_air = goal_for_bot(bot, &spec);
                    let delay = if spec.wave_ticks > 0 {
                        let row = bot.bot_index / cols;
                        ((row * u32::from(spec.wave_ticks)) / rows) as u16
                    } else {
                        0
                    };
                    let out = solve_slot(
                        bot,
                        goal_air,
                        delay,
                        &world,
                        &self.store,
                        &mut self.codec,
                        &self.programs,
                        &mut self.step_curves,
                    );
                    if collect_footprints
                        && out.status == PLAN_STATUS_READY
                        && let BotPhase::Executing { steps, delay_ticks, .. } = &bot.phase
                    {
                        footprints.push(extract_footprint(
                            bot.bot_id,
                            bot.bot_index,
                            *delay_ticks,
                            steps,
                        ));
                    }
                    record_solve(&mut sum, out, emit_statuses);
                }
                sum.worker_topo_builds = self.store.topo_cache_builds().saturating_sub(topo_before);
                sum.worker_far_builds = self.store.far_field_builds().saturating_sub(far_before);
                WorkerReply::SolveDone(sum, footprints)
            }
            WorkerTask::ApplyDelays { delays } => {
                for (bot_id, extra) in delays {
                    if let Some(bot) = self.get_bot_mut(bot_id)
                        && let BotPhase::Executing { delay_ticks, .. } = &mut bot.phase
                    {
                        *delay_ticks = delay_ticks.saturating_add(extra);
                    }
                }
                WorkerReply::Ack
            }
            WorkerTask::CollectStepIntents {
                server_tick,
                total_bots,
                cfg,
                observers,
            } => {
                let mut candidates = Vec::new();
                let mut ticket_needs = Vec::new();
                let mat_rad_enter_sq = f64::from(cfg.materialize_radius) * f64::from(cfg.materialize_radius);
                let mat_rad_stay = f64::from(cfg.materialize_radius + 2.0);
                let mat_rad_stay_sq = mat_rad_stay * mat_rad_stay;
                let want_vis = cfg.max_visual > 0;

                for bot in &self.bots {
                    let hooked = bot_wants_hooked(bot);
                    let canary = bot.mode == MODE_AUTO
                        && is_canary(bot.bot_index, total_bots, server_tick, &cfg);
                    let force_mat = bot.mode == MODE_MATERIALIZED || hooked || canary;
                    if force_mat {
                        let (cx, cz, next_cx, next_cz) = bot_and_next_chunk(bot);
                        ticket_needs.push(TicketNeed {
                            bot_id: bot.bot_id,
                            cx,
                            cz,
                            next_cx,
                            next_cz,
                            level: if hooked { 32 } else { 33 },
                            hooked,
                            already_mat: bot.tier == TIER_MATERIALIZED,
                        });
                    } else if bot.mode == MODE_AUTO && !observers.is_empty() {
                        let mat_rad_sq = if bot.tier == TIER_MATERIALIZED {
                            mat_rad_stay_sq
                        } else {
                            mat_rad_enter_sq
                        };
                        if let Some((dist_sq, within_mat)) =
                            nearest_observer_dist(bot, &observers, mat_rad_sq)
                            && (within_mat || want_vis)
                        {
                            let (cx, cz, next_cx, next_cz) = bot_and_next_chunk(bot);
                            candidates.push(CandidateOut {
                                dist_sq,
                                bot_id: bot.bot_id,
                                within_mat_radius: within_mat,
                                already_mat: bot.tier == TIER_MATERIALIZED,
                                cx,
                                cz,
                                next_cx,
                                next_cz,
                            });
                        }
                    }
                }
                WorkerReply::StepIntents {
                    candidates,
                    ticket_needs,
                }
            }
            WorkerTask::StepBots {
                server_tick,
                total_bots,
                cfg,
                posture,
                observers,
                mat_allowed,
                vis_allowed,
                promote_allowed,
                emit_statuses,
                emit_render_bots,
                force_full_render,
            } => {
                let world = self.world.read().unwrap();
                let mut out = StepOutputs::default();
                out.stats.min_act_t = u32::MAX;
                if emit_render_bots && force_full_render {
                    out.render_bots.reserve(self.bots.len());
                }

                for bot in &mut self.bots {
                    let mut was_dirty = std::mem::take(&mut bot.dirty_render);
                    let was_active = !matches!(bot.phase, BotPhase::Idle);
                    if let Some(goal_air) = bot.pending_replan.take() {
                        materialize_lazy_state(bot, &world, &self.programs);
                        let p = solve_slot(
                            bot,
                            goal_air,
                            0,
                            &world,
                            &self.store,
                            &mut self.codec,
                            &self.programs,
                            &mut self.step_curves,
                        );
                        if emit_statuses || p.status == PLAN_STATUS_NEEDS_JOINTS {
                            out.plan_statuses.push(p);
                        }
                    }

                    let hooked = bot_wants_hooked(bot);
                    let canary = bot.mode == MODE_AUTO
                        && is_canary(bot.bot_index, total_bots, server_tick, &cfg);
                    let desired = if bot.mode == MODE_MATERIALIZED {
                        TIER_MATERIALIZED
                    } else if bot.mode == MODE_VIRTUAL {
                        TIER_VIRTUAL
                    } else if mat_allowed.contains(&bot.bot_id) {
                        TIER_MATERIALIZED
                    } else if vis_allowed.contains(&bot.bot_id) {
                        TIER_VISUAL
                    } else if hooked || canary {
                        TIER_MATERIALIZED
                    } else {
                        TIER_VIRTUAL
                    };

                    // Observer proximity and canaries require server chunks loaded + paced joins (max 1 non-hooked/tick).
                    let next_tier = if desired == TIER_MATERIALIZED && bot.mode != MODE_MATERIALIZED
                    {
                        let may_materialize = bot.tier == TIER_MATERIALIZED
                            || hooked
                            || promote_allowed.contains(&bot.bot_id);
                        let (cx, cz, next_cx, next_cz) = bot_and_next_chunk(bot);
                        if may_materialize
                            && world.is_server_loaded(cx, cz)
                            && world.is_server_loaded(next_cx, next_cz)
                        {
                            TIER_MATERIALIZED
                        } else if cfg.max_visual > 0
                            && !observers.is_empty()
                            && nearest_observer_dist(bot, &observers, 0.0).is_some()
                        {
                            TIER_VISUAL
                        } else if bot.tier == TIER_MATERIALIZED {
                            TIER_MATERIALIZED
                        } else {
                            bot.tier
                        }
                    } else {
                        desired
                    };

                    let prev_tier = bot.tier;
                    let promoted_to_mat =
                        prev_tier != TIER_MATERIALIZED && next_tier == TIER_MATERIALIZED;
                    if promoted_to_mat {
                        out.stats.promotions += 1;
                        if hooked {
                            out.stats.hooked_promotions += 1;
                        }
                    } else if prev_tier == TIER_MATERIALIZED && next_tier != TIER_MATERIALIZED {
                        out.stats.demotions += 1;
                    }
                    bot.tier = next_tier;

                    let stay_lazy = next_tier == TIER_VIRTUAL
                        && prev_tier == TIER_VIRTUAL
                        && bot.mode != MODE_MATERIALIZED
                        && bot.hooked_corridor.is_none();
                    if !stay_lazy {
                        materialize_lazy_state(bot, &world, &self.programs);
                    }

                    let prev_state = if next_tier != prev_tier {
                        Some(bot.state.clone())
                    } else {
                        None
                    };

                    step_bot_physics(
                        bot,
                        next_tier,
                        prev_tier,
                        prev_state.as_ref(),
                        hooked,
                        stay_lazy,
                        posture,
                        &world,
                        &self.programs,
                        &mut out,
                        &mut was_dirty,
                        emit_statuses,
                    );

                    match bot.tier {
                        TIER_VIRTUAL => out.stats.virt_count += 1,
                        TIER_VISUAL => out.stats.vis_count += 1,
                        TIER_MATERIALIZED => out.stats.mat_count += 1,
                        _ => {}
                    }
                    if bot.arrived {
                        out.stats.arrived_count += 1;
                        if bot.active_ticks == bot.predicted_ticks && bot.predicted_ticks > 0 {
                            out.stats.exact_t_count += 1;
                        }
                        if bot.active_ticks > 0 {
                            out.stats.min_act_t = out.stats.min_act_t.min(bot.active_ticks);
                            out.stats.max_act_t = out.stats.max_act_t.max(bot.active_ticks);
                        }
                    }
                    if bot.settled {
                        out.stats.settled_count += 1;
                    }
                    let is_now_active = !matches!(bot.phase, BotPhase::Idle);
                    let tier_changed = prev_tier != bot.tier;
                    if emit_render_bots
                        && (force_full_render || was_dirty || was_active || is_now_active || tier_changed)
                    {
                        let (pos, yaw, pitch, speed, on_ground, sprinting) = bot.approx_pose();
                        let speed_q = (speed * 200.0).clamp(0.0, 255.0).round() as u8;
                        let flags = (on_ground as u8)
                            | ((sprinting as u8) << 1)
                            | ((bot.state.crouching as u8) << 2)
                            | (((bot.tier == TIER_MATERIALIZED) as u8) << 3);
                        out.render_bots.push(RenderBotOut {
                            x: pos.x,
                            y: pos.y,
                            z: pos.z,
                            yaw,
                            pitch,
                            bot_index: (bot.bot_index & 0xFFFF) as u16,
                            speed_q,
                            flags,
                        });
                    }
                }
                WorkerReply::StepDone(out)
            }
        }
    }
}

fn spawn_worker(
    world: Arc<RwLock<WorldMirror>>,
    programs: Arc<Vec<words::Program>>,
) -> WorkerHandle {
    let (task_tx, task_rx) = mpsc::sync_channel::<WorkerTask>(4);
    let (rep_tx, rep_rx) = mpsc::sync_channel::<WorkerReply>(4);
    let join = std::thread::Builder::new()
        .name("mind-worker".into())
        .spawn(move || {
            let mut st = WorkerState {
                bots: Vec::new(),
                by_id: FxHashMap::default(),
                store: GeneratorStore::default(),
                codec: Interner::default(),
                step_curves: FxHashMap::default(),
                world,
                programs,
            };
            while let Ok(task) = task_rx.recv() {
                let rep = st.handle(task);
                if rep_tx.send(rep).is_err() {
                    break;
                }
            }
        })
        .expect("spawn mind worker");
    WorkerHandle {
        tx: task_tx,
        rx: rep_rx,
        join: Some(join),
    }
}

pub struct WorkerPool {
    workers: Vec<WorkerHandle>,
    bot_owner: FxHashMap<u32, usize>,
    world: Arc<RwLock<WorldMirror>>,
    next_rr: usize,
    spawn_cols: u32,
    spawn_spacing_x: f64,
    spawn_spacing_z: f64,
    shared_store: GeneratorStore,
    shared_codec: Interner,
    shared_world_prov: [u8; 32],
    reservations: ReservationTable,
}

impl WorkerPool {
    pub fn new(world: Arc<RwLock<WorldMirror>>, programs: Arc<Vec<words::Program>>) -> Self {
        let count = std::thread::available_parallelism()
            .map_or(4, |n| n.get())
            .min(12)
            .max(1);
        Self::with_workers(count, world, programs)
    }

    pub fn with_workers(
        count: usize,
        world: Arc<RwLock<WorldMirror>>,
        programs: Arc<Vec<words::Program>>,
    ) -> Self {
        let count = count.max(1);
        let workers = (0..count)
            .map(|_| spawn_worker(world.clone(), programs.clone()))
            .collect();
        Self {
            workers,
            bot_owner: FxHashMap::default(),
            world,
            next_rr: 0,
            spawn_cols: 1,
            spawn_spacing_x: 1.5,
            spawn_spacing_z: 1.5,
            shared_store: GeneratorStore::default(),
            shared_codec: Interner::default(),
            shared_world_prov: [0u8; 32],
            reservations: ReservationTable::default(),
        }
    }

    pub fn total_bots(&self) -> usize {
        self.bot_owner.len()
    }

    pub fn clear(&mut self) {
        self.bot_owner.clear();
        self.spawn_cols = 1;
        self.spawn_spacing_x = 1.5;
        self.spawn_spacing_z = 1.5;
        self.shared_store.clear_far();
        self.reservations.clear();
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::Clear);
        }
        for w in &self.workers {
            let _ = w.rx.recv();
        }
    }

    pub fn reset_stores(&mut self) {
        self.shared_store = GeneratorStore::default();
        self.shared_codec = Interner::default();
        self.shared_world_prov = [0u8; 32];
        self.reservations.clear();
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::ResetStores);
        }
        for w in &self.workers {
            let _ = w.rx.recv();
        }
    }

    pub fn spawn_one(&mut self, bot_id: u32, mode: u8, state: PlayerState) {
        let owner = if let Some(&existing) = self.bot_owner.get(&bot_id) {
            existing
        } else {
            let idx = self.next_rr % self.workers.len();
            self.next_rr += 1;
            self.bot_owner.insert(bot_id, idx);
            idx
        };
        let bot_index = (self.bot_owner.len().saturating_sub(1)) as u32;
        let _ = self.workers[owner].tx.send(WorkerTask::SpawnOne {
            bot_id,
            bot_index,
            mode,
            state,
        });
        let _ = self.workers[owner].rx.recv();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn spawn_batch(
        &mut self,
        count: u32,
        start_bot_id: u32,
        start_index: u32,
        mode: u8,
        cols: u32,
        base_x: f64,
        base_y: f64,
        base_z: f64,
        spacing_x: f64,
        spacing_z: f64,
    ) {
        if count == 0 {
            return;
        }
        let cols_eff = cols.max(1);
        self.spawn_cols = cols_eff;
        self.spawn_spacing_x = spacing_x;
        self.spawn_spacing_z = spacing_z;
        let mut chunks = FxHashSet::default();
        for i in 0..count {
            let x = base_x + (i % cols_eff) as f64 * spacing_x;
            let z = base_z + (i / cols_eff) as f64 * spacing_z;
            let cx = x.floor() as i32;
            let cz = z.floor() as i32;
            let cmin_x = (cx - 16).div_euclid(16);
            let cmax_x = (cx + 16).div_euclid(16);
            let cmin_z = (cz - 16).div_euclid(16);
            let cmax_z = (cz + 16).div_euclid(16);
            for sx in cmin_x..=cmax_x {
                for sz in cmin_z..=cmax_z {
                    chunks.insert([sx, sz]);
                }
            }
        }
        let chunk_list: Vec<[i32; 2]> = chunks.into_iter().collect();
        self.world.write().unwrap().ensure_chunks_from_disk(&chunk_list);

        let wcount = self.workers.len();
        self.bot_owner.reserve(count as usize);
        for i in 0..count as usize {
            let bot_id = start_bot_id + i as u32;
            let owner = i % wcount;
            self.bot_owner.insert(bot_id, owner);
        }

        let job = Arc::new(SpawnBatchJob {
            count,
            start_bot_id,
            start_index,
            mode,
            cols: cols_eff,
            base_x,
            base_y,
            base_z,
            spacing_x,
            spacing_z,
        });
        for (worker_idx, w) in self.workers.iter().enumerate() {
            let _ = w.tx.send(WorkerTask::SpawnBatch {
                worker_idx,
                worker_count: wcount,
                job: job.clone(),
            });
        }
        for w in &self.workers {
            let _ = w.rx.recv();
        }
    }

    pub fn remove_one(&mut self, bot_id: u32) {
        if let Some(owner) = self.bot_owner.remove(&bot_id) {
            let _ = self.workers[owner].tx.send(WorkerTask::RemoveOne { bot_id });
            let _ = self.workers[owner].rx.recv();
        }
    }

    pub fn self_sync(&mut self, bot_id: u32, state: PlayerState, reason: u32) {
        let owner = *self.bot_owner.entry(bot_id).or_insert_with(|| {
            let idx = self.next_rr % self.workers.len();
            self.next_rr += 1;
            idx
        });
        let _ = self.workers[owner].tx.send(WorkerTask::SelfSync {
            bot_id,
            state,
            reason,
        });
        let _ = self.workers[owner].rx.recv();
    }

    pub fn stop_one(&mut self, bot_id: u32) {
        if let Some(&owner) = self.bot_owner.get(&bot_id) {
            let _ = self.workers[owner].tx.send(WorkerTask::StopOne { bot_id });
            let _ = self.workers[owner].rx.recv();
        }
    }

    pub fn stop_all(&mut self) {
        self.reservations.clear();
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::StopAll);
        }
        for w in &self.workers {
            let _ = w.rx.recv();
        }
    }

    pub fn goal_one(&mut self, bot_id: u32, goal_air: BlockPos) -> Vec<PlanStatusOut> {
        let Some(&owner) = self.bot_owner.get(&bot_id) else {
            return vec![PlanStatusOut {
                bot_id,
                status: PLAN_STATUS_REFUSED,
                predicted_ticks: 0,
                solve_ns: 0,
                msg: "unknown bot".into(),
            }];
        };
        let _ = self.workers[owner].tx.send(WorkerTask::CollectGoalOneChunks {
            bot_id,
            goal_air,
        });
        if let Ok(WorkerReply::GoalOneChunks(chunks)) = self.workers[owner].rx.recv()
            && !chunks.is_empty()
        {
            self.world.write().unwrap().ensure_chunks_from_disk(&chunks);
        }

        let _ = self.workers[owner].tx.send(WorkerTask::GoalOne {
            bot_id,
            goal_air,
            emit_status: true,
        });
        match self.workers[owner].rx.recv() {
            Ok(WorkerReply::SolveDone(s, _)) => s.statuses,
            _ => Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn goal_batch(
        &mut self,
        gx: i32,
        gy: i32,
        gz: i32,
        grid_formation: bool,
    ) -> SolveSummary {
        self.goal_batch_ext(
            gx,
            gy,
            gz,
            if grid_formation { FORM_GRID } else { FORM_POINT },
            0,
            0,
        )
    }

    pub fn goal_batch_ext(
        &mut self,
        gx: i32,
        gy: i32,
        gz: i32,
        formation_mode: u8,
        param: i32,
        wave_ticks: u16,
    ) -> SolveSummary {
        let total = self.bot_owner.len().max(1) as u32;
        let spec = Arc::new(SolveGoalSpec {
            gx,
            gy,
            gz,
            formation_mode,
            param,
            wave_ticks,
            total_bots: total,
            cols: self.spawn_cols.max(1),
            spacing_x: self.spawn_spacing_x,
            spacing_z: self.spawn_spacing_z,
        });
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::CollectGoalChunks { spec: spec.clone() });
        }
        let mut needed = FxHashSet::default();
        let mut bot_min: Option<BlockPos> = None;
        let mut bot_max: Option<BlockPos> = None;
        let mut sample_state: Option<PlayerState> = None;
        let mut goal_min: Option<BlockPos> = None;
        let mut goal_max: Option<BlockPos> = None;
        let mut unique_goal: Option<BlockPos> = None;
        let mut multi_goal = false;
        let merge_box = |lo: &mut Option<BlockPos>, hi: &mut Option<BlockPos>, a: Option<BlockPos>, b: Option<BlockPos>| {
            if let Some(p) = a {
                *lo = Some(lo.map_or(p, |m| [m[0].min(p[0]), m[1].min(p[1]), m[2].min(p[2])]));
            }
            if let Some(p) = b {
                *hi = Some(hi.map_or(p, |m| [m[0].max(p[0]), m[1].max(p[1]), m[2].max(p[2])]));
            }
        };
        for w in &self.workers {
            if let Ok(WorkerReply::GoalBatchChunks(info)) = w.rx.recv() {
                needed.extend(info.chunks);
                merge_box(&mut bot_min, &mut bot_max, info.bot_min, info.bot_max);
                if sample_state.is_none() {
                    sample_state = info.sample_state;
                }
                merge_box(&mut goal_min, &mut goal_max, info.goal_min, info.goal_max);
                if info.multi_goal {
                    multi_goal = true;
                } else if !multi_goal && let Some(g) = info.unique_goal {
                    match unique_goal {
                        None => unique_goal = Some(g),
                        Some(prev) if prev != g => multi_goal = true,
                        _ => {}
                    }
                }
            }
        }
        let needed_vec: Vec<[i32; 2]> = needed.into_iter().collect();
        self.world.write().unwrap().ensure_chunks_from_disk(&needed_vec);

        let all_same_goal = if !multi_goal { unique_goal } else { None };
        let shared_topo_before = self.shared_store.topo_cache_builds();
        let shared_far_before = self.shared_store.far_field_builds();
        let shared_fields = if total >= 2
            && let (Some(mut sample_from), Some(bmin), Some(bmax), Some(gmin), Some(gmax)) =
                (sample_state, bot_min, bot_max, goal_min, goal_max)
        {
            let world = self.world.read().unwrap();
            let prov = world.provenance_version();
            if prov != self.shared_world_prov {
                self.shared_store.clear_far();
                self.shared_world_prov = prov;
            }
            let sp = [
                sample_from.position.x.floor() as i32,
                sample_from.position.y.floor() as i32,
                sample_from.position.z.floor() as i32,
            ];
            let sa = world.support_cell(&sample_from);
            if sample_from.main_supporting_block.is_none() && sample_from.on_ground && world.loaded(sa) && !world.is_air(sa) {
                sample_from.main_supporting_block = Some(sa);
            }
            let sample_goal_air = snap_airborne_goal(&world, all_same_goal.unwrap_or(gmin));
            let sample_goal_solid = [sample_goal_air[0], sample_goal_air[1] - 1, sample_goal_air[2]];
            let cmin = [
                bmin[0].min(gmin[0]) - 2,
                sa[1],
                bmin[2].min(gmin[2]) - 2,
            ];
            let cmax = [
                bmax[0].max(gmax[0]) + 2,
                (bmax[1].max(gmax[1]) + 4).min(world.max_loaded_y()),
                bmax[2].max(gmax[2]) + 2,
            ];
            let flat_clear = world
                .entry_label(&mut self.shared_codec, &sample_from)
                .ok()
                .is_some_and(|label| {
                    matches!(
                        world.corridor_query(&mut self.shared_codec, label, cmin, cmax),
                        CorridorResult::Clear(_)
                    )
                });
            let _ = sp;
            if !flat_clear {
                if let Some(goal_air) = all_same_goal {
                    let _ = sample_goal_solid;
                    let snapped = snap_airborne_goal(&world, goal_air);
                    let gsolid = [snapped[0], snapped[1] - 1, snapped[2]];
                    self.shared_store.prewarm_batch_fields(
                        &*world,
                        &sample_from,
                        Goal::of(gsolid),
                        bmin,
                        bmax,
                        &mut self.shared_codec,
                    );
                } else {
                    self.shared_store.warm_far_sections_in(
                        prov,
                        &*world,
                        [cmin[0] - 16, world.min_loaded_y(), cmin[2] - 16],
                        [cmax[0] + 16, cmax[1] + 16, cmax[2] + 16],
                        256,
                    );
                }
            }
            let snap = self.shared_store.export_shared_fields();
            if snap.is_empty() {
                None
            } else {
                Some(Arc::new(snap))
            }
        } else {
            None
        };

        let collect_footprints = total <= 2048
            || all_same_goal.is_some()
            || shared_fields.is_some()
            || spec.spacing_x < 1.0
            || spec.spacing_z < 1.0;

        let emit_statuses = self.bot_owner.len() <= 64;
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::SolveBatch {
                spec: spec.clone(),
                emit_statuses,
                shared_fields: shared_fields.clone(),
                collect_footprints,
            });
        }

        let mut merged = SolveSummary::default();
        merged.min_pred_t = u32::MAX;
        let mut all_footprints = Vec::new();
        for w in &self.workers {
            if let Ok(WorkerReply::SolveDone(mut part, mut fps)) = w.rx.recv() {
                merged.ready += part.ready;
                merged.already += part.already;
                merged.joints += part.joints;
                merged.refused += part.refused;
                merged.worker_topo_builds += part.worker_topo_builds;
                merged.worker_far_builds += part.worker_far_builds;
                if part.min_pred_t > 0 {
                    merged.min_pred_t = merged.min_pred_t.min(part.min_pred_t);
                }
                merged.max_pred_t = merged.max_pred_t.max(part.max_pred_t);
                merged.solve_ns_samples.append(&mut part.solve_ns_samples);
                if merged.sample_msg.is_empty() && !part.sample_msg.is_empty() {
                    merged.sample_msg = part.sample_msg;
                }
                merged.statuses.append(&mut part.statuses);
                all_footprints.append(&mut fps);
            }
        }
        if merged.min_pred_t == u32::MAX {
            merged.min_pred_t = 0;
        }
        merged.shared_topo_builds = self.shared_store.topo_cache_builds().saturating_sub(shared_topo_before);
        merged.shared_far_builds = self.shared_store.far_field_builds().saturating_sub(shared_far_before);

        if !all_footprints.is_empty() {
            let (extra_delays, stats) = self.reservations.resolve_batch(&mut all_footprints);
            merged.reserve_stats = stats;
            if !extra_delays.is_empty() {
                let mut per_worker: Vec<Vec<(u32, u16)>> = vec![Vec::new(); self.workers.len()];
                for (bot_id, extra) in extra_delays {
                    if let Some(&owner) = self.bot_owner.get(&bot_id) {
                        per_worker[owner].push((bot_id, extra));
                    }
                }
                let mut awaited = Vec::new();
                for (wi, delays) in per_worker.into_iter().enumerate() {
                    if !delays.is_empty() {
                        let _ = self.workers[wi].tx.send(WorkerTask::ApplyDelays { delays });
                        awaited.push(wi);
                    }
                }
                for wi in awaited {
                    let _ = self.workers[wi].rx.recv();
                }
            }
        } else {
            self.reservations.clear();
        }
        merged
    }

    pub fn step_tick(
        &mut self,
        server_tick: u64,
        cfg: SwarmConfig,
        posture: SwarmPosture,
        observers: &[Observer],
        ticket_last_sent: &mut FxHashMap<([i32; 2], u8), u64>,
        emit_render_bots: bool,
        force_full_render: bool,
    ) -> (StepOutputs, Vec<([i32; 2], u8)>) {
        let total_bots = self.bot_owner.len();
        let obs_arc = Arc::new(observers.to_vec());

        // Pass 1: Collect observer proximity candidates & chunk ticket needs for promoted bots.
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::CollectStepIntents {
                server_tick,
                total_bots,
                cfg,
                observers: obs_arc.clone(),
            });
        }
        let mut all_candidates = Vec::new();
        let mut all_needs = Vec::new();
        for w in &self.workers {
            if let Ok(WorkerReply::StepIntents {
                mut candidates,
                mut ticket_needs,
            }) = w.rx.recv()
            {
                all_candidates.append(&mut candidates);
                all_needs.append(&mut ticket_needs);
            }
        }

        all_candidates.sort_by(|a, b| a.dist_sq.total_cmp(&b.dist_sq));
        let mut mat_allowed = FxHashSet::default();
        let mut vis_allowed = FxHashSet::default();

        let forced_mat_count = all_needs.len();
        let mut mat_left = (cfg.max_materialized as usize).saturating_sub(forced_mat_count);
        let mut vis_left = cfg.max_visual as usize;

        for c in &all_candidates {
            if c.within_mat_radius && mat_left > 0 {
                mat_allowed.insert(c.bot_id);
                mat_left -= 1;
            } else if vis_left > 0 {
                vis_allowed.insert(c.bot_id);
                vis_left -= 1;
            }
        }

        // Check server-loaded chunks, queue missing tickets, and pace non-hooked promotions (<= 1/tick).
        const MAX_PACED_PROMOTES_PER_TICK: usize = 1;
        let mut promote_allowed = FxHashSet::default();
        let mut paced_left = MAX_PACED_PROMOTES_PER_TICK;
        let mut tickets_to_send = Vec::new();
        {
            let world = self.world.read().unwrap();
            let mut maybe_ticket = |cx: i32, cz: i32, level: u8| {
                let key = ([cx, cz], level);
                let due = ticket_last_sent
                    .get(&key)
                    .is_none_or(|&prev| server_tick.saturating_sub(prev) >= 15);
                if due {
                    ticket_last_sent.insert(key, server_tick);
                    tickets_to_send.push(([cx, cz], level));
                }
            };
            for need in &all_needs {
                let loaded_cur = world.is_server_loaded(need.cx, need.cz);
                let loaded_next = world.is_server_loaded(need.next_cx, need.next_cz);
                maybe_ticket(need.cx, need.cz, need.level);
                if need.next_cx != need.cx || need.next_cz != need.cz {
                    maybe_ticket(need.next_cx, need.next_cz, need.level);
                }
                if !need.already_mat && !need.hooked && loaded_cur && loaded_next && paced_left > 0 {
                    if promote_allowed.insert(need.bot_id) {
                        paced_left -= 1;
                    }
                }
            }
            for c in &all_candidates {
                if mat_allowed.contains(&c.bot_id) {
                    let loaded_cur = world.is_server_loaded(c.cx, c.cz);
                    let loaded_next = world.is_server_loaded(c.next_cx, c.next_cz);
                    maybe_ticket(c.cx, c.cz, 33);
                    if c.next_cx != c.cx || c.next_cz != c.cz {
                        maybe_ticket(c.next_cx, c.next_cz, 33);
                    }
                    if !c.already_mat && loaded_cur && loaded_next && paced_left > 0 {
                        if promote_allowed.insert(c.bot_id) {
                            paced_left -= 1;
                        }
                    }
                }
            }
        }
        let _ = &obs_arc;

        let mat_arc = Arc::new(mat_allowed);
        let vis_arc = Arc::new(vis_allowed);
        let promote_arc = Arc::new(promote_allowed);
        let emit_statuses = total_bots <= 64;

        // Pass 2: Step all bots in parallel and collect compact e2s outputs.
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::StepBots {
                server_tick,
                total_bots,
                cfg,
                posture,
                observers: obs_arc.clone(),
                mat_allowed: mat_arc.clone(),
                vis_allowed: vis_arc.clone(),
                promote_allowed: promote_arc.clone(),
                emit_statuses,
                emit_render_bots,
                force_full_render,
            });
        }

        let mut total = StepOutputs::default();
        total.stats.min_act_t = u32::MAX;
        if emit_render_bots && force_full_render {
            total.render_bots.reserve(total_bots);
        }
        for w in &self.workers {
            if let Ok(WorkerReply::StepDone(mut part)) = w.rx.recv() {
                total.plan_statuses.append(&mut part.plan_statuses);
                total.tier_changes.append(&mut part.tier_changes);
                total.tick_cmds.append(&mut part.tick_cmds);
                total.visuals.append(&mut part.visuals);
                total.render_bots.append(&mut part.render_bots);
                total.stats.active_bots += part.stats.active_bots;
                total.stats.virt_count += part.stats.virt_count;
                total.stats.vis_count += part.stats.vis_count;
                total.stats.mat_count += part.stats.mat_count;
                total.stats.arrived_count += part.stats.arrived_count;
                total.stats.settled_count += part.stats.settled_count;
                total.stats.exact_t_count += part.stats.exact_t_count;
                total.stats.min_act_t = total.stats.min_act_t.min(part.stats.min_act_t);
                total.stats.max_act_t = total.stats.max_act_t.max(part.stats.max_act_t);
                total.stats.promotions += part.stats.promotions;
                total.stats.demotions += part.stats.demotions;
                total.stats.hooked_promotions += part.stats.hooked_promotions;
            }
        }
        if total.stats.min_act_t == u32::MAX {
            total.stats.min_act_t = 0;
        }

        // Also issue tickets for any newly requested mat_allowed bots whose chunks weren't server-loaded.
        if !mat_arc.is_empty() {
            let world = self.world.read().unwrap();
            for tc in &total.tier_changes {
                if tc.new_tier == TIER_MATERIALIZED {
                    let cx = tc.state.position.x.floor() as i32 >> 4;
                    let cz = tc.state.position.z.floor() as i32 >> 4;
                    if !world.is_server_loaded(cx, cz) {
                        let key = ([cx, cz], 33u8);
                        if ticket_last_sent
                            .get(&key)
                            .is_none_or(|&prev| server_tick.saturating_sub(prev) >= 15)
                        {
                            ticket_last_sent.insert(key, server_tick);
                            tickets_to_send.push(([cx, cz], 33u8));
                        }
                    }
                }
            }
        }

        (total, tickets_to_send)
    }

    pub fn replan_in_corridor(&mut self, pos: BlockPos) {
        self.shared_store.invalidate_far_section([
            pos[0].div_euclid(16),
            pos[1].div_euclid(16),
            pos[2].div_euclid(16),
        ]);
        for w in &self.workers {
            let _ = w.tx.send(WorkerTask::MarkReplanAt { pos });
        }
        for w in &self.workers {
            let _ = w.rx.recv();
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        let handles: Vec<WorkerHandle> = std::mem::take(&mut self.workers);
        for mut w in handles {
            drop(w.tx);
            if let Some(j) = w.join.take() {
                let _ = j.join();
            }
        }
    }
}

fn goal_for_bot(bot: &BotSlot, spec: &SolveGoalSpec) -> BlockPos {
    let cols = spec.cols.max(1);
    let total = spec.total_bots.max(1);
    let rows = total.div_ceil(cols).max(1);
    let col = bot.bot_index % cols;
    let row = bot.bot_index / cols;
    match spec.formation_mode {
        FORM_GRID => [
            spec.gx + bot.rel_x.round() as i32,
            spec.gy,
            spec.gz + bot.rel_z.round() as i32,
        ],
        FORM_WEDGE => {
            let half_c = (f64::from(cols) - 1.0) * 0.5;
            let wedge_back = ((f64::from(col) - half_c).abs() * 0.65).round() as i32;
            [
                spec.gx + bot.rel_x.round() as i32 - wedge_back,
                spec.gy,
                spec.gz + bot.rel_z.round() as i32,
            ]
        }
        FORM_QUAD => {
            let gap = spec.param.max(6);
            let sx = if col < cols / 2 { -gap } else { gap };
            let sz = if row < rows / 2 { -gap } else { gap };
            [
                spec.gx + bot.rel_x.round() as i32 + sx,
                spec.gy,
                spec.gz + bot.rel_z.round() as i32 + sz,
            ]
        }
        FORM_CITADEL => {
            let spx = spec.spacing_x.max(0.5);
            let spz = spec.spacing_z.max(0.5);
            let cx = spec.gx + (f64::from(cols.saturating_sub(1)) * spx * 0.5).round() as i32;
            let cz = spec.gz + (f64::from(rows.saturating_sub(1)) * spz * 0.5).round() as i32;
            let side = bot.bot_index % 4;
            let k = bot.bot_index / 4;
            let per_side = total.div_ceil(4).max(1);
            let thickness = (f64::from(per_side).sqrt() * 0.4).round().clamp(4.0, 20.0) as u32;
            let len = per_side.div_ceil(thickness).max(1);
            let along = k % len;
            let depth = k / len;
            let half_span = (f64::from(len) * spx * 0.5).round() as i32;
            let pad = spec.param.max(0);
            let along_x = (f64::from(along) * spx).round() as i32;
            let along_z = (f64::from(along) * spz).round() as i32;
            let depth_x = (f64::from(depth) * spx).round() as i32;
            let depth_z = (f64::from(depth) * spz).round() as i32;
            match side {
                0 => [cx - half_span + along_x, spec.gy, cz - half_span - pad - depth_z],
                1 => [cx - half_span + along_x, spec.gy, cz + half_span + pad + depth_z],
                2 => [cx - half_span - pad - depth_x, spec.gy, cz - half_span + along_z],
                _ => [cx + half_span + pad + depth_x, spec.gy, cz - half_span + along_z],
            }
        }
        FORM_CORRIDOR => {
            let gap = spec.param.max(8);
            let sz = if row < rows / 2 { -gap } else { gap };
            [
                spec.gx + bot.rel_x.round() as i32,
                spec.gy,
                spec.gz + bot.rel_z.round() as i32 + sz,
            ]
        }
        _ => [spec.gx, spec.gy, spec.gz],
    }
}

pub fn corridor_bounds(
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

fn is_canary(bot_index: u32, total_bots: usize, server_tick: u64, cfg: &SwarmConfig) -> bool {
    if cfg.canary_count == 0 || total_bots == 0 {
        return false;
    }
    let count = (cfg.canary_count as usize).min(total_bots);
    let period = (cfg.canary_period as u64).max(1);
    let stride = (total_bots / count).max(1);
    let idx = bot_index as usize;
    if total_bots < 2 * count {
        let phase = ((server_tick / period) as usize) % total_bots;
        let shifted = (idx + total_bots - phase) % total_bots;
        return (shifted % stride == 0) && (shifted / stride < count);
    }
    for slot in 0..count {
        let offset = (slot as u64) * period / (count as u64);
        let rot = ((server_tick + offset) / period) as usize;
        let target = (slot * stride + rot) % total_bots;
        if idx == target {
            return true;
        }
    }
    false
}

fn insert_corridor_chunks(chunks: &mut FxHashSet<[i32; 2]>, pos: Vec3, goal_air: BlockPos) {
    let sx0 = (pos.x.floor() as i32)
        .min(goal_air[0])
        .saturating_sub(8)
        .div_euclid(16);
    let sx1 = (pos.x.floor() as i32)
        .max(goal_air[0])
        .saturating_add(8)
        .div_euclid(16);
    let sz0 = (pos.z.floor() as i32)
        .min(goal_air[2])
        .saturating_sub(8)
        .div_euclid(16);
    let sz1 = (pos.z.floor() as i32)
        .max(goal_air[2])
        .saturating_add(8)
        .div_euclid(16);
    for sz in sz0..=sz1 {
        for sx in sx0..=sx1 {
            chunks.insert([sx, sz]);
        }
    }
}

fn bot_and_next_chunk(bot: &BotSlot) -> (i32, i32, i32, i32) {
    let (pos, _, _, _, _, _) = bot.approx_pose();
    let bx = pos.x.floor() as i32;
    let bz = pos.z.floor() as i32;
    let cx = bx.div_euclid(16);
    let cz = bz.div_euclid(16);
    let (dx, dz) = match &bot.phase {
        BotPhase::Executing { goal_air, .. } => {
            ((goal_air[0] - bx).signum() * 16, (goal_air[2] - bz).signum() * 16)
        }
        _ => (
            (bot.state.velocity.x * 16.0).round() as i32,
            (bot.state.velocity.z * 16.0).round() as i32,
        ),
    };
    let next_cx = (bx + dx).div_euclid(16);
    let next_cz = (bz + dz).div_euclid(16);
    (cx, cz, next_cx, next_cz)
}

fn bot_wants_hooked(bot: &BotSlot) -> bool {
    if matches!(bot.phase, BotPhase::Idle) {
        return false;
    }
    let Some((hlo, hhi)) = bot.hooked_corridor else {
        return false;
    };
    let pos = bot.approx_pose().0;
    let bx = pos.x;
    let by = pos.y;
    let bz = pos.z;
    let dx = if bx < f64::from(hlo[0]) {
        f64::from(hlo[0]) - bx
    } else if bx > f64::from(hhi[0] + 1) {
        bx - f64::from(hhi[0] + 1)
    } else {
        0.0
    };
    let dy = if by < f64::from(hlo[1]) {
        f64::from(hlo[1]) - by
    } else if by > f64::from(hhi[1] + 2) {
        by - f64::from(hhi[1] + 2)
    } else {
        0.0
    };
    let dz = if bz < f64::from(hlo[2]) {
        f64::from(hlo[2]) - bz
    } else if bz > f64::from(hhi[2] + 1) {
        bz - f64::from(hhi[2] + 1)
    } else {
        0.0
    };
    dx * dx + dy * dy + dz * dz <= 24.0 * 24.0
}

fn nearest_observer_dist(bot: &BotSlot, observers: &[Observer], mat_rad_sq: f64) -> Option<(f64, bool)> {
    let pos = bot.approx_pose().0;
    let mut best = f64::INFINITY;
    for obs in observers {
        let dx = pos.x - obs.x;
        let dy = pos.y - obs.y;
        let dz = pos.z - obs.z;
        let d2 = dx * dx + dy * dy + dz * dz;
        let vd = f64::from(obs.view_dist);
        if d2 <= vd * vd && d2 < best {
            best = d2;
        }
    }
    if best.is_finite() {
        Some((best, best <= mat_rad_sq))
    } else {
        None
    }
}

fn materialize_lazy_state(bot: &mut BotSlot, world: &WorldMirror, programs: &[words::Program]) {
    if let BotPhase::Executing {
        steps,
        idx,
        elapsed,
        lazy_ticks,
        step_start_state,
        ..
    } = &mut bot.phase
        && *lazy_ticks > 0
    {
        *lazy_ticks = 0;
        if let Some(step) = steps.get(*idx) {
            let mut s = step_start_state.clone();
            let target_t = (*elapsed).min(step.ticks);
            for t in 0..target_t {
                let sel = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    programs[step.program as usize].select(world, &s, t, step.args)
                }));
                let Ok(Some((_, input))) = sel else {
                    break;
                };
                let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    law::try_tick(&mut s, world, &input)
                }));
                if !matches!(ok, Ok(Ok(()))) {
                    break;
                }
            }
            bot.state = s;
        }
    }
}

fn find_hooked_box(world: &WorldMirror, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
    let first = world.corridor_has_hooked(lo, hi)?;
    // Bound box closely around hooked cell(s) within +/-16 of first hooked cell (or full corridor).
    let mut min_p = first;
    let mut max_p = first;
    let sx0 = first[0].max(lo[0]);
    let sx1 = (first[0] + 32).min(hi[0]);
    let sy0 = lo[1];
    let sy1 = hi[1];
    let sz0 = (first[2] - 16).max(lo[2]);
    let sz1 = (first[2] + 16).min(hi[2]);
    for z in sz0..=sz1 {
        for y in sy0..=sy1 {
            for x in sx0..=sx1 {
                if world.loaded([x, y, z])
                    && let Ok(b) = world.cell_block([x, y, z])
                    && crate::world_mirror::is_hooked_block(b)
                {
                    for a in 0..3 {
                        min_p[a] = min_p[a].min([x, y, z][a]);
                        max_p[a] = max_p[a].max([x, y, z][a]);
                    }
                }
            }
        }
    }
    Some((min_p, max_p))
}

fn snap_airborne_goal(world: &WorldMirror, goal_air: BlockPos) -> BlockPos {
    let below = [goal_air[0], goal_air[1] - 1, goal_air[2]];
    if !world.loaded(below) || !world.is_air(below) {
        return goal_air;
    }
    let min_y = world.min_loaded_y();
    for d in 1..=128 {
        let y = below[1] - d;
        if y < min_y {
            break;
        }
        let solid = [goal_air[0], y, goal_air[2]];
        let air1 = [goal_air[0], y + 1, goal_air[2]];
        let air2 = [goal_air[0], y + 2, goal_air[2]];
        if world.loaded(solid)
            && !world.is_air(solid)
            && world.loaded(air1)
            && world.is_air(air1)
            && world.loaded(air2)
            && world.is_air(air2)
        {
            return air1;
        }
    }
    goal_air
}

fn sample_flat_step_curve(
    step_start: &PlayerState,
    step: &plan::WordStep,
    world: &WorldMirror,
    programs: &[words::Program],
    cache: &mut FxHashMap<StepCurveKey, Arc<Vec<StepPose>>>,
) -> Option<Arc<Vec<StepPose>>> {
    if step.ticks == 0
        || step.ticks > 2000
        || (step.end.position.y - step_start.position.y).abs() >= 1e-6
    {
        return None;
    }
    let exp_dx = step.end.position.x - step_start.position.x;
    let exp_dy = step.end.position.y - step_start.position.y;
    let exp_dz = step.end.position.z - step_start.position.z;
    let key = StepCurveKey {
        program: step.program,
        ticks: step.ticks,
        yaw_index: step.args.yaw_index,
        refined_yaw_bits: step.args.refined_yaw.map_or(u32::MAX, f32::to_bits),
        init_yaw_bits: step_start.yaw.to_bits(),
        vx_bits: step_start.velocity.x.to_bits(),
        vy_bits: step_start.velocity.y.to_bits(),
        vz_bits: step_start.velocity.z.to_bits(),
        on_ground: step_start.on_ground,
        sprinting: step_start.sprinting,
        no_jump_delay: step_start.no_jump_delay.min(255) as u8,
    };
    if let Some(arc) = cache.get(&key)
        && let Some(last) = arc.last()
        && (f64::from(last.dx) - exp_dx).abs() < 1e-3
        && (f64::from(last.dy) - exp_dy).abs() < 1e-3
        && (f64::from(last.dz) - exp_dz).abs() < 1e-3
    {
        return Some(arc.clone());
    }
    if cache.len() >= 512 {
        return None;
    }
    let start = step_start.position;
    let mut s = step_start.clone();
    let mut poses = Vec::with_capacity(step.ticks as usize);
    for t in 0..step.ticks {
        let sel = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            programs[step.program as usize].select(world, &s, t, step.args)
        }));
        let Ok(Some((_, input))) = sel else {
            return None;
        };
        let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            law::try_tick(&mut s, world, &input)
        }));
        if !matches!(ok, Ok(Ok(()))) {
            return None;
        }
        poses.push(StepPose {
            dx: (s.position.x - start.x) as f32,
            dy: (s.position.y - start.y) as f32,
            dz: (s.position.z - start.z) as f32,
            yaw: s.yaw,
            speed: s.velocity.x.hypot(s.velocity.z) as f32,
            on_ground: s.on_ground,
            sprinting: s.sprinting,
        });
    }
    let last = poses.last()?;
    if (f64::from(last.dx) - exp_dx).abs() >= 1e-3
        || (f64::from(last.dy) - exp_dy).abs() >= 1e-3
        || (f64::from(last.dz) - exp_dz).abs() >= 1e-3
    {
        return None;
    }
    let arc = Arc::new(poses);
    cache.insert(key, arc.clone());
    Some(arc)
}

#[allow(clippy::too_many_arguments)]
fn solve_slot(
    bot: &mut BotSlot,
    goal_air: BlockPos,
    delay_ticks: u16,
    world: &WorldMirror,
    store: &GeneratorStore,
    codec: &mut Interner,
    programs: &[words::Program],
    step_curves: &mut FxHashMap<StepCurveKey, Arc<Vec<StepPose>>>,
) -> PlanStatusOut {
    let bot_id = bot.bot_id;
    let goal_air = snap_airborne_goal(world, goal_air);
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
        return PlanStatusOut {
            bot_id,
            status: PLAN_STATUS_REFUSED,
            predicted_ticks: 0,
            solve_ns: 0,
            msg: "no ground under bot within 8 blocks".into(),
        };
    };
    if bot.state.main_supporting_block.is_none() && bot.state.on_ground {
        bot.state.main_supporting_block = Some(a);
    }

    let (cl, ch) = corridor_bounds(p, a, goal_solid, world.min_loaded_y(), world.max_loaded_y());
    if let Err(why) = world.check_corridor_valid(cl, ch) {
        bot.phase = BotPhase::Idle;
        return PlanStatusOut {
            bot_id,
            status: PLAN_STATUS_REFUSED,
            predicted_ticks: 0,
            solve_ns: 0,
            msg: why,
        };
    }

    let t0 = Instant::now();
    let solved = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        plan::solve(
            &bot.state,
            Goal::of(goal_solid),
            Kit::BareLegs,
            world,
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
            return PlanStatusOut {
                bot_id,
                status: PLAN_STATUS_REFUSED,
                predicted_ticks: 0,
                solve_ns,
                msg,
            };
        }
    };

    bot.active_ticks = 0;
    bot.arrived = false;
    bot.settled = false;
    bot.hooked_corridor = None;

    match result.outcome {
        PlanOutcome::Ready(found) => {
            if found.ticks == 0 || found.words.is_empty() {
                bot.phase = BotPhase::Idle;
                bot.predicted_ticks = 0;
                bot.arrived = true;
                bot.settled = true;
                PlanStatusOut {
                    bot_id,
                    status: PLAN_STATUS_ALREADY_THERE,
                    predicted_ticks: 0,
                    solve_ns,
                    msg: "Already there.".into(),
                }
            } else {
                let mut cur_state = bot.state.clone();
                let mut steps = Vec::with_capacity(found.words.len());
                for step in &found.words {
                    let curve = sample_flat_step_curve(
                        &cur_state,
                        step,
                        world,
                        programs,
                        step_curves,
                    );
                    steps.push(CertifiedStep {
                        program: step.program,
                        args: step.args,
                        ticks: step.ticks,
                        start_pos: cur_state.position,
                        end: step.end.clone(),
                        curve,
                    });
                    cur_state = step.end.materialize();
                }
                let ticks = found.ticks.min(u32::MAX as u64) as u32;
                bot.predicted_ticks = ticks;
                bot.hooked_corridor = find_hooked_box(world, cl, ch);
                bot.dirty_render = true;
                bot.phase = BotPhase::Executing {
                    steps,
                    idx: 0,
                    elapsed: 0,
                    lazy_ticks: 0,
                    delay_ticks,
                    step_start_state: bot.state.clone(),
                    goal_air,
                    corridor: (cl, ch),
                };
                PlanStatusOut {
                    bot_id,
                    status: PLAN_STATUS_READY,
                    predicted_ticks: ticks,
                    solve_ns,
                    msg: String::new(),
                }
            }
        }
        PlanOutcome::NeedsJoints { exception, .. } => {
            bot.phase = BotPhase::Idle;
            PlanStatusOut {
                bot_id,
                status: PLAN_STATUS_NEEDS_JOINTS,
                predicted_ticks: 0,
                solve_ns,
                msg: format!("NeedsJoints at {exception:?}"),
            }
        }
        PlanOutcome::Prefix(_) => unreachable!("plan::solve chains prefixes into Ready"),
        PlanOutcome::Unknown(refusal) => {
            bot.phase = BotPhase::Idle;
            let why = result
                .explanation
                .unwrap_or_else(|| format!("{refusal:?}"));
            PlanStatusOut {
                bot_id,
                status: PLAN_STATUS_REFUSED,
                predicted_ticks: 0,
                solve_ns,
                msg: why,
            }
        }
    }
}

fn record_solve(sum: &mut SolveSummary, out: PlanStatusOut, emit_status: bool) {
    match out.status {
        PLAN_STATUS_READY => {
            sum.ready += 1;
            if out.predicted_ticks > 0 {
                sum.min_pred_t = if sum.min_pred_t == 0 {
                    out.predicted_ticks
                } else {
                    sum.min_pred_t.min(out.predicted_ticks)
                };
                sum.max_pred_t = sum.max_pred_t.max(out.predicted_ticks);
            }
        }
        PLAN_STATUS_ALREADY_THERE => sum.already += 1,
        PLAN_STATUS_NEEDS_JOINTS => {
            sum.joints += 1;
            if sum.sample_msg.is_empty() {
                sum.sample_msg = out.msg.clone();
            }
        }
        _ => {
            sum.refused += 1;
            if sum.sample_msg.is_empty() {
                sum.sample_msg = out.msg.clone();
            }
        }
    }
    if out.solve_ns > 0 {
        sum.solve_ns_samples.push(out.solve_ns);
    }
    if emit_status {
        sum.statuses.push(out);
    }
}

#[allow(clippy::too_many_arguments)]
fn step_bot_physics(
    bot: &mut BotSlot,
    next_tier: u8,
    prev_tier: u8,
    prev_state: Option<&PlayerState>,
    hooked: bool,
    stay_lazy: bool,
    posture: SwarmPosture,
    world: &WorldMirror,
    programs: &[words::Program],
    out: &mut StepOutputs,
    pose_changed: &mut bool,
    emit_statuses: bool,
) {
    let bot_id = bot.bot_id;
    let tier_changed = next_tier != prev_tier;
    if tier_changed && next_tier != TIER_MATERIALIZED {
        // Demoting away from Materialized or changing Virtual <-> Visual: emit before/after cleanly.
        let s = prev_state.unwrap_or(&bot.state);
        out.tier_changes.push(TierChangeOut {
            bot_id,
            bot_index: bot.bot_index,
            new_tier: next_tier,
            flags: state_tier_flags(s, hooked),
            state: s.clone(),
        });
    } else if tier_changed && next_tier == TIER_MATERIALIZED {
        // CRITICAL: emit TIER_MATERIALIZED with prev_state BEFORE running this tick's try_tick!
        let s = prev_state.unwrap_or(&bot.state);
        out.tier_changes.push(TierChangeOut {
            bot_id,
            bot_index: bot.bot_index,
            new_tier: TIER_MATERIALIZED,
            flags: state_tier_flags(s, hooked),
            state: s.clone(),
        });
    }

    let phase = std::mem::replace(&mut bot.phase, BotPhase::Idle);
    match phase {
        BotPhase::Idle => {
            if (posture.flags & POSTURE_STARE) != 0 {
                let dx = posture.tx - bot.state.position.x;
                let dy = posture.ty - (bot.state.position.y + 1.62);
                let dz = posture.tz - bot.state.position.z;
                let horiz = dx.hypot(dz);
                let new_yaw = if horiz > 1e-6 {
                    (-dx).atan2(dz).to_degrees() as f32
                } else {
                    bot.state.yaw
                };
                let new_pitch = (-dy.atan2(horiz.max(1e-6)).to_degrees()).clamp(-85.0, 85.0) as f32;
                let want_crouch = (posture.flags & POSTURE_SALUTE) != 0;
                let mut yaw_diff = (new_yaw - bot.state.yaw).rem_euclid(360.0);
                if yaw_diff > 180.0 {
                    yaw_diff = 360.0 - yaw_diff;
                }
                let changed = yaw_diff > 0.5
                    || (new_pitch - bot.state.pitch).abs() > 0.5
                    || bot.state.crouching != want_crouch;
                if changed {
                    bot.state.yaw = new_yaw;
                    bot.state.pitch = new_pitch;
                    bot.state.crouching = want_crouch;
                    *pose_changed = true;
                }
                if next_tier == TIER_MATERIALIZED && changed {
                    let mut flags = 0u32;
                    if bot.state.on_ground {
                        flags |= FLAG_ON_GROUND;
                    }
                    out.tick_cmds.push(TickCmdOut {
                        bot_id,
                        keys: if want_crouch { 32 } else { 0 },
                        flags,
                        x: bot.state.position.x,
                        y: bot.state.position.y,
                        z: bot.state.position.z,
                        yaw: bot.state.yaw,
                        pitch: bot.state.pitch,
                    });
                }
            } else if bot.state.crouching {
                bot.state.crouching = false;
                bot.state.pitch = 0.0;
                *pose_changed = true;
                if next_tier == TIER_MATERIALIZED {
                    let mut flags = 0u32;
                    if bot.state.on_ground {
                        flags |= FLAG_ON_GROUND;
                    }
                    out.tick_cmds.push(TickCmdOut {
                        bot_id,
                        keys: 0,
                        flags,
                        x: bot.state.position.x,
                        y: bot.state.position.y,
                        z: bot.state.position.z,
                        yaw: bot.state.yaw,
                        pitch: bot.state.pitch,
                    });
                }
            }
            if next_tier == TIER_VISUAL {
                out.visuals.push(visual_from_state(bot_id, &bot.state));
            }
        }
        BotPhase::Executing {
            steps,
            mut idx,
            mut elapsed,
            mut lazy_ticks,
            mut delay_ticks,
            mut step_start_state,
            goal_air,
            corridor,
        } => {
            out.stats.active_bots += 1;
            if delay_ticks > 0 {
                delay_ticks -= 1;
                bot.phase = BotPhase::Executing {
                    steps,
                    idx,
                    elapsed,
                    lazy_ticks,
                    delay_ticks,
                    step_start_state,
                    goal_air,
                    corridor,
                };
                if next_tier == TIER_MATERIALIZED {
                    let mut flags = 0u32;
                    if bot.state.on_ground {
                        flags |= FLAG_ON_GROUND;
                    }
                    out.tick_cmds.push(TickCmdOut {
                        bot_id,
                        keys: 0,
                        flags,
                        x: bot.state.position.x,
                        y: bot.state.position.y,
                        z: bot.state.position.z,
                        yaw: bot.state.yaw,
                        pitch: bot.state.pitch,
                    });
                } else if next_tier == TIER_VISUAL {
                    out.visuals.push(visual_from_state(bot_id, &bot.state));
                }
                return;
            }
            if bot.state.crouching {
                bot.state.crouching = false;
            }
            if stay_lazy {
                let step_ticks = steps[idx].ticks;
                bot.active_ticks += 1;
                elapsed += 1;
                lazy_ticks += 1;
                if elapsed >= step_ticks {
                    bot.state = steps[idx].end.materialize();
                    idx += 1;
                    elapsed = 0;
                    lazy_ticks = 0;
                    step_start_state = bot.state.clone();
                }
                let arrived = idx >= steps.len();
                if arrived {
                    bot.arrived = true;
                    if is_settled(&bot.state) {
                        bot.settled = true;
                        bot.phase = BotPhase::Idle;
                        bot.hooked_corridor = None;
                    } else {
                        bot.phase = BotPhase::Settling;
                    }
                } else {
                    bot.phase = BotPhase::Executing {
                        steps,
                        idx,
                        elapsed,
                        lazy_ticks,
                        delay_ticks: 0,
                        step_start_state,
                        goal_air,
                        corridor,
                    };
                }
                return;
            }

            let step = &steps[idx];
            let (prog, args, ticks) = (step.program, step.args, step.ticks);
            let selected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                programs[prog as usize].select(world, &bot.state, elapsed, args)
            }));
            let input = match selected {
                Ok(Some((_, inp))) => inp,
                Ok(None) | Err(_) => {
                    if emit_statuses {
                        out.plan_statuses.push(PlanStatusOut {
                            bot_id,
                            status: PLAN_STATUS_REFUSED,
                            predicted_ticks: 0,
                            solve_ns: 0,
                            msg: "word program select failed".into(),
                        });
                    }
                    return;
                }
            };

            let prev_sprinting = bot.state.sprinting;
            let tick_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                law::try_tick(&mut bot.state, world, &input)
            }));
            match tick_res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if emit_statuses {
                        out.plan_statuses.push(PlanStatusOut {
                            bot_id,
                            status: PLAN_STATUS_REFUSED,
                            predicted_ticks: 0,
                            solve_ns: 0,
                            msg: e.0,
                        });
                    }
                    return;
                }
                Err(_) => {
                    if emit_statuses {
                        out.plan_statuses.push(PlanStatusOut {
                            bot_id,
                            status: PLAN_STATUS_REFUSED,
                            predicted_ticks: 0,
                            solve_ns: 0,
                            msg: "law tick panicked".into(),
                        });
                    }
                    return;
                }
            }

            bot.active_ticks += 1;
            elapsed += 1;
            if elapsed >= ticks {
                idx += 1;
                elapsed = 0;
                lazy_ticks = 0;
                step_start_state = bot.state.clone();
            }
            let arrived = idx >= steps.len();
            let mut settled = false;
            if arrived {
                bot.arrived = true;
                if is_settled(&bot.state) {
                    settled = true;
                    bot.settled = true;
                    bot.phase = BotPhase::Idle;
                    bot.hooked_corridor = None;
                } else {
                    bot.phase = BotPhase::Settling;
                }
            } else {
                bot.phase = BotPhase::Executing {
                    steps,
                    idx,
                    elapsed,
                    lazy_ticks,
                    delay_ticks: 0,
                    step_start_state,
                    goal_air,
                    corridor,
                };
            }

            if next_tier == TIER_MATERIALIZED {
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
                out.tick_cmds.push(TickCmdOut {
                    bot_id,
                    keys: input_keys(&input),
                    flags,
                    x: bot.state.position.x,
                    y: bot.state.position.y,
                    z: bot.state.position.z,
                    yaw: bot.state.yaw,
                    pitch: bot.state.pitch,
                });
            } else if next_tier == TIER_VISUAL {
                out.visuals.push(visual_from_state(bot_id, &bot.state));
            }
        }
        BotPhase::Settling => {
            out.stats.active_bots += 1;
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
                _ => {
                    if emit_statuses {
                        out.plan_statuses.push(PlanStatusOut {
                            bot_id,
                            status: PLAN_STATUS_REFUSED,
                            predicted_ticks: 0,
                            solve_ns: 0,
                            msg: "law settle failed".into(),
                        });
                    }
                    return;
                }
            }

            let settled = is_settled(&bot.state);
            if settled {
                bot.settled = true;
                bot.phase = BotPhase::Idle;
                bot.hooked_corridor = None;
            } else {
                bot.phase = BotPhase::Settling;
            }

            if next_tier == TIER_MATERIALIZED {
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
                out.tick_cmds.push(TickCmdOut {
                    bot_id,
                    keys: 0,
                    flags,
                    x: bot.state.position.x,
                    y: bot.state.position.y,
                    z: bot.state.position.z,
                    yaw: bot.state.yaw,
                    pitch: bot.state.pitch,
                });
            } else if next_tier == TIER_VISUAL {
                out.visuals.push(visual_from_state(bot_id, &bot.state));
            }
        }
    }
}

fn state_tier_flags(s: &PlayerState, hooked: bool) -> u32 {
    let mut f = 0u32;
    if s.on_ground {
        f |= TIER_FLAG_ON_GROUND;
    }
    if s.sprinting {
        f |= TIER_FLAG_SPRINTING;
    }
    if s.crouching {
        f |= TIER_FLAG_SNEAKING;
    }
    if hooked {
        f |= 1 << 7;
    }
    f
}

fn visual_from_state(bot_id: u32, s: &PlayerState) -> VisualOut {
    let mut flags = 0u8;
    if s.on_ground {
        flags |= 1;
    }
    if s.sprinting {
        flags |= 2;
    }
    if s.crouching {
        flags |= 4;
    }
    VisualOut {
        bot_id,
        x: s.position.x,
        y: s.position.y,
        z: s.position.z,
        vx: s.velocity.x as f32,
        vy: s.velocity.y as f32,
        vz: s.velocity.z as f32,
        yaw: s.yaw,
        pitch: s.pitch,
        flags,
    }
}
