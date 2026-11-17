// Generator for "III. The Hollow Meridian" (Minecraft Java 26.2)
// Deterministic, stdlib-only, writes `build.txt` into the folder given as argv[1].

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const X_MIN: i32 = 2300;
const X_MAX: i32 = 2749;
const Y_MIN: i32 = -61;
const Y_MAX: i32 = 170;
const Z_MIN: i32 = -700;
const Z_MAX: i32 = -260;

const NX: usize = (X_MAX - X_MIN + 1) as usize; // 450
const NY: usize = (Y_MAX - Y_MIN + 1) as usize; // 232
const NZ: usize = (Z_MAX - Z_MIN + 1) as usize; // 441

const HX: i32 = 2525;
const HZ: i32 = -490;

const MAX_FILL: i64 = 32768;

struct World {
    palette: Vec<String>,
    map: HashMap<String, u16>,
    grid: Vec<u16>,
    out: BufWriter<File>,
    emit: bool,
    fill_cmds: usize,
    setblock_cmds: usize,
    non_air_placed: u64,
    air_placed: u64,
}

impl World {
    fn new(out: BufWriter<File>) -> Self {
        let mut palette = Vec::with_capacity(128);
        let mut map = HashMap::with_capacity(128);
        let air = "minecraft:air".to_string();
        map.insert(air.clone(), 0);
        palette.push(air);
        World {
            palette,
            map,
            grid: vec![0u16; NX * NY * NZ],
            out,
            emit: false,
            fill_cmds: 0,
            setblock_cmds: 0,
            non_air_placed: 0,
            air_placed: 0,
        }
    }

    #[inline]
    fn idx(x: i32, y: i32, z: i32) -> usize {
        debug_assert!(x >= X_MIN && x <= X_MAX);
        debug_assert!(y >= Y_MIN && y <= Y_MAX);
        debug_assert!(z >= Z_MIN && z <= Z_MAX);
        (((y - Y_MIN) as usize) * NZ + ((z - Z_MIN) as usize)) * NX + ((x - X_MIN) as usize)
    }

    #[inline]
    fn get_id(&self, x: i32, y: i32, z: i32) -> u16 {
        if x < X_MIN || x > X_MAX || y < Y_MIN || y > Y_MAX || z < Z_MIN || z > Z_MAX { return 0; }
        self.grid[Self::idx(x, y, z)]
    }

    #[inline]
    fn get_name(&self, id: u16) -> &str {
        &self.palette[id as usize]
    }

    fn intern(&mut self, block: &str) -> u16 {
        if let Some(&id) = self.map.get(block) {
            return id;
        }
        let id = self.palette.len() as u16;
        let s = block.to_string();
        self.map.insert(s.clone(), id);
        self.palette.push(s);
        id
    }

    fn comment(&mut self, line: &str) {
        if self.emit {
            writeln!(self.out, "# {}", line).expect("write comment");
        }
    }

    fn blank(&mut self) {
        if self.emit {
            writeln!(self.out).expect("write blank");
        }
    }

    #[inline]
    fn check_bounds(x: i32, y: i32, z: i32) {
        if x < X_MIN || x > X_MAX || y < Y_MIN || y > Y_MAX || z < Z_MIN || z > Z_MAX {
            panic!("Coordinate out of box: ({}, {}, {})", x, y, z);
        }
    }

    fn set_grid_only(&mut self, x: i32, y: i32, z: i32, block: &str) {
        Self::check_bounds(x, y, z);
        let id = self.intern(block);
        let i = Self::idx(x, y, z);
        self.grid[i] = id;
    }

    fn set(&mut self, x: i32, y: i32, z: i32, block: &str) {
        Self::check_bounds(x, y, z);
        let id = self.intern(block);
        let i = Self::idx(x, y, z);
        self.grid[i] = id;
        if self.emit {
            writeln!(self.out, "/setblock {} {} {} {}", x, y, z, block).expect("write setblock");
            self.setblock_cmds += 1;
            if id == 0 {
                self.air_placed += 1;
            } else {
                self.non_air_placed += 1;
            }
        }
    }

    fn fill(&mut self, x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32, block: &str) {
        let (lx, hx) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (ly, hy) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        let (lz, hz) = if z1 <= z2 { (z1, z2) } else { (z2, z1) };
        Self::check_bounds(lx, ly, lz);
        Self::check_bounds(hx, hy, hz);
        let id = self.intern(block);
        for y in ly..=hy {
            for z in lz..=hz {
                let row_start = Self::idx(lx, y, z);
                let len = (hx - lx + 1) as usize;
                for i in row_start..row_start + len {
                    self.grid[i] = id;
                }
            }
        }
        if self.emit {
            self.emit_fill_split(lx, ly, lz, hx, hy, hz, id, block);
        }
    }

    fn emit_fill_split(
        &mut self,
        lx: i32,
        ly: i32,
        lz: i32,
        hx: i32,
        hy: i32,
        hz: i32,
        id: u16,
        block: &str,
    ) {
        let dx = (hx - lx + 1) as i64;
        let dy = (hy - ly + 1) as i64;
        let dz = (hz - lz + 1) as i64;
        let vol = dx * dy * dz;
        if vol <= MAX_FILL {
            if vol == 1 {
                writeln!(self.out, "/setblock {} {} {} {}", lx, ly, lz, block)
                    .expect("write setblock");
                self.setblock_cmds += 1;
            } else {
                writeln!(
                    self.out,
                    "/fill {} {} {} {} {} {} {}",
                    lx, ly, lz, hx, hy, hz, block
                )
                .expect("write fill");
                self.fill_cmds += 1;
            }
            if id == 0 {
                self.air_placed += vol as u64;
            } else {
                self.non_air_placed += vol as u64;
            }
            return;
        }
        if dx >= dy && dx >= dz && dx > 1 {
            let mid = lx + (hx - lx) / 2;
            self.emit_fill_split(lx, ly, lz, mid, hy, hz, id, block);
            self.emit_fill_split(mid + 1, ly, lz, hx, hy, hz, id, block);
        } else if dy >= dz && dy > 1 {
            let mid = ly + (hy - ly) / 2;
            self.emit_fill_split(lx, ly, lz, hx, mid, hz, id, block);
            self.emit_fill_split(lx, mid + 1, lz, hx, hy, hz, id, block);
        } else {
            let mid = lz + (hz - lz) / 2;
            self.emit_fill_split(lx, ly, lz, hx, hy, mid, id, block);
            self.emit_fill_split(lx, ly, mid + 1, hx, hy, hz, id, block);
        }
    }

    // Emits greedy 2D horizontal slabs for all non-air blocks currently in grid.
    fn emit_landscape_from_grid(&mut self) {
        let mut visited = vec![false; NX * NZ];
        for y in Y_MIN..=Y_MAX {
            visited.fill(false);
            for z in Z_MIN..=Z_MAX {
                let zr = (z - Z_MIN) as usize;
                let mut x = X_MIN;
                while x <= X_MAX {
                    let xr = (x - X_MIN) as usize;
                    let cell = zr * NX + xr;
                    if visited[cell] {
                        x += 1;
                        continue;
                    }
                    let id = self.get_id(x, y, z);
                    if id == 0 {
                        visited[cell] = true;
                        x += 1;
                        continue;
                    }
                    let mut x2 = x;
                    while x2 + 1 <= X_MAX {
                        let c2 = zr * NX + ((x2 + 1 - X_MIN) as usize);
                        if !visited[c2] && self.get_id(x2 + 1, y, z) == id {
                            x2 += 1;
                        } else {
                            break;
                        }
                    }
                    let width = (x2 - x + 1) as i64;
                    let mut z2 = z;
                    'outer: while z2 + 1 <= Z_MAX {
                        let depth = (z2 + 1 - z + 1) as i64;
                        if width * depth > MAX_FILL {
                            break;
                        }
                        let zr_next = (z2 + 1 - Z_MIN) as usize;
                        for tx in x..=x2 {
                            let c_next = zr_next * NX + ((tx - X_MIN) as usize);
                            if visited[c_next] || self.get_id(tx, y, z2 + 1) != id {
                                break 'outer;
                            }
                        }
                        z2 += 1;
                    }
                    for tz in z..=z2 {
                        let z_off = (tz - Z_MIN) as usize * NX;
                        for tx in x..=x2 {
                            visited[z_off + (tx - X_MIN) as usize] = true;
                        }
                    }
                    let blk = self.palette[id as usize].clone();
                    let vol = ((x2 - x + 1) as i64) * ((z2 - z + 1) as i64);
                    if vol == 1 {
                        writeln!(self.out, "/setblock {} {} {} {}", x, y, z, blk)
                            .expect("write setblock");
                        self.setblock_cmds += 1;
                    } else {
                        writeln!(
                            self.out,
                            "/fill {} {} {} {} {} {} {}",
                            x, y, z, x2, y, z2, blk
                        )
                        .expect("write fill");
                        self.fill_cmds += 1;
                    }
                    self.non_air_placed += vol as u64;
                    x = x2 + 1;
                }
            }
        }
    }
}

// ---------- Deterministic Noise & Geological Functions ----------

