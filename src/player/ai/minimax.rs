use crate::core::{Board, Move, PlayerId};
use crate::logic::{apply_move, evaluate, legal_moves};
use crate::player::PlayerController;
use std::f64;

pub struct MinimaxAI {
    pub player_id: PlayerId,
    pub name: String,
    pub depth: usize,
}

impl MinimaxAI {
    pub fn new(player_id: PlayerId, name: &str) -> Self {
        Self {
            player_id,
            name: name.to_string(),
            depth: 2,
        }
    }

    fn minimax(
        &self,
        board: &Board,
        depth: usize,
        alpha: i32,
        beta: i32,
        is_maximizing: bool,
        current_player: PlayerId,
    ) -> i32 {
        if depth == 0 {
            return evaluate(board, self.player_id);
        }

        let moves = legal_moves(board, current_player);
        if moves.is_empty() {
            // 王がいない（取られた）場合は極端な値を返す
            return evaluate(board, self.player_id);
        }

        let next_player = if current_player == PlayerId::Player1 {
            PlayerId::Player2
        } else {
            PlayerId::Player1
        };

        if is_maximizing {
            let mut max_eval = i32::MIN;
            let mut alpha = alpha;
            for mv in moves {
                let next_board = apply_move(board, &mv, current_player);
                let eval = self.minimax(&next_board, depth - 1, alpha, beta, false, next_player);
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            }
            max_eval
        } else {
            let mut min_eval = i32::MAX;
            let mut beta = beta;
            for mv in moves {
                let next_board = apply_move(board, &mv, current_player);
                let eval = self.minimax(&next_board, depth - 1, alpha, beta, true, next_player);
                min_eval = min_eval.min(eval);
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
            min_eval
        }
    }
}

impl PlayerController for MinimaxAI {
    fn choose_move(&self, board: &Board, moves: &[Move]) -> Option<Move> {
        if moves.is_empty() {
            return None;
        }

        let mut best_move = None;
        let mut best_value = i32::MIN;

        let opponent = if self.player_id == PlayerId::Player1 {
            PlayerId::Player2
        } else {
            PlayerId::Player1
        };

        for mv in moves {
            let next_board = apply_move(board, mv, self.player_id);
            // 既に depth=1 の探索（自分の一手）は終わっているので、次は相手の番(is_maximizing=false)
            let board_value = self.minimax(
                &next_board,
                self.depth - 1,
                i32::MIN,
                i32::MAX,
                false,
                opponent,
            );

            if board_value > best_value {
                best_value = board_value;
                best_move = Some(mv.clone());
            }
        }

        best_move.or_else(|| Some(moves[0].clone()))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_local(&self) -> bool {
        true
    }
}
