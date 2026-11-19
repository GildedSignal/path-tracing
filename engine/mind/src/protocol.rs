//! Binary protocol codecs matching Java `ShmBridge` byte-for-byte (little-endian).

pub const MSG_WORLD_RESET: u16 = 1;
pub const MSG_WORLD_RULES: u16 = 2;
pub const MSG_SECTION_AIR: u16 = 3;
pub const MSG_SECTION_RLE: u16 = 4;
pub const MSG_BLOCK_DELTA: u16 = 5;
pub const MSG_CHUNK_UNLOAD: u16 = 6;

pub const MSG_BOT_SPAWN: u16 = 10;
pub const MSG_BOT_REMOVE: u16 = 11;
pub const MSG_BOT_SELF_SYNC: u16 = 12;
pub const MSG_BOT_GOAL: u16 = 13;
pub const MSG_BOT_STOP: u16 = 14;
pub const MSG_SWARM_SPAWN_BATCH: u16 = 15;
pub const MSG_SWARM_CLEAR: u16 = 16;
pub const MSG_SWARM_GOAL_BATCH: u16 = 17;
pub const MSG_SWARM_STOP_ALL: u16 = 18;
pub const MSG_OBSERVERS: u16 = 19;
pub const MSG_TICK_BEGIN: u16 = 20;
pub const MSG_SWARM_CONFIG: u16 = 21;
pub const MSG_RENDER_STREAM: u16 = 22;

pub const MSG_BOT_PLAN_STATUS: u16 = 100;
pub const MSG_BOT_TICK_CMD: u16 = 101;
pub const MSG_BOT_TIER_CHANGE: u16 = 102;
pub const MSG_VISUAL_BATCH: u16 = 103;
pub const MSG_CHUNK_TICKET: u16 = 104;
pub const MSG_SWARM_PLAN_SUMMARY: u16 = 105;
pub const MSG_RENDER_FRAME: u16 = 106;
pub const MSG_TICK_END: u16 = 120;

pub const PLAN_STATUS_READY: u32 = 0;
pub const PLAN_STATUS_ALREADY_THERE: u32 = 1;
pub const PLAN_STATUS_NEEDS_JOINTS: u32 = 2;
pub const PLAN_STATUS_REFUSED: u32 = 3;

pub const FLAG_ON_GROUND: u32 = 1 << 0;
pub const FLAG_HORIZ_COL: u32 = 1 << 1;
pub const FLAG_START_SPRINT: u32 = 1 << 2;
pub const FLAG_STOP_SPRINT: u32 = 1 << 3;
pub const FLAG_ARRIVED: u32 = 1 << 4;
pub const FLAG_SETTLED: u32 = 1 << 5;

pub const TIER_VIRTUAL: u8 = 0;
pub const TIER_VISUAL: u8 = 1;
pub const TIER_MATERIALIZED: u8 = 2;

pub const MODE_VIRTUAL: u8 = 0;
pub const MODE_AUTO: u8 = 1;
pub const MODE_MATERIALIZED: u8 = 2;

pub const TIER_FLAG_ON_GROUND: u32 = 1 << 0;
pub const TIER_FLAG_SPRINTING: u32 = 1 << 1;
pub const TIER_FLAG_SNEAKING: u32 = 1 << 2;

pub struct Reader<'a> {
    buf: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, at: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.at)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.at + n > self.buf.len() {
            return Err("truncated protocol payload".into());
        }
        let s = &self.buf[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }

    pub fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16, String> {
        let s = self.take(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }

    pub fn i32(&mut self) -> Result<i32, String> {
        let s = self.take(4)?;
        Ok(i32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }

    pub fn u32(&mut self) -> Result<u32, String> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }

    pub fn u64(&mut self) -> Result<u64, String> {
        let s = self.take(8)?;
        Ok(u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]))
    }

    pub fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_bits(self.u32()?))
    }

    pub fn f64(&mut self) -> Result<f64, String> {
        Ok(f64::from_bits(self.u64()?))
    }

    pub fn str(&mut self) -> Result<&'a str, String> {
        let len = self.u16()? as usize;
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes).map_err(|_| "invalid utf8 in protocol string".into())
    }
}

pub struct BotSyncPayload {
    pub bot_id: u32,
    pub on_ground: bool,
    pub sprinting: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub food_level: i32,
    pub food_tick_timer: u32,
    pub hurt_rem: i32,
    pub last_hurt: f32,
}

impl BotSyncPayload {
    pub fn read(r: &mut Reader<'_>) -> Result<Self, String> {
        let bot_id = r.u32()?;
        let flags = r.u32()?;
        Ok(Self {
            bot_id,
            on_ground: (flags & 1) != 0,
            sprinting: (flags & 2) != 0,
            x: r.f64()?,
            y: r.f64()?,
            z: r.f64()?,
            vx: r.f64()?,
            vy: r.f64()?,
            vz: r.f64()?,
            yaw: r.f32()?,
            pitch: r.f32()?,
            health: r.f32()?,
            saturation: r.f32()?,
            exhaustion: r.f32()?,
            food_level: r.i32()?,
            food_tick_timer: r.u32()?,
            hurt_rem: r.i32()?,
            last_hurt: r.f32()?,
        })
    }
}

pub struct Writer {
    pub buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i16(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }

    pub fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }

    pub fn str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len().min(u16::MAX as usize);
        self.u16(len as u16);
        self.buf.extend_from_slice(&bytes[..len]);
    }
}
