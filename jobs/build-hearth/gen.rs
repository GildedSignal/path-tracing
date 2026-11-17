use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const X_MIN: i32 = 2000;
const X_MAX: i32 = 2179;
const Y_MIN: i32 = -61;
const Y_MAX: i32 = 120;
const Z_MIN: i32 = -60;
const Z_MAX: i32 = 59;

const NX: usize = (X_MAX - X_MIN + 1) as usize; // 180
const NY: usize = (Y_MAX - Y_MIN + 1) as usize; // 182
const NZ: usize = (Z_MAX - Z_MIN + 1) as usize; // 120

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Phase {
    Empty = 0,
    Landscape = 1,
    Structure = 2,
    Details = 3,
    Interiors = 4,
}

#[derive(Clone, Copy)]
struct Cell {
    phase: Phase,
    block: u16,
}

struct World {
    cells: Vec<Cell>,
    palette: Vec<String>,
    lookup: HashMap<String, u16>,
}

impl World {
    fn new() -> Self {
        let mut palette = Vec::new();
        let mut lookup = HashMap::new();
        palette.push("minecraft:air".to_string());
        lookup.insert("minecraft:air".to_string(), 0);
        Self {
            cells: vec![
                Cell {
                    phase: Phase::Empty,
                    block: 0,
                };
                NX * NY * NZ
            ],
            palette,
            lookup,
        }
    }

    fn idx(x: i32, y: i32, z: i32) -> Option<usize> {
        if x < X_MIN || x > X_MAX || y < Y_MIN || y > Y_MAX || z < Z_MIN || z > Z_MAX {
            return None;
        }
        let ix = (x - X_MIN) as usize;
        let iy = (y - Y_MIN) as usize;
        let iz = (z - Z_MIN) as usize;
        Some((iy * NZ + iz) * NX + ix)
    }

    fn intern(&mut self, block: &str) -> u16 {
        if let Some(&id) = self.lookup.get(block) {
            return id;
        }
        let id = self.palette.len() as u16;
        let s = block.to_string();
        self.lookup.insert(s.clone(), id);
        self.palette.push(s);
        id
    }

    fn set(&mut self, x: i32, y: i32, z: i32, phase: Phase, block: &str) {
        if let Some(i) = Self::idx(x, y, z) {
            let id = self.intern(block);
            self.cells[i] = Cell {
                phase: if id == 0 { Phase::Empty } else { phase },
                block: id,
            };
        }
    }

    fn get_name(&self, x: i32, y: i32, z: i32) -> &str {
        match Self::idx(x, y, z) {
            Some(i) => &self.palette[self.cells[i].block as usize],
            None => "minecraft:air",
        }
    }

    fn is_air(&self, x: i32, y: i32, z: i32) -> bool {
        match Self::idx(x, y, z) {
            Some(i) => self.cells[i].block == 0,
            None => true,
        }
    }

    fn fill(&mut self, x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32, phase: Phase, block: &str) {
        let id = self.intern(block);
        let xa = x1.min(x2).max(X_MIN);
        let xb = x1.max(x2).min(X_MAX);
        let ya = y1.min(y2).max(Y_MIN);
        let yb = y1.max(y2).min(Y_MAX);
        let za = z1.min(z2).max(Z_MIN);
        let zb = z1.max(z2).min(Z_MAX);
        let p = if id == 0 { Phase::Empty } else { phase };
        for y in ya..=yb {
            for z in za..=zb {
                for x in xa..=xb {
                    if let Some(i) = Self::idx(x, y, z) {
                        self.cells[i] = Cell { phase: p, block: id };
                    }
                }
            }
        }
    }
}

fn hash_u64(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;
    x
}

fn hash2(x: i32, z: i32, seed: u64) -> f64 {
    let key = ((x as i64 as u64) << 32) ^ (z as i64 as u32 as u64) ^ seed;
    let h = hash_u64(key);
    (h & 0xffffffff) as f64 / (0xffffffffu64 as f64)
}

fn smoothstep(t: f64) -> f64 {
    let c = t.clamp(0.0, 1.0);
    c * c * (3.0 - 2.0 * c)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn value_noise(x: f64, z: f64, seed: u64) -> f64 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let fx = smoothstep(x - x0 as f64);
    let fz = smoothstep(z - z0 as f64);
    let v00 = hash2(x0, z0, seed);
    let v10 = hash2(x0 + 1, z0, seed);
    let v01 = hash2(x0, z0 + 1, seed);
    let v11 = hash2(x0 + 1, z0 + 1, seed);
    lerp(lerp(v00, v10, fx), lerp(v01, v11, fx), fz)
}

fn fbm(x: f64, z: f64, seed: u64) -> f64 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    for i in 0..4 {
        sum += amp * (value_noise(x * freq, z * freq, seed.wrapping_add(i * 977)) * 2.0 - 1.0);
        freq *= 2.03;
        amp *= 0.5;
    }
    sum
}

fn gauss2(dx: f64, dz: f64, sx: f64, sz: f64) -> f64 {
    (-(dx * dx) / (2.0 * sx * sx) - (dz * dz) / (2.0 * sz * sz)).exp()
}

fn ravine_x(z: f64) -> f64 {
    2104.0 + 0.12 * z + 1.8 * (z * 0.08).sin()
}

fn edge_blend(x: f64, z: f64) -> f64 {
    let dx = (x - X_MIN as f64).min(X_MAX as f64 - x);
    let dz = (z - Z_MIN as f64).min(Z_MAX as f64 - z);
    let bx = smoothstep((dx - 2.0) / 14.0);
    let bz = smoothstep((dz - 2.0) / 12.0);
    bx * bz
}

fn terrain_height(x: f64, z: f64) -> f64 {
    let base = -61.0;

    // North mountain mass and east/west spurs
    let north_peak = 64.0 * gauss2(x - 2092.0, z + 34.0, 26.0, 14.0);
    let north_west_spur = 42.0 * gauss2(x - 2044.0, z + 38.0, 20.0, 11.0);
    let north_east_spur = 36.0 * gauss2(x - 2134.0, z + 26.0, 18.0, 12.0);

    // South mountain mass and arms
    let south_peak = 56.0 * gauss2(x - 2088.0, z - 33.0, 27.0, 14.0);
    let south_west_arm = 34.0 * gauss2(x - 2042.0, z - 36.0, 22.0, 11.0);
    let south_east_spur = 28.0 * gauss2(x - 2132.0, z - 25.0, 18.0, 11.0);

    // Saddle bridge between north and south peaks at x=2070..2095
    let saddle = 24.5 * gauss2(x - 2084.0, z - 0.0, 24.0, 24.0);
    // Foothill ramp down toward the east exit (2179, -60, 0)
    let east_ramp = 14.0 * gauss2(x - 2124.0, z - 2.0, 24.0, 18.0);

    // Natural fold noise
    let ridge_noise = 4.2 * fbm(x * 0.032, z * 0.032, 4211) + 1.6 * fbm(x * 0.09, z * 0.09, 8821);

    let mut h = base
        + (north_peak + north_west_spur + north_east_spur
            + south_peak + south_west_arm + south_east_spur
            + saddle + east_ramp + ridge_noise)
            .max(0.0);

    // Carve the Second Valley (quiet western alpine bowl, x=2006..2067, z=-26..+8)
    let valley_bowl = gauss2(x - 2038.0, z + 10.0, 20.0, 14.0);
    h -= 18.0 * valley_bowl;
    // Open the western sunset horizon gap around x=2000..2028, z=-18..+4
    let west_gap = gauss2(x - 2015.0, z + 9.0, 15.0, 13.0);
    h -= 10.0 * west_gap;

    // Carve the Ravine cutting NNE to SSW around x=2104
    let rx = ravine_x(z);
    let r_dist = (x - rx).abs();
    if z >= -34.0 && z <= 34.0 {
        let z_env = smoothstep((z + 34.0) / 8.0) * smoothstep((34.0 - z) / 8.0);
        let gorge_cut = (-((r_dist / 5.2).powi(2))).exp() * z_env;
        h -= 15.5 * gorge_cut;
    }

    // Landslide scar on north cliff just NE of the broken bridge (x=2097..2111, z=-18..-2)
    let scar = gauss2(x - 2104.0, z + 11.0, 5.5, 6.0);
    h -= 4.5 * scar;

    // Smoothly taper all edges to the superflat ground level (y = -61)
    let blend = edge_blend(x, z);
    h = base + (h - base) * blend;

    // Keep building terrace shoulder stable before explicit grading
    let house_pad = gauss2(x - 2082.0, z + 3.0, 12.0, 8.0);
    h = lerp(h, -38.0, 0.72 * house_pad);

    h.max(-61.0)
}

