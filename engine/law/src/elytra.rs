//! Elytra firework boost and wall-impact damage laws (26.2).
//!
//! Glide itself (`LivingEntity.travelFallFlying`) is not in this module; the boost
//! is a foreign write from `FireworkRocketEntity.tick` that lands after the rider's
//! same-tick `travelFallFlying` and takes effect on tick N + 1.

use crate::{math, Vec3};

/// `FireworkRocketEntity.tick` (`@75-174`) velocity rewrite in Mojang's exact
/// bytecode association: `v + look * 0.1 + (look * 1.5 - v) * 0.5`.
pub fn firework_boost(vel: Vec3, yaw: f32, pitch: f32) -> Vec3 {
    let look = crate::math::calculate_view_vector(yaw, pitch);
    vel.plus(Vec3::new(
        look.x * 0.1 + (look.x * 1.5 - vel.x) * 0.5,
        look.y * 0.1 + (look.y * 1.5 - vel.y) * 0.5,
        look.z * 0.1 + (look.z * 1.5 - vel.z) * 0.5,
    ))
}

/// `FireworkRocketEntity.<init>(Level, double, double, double, ItemStack)` (`@47-141`):
/// `lifetime = 10 * (1 + flight_duration) + nextInt(6) + nextInt(7)`.
pub fn firework_lifetime_bounds(flight_duration: u8) -> (u32, u32) {
    let base = 10 * (1 + flight_duration as u32);
    (base, base + 11)
}

/// Inclusive range of boost ticks while the attached rider remains fall-flying:
/// `FireworkRocketEntity.tick` boosts before `life++` and explodes when `life > lifetime`.
pub fn firework_boost_ticks_bounds(flight_duration: u8) -> (u32, u32) {
    let (lo, hi) = firework_lifetime_bounds(flight_duration);
    (lo + 1, hi + 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FireworkStep {
    pub boosted: bool,
    pub exploded: bool,
}

/// Attached `FireworkRocketEntity` tick state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttachedFirework {
    pub life: u32,
    pub lifetime: u32,
}

impl AttachedFirework {
    pub fn new(lifetime: u32) -> Self {
        Self { life: 0, lifetime }
    }

    /// `FireworkRocketItem.use` (`@0-4,49-57`) only spawns when `isFallFlying()`.
    pub fn try_use(fall_flying: bool, lifetime: u32) -> Option<Self> {
        fall_flying.then_some(Self::new(lifetime))
    }

    /// `FireworkRocketEntity.tick` (`@40-174, 420-427, 510-540`).
    /// Runs in `EntityTickList` after the rider's same-tick `travelFallFlying`,
    /// so any boost written on tick `N` takes effect on the rider's travel at tick `N + 1`.
    pub fn tick(
        &mut self,
        attached_fall_flying: bool,
        vel: &mut Vec3,
        yaw: f32,
        pitch: f32,
    ) -> FireworkStep {
        if attached_fall_flying {
            *vel = firework_boost(*vel, yaw, pitch);
        }
        self.life += 1;
        let exploded = self.life > self.lifetime;
        FireworkStep {
            boosted: attached_fall_flying,
            exploded,
        }
    }
}

/// `LivingEntity.handleFallFlyingCollisions(double, double)` (`@0-58`):
/// `f7 = (float)((pre_h_speed - post_h_speed) * 10.0 - 3.0)`, firing `hurt(flyIntoWall, f7)`
/// on `fcmpl; ifle` (so `f7 > 0.0f` OR `f7.is_nan()` fires; `0.3` drop -> `0.0f` silent).
pub fn fly_into_wall_damage(
    horizontal_collision: bool,
    pre_h_speed: f64,
    post_h_speed: f64,
) -> Option<f32> {
    if !horizontal_collision {
        return None;
    }
    let drop = pre_h_speed - post_h_speed;
    let f7 = (drop * 10.0 - 3.0) as f32;
    (f7 > 0.0f32 || f7.is_nan()).then_some(f7)
}

/// `Entity.checkFallDistanceAccumulation()` (`@0-28`): clamps `fall_distance` down to `1.0`
/// when falling slower than `-0.5` blocks/tick while fall-flying.
pub fn check_fall_distance_accumulation(vel_y: f64, fall_distance: f64) -> f64 {
    if vel_y > -0.5 && fall_distance > 1.0 {
        1.0
    } else {
        fall_distance
    }
}

fn f32_rad(pitch_degrees: f32) -> f32 {
    pitch_degrees * 0.017_453_292_f32
}

/// `java.lang.Math.cos` on HotSpot x86-64 (`java-runtime-epsilon`) for the elytra pitch
/// domain `|x| = |pitch * 0.017453292f| <= pi/2`, bit-exact with Intel LIBM intrinsic.
pub fn jvm_dcos(x: f64) -> f64 {
    if let Ok(i) = DCOS_DELTA.binary_search_by_key(&x.to_bits(), |&(input, _)| input) {
        return f64::from_bits(DCOS_DELTA[i].1);
    }
    fdlibm_cos(x)
}

/// `LivingEntity.updateFallFlyingMovement` (`@0-262`): one tick of the glide velocity
/// recurrence. `look` is `crate::math::calculate_view_vector(yaw, pitch)`; `gravity` is `effective_gravity`.
pub fn update_fall_flying_movement(v: Vec3, look: Vec3, pitch: f32, gravity: f64) -> Vec3 {
    let f = f32_rad(pitch);
    // @14-36 d0 = sqrt(look.x*look.x + look.z*look.z)
    let d0 = (look.x * look.x + look.z * look.z).sqrt();
    // @38-42 d1 = vec.horizontalDistance()
    let d1 = (v.x * v.x + v.z * v.z).sqrt();
    // @50-58 d10 = Mth.square(Math.cos((double) f))
    let c = jvm_dcos(f as f64);
    let d10 = c * c;
    // @60-75 vec = vec.add(0, gravity * (-1 + d10*0.75), 0)
    let mut v = Vec3 {
        x: v.x,
        y: v.y + gravity * (-1.0 + d10 * 0.75),
        z: v.z,
    };
    // @80-135 dive boost: vec.y < 0 && d0 > 0
    if v.y < 0.0 && d0 > 0.0 {
        let d12 = v.y * -0.1 * d10;
        v = Vec3 {
            x: v.x + look.x * d12 / d0,
            y: v.y + d12,
            z: v.z + look.z * d12 / d0,
        };
    }
    // @136-197 pull-up conversion: f < 0 && d0 > 0
    if f < 0.0 && d0 > 0.0 {
        let d12 = d1 * (-(math::sin(f as f64) as f64)) * 0.04;
        v = Vec3 {
            x: v.x + -look.x * d12 / d0,
            y: v.y + d12 * 3.2,
            z: v.z + -look.z * d12 / d0,
        };
    }
    // @198-245 horizontal alignment: d0 > 0
    if d0 > 0.0 {
        v = Vec3 {
            x: v.x + (look.x / d0 * d1 - v.x) * 0.1,
            y: v.y,
            z: v.z + (look.z / d0 * d1 - v.z) * 0.1,
        };
    }
    // @249-262 final drag
    Vec3 {
        x: v.x * 0.9900000095367432,
        y: v.y * 0.9800000190734863,
        z: v.z * 0.9900000095367432,
    }
}

const PIO2_1: f64 = 1.5707963267341256;
const PIO2_1T: f64 = 6.077100506506192E-11;
const PIO2_2: f64 = 6.077100506303966E-11;
const PIO2_2T: f64 = 2.0222662487959506E-21;
const PIO2_3: f64 = 2.0222662487111665E-21;
const PIO2_3T: f64 = 8.4784276603689E-32;
const INVPIO2: f64 = 0.6366197723675814;

const NPIO2_HW: [i32; 32] = [
    1073291771, 1074340347, 1074977148, 1075388923, 1075800698, 1076025724, 1076231611,
    1076437499, 1076643386, 1076849274, 1076971356, 1077074300, 1077177244, 1077280187,
    1077383131, 1077486075, 1077589019, 1077691962, 1077794906, 1077897850, 1077968460,
    1078019932, 1078071404, 1078122876, 1078174348, 1078225820, 1078277292, 1078328763,
    1078380235, 1078431707, 1078483179, 1078534651,
];

fn hi(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

fn from_hi(h: i32) -> f64 {
    f64::from_bits((h as u32 as u64) << 32)
}

// FdLibm$Cos.compute @4-120
fn fdlibm_cos(x: f64) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix <= 0x3FE9_21FB {
        return kernel_cos(x, 0.0);
    }
    if ix >= 0x7FF0_0000 {
        return x - x;
    }
    let mut y = [0.0f64; 2];
    let n = rem_pio2(x, &mut y);
    match n & 3 {
        0 => kernel_cos(y[0], y[1]),
        1 => -kernel_sin(y[0], y[1], 1),
        2 => -kernel_cos(y[0], y[1]),
        _ => kernel_sin(y[0], y[1], 1),
    }
}

// FdLibm$Cos.__kernel_cos @0-158
fn kernel_cos(x: f64, y: f64) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix < 0x3E40_0000 && (x as i32) == 0 {
        return 1.0;
    }
    let z = x * x;
    let r = z
        * (0.0416666666666666
            + z * (-0.001388888888887411
                + z * (2.480158728947673E-5
                    + z * (-2.7557314351390663E-7
                        + z * (2.087572321298175E-9 + z * -1.1359647557788195E-11)))));
    if ix < 0x3FD3_3333 {
        return 1.0 - (0.5 * z - (z * r - x * y));
    }
    let hx = if ix > 0x3FE9_0000 {
        0.28125
    } else {
        from_hi(ix - 0x20_0000)
    };
    let d = 0.5 * z - hx;
    let e = 1.0 - hx;
    e - (d - (z * r - x * y))
}

