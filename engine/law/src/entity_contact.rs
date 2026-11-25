//! Mojang 26.2 entity pushing (`Entity.push`, `LivingEntity.pushEntities`,
//! `EntitySelector.pushableBy`, `AbstractBoat.push`, `Shulker`, `HappyGhast`),
//! entity cramming, solid entity colliders, and an O(N) bit-exact `SpatialCrowdIndex`
//! matching `EntitySectionStorage.forEachAccessibleNonEmptySection` traversal order.
//!
//! Ported from:
//! - `lab/results/local-20261118-1812-1-0-port-entity-push-and-crowd-loop/dossier.md`
//! - `lab/results/local-20261115-1337-1-0-port-ordered-push-query/dossier.md`
//! - `lab/results/local-20261115-1340-1-0-port-push-order-noncommuting/dossier.md`
//! - `lab/results/local-20261115-1302-1-0-port-push-threshold/dossier.md`

use crate::{math, Aabb, Vec3};

pub(crate) fn clearance(
    world: &(impl crate::World + ?Sized),
    bounds: Aabb,
    context: crate::CollisionContext,
    mut bodies: impl Iterator<Item = Aabb>,
) -> bool {
    world.no_collision(bounds, context) && !bodies.any(|body| body.intersects(bounds))
}

/// `GameRules.MAX_ENTITY_CRAMMING` default (`GameRules.<clinit> @376-388`).
pub const CRAMMING_DEFAULT_MAX: i32 = 24;

/// `LivingEntity.pushEntities @141-156`: `damageSources().cramming()` amount.
pub const CRAMMING_DAMAGE: f32 = 6.0;

/// Virtual override dispatch class for `push`, `isPushable`, and `canBeCollidedWith`.
/// Complete jar declaration lists (`ScanOverriders`, `server-26.2.jar`):
/// - `canBeCollidedWith(Entity)`: `Entity` (false), `HappyGhast`, `Shulker`, `AbstractBoat`.
/// - `canCollideWith(Entity)`: `Entity`, `AbstractWindCharge`, `AbstractBoat`, `AbstractMinecart`.
/// - `push(Entity)`: `Entity`, `LivingEntity`, `Shulker`, `AbstractCubeMob`, `AbstractBoat`, `AbstractMinecart`.
/// - `isPushable()`: `Entity` (false), `LivingEntity`, `Bat`, `AbstractHorse`, `Parrot`,
///   `ArmorStand`, `Creaking`, `Warden`, `AbstractBoat`, `AbstractMinecart`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContactKind {
    /// `LivingEntity`: `isPushable` = `alive && !spectator && !on_climbable`; `push` checks `!sleeping`.
    #[default]
    Living,
    /// `AbstractBoat`: `isPushable = true`, `canBeCollidedWith = true`, Y-gated `push`.
    Boat,
    /// `Shulker`: `canBeCollidedWith = alive`, `push @0` is a bare return (immovable).
    Shulker,
    /// `HappyGhast`: `canBeCollidedWith = alive && !baby`.
    HappyGhast { baby: bool },
    /// `AbstractMinecart`: `isPushable = true`, NOT a solid collider.
    Minecart,
    /// Other non-living entities (`Entity` defaults: `canBeCollidedWith = false`).
    Other,
}

/// `Team.CollisionRule` used by `EntitySelector.pushableBy`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TeamCollisionRule {
    #[default]
    Always,
    Never,
    PushOwnTeam,
    PushOtherTeams,
}

/// Inputs for entity pushing, solid collision selection, and `LivingEntity.pushEntities`.
#[derive(Clone, Copy, Debug)]
pub struct ContactEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub aabb: Aabb,
    pub no_physics: bool,
    pub vehicle: bool,
    pub pushable: bool,
    pub kind: ContactKind,
    pub is_passenger: bool,
    pub alive: bool,
    pub spectator: bool,
    pub on_climbable: bool,
    pub sleeping: bool,
    pub team_id: Option<u32>,
    pub team_rule: TeamCollisionRule,
}

impl Default for ContactEntity {
    fn default() -> Self {
        Self {
            position: Vec3::default(),
            velocity: Vec3::default(),
            aabb: Aabb::player(Vec3::default(), 1.8),
            no_physics: false,
            vehicle: false,
            pushable: true,
            kind: ContactKind::Living,
            is_passenger: false,
            alive: true,
            spectator: false,
            on_climbable: false,
            sleeping: false,
            team_id: None,
            team_rule: TeamCollisionRule::Always,
        }
    }
}

impl ContactEntity {
    /// Constructs a standing living player entity (`0.6 x 1.8 x 0.6` AABB).
    pub fn player(position: Vec3, velocity: Vec3) -> Self {
        Self {
            position,
            velocity,
            aabb: Aabb::player(position, 1.8),
            ..Self::default()
        }
    }