fn path_polyline_points() -> (Vec<(f64, f64, f64)>, Vec<(f64, f64, f64)>, Vec<(f64, f64, f64)>) {
    // 1. East Trail: (2179, -61, 0) -> climbs gently to broken bridge east lip (2111, -39, 0)
    let east_anchors: [(f64, f64, f64); 8] = [
        (2179.0, -61.0, 0.0),
        (2165.0, -59.0, -2.0),
        (2152.0, -55.0, -4.0),
        (2140.0, -51.0, -1.0),
        (2130.0, -47.0, 3.0),
        (2121.0, -43.0, 2.0),
        (2115.0, -40.0, 0.0),
        (2111.0, -39.0, 0.0),
    ];

    // 2. Pedestrian Detour around the broken bridge: (2112, -39, 1) -> stream crossing -> (2095, -38, 1)
    let detour_anchors: [(f64, f64, f64); 8] = [
        (2112.0, -39.0, 1.0),
        (2114.0, -42.0, 6.0),
        (2113.0, -45.0, 11.0),
        (2109.0, -48.0, 14.5),
        (2102.0, -48.0, 14.5),
        (2098.0, -45.0, 11.0),
        (2096.0, -41.0, 6.0),
        (2095.0, -38.0, 1.5),
    ];

    // 3. Old West Road: from gatehouse yard (2090, -38, 2) -> SW shoulder -> (2000, -61, 28)
    let west_anchors: [(f64, f64, f64); 8] = [
        (2090.0, -38.0, 2.0),
        (2077.0, -38.0, 2.5),
        (2066.0, -40.0, 7.0),
        (2054.0, -44.0, 14.0),
        (2040.0, -49.0, 20.0),
        (2025.0, -54.0, 24.0),
        (2012.0, -58.0, 27.0),
        (2000.0, -61.0, 28.0),
    ];

    fn sample_poly(anchors: &[(f64, f64, f64)]) -> Vec<(f64, f64, f64)> {
        let mut pts = Vec::new();
        for w in anchors.windows(2) {
            let (x0, y0, z0) = w[0];
            let (x1, y1, z1) = w[1];
            let dist = ((x1 - x0).powi(2) + (y1 - y0).powi(2) + (z1 - z0).powi(2)).sqrt();
            let steps = (dist * 4.0).ceil().max(1.0) as i32;
            for s in 0..=steps {
                let t = s as f64 / steps as f64;
                pts.push((lerp(x0, x1, t), lerp(y0, y1, t), lerp(z0, z1, t)));
            }
        }
        pts
    }

    (
        sample_poly(&east_anchors),
        sample_poly(&detour_anchors),
        sample_poly(&west_anchors),
    )
}

fn near_path(x: f64, z: f64, pts: &[(f64, f64, f64)], max_r: f64) -> bool {
    let r2 = max_r * max_r;
    for &(px, _, pz) in pts {
        let dx = x - px;
        let dz = z - pz;
        if dx * dx + dz * dz <= r2 {
            return true;
        }
    }
    false
}

fn is_sightline_corridor(x: i32, z: i32) -> bool {
    // 1. East landmark window (2089, -8..-7) -> East trail exit (2179, 0)
    if x >= 2089 && x <= 2179 {
        let t = (x - 2089) as f64 / 90.0;
        let z_center = lerp(-7.5, 0.0, t);
        if (z as f64 - z_center).abs() <= 4.5 {
            return true;
        }
    }
    // 2. West alcove window (2070, -6..-5) -> Tarn (2038, -11) -> West sunset gap (2000, -14)
    if x >= 2000 && x <= 2070 {
        let t = (2070 - x) as f64 / 70.0;
        let z_center = lerp(-5.5, -14.0, t);
        if (z as f64 - z_center).abs() <= 6.5 {
            return true;
        }
    }
    false
}

