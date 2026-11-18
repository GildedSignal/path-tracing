//! Optional accounting for hash-table operations on the command path. The
//! instrumented build keeps the standard randomly seeded hash function.
#[cfg(not(feature="solve-counters"))]
pub use std::collections::HashMap;
#[cfg(feature="solve-counters")]
pub use metered::{HashMap, reset, probes};

#[cfg(feature="solve-counters")]
mod metered {
    use std::{cell::Cell, hash::{BuildHasher, RandomState}};
    thread_local! {static PROBES: Cell<u64> = const {Cell::new(0)};}
    pub fn reset() {PROBES.set(0);}
    pub fn probes() -> u64 {PROBES.get()}
    #[derive(Clone, Debug, Default)]
    pub struct Counted(RandomState);
    impl BuildHasher for Counted {
        type Hasher = <RandomState as BuildHasher>::Hasher;
        fn build_hasher(&self) -> Self::Hasher {
            PROBES.set(PROBES.get()+1);
            self.0.build_hasher()
        }
    }
    pub type HashMap<K,V> = std::collections::HashMap<K,V,Counted>;
}
