// II. The City That Borrowed a River — Deterministic World Generator
// Minecraft Java 26.2 | Box: x 2300..2749, z -125..124, y -61..80
// Compile: rustc -O gen.rs
// Run:     ./gen <output_dir>   (writes <output_dir>/build.txt)

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

const X_MIN: i32 = 2300;
const X_MAX: i32 = 2749;
const Z_MIN: i32 = -125;
const Z_MAX: i32 = 124;
const Y_MIN: i32 = -61;
const Y_MAX: i32 = 80;

const SX: usize = (X_MAX - X_MIN + 1) as usize; // 450
const SZ: usize = (Z_MAX - Z_MIN + 1) as usize; // 250
const SY: usize = (Y_MAX - Y_MIN + 1) as usize; // 142
const MAX_FILL_VOL: usize = 32768;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Terrain = 0,
    Structure = 1,
    Water = 2,
    Detail = 3,
}

#[derive(Clone)]
struct BlockSpec {
    state: String,
    phase: Phase,
    solid: bool,
    falling: bool,
    water: bool,
}

struct World {
    cells: Vec<u16>,
    palette: Vec<BlockSpec>,
    lookup: HashMap<String, u16>,
}

impl World {
    fn new() -> Self {
        let mut w = World {
            cells: vec![0; SX * SZ * SY],
            palette: Vec::new(),
            lookup: HashMap::new(),
        };
        w.intern("minecraft:air", Phase::Terrain, false, false, false);
        w
    }

    fn intern(&mut self, state: &str, phase: Phase, solid: bool, falling: bool, water: bool) -> u16 {
        if let Some(&id) = self.lookup.get(state) {
            return id;
        }
        let id = self.palette.len() as u16;
        self.palette.push(BlockSpec {
            state: state.to_string(),
            phase,
            solid,
            falling,
            water,
        });
        self.lookup.insert(state.to_string(), id);
        id
    }

    #[inline]
    fn in_bounds(x: i32, y: i32, z: i32) -> bool {
        x >= X_MIN && x <= X_MAX && y >= Y_MIN && y <= Y_MAX && z >= Z_MIN && z <= Z_MAX
    }

    #[inline]
    fn idx(x: i32, y: i32, z: i32) -> usize {
        let lx = (x - X_MIN) as usize;
        let lz = (z - Z_MIN) as usize;
        let ly = (y - Y_MIN) as usize;
        (lx * SZ + lz) * SY + ly
    }

    fn id_for(&mut self, state: &str, phase: Phase) -> u16 {
        let is_air = state == "minecraft:air";
        let is_water = state.starts_with("minecraft:water");
        let is_falling = state == "minecraft:gravel" || state == "minecraft:sand";
        let is_non_solid = is_air
            || is_water
            || state.contains("lantern")
            || state.contains("torch")
            || state.contains("short_grass")
            || state.contains("fern")
            || state.contains("dandelion")
            || state.contains("poppy")
            || state.contains("blue_orchid")
            || state.contains("allium")
            || state.contains("cornflower")
            || state.contains("lily_of_the_valley")
            || state.contains("wildflowers")
            || state.contains("pink_petals")
            || state.contains("leaf_litter")
            || state.contains("sugar_cane")
            || state.contains("rose_bush")
            || state.contains("peony")
            || state.contains("lilac")
            || state.contains("tall_grass")
            || state.contains("firefly_bush")
            || state.contains("flower_pot")
            || state.contains("potted_")
            || state.contains("carpet")
            || state.contains("_door")
            || state.contains("sign");
        self.intern(state, phase, !is_non_solid, is_falling, is_water)
    }

    fn set(&mut self, x: i32, y: i32, z: i32, state: &str, phase: Phase) {
        if !Self::in_bounds(x, y, z) {
            return;
        }
        let id = self.id_for(state, phase);
        let i = Self::idx(x, y, z);
        self.cells[i] = id;
    }

    fn set_if_air(&mut self, x: i32, y: i32, z: i32, state: &str, phase: Phase) {
        if !Self::in_bounds(x, y, z) {
            return;
        }
        let i = Self::idx(x, y, z);
        if self.cells[i] == 0 {
            let id = self.id_for(state, phase);
            self.cells[i] = id;
        }
    }

    fn get_id(&self, x: i32, y: i32, z: i32) -> u16 {
        if !Self::in_bounds(x, y, z) {
            return 0;
        }
        self.cells[Self::idx(x, y, z)]
    }

    fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        if !Self::in_bounds(x, y, z) {
            return false;
        }
        self.palette[self.cells[Self::idx(x, y, z)] as usize].solid
    }

    fn is_water(&self, x: i32, y: i32, z: i32) -> bool {
        if !Self::in_bounds(x, y, z) {
            return false;
        }
        self.palette[self.cells[Self::idx(x, y, z)] as usize].water
    }

    fn fill(&mut self, x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32, state: &str, phase: Phase) {
        let xa = x1.min(x2).max(X_MIN);
        let xb = x1.max(x2).min(X_MAX);
        let ya = y1.min(y2).max(Y_MIN);
        let yb = y1.max(y2).min(Y_MAX);
        let za = z1.min(z2).max(Z_MIN);
        let zb = z1.max(z2).min(Z_MAX);
        if xa > xb || ya > yb || za > zb {
            return;
        }
        let id = self.id_for(state, phase);
        for x in xa..=xb {
            for z in za..=zb {
                let base = ((x - X_MIN) as usize * SZ + (z - Z_MIN) as usize) * SY;
                for y in ya..=yb {
                    self.cells[base + (y - Y_MIN) as usize] = id;
                }
            }
        }
    }

    fn top_solid_y(&self, x: i32, z: i32) -> i32 {
        if x < X_MIN || x > X_MAX || z < Z_MIN || z > Z_MAX {
            return Y_MIN;
        }
        let base = ((x - X_MIN) as usize * SZ + (z - Z_MIN) as usize) * SY;
        for ly in (0..SY).rev() {
            let id = self.cells[base + ly];
            if self.palette[id as usize].solid {
                return Y_MIN + ly as i32;
            }
        }
        Y_MIN
    }

    fn column_ground(&mut self, x: i32, z: i32, y_top: i32, y_start: i32, stone_block: &str, sub_block: &str, top_block: &str) {
        if y_top < Y_MIN {
            return;
        }
        let yt = y_top.min(Y_MAX);
        let ys = y_start.max(Y_MIN);
        if ys > yt {
            return;
        }
        for y in ys..=yt {
            let blk = if y == yt {
                top_block
            } else if y >= yt - 2 {
                sub_block
            } else {
                stone_block
            };
            self.set(x, y, z, blk, Phase::Terrain);
        }
    }

    fn place_door(&mut self, x: i32, y: i32, z: i32, door_type: &str, facing: &str) {
        let lower = format!("minecraft:{}[facing={},half=lower,hinge=left,open=false]", door_type, facing);
        let upper = format!("minecraft:{}[facing={},half=upper,hinge=left,open=false]", door_type, facing);
        self.set(x, y, z, &lower, Phase::Detail);
        self.set(x, y + 1, z, &upper, Phase::Detail);
    }

    fn place_bed(&mut self, x: i32, y: i32, z: i32, color: &str, facing: &str) {
        let (dx, dz) = match facing {
            "north" => (0, -1),
            "south" => (0, 1),
            "east" => (1, 0),
            "west" => (-1, 0),
            _ => (0, -1),
        };
        if !self.is_solid(x, y - 1, z) || !self.is_solid(x + dx, y - 1, z + dz) {
            return;
        }
        let foot = format!("minecraft:{}_bed[facing={},part=foot]", color, facing);
        let head = format!("minecraft:{}_bed[facing={},part=head]", color, facing);
        self.set(x, y, z, &foot, Phase::Detail);
        self.set(x + dx, y, z + dz, &head, Phase::Detail);
    }

    fn place_lantern(&mut self, x: i32, y: i32, z: i32, lantern_id: &str, hanging: bool) {
        if hanging {
            if self.is_solid(x, y + 1, z) && self.get_id(x, y, z) == 0 {
                let s = format!("minecraft:{}[hanging=true]", lantern_id);
                self.set(x, y, z, &s, Phase::Detail);
            }
        } else if self.is_solid(x, y - 1, z) && self.get_id(x, y, z) == 0 {
            let s = format!("minecraft:{}[hanging=false]", lantern_id);
            self.set(x, y, z, &s, Phase::Detail);
        }
    }

    fn place_double_plant(&mut self, x: i32, y: i32, z: i32, plant: &str) {
        if !Self::in_bounds(x, y, z) || !Self::in_bounds(x, y + 1, z) {
            return;
        }
        if self.get_id(x, y, z) != 0 || self.get_id(x, y + 1, z) != 0 {
            return;
        }
        let lo = format!("minecraft:{}[half=lower]", plant);
        let hi = format!("minecraft:{}[half=upper]", plant);
        self.set(x, y, z, &lo, Phase::Detail);
        self.set(x, y + 1, z, &hi, Phase::Detail);
    }

    // Automated integrity pass before emitting build.txt
    fn audit_and_repair(&mut self) -> (usize, usize, usize) {
        let mut water_walled = 0usize;
        let mut falling_fixed = 0usize;
        let mut lantern_fixed = 0usize;

        // 1. Ensure no water touches any box edge or y == Y_MIN
        for x in X_MIN..=X_MAX {
            for z in Z_MIN..=Z_MAX {
                for y in Y_MIN..=Y_MAX {
                    if self.is_water(x, y, z) {
                        if x <= X_MIN + 2 || x >= X_MAX - 2 || z <= Z_MIN + 2 || z >= Z_MAX - 2 || y <= Y_MIN {
                            self.set(x, y, z, "minecraft:stone", Phase::Terrain);
                            water_walled += 1;
                        }
                    }
                }
            }
        }

        // 2. Ensure every water block is supported below (unless above another water block)
        //    and walled horizontally (unless pouring straight into a lower pool).
        let mut to_wall: Vec<(i32, i32, i32)> = Vec::new();
        for x in (X_MIN + 1)..=(X_MAX - 1) {
            for z in (Z_MIN + 1)..=(Z_MAX - 1) {
                for y in (Y_MIN + 1)..=(Y_MAX - 1) {
                    if !self.is_water(x, y, z) {
                        continue;
                    }
                    // Support underneath
                    if !self.is_solid(x, y - 1, z) && !self.is_water(x, y - 1, z) {
                        self.set(x, y - 1, z, "minecraft:stone", Phase::Terrain);
                        water_walled += 1;
                    }
                    // Check 4 horizontal neighbors
                    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let nx = x + dx;
                        let nz = z + dz;
                        if !self.is_solid(nx, y, nz) && !self.is_water(nx, y, nz) {
                            // Allow water to spill only if there's water directly below (nx, y-1, nz)
                            // AND the drop is boxed on the sides; otherwise wall it!
                            if !self.is_water(nx, y - 1, nz) {
                                to_wall.push((nx, y, nz));
                            }
                        }
                    }
                }
            }
        }
        for (wx, wy, wz) in to_wall {
            if !self.is_solid(wx, wy, wz) && !self.is_water(wx, wy, wz) {
                let wall_blk = if wy >= -30 && wx >= 2450 && wx <= 2630 && wz <= -10 {
                    "minecraft:tuff_bricks"
                } else {
                    "minecraft:mossy_cobblestone"
                };
                self.set(wx, wy, wz, wall_blk, Phase::Terrain);
                if wy > Y_MIN && !self.is_solid(wx, wy - 1, wz) && !self.is_water(wx, wy - 1, wz) {
                    self.set(wx, wy - 1, wz, "minecraft:stone", Phase::Terrain);
                }
                water_walled += 1;
            }
        }

        // 3. Ensure falling blocks (gravel/sand) always rest on solid support
        for x in X_MIN..=X_MAX {
            for z in Z_MIN..=Z_MAX {
                for y in (Y_MIN + 1)..=Y_MAX {
                    let id = self.get_id(x, y, z);
                    if self.palette[id as usize].falling && !self.is_solid(x, y - 1, z) {
                        self.set(x, y, z, "minecraft:cobblestone", Phase::Terrain);
                        falling_fixed += 1;
                    }
                }
            }
        }

        // 4. Verify hanging/standing lanterns have solid support
        for x in X_MIN..=X_MAX {
            for z in Z_MIN..=Z_MAX {
                for y in Y_MIN..=Y_MAX {
                    let id = self.get_id(x, y, z);
                    if id == 0 {
                        continue;
                    }
                    let s = &self.palette[id as usize].state;
                    if s.contains("lantern") {
                        if s.contains("hanging=true") && !self.is_solid(x, y + 1, z) {
                            self.set(x, y, z, "minecraft:air", Phase::Detail);
                            lantern_fixed += 1;
                        } else if s.contains("hanging=false") && !self.is_solid(x, y - 1, z) {
                            self.set(x, y, z, "minecraft:air", Phase::Detail);
                            lantern_fixed += 1;
                        }
                    }
                }
            }
        }

        (water_walled, falling_fixed, lantern_fixed)
    }

    fn emit_build_txt(&self, out_path: &Path, stats: (usize, usize, usize)) -> std::io::Result<()> {
        let file = File::create(out_path)?;
        let mut w = BufWriter::new(file);

        writeln!(w, "# II. The City That Borrowed a River — Build Commands")?;
        writeln!(w, "# Minecraft Java 26.2 | Box: x {}..{}, z {}..{}, y {}..{}", X_MIN, X_MAX, Z_MIN, Z_MAX, Y_MIN, Y_MAX)?;
        writeln!(w, "# Connections: West Road (2300, -60, 0) | North Quarry Road (2525, -60, -125)")?;

        let phases = [
            (Phase::Terrain, "PART 1: GEOLOGICAL LANDSCAPE & GORGE"),
            (Phase::Structure, "PART 2: INFRASTRUCTURE, DISTRICTS & BUILDINGS"),
            (Phase::Water, "PART 3: CONTAINED RIVER, AQUEDUCT & CANAL WATER"),
            (Phase::Detail, "PART 4: INTERIORS, DOORS, BEDS, TREES, FLORA & LIGHTS"),
        ];

        let mut total_fills = 0usize;
        let mut total_sets = 0usize;
        let mut total_blocks = 0usize;

        let mut visited = vec![false; SX * SZ * SY];

        for &(phase, header) in &phases {
            writeln!(w, "\n# ============================================================")?;
            writeln!(w, "# {}", header)?;
            writeln!(w, "# ============================================================")?;
            visited.fill(false);

            for lx in 0..SX {
                for lz in 0..SZ {
                    for ly in 0..SY {
                        let i0 = (lx * SZ + lz) * SY + ly;
                        if visited[i0] {
                            continue;
                        }
                        let id = self.cells[i0];
                        if id == 0 {
                            continue;
                        }
                        let spec = &self.palette[id as usize];
                        if spec.phase != phase {
                            continue;
                        }

                        // Greedy 3D box expansion: Y first (columns), then Z, then X
                        let mut dy = 1usize;
                        while ly + dy < SY && dy < 128 {
                            let i = (lx * SZ + lz) * SY + (ly + dy);
                            if visited[i] || self.cells[i] != id {
                                break;
                            }
                            dy += 1;
                        }

                        let mut dz = 1usize;
                        'z_loop: while lz + dz < SZ && dz * dy <= MAX_FILL_VOL {
                            for yy in 0..dy {
                                let i = (lx * SZ + (lz + dz)) * SY + (ly + yy);
                                if visited[i] || self.cells[i] != id {
                                    break 'z_loop;
                                }
                            }
                            if (dz + 1) * dy > MAX_FILL_VOL {
                                break;
                            }
                            dz += 1;
                        }

                        let mut dx = 1usize;
                        'x_loop: while lx + dx < SX && (dx + 1) * dz * dy <= MAX_FILL_VOL {
                            for zz in 0..dz {
                                for yy in 0..dy {
                                    let i = ((lx + dx) * SZ + (lz + zz)) * SY + (ly + yy);
                                    if visited[i] || self.cells[i] != id {
                                        break 'x_loop;
                                    }
                                }
                            }
                            dx += 1;
                        }

                        for xx in 0..dx {
                            for zz in 0..dz {
                                let base = ((lx + xx) * SZ + (lz + zz)) * SY + ly;
                                for yy in 0..dy {
                                    visited[base + yy] = true;
                                }
                            }
                        }

                        let x1 = X_MIN + lx as i32;
                        let y1 = Y_MIN + ly as i32;
                        let z1 = Z_MIN + lz as i32;
                        let x2 = x1 + dx as i32 - 1;
                        let y2 = y1 + dy as i32 - 1;
                        let z2 = z1 + dz as i32 - 1;
                        let vol = dx * dy * dz;
                        total_blocks += vol;

                        if vol == 1 {
                            writeln!(w, "/setblock {} {} {} {}", x1, y1, z1, spec.state)?;
                            total_sets += 1;
                        } else {
                            writeln!(w, "/fill {} {} {} {} {} {} {}", x1, y1, z1, x2, y2, z2, spec.state)?;
                            total_fills += 1;
                        }
                    }
                }
            }
        }

        writeln!(w, "\n# Summary: fills={}, setblocks={}, total_commands={}, total_blocks={}",
                 total_fills, total_sets, total_fills + total_sets, total_blocks)?;
        writeln!(w, "# Audit repairs: water_walled={}, falling_fixed={}, lantern_fixed={}",
                 stats.0, stats.1, stats.2)?;
        w.flush()?;
        Ok(())
    }
}