fn build_landscape(w: &mut World) {
    let mut hmap = vec![-61i32; NX * NZ];
    for z in Z_MIN..=Z_MAX {
        for x in X_MIN..=X_MAX {
            let h = terrain_height(x as f64, z as f64).round() as i32;
            hmap[((z - Z_MIN) as usize) * NX + (x - X_MIN) as usize] = h.clamp(Y_MIN, 40);
        }
    }

    // Grade the building and yard terrace cleanly at y = -38
    for z in -12..=10 {
        for x in 2069..=2095 {
            let idx = ((z - Z_MIN) as usize) * NX + (x - X_MIN) as usize;
            if z <= -11 {
                hmap[idx] = hmap[idx].max(-38);
            } else {
                hmap[idx] = -38;
            }
        }
    }
    // Keep north cliff rising cleanly right behind z = -12
    for z in -22..=-12 {
        for x in 2070..=2094 {
            let idx = ((z - Z_MIN) as usize) * NX + (x - X_MIN) as usize;
            let min_cliff = -37 + (-12 - z);
            hmap[idx] = hmap[idx].max(min_cliff);
        }
    }

    let get_h = |hmap: &[i32], x: i32, z: i32| -> i32 {
        let cx = x.clamp(X_MIN, X_MAX);
        let cz = z.clamp(Z_MIN, Z_MAX);
        hmap[((cz - Z_MIN) as usize) * NX + (cx - X_MIN) as usize]
    };

    // Paint terrain strata and soils
    for z in Z_MIN..=Z_MAX {
        for x in X_MIN..=X_MAX {
            let top = get_h(&hmap, x, z);
            let max_nbr = get_h(&hmap, x + 1, z)
                .max(get_h(&hmap, x - 1, z))
                .max(get_h(&hmap, x, z + 1))
                .max(get_h(&hmap, x, z - 1));
            let min_nbr = get_h(&hmap, x + 1, z)
                .min(get_h(&hmap, x - 1, z))
                .min(get_h(&hmap, x, z + 1))
                .min(get_h(&hmap, x, z - 1));
            let slope = (max_nbr - min_nbr).max(top - min_nbr);
            let n = hash2(x, z, 1009);
            let scar_dist = (((x - 2104) as f64 / 6.0).powi(2) + ((z + 9) as f64 / 7.5).powi(2)).sqrt();

            for y in Y_MIN..=top {
                let depth = top - y;
                let block = if depth == 0 {
                    if scar_dist < 1.1 && top <= -34 {
                        if n < 0.45 {
                            "minecraft:gravel"
                        } else if n < 0.78 {
                            "minecraft:andesite"
                        } else {
                            "minecraft:cobblestone"
                        }
                    } else if top >= -18 || slope >= 3 {
                        let strata = ((y as f64 + 0.25 * x as f64 + 0.15 * z as f64) * 0.45).sin();
                        if strata > 0.25 {
                            "minecraft:andesite"
                        } else if slope >= 4 && n < 0.2 {
                            "minecraft:cobblestone"
                        } else {
                            "minecraft:stone"
                        }
                    } else if top <= -46 && (x - ravine_x(z as f64).round() as i32).abs() <= 5 && z >= -26 && z <= 32 {
                        if n < 0.35 {
                            "minecraft:mossy_cobblestone"
                        } else if n < 0.7 {
                            "minecraft:andesite"
                        } else {
                            "minecraft:moss_block"
                        }
                    } else if top >= -25 && n < 0.35 {
                        "minecraft:coarse_dirt"
                    } else if top <= -54 && x >= 2022 && x <= 2056 && z >= -24 && z <= 2 {
                        if n < 0.25 {
                            "minecraft:moss_block"
                        } else {
                            "minecraft:grass_block"
                        }
                    } else {
                        "minecraft:grass_block"
                    }
                } else if depth <= 2 && top < -18 && slope < 3 {
                    "minecraft:dirt"
                } else {
                    let strata = ((y as f64 + 0.22 * x as f64 + 0.14 * z as f64) * 0.42).sin();
                    if strata > 0.35 {
                        "minecraft:andesite"
                    } else {
                        "minecraft:stone"
                    }
                };
                w.set(x, y, z, Phase::Landscape, block);
            }
        }
    }

    // Carve & fill the Second Valley Tarn (x=2033..2044, z=-15..-7, water y=-56)
    for z in -18..=-4 {
        for x in 2030..=2047 {
            let d = (((x as f64 - 2038.5) / 5.5).powi(2) + ((z as f64 + 11.0) / 4.2).powi(2)).sqrt();
            if d <= 1.0 {
                for y in Y_MIN..=-59 {
                    w.set(x, y, z, Phase::Landscape, "minecraft:andesite");
                }
                w.set(x, -58, z, Phase::Landscape, if d > 0.72 { "minecraft:gravel" } else { "minecraft:andesite" });
                w.set(x, -57, z, Phase::Landscape, "minecraft:water");
                w.set(x, -56, z, Phase::Landscape, "minecraft:water");
                for y in -55..=40 {
                    w.set(x, y, z, Phase::Landscape, "minecraft:air");
                }
            } else if d <= 1.45 {
                // Guaranteed solid rim at y >= -55 so no water can ever leave the tarn
                let rim_top = if d <= 1.2 { -55 } else { -54 };
                for y in Y_MIN..=rim_top {
                    let b = if y == rim_top {
                        if hash2(x, z, 77) < 0.45 { "minecraft:moss_block" } else { "minecraft:andesite" }
                    } else {
                        "minecraft:stone"
                    };
                    w.set(x, y, z, Phase::Landscape, b);
                }
                for y in (rim_top + 1)..=40 {
                    w.set(x, y, z, Phase::Landscape, "minecraft:air");
                }
            }
        }
    }

    // Ravine Contained Alpine Pools (upper spring pool, bridge pool, lower plunge pool)
    let pools: [(i32, i32, i32, i32, i32); 3] = [
        (-18, -13, 2, -43, -45), // upper snowmelt spring pool
        (-3, 5, 3, -49, -51),    // pool directly under the broken bridge
        (21, 28, 3, -53, -55),   // lower plunge basin well inside the box
    ];
    for &(z_start, z_end, half_w, water_y, floor_y) in &pools {
        for z in (z_start - 1)..=(z_end + 1) {
            let cx = ravine_x(z as f64).round() as i32;
            for x in (cx - half_w - 1)..=(cx + half_w + 1) {
                let inside = z >= z_start && z <= z_end && (x - cx).abs() <= half_w;
                if inside {
                    for y in Y_MIN..=floor_y {
                        w.set(x, y, z, Phase::Landscape, "minecraft:andesite");
                    }
                    for y in (floor_y + 1)..=water_y {
                        w.set(x, y, z, Phase::Landscape, "minecraft:water");
                    }
                    for y in (water_y + 1)..=40 {
                        w.set(x, y, z, Phase::Landscape, "minecraft:air");
                    }
                } else {
                    // Solid stone rim around every pool
                    for y in Y_MIN..=water_y {
                        let blk = if y == water_y {
                            if hash2(x, z, 313) < 0.4 {
                                "minecraft:mossy_cobblestone"
                            } else {
                                "minecraft:andesite"
                            }
                        } else {
                            "minecraft:stone"
                        };
                        w.set(x, y, z, Phase::Landscape, blk);
                    }
                }
            }
        }
    }
    // Fallen stone bridge boulders resting in the -49 bridge pool and gorge
    let fallen_stones: [(i32, i32, i32, &str); 7] = [
        (2103, -49, 0, "minecraft:mossy_stone_bricks"),
        (2104, -49, 1, "minecraft:cracked_stone_bricks"),
        (2105, -48, 0, "minecraft:stone_bricks"),
        (2102, -49, -1, "minecraft:cobblestone"),
        (2106, -49, 2, "minecraft:andesite"),
        (2105, -49, -2, "minecraft:mossy_cobblestone"),
        (2103, -48, 0, "minecraft:stone_brick_slab[type=bottom]"),
    ];
    for &(x, y, z, blk) in &fallen_stones {
        w.set(x, y, z, Phase::Landscape, blk);
    }

    // Stamp East Trail, Detour, and Old West Road
    let (east_pts, detour_pts, west_pts) = path_polyline_points();

    let stamp_path = |w: &mut World, pts: &[(f64, f64, f64)], half_w: f64, is_old_road: bool, is_detour: bool| {
        for &(px, py, pz) in pts {
            let yi = py.round() as i32;
            let x0 = (px - half_w).floor() as i32;
            let x1 = (px + half_w).ceil() as i32;
            let z0 = (pz - half_w).floor() as i32;
            let z1 = (pz + half_w).ceil() as i32;
            for z in z0..=z1 {
                for x in x0..=x1 {
                    let dist = ((x as f64 - px).powi(2) + (z as f64 - pz).powi(2)).sqrt();
                    if dist <= half_w {
                        // Retaining stone base down to Y_MIN
                        for y in Y_MIN..yi {
                            let sup = if y == yi - 1 { "minecraft:cobblestone" } else { "minecraft:stone" };
                            if w.is_air(x, y, z) || y >= yi - 2 {
                                w.set(x, y, z, Phase::Landscape, sup);
                            }
                        }
                        let n = hash2(x, z, 555);
                        let surf = if is_old_road {
                            if n < 0.35 {
                                "minecraft:stone_bricks"
                            } else if n < 0.60 {
                                "minecraft:andesite"
                            } else if n < 0.80 {
                                "minecraft:cobblestone"
                            } else {
                                "minecraft:coarse_dirt"
                            }
                        } else if is_detour {
                            if px >= 2101.0 && px <= 2109.0 && pz >= 13.0 {
                                "minecraft:spruce_planks"
                            } else if n < 0.45 {
                                "minecraft:andesite"
                            } else if n < 0.75 {
                                "minecraft:cobblestone"
                            } else {
                                "minecraft:coarse_dirt"
                            }
                        } else {
                            // East approach trail
                            if x >= 2177 && z.abs() <= 1 {
                                "minecraft:coarse_dirt"
                            } else if n < 0.35 {
                                "minecraft:andesite"
                            } else if n < 0.65 {
                                "minecraft:coarse_dirt"
                            } else {
                                "minecraft:gravel"
                            }
                        };
                        w.set(x, yi, z, Phase::Landscape, surf);
                        // Carve headroom
                        for y in (yi + 1)..=(yi + 4) {
                            w.set(x, y, z, Phase::Landscape, "minecraft:air");
                        }
                    }
                }
            }
        }
    };

    stamp_path(w, &east_pts, 1.3, false, false);
    stamp_path(w, &detour_pts, 1.15, false, true);
    stamp_path(w, &west_pts, 1.45, true, false);

    // Ensure exact ground contact at East Exit (2179, -60, 0) and West Exit (2000, -60, 28)
    for z in -1..=1 {
        w.set(2179, -61, z, Phase::Landscape, "minecraft:coarse_dirt");
        for y in -60..=-56 {
            w.set(2179, y, z, Phase::Landscape, "minecraft:air");
        }
    }
    for z in 27..=29 {
        w.set(2000, -61, z, Phase::Landscape, "minecraft:coarse_dirt");
        for y in -60..=-56 {
            w.set(2000, y, z, Phase::Landscape, "minecraft:air");
        }
    }

    // Add detour footbridge deck, stone piers, spruce handrail & cairn markers (Details)
    for x in 2102..=2108 {
        for z in 13..=15 {
            w.set(x, -48, z, Phase::Details, "minecraft:spruce_planks");
            for y in -47..=-44 {
                w.set(x, y, z, Phase::Details, "minecraft:air");
            }
        }
        if x == 2102 || x == 2108 {
            for z in 13..=15 {
                for y in Y_MIN..=-49 {
                    w.set(x, y, z, Phase::Details, "minecraft:cobblestone");
                }
            }
        }
        let east = x < 2108;
        let west = x > 2102;
        let fence = format!(
            "minecraft:spruce_fence[north=false,south=false,east={},west={}]",
            east, west
        );
        w.set(x, -47, 13, Phase::Details, &fence);
    }
    // Wayfinding stone cairns along east trail and detour
    let cairns: [(i32, i32, i32); 5] = [
        (2168, -60, -3),
        (2143, -52, 1),
        (2124, -44, 4),
        (2115, -40, -2),
        (2097, -41, 9),
    ];
    for &(cx, cy, cz) in &cairns {
        // Rest cairn on solid ground right beside path
        let mut gy = cy;
        while gy > Y_MIN && w.is_air(cx, gy - 1, cz) {
            gy -= 1;
        }
        if !w.is_air(cx, gy - 1, cz) {
            w.set(cx, gy, cz, Phase::Details, "minecraft:cobblestone");
            w.set(
                cx,
                gy + 1,
                cz,
                Phase::Details,
                "minecraft:andesite_wall[up=true,north=none,south=none,east=none,west=none]",
            );
        }
    }

    // Plant wind-sheltered alpine spruce trees (never blocking sightlines or paths)
    let tree_candidates: [(i32, i32, i32); 24] = [
        (2158, -16, 10),
        (2146, -15, 11),
        (2154, 12, 9),
        (2138, 14, 12),
        (2126, -16, 10),
        (2118, -21, 9),
        (2128, 22, 11),
        (2061, -24, 12),
        (2049, -27, 13),
        (2034, -27, 11),
        (2021, -24, 10),
        (2056, 2, 11),
        (2046, 7, 12),
        (2031, 4, 10),
        (2019, 2, 9),
        (2024, -2, 8),
        (2063, 18, 10),
        (2049, 27, 11),
        (2030, 34, 9),
        (2074, 21, 10),
        (2086, 18, 9),
        (2108, 26, 8),
        (2165, 18, 9),
        (2168, -22, 8),
    ];

    for &(tx, tz, height) in &tree_candidates {
        if is_sightline_corridor(tx, tz) {
            continue;
        }
        if near_path(tx as f64, tz as f64, &east_pts, 3.5)
            || near_path(tx as f64, tz as f64, &detour_pts, 3.5)
            || near_path(tx as f64, tz as f64, &west_pts, 3.5)
        {
            continue;
        }
        let top = get_h(&hmap, tx, tz);
        if top > -22 || top < -60 {
            continue;
        }
        // Podzol skirt around trunk
        for dz in -2..=2 {
            for dx in -2..=2 {
                if dx * dx + dz * dz <= 5 {
                    let sx = tx + dx;
                    let sz = tz + dz;
                    if sx >= X_MIN && sx <= X_MAX && sz >= Z_MIN && sz <= Z_MAX {
                        let st = get_h(&hmap, sx, sz);
                        if st <= -22 && w.get_name(sx, st, sz) == "minecraft:grass_block" {
                            w.set(sx, st, sz, Phase::Landscape, "minecraft:podzol");
                        }
                    }
                }
            }
        }
        // Trunk
        for dy in 1..=height {
            w.set(tx, top + dy, tz, Phase::Landscape, "minecraft:spruce_log[axis=y]");
        }
        // Layered conical alpine boughs
        let crown_start = 3;
        for dy in crown_start..=(height + 1) {
            let frac = (dy - crown_start) as f64 / ((height + 1 - crown_start) as f64).max(1.0);
            let mut rad = ((1.0 - frac) * 3.0).ceil() as i32;
            if (dy - crown_start) % 2 == 1 && rad > 1 {
                rad -= 1;
            }
            if dy == height + 1 {
                rad = 0;
            }
            for dz in -rad..=rad {
                for dx in -rad..=rad {
                    if dx == 0 && dz == 0 && dy <= height {
                        continue;
                    }
                    if dx.abs() + dz.abs() <= rad + 1 && dx * dx + dz * dz <= rad * rad + 1 {
                        let lx = tx + dx;
                        let lz = tz + dz;
                        if !is_sightline_corridor(lx, lz) && w.is_air(lx, top + dy, lz) {
                            w.set(
                                lx,
                                top + dy,
                                lz,
                                Phase::Landscape,
                                "minecraft:spruce_leaves[persistent=true]",
                            );
                        }
                    }
                }
            }
        }
    }

    // Subtle snow cover on high ridges & shaded north slopes, and wildflowers/ferns in sheltered valleys
    for z in Z_MIN..=Z_MAX {
        for x in X_MIN..=X_MAX {
            // Skip building & road footprint
            if x >= 2068 && x <= 2097 && z >= -13 && z <= 12 {
                continue;
            }
            if near_path(x as f64, z as f64, &east_pts, 2.2)
                || near_path(x as f64, z as f64, &detour_pts, 2.0)
                || near_path(x as f64, z as f64, &west_pts, 2.2)
            {
                continue;
            }
            let top = get_h(&hmap, x, z);
            if !w.is_air(x, top + 1, z) {
                continue;
            }
            let base_blk = w.get_name(x, top, z);
            if base_blk == "minecraft:water" {
                continue;
            }
            let n = hash2(x, z, 9091);
            let north_aspect = if z < 0 { -2.0 } else { 2.0 };
            let snow_thresh = -19.0 + north_aspect + 2.5 * (n - 0.5);
            if (top as f64) > snow_thresh && (base_blk == "minecraft:stone" || base_blk == "minecraft:andesite" || base_blk == "minecraft:coarse_dirt") {
                let layers = if top > -8 {
                    3
                } else if top > -14 {
                    2
                } else {
                    1
                };
                let snow_cmd = format!("minecraft:snow[layers={}]", layers);
                w.set(x, top + 1, z, Phase::Details, &snow_cmd);
            } else if top <= -30 && (base_blk == "minecraft:grass_block" || base_blk == "minecraft:podzol" || base_blk == "minecraft:moss_block") {
                if n < 0.14 {
                    w.set(x, top + 1, z, Phase::Details, "minecraft:short_grass");
                } else if n < 0.21 {
                    w.set(x, top + 1, z, Phase::Details, "minecraft:fern");
                } else if n < 0.225 && base_blk == "minecraft:grass_block" {
                    let fl = if hash2(x, z, 404) < 0.5 {
                        "minecraft:oxeye_daisy"
                    } else {
                        "minecraft:lily_of_the_valley"
                    };
                    w.set(x, top + 1, z, Phase::Details, fl);
                } else if n < 0.24 && base_blk == "minecraft:podzol" {
                    w.set(x, top + 1, z, Phase::Details, "minecraft:leaf_litter");
                }
            }
        }
    }
}