    /// Constructs a standard `AbstractBoat` (`1.375 x 0.5625 x 1.375` AABB).
    pub fn boat(position: Vec3, velocity: Vec3) -> Self {
        Self {
            position,
            velocity,
            aabb: crate::boat::boat_aabb(position),
            kind: ContactKind::Boat,
            ..Self::default()
        }
    }

    /// Constructs an alive `Shulker` (`1.0 x 1.0 x 1.0` AABB).
    pub fn shulker(position: Vec3) -> Self {
        Self {
            position,
            aabb: Aabb::from_bottom(position, 1.0, 1.0),
            kind: ContactKind::Shulker,
            ..Self::default()
        }
    }
}

/// Applies the base `Entity.push(Entity)` pair law (`0..169`) in `this`/`other` order.
/// Immediately mutates `velocity` per pair (vanilla is non-commutative when `v0 != 0`).
pub fn push_pair(this: &mut ContactEntity, other: &mut ContactEntity, same_vehicle: bool) {
    if same_vehicle || other.no_physics || this.no_physics {
        return;
    }
    let mut x = other.position.x - this.position.x;
    let mut z = other.position.z - this.position.z;
    // Mth.absMax uses Math.max(Math.abs(x), Math.abs(z)).
    let ax = x.abs();
    let az = z.abs();
    let mut distance = if ax.is_nan() || az.is_nan() {
        f64::NAN
    } else {
        ax.max(az)
    };
    // Non-strict threshold: fires when absMax >= (double) 0.01f.
    if distance < 0.01_f32 as f64 {
        return;
    }
    distance = distance.sqrt();
    x /= distance;
    z /= distance;
    let mut scale = 1.0 / distance;
    if scale > 1.0 {
        scale = 1.0;
    }
    x *= scale;
    z *= scale;
    x *= 0.05_f32 as f64;
    z *= 0.05_f32 as f64;
    if x.is_finite() && z.is_finite() {
        if !this.vehicle && this.pushable {
            this.velocity = this.velocity.plus(Vec3::new(-x, 0.0, -z));
        }
        if !other.vehicle && other.pushable {
            other.velocity = other.velocity.plus(Vec3::new(x, 0.0, z));
        }
    }
}

/// `LivingEntity.isPushable @0-26`, `AbstractBoat.isPushable`, `Entity.isPushable`.
#[inline]
pub fn is_pushable(entity: &ContactEntity) -> bool {
    match entity.kind {
        ContactKind::Living | ContactKind::Shulker | ContactKind::HappyGhast { .. } => {
            entity.pushable && entity.alive && !entity.spectator && !entity.on_climbable
        }
        ContactKind::Boat | ContactKind::Minecart | ContactKind::Other => entity.pushable,
    }
}

/// `EntitySelector.pushableBy(source)` (@0-47) + `lambda$pushableBy$0` (@0-141).
#[inline]
pub fn pushable_by(source: &ContactEntity, candidate: &ContactEntity) -> bool {
    if source.team_rule == TeamCollisionRule::Never {
        return false;
    }
    if candidate.spectator || !is_pushable(candidate) {
        return false;
    }
    if candidate.team_rule == TeamCollisionRule::Never {
        return false;
    }
    let allied = source.team_id.is_some() && source.team_id == candidate.team_id;
    if (source.team_rule == TeamCollisionRule::PushOwnTeam
        || candidate.team_rule == TeamCollisionRule::PushOwnTeam)
        && !allied
    {
        return false;
    }
    if (source.team_rule == TeamCollisionRule::PushOtherTeams
        || candidate.team_rule == TeamCollisionRule::PushOtherTeams)
        && allied
    {
        return false;
    }
    true
}

/// `AbstractBoat.push(Entity) @0-56`: receiver is `boat`, pusher is `other`.
/// - boat vs boat: `other.minY < boat.maxY` (strict; stacked boats touching at `maxY` do not push)
/// - non-boat vs boat: `other.minY <= boat.minY` (standing on top does not push; NaN -> false)
#[inline]
pub fn boat_push_gate(boat: &ContactEntity, other: &ContactEntity) -> bool {
    if other.kind == ContactKind::Boat {
        other.aabb.min.y < boat.aabb.max.y
    } else {
        other.aabb.min.y <= boat.aabb.min.y
    }
}