// Deterministic hash & fBm value noise
#[inline]
fn hash2(ix: i32, iz: i32, seed: u64) -> f64 {
    let mut h = (ix as i64 as u64).wrapping_mul(0x9E3779B97F4A7C15)
        ^ (iz as i64 as u64).wrapping_mul(0xBF58476D1CE4E5B9)
        ^ seed;
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D049BB133111EB);
    h ^= h >> 31;
    (h & 0xFFFF_FFFF) as f64 / 4294967295.0
}

#[inline]
fn smoothstep(t: f64) -> f64 {
    let c = t.clamp(0.0, 1.0);
    c * c * (3.0 - 2.0 * c)
}

fn vnoise(x: f64, z: f64, seed: u64) -> f64 {
    let ix = x.floor() as i32;
    let iz = z.floor() as i32;
    let fx = smoothstep(x - ix as f64);
    let fz = smoothstep(z - iz as f64);
    let a = hash2(ix, iz, seed);
    let b = hash2(ix + 1, iz, seed);
    let c = hash2(ix, iz + 1, seed);
    let d = hash2(ix + 1, iz + 1, seed);
    let ab = a + (b - a) * fx;
    let cd = c + (d - c) * fx;
    ab + (cd - ab) * fz
}

fn fbm(x: f64, z: f64, seed: u64) -> f64 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    for i in 0..4 {
        sum += amp * vnoise(x * freq, z * freq, seed.wrapping_add(i * 1013));
        freq *= 2.03;
        amp *= 0.5;
    }
    sum
}

#[derive(Clone, Copy)]
struct RiverNode {
    x: f64,
    z: f64,
    wy: i32,
    hw: f64,
}

const RIVER_NODES: [RiverNode; 13] = [
    RiverNode { x: 2672.0, z: 19.0, wy: 4, hw: 6.5 },
    RiverNode { x: 2652.0, z: 18.0, wy: 4, hw: 5.0 },
    RiverNode { x: 2634.0, z: 16.0, wy: 2, hw: 4.5 },
    RiverNode { x: 2614.0, z: 10.0, wy: -6, hw: 4.2 },
    RiverNode { x: 2592.0, z: -6.0, wy: -14, hw: 4.2 },
    RiverNode { x: 2568.0, z: -20.0, wy: -22, hw: 4.5 },
    RiverNode { x: 2542.0, z: -25.0, wy: -28, hw: 4.8 },
    RiverNode { x: 2522.0, z: -23.0, wy: -32, hw: 5.0 }, // Old Bridge Throat
    RiverNode { x: 2494.0, z: -8.0, wy: -40, hw: 5.2 },
    RiverNode { x: 2474.0, z: 10.0, wy: -48, hw: 5.5 },
    RiverNode { x: 2454.0, z: 19.0, wy: -54, hw: 6.2 },
    RiverNode { x: 2425.0, z: 21.0, wy: -58, hw: 7.2 },
    RiverNode { x: 2368.0, z: 24.0, wy: -58, hw: 18.0 }, // Terminal Estuary Lake
];