// FdLibm$Sin.__kernel_sin @0-112
fn kernel_sin(x: f64, y: f64, iy: i32) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix < 0x3E40_0000 && (x as i32) == 0 {
        return x;
    }
    let z = x * x;
    let w = z * x;
    let v = 0.00833333333332249
        + z * (-1.984126982985795E-4
            + z * (2.7557313707070068E-6
                + z * (-2.5050760253406863E-8 + z * 1.58969099521155E-10)));
    if iy == 0 {
        return x + w * (-0.16666666666666632 + z * v);
    }
    x - ((z * (0.5 * y - w * v) - y) - w * -0.16666666666666632)
}

// FdLibm$RemPio2.__ieee754_rem_pio2 @7-448
fn rem_pio2(x: f64, y: &mut [f64; 2]) -> i32 {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;
    if ix <= 0x3FE9_21FB {
        y[0] = x;
        y[1] = 0.0;
        return 0;
    }
    if ix < 0x4005_0D2E {
        if hx > 0 {
            let mut z = x - PIO2_1;
            if ix != 0x3FF9_21FB {
                y[0] = z - PIO2_1T;
                y[1] = (z - y[0]) - PIO2_1T;
            } else {
                z -= PIO2_2;
                y[0] = z - PIO2_2T;
                y[1] = (z - y[0]) - PIO2_2T;
            }
            return 1;
        }
        let mut z = x + PIO2_1;
        if ix != 0x3FF9_21FB {
            y[0] = z + PIO2_1T;
            y[1] = (z - y[0]) + PIO2_1T;
        } else {
            z += PIO2_2;
            y[0] = z + PIO2_2T;
            y[1] = (z - y[0]) + PIO2_2T;
        }
        return -1;
    }
    if ix > 0x4139_21FB {
        y[0] = x.cos();
        y[1] = 0.0;
        return 0;
    }
    let t0 = x.abs();
    let n = (t0 * INVPIO2 + 0.5) as i32;
    let fn_ = n as f64;
    let mut r = t0 - fn_ * PIO2_1;
    let mut w = fn_ * PIO2_1T;
    if n >= 32 || ix == NPIO2_HW[(n - 1) as usize] {
        let jv = ix >> 20;
        y[0] = r - w;
        let mut i = jv - ((y[0].to_bits() >> 52) as i32 & 0x7ff);
        if i > 16 {
            let t = r;
            w = fn_ * PIO2_2;
            r = t - w;
            w = fn_ * PIO2_2T - ((t - r) - w);
            y[0] = r - w;
            i = jv - ((y[0].to_bits() >> 52) as i32 & 0x7ff);
            if i > 49 {
                let t = r;
                w = fn_ * PIO2_3;
                r = t - w;
                w = fn_ * PIO2_3T - ((t - r) - w);
                y[0] = r - w;
            }
        }
    } else {
        y[0] = r - w;
    }
    y[1] = (r - y[0]) - w;
    if hx < 0 {
        y[0] = -y[0];
        y[1] = -y[1];
        return -n;
    }
    n
}

