//! Derived leaf presence only: no terrain copy and no section-wide epoch.
use super::{ClassId, LegsRole};

/// Mojang's 16^3 section addresses, packed in native 64-bit words.
pub const WORDS: usize = 16 * 16 * 16 / 64;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plane { AllZero, AllOne, Words(Box<[u64; WORDS]>) }
impl Plane {
    pub fn from_words(words: [u64; WORDS]) -> Self {
        if words.iter().all(|&w| w == 0) { Self::AllZero }
        else if words.iter().all(|&w| w == u64::MAX) { Self::AllOne }
        else { Self::Words(Box::new(words)) }
    }
    pub fn word(&self, index: usize) -> Option<u64> {
        if index >= WORDS { return None; }
        Some(match self { Self::AllZero => 0, Self::AllOne => u64::MAX, Self::Words(words) => words[index] })
    }
    pub fn set(&mut self, cell: usize, value: bool) -> bool {
        if cell >= WORDS * 64 { return false; }
        let mut words = std::array::from_fn(|i| self.word(i).unwrap());
        let bit = 1u64 << (cell % 64);
        if value { words[cell / 64] |= bit; } else { words[cell / 64] &= !bit; }
        *self = Self::from_words(words);
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Missing { Coverage, Loaded, Address }

/// Role bits mean unequal to the prepared word's requested class. The caller
/// selects the kit projection before building/updating these derived planes.
pub struct SectionRoles {
    pub complete: Plane,
    pub loaded: Plane,
    pub roles: [Plane; 4],
}
impl SectionRoles {
    /// Build derived role planes from the source's fine-ID projection. Source
    /// storage stays outside; this retains only presence/coverage bits.
    pub fn from_classes(expected: [ClassId; 4], mut read: impl FnMut(usize) -> Option<([ClassId; 4], bool)>) -> Self {
        let mut complete = [0; WORDS];
        let mut loaded = [0; WORDS];
        let mut roles = [[0; WORDS]; 4];
        for cell in 0..WORDS * 64 {
            let Some((classes, is_loaded)) = read(cell) else { continue };
            let (word, bit) = (cell / 64, 1u64 << (cell % 64));
            complete[word] |= bit;
            if is_loaded { loaded[word] |= bit; }
            for role in 0..4 { if classes[role] != expected[role] { roles[role][word] |= bit; } }
        }
        Self { complete: Plane::from_words(complete), loaded: Plane::from_words(loaded), roles: roles.map(Plane::from_words) }
    }
    /// Query requested words first; locate exception cells only in returned words.
    pub fn query(&self, role: LegsRole, requested: &[(usize, u64)]) -> Result<Vec<(usize, u64)>, Missing> {
        let mut exceptions = Vec::new();
        for pair in requested.chunks(2) {
            let mut masks = [0;2]; let mut complete = [0;2]; let mut loaded = [0;2]; let mut bits = [0;2];
            for (lane, &(index, mask)) in pair.iter().enumerate() {
                masks[lane] = mask;
                complete[lane] = self.complete.word(index).ok_or(Missing::Address)?;
                loaded[lane] = self.loaded.word(index).ok_or(Missing::Address)?;
                bits[lane] = self.roles[role as usize].word(index).ok_or(Missing::Address)?;
            }
            let (unknown, unloaded, found) = masked_pair(masks, complete, loaded, bits);
            if unknown.iter().any(|&v| v != 0) { return Err(Missing::Coverage); }
            if unloaded.iter().any(|&v| v != 0) { return Err(Missing::Loaded); }
            for (lane, &(index, _)) in pair.iter().enumerate() { if found[lane] != 0 { exceptions.push((index, found[lane])); } }
        }
        Ok(exceptions)
    }
}

#[cfg(target_arch = "x86_64")]
fn masked_pair(mask: [u64;2], complete: [u64;2], loaded: [u64;2], bits: [u64;2]) -> ([u64;2], [u64;2], [u64;2]) {
    use std::arch::x86_64::*;
    // SSE2 is mandatory on x86_64. Unaligned loads/stores accept these stack arrays.
    unsafe {
        let m = _mm_loadu_si128(mask.as_ptr().cast());
        let c = _mm_loadu_si128(complete.as_ptr().cast());
        let l = _mm_loadu_si128(loaded.as_ptr().cast());
        let b = _mm_loadu_si128(bits.as_ptr().cast());
        let (mut unknown, mut unloaded, mut found) = ([0;2], [0;2], [0;2]);
        _mm_storeu_si128(unknown.as_mut_ptr().cast(), _mm_andnot_si128(c, m));
        _mm_storeu_si128(unloaded.as_mut_ptr().cast(), _mm_andnot_si128(l, m));
        _mm_storeu_si128(found.as_mut_ptr().cast(), _mm_and_si128(b, m));
        (unknown, unloaded, found)
    }
}
#[cfg(not(target_arch = "x86_64"))]
fn masked_pair(mask: [u64;2], complete: [u64;2], loaded: [u64;2], bits: [u64;2]) -> ([u64;2], [u64;2], [u64;2]) {
    (std::array::from_fn(|i| mask[i] & !complete[i]), std::array::from_fn(|i| mask[i] & !loaded[i]), std::array::from_fn(|i| mask[i] & bits[i]))
}
