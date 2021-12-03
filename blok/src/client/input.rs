//! Dealing with player input.

use crate::state::Tick;

use std::collections::VecDeque;

/// High-level description of the input.
///
/// This describes the input in terms of gameplay actions
/// rather than signals from human interface devices.
/// Generating abstract input requires knowledge about
/// the control mapping that the player configured and
/// the user interface elements that are being displayed.
#[allow(missing_docs)]
pub struct AbstractInput
{
    pub move_forward: bool,
    pub move_backward: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
}

/// Buffer of recent inputs for server reconciliation.
pub struct ReconciliationBuffer<T>
{
    // INVARIANT: Ticks are monotonically increasing.
    queue: VecDeque<(Tick, T)>,
}

impl<T> ReconciliationBuffer<T>
{
    /// Create a new buffer with no entries.
    pub fn new() -> Self
    {
        Self{queue: VecDeque::new()}
    }

    /// Insert an entry at end of the buffer.
    ///
    /// # Panics
    ///
    /// Panics if `tick` does not monotonically increase.
    pub fn push(&mut self, tick: Tick, input: T)
    {
        if let Some(&(latest, _)) = self.queue.back() {
            assert!(tick > latest);
        }
        self.queue.push_back((tick, input));
    }

    /// Remove all buffer entries with a tick older than `tick`.
    pub fn drain(&mut self, tick: Tick)
    {
        let pp = self.queue.partition_point(|&(other, _)| other < tick);
        self.queue.drain(0 .. pp);
    }

    /// Iterator over the entries in the buffer.
    ///
    /// The iterator yields the entries in chronological order.
    pub fn iter(&self) -> impl Iterator<Item=(Tick, &T)>
    {
        self.queue.iter().map(|&(tick, ref input)| (tick, input))
    }

    /// The oldest tick for which entries exist in the buffer.
    ///
    /// If the buffer is empty, this method returns [`None`].
    pub fn oldest(&self) -> Option<Tick>
    {
        self.queue.front()
            .map(|(tick, _)| tick)
            .copied()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn drain_until_iter()
    {
        let mut buf = ReconciliationBuffer::new();
        buf.push(Tick(0), 'W');
        buf.push(Tick(1), 'A');
        buf.push(Tick(2), 'S');
        buf.push(Tick(3), 'D');
        buf.drain(Tick(2));
        assert_eq!(
            buf.iter().collect::<Vec<_>>(),
            &[(Tick(2), &'S'), (Tick(3), &'D')],
        );
    }
}
