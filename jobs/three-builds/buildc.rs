// buildc: replay a build.txt into a grid, validate it, and re-emit it as few commands, safely ordered.
// Usage: buildc <build.txt> <blocks.tsv> <x0> <y0> <z0> <x1> <y1> <z1> <out.txt>
// Validates: every block id is in blocks.tsv; every coordinate is inside the box; only /fill and /setblock.
// The world before the build is superflat (grass at y = -61, air above) and the box is checked empty beforehand,
// so only cells whose final state differs from that need emitting.
// Order of the output:
//   1. full, self-supporting blocks (no state needing support), merged into 3D boxes of <= 32,768;
//   2. blocks that need support or attach (plants, lanterns, doors, carpets, torches, rails, redstone, ...), bottom-up,
//      merged along x only, lower halves before upper halves;
//   3. falling blocks, bottom-up;
//   4. water and waterlogged blocks, last.
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{BufWriter, Write};

const UNSET: u32 = 0;

fn needs_support(name: &str) -> bool {
    let id = name.split('[').next().unwrap();
    let id = id.trim_start_matches("minecraft:");
    const KEYS: &[&str] = &[
        "lantern", "torch", "door", "bed", "carpet", "rail", "redstone_wire", "repeater", "comparator",
        "button", "pressure_plate", "ladder", "vine", "sign", "banner", "candle", "sea_pickle", "lily_pad",
        "sugar_cane", "wheat", "carrots", "potatoes", "beetroots", "sapling", "flower", "tulip", "poppy",
        "dandelion", "orchid", "allium", "azure_bluet", "oxeye_daisy", "cornflower", "lily_of_the_valley",
        "short_grass", "tall_grass", "fern", "dead_bush", "sweet_berry", "petals", "bell", "lever",
        "snow", "moss_carpet", "pointed_dripstone", "scaffolding", "chain", "glow_lichen", "hanging_roots",
        "spore_blossom", "cocoa", "bamboo", "kelp", "seagrass", "coral", "mushroom", "nether_sprouts",
        "roots", "fungus", "lichen", "dripleaf", "frogspawn", "tripwire", "item_frame", "flower_pot",
        "potted_", "rose_bush", "peony", "lilac", "sunflower", "pitcher", "torchflower", "cactus",
        "melon_stem", "pumpkin_stem", "nether_wart", "leaf_litter", "bush", "firefly", "cactus_flower",
        "wildflowers", "resin_clump", "pale_moss_carpet", "pale_hanging_moss", "heavy_core", "lightning_rod",
        "end_rod", "amethyst_cluster", "amethyst_bud", "daylight_detector", "trapdoor",
    ];
    KEYS.iter().any(|k| id.contains(k))
}
fn is_falling(name: &str) -> bool {
    let id = name.split('[').next().unwrap();
    id.ends_with(":sand") || id.ends_with(":red_sand") || id.ends_with(":gravel") || id.ends_with("concrete_powder")
        || id.ends_with(":suspicious_sand") || id.ends_with(":suspicious_gravel") || id.ends_with("anvil")
        || id.ends_with(":dragon_egg") || id.ends_with("scaffolding")
}
fn is_water(name: &str) -> bool {
    name.starts_with("minecraft:water") || name.contains("waterlogged=true") || name.starts_with("minecraft:bubble_column")
}

struct Grid { x0: i32, y0: i32, z0: i32, nx: usize, ny: usize, nz: usize, cells: Vec<u32> }
impl Grid {
    fn inside(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.x0 && y >= self.y0 && z >= self.z0
            && ((x - self.x0) as usize) < self.nx && ((y - self.y0) as usize) < self.ny && ((z - self.z0) as usize) < self.nz
    }
    fn idx(&self, x: i32, y: i32, z: i32) -> usize {
        (((y - self.y0) as usize) * self.nz + (z - self.z0) as usize) * self.nx + (x - self.x0) as usize
    }
}

