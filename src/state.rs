//! Data structures for game state.

/// Monotonically increasing number identifying a tick.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Tick(pub u64);
