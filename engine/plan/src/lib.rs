//! The planner (`map/architecture.md`): one loop from the million-block line
//! through arbitrary terrain defects, using coordinate-free Words, raw exploration,
//! and single-winner certification.
pub mod goal;
pub mod flight;
pub mod time;
pub mod walk;
pub use goal::{Face, Goal};
pub use time::{Still, Timeline};
pub use walk::{Certificate, Lane, Step, certify, settle};
pub mod deviation;
pub mod events;
pub mod lattice;
pub mod source;
pub mod store;
pub mod word_store;
mod cut;
mod defect;
mod far_field;
mod certify;
pub mod solve;
pub use solve::{solve, solve_step, Bracket, Kit, Plan, PlanOutcome, PrefixPlan, Stage, WordStep};

pub mod inline;
pub mod kit;
pub use kit::{KitCounts, KitOp, KitSpec, KitView};
pub mod state_view;

#[cfg(feature = "solve-counters")]
pub mod solve_meter;