#[inline]
fn hash2(x: i32, z: i32, seed: u64) -> f64 {
    let mut h = (x as i64 as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (z as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ seed.wrapping_mul(0x94D0_49BB_1331_11EB);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    ((h >> 11) as f64) / ((1u64 << 53) as f64)
}

#[inline]
fn hash3(x: i32, y: i32, z: i32, seed: u64) -> f64 {
    let mut h = (x as i64 as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as i64 as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (z as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ seed;
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    ((h >> 11) as f64) / ((1u64 << 53) as f64)
}

#[inline]
fn smoothstep(t: f64) -> f64 {
    let c = t.clamp(0.0, 1.0);
    c * c * c * (c * (c * 6.0 - 15.0) + 10.0)
}

fn value_noise2(x: f64, z: f64, seed: u64) -> f64 {
    let ix = x.floor() as i32;
    let iz = z.floor() as i32;
    let fx = smoothstep(x - ix as f64);
    let fz = smoothstep(z - iz as f64);
    let v00 = hash2(ix, iz, seed);
    let v10 = hash2(ix + 1, iz, seed);
    let v01 = hash2(ix, iz + 1, seed);
    let v11 = hash2(ix + 1, iz + 1, seed);
    let a = v00 + (v10 - v00) * fx;
    let b = v01 + (v11 - v01) * fx;
    a + (b - a) * fz
}

fn fbm2(x: f64, z: f64, seed: u64, octaves: i32) -> f64 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for i in 0..octaves {
        sum += amp * value_noise2(x * freq, z * freq, seed.wrapping_add(i as u64 * 1013));
        norm += amp;
        amp *= 0.5;
        freq *= 2.03;
    }
    sum / norm
}

fn ridged2(x: f64, z: f64, seed: u64, octaves: i32) -> f64 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for i in 0..octaves {
        let n = value_noise2(x * freq, z * freq, seed.wrapping_add(i as u64 * 2029));
        let r = 1.0 - (2.0 * n - 1.0).abs();
        sum += amp * r * r;
        norm += amp;
        amp *= 0.5;
        freq *= 2.07;
    }
    sum / norm
}

fn hollow_radius(theta: f64, y: i32) -> f64 {
    let base = 42.5
        + 3.6 * (3.0 * theta + 0.4).cos()
        + 2.8 * (5.0 * theta - 0.7).sin()
        + 1.9 * (2.0 * theta + 1.1).cos();
    // Stepped ancient quarry benches every 14 blocks of height
    let bench_phase = ((y + 48).rem_euclid(14)) as f64;
    let bench = if y > -46 && y < 86 {
        if bench_phase < 4.0 {
            1.6
        } else {
            0.0
        }
    } else {
        0.0
    };
    // Natural widening near upper crater rim
    let flare = if y > 86 {
        ((y - 86) as f64 / 70.0).clamp(0.0, 1.0) * 5.5
    } else {
        0.0
    };
    base + bench + flare
}

fn is_in_hollow(x: i32, y: i32, z: i32) -> bool {
    if y < -47 {
        return false;
    }
    let dx = (x - HX) as f64;
    let dz = (z - HZ) as f64;
    let dist = dx.hypot(dz);
    let theta = dz.atan2(dx);
    let r = hollow_radius(theta, y);
    if dist <= r {
        return true;
    }
    false
}

fn mountain_top_y(x: i32, z: i32) -> i32 {
    // Smooth border fade to plain (y = -61) at least 18 blocks inside every box edge
    let bx = ((x - X_MIN).min(X_MAX - x) as f64 / 30.0).clamp(0.0, 1.0);
    let bz_n = ((z - Z_MIN) as f64 / 30.0).clamp(0.0, 1.0);
    let bz_s = ((-282 - z) as f64 / 36.0).clamp(0.0, 1.0);
    let edge_fade = smoothstep(bx) * smoothstep(bz_n) * smoothstep(bz_s);
    if edge_fade <= 0.001 {
        return Y_MIN;
    }

    let fx = x as f64;
    let fz = z as f64;
    let warp_x = (fbm2(fx * 0.014, fz * 0.014, 11, 3) - 0.5) * 26.0;
    let warp_z = (fbm2(fx * 0.014, fz * 0.014, 29, 3) - 0.5) * 26.0;

    let dx = (fx + warp_x - HX as f64) / 145.0;
    let dz = (fz + warp_z - (HZ - 6) as f64) / 138.0;
    let radial = dx.hypot(dz);
    let massif = (1.0 - (radial / 1.15).powf(1.55)).clamp(0.0, 1.0);

    // Twin northern massif peaks (NW & NE shoulders)
    let nw = (-(((fx - 2450.0) / 72.0).powi(2) + ((fz + 558.0) / 62.0).powi(2))).exp();
    let ne = (-(((fx - 2604.0) / 74.0).powi(2) + ((fz + 548.0) / 64.0).powi(2))).exp();
    // Protective southern front ridge hiding the hollow from the valley
    let s_ridge = (-(((fx - 2545.0) / 78.0).powi(2) + ((fz + 422.0) / 28.0).powi(2))).exp();

    let ridge_tex = ridged2(fx * 0.021, fz * 0.021, 77, 4);
    let fine_tex = fbm2(fx * 0.055, fz * 0.055, 131, 3);

    let mut h = -61.0
        + edge_fade
            * (massif * (168.0 + 36.0 * ridge_tex + 10.0 * fine_tex)
                + nw * 52.0
                + ne * 46.0
                + s_ridge * 40.0);

    // Carve natural West Saddle pass near (2452, -432) where the quarry road enters
    let d_saddle = (((fx - 2450.0) / 24.0).powi(2) + ((fz + 432.0) / 22.0).powi(2)).sqrt();
    if d_saddle < 1.35 {
        let k = smoothstep((1.35 - d_saddle) / 1.35);
        let target = 30.0 + 4.0 * fine_tex;
        if h > target {
            h = h * (1.0 - 0.85 * k) + target * (0.85 * k);
        }
    }

    // Ravine/gully outside West tunnel slits (x=2436..2452, z=-472..-445) so daylight reaches slits
    let d_gully = (((fx - 2436.0) / 16.0).powi(2) + ((fz + 458.0) / 20.0).powi(2)).sqrt();
    if d_gully < 1.2 {
        let k = smoothstep((1.2 - d_gully) / 1.2);
        let target = 16.0;
        if h > target {
            h = h * (1.0 - 0.9 * k) + target * (0.9 * k);
        }
    }

    // South slope lower gully at (2468, -424) so middle daylight slit sees sky
    let d_gully2 = (((fx - 2468.0) / 12.0).powi(2) + ((fz + 422.0) / 14.0).powi(2)).sqrt();
    if d_gully2 < 1.2 {
        let k = smoothstep((1.2 - d_gully2) / 1.2);
        let target = 20.0;
        if h > target {
            h = h * (1.0 - 0.9 * k) + target * (0.9 * k);
        }
    }

    // Ensure rim immediately around hollow stays substantial on South (except saddle) & high North
    let dist_h = ((fx - HX as f64).powi(2) + (fz - HZ as f64).powi(2)).sqrt();
    if dist_h > 40.0 && dist_h < 68.0 {
        let theta = (fz - HZ as f64).atan2(fx - HX as f64);
        let rim_floor = if theta > 0.15 && theta < 2.95 {
            // South wall of hollow
            82.0 + 26.0 * (theta - 1.5).sin().abs() + 14.0 * ridge_tex
        } else {
            132.0 + 24.0 * ridge_tex
        };
        // Respect saddle on SW (around x=2452, z=-432) only outside r=58
        if dist_h < 58.0 && h < rim_floor {
            h = rim_floor;
        }
    }

    (h.round() as i32).clamp(Y_MIN, 165)
}

fn strata_block(x: i32, y: i32, z: i32, top_y: i32, in_hollow_wall: bool) -> &'static str {
    let dip = (x - HX) as f64 * 0.08 + (z - HZ) as f64 * 0.11;
    let warp = (fbm2(x as f64 * 0.06, z as f64 * 0.06, 401, 2) - 0.5) * 8.0;
    let sy = y as f64 + dip + warp;

    let n = hash3(x, y, z, 907);

    // Surface dressing on gentle low foothills
    if y == top_y && !in_hollow_wall {
        if top_y <= -57 {
            return if n < 0.12 {
                "minecraft:coarse_dirt"
            } else {
                "minecraft:grass_block"
            };
        }
        if top_y <= 6 && n < 0.45 {
            return if n < 0.14 {
                "minecraft:moss_block"
            } else if n < 0.28 {
                "minecraft:coarse_dirt"
            } else {
                "minecraft:grass_block"
            };
        }
    }

    if sy < -18.0 {
        if in_hollow_wall && n < 0.14 {
            "minecraft:cobbled_deepslate"
        } else {
            "minecraft:deepslate"
        }
    } else if sy < 56.0 {
        // Rich tuff beds inside stone & andesite (especially exposed along quarry pipe)
        let dist_h = (((x - HX) as f64).powi(2) + ((z - HZ) as f64).powi(2)).sqrt();
        let band = ((sy + 18.0) * 0.16).sin();
        if dist_h < 62.0 && (band > -0.25 || n < 0.42) {
            "minecraft:tuff"
        } else if band > 0.25 {
            "minecraft:tuff"
        } else if n < 0.48 {
            "minecraft:andesite"
        } else {
            "minecraft:stone"
        }
    } else if n < 0.58 {
        "minecraft:andesite"
    } else {
        "minecraft:stone"
    }
}

fn place_spruce_tree(w: &mut World, tx: i32, ground_y: i32, tz: i32, seed: u64) {
    let h = 6 + (hash2(tx, tz, seed) * 4.0) as i32;
    if ground_y + h + 2 > Y_MAX {
        return;
    }
    for dy in 1..=h {
        w.set_grid_only(tx, ground_y + dy, tz, "minecraft:spruce_log[axis=y]");
    }
    for layer in 2..=h + 1 {
        let r = if layer == h + 1 {
            0
        } else if layer >= h - 1 {
            1
        } else if (layer % 2) == 0 {
            2
        } else {
            1
        };
        let ly = ground_y + layer;
        for dz in -(r as i32)..=(r as i32) {
            for dx in -(r as i32)..=(r as i32) {
                if dx == 0 && dz == 0 && layer <= h {
                    continue;
                }
                if r == 2 && dx.abs() == 2 && dz.abs() == 2 {
                    continue;
                }
                let px = tx + dx;
                let pz = tz + dz;
                if px >= X_MIN && px <= X_MAX && pz >= Z_MIN && pz <= Z_MAX {
                    if w.get_id(px, ly, pz) == 0 {
                        w.set_grid_only(px, ly, pz, "minecraft:spruce_leaves[persistent=true]");
                    }
                }
            }
        }
    }
}

fn build_landscape_into_grid(w: &mut World) {
    let mut tops = vec![Y_MIN; NX * NZ];
    for z in Z_MIN..=Z_MAX {
        for x in X_MIN..=X_MAX {
            let ty = mountain_top_y(x, z);
            tops[(z - Z_MIN) as usize * NX + (x - X_MIN) as usize] = ty;
        }
    }

    for z in Z_MIN..=Z_MAX {
        for x in X_MIN..=X_MAX {
            let ty = tops[(z - Z_MIN) as usize * NX + (x - X_MIN) as usize];
            for y in Y_MIN..=ty {
                if is_in_hollow(x, y, z) {
                    continue;
                }
                let dx = (x - HX) as f64;
                let dz = (z - HZ) as f64;
                let dist_h = dx.hypot(dz);
                let in_wall = dist_h < 60.0 && y >= -48;
                let blk = strata_block(x, y, z, ty, in_wall);
                w.set_grid_only(x, y, z, blk);
            }
        }
    }

    // Guarantee dark deepslate basin floor below hollow (-61..-48)
    for z in (HZ - 52)..=(HZ + 52) {
        for x in (HX - 52)..=(HX + 52) {
            let d = ((x - HX) as f64).hypot((z - HZ) as f64);
            if d <= 45.0 {
                for y in Y_MIN..=-48 {
                    if w.get_id(x, y, z) == 0 || y >= -50 {
                        let b = if y == -48 && hash3(x, y, z, 33) < 0.18 {
                            "minecraft:cobbled_deepslate"
                        } else {
                            "minecraft:deepslate"
                        };
                        w.set_grid_only(x, y, z, b);
                    }
                }
            }
        }
    }

    // Supported gravel talus & quarry spoil heaps on south lower apron (z = -355..-290)
    for z in -355..=-290 {
        for x in 2480..=2575 {
            let n = fbm2(x as f64 * 0.08, z as f64 * 0.08, 709, 2);
            let ty = tops[(z - Z_MIN) as usize * NX + (x - X_MIN) as usize];
            if ty > -59 && ty < -28 && (x - 2525).abs() > 8 && n > 0.63 {
                // Only place gravel directly on top of solid rock
                if w.get_id(x, ty, z) != 0 && w.get_id(x, ty + 1, z) == 0 {
                    w.set_grid_only(x, ty, z, "minecraft:gravel");
                }
            }
        }
    }

    // Sparse alpine spruce trees in sheltered lower foothill hollows (away from road & spoil)
    let tree_candidates = [
        (2375, -360),
        (2390, -345),
        (2365, -405),
        (2388, -420),
        (2355, -470),
        (2372, -520),
        (2402, -595),
        (2435, -625),
        (2640, -345),
        (2662, -370),
        (2675, -415),
        (2660, -475),
        (2650, -545),
        (2620, -610),
        (2570, -635),
        (2495, -630),
        (2460, -320),
        (2442, -335),
        (2595, -305),
        (2618, -320),
    ];
    for (i, &(tx, tz)) in tree_candidates.iter().enumerate() {
        let ty = tops[(tz - Z_MIN) as usize * NX + (tx - X_MIN) as usize];
        if ty >= -61 && ty <= -18 && !is_in_hollow(tx, ty, tz) {
            w.set_grid_only(tx, ty, tz, "minecraft:grass_block");
            place_spruce_tree(w, tx, ty, tz, 500 + i as u64 * 17);
        }
    }
}

// ---------- Architectural Helpers ----------

fn stair_dir_str(dx: i32, dz: i32) -> &'static str {
    if dx > 0 {
        "east"
    } else if dx < 0 {
        "west"
    } else if dz > 0 {
        "south"
    } else {
        "north"
    }
}