/// `LivingEntity.doPush(Entity) @0-5`: invokes `receiver.push(pusher)` where `receiver`
/// is the neighbor and `pusher` (`this`) is the entity running `pushEntities`.
#[inline]
pub fn do_push(receiver: &mut ContactEntity, pusher: &mut ContactEntity, same_vehicle: bool) {
    match receiver.kind {
        // Shulker.push @0: bare return.
        ContactKind::Shulker => {}
        ContactKind::Boat => {
            if boat_push_gate(receiver, pusher) {
                push_pair(receiver, pusher, same_vehicle);
            }
        }
        // LivingEntity.push @0-12: sleeping bodies absorb no push.
        ContactKind::Living | ContactKind::HappyGhast { .. } => {
            if !receiver.sleeping {
                push_pair(receiver, pusher, same_vehicle);
            }
        }
        ContactKind::Minecart | ContactKind::Other => {
            push_pair(receiver, pusher, same_vehicle);
        }
    }
}

/// `Entity.canBeCollidedWith(Entity)`: solid entity colliders in 26.2 are strictly
/// `AbstractBoat` (true), `Shulker` (`isAlive`), and `HappyGhast` (grown and alive).
/// Minecarts are NOT solid.
#[inline]
pub const fn can_be_collided_with(entity: &ContactEntity) -> bool {
    match entity.kind {
        ContactKind::Boat => true,
        ContactKind::Shulker => entity.alive,
        ContactKind::HappyGhast { baby } => entity.alive && !baby,
        ContactKind::Living | ContactKind::Minecart | ContactKind::Other => false,
    }
}

/// Convenience summary for `ContactKind` solidity (alive, grown).
#[inline]
pub const fn is_solid_collider(kind: ContactKind) -> bool {
    matches!(
        kind,
        ContactKind::Boat | ContactKind::Shulker | ContactKind::HappyGhast { baby: false }
    )
}

/// `LivingEntity.pushEntities @0-191` (`aiStep @817`, after `travel @617`).
///
/// `neighbors` must be collected using the mover's **plain** bounding box (no inflate)
/// via `pushable_by` in vanilla section order, committing each pair in order.
/// Returns `Some(6.0f32)` (cramming damage) BEFORE the push loop when
/// `max_entity_cramming > 0 && neighbors.len() >= max && random_next_int_4 == 0 && non_passengers >= max`.
pub fn push_entities(
    this: &mut ContactEntity,
    neighbors: &mut [(ContactEntity, bool)],
    max_entity_cramming: i32,
    random_next_int_4: i32,
) -> Option<f32> {
    if neighbors.is_empty() {
        return None;
    }
    let cramming = if max_entity_cramming > 0
        && neighbors.len() as i32 >= max_entity_cramming
        && random_next_int_4 == 0
    {
        let standing = neighbors
            .iter()
            .filter(|(e, _)| !e.is_passenger)
            .count() as i32;
        if standing >= max_entity_cramming {
            Some(CRAMMING_DAMAGE)
        } else {
            None
        }
    } else {
        None
    };

    for (other, same_vehicle) in neighbors.iter_mut() {
        do_push(other, this, *same_vehicle);
    }
    cramming
}

/// `SectionPos.posToSectionCoord(double)` = `Mth.floor(v) >> 4`.
#[inline]
pub fn section_coord(coord: f64) -> i32 {
    math::floor(coord) >> 4
}

/// Masked `(z, y)` sort key matching FastUtil `LongSortedSet.subSet(asLong(sx,0,0), asLong(sx,-1,-1)+1)`
/// inside `EntitySectionStorage.forEachAccessibleNonEmptySection` (`SectionPos.asLong @0-38`).
#[inline]
pub const fn mojang_section_zy_key(sy: i32, sz: i32) -> u64 {
    (((sz as i64 & 0x3F_FFFF) as u64) << 20) | ((sy as i64 & 0xF_FFFF) as u64)
}

#[inline]
fn hash_key(key: u64) -> usize {
    let h = key.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (h ^ (h >> 32)) as usize
}

/// High-throughput O(N), zero-steady-state-allocation spatial index for crowd pushes.
///
/// Inserts entities into 2x2-block XZ CSR grid buckets and returns pushable neighbors
/// in **exact Mojang `EntitySectionStorage` order** (`sx` ascending signed, then masked `(sz, sy)`,
/// then entity insertion index), producing bit-identical velocity results to Mojang at O(N).
pub struct SpatialCrowdIndex {
    // Open-addressed hash table from packed `(cx, cz)` cell to dense bucket id (`u32::MAX` = empty).
    table_keys: Vec<u64>,
    table_vals: Vec<u32>,
    table_gen: Vec<u32>,
    generation: u32,
    counts: Vec<u32>,
    offsets: Vec<u32>,
    cursors: Vec<u32>,
    entries: Vec<usize>,
    entity_sections: Vec<(i32, i32, i32, u64)>,
    entity_buckets: Vec<[u32; 4]>,
    entity_bucket_lens: Vec<u8>,
    large_entities: Vec<usize>,
    hit_scratch: Vec<(i32, u64, usize)>,
    neighbor_scratch: Vec<usize>,
}

