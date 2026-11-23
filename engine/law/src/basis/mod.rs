//! The basis: a game of boxes that never rotate. A body is position, velocity, a box
//! and a row; the law is these generators run over rows, plus thin scars named in the
//! rows where Mojang breaks its own pattern.
//!
//! - `body`: per tick, velocity += input + gravity, move with `sweep`, velocity × drag.
//! - `sweep`: an AABB through a box set, axis by axis; step-up is the same sweep lifted.
//! - `ray`: DDA through cells, each clipped against its boxes.
//! - `clock`: a counter, a threshold that fires, a reset.
pub mod body;
pub mod clock;
pub mod ray;
pub mod sweep;