// Builds a continuous, 0-jump axis-aligned stepped path with headroom, foundation, and optional parapets.
fn build_axis_path(
    w: &mut World,
    pts: &[(i32, i32, i32)],
    half_w: i32,
    headroom: i32,
    road_mat: &str,
    stair_mat: &str,
    curb_mat: &str,
    parapet: Option<&str>,
) {
    let mut steps: Vec<(i32, i32, i32, i32, i32, bool)> = Vec::new(); // (x, y, z, dir_x, dir_z, is_corner)
    for seg in 0..pts.len() - 1 {
        let (x0, y0, z0) = pts[seg];
        let (x1, y1, z1) = pts[seg + 1];
        let dx_tot = x1 - x0;
        let dz_tot = z1 - z0;
        assert!(
            dx_tot == 0 || dz_tot == 0,
            "Segment must be axis-aligned: {:?} -> {:?}",
            pts[seg],
            pts[seg + 1]
        );
        let len = dx_tot.abs().max(dz_tot.abs());
        let sx = dx_tot.signum();
        let sz = dz_tot.signum();
        for i in 0..=len {
            if seg > 0 && i == 0 {
                continue;
            }
            let cx = x0 + sx * i;
            let cz = z0 + sz * i;
            // Keep first and last step of sloped segments flat so corners never carry stairs
            let cy = if len <= 2 || y1 == y0 {
                y0
            } else if i == 0 {
                y0
            } else if i == len {
                y1
            } else {
                y0 + ((y1 - y0) * (i - 1)) / (len - 2)
            };
            let is_corner = i == 0 || i == len;
            steps.push((cx, cy, cz, sx, sz, is_corner));
        }
    }

    // Ensure every corner waypoint has a full (2*half_w+1)^2 flat landing & headroom
    for &(wx, wy, wz) in pts {
        w.fill(
            wx - half_w,
            wy + 1,
            wz - half_w,
            wx + half_w,
            wy + headroom,
            wz + half_w,
            "minecraft:air",
        );
        for oz in (wz - half_w - 1)..=(wz + half_w + 1) {
            for ox in (wx - half_w - 1)..=(wx + half_w + 1) {
                let is_curb = (ox - wx).abs() > half_w || (oz - wz).abs() > half_w;
                let base = (wy - 12).max(Y_MIN);
                for fy in (base..=wy).rev() {
                    if fy < wy && w.get_id(ox, fy, oz) != 0 {
                        break;
                    }
                    let mat = if fy == wy && !is_curb {
                        road_mat
                    } else if fy >= wy - 2 {
                        curb_mat
                    } else {
                        "minecraft:tuff_bricks"
                    };
                    w.set(ox, fy, oz, mat);
                }
            }
        }
    }

    for i in 0..steps.len() {
        let (cx, cy, cz, sx, sz, is_corner) = steps[i];
        let prev_y = if i > 0 { steps[i - 1].1 } else { cy };
        let px = -sz; // perpendicular
        let pz = sx;

        // Clear generous headroom across width
        for off in -half_w..=half_w {
            let bx = cx + px * off;
            let bz = cz + pz * off;
            w.fill(bx, cy + 1, bz, bx, cy + headroom, bz, "minecraft:air");
        }

        // Solid stone support underneath down to terrain or max 12 blocks retaining wall
        for off in -(half_w + 1)..=(half_w + 1) {
            let bx = cx + px * off;
            let bz = cz + pz * off;
            let base = (cy - 12).max(Y_MIN);
            for fy in (base..=cy).rev() {
                if fy < cy && w.get_id(bx, fy, bz) != 0 {
                    break;
                }
                let mat = if fy == cy && off.abs() <= half_w {
                    road_mat
                } else if fy == cy {
                    curb_mat
                } else if fy >= cy - 2 {
                    curb_mat
                } else {
                    "minecraft:tuff_bricks"
                };
                w.set(bx, fy, bz, mat);
            }
        }

        // Place stair on step transitions (never on corner waypoints)
        if !is_corner && cy > prev_y {
            let f = stair_dir_str(sx, sz);
            let st = format!("{}[facing={},half=bottom,shape=straight]", stair_mat, f);
            for off in -half_w..=half_w {
                w.set(cx + px * off, cy, cz + pz * off, &st);
            }
        } else if cy < prev_y && i > 0 && !steps[i - 1].5 {
            let f = stair_dir_str(-sx, -sz);
            let st = format!("{}[facing={},half=bottom,shape=straight]", stair_mat, f);
            let (px_prev, py_prev, pz_prev, _, _, _) = steps[i - 1];
            for off in -half_w..=half_w {
                w.set(px_prev + px * off, py_prev, pz_prev + pz * off, &st);
            }
        }

        // Optional safety parapet along outer edges where drop > 2 blocks
        if let Some(wall_mat) = parapet {
            for &side in &[-(half_w + 1), half_w + 1] {
                let wx = cx + px * side;
                let wz = cz + pz * side;
                let ox = wx + px * side.signum();
                let oz = wz + pz * side.signum();
                if ox >= X_MIN
                    && ox <= X_MAX
                    && oz >= Z_MIN
                    && oz <= Z_MAX
                    && w.get_id(ox, cy - 1, oz) == 0
                {
                    w.set(wx, cy + 1, wz, wall_mat);
                }
            }
        }
    }
}

