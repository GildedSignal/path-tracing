//! Isolated `OldMinecartBehavior` minecart rail and motion law (Mojang 26.2,
//! `FeatureFlags.MINECART_IMPROVEMENTS` off — the vanilla default).
//!
//! References in Mojang 26.2 server bytecode:
//! - `AbstractMinecart.<init>` (`@16-50`), `FeatureFlags.DEFAULT_FLAGS` (`@104-111`)
//! - `AbstractMinecart.tick` (`@0-92`), `applyNaturalSlowdown` (`@946-966`),
//!   `comeOffTrack` (`@694-744`), `getDefaultGravity` (`@476-484`), `exits` (`lambda$static$0`)
//! - `OldMinecartBehavior.tick` (`@58-240`), `moveAlongTrack` (`@242-970`),
//!   `getPos` (`@1140-1359`), `getPosOffs` (`@972-1138`), `getSlowdownFactor` (`@1533-1542`)

use crate::{math, BlockPos, Vec3};

/// `RailShape` ordinals 0..=9 (`RailShape`); slopes are `2..=5`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RailShape {
    NorthSouth = 0,
    EastWest = 1,
    AscendingEast = 2,
    AscendingWest = 3,
    AscendingNorth = 4,
    AscendingSouth = 5,
    SouthEast = 6,
    SouthWest = 7,
    NorthWest = 8,
    NorthEast = 9,
}

impl RailShape {
    /// `AbstractMinecart.lambda$static$0` (`EXITS` map):
    /// exit pairs from unit horizontal vectors and their `.below()` variants.
    pub const fn exits(self) -> (BlockPos, BlockPos) {
        const W: BlockPos = [-1, 0, 0];
        const E: BlockPos = [1, 0, 0];
        const N: BlockPos = [0, 0, -1];
        const S: BlockPos = [0, 0, 1];
        const fn below([x, y, z]: BlockPos) -> BlockPos {
            [x, y - 1, z]
        }
        match self {
            Self::NorthSouth => (N, S),
            Self::EastWest => (W, E),
            Self::AscendingEast => (below(W), E),
            Self::AscendingWest => (W, below(E)),
            Self::AscendingNorth => (N, below(S)),
            Self::AscendingSouth => (below(N), S),
            Self::SouthEast => (S, E),
            Self::SouthWest => (S, W),
            Self::NorthWest => (N, W),
            Self::NorthEast => (N, E),
        }
    }

    /// `RailShape.isSlope()`: true for the four ascending shapes.
    #[inline]
    pub const fn is_slope(self) -> bool {
        matches!(
            self,
            Self::AscendingEast
                | Self::AscendingWest
                | Self::AscendingNorth
                | Self::AscendingSouth
        )
    }
}

/// Evaluated rail state at one block position (`BaseRailBlock.isRail` = `BlockTags.RAILS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RailInfo {
    pub shape: RailShape,
    pub powered: bool,
    pub is_powered_rail: bool,
}

impl RailInfo {
    #[inline]
    pub const fn new(shape: RailShape, powered: bool, is_powered_rail: bool) -> Self {
        Self {
            shape,
            powered,
            is_powered_rail,
        }
    }

}

/// World queries required by `OldMinecartBehavior` (`getCurrentBlockPosOrRailBelow`,
/// `moveAlongTrack`, `getPos`, and `isRedstoneConductor`).
pub trait Rails {
    fn rail_at(&self, pos: BlockPos) -> Option<RailInfo>;

    /// `BlockGetter.getBlockState(pos.below()).is(BlockTags.RAILS)`.
    fn rails_tag_below(&self, pos: BlockPos) -> bool;

