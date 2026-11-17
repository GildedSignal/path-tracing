//! The planner (job 3.2b; docs/learning.md, vocabulary.md section 10): standard pathfinding
//! over words. Nodes are exact states, edges are words certified by the law, the cost is
//! ticks. A lossy layer (the bounds, the backward sets) orders and never decides.
pub mod arith;
pub mod goal;
pub mod flight;
pub mod polish;
pub mod reach;
pub mod search;
pub mod time;
pub mod walk;
pub use goal::Goal;
pub use polish::{Slide, slide};
pub use reach::{Field, Reach};
pub use search::{Options, Order, Outcome, Planner, Problem, Stats, Stopped};
pub use time::{Free, Scheduled, Still, Timeline};
pub use walk::{Certificate, Halt, Lane, Step, advance, certify, settle};
pub mod deviation;
pub mod events;
pub mod joints;
pub mod faces;
pub mod lattice;
pub mod source;
pub mod store;
pub mod solve;
pub use solve::{solve,Bracket,Kit,Plan,PlanOutcome,Stage,WordStep};