// ---------- PART 2: QUARRY ROAD, SPOIL TERRACE & RETAINING WALLS ----------

fn build_quarry_road(w: &mut World) {
    w.comment("============================================================");
    w.comment("PART 2: STRUCTURE — QUARRY HAUL ROAD, SPOIL TERRACE & RETAINING WALLS");
    w.comment("============================================================");

    let road_pts = [
        (2525, -61, -263),
        (2525, -61, -280),
        (2525, -45, -328),
        (2525, -45, -334),
        (2609, -12, -334),
        (2609, -6, -366),
        (2452, 22, -366),
        (2452, 28, -423),
    ];
    build_axis_path(
        w,
        &road_pts,
        2,
        5,
        "minecraft:tuff",
        "minecraft:tuff_brick_stairs",
        "minecraft:tuff_bricks",
        Some("minecraft:tuff_brick_wall"),
    );

    // Worn cart wheel tracks & gravel patches along flat plain approach (supported on -61)
    for z in (-282..=-260).step_by(2) {
        w.set(2524, -61, z, "minecraft:coarse_dirt");
        w.set(2526, -61, z, "minecraft:coarse_dirt");
    }

    // Abandoned Quarry Spoil Terrace & Crane Base Plinth at (2514..2521, y=-45, z=-336..-328)
    w.fill(2514, -52, -336, 2521, -45, -328, "minecraft:tuff_bricks");
    w.fill(2515, -45, -335, 2520, -45, -329, "minecraft:polished_tuff");
    w.fill(2515, -44, -335, 2520, -40, -329, "minecraft:air");
    // Ruined timber crane stump
    w.fill(2517, -44, -332, 2517, -40, -332, "minecraft:spruce_log[axis=y]");
    w.set(2518, -41, -332, "minecraft:spruce_log[axis=x]");
    w.set(2519, -41, -332, "minecraft:spruce_log[axis=x]");
    w.set(2519, -42, -332, "minecraft:iron_chain[axis=y]");
    // Cut tuff blocks waiting on loading terrace
    w.fill(2515, -44, -330, 2516, -44, -329, "minecraft:tuff_bricks");
    w.set(2515, -43, -330, "minecraft:chiseled_tuff_bricks");

    // Roadway marker lanterns at switchbacks & milestones
    let road_posts = [
        (2522, -61, -264),
        (2522, -45, -331),
        (2612, -12, -334),
        (2612, -6, -366),
        (2530, 8, -363),
        (2449, 22, -366),
        (2449, 28, -421),
    ];
    for &(px, py, pz) in &road_posts {
        w.fill(px, (py - 4).max(Y_MIN), pz, px, py, pz, "minecraft:tuff_bricks");
        w.set(px, py + 1, pz, "minecraft:spruce_fence");
        w.set(px, py + 2, pz, "minecraft:spruce_fence");
        w.set(px, py + 3, pz, "minecraft:lantern[hanging=false]");
    }
    // Weathered chiseled tuff milestone at East Switchback
    w.set(2611, -5, -368, "minecraft:chiseled_tuff_bricks");
    w.set(2611, -4, -368, "minecraft:polished_tuff_slab[type=bottom]");
}

// ---------- PART 3: GATEHOUSE, WINCH HOUSE & ENTRY PASSAGE ----------

fn build_gatehouse_and_tunnel(w: &mut World) {
    w.comment("============================================================");
    w.comment("PART 3: STRUCTURE — QUARRY GATEHOUSE & RESTRAINED ENTRY PASSAGE");
    w.comment("============================================================");

    // Gatehouse Footprint: x 2444..2462, z -441..-423, floor y = 28 (feet y = 29)
    // Solid masonry terrace foundation
    w.fill(2444, 20, -441, 2462, 28, -423, "minecraft:tuff_bricks");
    w.fill(2446, 28, -439, 2460, 28, -425, "minecraft:polished_tuff");
    // Clear interior
    w.fill(2445, 29, -440, 2461, 35, -424, "minecraft:air");

    // Lower rough tuff & tuff_bricks walls (y=29..31)
    w.fill(2444, 29, -441, 2462, 31, -441, "minecraft:tuff_bricks");
    w.fill(2444, 29, -423, 2462, 31, -423, "minecraft:tuff_bricks");
    w.fill(2444, 29, -440, 2444, 31, -424, "minecraft:tuff_bricks");
    w.fill(2462, 29, -440, 2462, 31, -424, "minecraft:tuff_bricks");

    // Upper spruce timber walls (y=32..33)
    w.fill(2444, 32, -441, 2462, 33, -441, "minecraft:spruce_planks");
    w.fill(2444, 32, -423, 2462, 33, -423, "minecraft:spruce_planks");
    w.fill(2444, 32, -440, 2444, 33, -424, "minecraft:spruce_planks");
    w.fill(2462, 32, -440, 2462, 33, -424, "minecraft:spruce_planks");

    // Corner & bay spruce structural posts
    for &px in &[2444, 2450, 2456, 2462] {
        for &pz in &[-441, -432, -423] {
            if (px == 2450 || px == 2456) && pz == -432 {
                continue;
            }
            w.fill(px, 29, pz, px, 33, pz, "minecraft:spruce_log[axis=y]");
        }
    }

    // Pitched spruce roof (ridge running N-S at x=2453, eaves overhanging by 1)
    for step in 0..=9 {
        let rx_w = 2443 + step;
        let rx_e = 2463 - step;
        let ry = 33 + step / 2;
        let half = if step % 2 == 0 { "bottom" } else { "top" };
        if rx_w < rx_e {
            let st_w = format!("minecraft:spruce_stairs[facing=east,half={},shape=straight]", half);
            let st_e = format!("minecraft:spruce_stairs[facing=west,half={},shape=straight]", half);
            if step % 2 == 0 {
                w.fill(rx_w, ry, -442, rx_w, ry, -422, &st_w);
                w.fill(rx_e, ry, -442, rx_e, ry, -422, &st_e);
            } else {
                w.fill(rx_w, ry, -442, rx_w, ry, -422, "minecraft:spruce_planks");
                w.fill(rx_e, ry, -442, rx_e, ry, -422, "minecraft:spruce_planks");
            }
        }
    }
    w.fill(2453, 38, -442, 2453, 38, -422, "minecraft:spruce_slab[type=bottom]");
    // Gable end fill
    for ry in 34..=37 {
        let inset = (ry - 33) * 2;
        w.fill(2444 + inset, ry, -441, 2462 - inset, ry, -441, "minecraft:spruce_planks");
        w.fill(2444 + inset, ry, -423, 2462 - inset, ry, -423, "minecraft:spruce_planks");
    }

    // South entrance portal (3 wide, 3 tall) facing road at x=2451..2453, z=-423
    w.fill(2451, 29, -423, 2453, 31, -423, "minecraft:air");
    w.fill(2450, 29, -422, 2450, 31, -422, "minecraft:tuff_brick_wall");
    w.fill(2454, 29, -422, 2454, 31, -422, "minecraft:tuff_brick_wall");
    w.fill(2451, 32, -422, 2453, 32, -422, "minecraft:spruce_slab[type=top]");
    w.set(2450, 32, -422, "minecraft:lantern[hanging=false]");
    w.set(2454, 32, -422, "minecraft:lantern[hanging=false]");

    // Small guarded industrial windows on West & East walls
    w.fill(2444, 30, -436, 2444, 31, -435, "minecraft:iron_bars");
    w.fill(2444, 30, -429, 2444, 31, -428, "minecraft:iron_bars");
    w.fill(2462, 30, -436, 2462, 31, -435, "minecraft:iron_bars");

    // Cart rail leading from South door to Winch Pit (no bypass staircase!)
    for rz in -431..=-423 {
        w.set(2452, 29, rz, "minecraft:rail");
    }
    // Capped Hoist Pit (x=2450..2454, z=-437..-433) with dark abyss glimpse under bars/trapdoors
    w.fill(2451, 14, -436, 2453, 28, -434, "minecraft:deepslate");
    w.fill(2451, 16, -436, 2453, 28, -434, "minecraft:air");
    w.fill(2451, 28, -436, 2453, 28, -434, "minecraft:iron_bars");
    w.fill(2450, 29, -437, 2454, 29, -437, "minecraft:spruce_fence");
    w.fill(2450, 29, -433, 2450, 29, -436, "minecraft:spruce_fence");
    w.fill(2454, 29, -433, 2454, 29, -436, "minecraft:spruce_fence");
    // Overhead winch axle
    w.fill(2450, 32, -435, 2454, 32, -435, "minecraft:stripped_spruce_log[axis=x]");
    w.fill(2452, 29, -435, 2452, 31, -435, "minecraft:iron_chain[axis=y]");

    // Gatehouse interior fittings (ordinary quarry office/smithy traces)
    w.set(2446, 29, -439, "minecraft:furnace[facing=east,lit=false]");
    w.set(2446, 29, -438, "minecraft:crafting_table");
    w.set(2446, 29, -436, "minecraft:barrel[facing=up,open=false]");
    w.set(2446, 29, -435, "minecraft:barrel[facing=up,open=false]");
    w.set(2447, 29, -439, "minecraft:chest[facing=south,type=single]");
    w.set(2456, 29, -427, "minecraft:lectern[facing=west,has_book=false,powered=false]");
    w.fill(2445, 34, -432, 2461, 34, -432, "minecraft:spruce_log[axis=x]");
    w.set(2459, 29, -430, "minecraft:lantern[hanging=false]");
    w.set(2447, 33, -432, "minecraft:lantern[hanging=true]");
    w.set(2459, 33, -432, "minecraft:lantern[hanging=true]");

    // Rear doorway to Entry Passage at x=2457..2459, z=-441
    w.fill(2457, 29, -441, 2459, 32, -441, "minecraft:air");

    // Long, Restrained Entry Passage (3 blocks wide, 4 blocks tall, 56 blocks long)
    let tunnel_pts = [
        (2458, 28, -441),
        (2458, 29, -455),
        (2478, 30, -455),
        (2478, 30, -472),
        (2483, 30, -472),
    ];
    build_axis_path(
        w,
        &tunnel_pts,
        1,
        4,
        "minecraft:polished_tuff",
        "minecraft:tuff_brick_stairs",
        "minecraft:tuff_bricks",
        None,
    );

    // Timber mine-props & dim spaced lanterns inside tunnel
    for tz in [-445, -450].iter() {
        w.fill(2457, 30, *tz, 2457, 33, *tz, "minecraft:spruce_log[axis=y]");
        w.fill(2459, 30, *tz, 2459, 33, *tz, "minecraft:spruce_log[axis=y]");
        w.set(2458, 33, *tz, "minecraft:spruce_log[axis=x]");
    }
    for tx in [2464, 2472].iter() {
        w.fill(*tx, 30, -456, *tx, 33, -456, "minecraft:spruce_log[axis=y]");
        w.fill(*tx, 30, -454, *tx, 33, -454, "minecraft:spruce_log[axis=y]");
        w.set(*tx, 33, -455, "minecraft:spruce_log[axis=z]");
    }
    for tz in [-460, -467].iter() {
        w.fill(2477, 31, *tz, 2477, 34, *tz, "minecraft:spruce_log[axis=y]");
        w.fill(2479, 31, *tz, 2479, 34, *tz, "minecraft:spruce_log[axis=y]");
        w.set(2478, 34, *tz, "minecraft:spruce_log[axis=x]");
    }
    w.set(2458, 32, -445, "minecraft:lantern[hanging=true]");
    w.set(2472, 32, -455, "minecraft:lantern[hanging=true]");
    w.set(2478, 33, -467, "minecraft:lantern[hanging=true]");

    // 3 Narrow Daylight Slits cut out to outer cliff air
    w.fill(2434, 30, -449, 2456, 32, -449, "minecraft:air");
    w.fill(2468, 30, -453, 2468, 32, -420, "minecraft:air");
    w.fill(2434, 31, -464, 2476, 33, -464, "minecraft:air");
}