fn nearest_river(px: f64, pz: f64) -> (f64, i32, f64, f64) {
    let mut best_d2 = 1e18;
    let mut best_wy = -58;
    let mut best_hw = 5.0;
    let mut best_t = 0.0;
    let n = RIVER_NODES.len();
    for i in 0..(n - 1) {
        let a = RIVER_NODES[i];
        let b = RIVER_NODES[i + 1];
        let vx = b.x - a.x;
        let vz = b.z - a.z;
        let len2 = vx * vx + vz * vz;
        let s = if len2 > 1e-6 {
            (((px - a.x) * vx + (pz - a.z) * vz) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let cx = a.x + vx * s;
        let cz = a.z + vz * s;
        let d2 = (px - cx) * (px - cx) + (pz - cz) * (pz - cz);
        if d2 < best_d2 {
            best_d2 = d2;
            best_wy = if s < 0.5 { a.wy } else { b.wy };
            best_hw = a.hw + (b.hw - a.hw) * s;
            best_t = (i as f64 + s) / (n as f64 - 1.0);
        }
    }
    (best_d2.sqrt(), best_wy, best_hw, best_t)
}

fn gauss(x: f64, z: f64, cx: f64, cz: f64, sx: f64, sz: f64, h: f64) -> f64 {
    let dx = (x - cx) / sx;
    let dz = (z - cz) / sz;
    h * (-0.5 * (dx * dx + dz * dz)).exp()
}

fn build_landscape(w: &mut World) {
    let mut heights = vec![Y_MIN; SX * SZ];

    for x in X_MIN..=X_MAX {
        for z in Z_MIN..=Z_MAX {
            let fx = x as f64;
            let fz = z as f64;
            let n1 = fbm(fx * 0.022, fz * 0.022, 42);
            let n2 = fbm(fx * 0.06, fz * 0.06, 1337);

            // Broad geological masses (Plateau East, Gorge Rims, West Meadow)
            let mut h = -61.0;
            h += gauss(fx, fz, 2655.0, 12.0, 62.0, 58.0, 74.0); // East plateau base (~+13)
            h += gauss(fx, fz, 2645.0, -56.0, 34.0, 26.0, 30.0); // NE escarpment (~+38)
            h += gauss(fx, fz, 2666.0, 68.0, 38.0, 28.0, 38.0);  // SE ridge peak (~+48)
            h += gauss(fx, fz, 2558.0, 74.0, 52.0, 28.0, 78.0);  // South gorge rim / Upper Terraces hillside
            h += gauss(fx, fz, 2545.0, -72.0, 48.0, 30.0, 62.0); // North canyon rim
            h += gauss(fx, fz, 2515.0, -34.0, 34.0, 26.0, 42.0); // Old Crossing North Shelf (~-18)
            h += gauss(fx, fz, 2410.0, -25.0, 38.0, 26.0, 22.0); // Lower Quays cliff backdrop
            h += gauss(fx, fz, 2368.0, 24.0, 42.0, 34.0, 5.0);   // Gentle natural levee around Estuary Lake
            h += (n1 - 0.5) * 10.0 + (n2 - 0.5) * 3.5;

            // Smooth perimeter blend down to superflat y = -61 at all 4 box edges
            let ed = (x - X_MIN)
                .min(X_MAX - x)
                .min(z - Z_MIN)
                .min(Z_MAX - z) as f64;
            let edge_t = smoothstep((ed - 2.0) / 36.0);
            h = -61.0 + (h + 61.0) * edge_t;

            // Carve S-bend river gorge
            let (dist, rwy, hw, _t) = nearest_river(fx, fz);
            let bed = (rwy - 3).max(Y_MIN) as f64;
            if dist <= hw {
                h = bed;
            } else if dist <= hw + 18.0 {
                let g = smoothstep((dist - hw) / 18.0);
                // Keep immediate bank solid above water
                let bank_min = (rwy + 1).min(Y_MAX) as f64;
                let cliff = bank_min + (h - bank_min).max(0.0) * g;
                h = if g < 0.14 { bank_min } else { cliff };
            }

            let ih = (h.round() as i32).clamp(Y_MIN, Y_MAX);
            heights[(x - X_MIN) as usize * SZ + (z - Z_MIN) as usize] = ih;
        }
    }

    // Smooth flat/graded architectural terraces & road corridors BEFORE voxelizing
    let flatten = |heights: &mut [i32], x1: i32, z1: i32, x2: i32, z2: i32, target_y: i32, feather: i32| {
        for x in (x1 - feather).max(X_MIN + 4)..=(x2 + feather).min(X_MAX - 4) {
            for z in (z1 - feather).max(Z_MIN + 4)..=(z2 + feather).min(Z_MAX - 4) {
                let dx = if x < x1 { x1 - x } else if x > x2 { x - x2 } else { 0 };
                let dz = if z < z1 { z1 - z } else if z > z2 { z - z2 } else { 0 };
                let d = ((dx * dx + dz * dz) as f64).sqrt();
                if d <= feather as f64 {
                    let (rdist, _, hw, _) = nearest_river(x as f64, z as f64);
                    if rdist <= hw + 1.5 {
                        continue; // Never fill over river channel!
                    }
                    let idx = (x - X_MIN) as usize * SZ + (z - Z_MIN) as usize;
                    let t = 1.0 - smoothstep(d / feather.max(1) as f64);
                    let cur = heights[idx] as f64;
                    heights[idx] = (cur + (target_y as f64 - cur) * t).round() as i32;
                }
            }
        }
    };

    // District platforms (carefully ordered & feathered into natural rock)
    flatten(&mut heights, 2334, -14, 2462, 13, -57, 6);   // Lower Quays terrace
    flatten(&mut heights, 2452, -28, 2475, -3, -47, 5);   // Waterworks Smithy & Tannery shelf
    flatten(&mut heights, 2504, -54, 2554, -28, -19, 6);  // Old Market Square shelf
    flatten(&mut heights, 2479, -27, 2507, -8, -19, 4);   // Old Fulling Mill & Toll shelf
    flatten(&mut heights, 2516, -8, 2558, 16, -17, 5);    // South Courtyard & Watchtower base
    flatten(&mut heights, 2493, 2, 2517, 23, -15, 4);     // West Gorge Overlook Houses
    flatten(&mut heights, 2520, 19, 2538, 34, -11, 4);    // Potter & Tea House
    flatten(&mut heights, 2543, 21, 2562, 37, -7, 4);     // Terrace Cottage 1
    flatten(&mut heights, 2564, 24, 2588, 44, -3, 4);     // Terrace Cottage 2 & 3
    flatten(&mut heights, 2570, 16, 2612, 42, -1, 5);     // Irrigated Orchard Terraces

    // Grade West Road (2300, -60, 0) -> (2335, -56, +2)
    for x in 2300..=2336 {
        let t = smoothstep((x - 2300) as f64 / 36.0);
        let ry = (-61.0 + 4.0 * t).round() as i32;
        let rz = (2.0 * t).round() as i32;
        for dz in -3..=3 {
            let z = rz + dz;
            if z >= Z_MIN + 2 && z <= Z_MAX - 2 {
                let idx = (x - X_MIN) as usize * SZ + (z - Z_MIN) as usize;
                heights[idx] = ry;
            }
        }
    }

    // Grade North Quarry Road (2525, -60, -125) -> (2525, -18, -50)
    for z in -125..=-50 {
        let t = smoothstep((z + 125) as f64 / 75.0);
        let ry = (-61.0 + 42.0 * t).round() as i32;
        for dx in -4..=4 {
            let x = 2525 + dx;
            let idx = (x - X_MIN) as usize * SZ + (z - Z_MIN) as usize;
            if dx.abs() <= 2 {
                heights[idx] = ry;
            } else {
                heights[idx] = heights[idx].min(ry + 3).max(ry);
            }
        }
    }

    // Re-enforce river channel bed & immediate solid banks after terrace flattening
    for x in X_MIN..=X_MAX {
        for z in Z_MIN..=Z_MAX {
            let (dist, rwy, hw, _) = nearest_river(x as f64, z as f64);
            let idx = (x - X_MIN) as usize * SZ + (z - Z_MIN) as usize;
            if dist <= hw {
                heights[idx] = (rwy - 3).max(Y_MIN);
            } else if dist <= hw + 2.2 {
                // Guarantee solid bank >= rwy + 1
                heights[idx] = heights[idx].max((rwy + 1).min(Y_MAX));
            }
        }
    }

    // Voxelize terrain with geological strata
    for x in X_MIN..=X_MAX {
        for z in Z_MIN..=Z_MAX {
            let idx = (x - X_MIN) as usize * SZ + (z - Z_MIN) as usize;
            let h = heights[idx];
            let (dist, rwy, hw, _) = nearest_river(x as f64, z as f64);
            let slope = [
                if x > X_MIN { (h - heights[((x - 1 - X_MIN) as usize * SZ + (z - Z_MIN) as usize)]).abs() } else { 0 },
                if x < X_MAX { (h - heights[((x + 1 - X_MIN) as usize * SZ + (z - Z_MIN) as usize)]).abs() } else { 0 },
                if z > Z_MIN { (h - heights[(x - X_MIN) as usize * SZ + (z - 1 - Z_MIN) as usize]).abs() } else { 0 },
                if z < Z_MAX { (h - heights[(x - X_MIN) as usize * SZ + (z + 1 - Z_MIN) as usize]).abs() } else { 0 },
            ]
            .into_iter()
            .max()
            .unwrap_or(0);

            let stratum = fbm(x as f64 * 0.045, z as f64 * 0.045, 777);
            let stone_blk = if h <= -50 {
                "minecraft:deepslate"
            } else if z < -20 && stratum > 0.55 {
                "minecraft:tuff"
            } else if stratum > 0.52 {
                "minecraft:andesite"
            } else {
                "minecraft:stone"
            };

            let (sub_blk, top_blk) = if dist <= hw + 0.5 {
                let rv = hash2(x, z, 91);
                if rv > 0.72 {
                    ("minecraft:stone", "minecraft:gravel")
                } else if rv > 0.52 {
                    ("minecraft:stone", "minecraft:mossy_cobblestone")
                } else {
                    ("minecraft:stone", "minecraft:cobblestone")
                }
            } else if dist <= hw + 2.5 && h <= rwy + 2 {
                ("minecraft:dirt", "minecraft:mossy_cobblestone")
            } else if slope >= 2 {
                (stone_blk, stone_blk)
            } else {
                ("minecraft:dirt", "minecraft:grass_block")
            };

            w.column_ground(x, z, h, Y_MIN, stone_blk, sub_blk, top_blk);
        }
    }

    // Carve Karst Spring Amphitheater & Cascade (x = 2652..2678, z = 11..27, y = 0..14)
    for x in 2652..=2678 {
        for z in 11..=27 {
            let dx = (x as f64 - 2664.0) / 10.0;
            let dz = (z as f64 - 19.0) / 7.5;
            if dx * dx + dz * dz <= 1.0 {
                w.fill(x, 1, z, x, 16, z, "minecraft:air", Phase::Terrain);
                w.fill(x, -2, z, x, 0, z, "minecraft:mossy_cobblestone", Phase::Terrain);
            }
        }
    }
    // Spring cliff source ledge at x = 2675..2679, z = 17..21
    w.fill(2675, 4, 16, 2679, 13, 22, "minecraft:stone", Phase::Terrain);
    w.fill(2675, 10, 18, 2677, 13, 20, "minecraft:air", Phase::Terrain);
    w.fill(2675, 9, 17, 2678, 9, 21, "minecraft:mossy_cobblestone", Phase::Terrain);
    w.fill(2673, 4, 17, 2675, 8, 21, "minecraft:air", Phase::Terrain);
    w.fill(2674, 4, 17, 2675, 8, 17, "minecraft:tuff", Phase::Terrain);
    w.fill(2674, 4, 21, 2675, 8, 21, "minecraft:tuff", Phase::Terrain);

    // Fill stepped River Water pools + contained cascade waterfalls
    for x in (X_MIN + 5)..=(X_MAX - 5) {
        for z in (Z_MIN + 5)..=(Z_MAX - 5) {
            let (dist, rwy, hw, _) = nearest_river(x as f64, z as f64);
            if dist <= hw - 0.8 {
                let top = w.top_solid_y(x, z);
                if top < rwy {
                    for y in (top + 1)..=rwy {
                        w.set(x, y, z, "minecraft:water[level=0]", Phase::Water);
                    }
                }
            }
        }
    }
    // Terminal Estuary Lake & Lower Quays Navigable Canal (clean straight masonry basin)
    w.fill(2342, -61, 6, 2398, -61, 44, "minecraft:gravel", Phase::Terrain);
    w.fill(2398, -61, 14, 2456, -61, 28, "minecraft:stone", Phase::Terrain);
    // Solid quay/levee rims around lake & canal
    w.fill(2340, -61, 4, 2398, -57, 5, "minecraft:cobblestone", Phase::Terrain);
    w.fill(2340, -61, 45, 2398, -57, 46, "minecraft:cobblestone", Phase::Terrain);
    w.fill(2339, -61, 4, 2341, -57, 46, "minecraft:cobblestone", Phase::Terrain);
    w.fill(2342, -60, 6, 2398, -58, 44, "minecraft:water[level=0]", Phase::Water);
    w.fill(2398, -60, 14, 2456, -58, 28, "minecraft:water[level=0]", Phase::Water);

    // Spring source pool & cascade
    w.fill(2676, 10, 18, 2676, 10, 20, "minecraft:water[level=0]", Phase::Water);
    w.fill(2675, 5, 18, 2675, 10, 20, "minecraft:water[level=0]", Phase::Water);
    w.fill(2656, 1, 13, 2674, 4, 25, "minecraft:water[level=0]", Phase::Water);
}

// Helper: build a framed stone+timber or stone building with floor, walls, windows, roof & door
struct BuildingBox {
    x1: i32,
    z1: i32,
    x2: i32,
    z2: i32,
    floor_y: i32,
    wall_h: i32,
}

fn build_stone_timber_house(
    w: &mut World,
    b: BuildingBox,
    foundation_mat: &str,
    wall_mat: &str,
    post_mat: &str,
    floor_mat: &str,
    roof_mat: &str,
    roof_slab: &str,
    eave_east_west: bool,
    door_side: &str, // "north", "south", "east", "west"
) {
    // Solid stone plinth down 5 blocks so nothing ever floats on slopes
    w.fill(b.x1, b.floor_y - 5, b.z1, b.x2, b.floor_y, b.z2, foundation_mat, Phase::Structure);
    w.fill(b.x1 + 1, b.floor_y, b.z1 + 1, b.x2 - 1, b.floor_y, b.z2 - 1, floor_mat, Phase::Structure);
    // Clear interior
    w.fill(b.x1 + 1, b.floor_y + 1, b.z1 + 1, b.x2 - 1, b.floor_y + b.wall_h, b.z2 - 1, "minecraft:air", Phase::Structure);

    let y_top = b.floor_y + b.wall_h;
    // Walls
    w.fill(b.x1, b.floor_y + 1, b.z1, b.x2, y_top, b.z1, wall_mat, Phase::Structure);
    w.fill(b.x1, b.floor_y + 1, b.z2, b.x2, y_top, b.z2, wall_mat, Phase::Structure);
    w.fill(b.x1, b.floor_y + 1, b.z1, b.x1, y_top, b.z2, wall_mat, Phase::Structure);
    w.fill(b.x2, b.floor_y + 1, b.z1, b.x2, y_top, b.z2, wall_mat, Phase::Structure);

    // Corner timber/stone pilasters
    for &(px, pz) in &[(b.x1, b.z1), (b.x2, b.z1), (b.x1, b.z2), (b.x2, b.z2)] {
        w.fill(px, b.floor_y + 1, pz, px, y_top, pz, post_mat, Phase::Structure);
    }

    // Timber top plate beam
    w.fill(b.x1, y_top, b.z1, b.x2, y_top, b.z1, "minecraft:spruce_log[axis=x]", Phase::Structure);
    w.fill(b.x1, y_top, b.z2, b.x2, y_top, b.z2, "minecraft:spruce_log[axis=x]", Phase::Structure);
    w.fill(b.x1, y_top, b.z1, b.x1, y_top, b.z2, "minecraft:spruce_log[axis=z]", Phase::Structure);
    w.fill(b.x2, y_top, b.z1, b.x2, y_top, b.z2, "minecraft:spruce_log[axis=z]", Phase::Structure);

    // Windows at eye height (floor_y + 2..3)
    let wy = b.floor_y + 2;
    let mx = (b.x1 + b.x2) / 2;
    let mz = (b.z1 + b.z2) / 2;
    if b.x2 - b.x1 >= 6 {
        for &wx in &[b.x1 + 2, b.x2 - 2] {
            if door_side != "north" || (wx - mx).abs() > 1 {
                w.set(wx, wy, b.z1, "minecraft:glass_pane", Phase::Structure);
            }
            if door_side != "south" || (wx - mx).abs() > 1 {
                w.set(wx, wy, b.z2, "minecraft:glass_pane", Phase::Structure);
            }
        }
    }
    if b.z2 - b.z1 >= 6 {
        for &wz in &[b.z1 + 2, b.z2 - 2] {
            if door_side != "west" || (wz - mz).abs() > 1 {
                w.set(b.x1, wy, wz, "minecraft:glass_pane", Phase::Structure);
            }
            if door_side != "east" || (wz - mz).abs() > 1 {
                w.set(b.x2, wy, wz, "minecraft:glass_pane", Phase::Structure);
            }
        }
    }

    // Structural ceiling ridge/tie-beam so hanging lanterns always have solid support
    if eave_east_west {
        w.fill(b.x1 + 1, y_top, mz, b.x2 - 1, y_top, mz, "minecraft:spruce_log[axis=x]", Phase::Structure);
    } else {
        w.fill(mx, y_top, b.z1 + 1, mx, y_top, b.z2 - 1, "minecraft:spruce_log[axis=z]", Phase::Structure);
    }

    // Pitched Roof with 1-block overhang + gable infill
    let r_base = y_top + 1;
    if eave_east_west {
        // Ridge runs North-South (along Z); slopes drop West & East
        let half = (b.x2 - b.x1) / 2 + 1;
        for step in 0..=half {
            let ry = r_base + step;
            let xl = b.x1 - 1 + step;
            let xr = b.x2 + 1 - step;
            if xl > xr {
                break;
            }
            if xl == xr {
                w.fill(xl, ry, b.z1 - 1, xl, ry, b.z2 + 1, roof_slab, Phase::Structure);
            } else {
                let sl = format!("{}[facing=east,half=bottom,shape=straight]", roof_mat);
                let sr = format!("{}[facing=west,half=bottom,shape=straight]", roof_mat);
                w.fill(xl, ry, b.z1 - 1, xl, ry, b.z2 + 1, &sl, Phase::Structure);
                w.fill(xr, ry, b.z1 - 1, xr, ry, b.z2 + 1, &sr, Phase::Structure);
                if xl + 1 <= xr - 1 {
                    w.fill(xl + 1, ry, b.z1, xr - 1, ry, b.z1, wall_mat, Phase::Structure);
                    w.fill(xl + 1, ry, b.z2, xr - 1, ry, b.z2, wall_mat, Phase::Structure);
                }
            }
        }
    } else {
        // Ridge runs East-West (along X); slopes drop North & South
        let half = (b.z2 - b.z1) / 2 + 1;
        for step in 0..=half {
            let ry = r_base + step;
            let zn = b.z1 - 1 + step;
            let zs = b.z2 + 1 - step;
            if zn > zs {
                break;
            }
            if zn == zs {
                w.fill(b.x1 - 1, ry, zn, b.x2 + 1, ry, zn, roof_slab, Phase::Structure);
            } else {
                let sn = format!("{}[facing=south,half=bottom,shape=straight]", roof_mat);
                let ss = format!("{}[facing=north,half=bottom,shape=straight]", roof_mat);
                w.fill(b.x1 - 1, ry, zn, b.x2 + 1, ry, zn, &sn, Phase::Structure);
                w.fill(b.x1 - 1, ry, zs, b.x2 + 1, ry, zs, &ss, Phase::Structure);
                if zn + 1 <= zs - 1 {
                    w.fill(b.x1, ry, zn + 1, b.x1, ry, zs - 1, wall_mat, Phase::Structure);
                    w.fill(b.x2, ry, zn + 1, b.x2, ry, zs - 1, wall_mat, Phase::Structure);
                }
            }
        }
    }

    // Door, clear outside landing pad & bracketed entry lantern
    let dy = b.floor_y + 1;
    match door_side {
        "north" => {
            w.fill(mx - 1, dy, b.z1 - 2, mx + 1, dy + 2, b.z1 - 1, "minecraft:air", Phase::Structure);
            w.fill(mx - 1, dy - 1, b.z1 - 2, mx + 1, dy - 1, b.z1 - 1, foundation_mat, Phase::Structure);
            w.place_door(mx, dy, b.z1, "spruce_door", "south");
            w.set(mx - 1, dy + 3, b.z1 - 1, "minecraft:spruce_fence", Phase::Detail);
            w.place_lantern(mx - 1, dy + 2, b.z1 - 1, "lantern", true);
        }
        "south" => {
            w.fill(mx - 1, dy, b.z2 + 1, mx + 1, dy + 2, b.z2 + 2, "minecraft:air", Phase::Structure);
            w.fill(mx - 1, dy - 1, b.z2 + 1, mx + 1, dy - 1, b.z2 + 2, foundation_mat, Phase::Structure);
            w.place_door(mx, dy, b.z2, "spruce_door", "north");
            w.set(mx + 1, dy + 3, b.z2 + 1, "minecraft:spruce_fence", Phase::Detail);
            w.place_lantern(mx + 1, dy + 2, b.z2 + 1, "lantern", true);
        }
        "west" => {
            w.fill(b.x1 - 2, dy, mz - 1, b.x1 - 1, dy + 2, mz + 1, "minecraft:air", Phase::Structure);
            w.fill(b.x1 - 2, dy - 1, mz - 1, b.x1 - 1, dy - 1, mz + 1, foundation_mat, Phase::Structure);
            w.place_door(b.x1, dy, mz, "spruce_door", "east");
            w.set(b.x1 - 1, dy + 3, mz - 1, "minecraft:spruce_fence", Phase::Detail);
            w.place_lantern(b.x1 - 1, dy + 2, mz - 1, "lantern", true);
        }
        "east" => {
            w.fill(b.x2 + 1, dy, mz - 1, b.x2 + 2, dy + 2, mz + 1, "minecraft:air", Phase::Structure);
            w.fill(b.x2 + 1, dy - 1, mz - 1, b.x2 + 2, dy - 1, mz + 1, foundation_mat, Phase::Structure);
            w.place_door(b.x2, dy, mz, "spruce_door", "west");
            w.set(b.x2 + 1, dy + 3, mz + 1, "minecraft:spruce_fence", Phase::Detail);
            w.place_lantern(b.x2 + 1, dy + 2, mz + 1, "lantern", true);
        }
        _ => {}
    }
    // Interior hanging ceiling lantern
    w.place_lantern(mx, y_top - 1, mz, "lantern", true);
}

fn build_roads_and_bridge(w: &mut World) {
    // 1. West Road (2300, -60, 0) -> (2336, -56, +2)
    for x in 2300..=2336 {
        let t = smoothstep((x - 2300) as f64 / 36.0);
        let cz = (2.0 * t).round() as i32;
        let top = w.top_solid_y(x, cz);
        for dz in -2..=2 {
            let z = cz + dz;
            let ty = w.top_solid_y(x, z).max(top - 1).min(top + 1);
            let mat = if dz.abs() <= 1 {
                if (x + z) % 3 == 0 { "minecraft:tuff" } else { "minecraft:cobblestone" }
            } else {
                "minecraft:coarse_dirt"
            };
            w.set(x, ty, z, mat, Phase::Structure);
            w.fill(x, ty + 1, z, x, ty + 5, z, "minecraft:air", Phase::Structure);
        }
        if x % 12 == 0 {
            let ly = w.top_solid_y(x, cz - 3);
            w.set(x, ly + 1, cz - 3, "minecraft:tuff_wall", Phase::Detail);
            w.set(x, ly + 2, cz - 3, "minecraft:spruce_fence", Phase::Detail);
            w.place_lantern(x, ly + 3, cz - 3, "lantern", false);
        }
    }

    // 2. North Quarry Road (2525, -60, -125) -> (2525, -18, -50)
    for z in -125..=-50 {
        let t = smoothstep((z + 125) as f64 / 75.0);
        let ry = (-61.0 + 42.0 * t).round() as i32;
        w.fill(2523, ry - 4, z, 2527, ry, z, "minecraft:tuff", Phase::Structure);
        w.set(2525, ry, z, "minecraft:polished_tuff", Phase::Structure);
        w.set(2523, ry, z, "minecraft:cobblestone", Phase::Structure);
        w.set(2527, ry, z, "minecraft:cobblestone", Phase::Structure);
        w.fill(2522, ry + 1, z, 2528, ry + 6, z, "minecraft:air", Phase::Structure);
        // Step transitions when ry rises
        if z > -125 {
            let prev_t = smoothstep((z - 1 + 125) as f64 / 75.0);
            let prev_ry = (-61.0 + 42.0 * prev_t).round() as i32;
            if ry > prev_ry {
                w.fill(2523, ry, z, 2527, ry, z, "minecraft:tuff_brick_stairs[facing=south,half=bottom,shape=straight]", Phase::Structure);
            }
        }
        // Curb & spaced lanterns
        w.set(2522, ry + 1, z, "minecraft:tuff_wall", Phase::Structure);
        w.set(2528, ry + 1, z, "minecraft:tuff_wall", Phase::Structure);
        if z % 14 == 0 {
            w.set(2522, ry + 2, z, "minecraft:spruce_fence", Phase::Detail);
            w.place_lantern(2522, ry + 3, z, "lantern", false);
        }
    }

    // Quarry Weigh-Post beside North Road (x = 2516..2521, z = -92..-86, y = -42)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2516, z1: -92, x2: 2521, z2: -86, floor_y: -42, wall_h: 3 },
        "minecraft:tuff_bricks",
        "minecraft:tuff_bricks",
        "minecraft:chiseled_tuff_bricks",
        "minecraft:spruce_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        false,
        "east",
    );
    w.set(2518, -41, -89, "minecraft:crafting_table", Phase::Detail);
    w.set(2518, -41, -88, "minecraft:barrel[facing=up]", Phase::Detail);

    // 3. Quarry Freight Haul Ramp: forks at (2518, -32, -74) down to Lower Quays Stone Yard (2444, -56, -16)
    let ramp_steps = 74;
    for i in 0..=ramp_steps {
        let s = i as f64 / ramp_steps as f64;
        let rx = (2520.0 - 76.0 * s).round() as i32;
        let rz = (-74.0 + 58.0 * s).round() as i32;
        let ry = (-32.0 - 25.0 * s).round() as i32;
        w.fill(rx - 1, ry - 4, rz - 1, rx + 1, ry, rz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(rx - 1, ry + 1, rz - 1, rx + 1, ry + 5, rz + 1, "minecraft:air", Phase::Structure);
        if i % 15 == 0 {
            w.set(rx - 2, ry + 1, rz, "minecraft:tuff_wall", Phase::Detail);
            w.place_lantern(rx - 2, ry + 2, rz, "lantern", false);
        }
    }

    // 4. The Old Tuff Bridge (x = 2521..2529, z = -28..-8, deck y = -18, arch crown y = -20)
    // Heavy Episode 1 tuff masonry abutments anchored deep into the gorge walls
    w.fill(2521, -36, -31, 2529, -18, -27, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2521, -36, -9, 2529, -18, -5, "minecraft:tuff_bricks", Phase::Structure);
    // Semicircular arch spanning z = -27..-9 (center z = -18, radius 9)
    for z in -28..=-8 {
        let dz = (z + 18) as f64;
        let arch_y = (-29.0 + (81.0 - dz * dz).max(0.0).sqrt()).round() as i32;
        // Spandrel masonry from arch ring up to deck (-19)
        w.fill(2521, arch_y, z, 2529, -19, z, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(2521, Y_MIN.max(-34), z, 2529, arch_y - 1, z, "minecraft:air", Phase::Structure);
        // Carve deck & parapets
        w.fill(2522, -18, z, 2528, -18, z, "minecraft:polished_tuff", Phase::Structure);
        w.fill(2523, -17, z, 2527, -14, z, "minecraft:air", Phase::Structure);
        // Parapet walls & keystone accent
        w.set(2521, -18, z, "minecraft:tuff_bricks", Phase::Structure);
        w.set(2529, -18, z, "minecraft:tuff_bricks", Phase::Structure);
        w.set(2521, -17, z, "minecraft:tuff_brick_wall", Phase::Structure);
        w.set(2529, -17, z, "minecraft:tuff_brick_wall", Phase::Structure);
    }
    // Keystone & bridge piers with lanterns
    w.set(2521, -20, -18, "minecraft:chiseled_tuff_bricks", Phase::Structure);
    w.set(2529, -20, -18, "minecraft:chiseled_tuff_bricks", Phase::Structure);
    for &pz in &[-28, -18, -8] {
        for &px in &[2521, 2529] {
            w.set(px, -17, pz, "minecraft:chiseled_tuff_bricks", Phase::Structure);
            w.set(px, -16, pz, "minecraft:tuff_brick_wall", Phase::Structure);
            w.place_lantern(px, -15, pz, "lantern", false);
        }
    }
    // Re-ensure water pool under bridge at y = -32
    w.fill(2519, -35, -26, 2531, -32, -10, "minecraft:water[level=0]", Phase::Water);

    // 5. Under-Bridge Service Ledge & Shortcut Stair (x = 2514..2538, z = -13..-11, y = -30 -> -16)
    w.fill(2514, -31, -14, 2538, -31, -11, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2514, -30, -14, 2538, -28, -11, "minecraft:air", Phase::Structure);
    w.place_lantern(2525, -30, -13, "lantern", false);
    // Steep hidden stair behind South Courtyard (x = 2514..2517, z = -11 -> +2, y = -30 -> -16)
    for step in 0..=14 {
        let sz = -11 + step;
        let sy = -30 + step;
        w.fill(2514, sy - 2, sz, 2516, sy - 1, sz, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(2514, sy, sz, 2516, sy + 3, sz, "minecraft:air", Phase::Structure);
        w.fill(2514, sy, sz, 2516, sy, sz, "minecraft:tuff_brick_stairs[facing=south,half=bottom,shape=straight]", Phase::Structure);
    }
}

fn build_lower_quays(w: &mut World) {
    // Wharf Street & Quay Wall (x = 2338..2464, z = 3..13, walkable y = -56)
    w.fill(2338, -61, 3, 2464, -57, 13, "minecraft:deepslate_bricks", Phase::Structure);
    w.fill(2338, -57, 3, 2464, -57, 12, "minecraft:cobblestone", Phase::Structure);
    w.fill(2338, -57, 13, 2464, -57, 13, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2338, -56, 3, 2464, -50, 13, "minecraft:air", Phase::Structure);

    // Mooring bollards & lanterns along quay edge
    for x in (2344..=2460).step_by(12) {
        w.set(x, -56, 13, "minecraft:spruce_fence", Phase::Detail);
        if x % 24 == 0 {
            w.set(x, -55, 13, "minecraft:spruce_fence", Phase::Detail);
            w.place_lantern(x, -54, 13, "lantern", false);
        }
    }

    // West Boat Landing Timber Pier & Moored Skiff (x = 2338..2346, z = 7..11, y = -57)
    w.fill(2338, -57, 7, 2346, -57, 10, "minecraft:spruce_planks", Phase::Structure);
    w.fill(2338, -60, 7, 2338, -57, 7, "minecraft:spruce_log[axis=y]", Phase::Structure);
    w.fill(2346, -60, 7, 2346, -57, 7, "minecraft:spruce_log[axis=y]", Phase::Structure);
    w.fill(2346, -60, 10, 2346, -57, 10, "minecraft:spruce_log[axis=y]", Phase::Structure);
    w.set(2346, -56, 10, "minecraft:spruce_fence", Phase::Detail);
    w.place_lantern(2346, -55, 10, "lantern", false);
    w.fill(2343, -58, 12, 2346, -58, 13, "minecraft:oak_slab[type=bottom]", Phase::Structure); // Skiff

    // Two boat slip stairs down to water y = -58
    for &sx in &[2368, 2420] {
        w.fill(sx, -57, 12, sx + 3, -57, 13, "minecraft:tuff_brick_stairs[facing=north,half=bottom,shape=straight]", Phase::Structure);
        w.fill(sx, -58, 13, sx + 3, -58, 13, "minecraft:tuff_bricks", Phase::Structure);
    }

    // Dark retaining cliff wall behind Lower Quays (z = -12)
    w.fill(2352, -57, -12, 2462, -40, -10, "minecraft:tuff_bricks", Phase::Structure);

    // West Ferry & Customs Shed (x = 2338..2346, z = -7..+1, floor y = -57, walk -56)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2338, z1: -7, x2: 2346, z2: 1, floor_y: -57, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:spruce_planks",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:spruce_stairs",
        "minecraft:spruce_slab[type=bottom]",
        true,
        "south",
    );
    w.set(2340, -56, -5, "minecraft:crafting_table", Phase::Detail);
    w.set(2344, -56, -5, "minecraft:barrel[facing=up]", Phase::Detail);
    w.place_bed(2340, -56, -2, "red", "north");

    // 1. Boatbuilder's Shed (x = 2354..2372, z = -9..+2, floor y = -57)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2354, z1: -9, x2: 2372, z2: 2, floor_y: -57, wall_h: 5 },
        "minecraft:deepslate_bricks",
        "minecraft:spruce_planks",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        false,
        "south",
    );
    // Wide boatbay opening on south face
    w.fill(2358, -56, 2, 2368, -53, 2, "minecraft:air", Phase::Structure);
    // Half-built wooden boat hull inside
    w.fill(2360, -56, -5, 2366, -56, -2, "minecraft:oak_slab[type=bottom]", Phase::Detail);
    w.set(2356, -56, -7, "minecraft:crafting_table", Phase::Detail);
    w.set(2357, -56, -7, "minecraft:chest[facing=south,type=single]", Phase::Detail);
    w.set(2370, -56, -7, "minecraft:barrel[facing=up]", Phase::Detail);

    // 2. Timber & Charcoal Warehouse (x = 2376..2396, z = -10..+2, floor y = -57, 2 storeys)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2376, z1: -10, x2: 2396, z2: 2, floor_y: -57, wall_h: 8 },
        "minecraft:deepslate_bricks",
        "minecraft:tuff_bricks",
        "minecraft:tuff_bricks",
        "minecraft:cobblestone",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        false,
        "south",
    );
    // Wide 3-block loading bays on south ground floor + upper loft floor
    w.fill(2380, -56, 2, 2382, -54, 2, "minecraft:air", Phase::Structure);
    w.fill(2390, -56, 2, 2392, -54, 2, "minecraft:air", Phase::Structure);
    w.fill(2377, -52, -9, 2395, -52, 1, "minecraft:spruce_planks", Phase::Structure);
    w.fill(2385, -51, 2, 2387, -50, 2, "minecraft:air", Phase::Structure); // Loft hoist door
    w.fill(2386, -49, 2, 2386, -49, 5, "minecraft:spruce_log[axis=z]", Phase::Structure); // Hoist beam
    w.set(2386, -50, 5, "minecraft:iron_chain[axis=y]", Phase::Detail);
    w.set(2386, -51, 5, "minecraft:iron_chain[axis=y]", Phase::Detail);
    w.fill(2378, -56, -8, 2381, -55, -6, "minecraft:coal_block", Phase::Detail);
    w.fill(2391, -56, -8, 2394, -55, -6, "minecraft:spruce_log[axis=x]", Phase::Detail);
    w.set(2378, -56, -2, "minecraft:chest[facing=east,type=single]", Phase::Detail);
    w.set(2394, -56, -2, "minecraft:barrel[facing=up]", Phase::Detail);

    // 3. Quay Granary & Salt Store (x = 2400..2420, z = -10..+2, floor y = -57)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2400, z1: -10, x2: 2420, z2: 2, floor_y: -57, wall_h: 7 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        false,
        "south",
    );
    w.fill(2401, -56, -9, 2419, -54, -9, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2402, -56, -7, 2406, -55, -5, "minecraft:hay_block[axis=y]", Phase::Detail);
    w.set(2416, -56, -7, "minecraft:barrel[facing=up]", Phase::Detail);
    w.set(2417, -56, -7, "minecraft:chest[facing=south,type=single]", Phase::Detail);

    // 4. Quarry Stone Yard & Harbormaster (x = 2424..2456, z = -13..+2, floor y = -57)
    w.fill(2424, -60, -13, 2456, -57, 2, "minecraft:tuff", Phase::Structure);
    w.fill(2424, -56, -13, 2456, -51, 2, "minecraft:air", Phase::Structure);
    w.fill(2426, -56, -9, 2430, -55, -5, "minecraft:tuff_bricks", Phase::Detail);
    w.fill(2433, -56, -8, 2436, -55, -6, "minecraft:tuff", Phase::Detail);
    w.set(2428, -56, -1, "minecraft:stonecutter[facing=south]", Phase::Detail);
    w.set(2430, -56, -1, "minecraft:grindstone[face=floor,facing=south]", Phase::Detail);
    // Jib crane on quay edge (x = 2434, z = 12..17)
    w.fill(2434, -56, 12, 2434, -49, 12, "minecraft:spruce_log[axis=y]", Phase::Structure);
    w.fill(2434, -49, 12, 2434, -49, 17, "minecraft:spruce_log[axis=z]", Phase::Structure);
    w.fill(2434, -54, 17, 2434, -50, 17, "minecraft:iron_chain[axis=y]", Phase::Detail);

    // Harbormaster's Office (x = 2444..2456, z = -12..-1, floor y = -57)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2444, z1: -12, x2: 2456, z2: -1, floor_y: -57, wall_h: 5 },
        "minecraft:tuff_bricks",
        "minecraft:tuff_bricks",
        "minecraft:spruce_log[axis=y]",
        "minecraft:oak_planks",
        "minecraft:spruce_stairs",
        "minecraft:spruce_slab[type=bottom]",
        true,
        "south",
    );
    w.set(2446, -56, -9, "minecraft:cartography_table", Phase::Detail);
    w.set(2448, -56, -9, "minecraft:lectern[facing=south]", Phase::Detail);
    w.set(2454, -56, -9, "minecraft:furnace[facing=west]", Phase::Detail);
    w.place_bed(2452, -56, -4, "blue", "north");

    // Moored Cargo Barge in Canal (x = 2402..2416, z = 18..22, water y = -58)
    w.fill(2402, -58, 18, 2416, -58, 22, "minecraft:dark_oak_planks", Phase::Structure);
    w.fill(2403, -57, 18, 2415, -57, 18, "minecraft:spruce_slab[type=bottom]", Phase::Structure);
    w.fill(2403, -57, 22, 2415, -57, 22, "minecraft:spruce_slab[type=bottom]", Phase::Structure);
    w.set(2402, -57, 20, "minecraft:dark_oak_planks", Phase::Structure);
    w.set(2416, -57, 20, "minecraft:dark_oak_planks", Phase::Structure);
    w.fill(2405, -57, 19, 2408, -57, 21, "minecraft:tuff_bricks", Phase::Detail);
    w.set(2412, -57, 20, "minecraft:barrel[facing=up]", Phase::Detail);
    w.place_lantern(2416, -56, 20, "lantern", false);
}