fn main() {
    let a: Vec<String> = env::args().collect();
    if a.len() != 10 { eprintln!("usage: buildc build.txt blocks.tsv x0 y0 z0 x1 y1 z1 out.txt"); std::process::exit(2); }
    let p = |i: usize| a[i].parse::<i32>().unwrap();
    let (x0, y0, z0, x1, y1, z1) = (p(3), p(4), p(5), p(6), p(7), p(8));
    let known: HashSet<String> = fs::read_to_string(&a[2]).unwrap().lines()
        .filter_map(|l| l.split('\t').next()).map(|s| s.trim().to_string()).collect();
    let mut g = Grid { x0, y0, z0, nx: (x1 - x0 + 1) as usize, ny: (y1 - y0 + 1) as usize, nz: (z1 - z0 + 1) as usize, cells: vec![] };
    g.cells = vec![UNSET; g.nx * g.ny * g.nz];
    let mut palette: Vec<String> = vec!["<unset>".into()];
    let mut pidx: HashMap<String, u32> = HashMap::new();
    let mut errors: Vec<String> = vec![];
    let mut unknown: HashSet<String> = HashSet::new();
    let (mut n_fill, mut n_set, mut max_vol) = (0usize, 0usize, 0i64);
    let text = fs::read_to_string(&a[1]).unwrap();
    for (ln, line) in text.lines().enumerate() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') { continue; }
        let l = l.trim_start_matches('/');
        let mut it = l.split_whitespace();
        let cmd = it.next().unwrap();
        let nums_needed = match cmd { "fill" => 6, "setblock" => 3, _ => { errors.push(format!("line {}: command {}", ln + 1, cmd)); continue; } };
        let mut v = vec![];
        for _ in 0..nums_needed { v.push(it.next().and_then(|s| s.parse::<i32>().ok())); }
        if v.iter().any(|o| o.is_none()) { errors.push(format!("line {}: bad coordinates", ln + 1)); continue; }
        let v: Vec<i32> = v.into_iter().map(|o| o.unwrap()).collect();
        let block: String = it.collect::<Vec<_>>().join(" ");
        if block.is_empty() || block.contains('{') { errors.push(format!("line {}: block data or missing block: {}", ln + 1, block)); continue; }
        let bid = block.split('[').next().unwrap().to_string();
        if !known.contains(&bid) { unknown.insert(bid.clone()); }
        let id = *pidx.entry(block.clone()).or_insert_with(|| { palette.push(block.clone()); (palette.len() - 1) as u32 });
        let (ax, ay, az, bx, by, bz) = if cmd == "fill" { (v[0].min(v[3]), v[1].min(v[4]), v[2].min(v[5]), v[0].max(v[3]), v[1].max(v[4]), v[2].max(v[5])) } else { (v[0], v[1], v[2], v[0], v[1], v[2]) };
        let vol = ((bx - ax + 1) as i64) * ((by - ay + 1) as i64) * ((bz - az + 1) as i64);
        if cmd == "fill" { n_fill += 1; max_vol = max_vol.max(vol); if vol > 32768 { errors.push(format!("line {}: fill volume {}", ln + 1, vol)); } } else { n_set += 1; }
        if !g.inside(ax, ay, az) || !g.inside(bx, by, bz) { errors.push(format!("line {}: outside the box: {}", ln + 1, l)); continue; }
        for y in ay..=by { for z in az..=bz { for x in ax..=bx { let i = g.idx(x, y, z); g.cells[i] = id; } } }
    }
    let air = pidx.get("minecraft:air").copied();
    // Which cells must be emitted: anything set, except explicit air above the ground (already air); air at y = -61 carves grass.
    let emit = |y: i32, c: u32| -> bool { c != UNSET && (Some(c) != air || y == -61) && !(y == -61 && palette[c as usize] == "minecraft:grass_block") };
    let class = |c: u32| -> u8 { let n = &palette[c as usize]; if is_water(n) { 4 } else if is_falling(n) { 3 } else if needs_support(n) { 2 } else { 1 } };
    let mut out = BufWriter::new(fs::File::create(&a[9]).unwrap());
    let mut done = vec![false; g.cells.len()];
    let mut n_out = 0usize;
    let mut counts = [0usize; 5];
    // Phase 1: greedy 3D boxes over class-1 cells.
    for y in y0..=y1 { for z in z0..=z1 { for x in x0..=x1 {
        let i = g.idx(x, y, z); let c = g.cells[i];
        if done[i] || !emit(y, c) || class(c) != 1 { continue; }
        let same = |g: &Grid, done: &Vec<bool>, xx: i32, yy: i32, zz: i32| -> bool { g.inside(xx, yy, zz) && { let j = g.idx(xx, yy, zz); !done[j] && g.cells[j] == c } };
        let mut ex = x; while ex < x1 && same(&g, &done, ex + 1, y, z) && ((ex - x + 2) as i64) <= 32768 { ex += 1; }
        let w = (ex - x + 1) as i64;
        let mut ez = z; 'zl: while ez < z1 && w * ((ez - z + 2) as i64) <= 32768 { for xx in x..=ex { if !same(&g, &done, xx, y, ez + 1) { break 'zl; } } ez += 1; }
        let d = (ez - z + 1) as i64;
        let mut ey = y; 'yl: while ey < y1 && w * d * ((ey - y + 2) as i64) <= 32768 { for zz in z..=ez { for xx in x..=ex { if !same(&g, &done, xx, ey + 1, zz) { break 'yl; } } } ey += 1; }
        for yy in y..=ey { for zz in z..=ez { for xx in x..=ex { let j = g.idx(xx, yy, zz); done[j] = true; } } }
        let b = &palette[c as usize];
        if (ex, ey, ez) == (x, y, z) { writeln!(out, "setblock {} {} {} {}", x, y, z, b).unwrap(); } else { writeln!(out, "fill {} {} {} {} {} {} {}", x, y, z, ex, ey, ez, b).unwrap(); }
        n_out += 1; counts[1] += 1;
    }}}
    // Phases 2-4: bottom-up, runs along x; lower halves before upper halves within a layer.
    for phase in 2..=4u8 {
        for y in y0..=y1 {
            for upper in [false, true] {
                for z in z0..=z1 { let mut x = x0; while x <= x1 {
                    let i = g.idx(x, y, z); let c = g.cells[i];
                    if done[i] || !emit(y, c) || class(c) != phase || palette[c as usize].contains("half=upper") != upper { x += 1; continue; }
                    let mut ex = x; while ex < x1 { let j = g.idx(ex + 1, y, z); if !done[j] && g.cells[j] == c { ex += 1; } else { break; } }
                    for xx in x..=ex { let j = g.idx(xx, y, z); done[j] = true; }
                    let b = &palette[c as usize];
                    if ex == x { writeln!(out, "setblock {} {} {} {}", x, y, z, b).unwrap(); } else { writeln!(out, "fill {} {} {} {} {} {} {}", x, y, z, ex, y, z, b).unwrap(); }
                    n_out += 1; counts[phase as usize] += 1; x = ex + 1;
                }}
            }
        }
    }
    let nonair = g.cells.iter().filter(|&&c| c != UNSET && Some(c) != air).count();
    println!("input: {} fill, {} setblock, largest fill {}; palette {} states", n_fill, n_set, max_vol, palette.len() - 1);
    println!("output: {} commands (solid boxes {}, supported {}, falling {}, water {}); non-air blocks {}", n_out, counts[1], counts[2], counts[3], counts[4], nonair);
    let mut u: Vec<_> = unknown.into_iter().collect(); u.sort();
    println!("unknown block ids: {}", if u.is_empty() { "none".to_string() } else { u.join(", ") });
    println!("errors: {}", errors.len());
    for e in errors.iter().take(20) { println!("  {}", e); }
    if !errors.is_empty() || !u.is_empty() { std::process::exit(1); }
}