// ---------- PART 4: ARRIVAL GALLERY, WEST GALLERY, LIBRARY & CROSS-BRIDGE ----------

fn build_west_side_and_bridge(w: &mut World) {
    w.comment("============================================================");
    w.comment("PART 4: STRUCTURE — ARRIVAL GALLERY, WEST GALLERY, CURVED LIBRARY & CROSS-BRIDGE");
    w.comment("============================================================");

    // Arrival Gallery (x=2483..2488, z=-476..-468, floor y=30)
    w.fill(2483, 28, -476, 2488, 30, -468, "minecraft:tuff_bricks");
    w.fill(2483, 27, -475, 2486, 27, -469, "minecraft:tuff_bricks");
    w.fill(2483, 26, -474, 2484, 26, -470, "minecraft:tuff_bricks");
    w.fill(2484, 30, -475, 2487, 30, -469, "minecraft:polished_tuff");
    w.fill(2483, 31, -476, 2488, 36, -468, "minecraft:air");
    // Quiet low parapet on South & East void edges
    w.fill(2488, 31, -475, 2488, 31, -468, "minecraft:tuff_brick_wall");
    w.fill(2483, 31, -468, 2487, 31, -468, "minecraft:tuff_brick_wall");
    w.set(2488, 31, -476, "minecraft:chiseled_tuff_bricks");

    // West Cliff Gallery: (2484, 30, -476) -> (2484, 20, -510)
    let west_pts = [(2484, 30, -476), (2484, 20, -510)];
    build_axis_path(
        w,
        &west_pts,
        1,
        5,
        "minecraft:polished_tuff",
        "minecraft:tuff_brick_stairs",
        "minecraft:tuff_bricks",
        Some("minecraft:tuff_brick_wall"),
    );

    // Curved Rock Library (x=2473..2485, z=-534..-510, floor y=20, ceiling y=25)
    for z in -534..=-510 {
        let dz = (z + 522) as f64;
        // Concave rock curve along back West wall
        let west_wall_x = 2474 + ((dz * dz) / 56.0).round() as i32;
        w.fill(west_wall_x, 19, z, 2485, 20, z, "minecraft:tuff_bricks");
        w.fill(west_wall_x + 1, 20, z, 2483, 20, z, "minecraft:spruce_planks");
        w.fill(west_wall_x + 1, 21, z, 2485, 25, z, "minecraft:air");
        w.fill(west_wall_x, 26, z, 2485, 26, z, "minecraft:tuff_bricks");
        // Back wall bookshelves & curved rock ribs
        if z % 5 == 0 {
            w.fill(west_wall_x, 21, z, west_wall_x, 25, z, "minecraft:spruce_log[axis=y]");
        } else {
            w.fill(west_wall_x, 21, z, west_wall_x, 23, z, "minecraft:bookshelf");
            w.set(west_wall_x, 24, z, "minecraft:chiseled_bookshelf[facing=east]");
        }
        // Void-facing arched tuff piers (x=2485); keep south entrance z=-513..-510 open!
        if z == -534 || z == -526 || z == -518 {
            w.fill(2485, 21, z, 2485, 25, z, "minecraft:tuff_bricks");
        } else if z <= -514 {
            w.set(2485, 21, z, "minecraft:tuff_brick_wall");
        }
    }
    // Library end walls & fittings
    w.fill(2476, 21, -534, 2484, 25, -534, "minecraft:tuff_bricks");
    w.fill(2477, 21, -533, 2483, 23, -533, "minecraft:bookshelf");
    w.set(2479, 21, -522, "minecraft:cartography_table");
    w.set(2483, 21, -522, "minecraft:polished_tuff_slab[type=bottom]"); // reading window seat
    w.set(2483, 21, -521, "minecraft:polished_tuff_slab[type=bottom]");
    w.set(2480, 21, -515, "minecraft:lectern[facing=east,has_book=false,powered=false]");
    w.set(2478, 25, -527, "minecraft:lantern[hanging=true]");
    w.set(2478, 25, -516, "minecraft:lantern[hanging=true]");

    // Transition stair from Library SE exit (2485, 20, -510) -> Bridge West Abutment (2485, 16, -502)
    let lib_to_bridge = [(2485, 20, -510), (2485, 16, -502)];
    build_axis_path(
        w,
        &lib_to_bridge,
        1,
        5,
        "minecraft:polished_tuff",
        "minecraft:tuff_brick_stairs",
        "minecraft:tuff_bricks",
        Some("minecraft:tuff_brick_wall"),
    );

    // THE GREAT MERIDIAN CROSS-BRIDGE (x=2485..2566, z=-504..-500, walkable z=-503..-501, floor=16)
    // Clear generous airspace above & below deck
    w.fill(2486, 17, -504, 2565, 24, -500, "minecraft:air");
    // Deck & Railings
    for bx in 2486..=2565 {
        // Deck
        w.fill(bx, 16, -503, bx, 16, -501, "minecraft:polished_tuff");
        w.fill(bx, 16, -504, bx, 16, -504, "minecraft:tuff_bricks");
        w.fill(bx, 16, -500, bx, 16, -500, "minecraft:tuff_bricks");
        // Slender oxidized copper railings
        w.set(bx, 17, -504, "minecraft:waxed_oxidized_copper_bars");
        w.set(bx, 17, -500, "minecraft:waxed_oxidized_copper_bars");

        // West Ancient Masonry Arch & Pier (x=2486..2505)
        if bx <= 2504 {
            let arch_drop = ((bx - 2486) as f64 / 18.0).powi(2) * 8.0;
            let base_y = (4 + arch_drop.round() as i32).min(15);
            w.fill(bx, base_y, -503, bx, 15, -501, "minecraft:tuff_bricks");
            if bx == 2488 || bx == 2489 {
                w.fill(bx, 2, -503, bx, 15, -501, "minecraft:tuff_bricks");
            }
        }
        // Mid-span slender stone & deepslate rib (x=2505..2546)
        else if bx <= 2546 {
            w.fill(bx, 15, -503, bx, 15, -501, "minecraft:polished_deepslate");
            if (bx - HX).abs() % 6 == 0 {
                w.set(bx, 14, -504, "minecraft:tuff_brick_wall");
                w.set(bx, 14, -500, "minecraft:tuff_brick_wall");
            }
        }
        // East Newer Copper Truss Reinforcement (x=2547..2565)
        else {
            w.fill(bx, 15, -503, bx, 15, -501, "minecraft:waxed_weathered_cut_copper");
            let strut_drop = ((2565 - bx) as f64 / 18.0).powi(2) * 8.0;
            let base_y = (7 + strut_drop.round() as i32).min(14);
            w.fill(bx, base_y, -504, bx, 14, -504, "minecraft:waxed_oxidized_copper_bars");
            w.fill(bx, base_y, -500, bx, 14, -500, "minecraft:waxed_oxidized_copper_bars");
            if bx >= 2561 {
                w.fill(bx, 6, -503, bx, 15, -501, "minecraft:waxed_weathered_cut_copper");
            }
        }
    }
    // Bridge Oculus & Gold Meridian Calibration Strip at x=2524..2526
    w.fill(2524, 15, -503, 2526, 16, -501, "minecraft:calcite");
    w.set(2525, 16, -503, "minecraft:gold_block");
    w.set(2525, 16, -501, "minecraft:gold_block");
    w.set(2525, 16, -502, "minecraft:waxed_oxidized_copper_grate");
    w.set(2525, 15, -502, "minecraft:air");
    // Restrained bridge lanterns at piers & center
    w.set(2488, 18, -504, "minecraft:waxed_oxidized_copper_lantern[hanging=false]");
    w.set(2525, 18, -504, "minecraft:waxed_exposed_copper_lantern[hanging=false]");
    w.set(2562, 18, -504, "minecraft:waxed_oxidized_copper_lantern[hanging=false]");
}