fn build_cliff_stairs(w: &mut World) {
    // Flight 1: (2462, -56, +6) -> (2472, -46, -4)
    for i in 0..=10 {
        let sx = 2462 + i;
        let sz = 6 - i;
        let sy = -57 + i;
        w.fill(sx - 1, sy - 4, sz - 1, sx + 1, sy, sz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(sx - 1, sy + 1, sz - 1, sx + 1, sy + 5, sz + 1, "minecraft:air", Phase::Structure);
        w.set(sx - 1, sy + 1, sz + 1, "minecraft:tuff_brick_wall", Phase::Structure);
        if i % 5 == 0 {
            w.place_lantern(sx - 1, sy + 2, sz + 1, "lantern", false);
        }
    }
    // Landing at y = -46 connecting to Foundry/Smithy (x = 2460..2474, z = -16..-4)
    w.fill(2460, -50, -16, 2475, -47, -4, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2460, -46, -16, 2475, -41, -4, "minecraft:air", Phase::Structure);

    // Flight 2: (2474, -46, -8) -> (2496, -32, -16) passing UNDER Toll House (y = -28)
    for i in 0..=14 {
        let sx = 2474 + (i * 22) / 14;
        let sz = -8 - (i * 8) / 14;
        let sy = -47 + i;
        w.fill(sx - 1, sy - 4, sz - 1, sx + 2, sy, sz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(sx - 1, sy + 1, sz - 1, sx + 2, sy + 5, sz + 1, "minecraft:air", Phase::Structure);
        if i % 6 == 0 {
            w.set(sx, sy + 1, sz + 2, "minecraft:tuff_brick_wall", Phase::Structure);
            w.place_lantern(sx, sy + 2, sz + 2, "lantern", false);
        }
    }

    // Flight 3: (2496, -32, -16) -> (2507, -18, -28) emerging into Old Market Square
    for i in 0..=14 {
        let sx = 2496 + (i * 11) / 14;
        let sz = -16 - (i * 12) / 14;
        let sy = -33 + i;
        w.fill(sx - 1, sy - 4, sz - 1, sx + 1, sy, sz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(sx - 1, sy + 1, sz - 1, sx + 1, sy + 5, sz + 1, "minecraft:air", Phase::Structure);
        if i % 5 == 0 {
            w.set(sx - 1, sy + 1, sz + 1, "minecraft:tuff_brick_wall", Phase::Structure);
            w.place_lantern(sx - 1, sy + 2, sz + 1, "lantern", false);
        }
    }
}

fn build_waterworks(w: &mut World) {
    // 1. Intake Dam, Weir & Footbridge (x = 2623..2626, z = +8..+26)
    w.fill(2623, -8, 8, 2625, 2, 26, "minecraft:tuff_bricks", Phase::Structure);
    // Spillway notch in center (z = 13..20, crest y = 1)
    w.fill(2623, 2, 13, 2625, 4, 20, "minecraft:air", Phase::Structure);
    w.fill(2622, -6, 13, 2622, 2, 20, "minecraft:water[level=0]", Phase::Water);
    // Copper sluice gate at north intake (x = 2624, z = 9..11)
    w.fill(2624, 2, 9, 2624, 4, 11, "minecraft:waxed_exposed_copper_bars", Phase::Structure);
    // Weir Footbridge above (y = +5)
    w.fill(2623, 4, 8, 2625, 4, 26, "minecraft:spruce_planks", Phase::Structure);
    w.fill(2623, 5, 8, 2623, 5, 26, "minecraft:tuff_brick_wall", Phase::Structure);
    w.fill(2625, 5, 8, 2625, 5, 26, "minecraft:tuff_brick_wall", Phase::Structure);
    // Open access gap on west parapet for Terrace Promenade (z=11..13) & stair to Aqueduct Walkway
    w.fill(2623, 5, 11, 2623, 6, 13, "minecraft:air", Phase::Structure);
    for step in 0..3 {
        let sx = 2622 - step;
        let sy = 3 - step;
        w.fill(sx, sy - 2, 11, sx, sy, 12, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(sx, sy + 1, 11, sx, sy + 4, 12, "minecraft:air", Phase::Structure);
    }
    for &fz in &[8, 17, 26] {
        w.place_lantern(2623, 6, fz, "waxed_copper_lantern", false);
    }

    // 2. Active Elevated Aqueduct Arcade (x = 2624 -> 2476, curving z = 9 -> -42 -> -32, y = 2 -> 0 -> -14)
    let aq_pts: [(f64, f64, i32); 6] = [
        (2624.0, 9.0, 1),
        (2596.0, -24.0, 0),
        (2568.0, -42.0, -1),
        (2538.0, -42.0, -3),
        (2505.0, -38.0, -6),
        (2478.0, -32.0, -14),
    ];
    let steps = 190;
    for s in 0..=steps {
        let t = s as f64 / steps as f64;
        let seg_f = t * (aq_pts.len() - 1) as f64;
        let idx = (seg_f.floor() as usize).min(aq_pts.len() - 2);
        let u = seg_f - idx as f64;
        let (x0, z0, y0) = aq_pts[idx];
        let (x1, z1, y1) = aq_pts[idx + 1];
        let ax = (x0 + (x1 - x0) * u).round() as i32;
        let az = (z0 + (z1 - z0) * u).round() as i32;
        let ay = if u < 0.5 { y0 } else { y1 };

        // Trough floor & solid masonry piers every 9 steps
        w.fill(ax - 1, ay, az - 1, ax + 1, ay, az + 1, "minecraft:tuff_bricks", Phase::Structure);
        // Channel walls (north & south) + clear space above
        w.set(ax, ay + 1, az - 1, "minecraft:tuff_bricks", Phase::Structure);
        w.set(ax, ay + 1, az + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(ax, ay + 1, az, ax, ay + 4, az, "minecraft:air", Phase::Structure);
        // Water inside trough
        w.set(ax, ay + 1, az, "minecraft:water[level=0]", Phase::Water);

        // 2-wide Maintenance Walkway along south side (az + 2..az + 3)
        w.fill(ax, ay, az + 2, ax, ay, az + 3, "minecraft:polished_tuff", Phase::Structure);
        w.fill(ax, ay + 1, az + 2, ax, ay + 4, az + 3, "minecraft:air", Phase::Structure);
        w.set(ax, ay + 1, az + 4, "minecraft:tuff_brick_wall", Phase::Structure);

        if s % 10 == 0 {
            let ground = w.top_solid_y(ax, az).min(ay - 1);
            if ground < ay {
                w.fill(ax - 1, ground, az - 1, ax + 1, ay - 1, az + 1, "minecraft:tuff_bricks", Phase::Structure);
            }
            if s % 20 == 0 {
                w.place_lantern(ax, ay + 2, az + 4, "waxed_exposed_copper_lantern", false);
            }
        }
        // Copper sluice collar at transitions
        if s % 38 == 0 {
            w.set(ax, ay + 2, az - 1, "minecraft:waxed_exposed_cut_copper", Phase::Structure);
            w.set(ax, ay + 2, az + 1, "minecraft:waxed_exposed_cut_copper", Phase::Structure);
        }
    }

    // 3. Upper Waterworks Cistern Tower (x = 2472..2484, z = -37..-27, base -22 -> top -11)
    w.fill(2472, -24, -37, 2484, -12, -27, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2474, -15, -35, 2482, -11, -29, "minecraft:air", Phase::Structure);
    w.fill(2474, -15, -35, 2482, -14, -29, "minecraft:water[level=0]", Phase::Water);
    w.fill(2473, -12, -36, 2483, -12, -28, "minecraft:waxed_weathered_cut_copper_slab[type=bottom]", Phase::Structure);
    w.fill(2474, -12, -35, 2482, -12, -29, "minecraft:air", Phase::Structure);

    // 4. Stepped Tailrace Chute (2474,-14,-28 -> 2456,-56,+10) feeding workshops
    for i in 0..=38 {
        let s = i as f64 / 38.0;
        let tx = (2474.0 - 18.0 * s).round() as i32;
        let tz = (-28.0 + 38.0 * s).round() as i32;
        let ty = (-15.0 - 42.0 * s).round() as i32;
        w.fill(tx - 1, ty - 2, tz - 1, tx + 1, ty, tz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(tx, ty + 1, tz, tx, ty + 3, tz, "minecraft:air", Phase::Structure);
        w.set(tx, ty + 1, tz, "minecraft:water[level=0]", Phase::Water);
    }

    // 5. Tannery & Washhouse (x = 2452..2466, z = -26..-17, floor y = -47)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2452, z1: -26, x2: 2466, z2: -17, floor_y: -47, wall_h: 4 },
        "minecraft:deepslate_bricks",
        "minecraft:tuff_bricks",
        "minecraft:spruce_log[axis=y]",
        "minecraft:cobblestone",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        false,
        "east",
    );
    // Washing vats & functional tools
    w.fill(2454, -46, -24, 2458, -46, -22, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2455, -46, -23, 2457, -46, -23, "minecraft:water[level=0]", Phase::Water);
    w.set(2455, -46, -19, "minecraft:cauldron", Phase::Detail);
    w.set(2457, -46, -19, "minecraft:cauldron", Phase::Detail);
    w.set(2461, -46, -24, "minecraft:loom[facing=south]", Phase::Detail);
    w.set(2463, -46, -24, "minecraft:barrel[facing=up]", Phase::Detail);

    // 6. Copper Foundry & Smithy (x = 2458..2472, z = -15..-5, floor y = -47)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2458, z1: -15, x2: 2472, z2: -5, floor_y: -47, wall_h: 5 },
        "minecraft:deepslate_bricks",
        "minecraft:tuff_bricks",
        "minecraft:deepslate_bricks",
        "minecraft:deepslate_bricks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        true,
        "east",
    );
    // Wide open smithy front & chimney
    w.fill(2472, -46, -12, 2472, -44, -8, "minecraft:air", Phase::Structure);
    w.fill(2459, -46, -13, 2461, -36, -11, "minecraft:deepslate_bricks", Phase::Structure);
    w.set(2461, -46, -12, "minecraft:blast_furnace[facing=east]", Phase::Detail);
    w.set(2461, -46, -11, "minecraft:blast_furnace[facing=east]", Phase::Detail);
    w.set(2461, -46, -9, "minecraft:furnace[facing=east]", Phase::Detail);
    w.set(2465, -46, -10, "minecraft:anvil[facing=north]", Phase::Detail);
    w.set(2465, -46, -8, "minecraft:smithing_table", Phase::Detail);
    w.set(2467, -46, -8, "minecraft:grindstone[face=floor,facing=west]", Phase::Detail);
    // Quenching water vat inside smithy
    w.fill(2467, -46, -14, 2470, -46, -12, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2468, -46, -13, 2469, -46, -13, "minecraft:water[level=0]", Phase::Water);
    w.set(2463, -46, -7, "minecraft:waxed_copper_block", Phase::Detail);
    w.set(2464, -46, -7, "minecraft:iron_block", Phase::Detail);

    // 7. Episode 3 Obsolete Aqueduct Spur (x = 2545 -> 2494, z = -42 -> -26, y = -8)
    // Bricked-up sluice at x = 2546, z = -42
    w.fill(2545, -8, -43, 2547, -5, -41, "minecraft:chiseled_tuff_bricks", Phase::Structure);
    w.set(2545, -6, -42, "minecraft:waxed_oxidized_cut_copper", Phase::Structure);
    for i in 0..=52 {
        let s = i as f64 / 52.0;
        let ox = (2545.0 - 51.0 * s).round() as i32;
        let oz = (-42.0 + 16.0 * s).round() as i32;
        // High footpath trough at y = -8
        w.fill(ox - 1, -9, oz - 1, ox + 1, -9, oz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(ox - 1, -8, oz - 1, ox + 1, -5, oz + 1, "minecraft:air", Phase::Structure);
        w.set(ox, -9, oz, "minecraft:spruce_planks", Phase::Structure);
        w.set(ox, -8, oz - 1, "minecraft:tuff_brick_wall", Phase::Structure);
        w.set(ox, -8, oz + 1, "minecraft:tuff_brick_wall", Phase::Structure);
        if i % 13 == 0 {
            w.fill(ox, -19, oz, ox, -10, oz, "minecraft:tuff_bricks", Phase::Structure);
            w.place_lantern(ox, -7, oz + 1, "waxed_oxidized_copper_lantern", false);
        }
    }
    // Connector stair: Active Aqueduct Walkway (y = -3, x = 2546, z = -39) -> Obsolete Spur (y = -9, x = 2540, z = -39)
    for step in 0..=6 {
        let sx = 2546 - step;
        let sy = -3 - step;
        w.fill(sx, sy - 2, -40, sx, sy, -39, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(sx, sy + 1, -40, sx, sy + 4, -39, "minecraft:air", Phase::Structure);
    }
    // Market Square -> Active Aqueduct Walkway Stair (2538,-19,-47 -> 2556,-2,-40)
    for step in 0..=18 {
        let s = step as f64 / 18.0;
        let sx = (2538.0 + 18.0 * s).round() as i32;
        let sz = (-47.0 + 7.0 * s).round() as i32;
        let sy = -19 + step;
        w.fill(sx, sy - 3, sz - 1, sx + 1, sy, sz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(sx, sy + 1, sz - 1, sx + 1, sy + 4, sz + 1, "minecraft:air", Phase::Structure);
        if step % 6 == 0 {
            w.place_lantern(sx, sy + 1, sz - 1, "lantern", false);
        }
    }
}

fn build_old_crossing_and_rooftop(w: &mut World) {
    // 1. Old Market Square Floor (x = 2505..2541, z = -48..-28, walkable at y = -18)
    w.fill(2505, -22, -48, 2541, -19, -28, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2506, -19, -47, 2540, -19, -29, "minecraft:cobblestone", Phase::Structure);
    // Decorative tuff bands across plaza
    w.fill(2522, -19, -48, 2528, -19, -28, "minecraft:polished_tuff", Phase::Structure);
    w.fill(2506, -19, -39, 2540, -19, -37, "minecraft:polished_tuff", Phase::Structure);
    w.fill(2506, -18, -47, 2540, -10, -29, "minecraft:air", Phase::Structure);

    // Central Conduit Fountain (x = 2523..2527, z = -40..-36, y = -18)
    w.fill(2523, -18, -40, 2527, -18, -36, "minecraft:chiseled_tuff_bricks", Phase::Structure);
    w.fill(2524, -18, -39, 2526, -18, -37, "minecraft:water[level=0]", Phase::Water);
    w.set(2525, -18, -38, "minecraft:chiseled_tuff_bricks", Phase::Structure);
    w.set(2525, -17, -38, "minecraft:tuff_brick_wall", Phase::Structure);
    w.place_lantern(2525, -16, -38, "lantern", false);

    // Market Benches & Stalls
    for &sx in &[2514, 2533] {
        w.fill(sx, -18, -35, sx + 4, -18, -35, "minecraft:spruce_fence", Phase::Detail);
        w.fill(sx, -18, -33, sx + 4, -18, -33, "minecraft:spruce_fence", Phase::Detail);
        w.fill(sx, -17, -35, sx + 4, -16, -33, "minecraft:spruce_fence", Phase::Detail);
        w.fill(sx, -15, -35, sx + 4, -15, -33,
               if sx == 2514 { "minecraft:red_wool" } else { "minecraft:cyan_wool" },
               Phase::Detail);
        w.set(sx + 1, -18, -34, "minecraft:crafting_table", Phase::Detail);
        w.set(sx + 2, -18, -34, "minecraft:barrel[facing=up]", Phase::Detail);
        w.set(sx + 3, -18, -34, "minecraft:chest[facing=north,type=single]", Phase::Detail);
    }

    // 2. North Customs Gatehouse (x = 2518..2532, z = -54..-48, floor y = -19)
    w.fill(2518, -23, -54, 2522, -9, -48, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2528, -23, -54, 2532, -9, -48, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2523, -13, -54, 2527, -9, -48, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2523, -18, -54, 2527, -14, -48, "minecraft:air", Phase::Structure);
    w.fill(2518, -8, -54, 2532, -8, -48, "minecraft:tuff_brick_wall", Phase::Structure);
    w.place_lantern(2525, -14, -54, "lantern", true);
    w.place_lantern(2525, -14, -48, "lantern", true);

    // 3. The Assay / Guild Hall — Extraordinary Civic Building (x = 2539..2553, z = -46..-30, floor y = -19)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2539, z1: -46, x2: 2553, z2: -30, floor_y: -19, wall_h: 9 },
        "minecraft:tuff_bricks",
        "minecraft:chiseled_tuff_bricks",
        "minecraft:polished_tuff",
        "minecraft:oak_planks",
        "minecraft:waxed_weathered_cut_copper_stairs",
        "minecraft:waxed_weathered_cut_copper_slab[type=bottom]",
        false,
        "west",
    );
    // Ground arcade along West Market façade + upper council floor
    w.fill(2540, -14, -45, 2552, -14, -31, "minecraft:oak_planks", Phase::Structure);
    for &az in &[-43, -38, -33] {
        w.fill(2539, -18, az - 1, 2539, -16, az + 1, "minecraft:air", Phase::Structure);
    }
    w.set(2539, -12, -38, "minecraft:waxed_chiseled_copper", Phase::Structure); // Guild emblem
    w.set(2546, -18, -38, "minecraft:lectern[facing=west]", Phase::Detail);
    w.set(2548, -18, -42, "minecraft:cartography_table", Phase::Detail);
    w.fill(2551, -18, -44, 2551, -16, -32, "minecraft:bookshelf", Phase::Detail);
    w.set(2538, -15, -38, "minecraft:spruce_fence", Phase::Detail);
    w.place_lantern(2538, -16, -38, "waxed_copper_lantern", true);
    w.place_lantern(2546, -15, -38, "waxed_copper_lantern", true);

    // 4. The Bakery (Ordinary Neighbour, x = 2505..2516, z = -48..-38, floor y = -19)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2505, z1: -48, x2: 2516, z2: -38, floor_y: -19, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:spruce_stairs",
        "minecraft:spruce_slab[type=bottom]",
        true,
        "south",
    );
    w.fill(2506, -18, -47, 2507, -10, -46, "minecraft:bricks", Phase::Structure); // Brick oven chimney
    w.set(2508, -18, -46, "minecraft:smoker[facing=south]", Phase::Detail);
    w.set(2509, -18, -46, "minecraft:furnace[facing=south]", Phase::Detail);
    w.set(2513, -18, -46, "minecraft:hay_block[axis=y]", Phase::Detail);
    w.set(2514, -18, -46, "minecraft:barrel[facing=up]", Phase::Detail);

    // 5. The Weir & Bell Inn (x = 2534..2549, z = -28..-16, floor y = -19)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2534, z1: -28, x2: 2549, z2: -16, floor_y: -19, wall_h: 7 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:oak_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        false,
        "west",
    );
    w.fill(2535, -14, -27, 2548, -14, -17, "minecraft:spruce_planks", Phase::Structure);
    w.set(2546, -18, -22, "minecraft:furnace[facing=west]", Phase::Detail);
    w.set(2540, -18, -22, "minecraft:oak_slab[type=top]", Phase::Detail);
    w.set(2532, -18, -24, "minecraft:spruce_stairs[facing=east,half=bottom,shape=straight]", Phase::Detail); // Outdoor bench
    w.set(2533, -14, -22, "minecraft:spruce_log[axis=x]", Phase::Structure);
    w.set(2533, -15, -22, "minecraft:bell[attachment=ceiling,facing=north]", Phase::Detail);
    w.place_lantern(2541, -15, -22, "lantern", true);
    w.place_bed(2544, -13, -24, "red", "east");
    w.place_bed(2544, -13, -19, "white", "east");

    // 6. Cantilevered Guild Toll House above Cliff Stairs (x = 2497..2507, z = -22..-10, floor y = -19)
    w.fill(2497, -34, -22, 2498, -19, -20, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2506, -34, -22, 2507, -19, -10, "minecraft:tuff_bricks", Phase::Structure);
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2497, z1: -22, x2: 2507, z2: -10, floor_y: -19, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:tuff_bricks",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        true,
        "east",
    );
    // Keep Flight 2 & Flight 3 stair passage open underneath (y = -32..-20)!
    w.fill(2496, -31, -21, 2505, -20, -12, "minecraft:air", Phase::Structure);
    for i in 0..=14 {
        let sx = 2496 + (i * 11) / 14;
        let sz = -16 - (i * 12) / 14;
        let sy = -33 + i;
        w.fill(sx - 1, sy - 2, sz - 1, sx + 1, sy, sz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(sx - 1, sy + 1, sz - 1, sx + 1, sy + 4, sz + 1, "minecraft:air", Phase::Structure);
    }
    w.set(2501, -18, -16, "minecraft:lectern[facing=east]", Phase::Detail);
    w.set(2499, -18, -18, "minecraft:chest[facing=east,type=single]", Phase::Detail);

    // 7. THE OLD FULLING MILL & ROOFTOP GARDEN (x = 2480..2495, z = -26..-9)
    // Ground floor (y = -19 -> -12): heavy Episode 2 industrial bays
    w.fill(2480, -34, -26, 2495, -19, -9, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2481, -18, -25, 2494, -5, -10, "minecraft:air", Phase::Structure);
    w.fill(2480, -18, -26, 2495, -12, -26, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2480, -18, -9, 2495, -12, -9, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2480, -18, -26, 2480, -12, -9, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2495, -18, -26, 2495, -12, -9, "minecraft:tuff_bricks", Phase::Structure);
    w.place_door(2495, -18, -17, "spruce_door", "west");
    w.fill(2496, -19, -19, 2498, -19, -15, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2496, -18, -19, 2498, -15, -15, "minecraft:air", Phase::Structure);
    w.set(2483, -18, -22, "minecraft:cauldron", Phase::Detail);
    w.set(2485, -18, -22, "minecraft:cauldron", Phase::Detail);
    w.set(2483, -18, -13, "minecraft:barrel[facing=up]", Phase::Detail);
    w.place_lantern(2488, -13, -17, "lantern", true);

    // Upper Loft (y = -12 -> -5): adapted bright gardener's workshop + connection to Obsolete Aqueduct (-8)
    w.fill(2480, -12, -26, 2495, -12, -9, "minecraft:spruce_planks", Phase::Structure);
    w.fill(2480, -11, -26, 2495, -6, -26, "minecraft:calcite", Phase::Structure);
    w.fill(2480, -11, -9, 2495, -6, -9, "minecraft:calcite", Phase::Structure);
    w.fill(2480, -11, -26, 2480, -6, -9, "minecraft:calcite", Phase::Structure);
    w.fill(2495, -11, -26, 2495, -6, -9, "minecraft:calcite", Phase::Structure);
    // Loft arrival doorway from Obsolete Aqueduct Spur (x = 2492..2494, y = -11..-6, z = -26)
    w.fill(2491, -11, -26, 2494, -6, -26, "minecraft:air", Phase::Structure);
    w.fill(2491, -9, -26, 2494, -9, -26, "minecraft:spruce_planks", Phase::Structure);
    w.fill(2492, -8, -26, 2494, -6, -26, "minecraft:air", Phase::Structure);
    // Step down inside from Obsolete Aqueduct arrival (-8 -> -11)
    for s in 0..3 {
        w.set(2492, -9 - s, -25 + s, "minecraft:spruce_stairs[facing=north,half=bottom,shape=straight]", Phase::Structure);
    }
    // Stair from Ground (-18) to Loft (-11) along south interior wall (z = -11)
    for s in 0..7 {
        w.set(2483 + s, -18 + s, -11, "minecraft:spruce_stairs[facing=east,half=bottom,shape=straight]", Phase::Structure);
        w.fill(2483 + s, -17 + s, -11, 2483 + s, -9, -11, "minecraft:air", Phase::Structure);
    }
    w.fill(2484, -11, -12, 2490, -11, -12, "minecraft:spruce_fence", Phase::Detail);
    w.set(2490, -11, -14, "minecraft:crafting_table", Phase::Detail);
    w.set(2492, -11, -14, "minecraft:composter", Phase::Detail);
    w.set(2493, -11, -14, "minecraft:bookshelf", Phase::Detail);
    w.place_lantern(2488, -6, -17, "lantern", true);

    // ROOFTOP GARDEN DECK (y = -5 slab/soil deck, walkable y = -4)
    w.fill(2480, -5, -26, 2495, -5, -9, "minecraft:spruce_planks", Phase::Structure);
    // Low unremarkable Stair Turret at West edge (x = 2480..2489, z = -20..-15) rising to y = -1
    w.fill(2480, -4, -20, 2489, -1, -15, "minecraft:calcite", Phase::Structure);
    w.fill(2481, -11, -19, 2488, -2, -16, "minecraft:air", Phase::Structure);
    // Clean 7-step staircase from Loft (-11) to Top Landing (-5) along z = -18
    for s in 0..7 {
        let sx = 2481 + s;
        let sy = -11 + s;
        w.fill(sx, sy - 2, -18, sx, sy - 1, -18, "minecraft:spruce_planks", Phase::Structure);
        w.set(sx, sy, -18, "minecraft:spruce_stairs[facing=east,half=bottom,shape=straight]", Phase::Structure);
        w.fill(sx, sy + 1, -18, sx, sy + 3, -18, "minecraft:air", Phase::Structure);
    }
    w.fill(2488, -5, -19, 2488, -5, -16, "minecraft:spruce_planks", Phase::Structure);
    w.fill(2488, -4, -19, 2488, -2, -16, "minecraft:air", Phase::Structure);
    w.fill(2480, -1, -20, 2489, -1, -15, "minecraft:spruce_slab[type=bottom]", Phase::Structure);
    // THE SMALL UNREMARKABLE DOORWAY (2489, -4..-3, -17) opening east onto the Rooftop Garden!
    w.place_door(2489, -4, -17, "spruce_door", "east");

    // Balustrade (open chest-height rail preserving full 360-degree panorama!)
    w.fill(2480, -4, -26, 2495, -4, -26, "minecraft:tuff_brick_wall", Phase::Structure);
    w.fill(2480, -4, -9, 2495, -4, -9, "minecraft:tuff_brick_wall", Phase::Structure);
    w.fill(2495, -4, -26, 2495, -4, -9, "minecraft:tuff_brick_wall", Phase::Structure);
    w.fill(2480, -4, -26, 2480, -4, -20, "minecraft:tuff_brick_wall", Phase::Structure);
    w.fill(2480, -4, -15, 2480, -4, -9, "minecraft:tuff_brick_wall", Phase::Structure);

    // Raised garden planter beds, quiet water mirror & tea bench
    w.fill(2490, -4, -24, 2494, -4, -21, "minecraft:moss_block", Phase::Detail);
    w.fill(2490, -4, -13, 2494, -4, -10, "minecraft:moss_block", Phase::Detail);
    w.set(2491, -3, -23, "minecraft:flowering_azalea", Phase::Detail);
    w.set(2493, -3, -22, "minecraft:blue_orchid", Phase::Detail);
    w.place_double_plant(2493, -3, -24, "rose_bush");
    w.set(2491, -3, -11, "minecraft:fern", Phase::Detail);
    w.set(2493, -3, -12, "minecraft:wildflowers", Phase::Detail);
    // Contained water mirror basin (x = 2491..2493, z = -18..-16, water at -4)
    w.fill(2491, -5, -18, 2493, -5, -16, "minecraft:polished_tuff", Phase::Structure);
    w.set(2492, -4, -17, "minecraft:water[level=0]", Phase::Water);
    // Pergola corner & tea seat facing East/South-East view
    w.fill(2494, -4, -19, 2494, -1, -19, "minecraft:spruce_fence", Phase::Detail);
    w.fill(2494, -4, -15, 2494, -1, -15, "minecraft:spruce_fence", Phase::Detail);
    w.fill(2494, -1, -19, 2494, -1, -15, "minecraft:spruce_slab[type=top]", Phase::Detail);
    w.place_lantern(2494, -2, -17, "lantern", true);
    w.set(2487, -4, -11, "minecraft:spruce_stairs[facing=south,half=bottom,shape=straight]", Phase::Detail);
}

fn build_upper_terraces(w: &mut World) {
    // 1. Demolished-Mill Communal Courtyard (x = 2518..2542, z = -5..+14, y = -17)
    w.fill(2518, -21, -5, 2542, -17, 14, "minecraft:tuff", Phase::Structure);
    w.fill(2519, -17, -4, 2541, -17, 13, "minecraft:cobblestone", Phase::Structure);
    w.fill(2519, -16, -4, 2541, -10, 13, "minecraft:air", Phase::Structure);
    // Surviving low foundation walls of old mill (repurposed as planters & benches)
    w.fill(2529, -16, 2, 2539, -16, 2, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2539, -16, 2, 2539, -16, 10, "minecraft:tuff_bricks", Phase::Structure);
    w.fill(2531, -16, 4, 2537, -16, 8, "minecraft:grass_block", Phase::Detail);
    // Central flowering courtyard tree
    w.fill(2534, -15, 6, 2534, -12, 6, "minecraft:oak_log[axis=y]", Phase::Detail);
    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            for dy in -12..=-10 {
                if dx.abs() + dz.abs() <= 3 {
                    w.set_if_air(2534 + dx, dy, 6 + dz, "minecraft:flowering_azalea_leaves[persistent=true]", Phase::Detail);
                }
            }
        }
    }
    w.set(2528, -16, 4, "minecraft:spruce_stairs[facing=east,half=bottom,shape=straight]", Phase::Detail);
    w.place_lantern(2529, -15, 2, "lantern", false);

    // 2. Watchtower-Base Merchant House (x = 2546..2558, z = 4..16)
    // Episode 1 battered tuff watchtower base (y = -17..-10)
    w.fill(2545, -21, 3, 2559, -11, 17, "minecraft:tuff_bricks", Phase::Structure);
    // Episode 3 house grafted on top (y = -10..-3)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2546, z1: 4, x2: 2558, z2: 16, floor_y: -10, wall_h: 5 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:oak_planks",
        "minecraft:spruce_stairs",
        "minecraft:spruce_slab[type=bottom]",
        false,
        "west",
    );
    // Awkward bent outer stair wrapping the old watchtower corner
    for s in 0..=6 {
        w.fill(2539 + s, -17, 9, 2539 + s, -17 + s, 11, "minecraft:tuff_bricks", Phase::Structure);
        if s >= 1 {
            w.fill(2539 + s, -17 + s, 9, 2539 + s, -17 + s, 11, "minecraft:tuff_brick_stairs[facing=east,half=bottom,shape=straight]", Phase::Structure);
        }
        w.fill(2539 + s, -16 + s, 9, 2539 + s, -12 + s, 11, "minecraft:air", Phase::Structure);
    }
    w.place_bed(2554, -9, 7, "yellow", "east");
    w.set(2548, -9, 14, "minecraft:furnace[facing=north]", Phase::Detail);

    // 3. Two West Gorge Overlook Houses (x = 2494..2516, z = 2..22, floor y = -15)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2494, z1: 2, x2: 2504, z2: 12, floor_y: -15, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        true,
        "east",
    );
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2506, z1: 12, x2: 2516, z2: 22, floor_y: -15, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:spruce_planks",
        "minecraft:spruce_log[axis=y]",
        "minecraft:oak_planks",
        "minecraft:spruce_stairs",
        "minecraft:spruce_slab[type=bottom]",
        false,
        "north",
    );
    w.place_bed(2496, -14, 4, "brown", "west");
    w.place_bed(2508, -14, 19, "white", "south");

    // 4. Potter's Kiln & Quiet Tea House (x = 2522..2536, z = 20..32, floor y = -11)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2522, z1: 20, x2: 2536, z2: 32, floor_y: -11, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        true,
        "north",
    );
    w.fill(2523, -10, 29, 2526, -4, 31, "minecraft:bricks", Phase::Structure); // Kiln
    w.set(2525, -10, 28, "minecraft:furnace[facing=north]", Phase::Detail);
    w.set(2533, -10, 23, "minecraft:decorated_pot", Phase::Detail);
    w.fill(2531, -10, 24, 2534, -10, 27, "minecraft:moss_carpet", Phase::Detail);

    // 5. Hillside Terrace Cottages (3 ordinary homes)
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2544, z1: 22, x2: 2558, z2: 34, floor_y: -7, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:spruce_stairs",
        "minecraft:spruce_slab[type=bottom]",
        false,
        "west",
    );
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2564, z1: 24, x2: 2576, z2: 34, floor_y: -3, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:spruce_planks",
        "minecraft:spruce_log[axis=y]",
        "minecraft:oak_planks",
        "minecraft:deepslate_tile_stairs",
        "minecraft:deepslate_tile_slab[type=bottom]",
        true,
        "north",
    );
    build_stone_timber_house(
        w,
        BuildingBox { x1: 2578, z1: 30, x2: 2588, z2: 42, floor_y: -3, wall_h: 4 },
        "minecraft:tuff_bricks",
        "minecraft:calcite",
        "minecraft:spruce_log[axis=y]",
        "minecraft:spruce_planks",
        "minecraft:spruce_stairs",
        "minecraft:spruce_slab[type=bottom]",
        false,
        "west",
    );

    // Stepped Terrace Promenade connecting Courtyard (-16) -> Terraces -> Intake Weir (+5)
    for i in 0..=82 {
        let s = i as f64 / 82.0;
        let px = (2540.0 + 83.0 * s).round() as i32;
        let pz = (12.0 + 10.0 * (s * std::f64::consts::PI).sin()).round() as i32;
        let py = (-17.0 + 21.0 * s).round() as i32;
        w.fill(px - 1, py - 4, pz - 1, px + 1, py, pz + 1, "minecraft:tuff_bricks", Phase::Structure);
        w.fill(px - 1, py + 1, pz - 1, px + 1, py + 4, pz + 1, "minecraft:air", Phase::Structure);
        if i % 16 == 0 {
            w.set(px, py + 1, pz - 1, "minecraft:tuff_brick_wall", Phase::Detail);
            w.place_lantern(px, py + 2, pz - 1, "lantern", false);
        }
    }

    // 6. Irrigated Terrace Plots (x = 2592..2614, z = 24..44, stepped y = -1, +1, +3)
    for (ti, &(tx1, tx2, ty)) in [(2592, 2598, -1), (2600, 2606, 1), (2608, 2614, 3)].iter().enumerate() {
        w.fill(tx1, ty - 4, 24, tx2, ty, 44, "minecraft:tuff", Phase::Structure);
        w.fill(tx1 + 1, ty, 25, tx2 - 1, ty, 43, "minecraft:dirt", Phase::Structure);
        // Contained 1-wide water irrigation trough down the middle
        w.fill(tx1 + 3, ty, 26, tx1 + 3, ty, 42, "minecraft:water[level=0]", Phase::Water);
        w.set(tx1 + 3, ty + 1, 34, "minecraft:spruce_slab[type=bottom]", Phase::Structure); // Footbridge
        for z in (27..=41).step_by(3) {
            if z == 33 || z == 34 {
                continue;
            }
            w.set(tx1 + 2, ty + 1, z, if ti == 0 { "minecraft:sugar_cane" } else { "minecraft:pumpkin" }, Phase::Detail);
            w.set(tx1 + 4, ty + 1, z, if ti == 1 { "minecraft:melon" } else { "minecraft:flowering_azalea" }, Phase::Detail);
        }
        w.set(tx1 + 1, ty + 1, 25, "minecraft:composter", Phase::Detail);
    }
}