static DCOS_DELTA: [(u64, u64); 252] = [
    (0x3fb0532ee0000000, 0x3fefef5963a71975),
    (0x3fc03f6da0000000, 0x3fefbe1751a736e9),
    (0x3fc441b2c0000000, 0x3fef99a190b3ce87),
    (0x3fc6b4fee0000000, 0x3fef7f6f81a37251),
    (0x3fcbeb3c80000000, 0x3fef3de7bb6829af),
    (0x3fcd72ec60000000, 0x3fef28251b81e0e7),
    (0x3fcf557ea0000000, 0x3fef0bc4dc1e40bf),
    (0x3fd11a6c60000000, 0x3feedd377c68eb45),
    (0x3fd18a7f00000000, 0x3feece3c0d521e59),
    (0x3fd1cb0c40000000, 0x3feec57007942fc3),
    (0x3fd55aa0a0000000, 0x3fee3c388638ac43),
    (0x3fd56240e0000000, 0x3fee3af8a0f36da1),
    (0x3fd5be9d80000000, 0x3fee2bb429a38bef),
    (0x3fd633eac0000000, 0x3fee17f5c103e259),
    (0x3fd646a260000000, 0x3fee14c5e2fffbb5),
    (0x3fda609660000000, 0x3fed52080d9348fb),
    (0x3fdab43f00000000, 0x3fed412d87dd8b79),
    (0x3fdeb565c0000000, 0x3fec62efe43076fd),
    (0x3fe2004640000000, 0x3feb11aaca371a49),
    (0x3fe5810aa0000000, 0x3fe90ade7ea65f4b),
    (0x3fe7348d20000000, 0x3fe7f2cfa4aee0c3),
    (0x3fe7d5a680000000, 0x3fe786c83962f7e1),
    (0x3fe802db80000000, 0x3fe7680c1009ae95),
    (0x3fe875cac0000000, 0x3fe7191636473c5d),
    (0x3fe883f760000000, 0x3fe70f449dd628d1),
    (0x3fe88b8020000000, 0x3fe70a0ab455458f),
    (0x3fe8c30200000000, 0x3fe6e3627a568c77),
    (0x3fe8f67b60000000, 0x3fe6bf4bc4048d61),
    (0x3fe9217280000000, 0x3fe6a0ff2669e26d),
    (0x3fe9219040000000, 0x3fe6a0ea1d66cb93),
    (0x3fe92195a0000000, 0x3fe6a0e650789687),
    (0x3fe921a5a0000000, 0x3fe6a0db004a849a),
    (0x3fe921b640000000, 0x3fe6a0cf3ef4a7cd),
    (0x3fe921be20000000, 0x3fe6a0c9ad790289),
    (0x3fe921cc40000000, 0x3fe6a0bfb0a3fd6f),
    (0x3fe921e740000000, 0x3fe6a0ac99325fb7),
    (0x3fe921e880000000, 0x3fe6a0abb6ecb945),
    (0x3fe921eca0000000, 0x3fe6a0a8cc39d54d),
    (0x3fe921ed20000000, 0x3fe6a0a871b78495),
    (0x3fe921ee60000000, 0x3fe6a0a78f71b499),
    (0x3fe921f020000000, 0x3fe6a0a652a98293),
    (0x3fe921fc00000000, 0x3fe6a09ded101c53),
    (0x3fe921fe60000000, 0x3fe6a09c3f2441c0),
    (0x3fe92209c0000000, 0x3fe6a0943409c15b),
    (0x3fe9220aa0000000, 0x3fe6a09395a52303),
    (0x3fe9220c20000000, 0x3fe6a092861d2e0d),
    (0x3fe9220e20000000, 0x3fe6a0911c127e4f),
    (0x3fe9221120000000, 0x3fe6a08efd024c45),
    (0x3fe92211e0000000, 0x3fe6a08e753e37ce),
    (0x3fe92217a0000000, 0x3fe6a08a645e8688),
    (0x3fe9221c60000000, 0x3fe6a0870883cb33),
    (0x3fe9221ca0000000, 0x3fe6a086db425f7d),
    (0x3fe9221d60000000, 0x3fe6a086537e1a3b),
    (0x3fe9221dc0000000, 0x3fe6a0860f9bf669),
    (0x3fe9221ee0000000, 0x3fe6a08543f5862d),
    (0x3fe92222c0000000, 0x3fe6a082867f2220),
    (0x3fe92223c0000000, 0x3fe6a081d1794d19),
    (0x3fe9222400000000, 0x3fe6a081a437d6f5),
    (0x3fe92224a0000000, 0x3fe6a08133142e0f),
    (0x3fe9222520000000, 0x3fe6a080d8913ef3),
    (0x3fe9222ba0000000, 0x3fe6a07c3fea99d5),
    (0x3fe9222e20000000, 0x3fe6a07a7b5b7d27),
    (0x3fe9222e60000000, 0x3fe6a07a4e19f856),
    (0x3fe9222f40000000, 0x3fe6a079afb4a4b5),
    (0x3fe9223200000000, 0x3fe6a077bde3cad2),
    (0x3fe92232a0000000, 0x3fe6a0774cbff06d),
    (0x3fe9223420000000, 0x3fe6a0763d36a811),
    (0x3fe9223640000000, 0x3fe6a074bc896bc3),
    (0x3fe9223780000000, 0x3fe6a073da4196b7),
    (0x3fe92237e0000000, 0x3fe6a073965f3b79),
    (0x3fe9223980000000, 0x3fe6a0727034fc3f),
    (0x3fe9223a40000000, 0x3fe6a071e8703c7d),
    (0x3fe9223f20000000, 0x3fe6a06e75f11082),
    (0x3fe9223fc0000000, 0x3fe6a06e04cd07b5),
    (0x3fe92240a0000000, 0x3fe6a06d66675e15),
    (0x3fe92242a0000000, 0x3fe6a06bfc5a5c61),
    (0x3fe92244c0000000, 0x3fe6a06a7bac71c9),
    (0x3fe9224560000000, 0x3fe6a06a0a885519),
    (0x3fe92245e0000000, 0x3fe6a069b005095c),
    (0x3fe9224660000000, 0x3fe6a0695581bc35),
    (0x3fe9224b00000000, 0x3fe6a0661042ef82),
    (0x3fe9224cc0000000, 0x3fe6a064d377288f),
    (0x3fe9224d00000000, 0x3fe6a064a635786f),
    (0x3fe9224d80000000, 0x3fe6a0644bb21721),
    (0x3fe9225380000000, 0x3fe6a0600d89192d),
    (0x3fe92255a0000000, 0x3fe6a05e8cda63bd),
    (0x3fe9225780000000, 0x3fe6a05d396d5413),
    (0x3fe92257c0000000, 0x3fe6a05d0c2b94c1),
    (0x3fe92258e0000000, 0x3fe6a05c4083b36b),
    (0x3fe9225960000000, 0x3fe6a05be6003087),
    (0x3fe9225d60000000, 0x3fe6a05911e3e67f),
    (0x3fe9225ea0000000, 0x3fe6a0582f9afccd),
    (0x3fe9226060000000, 0x3fe6a056f2ce7394),
    (0x3fe9226240000000, 0x3fe6a0559f60f1e7),
    (0x3fe9226260000000, 0x3fe6a05588c00aaf),
    (0x3fe92262a0000000, 0x3fe6a0555b7e3bfb),
    (0x3fe9226420000000, 0x3fe6a0544bf35c57),
    (0x3fe9226a20000000, 0x3fe6a0500dc75e81),
    (0x3fe9226bc0000000, 0x3fe6a04ee79b5161),
    (0x3fe9226e80000000, 0x3fe6a04cf5c6ca5f),
    (0x3fe9227180000000, 0x3fe6a04ad6b034d9),
    (0x3fe9227240000000, 0x3fe6a04a4eea8783),
    (0x3fe9227360000000, 0x3fe6a0498341fd8b),
    (0x3fe9227460000000, 0x3fe6a048ce3a6073),
    (0x3fe9227640000000, 0x3fe6a0477acc0aa7),
    (0x3fe9227960000000, 0x3fe6a0454513fa22),
    (0x3fe9227ea0000000, 0x3fe6a0418eab101b),
    (0x3fe9228080000000, 0x3fe6a0403b3c4d99),
    (0x3fe9228120000000, 0x3fe6a03fca175dad),
    (0x3fe9228680000000, 0x3fe6a03bfd0c8cbd),
    (0x3fe9575f40000000, 0x3fe67abe4b9527b3),
    (0x3fe9bbf040000000, 0x3fe632bd06677391),
    (0x3fea248ce0000000, 0x3fe5e6eda5dc125b),
    (0x3febd51060000000, 0x3fe4a3ede41fc5eb),
    (0x3fec90ba20000000, 0x3fe41325f5ae9b29),
    (0x3fecc437c0000000, 0x3fe3eaf2e180a001),
    (0x3fed004b40000000, 0x3fe3bbcabbeeb20b),
    (0x3fed40a960000000, 0x3fe388f70e779336),
    (0x3fed927360000000, 0x3fe347efcb3f777f),
    (0x3fef481b00000000, 0x3fe1e3c52f019364),
    (0x3ff0a393e0000000, 0x3fe03368ab62ebc3),
    (0x3ff113f000000000, 0x3fdee02966e7728f),
    (0x3ff16a2760000000, 0x3fddb05aa4375c29),
    (0x3ff302f9c0000000, 0x3fd7e43df291fb1f),
    (0x3ff5ffe520000000, 0x3fc8e7c359e0fcf4),
    (0x3ff77364a0000000, 0x3fbadcbc9b8ec166),
    (0xbfb0532ee0000000, 0x3fefef5963a71975),
    (0xbfc03f6da0000000, 0x3fefbe1751a736e9),
    (0xbfc441b2c0000000, 0x3fef99a190b3ce87),
    (0xbfc6b4fee0000000, 0x3fef7f6f81a37251),
    (0xbfcbeb3c80000000, 0x3fef3de7bb6829af),
    (0xbfcd72ec60000000, 0x3fef28251b81e0e7),
    (0xbfcf557ea0000000, 0x3fef0bc4dc1e40bf),
    (0xbfd11a6c60000000, 0x3feedd377c68eb45),
    (0xbfd18a7f00000000, 0x3feece3c0d521e59),
    (0xbfd1cb0c40000000, 0x3feec57007942fc3),
    (0xbfd55aa0a0000000, 0x3fee3c388638ac43),
    (0xbfd56240e0000000, 0x3fee3af8a0f36da1),
    (0xbfd5be9d80000000, 0x3fee2bb429a38bef),
    (0xbfd633eac0000000, 0x3fee17f5c103e259),
    (0xbfd646a260000000, 0x3fee14c5e2fffbb5),
    (0xbfda609660000000, 0x3fed52080d9348fb),
    (0xbfdab43f00000000, 0x3fed412d87dd8b79),
    (0xbfdeb565c0000000, 0x3fec62efe43076fd),
    (0xbfe2004640000000, 0x3feb11aaca371a49),
    (0xbfe5810aa0000000, 0x3fe90ade7ea65f4b),
    (0xbfe7348d20000000, 0x3fe7f2cfa4aee0c3),
    (0xbfe7d5a680000000, 0x3fe786c83962f7e1),
    (0xbfe802db80000000, 0x3fe7680c1009ae95),
    (0xbfe875cac0000000, 0x3fe7191636473c5d),
    (0xbfe883f760000000, 0x3fe70f449dd628d1),
    (0xbfe88b8020000000, 0x3fe70a0ab455458f),
    (0xbfe8c30200000000, 0x3fe6e3627a568c77),
    (0xbfe8f67b60000000, 0x3fe6bf4bc4048d61),
    (0xbfe9217280000000, 0x3fe6a0ff2669e26d),
    (0xbfe9219040000000, 0x3fe6a0ea1d66cb93),
    (0xbfe92195a0000000, 0x3fe6a0e650789687),
    (0xbfe921a5a0000000, 0x3fe6a0db004a849a),
    (0xbfe921b640000000, 0x3fe6a0cf3ef4a7cd),
    (0xbfe921be20000000, 0x3fe6a0c9ad790289),
    (0xbfe921cc40000000, 0x3fe6a0bfb0a3fd6f),
    (0xbfe921e740000000, 0x3fe6a0ac99325fb7),
    (0xbfe921e880000000, 0x3fe6a0abb6ecb945),
    (0xbfe921eca0000000, 0x3fe6a0a8cc39d54d),
    (0xbfe921ed20000000, 0x3fe6a0a871b78495),
    (0xbfe921ee60000000, 0x3fe6a0a78f71b499),
    (0xbfe921f020000000, 0x3fe6a0a652a98293),
    (0xbfe921fc00000000, 0x3fe6a09ded101c53),
    (0xbfe921fe60000000, 0x3fe6a09c3f2441c0),
    (0xbfe92209c0000000, 0x3fe6a0943409c15b),
    (0xbfe9220aa0000000, 0x3fe6a09395a52303),
    (0xbfe9220c20000000, 0x3fe6a092861d2e0d),
    (0xbfe9220e20000000, 0x3fe6a0911c127e4f),
    (0xbfe9221120000000, 0x3fe6a08efd024c45),
    (0xbfe92211e0000000, 0x3fe6a08e753e37ce),
    (0xbfe92217a0000000, 0x3fe6a08a645e8688),
    (0xbfe9221c60000000, 0x3fe6a0870883cb33),
    (0xbfe9221ca0000000, 0x3fe6a086db425f7d),
    (0xbfe9221d60000000, 0x3fe6a086537e1a3b),
    (0xbfe9221dc0000000, 0x3fe6a0860f9bf669),
    (0xbfe9221ee0000000, 0x3fe6a08543f5862d),
    (0xbfe92222c0000000, 0x3fe6a082867f2220),
    (0xbfe92223c0000000, 0x3fe6a081d1794d19),
    (0xbfe9222400000000, 0x3fe6a081a437d6f5),
    (0xbfe92224a0000000, 0x3fe6a08133142e0f),
    (0xbfe9222520000000, 0x3fe6a080d8913ef3),
    (0xbfe9222ba0000000, 0x3fe6a07c3fea99d5),
    (0xbfe9222e20000000, 0x3fe6a07a7b5b7d27),
    (0xbfe9222e60000000, 0x3fe6a07a4e19f856),
    (0xbfe9222f40000000, 0x3fe6a079afb4a4b5),
    (0xbfe9223200000000, 0x3fe6a077bde3cad2),
    (0xbfe92232a0000000, 0x3fe6a0774cbff06d),
    (0xbfe9223420000000, 0x3fe6a0763d36a811),
    (0xbfe9223640000000, 0x3fe6a074bc896bc3),
    (0xbfe9223780000000, 0x3fe6a073da4196b7),
    (0xbfe92237e0000000, 0x3fe6a073965f3b79),
    (0xbfe9223980000000, 0x3fe6a0727034fc3f),
    (0xbfe9223a40000000, 0x3fe6a071e8703c7d),
    (0xbfe9223f20000000, 0x3fe6a06e75f11082),
    (0xbfe9223fc0000000, 0x3fe6a06e04cd07b5),
    (0xbfe92240a0000000, 0x3fe6a06d66675e15),
    (0xbfe92242a0000000, 0x3fe6a06bfc5a5c61),
    (0xbfe92244c0000000, 0x3fe6a06a7bac71c9),
    (0xbfe9224560000000, 0x3fe6a06a0a885519),
    (0xbfe92245e0000000, 0x3fe6a069b005095c),
    (0xbfe9224660000000, 0x3fe6a0695581bc35),
    (0xbfe9224b00000000, 0x3fe6a0661042ef82),
    (0xbfe9224cc0000000, 0x3fe6a064d377288f),
    (0xbfe9224d00000000, 0x3fe6a064a635786f),
    (0xbfe9224d80000000, 0x3fe6a0644bb21721),
    (0xbfe9225380000000, 0x3fe6a0600d89192d),
    (0xbfe92255a0000000, 0x3fe6a05e8cda63bd),
    (0xbfe9225780000000, 0x3fe6a05d396d5413),
    (0xbfe92257c0000000, 0x3fe6a05d0c2b94c1),
    (0xbfe92258e0000000, 0x3fe6a05c4083b36b),
    (0xbfe9225960000000, 0x3fe6a05be6003087),
    (0xbfe9225d60000000, 0x3fe6a05911e3e67f),
    (0xbfe9225ea0000000, 0x3fe6a0582f9afccd),
    (0xbfe9226060000000, 0x3fe6a056f2ce7394),
    (0xbfe9226240000000, 0x3fe6a0559f60f1e7),
    (0xbfe9226260000000, 0x3fe6a05588c00aaf),
    (0xbfe92262a0000000, 0x3fe6a0555b7e3bfb),
    (0xbfe9226420000000, 0x3fe6a0544bf35c57),
    (0xbfe9226a20000000, 0x3fe6a0500dc75e81),
    (0xbfe9226bc0000000, 0x3fe6a04ee79b5161),
    (0xbfe9226e80000000, 0x3fe6a04cf5c6ca5f),
    (0xbfe9227180000000, 0x3fe6a04ad6b034d9),
    (0xbfe9227240000000, 0x3fe6a04a4eea8783),
    (0xbfe9227360000000, 0x3fe6a0498341fd8b),
    (0xbfe9227460000000, 0x3fe6a048ce3a6073),
    (0xbfe9227640000000, 0x3fe6a0477acc0aa7),
    (0xbfe9227960000000, 0x3fe6a0454513fa22),
    (0xbfe9227ea0000000, 0x3fe6a0418eab101b),
    (0xbfe9228080000000, 0x3fe6a0403b3c4d99),
    (0xbfe9228120000000, 0x3fe6a03fca175dad),
    (0xbfe9228680000000, 0x3fe6a03bfd0c8cbd),
    (0xbfe9575f40000000, 0x3fe67abe4b9527b3),
    (0xbfe9bbf040000000, 0x3fe632bd06677391),
    (0xbfea248ce0000000, 0x3fe5e6eda5dc125b),
    (0xbfebd51060000000, 0x3fe4a3ede41fc5eb),
    (0xbfec90ba20000000, 0x3fe41325f5ae9b29),
    (0xbfecc437c0000000, 0x3fe3eaf2e180a001),
    (0xbfed004b40000000, 0x3fe3bbcabbeeb20b),
    (0xbfed40a960000000, 0x3fe388f70e779336),
    (0xbfed927360000000, 0x3fe347efcb3f777f),
    (0xbfef481b00000000, 0x3fe1e3c52f019364),
    (0xbff0a393e0000000, 0x3fe03368ab62ebc3),
    (0xbff113f000000000, 0x3fdee02966e7728f),
    (0xbff16a2760000000, 0x3fddb05aa4375c29),
    (0xbff302f9c0000000, 0x3fd7e43df291fb1f),
    (0xbff5ffe520000000, 0x3fc8e7c359e0fcf4),
    (0xbff77364a0000000, 0x3fbadcbc9b8ec166),
];

