//! The planner (job 3.2b; map/detail/learning.md, vocabulary.md section 10): standard pathfinding
//! over words. Nodes are exact states, edges are words certified by the law, the cost is
//! ticks. A lossy layer (the bounds, the backward sets) orders and never decides.
pub mod goal;
pub mod flight;
pub mod time;
pub mod walk;
pub use goal::Goal;
pub use time::{Still, Timeline};
pub use walk::{Certificate, Lane, Step, certify, settle};
pub mod deviation;
pub mod events;
pub mod joints;
pub mod faces;
pub mod lattice;
pub mod source;
pub mod store;
pub mod solve;
pub use solve::{solve,Bracket,Kit,Plan,PlanOutcome,Stage,WordStep};
pub mod families;

pub mod inline;
pub mod state_view;

#[cfg(feature="solve-counters")]
pub mod solve_meter;