fn weathered_stone(x: i32, y: i32, z: i32, damp: bool) -> &'static str {
    let h = hash_u64(((x as u64) << 40) ^ ((y as u64 & 0xffff) << 20) ^ (z as u64 & 0xffff) ^ 0x9e3779b9);
    let r = (h % 100) as u32;
    if damp {
        if r < 38 {
            "minecraft:mossy_stone_bricks"
        } else if r < 68 {
            "minecraft:stone_bricks"
        } else if r < 85 {
            "minecraft:andesite"
        } else {
            "minecraft:cracked_stone_bricks"
        }
    } else if r < 58 {
        "minecraft:stone_bricks"
    } else if r < 76 {
        "minecraft:andesite"
    } else if r < 90 {
        "minecraft:cracked_stone_bricks"
    } else {
        "minecraft:mossy_stone_bricks"
    }
}

fn build_structure(w: &mut World) {
    // 1. Broken Bridge Abutment (West: x=2096..2100) & East Pier Stub (x=2108..2111)
    // West abutment
    for z in -2i32..=2 {
        for x in 2096..=2100 {
            let max_x = if z.abs() == 2 { 2099 } else { 2100 };
            if x > max_x {
                continue;
            }
            for y in -52..=-39 {
                let b = if y == -39 && z.abs() <= 1 {
                    weathered_stone(x, y, z, false)
                } else {
                    weathered_stone(x, y, z, y < -45)
                };
                w.set(x, y, z, Phase::Structure, b);
            }
            // Broken jagged edge at x=2100
            if z.abs() == 2 {
                w.set(x, -38, z, Phase::Structure, "minecraft:stone_brick_wall[up=true,north=none,south=none,east=low,west=low]");
            } else {
                w.set(x, -38, z, Phase::Structure, "minecraft:air");
            }
        }
    }
    // Arch springer hanging off west abutment at x=2101
    for z in -1..=1 {
        w.set(2101, -41, z, Phase::Structure, "minecraft:stone_brick_stairs[facing=west,half=top,shape=straight]");
        w.set(2101, -40, z, Phase::Structure, "minecraft:stone_bricks");
    }
    // East pier stub
    for z in -2..=2 {
        for x in 2108..=2111 {
            if x == 2108 && z == -2 {
                continue; // shattered by landslide
            }
            for y in -52..=-39 {
                w.set(x, y, z, Phase::Structure, weathered_stone(x, y, z, y < -45));
            }
            w.set(x, -38, z, Phase::Structure, if z.abs() == 2 && x >= 2110 {
                "minecraft:mossy_stone_brick_wall[up=true,north=none,south=none,east=low,west=low]"
            } else {
                "minecraft:air"
            });
        }
    }

    // 2. Clear interior air volumes for Gatehouse, Porch, Vestibule, Hearth Room, Alcove
    w.fill(2091, -37, 0, 2095, -33, 2, Phase::Structure, "minecraft:air");
    w.fill(2085, -37, 0, 2090, -33, 4, Phase::Structure, "minecraft:air");
    w.fill(2075, -37, -10, 2088, -31, -2, Phase::Structure, "minecraft:air");
    w.fill(2071, -38, -7, 2074, -34, -3, Phase::Structure, "minecraft:air");

    // 3. Old Gatehouse Arch (x=2091..2095, z=-2..+4, floor y=-38)
    for x in 2091..=2095 {
        for z in -2..=4 {
            // Road floor across z=0..2, pier foundations at z=-2..-1 and z=3..4
            w.set(x, -38, z, Phase::Structure, weathered_stone(x, -38, z, false));
            if z <= -1 || z >= 3 {
                for y in -37..=-32 {
                    w.set(x, y, z, Phase::Structure, weathered_stone(x, y, z, z == -2));
                }
            } else {
                // Arch vault ceiling at y=-34 (haunches) and y=-33..-32 (stone deck & parapet)
                if z == 0 {
                    w.set(x, -34, z, Phase::Structure, "minecraft:stone_brick_stairs[facing=south,half=top,shape=straight]");
                } else if z == 2 {
                    w.set(x, -34, z, Phase::Structure, "minecraft:stone_brick_stairs[facing=north,half=top,shape=straight]");
                }
                w.set(x, -33, z, Phase::Structure, weathered_stone(x, -33, z, false));
                w.set(x, -32, z, Phase::Structure, weathered_stone(x, -32, z, false));
            }
        }
    }
    // Parapet crenellations atop the Gatehouse (y=-31)
    for x in 2091..=2095 {
        for z in -2..=4 {
            let is_edge = x == 2091 || x == 2095 || z == -2 || z == 4;
            if is_edge && !(x == 2095 && z <= -1) && ((x + z) % 2 == 0) {
                w.set(x, -31, z, Phase::Structure, "minecraft:stone_bricks");
            }
        }
    }

    // 4. Era 1 Stone Core Walls & Floors (Hearth Room + Vestibule + Alcove)
    // Main floor y=-38
    w.fill(2075, -38, -10, 2089, -38, -1, Phase::Structure, "minecraft:spruce_planks");
    // Hearth apron embedded in floor (polished andesite)
    w.fill(2080, -38, -10, 2083, -38, -9, Phase::Structure, "minecraft:polished_andesite");
    // Vestibule stone threshold & entry floor
    w.set(2086, -38, -1, Phase::Structure, "minecraft:polished_andesite");
    w.fill(2085, -38, -4, 2087, -38, -2, Phase::Structure, "minecraft:andesite");
    // Covered porch stone paving
    w.fill(2085, -38, 0, 2090, -38, 3, Phase::Structure, "minecraft:andesite");
    w.set(2087, -38, 1, Phase::Structure, "minecraft:stone_bricks");
    w.set(2088, -38, 2, Phase::Structure, "minecraft:stone_bricks");

    // North stone core wall (z=-11, x=2075..2089) up to eaves y=-34
    for x in 2075..=2089 {
        for y in -38..=-34 {
            // Landslide repair on NE corner (x=2088..2089, y=-35..-34)
            let b = if x >= 2088 && y >= -35 {
                "minecraft:spruce_planks"
            } else {
                weathered_stone(x, y, -11, true)
            };
            w.set(x, y, -11, Phase::Structure, b);
        }
    }

    // Chimney & Hearth Mass (x=2080..2083, z=-12..-10, rising to y=-26)
    for y in -39..=-26 {
        for z in -12..=-10 {
            for x in 2080..=2083 {
                if y > -34 && z == -10 {
                    continue; // chimney steps back to z=-12..-11 above mantel
                }
                if (x == 2081 || x == 2082) && z == -11 && y >= -37 && y <= -27 {
                    // Flue interior
                    w.set(x, y, z, Phase::Structure, "minecraft:air");
                    continue;
                }
                w.set(x, y, z, Phase::Structure, weathered_stone(x, y, z, z == -12));
            }
        }
    }
    // Firebox inner soot back (hidden inside cliff at y=-37..-36) & top flue rim soot
    for y in -37..=-36 {
        w.set(2081, y, -12, Phase::Structure, "minecraft:basalt");
        w.set(2082, y, -12, Phase::Structure, "minecraft:basalt");
    }
    w.set(2081, -26, -12, Phase::Structure, "minecraft:basalt");
    w.set(2082, -26, -12, Phase::Structure, "minecraft:basalt");
    // Chimney rain cap at y=-25..-24
    w.set(2080, -25, -12, Phase::Structure, "minecraft:stone_brick_wall[up=true,north=none,south=none,east=none,west=none]");
    w.set(2083, -25, -12, Phase::Structure, "minecraft:stone_brick_wall[up=true,north=none,south=none,east=none,west=none]");
    w.set(2080, -25, -11, Phase::Structure, "minecraft:stone_brick_wall[up=true,north=none,south=none,east=none,west=none]");
    w.set(2083, -25, -11, Phase::Structure, "minecraft:stone_brick_wall[up=true,north=none,south=none,east=none,west=none]");
    w.fill(2080, -24, -12, 2083, -24, -11, Phase::Structure, "minecraft:stone_brick_slab[type=bottom]");

    // Carve firebox mouth (x=2081..2082, z=-10, y=-37..-36)
    w.fill(2081, -37, -10, 2082, -36, -10, Phase::Structure, "minecraft:air");
    // Hearth cheeks: salvaged bridge stone on west (2080), ONE carried tuff brick on east (2083)
    w.set(2080, -37, -10, Phase::Structure, "minecraft:mossy_stone_bricks");
    w.set(2083, -37, -10, Phase::Structure, "minecraft:chiseled_tuff_bricks");
    // Smoke-darkened mantel
    for x in 2080..=2083 {
        w.set(x, -35, -10, Phase::Structure, "minecraft:dark_oak_slab[type=bottom]");
    }

    // Bond East Wall (x=2089) cleanly to Gatehouse Arch (x=2091) at z=-2..-1
    for z in -2..=-1 {
        for y in -38..=-32 {
            w.set(2090, y, z, Phase::Structure, weathered_stone(2090, y, z, z == -2));
        }
    }

    // East wall of Hearth Room & Vestibule (x=2089, z=-10..-1)
    for z in -10..=-1 {
        for y in -38..=-34 {
            let b = if y == -38 {
                weathered_stone(2089, y, z, false)
            } else if z == -11 || z >= -4 {
                weathered_stone(2089, y, z, false)
            } else {
                weathered_stone(2089, y, z, z <= -9)
            };
            w.set(2089, y, z, Phase::Structure, b);
        }
    }
    // NE landmark window opening (x=2089, z=-8..-7, y=-36..-35)
    w.set(2089, -37, -8, Phase::Structure, "minecraft:polished_andesite");
    w.set(2089, -37, -7, Phase::Structure, "minecraft:polished_andesite");
    w.fill(2089, -36, -8, 2089, -35, -7, Phase::Structure, "minecraft:air");

    // South wall (z=-1, x=2075..2089): Era 2 timber wall over low stone plinth
    for x in 2075..=2089 {
        w.set(x, -38, -1, Phase::Structure, weathered_stone(x, -38, -1, false));
        w.set(x, -37, -1, Phase::Structure, if x == 2075 || x == 2084 || x == 2088 || x == 2089 {
            "minecraft:spruce_log[axis=y]"
        } else {
            "minecraft:spruce_planks"
        });
        w.set(x, -36, -1, Phase::Structure, if x == 2075 || x == 2084 || x == 2088 || x == 2089 {
            "minecraft:spruce_log[axis=y]"
        } else {
            "minecraft:spruce_planks"
        });
        w.set(x, -35, -1, Phase::Structure, "minecraft:stripped_spruce_log[axis=x]");
        w.set(x, -34, -1, Phase::Structure, "minecraft:spruce_planks");
    }
    // Small hearth-room south window slit at x=2080, z=-1, y=-36
    w.set(2080, -36, -1, Phase::Structure, "minecraft:glass_pane[north=false,south=false,east=true,west=true]");
    // Low later wooden doorway at x=2086, z=-1, y=-37..-36
    w.set(2086, -37, -1, Phase::Structure, "minecraft:air");
    w.set(2086, -36, -1, Phase::Structure, "minecraft:air");

    // Vestibule internal partitions (z=-5 for x=2085..2088; x=2084 for z=-4..-2) + 3-block ceiling at y=-34
    for x in 2085..=2088 {
        for y in -37..=-34 {
            w.set(x, y, -5, Phase::Structure, "minecraft:spruce_planks");
        }
    }
    for z in -4..=-2 {
        for y in -37..=-34 {
            w.set(2084, y, z, Phase::Structure, "minecraft:spruce_planks");
        }
    }
    // Inner vestibule -> hearth room doorway at x=2084, z=-3, y=-37..-36
    w.set(2084, -37, -3, Phase::Structure, "minecraft:air");
    w.set(2084, -36, -3, Phase::Structure, "minecraft:air");
    // Vestibule low ceiling at y=-34 (gives 3-block height: -37, -36, -35)
    w.fill(2085, -34, -4, 2088, -34, -2, Phase::Structure, "minecraft:spruce_planks");
    // Vestibule south-east corner post
    w.set(2088, -37, -4, Phase::Structure, "minecraft:stripped_spruce_log[axis=y]");
    w.set(2088, -36, -4, Phase::Structure, "minecraft:stripped_spruce_log[axis=y]");

    // West wall of Hearth Room (x=2075, z=-10..-2)
    for z in -10..=-2 {
        for y in -38..=-34 {
            let b = if z <= -8 {
                weathered_stone(2075, y, z, true)
            } else if y == -38 {
                "minecraft:stone_bricks"
            } else {
                "minecraft:spruce_planks"
            };
            w.set(2075, y, z, Phase::Structure, b);
        }
    }
    // Threshold opening into Quiet Alcove at x=2075, z=-6..-5, y=-37..-35
    w.fill(2075, -37, -6, 2075, -35, -5, Phase::Structure, "minecraft:air");
    // Timber lintel above alcove threshold
    w.set(2075, -34, -6, Phase::Structure, "minecraft:stripped_spruce_log[axis=z]");
    w.set(2075, -34, -5, Phase::Structure, "minecraft:stripped_spruce_log[axis=z]");

    // Sunken Quiet West Alcove (x=2070..2074, z=-8..-2, floor y=-39)
    w.fill(2070, -39, -8, 2074, -39, -2, Phase::Structure, "minecraft:spruce_planks");
    // Step down into alcove at x=2074, z=-6..-5
    w.set(2074, -38, -6, Phase::Structure, "minecraft:spruce_slab[type=bottom]");
    w.set(2074, -38, -5, Phase::Structure, "minecraft:spruce_slab[type=bottom]");
    // Alcove walls (north z=-8, south z=-2, west x=2070)
    for x in 2070..=2074 {
        for y in -39..=-35 {
            w.set(x, y, -8, Phase::Structure, if y == -39 { "minecraft:stone_bricks" } else { "minecraft:spruce_planks" });
            w.set(x, y, -2, Phase::Structure, if y == -39 { "minecraft:stone_bricks" } else { "minecraft:spruce_planks" });
        }
    }
    for z in -8..=-2 {
        for y in -41..=-35 {
            let b = if y <= -38 {
                weathered_stone(2070, y, z, true)
            } else if z == -8 || z == -2 {
                "minecraft:spruce_log[axis=y]"
            } else {
                "minecraft:spruce_planks"
            };
            w.set(2070, y, z, Phase::Structure, b);
        }
    }
    // Two-block valley window at x=2070, z=-6..-5, y=-36..-35
    w.fill(2070, -36, -6, 2070, -35, -5, Phase::Structure, "minecraft:air");
    w.set(2070, -37, -6, Phase::Structure, "minecraft:spruce_slab[type=top]");
    w.set(2070, -37, -5, Phase::Structure, "minecraft:spruce_slab[type=top]");
    // Window lintel & north/south wall tops at y=-34
    for z in -8..=-2 {
        w.set(2070, -34, z, Phase::Structure, "minecraft:stripped_spruce_log[axis=z]");
    }
    for x in 2071..=2074 {
        w.set(x, -34, -8, Phase::Structure, "minecraft:spruce_planks");
        w.set(x, -34, -2, Phase::Structure, "minecraft:spruce_planks");
    }
    // Alcove ceiling at y=-34 (with stripped tie-beam at y=-35 along x=2072)
    w.fill(2071, -34, -7, 2074, -34, -3, Phase::Structure, "minecraft:spruce_planks");
    for z in -7..=-3 {
        w.set(2072, -35, z, Phase::Structure, "minecraft:stripped_spruce_log[axis=x]");
    }

    // Gable ends & Dominant Steep 45-degree Alpine Roof over Main Building (x=2074..2090, z=-12..+1)
    // Ridge along z=-6..-5
    for x in 2074..=2090 {
        // West gable wall at x=2075, East gable wall at x=2089
        if x == 2075 || x == 2089 {
            for z in -11..=-1 {
                let roof_y = if z <= -6 { -28 - (-6 - z) } else { -28 - (z + 5) };
                for y in -33..roof_y {
                    w.set(x, y, z, Phase::Structure, if x == 2089 && y <= -33 {
                        weathered_stone(x, y, z, false)
                    } else {
                        "minecraft:spruce_planks"
                    });
                }
            }
        }
        // Exposed interior tie beams at x=2079 and x=2084
        if x == 2079 || x == 2084 {
            for z in -10..=-2 {
                w.set(x, -33, z, Phase::Structure, "minecraft:stripped_spruce_log[axis=z]");
            }
        }

        // Sloping 45-degree spruce roof (overhangs east/west to x=2074 and x=2090, north z=-12, south z=+1)
        for z in -12..=1 {
            // Leave chimney penetration open at x=2080..2083, z=-12..-11
            if x >= 2080 && x <= 2083 && z <= -11 {
                continue;
            }
            // Era 3 repair patch on south-west slope (x=2075..2081, z=-4..+1)
            let is_repaired_patch = x >= 2075 && x <= 2081 && z >= -4;
            let weathered = !is_repaired_patch && z <= -7 && hash2(x, z, 222) < 0.25;

            if z == -6 {
                w.set(x, -28, z, Phase::Structure, "minecraft:spruce_stairs[facing=south,half=bottom,shape=straight]");
                w.set(x, -29, z, Phase::Structure, "minecraft:spruce_planks");
            } else if z == -5 {
                w.set(x, -28, z, Phase::Structure, "minecraft:spruce_stairs[facing=north,half=bottom,shape=straight]");
                w.set(x, -29, z, Phase::Structure, "minecraft:spruce_planks");
            } else if z < -6 {
                let ry = -28 - (-6 - z);
                let blk = if weathered {
                    "minecraft:dark_oak_stairs[facing=south,half=bottom,shape=straight]"
                } else {
                    "minecraft:spruce_stairs[facing=south,half=bottom,shape=straight]"
                };
                w.set(x, ry, z, Phase::Structure, blk);
                if z >= -11 && x >= 2075 && x <= 2089 {
                    w.set(x, ry - 1, z, Phase::Structure, "minecraft:spruce_planks");
                }
            } else {
                let ry = -28 - (z + 5);
                let blk = if weathered {
                    "minecraft:dark_oak_stairs[facing=north,half=bottom,shape=straight]"
                } else {
                    "minecraft:spruce_stairs[facing=north,half=bottom,shape=straight]"
                };
                w.set(x, ry, z, Phase::Structure, blk);
                if z <= -1 && x >= 2075 && x <= 2089 {
                    w.set(x, ry - 1, z, Phase::Structure, "minecraft:spruce_planks");
                }
            }
        }
    }
    // Roof ridge cap slabs
    for x in 2074..=2090 {
        w.set(x, -27, -6, Phase::Structure, "minecraft:spruce_slab[type=bottom]");
        w.set(x, -27, -5, Phase::Structure, "minecraft:spruce_slab[type=bottom]");
    }

    // Secondary smaller lean-to roof over West Alcove (x=2069..2073, tucking cleanly under main eave at x=2074)
    for z in -8..=-2 {
        // Fill vertical gap between alcove ceiling (-34) and main roof eave at x=2074
        let main_ry = if z <= -6 { -28 - (-6 - z) } else { -28 - (z + 5) };
        for y in -33..main_ry {
            w.set(2074, y, z, Phase::Structure, "minecraft:spruce_planks");
        }
    }
    for z in -9..=-1 {
        let roof_specs: [(i32, i32, &str); 5] = [
            (2073, -32, "minecraft:spruce_slab[type=top]"),
            (2072, -32, "minecraft:spruce_slab[type=bottom]"),
            (2071, -33, "minecraft:spruce_slab[type=top]"),
            (2070, -33, "minecraft:spruce_slab[type=bottom]"),
            (2069, -34, "minecraft:spruce_slab[type=top]"),
        ];
        for &(x, ry, blk) in &roof_specs {
            w.set(x, ry, z, Phase::Structure, blk);
            if (z == -8 || z == -2) && x >= 2070 {
                for y in -33..ry {
                    w.set(x, y, z, Phase::Structure, "minecraft:spruce_planks");
                }
            }
        }
    }
}