// ---------- PART 5: INSTRUMENT CHAMBER, DAYLIGHT DIAL & MERIDIAN SLITS ----------

fn build_instrument_and_slits(w: &mut World) {
    w.comment("============================================================");
    w.comment("PART 5: INSTRUMENT — SOLAR DIAL CHAMBER, CALIBRATED REDSTONE & SKY SLITS");
    w.comment("============================================================");

    // Instrument Chamber (x=2566..2586, z=-508..-490, floor y=16, ceiling y=23)
    w.fill(2566, 14, -509, 2586, 16, -489, "minecraft:tuff_bricks");
    w.fill(2566, 16, -508, 2585, 16, -490, "minecraft:polished_deepslate");
    w.fill(2566, 17, -508, 2585, 23, -490, "minecraft:air");
    w.fill(2567, 24, -509, 2586, 24, -489, "minecraft:tuff_bricks");

    // Open Sky Light-Well above the 7 daylight detectors (x=2572..2584, z=-505..-491, y=20..170)
    // Line the perimeter of the sky chimney cleanly through the mountain
    for sy in 24..=165 {
        for sx in 2571..=2585 {
            for sz in -506..=-490 {
                let on_rim = sx == 2571 || sx == 2585 || sz == -506 || sz == -490;
                if on_rim {
                    if w.get_id(sx, sy, sz) != 0 {
                        w.set(sx, sy, sz, "minecraft:tuff_bricks");
                    }
                } else {
                    if w.get_id(sx, sy, sz) != 0 {
                        w.set(sx, sy, sz, "minecraft:air");
                    }
                }
            }
        }
    }

    // West Public Instrument Wall facing across the void (x=2568, y=17..21, z=-506..-491)
    w.fill(2568, 17, -506, 2568, 21, -491, "minecraft:polished_deepslate");
    // Open entrance archway from Cross-Bridge (z=-503..-501)? Wait: shift bridge landing!
    // Bridge lands at z=-503..-501 on x=2566 -> step onto North Chamber Entry at z=-508..-506!
    w.fill(2566, 17, -508, 2568, 20, -506, "minecraft:air");
    // Smooth walkway from Bridge (x=2566, z=-503..-501) into Chamber Entry (z=-506..-508) and South Descent
    w.fill(2566, 16, -508, 2567, 16, -490, "minecraft:polished_tuff");
    for gz in -508..=-491 {
        if gz < -503 || gz > -501 {
            w.set(2566, 17, gz, "minecraft:waxed_oxidized_copper_bars");
        }
    }

    // Instrument Bench behind West Display Wall (x=2569..2584, y=17..18, z=-505..-491)
    w.fill(2569, 17, -505, 2584, 18, -491, "minecraft:polished_tuff");
    // Divider fins between channels so signals stay isolated & readable
    for z in [-505, -503, -501, -499, -497, -495, -493, -491].iter() {
        w.fill(2569, 19, *z, 2584, 19, *z, "minecraft:polished_deepslate");
    }

    // 7 Calibrated Redstone Channels + Daylight Detectors (y=19)
    // Lamp wall at x=2568, y=19, z in {-504, -502, -500, -498, -496, -494, -492}
    let channels: [(i32, i32, bool); 7] = [
        (-504, 3, false),  // Dawn / Low Sun (power >= 3)
        (-502, 7, false),  // Mid-Morning (power >= 7)
        (-500, 11, false), // Late Morning (power >= 11)
        (-498, 14, false), // SOLAR NOON / ZENITH (power >= 14)
        (-496, 11, false), // Early Afternoon (power >= 11)
        (-494, 7, false),  // Late Afternoon (power >= 7)
        (-492, 3, true),   // NIGHT / STARLIGHT (inverted >= 3)
    ];

    for &(cz, wire_len, inverted) in &channels {
        // Lamp visible from across the void at x=2568, y=19
        w.set(2568, 19, cz, "minecraft:redstone_lamp");
        // Calcite scale tick right below each lamp on the void face
        w.set(2568, 18, cz, "minecraft:calcite");
        // Gold calibration frame ONLY at Noon Zenith (-498), oxidized copper at Night (-492)
        if cz == -498 {
            w.set(2568, 20, cz, "minecraft:gold_block");
            w.set(2568, 18, cz, "minecraft:gold_block");
        } else if cz == -492 {
            w.set(2568, 20, cz, "minecraft:waxed_oxidized_copper");
        }

        let x_det = 2569 + wire_len;
        // Wire from x=2569 .. x_det-1 pointing West into lamp at x=2568
        for wx in 2569..x_det {
            w.set(
                wx,
                19,
                cz,
                "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]",
            );
        }
        if inverted {
            w.set(x_det, 19, cz, "minecraft:daylight_detector[inverted=true]");
        } else {
            w.set(x_det, 19, cz, "minecraft:daylight_detector[inverted=false]");
        }
        // Ensure unobstructed vertical air column above detector all the way to sky
        w.fill(x_det, 20, cz, x_det, Y_MAX, cz, "minecraft:air");
        // Mark calibrated wire length on the inspection floor at y=18
        w.set(x_det, 18, cz + 1, "minecraft:calcite");
    }

    // Raised Inspection Catwalk along North edge of Instrument Chamber (z=-508..-506, x=2569..2584, floor=19)
    // Lets visitors stand right beside the channels and understand how the solar dial works!
    w.set(2569, 17, -507, "minecraft:polished_tuff_stairs[facing=east,half=bottom,shape=straight]");
    w.set(2570, 18, -507, "minecraft:polished_tuff_stairs[facing=east,half=bottom,shape=straight]");
    w.set(2571, 19, -507, "minecraft:polished_tuff_stairs[facing=east,half=bottom,shape=straight]");
    w.fill(2572, 19, -508, 2584, 19, -506, "minecraft:polished_tuff");
    w.fill(2572, 20, -506, 2584, 20, -506, "minecraft:waxed_oxidized_copper_bars");
    w.set(2574, 20, -507, "minecraft:lectern[facing=south,has_book=false,powered=false]");
    w.set(2576, 20, -507, "minecraft:cartography_table");
    w.set(2570, 23, -507, "minecraft:waxed_exposed_copper_lantern[hanging=true]");

    // ----- THREE ALIGNED ASTRONOMICAL SKY SLITS -----
    // 1. The Great North Meridian Cut (x=2524..2526, z=-588..-524, y=82..170)
    //    Precision vertical notch visible from valley plain AND from the lowest basin!
    for sz in -588..=-524 {
        for sy in 82..=165 {
            if w.get_id(2523, sy, sz) != 0 {
                w.set(2523, sy, sz, "minecraft:tuff_bricks");
            }
            if w.get_id(2527, sy, sz) != 0 {
                w.set(2527, sy, sz, "minecraft:tuff_bricks");
            }
            w.fill(2524, sy, sz, 2526, sy, sz, "minecraft:air");
        }
        w.fill(2524, 81, sz, 2526, 81, sz, "minecraft:polished_tuff");
    }
    w.set(2525, 81, -525, "minecraft:gold_block");

    // 2. East Dawn Horizon Slit (z=-491..-489, x=2565..2655, y=54..80)
    w.fill(2565, 54, -491, 2655, 80, -489, "minecraft:air");
    w.fill(2565, 53, -491, 2635, 53, -489, "minecraft:polished_tuff");

    // 3. West Dusk Horizon Slit (z=-491..-489, x=2395..2485, y=54..80)
    w.fill(2395, 54, -491, 2485, 80, -489, "minecraft:air");
    w.fill(2415, 53, -491, 2485, 53, -489, "minecraft:polished_tuff");
}

// ---------- PART 6: ASTRONOMER'S RESIDENCE & SHELTERED POCKET GARDEN ----------