// ============================================================================
// Part 11: Elytra: when it starts, stops, and breaks
// ============================================================================

/// Default Mojang 26.2 elytra `MAX_DAMAGE`.
pub const ELYTRA_MAX_DAMAGE: i32 = 432;

/// `EquipmentSlot.values()` declaration order (`@0-80`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EquipmentSlot {
    Mainhand = 0,
    Offhand = 1,
    Feet = 2,
    Legs = 3,
    Chest = 4,
    Head = 5,
    Body = 6,
    Saddle = 7,
}

impl EquipmentSlot {
    pub const VALUES: [Self; 8] = [
        Self::Mainhand,
        Self::Offhand,
        Self::Feet,
        Self::Legs,
        Self::Chest,
        Self::Head,
        Self::Body,
        Self::Saddle,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GliderSlotStack {
    pub has_glider: bool,
    pub equippable_slot: Option<EquipmentSlot>,
    pub damageable: bool,
    pub damage: i32,
    pub max_damage: i32,
}

impl GliderSlotStack {
    pub const fn elytra(damage: i32) -> Self {
        Self {
            has_glider: true,
            equippable_slot: Some(EquipmentSlot::Chest),
            damageable: true,
            damage,
            max_damage: ELYTRA_MAX_DAMAGE,
        }
    }
}

/// `ItemStack.nextDamageWillBreak()` (`@0-25`).
pub const fn next_damage_will_break(damageable: bool, damage: i32, max_damage: i32) -> bool {
    damageable && damage >= max_damage - 1
}

/// `LivingEntity.canGlideUsing(ItemStack, EquipmentSlot)` (`@0-37`).
pub fn can_glide_using(stack: &GliderSlotStack, slot: EquipmentSlot) -> bool {
    stack.has_glider
        && stack.equippable_slot == Some(slot)
        && !next_damage_will_break(stack.damageable, stack.damage, stack.max_damage)
}

/// `LivingEntity.canGlide()` (`@0-68`).
/// Water (`is_in_water`) and `SLOW_FALLING` do NOT block `canGlide`; only `on_ground`,
/// `is_passenger`, `MobEffects.LEVITATION`, and needing >= 1 qualifying non-breaking glider do.
pub fn living_can_glide(
    on_ground: bool,
    is_passenger: bool,
    has_levitation: bool,
    slots: &[Option<GliderSlotStack>; 8],
) -> bool {
    if on_ground || is_passenger || has_levitation {
        return false;
    }
    EquipmentSlot::VALUES.into_iter().any(|slot| {
        slots[slot as usize]
            .as_ref()
            .is_some_and(|stack| can_glide_using(stack, slot))
    })
}

/// `Player.canGlide()` (`@0-16`): creative/spectator `abilities.flying` blocks gliding.
pub fn player_can_glide(
    abilities_flying: bool,
    on_ground: bool,
    is_passenger: bool,
    has_levitation: bool,
    slots: &[Option<GliderSlotStack>; 8],
) -> bool {
    !abilities_flying && living_can_glide(on_ground, is_passenger, has_levitation, slots)
}

/// `Player.tryToStartFallFlying()` (`@0-22`): client+server entry gate (sets flag 7 when true).
/// Note: water (`is_in_water`) blocks starting here, NOT `canGlide()`.
pub const fn try_to_start_fall_flying(
    is_fall_flying: bool,
    can_glide: bool,
    is_in_water: bool,
) -> bool {
    !is_fall_flying && can_glide && !is_in_water
}

/// `LocalPlayer.aiStep` jump-edge gate (`@875-929`) before `tryToStartFallFlying()`.
pub const fn ai_step_start_fall_flying(
    was_jumping: bool,
    jump_pressed: bool,
    consumed_by_mayfly_double_tap: bool,
    on_climbable: bool,
    can_start_fall_flying: bool,
) -> bool {
    !was_jumping
        && jump_pressed
        && !consumed_by_mayfly_double_tap
        && !on_climbable
        && can_start_fall_flying
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FallFlyingUpdateResult {
    pub worn_slot: Option<EquipmentSlot>,
    pub emitted_glide_game_event: bool,
    pub cleared_fall_flying: bool,
}

/// `LivingEntity.updateFallFlying()` (`@0-135`).
/// Called BEFORE `aiStep`'s `++fallFlyTicks` increment; does not mutate `fall_fly_ticks`.
/// On `!can_glide`: clears `flag7_fall_flying` and returns without wear or gameEvent.
/// When `can_glide && is_server_level`:
/// - fires `gameEvent(ELYTRA_GLIDE)` whenever `(fall_fly_ticks + 1) % 10 == 0`;
/// - wears (`+1` damage) the first `EquipmentSlot::VALUES` glider slot passing `can_glide_using`
///   whenever `((fall_fly_ticks + 1) / 10) % 2 == 0` (i.e. `fall_fly_ticks = 19, 39, 59, ...`).
pub fn update_fall_flying(
    flag7_fall_flying: &mut bool,
    fall_fly_ticks: i32,
    abilities_flying: bool,
    on_ground: bool,
    is_passenger: bool,
    has_levitation: bool,
    is_server_level: bool,
    slots: &mut [Option<GliderSlotStack>; 8],
) -> FallFlyingUpdateResult {
    let can_glide = player_can_glide(
        abilities_flying,
        on_ground,
        is_passenger,
        has_levitation,
        slots,
    );
    if !can_glide {
        let was_flying = *flag7_fall_flying;
        *flag7_fall_flying = false;
        return FallFlyingUpdateResult {
            worn_slot: None,
            emitted_glide_game_event: false,
            cleared_fall_flying: was_flying,
        };
    }

    let mut worn_slot = None;
    let mut emitted_glide_game_event = false;
    if *flag7_fall_flying && is_server_level {
        let i = fall_fly_ticks + 1;
        if i % 10 == 0 {
            if (i / 10) % 2 == 0 {
                for slot in EquipmentSlot::VALUES {
                    if let Some(stack) = slots[slot as usize].as_mut() {
                        if can_glide_using(stack, slot) {
                            if stack.damageable {
                                stack.damage += 1;
                            }
                            worn_slot = Some(slot);
                            break;
                        }
                    }
                }
            }
            emitted_glide_game_event = true;
        }
    }

    FallFlyingUpdateResult {
        worn_slot,
        emitted_glide_game_event,
        cleared_fall_flying: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firework_look_and_3_tick_boost_witnesses() {
        let yaw = -30.0f32;
        let pitch = 15.0f32;
        let look = crate::math::calculate_view_vector(yaw, pitch);
        // witness.txt:3
        assert_eq!(look.x.to_bits(), 0x3fde_e88f_e000_0000);
        assert_eq!(look.y.to_bits(), 0xbfd0_8f7a_c000_0000);
        assert_eq!(look.z.to_bits(), 0x3fea_c4fc_4000_0000);

        // check/check-adversarial.txt:4-9
        let look_cases: &[(f32, f32, [u64; 3])] = &[
            (0.0, 0.0, [0x0, 0x8000_0000_0000_0000, 0x3ff0_0000_0000_0000]),
            (90.0, 0.0, [0xbff0_0000_0000_0000, 0x8000_0000_0000_0000, 0x0]),
            (-90.0, 0.0, [0x3ff0_0000_0000_0000, 0x8000_0000_0000_0000, 0x3ca1_a626_4000_0000]),
            (45.0, 180.0, [0x3fe6_a09e_6000_0000, 0xbca1_a626_4000_0000, 0xbfe6_a010_4000_0000]),
            (-89.5, 179.9, [0xbfefffaa_e000_0000, 0xbf5d_d859_6000_0000, 0xbf82_105b_4000_0000]),
            (33.3, -127.7, [0x3fd5_7caf_a000_0000, 0x3fe9_51bd_4000_0000, 0xbfe0_5acd_a000_0000]),
        ];
        for &(y, p, exp) in look_cases {
            let l = crate::math::calculate_view_vector(y, p);
            assert_eq!([l.x.to_bits(), l.y.to_bits(), l.z.to_bits()], exp, "yaw={y} pitch={p}");
        }

        // witness.txt:2,4-6 (ticks 1-3; tick 2 y=0xbfd6b6efb4cccccd differs from folded ...b4cccccc)
        let mut v = Vec3::new(
            f64::from_bits(0x3fd9_9999_9999_999a),
            f64::from_bits(0xbfb9_9999_9999_999a),
            f64::from_bits(0x3fa9_9999_9999_999a),
        );
        let expected_ticks = [
            [0x3fe3_893d_2599_999a, 0xbfd1_46c1_f000_0000, 0x3fe7_8dd6_6999_999a], // witness.txt:4
            [0x3fe6_e775_5200_0000, 0xbfd6_b6ef_b4cc_cccd, 0x3ff1_43fa_68cc_cccd], // witness.txt:5
            [0x3fe8_9691_6833_3333, 0xbfd9_6f06_9733_3333, 0x3ff4_0282_02cc_cccd], // witness.txt:6
        ];
        for (i, exp) in expected_ticks.iter().enumerate() {
            v = firework_boost(v, yaw, pitch);
            assert_eq!([v.x.to_bits(), v.y.to_bits(), v.z.to_bits()], *exp, "boost tick {}", i + 1);
        }
    }

    #[test]
    fn firework_lifetime_boost_count_and_lag_witnesses() {
        // witness.txt:21-24
        assert_eq!(firework_lifetime_bounds(0), (10, 21));
        assert_eq!(firework_boost_ticks_bounds(0), (11, 22));
        assert_eq!(firework_lifetime_bounds(1), (20, 31));
        assert_eq!(firework_boost_ticks_bounds(1), (21, 32));
        assert_eq!(firework_lifetime_bounds(2), (30, 41));
        assert_eq!(firework_boost_ticks_bounds(2), (31, 42));
        assert_eq!(firework_lifetime_bounds(3), (40, 51));
        assert_eq!(firework_boost_ticks_bounds(3), (41, 52));

        // FireworkRocketItem.use requires fall_flying; rocket runs after rider travel and boosts lifetime+1 ticks.
        assert!(AttachedFirework::try_use(false, 20).is_none());
        let mut fw = AttachedFirework::try_use(true, 20).unwrap();
        let mut vel = Vec3::new(0.4, -0.1, 0.05);
        // Simulate tick N: rider travelFallFlying sees pre-boost vel, then rocket ticks after rider.
        let mut boosts = 0;
        loop {
            let rider_seen_vel_before_rocket = vel;
            let step = fw.tick(true, &mut vel, -30.0, 15.0);
            if step.boosted {
                boosts += 1;
                assert_ne!(vel, rider_seen_vel_before_rocket);
            }
            if step.exploded {
                break;
            }
        }
        assert_eq!(boosts, 21); // lifetime 20 -> 21 boost ticks
    }

    #[test]
    fn fly_into_wall_damage_witnesses() {
        // witness.txt:9-18 & check/check-adversarial.txt:18-27
        assert_eq!(fly_into_wall_damage(false, 1.0, 0.0), None);
        assert_eq!(fly_into_wall_damage(true, 0.25, 0.0), None); // witness.txt:9
        assert_eq!(fly_into_wall_damage(true, 0.29, 0.0), None); // witness.txt:10
        assert_eq!(
            fly_into_wall_damage(true, f64::from_bits(0x3fd3_3333_3333_3332), 0.0),
            None
        );
        assert_eq!(
            fly_into_wall_damage(true, f64::from_bits(0x3fd3_3333_3333_3333), 0.0), // d5 = 0.3, witness.txt:12,18
            None
        );
        assert_eq!(
            fly_into_wall_damage(true, f64::from_bits(0x3fd3_3333_3333_3334), 0.0)
                .map(f32::to_bits), // witness.txt:13,18
            Some(0x2600_0000)
        );
        assert_eq!(
            fly_into_wall_damage(true, 0.31, 0.0).map(f32::to_bits), // witness.txt:14
            Some(0x3dcc_cccd)
        );
        assert_eq!(
            fly_into_wall_damage(true, 0.35, 0.0).map(f32::to_bits), // witness.txt:15
            Some(0x3f00_0000)
        );
        assert_eq!(
            fly_into_wall_damage(true, 0.5, 0.0).map(f32::to_bits), // witness.txt:16
            Some(0x4000_0000)
        );
        assert_eq!(
            fly_into_wall_damage(true, 1.0, 0.0).map(f32::to_bits), // witness.txt:17
            Some(0x40e0_0000)
        );
        assert_eq!(fly_into_wall_damage(true, -0.0, 0.0), None); // check-adversarial.txt:26
        assert!(fly_into_wall_damage(true, f64::NAN, 0.0).is_some_and(f32::is_nan)); // check-adversarial.txt:27
    }

    #[test]
    fn part11_elytra_start_and_stop_witnesses() {
        // evidence/probe-output.txt:10-25 & check-evidence/check-output.txt:4-16
        assert!(!next_damage_will_break(true, 429, ELYTRA_MAX_DAMAGE));
        assert!(!next_damage_will_break(true, 430, ELYTRA_MAX_DAMAGE));
        assert!(next_damage_will_break(true, 431, ELYTRA_MAX_DAMAGE));
        assert!(next_damage_will_break(true, 432, ELYTRA_MAX_DAMAGE));
        // Undamageable glider does not break even at max_damage-1 (check-output.txt:13-15)
        assert!(!next_damage_will_break(false, 431, ELYTRA_MAX_DAMAGE));

        // canGlideUsing checks stack.has_glider && slot match && !next_damage_will_break
        assert!(can_glide_using(&GliderSlotStack::elytra(430), EquipmentSlot::Chest));
        assert!(!can_glide_using(&GliderSlotStack::elytra(431), EquipmentSlot::Chest));
        for wrong in [
            EquipmentSlot::Mainhand,
            EquipmentSlot::Offhand,
            EquipmentSlot::Feet,
            EquipmentSlot::Legs,
            EquipmentSlot::Head,
            EquipmentSlot::Body,
            EquipmentSlot::Saddle,
        ] {
            assert!(!can_glide_using(&GliderSlotStack::elytra(0), wrong), "slot={wrong:?}");
        }
        // Head-equippable glider works only from Head, not Chest (check-output.txt:10-12)
        let head_glider = GliderSlotStack {
            has_glider: true,
            equippable_slot: Some(EquipmentSlot::Head),
            damageable: false,
            damage: 431,
            max_damage: ELYTRA_MAX_DAMAGE,
        };
        assert!(can_glide_using(&head_glider, EquipmentSlot::Head));
        assert!(!can_glide_using(&head_glider, EquipmentSlot::Chest));

        // Slot helper
        let slots_with = |slot: EquipmentSlot, stack: GliderSlotStack| -> [Option<GliderSlotStack>; 8] {
            let mut s = [None; 8];
            s[slot as usize] = Some(stack);
            s
        };

        // G1..G8 & A4-A5 (probe-output.txt:27-48 & check-output.txt:17-20)
        let good_slots = slots_with(EquipmentSlot::Chest, GliderSlotStack::elytra(0));
        // canGlide: true in air even in water (G6) and with slow falling (A5); false on_ground / passenger / levitation / creative flying
        assert!(living_can_glide(false, false, false, &good_slots));
        assert!(!living_can_glide(true, false, false, &good_slots));
        assert!(!living_can_glide(false, true, false, &good_slots));
        assert!(!living_can_glide(false, false, true, &good_slots));
        assert!(!player_can_glide(true, false, false, false, &good_slots));
        assert!(player_can_glide(false, false, false, false, &good_slots));

        // tryToStartFallFlying blocks in water even when canGlide is true (probe-output.txt:36-40)
        let cg = player_can_glide(false, false, false, false, &good_slots);
        assert!(try_to_start_fall_flying(false, cg, false));
        assert!(!try_to_start_fall_flying(false, cg, true));
        assert!(!try_to_start_fall_flying(true, cg, false));

        // LocalPlayer jump-edge gate requires !was_jumping && jump && !double_tap && !on_climbable
        assert!(ai_step_start_fall_flying(false, true, false, false, true));
        assert!(!ai_step_start_fall_flying(true, true, false, false, true));
        assert!(!ai_step_start_fall_flying(false, true, true, false, true));
        assert!(!ai_step_start_fall_flying(false, true, false, true, true));

        // 65-tick durability & ELYTRA_GLIDE cadence (probe-output.txt:52-79):
        // gameEvent at fallFlyTicks=9,19,29,39,49,59; wear at fallFlyTicks=19,39,59
        let mut slots = slots_with(EquipmentSlot::Chest, GliderSlotStack::elytra(0));
        let mut flag7 = true;
        let mut worn_at = Vec::new();
        let mut events_at = Vec::new();
        for t in 0..65 {
            let step = update_fall_flying(&mut flag7, t, false, false, false, false, true, &mut slots);
            if step.worn_slot.is_some() {
                worn_at.push(t);
            }
            if step.emitted_glide_game_event {
                events_at.push(t);
            }
        }
        assert_eq!(worn_at, vec![19, 39, 59]);
        assert_eq!(events_at, vec![9, 19, 29, 39, 49, 59]);
        assert_eq!(slots[EquipmentSlot::Chest as usize].unwrap().damage, 3);
        assert!(flag7);

        // Near-break boundary starting at dmg=429 (check-output.txt:22-77):
        // wears to 430 at t=19, wears to 431 at t=39 (flag7 still true on that tick!),
        // then at t=40 canGlide is false -> flag7 clears with NO wear or gameEvent.
        let mut near_break = slots_with(EquipmentSlot::Chest, GliderSlotStack::elytra(429));
        let mut fly = true;
        for t in 0..=40 {
            let res = update_fall_flying(&mut fly, t, false, false, false, false, true, &mut near_break);
            if t < 19 {
                assert_eq!(near_break[EquipmentSlot::Chest as usize].unwrap().damage, 429);
                assert!(fly);
            } else if t < 39 {
                assert_eq!(near_break[EquipmentSlot::Chest as usize].unwrap().damage, 430);
                assert!(fly);
            } else if t == 39 {
                assert_eq!(near_break[EquipmentSlot::Chest as usize].unwrap().damage, 431);
                assert_eq!(res.worn_slot, Some(EquipmentSlot::Chest));
                assert!(fly);
            } else {
                // t == 40
                assert!(!fly);
                assert!(res.cleared_fall_flying);
                assert_eq!(res.worn_slot, None);
                assert!(!res.emitted_glide_game_event);
            }
        }
    }
}