impl Default for SpatialCrowdIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialCrowdIndex {
    pub fn new() -> Self {
        let cap = 2048usize;
        Self {
            table_keys: vec![0; cap],
            table_vals: vec![u32::MAX; cap],
            table_gen: vec![0; cap],
            generation: 1,
            counts: Vec::new(),
            offsets: Vec::new(),
            cursors: Vec::new(),
            entries: Vec::new(),
            entity_sections: Vec::new(),
            entity_buckets: Vec::new(),
            entity_bucket_lens: Vec::new(),
            large_entities: Vec::new(),
            hit_scratch: Vec::with_capacity(64),
            neighbor_scratch: Vec::with_capacity(64),
        }
    }

    #[inline]
    fn pack_cell(cx: i32, cz: i32) -> u64 {
        ((cx as u32 as u64) << 32) | (cz as u32 as u64)
    }

    fn ensure_table_capacity(&mut self, needed_buckets: usize) {
        if needed_buckets * 2 <= self.table_keys.len() {
            return;
        }
        let mut new_cap = (self.table_keys.len() * 2).max(needed_buckets * 4).next_power_of_two();
        if new_cap < 2048 {
            new_cap = 2048;
        }
        self.table_keys.resize(new_cap, 0);
        self.table_vals.resize(new_cap, u32::MAX);
        self.table_gen.resize(new_cap, 0);
    }

    #[inline]
    fn get_or_insert_bucket(&mut self, cx: i32, cz: i32) -> u32 {
        if self.counts.len() * 2 >= self.table_keys.len() {
            let old_buckets = self.counts.len();
            // Trigger full rehash by bumping generation & copying active buckets if we ever grow mid-rebuild.
            let new_cap = (self.table_keys.len() * 2).next_power_of_two();
            let mut new_keys = vec![0u64; new_cap];
            let mut new_vals = vec![u32::MAX; new_cap];
            let mut new_gen = vec![0u32; new_cap];
            let mask = new_cap - 1;
            let epoch = self.generation;
            for idx in 0..self.table_keys.len() {
                if self.table_gen[idx] == epoch {
                    let k = self.table_keys[idx];
                    let v = self.table_vals[idx];
                    let mut slot = hash_key(k) & mask;
                    while new_gen[slot] == epoch {
                        slot = (slot + 1) & mask;
                    }
                    new_keys[slot] = k;
                    new_vals[slot] = v;
                    new_gen[slot] = epoch;
                }
            }
            self.table_keys = new_keys;
            self.table_vals = new_vals;
            self.table_gen = new_gen;
            let _ = old_buckets;
        }

        let key = Self::pack_cell(cx, cz);
        let mask = self.table_keys.len() - 1;
        let mut slot = hash_key(key) & mask;
        let epoch = self.generation;
        loop {
            if self.table_gen[slot] != epoch {
                let bucket_id = self.counts.len() as u32;
                self.table_gen[slot] = epoch;
                self.table_keys[slot] = key;
                self.table_vals[slot] = bucket_id;
                self.counts.push(0);
                return bucket_id;
            }
            if self.table_keys[slot] == key {
                return self.table_vals[slot];
            }
            slot = (slot + 1) & mask;
        }
    }