fn build_residence_and_garden(w: &mut World) {
    w.comment("============================================================");
    w.comment("PART 6: HUMAN SCALE — ASTRONOMER RESIDENCE (THE LIT WINDOW) & POCKET GARDEN");
    w.comment("============================================================");

    // Branch staircase from Instrument Chamber SE (2571, 16, -490) up to Residence Porch (2577, 25, -479)
    // Run tucked at x=2579 behind the room so the iconic West wall stays pristine
    let to_res = [
        (2571, 16, -490),
        (2579, 16, -490),
        (2579, 25, -479),
        (2577, 25, -479),
    ];
    build_axis_path(
        w,
        &to_res,
        1,
        4,
        "minecraft:spruce_planks",
        "minecraft:spruce_stairs",
        "minecraft:tuff_bricks",
        None,
    );

    // Astronomer's Timber Residence (x=2567..2577, z=-488..-477, floor y=25, ceiling y=30)
    w.fill(2566, 23, -489, 2577, 25, -477, "minecraft:tuff_bricks");
    w.fill(2568, 25, -487, 2576, 25, -478, "minecraft:spruce_planks");
    w.fill(2567, 26, -488, 2577, 31, -477, "minecraft:air");

    // Walls (spruce logs at corners, warm spruce planks)
    w.fill(2567, 26, -488, 2577, 29, -488, "minecraft:spruce_planks");
    w.fill(2567, 26, -477, 2577, 29, -477, "minecraft:spruce_planks");
    w.fill(2567, 26, -487, 2567, 29, -478, "minecraft:spruce_planks");
    w.fill(2577, 26, -487, 2577, 29, -478, "minecraft:spruce_planks");
    for &px in &[2567, 2577] {
        for &pz in &[-488, -477] {
            w.fill(px, 26, pz, px, 29, pz, "minecraft:spruce_log[axis=y]");
        }
    }
    // Clear interior space
    w.fill(2568, 26, -487, 2576, 29, -478, "minecraft:air");
    // East staircase entry door into room (aligned with to_res arrival at z=-479)
    w.fill(2577, 26, -480, 2577, 28, -478, "minecraft:air");
    // Pitched roof over residence
    w.fill(2566, 30, -489, 2578, 30, -476, "minecraft:spruce_slab[type=bottom]");
    w.fill(2568, 30, -488, 2576, 30, -477, "minecraft:spruce_planks");
    w.fill(2570, 31, -487, 2574, 31, -478, "minecraft:spruce_slab[type=bottom]");

    // THE SINGLE ILLUMINATED WINDOW visible from the Arrival Gallery 80 blocks across the void!
    // West wall at x=2567, y=27..28, z=-486..-485
    w.fill(2567, 27, -486, 2567, 28, -485, "minecraft:glass_pane");
    w.set(2566, 27, -487, "minecraft:spruce_trapdoor[facing=west,half=bottom,open=true]");
    w.set(2566, 28, -487, "minecraft:spruce_trapdoor[facing=west,half=bottom,open=true]");
    w.set(2566, 27, -484, "minecraft:spruce_trapdoor[facing=west,half=bottom,open=true]");
    w.set(2566, 28, -484, "minecraft:spruce_trapdoor[facing=west,half=bottom,open=true]");
    // Warm hanging lantern directly behind the window
    w.set(2569, 29, -485, "minecraft:lantern[hanging=true]");
    w.set(2568, 26, -486, "minecraft:spruce_slab[type=top]");
    w.set(2568, 26, -485, "minecraft:spruce_slab[type=top]");
    w.set(2568, 27, -486, "minecraft:flower_pot"); // tea cup beside window
    w.set(2568, 26, -484, "minecraft:lectern[facing=west,has_book=false,powered=false]");

    // Cozy Residence Interior
    w.set(2575, 26, -486, "minecraft:brown_bed[facing=north,occupied=false,part=foot]");
    w.set(2575, 26, -487, "minecraft:brown_bed[facing=north,occupied=false,part=head]");
    w.set(2572, 26, -487, "minecraft:bookshelf");
    w.set(2573, 26, -487, "minecraft:bookshelf");
    w.set(2572, 27, -487, "minecraft:potted_fern");
    w.set(2575, 26, -478, "minecraft:furnace[facing=north,lit=false]");
    w.set(2574, 26, -478, "minecraft:smoker[facing=north,lit=false]");
    w.set(2573, 26, -478, "minecraft:crafting_table");
    w.set(2569, 26, -478, "minecraft:barrel[facing=up,open=false]");
    w.set(2570, 26, -478, "minecraft:chest[facing=north,type=single]");
    w.set(2573, 29, -481, "minecraft:lantern[hanging=true]");

    // South door out to Sheltered Pocket Garden
    w.set(2572, 26, -477, "minecraft:spruce_door[facing=south,half=lower,hinge=left,open=false,powered=false]");
    w.set(2572, 27, -477, "minecraft:spruce_door[facing=south,half=upper,hinge=left,open=false,powered=false]");

    // Carve open sky chimney & terrace for the Sheltered Pocket Garden BEFORE placing the access stairs
    w.fill(2569, 28, -475, 2583, 29, -461, "minecraft:tuff");
    w.fill(2569, 30, -475, 2583, Y_MAX, -461, "minecraft:air");
    w.fill(2570, 29, -474, 2582, 29, -462, "minecraft:grass_block");
    w.fill(2570, 29, -474, 2573, 29, -470, "minecraft:tuff_bricks"); // stone terrace landing
    w.fill(2571, 29, -469, 2574, 29, -466, "minecraft:moss_block");

    // Step up to Sheltered Pocket Garden (x=2572, z=-476 -> -470, floor y=25 -> 29)
    let to_garden = [(2572, 25, -476), (2572, 29, -470)];
    build_axis_path(
        w,
        &to_garden,
        1,
        5,
        "minecraft:tuff_bricks",
        "minecraft:tuff_brick_stairs",
        "minecraft:tuff_bricks",
        None,
    );

    // Rainwater basin (3x3 solid tuff_bricks floor & rim, center water)
    w.fill(2577, 28, -473, 2581, 29, -469, "minecraft:tuff_bricks");
    w.fill(2578, 29, -472, 2580, 29, -470, "minecraft:water[level=0]");
    // Stone garden bench & flora
    w.set(2572, 30, -464, "minecraft:polished_tuff_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2573, 30, -464, "minecraft:polished_tuff_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2575, 30, -466, "minecraft:flowering_azalea");
    w.set(2576, 30, -468, "minecraft:azalea");
    w.set(2571, 30, -467, "minecraft:fern");
    w.set(2574, 30, -463, "minecraft:cornflower");
    w.set(2576, 30, -464, "minecraft:lily_of_the_valley");
    w.set(2571, 30, -470, "minecraft:lantern[hanging=false]");
    // Sheltered dwarf mountain spruce tree in garden corner
    place_spruce_tree(w, 2580, 29, -464, 912);
}

// ---------- PART 7: VISIBLE DESCENT GALLERIES & LOWEST PUBLIC CHAMBER ----------

fn build_descent_and_basin(w: &mut World) {
    w.comment("============================================================");
    w.comment("PART 7: DESCENT RIBBON, HOIST ALCOVE, LOWEST CHAMBER & MERIDIAN BASIN");
    w.comment("============================================================");

    // Continuous visible descent ribbon hugging the East & South-East cliff faces (y=16 -> -44)
    let descent_pts = [
        (2566, 16, -490),
        (2566, 4, -465),
        (2558, 4, -465),
        (2558, -4, -453),
        (2542, -18, -453),
        (2542, -26, -443),
        (2522, -44, -443),
    ];
    build_axis_path(
        w,
        &descent_pts,
        1,
        5,
        "minecraft:polished_tuff",
        "minecraft:tuff_brick_stairs",
        "minecraft:tuff_bricks",
        Some("minecraft:waxed_oxidized_copper_bars"),
    );

    // Timber & tuff corbel brackets under cantilevered descent flights + post-mounted lanterns
    for &(cx, cy, cz, lx, lz) in &[
        (2566, 10, -477, 2564, -477),
        (2566, 4, -465, 2564, -463),
        (2558, -4, -453, 2556, -451),
        (2542, -18, -453, 2540, -451),
        (2532, -35, -443, 2532, -445),
    ] {
        w.fill(cx - 1, cy - 3, cz - 1, cx + 1, cy - 1, cz + 1, "minecraft:tuff_bricks");
        w.set(lx, cy, lz, "minecraft:tuff_bricks");
        w.set(lx, cy + 1, lz, "minecraft:tuff_brick_wall");
        w.set(lx, cy + 2, lz, "minecraft:waxed_oxidized_copper_lantern[hanging=false]");
    }

    // Machinery / Hoist Overlook Alcove off SE landing at (2558, -4, -453)
    w.fill(2560, -4, -456, 2566, -4, -450, "minecraft:tuff_bricks");
    w.fill(2560, -3, -455, 2565, 2, -451, "minecraft:air");
    w.fill(2561, -3, -453, 2564, 2, -453, "minecraft:air");
    w.fill(2563, -3, -455, 2563, 2, -455, "minecraft:spruce_log[axis=y]");
    w.fill(2563, -3, -451, 2563, 2, -451, "minecraft:spruce_log[axis=y]");
    w.fill(2563, 2, -455, 2563, 2, -451, "minecraft:stripped_spruce_log[axis=z]");
    w.fill(2563, -2, -453, 2563, 1, -453, "minecraft:iron_chain[axis=y]");
    w.set(2564, -3, -452, "minecraft:barrel[facing=up,open=false]");
    w.set(2563, 1, -454, "minecraft:lantern[hanging=true]");

    // LOWEST PUBLIC CHAMBER (South Base Vaulted Hall: x=2510..2540, z=-446..-434, floor=-44)
    w.fill(2510, -49, -446, 2540, -44, -434, "minecraft:deepslate_bricks");
    w.fill(2512, -44, -445, 2538, -44, -435, "minecraft:polished_deepslate");
    w.fill(2512, -43, -445, 2538, -36, -435, "minecraft:air");
    w.fill(2510, -35, -446, 2540, -34, -434, "minecraft:tuff_bricks");

    // Three monumental north-facing arches opening onto the dark basin & sky view
    w.fill(2513, -43, -446, 2520, -37, -446, "minecraft:air");
    w.fill(2522, -43, -446, 2528, -36, -446, "minecraft:air"); // center main arch
    w.fill(2530, -43, -446, 2537, -37, -446, "minecraft:air");
    for &px in &[2511, 2521, 2529, 2539] {
        w.fill(px, -43, -446, px, -36, -446, "minecraft:tuff_bricks");
    }

    // Interior details in Lowest Public Chamber (quiet, awe-filled observation chamber)
    w.set(2525, -43, -438, "minecraft:lectern[facing=north,has_book=false,powered=false]");
    w.set(2515, -43, -437, "minecraft:polished_tuff_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2516, -43, -437, "minecraft:polished_tuff_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2534, -43, -437, "minecraft:polished_tuff_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2535, -43, -437, "minecraft:polished_tuff_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2513, -36, -444, "minecraft:waxed_oxidized_copper_lantern[hanging=true]");
    w.set(2537, -36, -444, "minecraft:waxed_oxidized_copper_lantern[hanging=true]");

    // Broad stepped descent down into the Basin Floor (z=-447..-450, floor=-44 -> -48)
    for step in 0..4 {
        let sy = -44 - step;
        let sz = -447 - step;
        w.fill(2522, sy - 4, sz, 2528, sy, sz, "minecraft:deepslate_bricks");
        w.fill(
            2522,
            sy,
            sz,
            2528,
            sy,
            sz,
            "minecraft:polished_deepslate_stairs[facing=south,half=bottom,shape=straight]",
        );
        w.fill(2521, sy + 1, sz, 2529, -36, sz, "minecraft:air");
    }

    // MERIDIAN BASIN FLOOR, MIRROR POOLS & CALIBRATION CAUSEWAY (floor=-48)
    // Twin strictly contained mirror water pools (y=-49 water over y=-50 solid deepslate, solid -48 walls)
    w.fill(2513, -51, -506, 2537, -48, -472, "minecraft:deepslate");
    w.fill(2514, -49, -504, 2523, -49, -474, "minecraft:water[level=0]");
    w.fill(2527, -49, -504, 2536, -49, -474, "minecraft:water[level=0]");
    // Clear air directly above pools
    w.fill(2514, -48, -504, 2523, -45, -474, "minecraft:air");
    w.fill(2527, -48, -504, 2536, -45, -474, "minecraft:air");

    // 3-wide Meridian Causeway (x=2524..2526, floor=-48, z=-526..-451)
    w.fill(2524, -50, -526, 2526, -48, -451, "minecraft:polished_deepslate");
    w.fill(2524, -47, -526, 2526, -44, -451, "minecraft:air");
    // Pale calcite meridian axis along x=2525, with gold calibration points at key alignments
    w.fill(2525, -48, -526, 2525, -48, -451, "minecraft:calcite");
    for gz in (-520..=-460).step_by(10) {
        w.set(2525, -48, gz, "minecraft:gold_block");
    }
    // Bridge Oculus plumb marker at (2525, -48, -502) & Center Zenith Ring at (2525, -48, -490)
    w.set(2525, -48, -502, "minecraft:gold_block");
    w.fill(2523, -48, -492, 2527, -48, -488, "minecraft:polished_tuff");
    w.fill(2524, -48, -491, 2526, -48, -489, "minecraft:calcite");
    w.set(2525, -48, -490, "minecraft:gold_block");
}

