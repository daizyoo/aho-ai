use crate::core::{Board, Move, PlayerId};
use crate::logic::{apply_move, evaluate, legal_moves};
use crate::player::PlayerController;

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
            depth: 3,
        }
    }

    /// 指し手の有用性で見積もりを行い、ソートする（αβ枝刈りの効率化）
    fn order_moves(&self, board: &Board, moves: &mut [Move]) {
        moves.sort_by_key(|mv| {
            let score = match mv {
                Move::Normal { to, promote, .. } => {
                    let mut s = 0;
                    // 駒を取る手を優先
                    if board.get_piece(*to).is_some() {
                        s += 100;
                    }
                    // 成る手を優先
                    if *promote {
                        s += 50;
                    }
                    -s // 降順ソートのためマイナス
                }
                Move::Drop { .. } => 0,
            };
            score
        });
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

        let mut moves = legal_moves(board, current_player);
        if moves.is_empty() {
            return evaluate(board, self.player_id);
        }

        // 枝刈り効率化のために指し手をソート
        self.order_moves(board, &mut moves);

        let next_player = current_player.opponent();

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

        let mut sorted_moves = moves.to_vec();
        self.order_moves(board, &mut sorted_moves);

        let mut best_move = None;
        let mut best_value = i32::MIN;

        let opponent = self.player_id.opponent();

        for mv in sorted_moves {
            let next_board = apply_move(board, &mv, self.player_id);
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
