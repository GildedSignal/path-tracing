use crate::codec::MAXC;
use crate::program::Prog;
use crate::shelf::Ship;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeltColumn;

impl BeltColumn {
    #[inline]
    pub fn at(self, _i: u64) -> u64 {
        unreachable!("belt unused in probe")
    }

    #[inline]
    pub fn shifted(self, _s: u64) -> Self {
        self
    }

    #[inline]
    pub fn bounds(self) -> (i128, i128) {
        (0, 0)
    }
}

pub struct Belt;

impl Belt {
    pub fn table_bytes() -> usize {
        0
    }
}

#[inline]
pub fn base(_prog: &[Prog; MAXC], _cols: usize, _i: u64) -> Option<Ship> {
    None
}
