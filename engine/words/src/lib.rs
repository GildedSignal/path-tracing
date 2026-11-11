//! Movement words (docs/vocabulary.md, sections 1 to 7): segments cut by the event
//! rule, the class that proposes, gaits, and the proposal table. The class only
//! proposes; every segment handed out was replayed through the law.
pub mod class;
pub mod gait;
pub mod program;
pub mod segment;
pub mod table;
pub use class::{Grid, body_block, class};
pub use gait::{Discovery, Gait, discover_gaits};
pub use program::{Arguments, Predicate, Program, Row, tiny_programs};
pub use segment::{Exit, ExitKind, Guards, Segment, SegmentError, Uses, Width, run_segment, width};
pub use table::{Certified, Proposal, ProposalTable};