    /// Rebuilds the spatial hash and Mojang section cache for `entities`.
    pub fn rebuild(&mut self, entities: &[ContactEntity]) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.table_gen.fill(0);
            self.generation = 1;
        }
        self.counts.clear();
        self.large_entities.clear();

        let n = entities.len();
        self.ensure_table_capacity(n * 2);
        self.entity_sections.clear();
        self.entity_sections.reserve(n);
        self.entity_buckets.clear();
        self.entity_buckets.resize(n, [u32::MAX; 4]);
        self.entity_bucket_lens.clear();
        self.entity_bucket_lens.resize(n, 0);

        for (i, e) in entities.iter().enumerate() {
            let sx = section_coord(e.position.x);
            let sy = section_coord(e.position.y);
            let sz = section_coord(e.position.z);
            self.entity_sections
                .push((sx, sy, sz, mojang_section_zy_key(sy, sz)));

            let b = e.aabb;
            if !(b.min.x < b.max.x && b.min.y < b.max.y && b.min.z < b.max.z)
                || !b.min.x.is_finite()
                || !b.max.x.is_finite()
                || !b.min.y.is_finite()
                || !b.max.y.is_finite()
                || !b.min.z.is_finite()
                || !b.max.z.is_finite()
            {
                continue;
            }

            let min_cx = math::floor(b.min.x) >> 1;
            let max_cx = math::floor(b.max.x) >> 1;
            let min_cz = math::floor(b.min.z) >> 1;
            let max_cz = math::floor(b.max.z) >> 1;
            let span_x = (max_cx as i64 - min_cx as i64) + 1;
            let span_z = (max_cz as i64 - min_cz as i64) + 1;

            if span_x <= 0 || span_z <= 0 || span_x * span_z > 4 {
                self.large_entities.push(i);
                continue;
            }

            let mut k = 0usize;
            for cx in min_cx..=max_cx {
                for cz in min_cz..=max_cz {
                    let bucket = self.get_or_insert_bucket(cx, cz);
                    self.counts[bucket as usize] += 1;
                    self.entity_buckets[i][k] = bucket;
                    k += 1;
                }
            }
            self.entity_bucket_lens[i] = k as u8;
        }

        let num_buckets = self.counts.len();
        self.offsets.clear();
        self.offsets.reserve(num_buckets + 1);
        let mut running = 0u32;
        for &c in &self.counts {
            self.offsets.push(running);
            running += c;
        }
        self.offsets.push(running);

        self.cursors.clear();
        self.cursors.extend_from_slice(&self.offsets[..num_buckets]);
        self.entries.resize(running as usize, 0);

        for i in 0..n {
            let len = self.entity_bucket_lens[i] as usize;
            for k in 0..len {
                let bucket = self.entity_buckets[i][k] as usize;
                let write = self.cursors[bucket];
                self.entries[write as usize] = i;
                self.cursors[bucket] = write + 1;
            }
        }
    }

    #[inline]
    fn lookup_bucket(&self, cx: i32, cz: i32) -> Option<usize> {
        let key = Self::pack_cell(cx, cz);
        let mask = self.table_keys.len() - 1;
        let mut slot = hash_key(key) & mask;
        let epoch = self.generation;
        loop {
            if self.table_gen[slot] != epoch {
                return None;
            }
            if self.table_keys[slot] == key {
                return Some(self.table_vals[slot] as usize);
            }
            slot = (slot + 1) & mask;
        }
    }

    /// Queries pushable neighbors for `entities[mover_idx]` using its plain bounding box
    /// (zero inflate) and returns entity indices in exact Mojang `EntitySectionStorage` order.
    pub fn query_pushable(
        &mut self,
        entities: &[ContactEntity],
        mover_idx: usize,
        out: &mut Vec<usize>,
    ) {
        out.clear();
        let mover = &entities[mover_idx];
        let q = mover.aabb;
        if !(q.min.x < q.max.x && q.min.y < q.max.y && q.min.z < q.max.z)
            || !q.min.x.is_finite()
            || !q.max.x.is_finite()
            || !q.min.y.is_finite()
            || !q.max.y.is_finite()
            || !q.min.z.is_finite()
            || !q.max.z.is_finite()
        {
            return;
        }

        // Exact EntitySectionStorage.forEachAccessibleNonEmptySection asymmetric box widening:
        // (-2.0, -4.0, -2.0, +2.0, +0.0, +2.0) on section lookup, while intersecting plain `q`.
        let min_sx = section_coord(q.min.x - 2.0);
        let min_sy = section_coord(q.min.y - 4.0);
        let min_sz = section_coord(q.min.z - 2.0);
        let max_sx = section_coord(q.max.x + 2.0);
        let max_sy = section_coord(q.max.y);
        let max_sz = section_coord(q.max.z + 2.0);

        self.hit_scratch.clear();
        let blen = self.entity_bucket_lens[mover_idx] as usize;
        if blen > 0 {
            for k in 0..blen {
                let bucket = self.entity_buckets[mover_idx][k] as usize;
                let start = self.offsets[bucket] as usize;
                let end = self.offsets[bucket + 1] as usize;
                for &j in &self.entries[start..end] {
                    if j == mover_idx {
                        continue;
                    }
                    let (sx, sy, sz, zy) = self.entity_sections[j];
                    if sx < min_sx
                        || sx > max_sx
                        || sy < min_sy
                        || sy > max_sy
                        || sz < min_sz
                        || sz > max_sz
                    {
                        continue;
                    }
                    let cand = &entities[j];
                    if q.intersects(cand.aabb) && pushable_by(mover, cand) {
                        self.hit_scratch.push((sx, zy, j));
                    }
                }
            }
        } else {
            // Mover exceeded 2x2 cell span: probe cells across mover's XZ span.
            let min_cx = math::floor(q.min.x) >> 1;
            let max_cx = math::floor(q.max.x) >> 1;
            let min_cz = math::floor(q.min.z) >> 1;
            let max_cz = math::floor(q.max.z) >> 1;
            for cx in min_cx..=max_cx {
                for cz in min_cz..=max_cz {
                    if let Some(bucket) = self.lookup_bucket(cx, cz) {
                        let start = self.offsets[bucket] as usize;
                        let end = self.offsets[bucket + 1] as usize;
                        for &j in &self.entries[start..end] {
                            if j == mover_idx {
                                continue;
                            }
                            let (sx, sy, sz, zy) = self.entity_sections[j];
                            if sx < min_sx
                                || sx > max_sx
                                || sy < min_sy
                                || sy > max_sy
                                || sz < min_sz
                                || sz > max_sz
                            {
                                continue;
                            }
                            let cand = &entities[j];
                            if q.intersects(cand.aabb) && pushable_by(mover, cand) {
                                self.hit_scratch.push((sx, zy, j));
                            }
                        }
                    }
                }
            }
        }

        // Also test any oversized entities not in CSR buckets.
        for &j in &self.large_entities {
            if j == mover_idx {
                continue;
            }
            let (sx, sy, sz, zy) = self.entity_sections[j];
            if sx < min_sx
                || sx > max_sx
                || sy < min_sy
                || sy > max_sy
                || sz < min_sz
                || sz > max_sz
            {
                continue;
            }
            let cand = &entities[j];
            if q.intersects(cand.aabb) && pushable_by(mover, cand) {
                self.hit_scratch.push((sx, zy, j));
            }
        }

        if self.hit_scratch.is_empty() {
            return;
        }
        if self.hit_scratch.len() > 1 {
            self.hit_scratch.sort_unstable();
            let mut prev = usize::MAX;
            for &(.., j) in &self.hit_scratch {
                if j != prev {
                    out.push(j);
                    prev = j;
                }
            }
        } else {
            out.push(self.hit_scratch[0].2);
        }
    }

    /// Runs `LivingEntity.pushEntities` across all living/mob entities in `entities[0..n]`
    /// with O(N) spatial lookup and Mojang-identical per-pair commit order.
    pub fn step_crowd_push(
        &mut self,
        entities: &mut [ContactEntity],
        max_entity_cramming: i32,
        same_vehicle: impl Fn(usize, usize) -> bool,
        mut random_next_int_4: impl FnMut(usize) -> i32,
        mut on_cram: impl FnMut(usize, f32),
    ) {
        self.rebuild(entities);
        let n = entities.len();
        let mut neighbors = std::mem::take(&mut self.neighbor_scratch);

        for i in 0..n {
            if !entities[i].alive
                || entities[i].spectator
                || !matches!(
                    entities[i].kind,
                    ContactKind::Living | ContactKind::Shulker | ContactKind::HappyGhast { .. }
                )
            {
                continue;
            }

            self.query_pushable(entities, i, &mut neighbors);
            if neighbors.is_empty() {
                continue;
            }

            if max_entity_cramming > 0
                && neighbors.len() as i32 >= max_entity_cramming
                && random_next_int_4(i) == 0
            {
                let standing = neighbors
                    .iter()
                    .filter(|&&j| !entities[j].is_passenger)
                    .count() as i32;
                if standing >= max_entity_cramming {
                    on_cram(i, CRAMMING_DAMAGE);
                }
            }

            for &j in &neighbors {
                let sv = same_vehicle(i, j);
                if j < i {
                    let (left, right) = entities.split_at_mut(i);
                    do_push(&mut left[j], &mut right[0], sv);
                } else {
                    let (left, right) = entities.split_at_mut(j);
                    do_push(&mut right[0], &mut left[i], sv);
                }
            }
        }

        neighbors.clear();
        self.neighbor_scratch = neighbors;
    }
}

