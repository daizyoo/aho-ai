use crate::core::{Board, Move};

/// プレイヤー操作のtrait
pub trait PlayerController {
    fn choose_move(&self, board: &Board, legal_moves: &[Move]) -> Option<Move>;
    fn name(&self) -> &str;
    fn is_local(&self) -> bool;
}
