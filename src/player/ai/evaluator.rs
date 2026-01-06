//! Evaluator trait for board evaluation
//!
//! Defines a common interface for different evaluation strategies.

use crate::core::Board;

/// Trait for evaluating board positions
pub trait Evaluator: Send + Sync {
    /// Evaluate the board from Player1's perspective
    ///
    /// Returns:
    ///   - Positive score: Player1 advantage
    ///   - Negative score: Player2 advantage
    ///   - Zero: Equal position
    fn evaluate(&mut self, board: &Board) -> i32;

    /// Get evaluator name for debugging
    fn name(&self) -> &str;
}