#[cfg(test)]
mod crowd_witness {
    use super::*;

    const IMPULSE: f64 = 0.05_f32 as f64; // bits 3fa99999a0000000

    fn boat() -> ContactEntity {
        ContactEntity::boat(Vec3::new(0.0, 64.0, 0.0), Vec3::default())
    }

    fn player(min_y: f64, x: f64) -> ContactEntity {
        let mut p = ContactEntity::player(Vec3::new(x, min_y, 0.0), Vec3::default());
        if !min_y.is_finite() {
            p.aabb.min.y = min_y;
            p.aabb.max.y = min_y;
        }
        p
    }

    // Harness A1/A4: beside or below boat pushes both sides.
    #[test]
    fn boat_push_open_below_and_beside() {
        for min_y in [64.0, 63.0] {
            let (mut b, mut p) = (boat(), player(min_y, 1.0));
            do_push(&mut b, &mut p, false);
            assert_eq!(p.velocity.x.to_bits(), IMPULSE.to_bits());
            assert_eq!(b.velocity.x.to_bits(), (-IMPULSE).to_bits());
        }
    }

    // Harness A2/A3/A5: standing overlapping, exact touch on top, or NaN minY -> gated.
    #[test]
    fn boat_push_gated_above() {
        for min_y in [64.4, 64.5625, f64::NAN] {
            let (mut b, mut p) = (boat(), player(min_y, 1.0));
            do_push(&mut b, &mut p, false);
            assert_eq!(p.velocity.x, 0.0, "min_y {min_y}");
            assert_eq!(b.velocity.x, 0.0, "min_y {min_y}");
        }
    }