    /// `BlockState.isRedstoneConductor(level, pos)` beside a powered rail at rail level.
    fn is_redstone_conductor(&self, pos: BlockPos) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Minecart {
    pub position: Vec3,
    pub velocity: Vec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CartConditions {
    pub in_water: bool,
    pub no_gravity: bool,
    /// `Entity.isVehicle()` (has passengers): move factor `0.75` and slowdown `0.997`
    /// instead of `1.0` / `0.96`.
    pub occupied: bool,
    /// Used by `comeOffTrack` (post-collision `onGround()`).
    pub on_ground: bool,
    /// `ServerPlayer.getLastClientMoveIntent()`; `None` if no `ServerPlayer` first passenger.
    pub passenger_intent: Option<Vec3>,
}

/// Slope acceleration per tick (`OldMinecartBehavior.moveAlongTrack @113`).
pub const SLOPE_ACCEL: f64 = 0.0078125;
/// Water slope multiplier (`OldMinecartBehavior.moveAlongTrack @128-134`, `0.2`, not `0.5`).
pub const WATER_SLOPE_FACTOR: f64 = 0.2;
/// `OldMinecartBehavior.getMaxSpeed` on land.
pub const MAX_SPEED_LAND: f64 = 0.4;
/// `OldMinecartBehavior.getMaxSpeed` in water.
pub const MAX_SPEED_WATER: f64 = 0.2;
/// `OldMinecartBehavior.getSlowdownFactor` when empty (`!isVehicle()`).
pub const SLOWDOWN_EMPTY: f64 = 0.96;
/// `OldMinecartBehavior.getSlowdownFactor` when occupied (`isVehicle()`).
pub const SLOWDOWN_OCCUPIED: f64 = 0.997;
/// `OldMinecartBehavior` rail-top vertical offset (`blockY + 0.0625`).
pub const RAIL_TOP_OFFSET: f64 = 0.0625;
/// Powered golden-rail boost magnitude (`0.06`) and speed gate (`> 0.01`).
pub const POWERED_BOOST: f64 = 0.06;
pub const POWERED_BOOST_MIN_SPEED: f64 = 0.01;
/// Powered golden-rail push away from an adjacent redstone conductor at rest.
pub const WALL_PUSH_SPEED: f64 = 0.02;
/// Unpowered golden-rail stop threshold (`< 0.03`).
pub const BRAKE_STOP_SPEED: f64 = 0.03;

/// `Mth.clamp(double, double, double)`.
#[inline]
fn clamp_f64(v: f64, min: f64, max: f64) -> f64 {
    if v < min {
        min
    } else {
        v.min(max)
    }
}

/// `Vec3.horizontalDistance()`: `Math.sqrt(x * x + z * z)`.
#[inline]
fn horizontal_distance(v: Vec3) -> f64 {
    (v.x * v.x + v.z * v.z).sqrt()
}

impl Minecart {
    /// Steps one server tick of `OldMinecartBehavior` motion (collision-free; caller supplies
    /// `c.on_ground` for `comeOffTrack`).
    pub fn tick(&mut self, rails: &(impl Rails + ?Sized), c: CartConditions) {
        if !c.no_gravity {
            crate::body::MINECART.gravity(&mut self.velocity, c.in_water);
        }
        let bx = math::floor(self.position.x);
        let by = math::floor(self.position.y);
        let bz = math::floor(self.position.z);
        // OldMinecartBehavior.tick @82-110
        let rail_y = if rails.rails_tag_below([bx, by, bz]) {
            by - 1
        } else {
            by
        };
        let rail_pos = [bx, rail_y, bz];
        match rails.rail_at(rail_pos) {
            Some(rail) => self.move_along_track(rails, rail, rail_pos, c),
            None => self.come_off_track(c),
        }
    }

    /// `OldMinecartBehavior.moveAlongTrack` (`@242-970`).
    fn move_along_track(
        &mut self,
        rails: &(impl Rails + ?Sized),
        rail: RailInfo,
        rail_pos: BlockPos,
        c: CartConditions,
    ) {
        let [bx, by, bz] = rail_pos;
        let pre = self.get_pos(rails, self.position.x, self.position.y, self.position.z);
        let mut target_y = by as f64;
        let powered = rail.is_powered_rail && rail.powered;
        let mut brake = rail.is_powered_rail && !rail.powered;

        // Slope gravity (@113-287).
        let mut slope = SLOPE_ACCEL;
        if c.in_water {
            slope *= WATER_SLOPE_FACTOR;
        }
        match rail.shape {
            RailShape::AscendingEast => {
                self.velocity.x -= slope;
                target_y += 1.0;
            }
            RailShape::AscendingWest => {
                self.velocity.x += slope;
                target_y += 1.0;
            }
            RailShape::AscendingNorth => {
                self.velocity.z += slope;
                target_y += 1.0;
            }
            RailShape::AscendingSouth => {
                self.velocity.z -= slope;
                target_y += 1.0;
            }
            _ => {}
        }

        // Rail-line velocity projection (@287-459).
        let (a, b) = rail.shape.exits();
        let mut dx = (b[0] - a[0]) as f64;
        let mut dz = (b[2] - a[2]) as f64;
        let exit_len = (dx * dx + dz * dz).sqrt();
        let dot = self.velocity.x * dx + self.velocity.z * dz;
        if dot < 0.0 {
            dx = -dx;
            dz = -dz;
        }
        let speed = 2.0f64.min(horizontal_distance(self.velocity));
        self.velocity = Vec3::new(speed * dx / exit_len, self.velocity.y, speed * dz / exit_len);

        // Passenger steer (@449-581): ServerPlayer move intent adds 0.001 when h^2 < 0.01
        // and cancels the unpowered golden-rail brake.
        if let Some(intent) = c.passenger_intent {
            if intent.length_sqr() > 0.0 && self.velocity.horizontal_sqr() < 0.01 {
                self.velocity.x += intent.x * 0.001;
                self.velocity.z += intent.z * 0.001;
                brake = false;
            }
        }

        // Unpowered golden-rail brake (@583-631).
        if brake {
            let h = horizontal_distance(self.velocity);
            if h < BRAKE_STOP_SPEED {
                self.velocity = Vec3::default();
            } else {
                self.velocity = Vec3::new(
                    self.velocity.x * 0.5,
                    self.velocity.y * 0.0,
                    self.velocity.z * 0.5,
                );
            }
        }

        // Exit-line snap before move (@634-834).
        let ax = bx as f64 + 0.5 + a[0] as f64 * 0.5;
        let az = bz as f64 + 0.5 + a[2] as f64 * 0.5;
        let bx_exit = bx as f64 + 0.5 + b[0] as f64 * 0.5;
        let bz_exit = bz as f64 + 0.5 + b[2] as f64 * 0.5;
        let seg_x = bx_exit - ax;
        let seg_z = bz_exit - az;
        let t = if seg_x == 0.0 {
            self.position.z - bz as f64
        } else if seg_z == 0.0 {
            self.position.x - bx as f64
        } else {
            ((self.position.x - ax) * seg_x + (self.position.z - az) * seg_z) * 2.0
        };
        self.position = Vec3::new(ax + seg_x * t, target_y, az + seg_z * t);

        // Clamped move (@837-919): occupied carts move at 0.75x; velocity itself is NOT clamped.
        let move_scale = if c.occupied { 0.75 } else { 1.0 };
        let max_speed = if c.in_water {
            MAX_SPEED_WATER
        } else {
            MAX_SPEED_LAND
        };
        let mv = Vec3::new(
            clamp_f64(move_scale * self.velocity.x, -max_speed, max_speed),
            0.0,
            clamp_f64(move_scale * self.velocity.z, -max_speed, max_speed),
        );
        self.position = self.position.plus(mv);

        // Raised slope-exit y snap (@922-1094).
        let cur_x = math::floor(self.position.x);
        let cur_z = math::floor(self.position.z);
        if a[1] != 0 && cur_x - bx == a[0] && cur_z - bz == a[2] {
            self.position.y += a[1] as f64;
        } else if b[1] != 0 && cur_x - bx == b[0] && cur_z - bz == b[2] {
            self.position.y += b[1] as f64;
        }

        // Natural slowdown (@1097-1109 + AbstractMinecart.applyNaturalSlowdown @946-966).
        let slowdown = if c.occupied {
            SLOWDOWN_OCCUPIED
        } else {
            SLOWDOWN_EMPTY
        };
        self.velocity = Vec3::new(
            self.velocity.x * slowdown,
            self.velocity.y * 0.0,
            self.velocity.z * slowdown,
        );
        if c.in_water {
            crate::body::MINECART.drag(&mut self.velocity, true);
        }

        // Rail-height y adjust (@1112-1237).
        let post = self.get_pos(rails, self.position.x, self.position.y, self.position.z);
        if let (Some(post), Some(pre)) = (post, pre) {
            let dy = (pre.y - post.y) * 0.05;
            let h = horizontal_distance(self.velocity);
            if h > 0.0 {
                let k = (h + dy) / h;
                self.velocity.x *= k;
                self.velocity.z *= k;
            }
            self.position.y = post.y;
        }

        // Block-crossing velocity redirect (@1235-1320).
        let nx = math::floor(self.position.x);
        let nz = math::floor(self.position.z);
        if nx != bx || nz != bz {
            let h = horizontal_distance(self.velocity);
            self.velocity = Vec3::new(
                h * (nx - bx) as f64,
                self.velocity.y,
                h * (nz - bz) as f64,
            );
        }

        // Powered golden-rail boost or wall push (@1321-1528).
        if powered {
            let h = horizontal_distance(self.velocity);
            if h > POWERED_BOOST_MIN_SPEED {
                self.velocity = self.velocity.plus(Vec3::new(
                    self.velocity.x / h * POWERED_BOOST,
                    0.0,
                    self.velocity.z / h * POWERED_BOOST,
                ));
            } else {
                let mut vx = self.velocity.x;
                let mut vz = self.velocity.z;
                match rail.shape {
                    RailShape::EastWest => {
                        if rails.is_redstone_conductor([bx - 1, by, bz]) {
                            vx = WALL_PUSH_SPEED;
                        } else if rails.is_redstone_conductor([bx + 1, by, bz]) {
                            vx = -WALL_PUSH_SPEED;
                        } else {
                            return;
                        }
                    }
                    RailShape::NorthSouth => {
                        if rails.is_redstone_conductor([bx, by, bz - 1]) {
                            vz = WALL_PUSH_SPEED;
                        } else if rails.is_redstone_conductor([bx, by, bz + 1]) {
                            vz = -WALL_PUSH_SPEED;
                        } else {
                            return;
                        }
                    }
                    _ => return,
                }
                self.velocity = Vec3::new(vx, self.velocity.y, vz);
            }
        }
    }

    /// `AbstractMinecart.comeOffTrack` (`@694-744`).
    pub fn come_off_track(&mut self, c: CartConditions) {
        let max_speed = if c.in_water {
            MAX_SPEED_WATER
        } else {
            MAX_SPEED_LAND
        };
        self.velocity.x = clamp_f64(self.velocity.x, -max_speed, max_speed);
        self.velocity.z = clamp_f64(self.velocity.z, -max_speed, max_speed);
        if c.on_ground {
            self.velocity = self.velocity.scale(0.5);
        }
        self.position = self.position.plus(self.velocity);
        if !c.on_ground {
            crate::body::MINECART.drag(&mut self.velocity, false);
        }
    }

    /// `OldMinecartBehavior.getPos` (`@1140-1359`): projects `(x, y, z)` onto the rail line at
    /// `blockY + 0.0625`, lifting slopes so the low end sits at `blockY + 0.0625`.
    pub fn get_pos(&self, rails: &(impl Rails + ?Sized), x: f64, y: f64, z: f64) -> Option<Vec3> {
        let bx = math::floor(x);
        let mut by = math::floor(y);
        let bz = math::floor(z);
        if rails.rails_tag_below([bx, by, bz]) {
            by -= 1;
        }
        let rail = rails.rail_at([bx, by, bz])?;
        let (a, b) = rail.shape.exits();
        let ax = bx as f64 + 0.5 + a[0] as f64 * 0.5;
        let ay = by as f64 + RAIL_TOP_OFFSET + a[1] as f64 * 0.5;
        let az = bz as f64 + 0.5 + a[2] as f64 * 0.5;
        let bx_exit = bx as f64 + 0.5 + b[0] as f64 * 0.5;
        let by_exit = by as f64 + RAIL_TOP_OFFSET + b[1] as f64 * 0.5;
        let bz_exit = bz as f64 + 0.5 + b[2] as f64 * 0.5;
        let dx = bx_exit - ax;
        let dy = (by_exit - ay) * 2.0;
        let dz = bz_exit - az;
        let t = if dx == 0.0 {
            z - bz as f64
        } else if dz == 0.0 {
            x - bx as f64
        } else {
            ((x - ax) * dx + (z - az) * dz) * 2.0
        };
        let px = ax + dx * t;
        let mut py = ay + dy * t;
        let pz = az + dz * t;
        if dy < 0.0 {
            py += 1.0;
        } else if dy > 0.0 {
            py += 0.5;
        }
        Some(Vec3::new(px, py, pz))
    }

    /// `OldMinecartBehavior.getPosOffs` (`@972-1138`): samples `getPos` shifted by `offset`
    /// along the unit rail exit tangent.
    pub fn get_pos_offs(
        &self,
        rails: &(impl Rails + ?Sized),
        mut x: f64,
        mut y: f64,
        mut z: f64,
        offset: f64,
    ) -> Option<Vec3> {
        let bx = math::floor(x);
        let mut by = math::floor(y);
        let bz = math::floor(z);
        if rails.rails_tag_below([bx, by, bz]) {
            by -= 1;
        }
        let rail = rails.rail_at([bx, by, bz])?;
        y = by as f64;
        if rail.shape.is_slope() {
            y = (by + 1) as f64;
        }
        let (a, b) = rail.shape.exits();
        let mut dx = (b[0] - a[0]) as f64;
        let mut dz = (b[2] - a[2]) as f64;
        let len = (dx * dx + dz * dz).sqrt();
        dx /= len;
        dz /= len;
        x += dx * offset;
        z += dz * offset;
        let fx = math::floor(x);
        let fz = math::floor(z);
        if a[1] != 0 && fx - bx == a[0] && fz - bz == a[2] {
            y += a[1] as f64;
        } else if b[1] != 0 && fx - bx == b[0] && fz - bz == b[2] {
            y += b[1] as f64;
        }
        self.get_pos(rails, x, y, z)
    }
}