fn build_details_and_annex(w: &mut World) {
    // 1. Windows & Shutters
    // East landmark window (x=2089, z=-8..-7, y=-36..-35)
    w.set(2089, -36, -8, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    w.set(2089, -35, -8, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    w.set(2089, -36, -7, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    w.set(2089, -35, -7, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    // Exterior shutters folded open
    w.set(2090, -36, -9, Phase::Details, "minecraft:spruce_trapdoor[facing=east,half=bottom,open=true]");
    w.set(2090, -35, -9, Phase::Details, "minecraft:spruce_trapdoor[facing=east,half=top,open=true]");
    w.set(2090, -36, -6, Phase::Details, "minecraft:spruce_trapdoor[facing=east,half=bottom,open=true]");
    w.set(2090, -35, -6, Phase::Details, "minecraft:spruce_trapdoor[facing=east,half=top,open=true]");

    // West alcove two-block window (x=2070, z=-6..-5, y=-36..-35)
    w.set(2070, -36, -6, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    w.set(2070, -35, -6, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    w.set(2070, -36, -5, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    w.set(2070, -35, -5, Phase::Details, "minecraft:glass_pane[north=true,south=true,east=false,west=false]");
    w.set(2069, -36, -7, Phase::Details, "minecraft:spruce_trapdoor[facing=west,half=bottom,open=true]");
    w.set(2069, -35, -7, Phase::Details, "minecraft:spruce_trapdoor[facing=west,half=top,open=true]");
    w.set(2069, -36, -4, Phase::Details, "minecraft:spruce_trapdoor[facing=west,half=bottom,open=true]");
    w.set(2069, -35, -4, Phase::Details, "minecraft:spruce_trapdoor[facing=west,half=top,open=true]");

    // 2. Low Later Door (x=2086, z=-1, y=-37..-36)
    w.set(2086, -37, -1, Phase::Details, "minecraft:spruce_door[facing=north,half=lower,hinge=left,open=false]");
    w.set(2086, -36, -1, Phase::Details, "minecraft:spruce_door[facing=north,half=upper,hinge=left,open=false]");

    // 3. Covered Porch Fittings (z=0..1, under south roof eave at y=-34)
    // Porch support post at west end (x=2084, z=1) leaving gatehouse exit lane z=1..2 clear
    w.set(2084, -38, 1, Phase::Details, "minecraft:cobblestone");
    w.set(2084, -37, 1, Phase::Details, "minecraft:spruce_fence[north=false,south=false,east=false,west=false]");
    w.set(2084, -36, 1, Phase::Details, "minecraft:spruce_fence[north=false,south=false,east=false,west=false]");
    w.set(2084, -35, 1, Phase::Details, "minecraft:spruce_log[axis=y]");
    // East corner porch bracket against gatehouse pier (x=2090, z=0) reaching roof at y=-33
    w.set(2090, -37, 0, Phase::Details, "minecraft:cobblestone");
    w.set(2090, -36, 0, Phase::Details, "minecraft:spruce_log[axis=y]");
    w.set(2090, -35, 0, Phase::Details, "minecraft:spruce_log[axis=y]");
    w.set(2090, -34, 0, Phase::Details, "minecraft:spruce_log[axis=y]");

    // Entrance bench & pack drop beside door
    w.set(2088, -37, 0, Phase::Details, "minecraft:spruce_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2087, -37, 0, Phase::Details, "minecraft:spruce_slab[type=bottom]");
    // Rainwater collection cauldron under the roof drip line
    w.set(2085, -37, 0, Phase::Details, "minecraft:cauldron");
    w.set(2085, -38, 0, Phase::Details, "minecraft:mossy_cobblestone");
    // Stacked firewood & chopping block along south wall (x=2078..2083, z=0)
    for x in 2078..=2081 {
        w.set(x, -37, 0, Phase::Details, "minecraft:spruce_log[axis=x]");
    }
    w.set(2079, -36, 0, Phase::Details, "minecraft:spruce_log[axis=x]");
    w.set(2080, -36, 0, Phase::Details, "minecraft:spruce_log[axis=x]");
    w.set(2083, -37, 0, Phase::Details, "minecraft:spruce_log[axis=y]");

    // Directional weathering: moss on shaded north roof overhang (except near warm chimney!)
    for x in 2074..=2090 {
        if x >= 2079 && x <= 2084 {
            continue; // chimney warmth keeps north roof snow-free and dry
        }
        if hash2(x, -12, 707) < 0.45 {
            w.set(x, -33, -12, Phase::Details, "minecraft:moss_carpet");
        } else {
            w.set(x, -33, -12, Phase::Details, "minecraft:snow[layers=1]");
        }
    }

    // 4. Small Stable & Maintenance Shed (x=2079..2088, z=5..9, across the lane)
    w.fill(2079, -38, 5, 2088, -38, 9, Phase::Details, "minecraft:coarse_dirt");
    w.fill(2080, -38, 6, 2082, -38, 8, Phase::Details, "minecraft:andesite"); // dry floor for tool bench
    // South retaining wall (z=9) and side stone/timber walls
    for x in 2079..=2088 {
        for y in -38..=-34 {
            w.set(x, y, 9, Phase::Details, weathered_stone(x, y, 9, true));
        }
    }
    for z in 5..=8 {
        for y in -37..=-35 {
            w.set(2079, y, z, Phase::Details, if z == 5 { "minecraft:spruce_log[axis=y]" } else { "minecraft:spruce_planks" });
            w.set(2088, y, z, Phase::Details, if z == 5 { "minecraft:spruce_log[axis=y]" } else { "minecraft:spruce_planks" });
        }
    }
    // Middle stall divider (x=2083, z=5..8)
    for z in 6..=8 {
        w.set(2083, -37, z, Phase::Details, "minecraft:spruce_planks");
        w.set(2083, -36, z, Phase::Details, "minecraft:spruce_fence[north=true,south=true,east=false,west=false]");
    }
    for y in -37..=-35 {
        w.set(2083, y, 5, Phase::Details, "minecraft:spruce_log[axis=y]");
    }
    // Stable front gate at z=5 (x=2084..2087)
    w.set(2084, -37, 5, Phase::Details, "minecraft:spruce_fence[north=false,south=false,east=true,west=true]");
    w.set(2085, -37, 5, Phase::Details, "minecraft:spruce_fence_gate[facing=north,open=false,in_wall=false]");
    w.set(2086, -37, 5, Phase::Details, "minecraft:spruce_fence[north=false,south=false,east=true,west=true]");
    w.set(2087, -37, 5, Phase::Details, "minecraft:spruce_fence[north=false,south=false,east=true,west=true]");
    // Shed roof sloping down toward lane (z=9 high -> z=5 low)
    for x in 2079..=2088 {
        w.set(x, -33, 9, Phase::Details, "minecraft:spruce_slab[type=bottom]");
        w.set(x, -34, 8, Phase::Details, "minecraft:spruce_stairs[facing=south,half=bottom,shape=straight]");
        w.set(x, -34, 7, Phase::Details, "minecraft:spruce_slab[type=bottom]");
        w.set(x, -35, 6, Phase::Details, "minecraft:spruce_stairs[facing=south,half=bottom,shape=straight]");
        w.set(x, -35, 5, Phase::Details, "minecraft:spruce_slab[type=bottom]");
    }

    // 5. Protected Alpine Garden (x=2071..2077, z=6..10)
    for z in 6..=10 {
        for x in 2071..=2077 {
            let is_border = x == 2071 || x == 2077 || z == 10;
            if is_border {
                w.set(x, -38, z, Phase::Details, "minecraft:cobblestone");
                let north = if (x == 2071 || x == 2077) && z > 6 { "low" } else { "none" };
                let south = if (x == 2071 || x == 2077) && z < 10 { "low" } else { "none" };
                let east = if z == 10 && x < 2077 { "low" } else { "none" };
                let west = if z == 10 && x > 2071 { "low" } else { "none" };
                let w_wall = format!(
                    "minecraft:cobblestone_wall[up=true,north={},south={},east={},west={}]",
                    north, south, east, west
                );
                w.set(x, -37, z, Phase::Details, &w_wall);
            } else {
                w.set(x, -38, z, Phase::Details, if (x + z) % 2 == 0 { "minecraft:rooted_dirt" } else { "minecraft:moss_block" });
            }
        }
    }
    // Contained 1x1 stone spring trough in garden (surrounded on all 4 sides and bottom by andesite)
    w.set(2074, -39, 8, Phase::Details, "minecraft:andesite");
    w.set(2073, -38, 8, Phase::Details, "minecraft:andesite");
    w.set(2075, -38, 8, Phase::Details, "minecraft:andesite");
    w.set(2074, -38, 7, Phase::Details, "minecraft:andesite");
    w.set(2074, -38, 9, Phase::Details, "minecraft:andesite");
    w.set(2074, -38, 8, Phase::Details, "minecraft:water");
    w.set(2072, -37, 7, Phase::Details, "minecraft:azalea");
    w.set(2076, -37, 9, Phase::Details, "minecraft:flowering_azalea");
    w.set(2073, -37, 9, Phase::Details, "minecraft:fern");
    w.set(2075, -37, 7, Phase::Details, "minecraft:lily_of_the_valley");
    w.set(2072, -37, 9, Phase::Details, "minecraft:cornflower");
}

fn build_interiors_and_light(w: &mut World) {
    // 1. Hearth Firebox (warm ember glow without fire or lava)
    w.set(2081, -38, -11, Phase::Interiors, "minecraft:shroomlight");
    w.set(2082, -38, -11, Phase::Interiors, "minecraft:shroomlight");
    w.set(2081, -37, -11, Phase::Interiors, "minecraft:iron_bars[north=true,south=true,east=true,west=true]");
    w.set(2082, -37, -11, Phase::Interiors, "minecraft:iron_bars[north=true,south=true,east=true,west=true]");
    w.set(2081, -37, -10, Phase::Interiors, "minecraft:coal_block");
    w.set(2082, -36, -10, Phase::Interiors, "minecraft:cauldron");
    w.set(2082, -35, -10, Phase::Interiors, "minecraft:iron_chain[axis=y]");

    // Hearth seating facing the fire
    w.set(2080, -37, -7, Phase::Interiors, "minecraft:spruce_stairs[facing=north,half=bottom,shape=straight]");
    w.set(2083, -37, -7, Phase::Interiors, "minecraft:spruce_stairs[facing=north,half=bottom,shape=straight]");

    // 2. Kitchen & Provisions Corner (NW Hearth Room)
    w.set(2076, -37, -10, Phase::Interiors, "minecraft:cauldron");
    w.set(2077, -37, -10, Phase::Interiors, "minecraft:furnace[facing=south]");
    w.set(2078, -37, -10, Phase::Interiors, "minecraft:smoker[facing=south]");
    w.set(2079, -37, -10, Phase::Interiors, "minecraft:barrel[facing=south]");
    w.set(2079, -36, -10, Phase::Interiors, "minecraft:barrel[facing=south]"); // added later cupboard
    w.set(2076, -37, -9, Phase::Interiors, "minecraft:crafting_table");
    w.set(2076, -37, -8, Phase::Interiors, "minecraft:chest[facing=east,type=single]");

    // 3. NE Bridge-View Window Bay (communal map & tea table)
    w.set(2088, -37, -8, Phase::Interiors, "minecraft:spruce_slab[type=top]");
    w.set(2088, -37, -7, Phase::Interiors, "minecraft:spruce_slab[type=top]");
    w.set(2088, -36, -8, Phase::Interiors, "minecraft:flower_pot");
    w.set(2088, -36, -7, Phase::Interiors, "minecraft:lantern[hanging=false]");
    // Table & chairs at x=2086, z=-8..-7
    w.set(2086, -37, -8, Phase::Interiors, "minecraft:spruce_stairs[facing=west,half=top,shape=straight]");
    w.set(2086, -37, -7, Phase::Interiors, "minecraft:spruce_stairs[facing=west,half=top,shape=straight]");
    w.set(2086, -36, -8, Phase::Interiors, "minecraft:white_candle[candles=2,lit=false]");
    w.set(2085, -37, -8, Phase::Interiors, "minecraft:spruce_stairs[facing=east,half=bottom,shape=straight]");
    w.set(2085, -37, -7, Phase::Interiors, "minecraft:spruce_stairs[facing=east,half=bottom,shape=straight]");
    w.set(2087, -31, -7, Phase::Interiors, "minecraft:iron_chain[axis=y]");
    w.set(2087, -32, -7, Phase::Interiors, "minecraft:iron_chain[axis=y]");
    w.set(2087, -33, -7, Phase::Interiors, "minecraft:lantern[hanging=true]");

    // 4. Sleeping Alcoves along South Wall (z=-2)
    w.set(2077, -37, -2, Phase::Interiors, "minecraft:brown_bed[facing=west,part=head]");
    w.set(2078, -37, -2, Phase::Interiors, "minecraft:brown_bed[facing=west,part=foot]");
    w.set(2079, -37, -2, Phase::Interiors, "minecraft:spruce_trapdoor[facing=west,half=bottom,open=true]");
    w.set(2080, -37, -2, Phase::Interiors, "minecraft:barrel[facing=up]");
    w.set(2080, -36, -2, Phase::Interiors, "minecraft:white_candle[candles=1,lit=false]");
    w.set(2081, -37, -2, Phase::Interiors, "minecraft:red_bed[facing=east,part=foot]");
    w.set(2082, -37, -2, Phase::Interiors, "minecraft:red_bed[facing=east,part=head]");
    // Bunk canopy shelf above beds
    for x in 2077..=2082 {
        w.set(x, -35, -2, Phase::Interiors, "minecraft:spruce_slab[type=top]");
    }
    w.set(2077, -34, -2, Phase::Interiors, "minecraft:potted_lily_of_the_valley");
    // Central hearth room hanging lantern on wrought chain from ridge ceiling
    w.set(2081, -30, -5, Phase::Interiors, "minecraft:iron_chain[axis=y]");
    w.set(2081, -31, -5, Phase::Interiors, "minecraft:iron_chain[axis=y]");
    w.set(2081, -32, -5, Phase::Interiors, "minecraft:lantern[hanging=true]");

    // 5. Vestibule Interior (3x3x3: x=2085..2087, z=-4..-2)
    w.set(2087, -37, -4, Phase::Interiors, "minecraft:spruce_stairs[facing=west,half=bottom,shape=straight]");
    w.set(2087, -36, -3, Phase::Interiors, "minecraft:spruce_shelf[facing=west]");
    w.set(2086, -35, -3, Phase::Interiors, "minecraft:lantern[hanging=true]");

    // 6. Partition Screen & Quiet Sunken West Alcove (x=2071..2075, z=-7..-3, floor y=-39)
    // Partial screen at x=2075
    w.set(2075, -37, -7, Phase::Interiors, "minecraft:bookshelf");
    w.set(2075, -36, -7, Phase::Interiors, "minecraft:bookshelf");
    w.set(2075, -37, -4, Phase::Interiors, "minecraft:chiseled_bookshelf[facing=east]");
    w.set(2075, -36, -4, Phase::Interiors, "minecraft:bookshelf");
    // Window seat beneath the 2-block valley window (x=2071, z=-7..-4)
    w.set(2071, -38, -7, Phase::Interiors, "minecraft:spruce_stairs[facing=south,half=bottom,shape=straight]");
    w.set(2071, -38, -6, Phase::Interiors, "minecraft:spruce_planks");
    w.set(2071, -38, -5, Phase::Interiors, "minecraft:spruce_planks");
    w.set(2071, -37, -6, Phase::Interiors, "minecraft:brown_carpet");
    w.set(2071, -37, -5, Phase::Interiors, "minecraft:brown_carpet");
    w.set(2071, -38, -4, Phase::Interiors, "minecraft:spruce_stairs[facing=north,half=bottom,shape=straight]");
    // Lectern for traveller's map/messages, tea side-table, quiet books, and subdued lantern
    w.set(2073, -38, -7, Phase::Interiors, "minecraft:lectern[facing=south]");
    w.set(2072, -38, -7, Phase::Interiors, "minecraft:spruce_slab[type=top]");
    w.set(2072, -37, -7, Phase::Interiors, "minecraft:potted_fern");
    w.set(2073, -38, -3, Phase::Interiors, "minecraft:bookshelf");
    w.set(2073, -37, -3, Phase::Interiors, "minecraft:lantern[hanging=false]");
    w.set(2074, -37, -3, Phase::Interiors, "minecraft:spruce_shelf[facing=north]");

    // 7. Porch, Gatehouse, Detour Safety, Maintenance Bay, and Stable Lights & Gear
    w.set(2087, -34, 0, Phase::Interiors, "minecraft:iron_chain[axis=y]");
    w.set(2087, -35, 0, Phase::Interiors, "minecraft:lantern[hanging=true]");
    w.set(2093, -34, 1, Phase::Interiors, "minecraft:lantern[hanging=true]");
    // Broken bridge brink warning post & detour footbridge crossing lantern
    w.set(2112, -40, 2, Phase::Interiors, "minecraft:cobblestone");
    w.set(2112, -39, 2, Phase::Interiors, "minecraft:stone_bricks");
    w.set(2112, -38, 2, Phase::Interiors, "minecraft:spruce_fence[north=false,south=false,east=false,west=false]");
    w.set(2112, -37, 2, Phase::Interiors, "minecraft:lantern[hanging=false]");
    w.set(2105, -46, 13, Phase::Interiors, "minecraft:lantern[hanging=false]");
    // Maintenance workspace (x=2080..2082, z=6..8)
    w.set(2080, -37, 8, Phase::Interiors, "minecraft:stonecutter[facing=north]");
    w.set(2081, -37, 8, Phase::Interiors, "minecraft:grindstone[face=floor,facing=north]");
    w.set(2082, -37, 8, Phase::Interiors, "minecraft:anvil[facing=north]");
    w.set(2080, -37, 7, Phase::Interiors, "minecraft:barrel[facing=north]");
    w.set(2082, -37, 7, Phase::Interiors, "minecraft:stripped_spruce_log[axis=x]");
    w.set(2081, -35, 7, Phase::Interiors, "minecraft:lantern[hanging=true]");
    // Stable interior (x=2084..2087, z=6..8)
    w.set(2084, -37, 8, Phase::Interiors, "minecraft:cauldron");
    w.set(2086, -37, 8, Phase::Interiors, "minecraft:hay_block");
    w.set(2087, -37, 8, Phase::Interiors, "minecraft:hay_block");
    w.set(2087, -37, 7, Phase::Interiors, "minecraft:barrel[facing=west]");
    w.set(2085, -35, 7, Phase::Interiors, "minecraft:copper_lantern[hanging=true]");
}

fn validate(w: &World) {
    let mut tuff_count = 0usize;
    let mut non_air = 0usize;

    for y in Y_MIN..=Y_MAX {
        for z in Z_MIN..=Z_MAX {
            for x in X_MIN..=X_MAX {
                let name = w.get_name(x, y, z);
                if name == "minecraft:air" {
                    continue;
                }
                non_air += 1;
                if name.contains("lava") || name == "minecraft:fire" || name.contains("campfire") {
                    panic!("Forbidden hazard block {} at ({},{},{})", name, x, y, z);
                }
                if name.contains('{') || name.contains('}') {
                    panic!("Forbidden NBT in {} at ({},{},{})", name, x, y, z);
                }
                if name.contains("tuff") {
                    tuff_count += 1;
                }
                if name.contains("leaves") && !name.contains("persistent=true") {
                    panic!("Non-persistent leaves {} at ({},{},{})", name, x, y, z);
                }
                if name.contains("lantern") {
                    if name.contains("hanging=true") && w.is_air(x, y + 1, z) {
                        panic!("Unsupported hanging lantern {} at ({},{},{})", name, x, y, z);
                    }
                    if name.contains("hanging=false") && w.is_air(x, y - 1, z) {
                        panic!("Unsupported standing lantern {} at ({},{},{})", name, x, y, z);
                    }
                }
                if name == "minecraft:gravel" || name == "minecraft:sand" {
                    if y == Y_MIN || w.is_air(x, y - 1, z) {
                        panic!("Unsupported falling block {} at ({},{},{})", name, x, y, z);
                    }
                }
                if name == "minecraft:water" {
                    if x == X_MIN || x == X_MAX || z == Z_MIN || z == Z_MAX || y == Y_MIN {
                        panic!("Water on box boundary at ({},{},{})", x, y, z);
                    }
                    if w.is_air(x, y - 1, z)
                        || w.is_air(x + 1, y, z)
                        || w.is_air(x - 1, y, z)
                        || w.is_air(x, y, z + 1)
                        || w.is_air(x, y, z - 1)
                    {
                        panic!("Uncontained water at ({},{},{})", x, y, z);
                    }
                }
            }
        }
    }
    if tuff_count > 1 {
        panic!("Too many tuff blocks: {} (max 1 allowed)", tuff_count);
    }
    if non_air == 0 {
        panic!("World is empty");
    }
}

fn emit_commands(w: &World, out_path: &Path) -> std::io::Result<(usize, usize)> {
    let file = File::create(out_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "# The Last Hearth — x 2000..2179, z -60..59, y -61..120")?;

    let mut visited = vec![false; NX * NY * NZ];
    let mut cmd_count = 0usize;
    let mut block_count = 0usize;

    let phases = [
        (Phase::Landscape, "# 1. Landscape"),
        (Phase::Structure, "# 2. Structure"),
        (Phase::Details, "# 3. Details"),
        (Phase::Interiors, "# 4. Interiors and Light"),
    ];

    for &(target_phase, header) in &phases {
        writeln!(writer, "{}", header)?;
        for iy in 0..NY {
            for iz in 0..NZ {
                for ix in 0..NX {
                    let i = (iy * NZ + iz) * NX + ix;
                    if visited[i] {
                        continue;
                    }
                    let c = w.cells[i];
                    if c.block == 0 || c.phase != target_phase {
                        continue;
                    }

                    // Greedy 3D box merge within 32,768 block limit
                    let mut x2 = ix;
                    while x2 + 1 < NX {
                        let ni = (iy * NZ + iz) * NX + (x2 + 1);
                        if !visited[ni] && w.cells[ni].phase == c.phase && w.cells[ni].block == c.block {
                            x2 += 1;
                        } else {
                            break;
                        }
                    }

                    let mut z2 = iz;
                    'z_grow: while z2 + 1 < NZ {
                        let next_vol = (x2 - ix + 1) * (z2 - iz + 2);
                        if next_vol > 32768 {
                            break;
                        }
                        for tx in ix..=x2 {
                            let ni = (iy * NZ + (z2 + 1)) * NX + tx;
                            if visited[ni] || w.cells[ni].phase != c.phase || w.cells[ni].block != c.block {
                                break 'z_grow;
                            }
                        }
                        z2 += 1;
                    }

                    let mut y2 = iy;
                    'y_grow: while y2 + 1 < NY {
                        let next_vol = (x2 - ix + 1) * (z2 - iz + 1) * (y2 - iy + 2);
                        if next_vol > 32768 {
                            break;
                        }
                        for tz in iz..=z2 {
                            for tx in ix..=x2 {
                                let ni = ((y2 + 1) * NZ + tz) * NX + tx;
                                if visited[ni] || w.cells[ni].phase != c.phase || w.cells[ni].block != c.block {
                                    break 'y_grow;
                                }
                            }
                        }
                        y2 += 1;
                    }

                    for yy in iy..=y2 {
                        for zz in iz..=z2 {
                            for xx in ix..=x2 {
                                visited[(yy * NZ + zz) * NX + xx] = true;
                            }
                        }
                    }

                    let wx1 = X_MIN + ix as i32;
                    let wy1 = Y_MIN + iy as i32;
                    let wz1 = Z_MIN + iz as i32;
                    let wx2 = X_MIN + x2 as i32;
                    let wy2 = Y_MIN + y2 as i32;
                    let wz2 = Z_MIN + z2 as i32;
                    let bname = &w.palette[c.block as usize];
                    let vol = (x2 - ix + 1) * (z2 - iz + 1) * (y2 - iy + 1);

                    if ix == x2 && iy == y2 && iz == z2 {
                        writeln!(writer, "/setblock {} {} {} {}", wx1, wy1, wz1, bname)?;
                    } else {
                        writeln!(
                            writer,
                            "/fill {} {} {} {} {} {} {}",
                            wx1, wy1, wz1, wx2, wy2, wz2, bname
                        )?;
                    }
                    cmd_count += 1;
                    block_count += vol;
                }
            }
        }
    }

    writer.flush()?;
    Ok((cmd_count, block_count))
}

fn main() {
    let out_dir = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let out_path = Path::new(&out_dir).join("build.txt");

    let mut world = World::new();
    build_landscape(&mut world);
    build_structure(&mut world);
    build_details_and_annex(&mut world);
    build_interiors_and_light(&mut world);
    validate(&world);

    let (cmds, blocks) = emit_commands(&world, &out_path).expect("failed to write build.txt");
    eprintln!("Generated {:?}: {} commands, {} blocks", out_path, cmds, blocks);
}
