//! Read-only Minecraft 26.2 region files and prevalidated law snapshots.
pub mod inflate;
mod nbt;
mod region;
mod snapshot;
pub use region::{BlockState, WorldReader, DEFAULT_CACHE_BYTES};
pub use snapshot::{WorldSnapshot, model, model_table};
use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub enum LoadError {
    Io { path: PathBuf, source: std::io::Error },
    Invalid(String),
    Unsupported(String),
    MissingChunk { x: i32, z: i32 },
    VersionMismatch { chunk: [i32; 2], expected: i32, found: i32 },
    CacheTooSmall { needed: usize, limit: usize },
    Unmodeled { position: [i32; 3], state: BlockState, reason: String },
    At { position: [i32; 3], source: Box<LoadError> },
}
impl LoadError {
    pub(crate) fn at(self, position: [i32; 3]) -> Self { Self::At { position, source: Box::new(self) } }
}
impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Invalid(s) => write!(f, "invalid world data: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported: {s}"),
            Self::MissingChunk { x, z } => write!(f, "missing generated chunk ({x}, {z})"),
            Self::VersionMismatch { chunk, expected, found } => write!(f, "chunk {chunk:?} DataVersion {found} differs from level.dat DataVersion {expected}"),
            Self::CacheTooSmall { needed, limit } => write!(f, "chunk cache needs {needed} bytes; fixed limit is {limit}"),
            Self::Unmodeled { position, state, reason } => write!(f, "unmodeled block state at {position:?}: {state} ({reason})"),
            Self::At { position, source } => write!(f, "at {position:?}: {source}"),
        }
    }
}
impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self { Self::Io { source, .. } => Some(source), Self::At { source, .. } => Some(source), _ => None }
    }
}
mod items;
pub use items::{UseEffects, use_effects};