    // Harness B1/B2/B3: boat vs boat uses strict other.minY < this.maxY.
    #[test]
    fn boat_boat_strict_max_y_gate() {
        let (mut b, mut beside) = (boat(), boat());
        beside.position.x = 1.0;
        do_push(&mut b, &mut beside, false);
        assert_eq!(b.velocity.x.to_bits(), (-IMPULSE).to_bits());
        assert_eq!(beside.velocity.x.to_bits(), IMPULSE.to_bits());

        let (mut b, mut stacked) = (boat(), boat());
        stacked.position.x = 1.0;
        stacked.aabb.min.y = b.aabb.max.y;
        stacked.aabb.max.y = stacked.aabb.min.y + 0.5625;
        do_push(&mut b, &mut stacked, false);
        assert_eq!(b.velocity.x, 0.0);
        assert_eq!(stacked.velocity.x, 0.0);
    }

    // Harness C1: player receiver has no Y gate.
    #[test]
    fn player_receiver_has_no_y_gate() {
        let (mut b, mut p) = (boat(), player(64.4, 1.0));
        do_push(&mut p, &mut b, false);
        assert_eq!(p.velocity.x.to_bits(), IMPULSE.to_bits());
        assert_eq!(b.velocity.x.to_bits(), (-IMPULSE).to_bits());
    }

    // Exact 0.01f32 threshold cutoff witness (`port-push-threshold`).
    #[test]
    fn exact_push_threshold_0_01_f32() {
        let d = 0.01_f32 as f64;
        let (mut a, mut b) = (player(0.0, 0.0), player(0.0, d));
        push_pair(&mut a, &mut b, false);
        assert_eq!(b.velocity.x, 0.005000000018626451);
        assert_eq!(a.velocity.x, -0.005000000018626451);

        let (mut a, mut b) = (player(0.0, 0.0), player(0.0, d - 1e-9));
        push_pair(&mut a, &mut b, false);
        assert_eq!(b.velocity.x, 0.0);
    }

    #[test]
    fn cramming_needs_size_standing_count_and_roll() {
        let mut this = player(64.0, 0.0);
        let run = |this: &mut ContactEntity, n: usize, passengers: usize, roll: i32| {
            let mut v: Vec<(ContactEntity, bool)> = (0..n)
                .map(|i| {
                    let mut p = player(64.0, i as f64 + 1.0);
                    p.is_passenger = i < passengers;
                    (p, false)
                })
                .collect();
            push_entities(this, &mut v, CRAMMING_DEFAULT_MAX, roll)
        };
        assert_eq!(run(&mut this, 23, 0, 0), None);
        assert_eq!(run(&mut this, 24, 0, 0), Some(6.0));
        assert_eq!(run(&mut this, 24, 1, 0), None);
        assert_eq!(run(&mut this, 30, 0, 1), None);
    }

    #[test]
    fn solid_colliders_and_teams_match_26_2() {
        assert!(is_solid_collider(ContactKind::Boat));
        assert!(is_solid_collider(ContactKind::Shulker));
        assert!(is_solid_collider(ContactKind::HappyGhast { baby: false }));
        assert!(!is_solid_collider(ContactKind::HappyGhast { baby: true }));
        assert!(!is_solid_collider(ContactKind::Minecart));
        assert!(!is_solid_collider(ContactKind::Living));

        let mut s = ContactEntity::shulker(Vec3::new(0.0, 64.0, 0.0));
        let mut p = player(64.0, 1.0);
        do_push(&mut s, &mut p, false);
        assert_eq!(s.velocity.x, 0.0);
        assert_eq!(p.velocity.x, 0.0);
        assert!(can_be_collided_with(&s));
        s.alive = false;
        assert!(!can_be_collided_with(&s));

        // Team collision rules
        let mut a = player(64.0, 0.0);
        let mut b = player(64.0, 0.5);
        a.team_id = Some(1);
        b.team_id = Some(1);
        a.team_rule = TeamCollisionRule::PushOtherTeams;
        assert!(!pushable_by(&a, &b));
        a.team_rule = TeamCollisionRule::PushOwnTeam;
        assert!(pushable_by(&a, &b));
        b.team_id = Some(2);
        assert!(!pushable_by(&a, &b));
        b.on_climbable = true;
        b.team_id = Some(1);
        assert!(!pushable_by(&a, &b));
    }

