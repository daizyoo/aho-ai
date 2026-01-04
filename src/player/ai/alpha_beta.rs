use super::eval::Evaluator;
use super::tt::{Bound, TranspositionTable};
use super::zobrist::ZobristHasher;
use crate::core::{Board, Move, PlayerId};
use crate::logic::{apply_move, is_in_check, legal_moves};
use crate::player::PlayerController;

use std::cell::RefCell;
use std::time::{Duration, Instant};

pub struct AlphaBetaAI {
    player_id: PlayerId,
    name: String,
    tt: RefCell<TranspositionTable>,
    nodes_evaluated: RefCell<usize>,
    time_limit: Duration,
}

impl AlphaBetaAI {
    pub fn new(player_id: PlayerId, name: &str) -> Self {
        Self {
            player_id,
            name: name.to_string(),
            tt: RefCell::new(TranspositionTable::new(64)), // 64MB
            nodes_evaluated: RefCell::new(0),
            time_limit: Duration::from_secs(3), // 3秒で指す
        }
    }

    // --- Search Root (Iterative Deepening) ---
    fn search_root(&self, board: &Board) -> Option<Move> {
        self.tt.borrow_mut(); // Access check
        *self.nodes_evaluated.borrow_mut() = 0;
        let start_time = Instant::now();

        let mut best_move = None;
        let mut alpha = -200000;
        let mut beta = 200000;

        let max_depth = 10; // 制限

        for depth in 1..=max_depth {
            let score = self.alpha_beta(board, depth, alpha, beta, true, self.player_id);

            // Check time
            if start_time.elapsed() > self.time_limit {
                break;
            }

            // Get best move from TT for this depth
            let hash = ZobristHasher::compute_hash(board, self.player_id);
            if let Some((entry, mv)) = self.tt.borrow().get(hash) {
                if let Some(m) = mv {
                    best_move = Some(m);
                    // println!("Depth {} score: {} move: {:?}", depth, score, m);
                }
            }
        }

        best_move
    }

    // --- Alpha-Beta Search ---
    fn alpha_beta(
        &self,
        board: &Board,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        is_hero_pov: bool, // 自分の視点(Max)か、相手の視点(Min)か
        current_player: PlayerId,
    ) -> i32 {
        *self.nodes_evaluated.borrow_mut() += 1;

        let alpha_orig = alpha;
        let hash = ZobristHasher::compute_hash(board, current_player);

        // TT Lookup
        if let Some((entry, _)) = self.tt.borrow().get(hash) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        // Base case or Checkmate/Stalemate
        let in_check = is_in_check(board, current_player);
        if depth == 0 {
            return self.quiescence(board, alpha, beta, current_player);
        }

        let mut moves = legal_moves(board, current_player);
        if moves.is_empty() {
            if in_check {
                return -100000 + (10 - depth as i32); // 早く負けるより長く粘る
            } else {
                return 0; // Stalemate
            }
        }

        // Move Ordering
        self.order_moves(board, &mut moves, current_player);

        let mut best_score = -200000; // init with -inf
        let mut best_move = None;

        for mv in moves {
            let next_board = apply_move(board, &mv, current_player);
            // 再帰
            // 相手のターンなのでスコアは反転する (Negamax形式ではないが、ここではMinimax的に実装)
            // Minimax形式:
            // 自分(Maximizing): 子ノード(Minimizing)の最大値
            // 相手(Minimizing): 子ノード(Maximizing)の最小値
            // ここではAlphaBeta関数自体を常に「自分視点のスコア」を返すように設計するか、
            // 素直に Maximizing/Minimizing フラグで分けるか。
            // ここでは Minimax型 で実装する。

            let score = if is_hero_pov {
                // 今は自分(Max)。次は相手(Min)
                let val = self.alpha_beta(
                    &next_board,
                    depth - 1,
                    alpha,
                    beta,
                    false,
                    current_player.opponent(),
                );
                val
            } else {
                // 今は相手(Min)。次は自分(Max)
                let val = self.alpha_beta(
                    &next_board,
                    depth - 1,
                    alpha,
                    beta,
                    true,
                    current_player.opponent(),
                );
                val
            };

            if is_hero_pov {
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv.clone());
                }
                alpha = alpha.max(score);
            } else {
                // Min node wants to minimize score
                if best_score == -200000 {
                    best_score = 200000;
                } // init for min
                if score < best_score {
                    best_score = score;
                    best_move = Some(mv.clone());
                }
                beta = beta.min(score);
            }

            if alpha >= beta {
                break; // Beta Cut / Alpha Cut
            }
        }

        // TT Store
        let bound = if best_score <= alpha_orig {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        self.tt
            .borrow_mut()
            .store(hash, depth, best_score, bound, best_move);

        best_score
    }

    // --- Quiescence Search (静止探索) ---
    // 評価値が安定するまで（駒の取り合いが終わるまで）探索を続ける
    fn quiescence(&self, board: &Board, _alpha: i32, _beta: i32, _current_player: PlayerId) -> i32 {
        *self.nodes_evaluated.borrow_mut() += 1;

        // Stand-pat (なにもしない場合の評価値)
        let stand_pat = Evaluator::evaluate(board, self.player_id);

        stand_pat
    }

    fn order_moves(&self, board: &Board, moves: &mut [Move], _player: PlayerId) {
        moves.sort_by_key(|mv| {
            let mut score = 0;
            match mv {
                Move::Normal { from, to, promote } => {
                    // 1. MVV-LVA (Most Valuable Victim - Least Valuable Aggressor)
                    if let Some(target) = board.get_piece(*to) {
                        // 取る駒の価値が高いほど優先
                        score -= Evaluator::evaluate(board, target.owner) - 1000;
                        // 取る自分の駒の価値が低いほど優先（リスク少）
                        if let Some(_attacker) = board.get_piece(*from) {
                            // ..
                        }
                        score -= 10000; // Capture Bonus
                    }
                    if *promote {
                        score -= 5000; // Promote Bonus
                    }
                }
                _ => {}
            }
            score
        });
    }
}

impl PlayerController for AlphaBetaAI {
    fn choose_move(&self, board: &Board, _moves: &[Move]) -> Option<Move> {
        // moves引数は使わず、AI内部でLegalMovesを生成して探索する
        // (UI側でフィルタリングされている可能性があるが、AIは全合法手を知る必要があるため)

        self.search_root(board)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_local(&self) -> bool {
        true
    }
}
