//! The class: the facts the law decides on (vocabulary.md, section 3). It names a piece so
//! that it is recognised and nothing else: no bit of it reaches the law, and the search
//! never merges states by it.
//!
//! One rule per field the segment read before writing it:
//! - a field whose values come from the block table or the toolkit enters by its bits;
//! - a field that accumulates enters only as the answers of the comparisons the law makes
//!   on it, each named below by its line in `engine/law/src`. None is ours.
//!
//! The answers are taken on the start state. The law makes some of them later in the tick,
//! on the value it has by then; the class only proposes, so that costs a replay at most.
use law::{BlockPos, PlayerState, pattern::Start, state_reads::StateReads};

/// What a table names states by. The table takes it as a value so that a lab can measure
/// another class against this one.
pub type ClassOf = fn(&PlayerState, StateReads) -> Box<[u64]>;

pub fn body_block(state: &PlayerState) -> BlockPos {
    [state.position.x, state.position.y, state.position.z].map(law::math::floor)
}

/// Compatibility wrapper: the canonical state query lives beside the cell codec.
pub fn class(state: &PlayerState, reads: StateReads) -> Box<[u64]> {
    law::codec::state_answers(state, reads)
}
/// A class as a pattern-table start, anchored at the body's block: the world part of a
/// proposal is then the pattern key of the cells its replay read.
pub fn start(state: &PlayerState, words: Box<[u64]>) -> Start {
    Start { question: law::codec::StateQuestion::Comparisons, origin: body_block(state), words, context: law::CollisionContext::of(state) }
}