    /// Proves `SpatialCrowdIndex::step_crowd_push` is bit-identical to a literal
    /// Mojang `EntitySectionStorage` O(N^2) scan across positive/negative X/Y/Z sections,
    /// dense cramming clusters, boats, shulkers, and nonzero initial velocities.
    #[test]
    fn spatial_crowd_index_is_bit_identical_to_mojang_section_order() {
        let mut rng = 0x9E37_79B9_7F4A_7C15u64;
        let mut next_f = |lo: f64, hi: f64| -> f64 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let u = ((rng >> 11) as f64) / ((1u64 << 53) as f64);
            lo + u * (hi - lo)
        };

        let mut base = Vec::with_capacity(400);
        for i in 0..400 {
            let (px, py, pz) = if i < 60 {
                // Dense cramming cluster straddling x=0, z=0 section borders
                (next_f(-0.4, 0.4), next_f(-0.5, 0.5), next_f(-0.4, 0.4))
            } else {
                (next_f(-36.0, 36.0), next_f(-20.0, 20.0), next_f(-36.0, 36.0))
            };
            let vel = Vec3::new(next_f(-0.2, 0.2), 0.0, next_f(-0.2, 0.2));
            let pos = Vec3::new(px, py, pz);
            let mut e = match i % 10 {
                0 => ContactEntity::boat(pos, vel),
                1 => ContactEntity::shulker(pos),
                _ => ContactEntity::player(pos, vel),
            };
            if i % 7 == 0 {
                e.is_passenger = true;
            }
            if i % 19 == 0 {
                e.on_climbable = true;
            }
            base.push(e);
        }

        // Literal Mojang reference O(N^2) section scan
        let mut expected = base.clone();
        let mut expected_crams = Vec::new();
        for i in 0..expected.len() {
            if !expected[i].alive
                || expected[i].spectator
                || !matches!(expected[i].kind, ContactKind::Living | ContactKind::Shulker | ContactKind::HappyGhast { .. })
            {
                continue;
            }
            let q = expected[i].aabb;
            let min_sx = section_coord(q.min.x - 2.0);
            let min_sy = section_coord(q.min.y - 4.0);
            let min_sz = section_coord(q.min.z - 2.0);
            let max_sx = section_coord(q.max.x + 2.0);
            let max_sy = section_coord(q.max.y);
            let max_sz = section_coord(q.max.z + 2.0);

            let mut hits = Vec::new();
            for j in 0..expected.len() {
                if i == j {
                    continue;
                }
                let p = expected[j].position;
                let (sx, sy, sz) = (section_coord(p.x), section_coord(p.y), section_coord(p.z));
                if sx < min_sx || sx > max_sx || sy < min_sy || sy > max_sy || sz < min_sz || sz > max_sz {
                    continue;
                }
                if q.intersects(expected[j].aabb) && pushable_by(&expected[i], &expected[j]) {
                    hits.push((sx, mojang_section_zy_key(sy, sz), j));
                }
            }
            if hits.is_empty() {
                continue;
            }
            hits.sort_unstable();
            if hits.len() >= 24 && (i % 4 == 0) {
                let standing = hits.iter().filter(|&&(.., j)| !expected[j].is_passenger).count();
                if standing >= 24 {
                    expected_crams.push((i, 6.0f32));
                }
            }
            for &(.., j) in &hits {
                if j < i {
                    let (l, r) = expected.split_at_mut(i);
                    do_push(&mut l[j], &mut r[0], false);
                } else {
                    let (l, r) = expected.split_at_mut(j);
                    do_push(&mut r[0], &mut l[i], false);
                }
            }
        }

        let mut actual = base;
        let mut actual_crams = Vec::new();
        let mut index = SpatialCrowdIndex::new();
        index.step_crowd_push(
            &mut actual,
            24,
            |_, _| false,
            |i| (i % 4) as i32,
            |i, dmg| actual_crams.push((i, dmg)),
        );

        assert_eq!(actual_crams, expected_crams);
        assert!(!actual_crams.is_empty(), "test should exercise cramming");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert_eq!(a.velocity.x.to_bits(), e.velocity.x.to_bits(), "vx mismatch at {i}");
            assert_eq!(a.velocity.y.to_bits(), e.velocity.y.to_bits(), "vy mismatch at {i}");
            assert_eq!(a.velocity.z.to_bits(), e.velocity.z.to_bits(), "vz mismatch at {i}");
        }
    }
}