fn add_natural_trees_and_flora(w: &mut World) {
    // Deterministic spruce & oak trees outside roads, river, buildings & sightlines
    for x in (X_MIN + 14..=X_MAX - 14).step_by(9) {
        for z in (Z_MIN + 14..=Z_MAX - 14).step_by(9) {
            let jx = x + ((hash2(x, z, 301) * 5.0) as i32 - 2);
            let jz = z + ((hash2(x, z, 302) * 5.0) as i32 - 2);
            // Keep clear of city core & key Market->Bridge->Rooftop sightline corridor
            if jx >= 2332 && jx <= 2635 && jz >= -58 && jz <= 50 {
                continue;
            }
            if (jx - 2525).abs() <= 10 && jz <= -48 {
                continue;
            }
            let (dist, _, hw, _) = nearest_river(jx as f64, jz as f64);
            if dist <= hw + 5.0 {
                continue;
            }
            let gy = w.top_solid_y(jx, jz);
            let gid = w.get_id(jx, gy, jz);
            if w.palette[gid as usize].state != "minecraft:grass_block" {
                continue;
            }
            if hash2(jx, jz, 303) < 0.42 {
                continue;
            }

            let h = 6 + (hash2(jx, jz, 304) * 5.0) as i32;
            w.fill(jx, gy + 1, jz, jx, gy + h, jz, "minecraft:spruce_log[axis=y]", Phase::Detail);
            for dy in 2..=h + 1 {
                let rad = if dy == h + 1 { 0 } else if dy >= h - 1 { 1 } else { 2 - (dy % 2) };
                for dx in -rad..=rad {
                    for dz in -rad..=rad {
                        if dx.abs() + dz.abs() <= rad + 1 {
                            w.set_if_air(jx + dx, gy + dy, jz + dz, "minecraft:spruce_leaves[persistent=true]", Phase::Detail);
                        }
                    }
                }
            }
        }
    }

    // Scattered wildflowers, grass, ferns & riverside reeds
    for x in (X_MIN + 6..=X_MAX - 6).step_by(3) {
        for z in (Z_MIN + 6..=Z_MAX - 6).step_by(3) {
            let r = hash2(x, z, 505);
            if r < 0.68 {
                continue;
            }
            let gy = w.top_solid_y(x, z);
            if gy >= Y_MAX - 1 || w.get_id(x, gy + 1, z) != 0 {
                continue;
            }
            let gid = w.get_id(x, gy, z);
            if w.palette[gid as usize].state != "minecraft:grass_block" {
                continue;
            }
            let flora = if r > 0.95 {
                "minecraft:wildflowers"
            } else if r > 0.90 {
                "minecraft:fern"
            } else if r > 0.85 {
                "minecraft:poppy"
            } else if r > 0.81 {
                "minecraft:blue_orchid"
            } else {
                "minecraft:short_grass"
            };
            w.set(x, gy + 1, z, flora, Phase::Detail);
        }
    }
}

fn main() {
    let out_dir = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let out_dir_path = Path::new(&out_dir);
    let _ = fs::create_dir_all(out_dir_path);
    let out_path = out_dir_path.join("build.txt");

    let mut w = World::new();
    build_landscape(&mut w);
    build_roads_and_bridge(&mut w);
    build_lower_quays(&mut w);
    build_cliff_stairs(&mut w);
    build_waterworks(&mut w);
    build_old_crossing_and_rooftop(&mut w);
    build_upper_terraces(&mut w);
    add_natural_trees_and_flora(&mut w);

    let stats = w.audit_and_repair();
    w.emit_build_txt(&out_path, stats)
        .expect("failed to write build.txt");
}