// ---------- Automated Self-Check / Verification ----------

fn run_audit(w: &mut World) {
    let mut water_count = 0usize;
    let mut gravel_count = 0usize;
    let mut detector_count = 0usize;
    let mut lamp_count = 0usize;
    let mut leaf_count = 0usize;

    for y in Y_MIN..=Y_MAX {
        for z in Z_MIN..=Z_MAX {
            for x in X_MIN..=X_MAX {
                let id = w.get_id(x, y, z);
                if id == 0 {
                    continue;
                }
                let name = w.get_name(id);
                if name.starts_with("minecraft:water") {
                    water_count += 1;
                    assert!(
                        x > X_MIN && x < X_MAX && y > Y_MIN && z > Z_MIN && z < Z_MAX,
                        "Water at box boundary!"
                    );
                    for &(nx, ny, nz) in &[
                        (x - 1, y, z),
                        (x + 1, y, z),
                        (x, y, z - 1),
                        (x, y, z + 1),
                        (x, y - 1, z),
                    ] {
                        let nid = w.get_id(nx, ny, nz);
                        assert!(
                            nid != 0,
                            "Uncontained water at ({},{},{}) touching air at ({},{},{})",
                            x,
                            y,
                            z,
                            nx,
                            ny,
                            nz
                        );
                    }
                } else if name == "minecraft:gravel" {
                    gravel_count += 1;
                    assert!(y > Y_MIN, "Gravel at Y_MIN");
                    assert!(
                        w.get_id(x, y - 1, z) != 0,
                        "Floating gravel at ({},{},{})",
                        x,
                        y,
                        z
                    );
                } else if name.starts_with("minecraft:daylight_detector") {
                    detector_count += 1;
                    for sy in (y + 1)..=Y_MAX {
                        assert!(
                            w.get_id(x, sy, z) == 0,
                            "Daylight detector at ({},{},{}) occluded at y={}",
                            x,
                            y,
                            z,
                            sy
                        );
                    }
                } else if name == "minecraft:redstone_lamp" {
                    lamp_count += 1;
                } else if name.contains("leaves") {
                    leaf_count += 1;
                    assert!(
                        name.contains("persistent=true"),
                        "Leaf without persistent=true: {}",
                        name
                    );
                } else if name.contains("lantern[hanging=true]") {
                    assert!(
                        y < Y_MAX && w.get_id(x, y + 1, z) != 0,
                        "Floating hanging lantern at ({},{},{})",
                        x,
                        y,
                        z
                    );
                } else if name.contains("lantern[hanging=false]") {
                    assert!(
                        y > Y_MIN && w.get_id(x, y - 1, z) != 0,
                        "Floating standing lantern at ({},{},{})",
                        x,
                        y,
                        z
                    );
                } else if name == "minecraft:rail"
                    || name.starts_with("minecraft:redstone_wire")
                    || name.starts_with("minecraft:lectern")
                {
                    assert!(
                        y > Y_MIN && w.get_id(x, y - 1, z) != 0,
                        "Unsupported {} at ({},{},{})",
                        name,
                        x,
                        y,
                        z
                    );
                } else if name.contains("half=lower") {
                    assert!(
                        y < Y_MAX && w.get_name(w.get_id(x, y + 1, z)).contains("half=upper"),
                        "Door missing upper half at ({},{},{})",
                        x,
                        y,
                        z
                    );
                } else if name.contains("part=foot") {
                    assert!(
                        z > Z_MIN && w.get_name(w.get_id(x, y, z - 1)).contains("part=head"),
                        "Bed missing head at ({},{},{})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    // Verify unobstructed sightline from Arrival Gallery (2487, 32, -472) to Astronomer's Lit Window (2566, 28, -485)
    for t in 1..80 {
        let sx = 2487 + ((2566 - 2487) * t) / 80;
        let sy = 32 + ((28 - 32) * t) / 80;
        let sz = -472 + ((-485 - (-472)) * t) / 80;
        assert!(
            w.get_id(sx, sy, sz) == 0,
            "Sightline Arrival->Window blocked at ({},{},{}) by {}",
            sx,
            sy,
            sz,
            w.get_name(w.get_id(sx, sy, sz))
        );
    }

    w.comment("============================================================");
    w.comment("SELF-AUDIT SUMMARY");
    w.comment(&format!(
        "Total /fill commands: {}, /setblock commands: {}, total commands: {}",
        w.fill_cmds,
        w.setblock_cmds,
        w.fill_cmds + w.setblock_cmds
    ));
    w.comment(&format!(
        "Blocks placed: {} non-air, {} air carves",
        w.non_air_placed, w.air_placed
    ));
    w.comment(&format!(
        "Verified invariants: water={}, gravel={}, detectors={}, lamps={}, persistent_leaves={}",
        water_count, gravel_count, detector_count, lamp_count, leaf_count
    ));
}

fn main() {
    let out_dir = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let out_path = Path::new(&out_dir).join("build.txt");
    let file = File::create(&out_path).expect("create build.txt");
    let writer = BufWriter::with_capacity(1 << 20, file);

    let mut w = World::new(writer);
    w.emit = true;

    w.comment("III. The Hollow Meridian — Minecraft Java 26.2 Build Commands");
    w.comment("Box: x in [2300, 2749], y in [-61, 170], z in [-700, -260]");
    w.comment("South quarry road exit: (2525, -60, -260)");
    w.blank();

    w.comment("============================================================");
    w.comment("PART 1: LANDSCAPE — FRACTURED MASSIF, TUFF QUARRY HOLLOW & BASIN");
    w.comment("============================================================");
    w.emit = false;
    build_landscape_into_grid(&mut w);
    w.emit = true;
    w.emit_landscape_from_grid();
    w.blank();

    build_quarry_road(&mut w);
    w.blank();

    build_gatehouse_and_tunnel(&mut w);
    w.blank();

    build_west_side_and_bridge(&mut w);
    w.blank();

    build_instrument_and_slits(&mut w);
    w.blank();

    build_residence_and_garden(&mut w);
    w.blank();

    build_descent_and_basin(&mut w);
    w.blank();

    run_audit(&mut w);
    w.out.flush().expect("flush build.txt");

    println!(
        "Wrote {} ({} /fill, {} /setblock, {} non-air blocks)",
        out_path.display(),
        w.fill_cmds,
        w.setblock_cmds,
        w.non_air_placed
    );
}
