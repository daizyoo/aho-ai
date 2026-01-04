use super::eval;
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
    strength: AIStrength,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AIStrength {
    Strong,
    Light,
}

impl AlphaBetaAI {
    pub fn new(player_id: PlayerId, name: &str, strength: AIStrength) -> Self {
        Self {
            player_id,
            name: name.to_string(),
            tt: RefCell::new(TranspositionTable::new(64)), // 64MB
            nodes_evaluated: RefCell::new(0),
            time_limit: Duration::from_secs(if strength == AIStrength::Strong { 3 } else { 1 }),
            strength,
        }
    }

    // --- Search Root (Iterative Deepening) ---
    fn search_root(&self, board: &Board) -> Option<Move> {
        self.tt.borrow_mut(); // Access check
        *self.nodes_evaluated.borrow_mut() = 0;
        let start_time = Instant::now();

        let mut best_move = None;
        let alpha = -200000;
        let beta = 200000;

        let max_depth = 10; // 制限

        for depth in 1..=max_depth {
            // Root Search
            let _score = self.alpha_beta(board, depth, alpha, beta, true, self.player_id);

            // Check time
            if start_time.elapsed() > self.time_limit {
                break;
            }

            // Get best move from TT for this depth
            let hash = ZobristHasher::compute_hash(board, self.player_id);
            if let Some((_entry, mv)) = self.tt.borrow().get(hash) {
                if let Some(m) = mv {
                    best_move = Some(m);
                }
            }

            // Aspiration Window (Optional, skipped for simplicity/safety)
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
            return eval::evaluate(board);
        }

        let mut moves = legal_moves(board, current_player);
        if moves.is_empty() {
            if in_check {
                return -200000 + (100 - depth) as i32; // Checkmate
            } else {
                // Stalemate
                return 0;
            }
        }

        // --- Null Move Pruning (NMP) ---
        // Only for Strong AI
        if self.strength == AIStrength::Strong && !in_check && depth >= 3 && beta.abs() < 100000 {
            let r = 2; // Reduction
                       // Null move: pass turn (swap current_player, board stays same)
                       // Result score is negated because we swapped perspective
            let score = -self.alpha_beta(
                board,
                depth - 1 - r,
                -beta,
                -beta + 1,
                !is_hero_pov,
                current_player.opponent(),
            );

            if score >= beta {
                return beta; // Cutoff
            }
        }

        // Move Ordering
        self.order_moves(board, &mut moves, current_player);

        let mut best_score = -200000; // init with -inf
        let mut best_move = None;
        let mut moves_searched = 0;

        for mv in moves.iter() {
            let next_board = apply_move(board, mv, current_player);
            // PVS & LMR Logic (Strong Only)
            let mut score;

            // PVS & LMR Logic (Strong Only)
            if self.strength == AIStrength::Strong && moves_searched > 0 {
                // Late Moves:
                // 1. LMR (Late Move Reduction)
                // Conditions: Depth >= 3, searched > 3 moves, not capture, not promote, not check.
                // Is it tactical?
                let mut is_capture = false;
                let mut is_promote_move = false;
                if let Move::Normal { to, promote, .. } = mv {
                    is_capture = board.get_piece(*to).is_some();
                    is_promote_move = *promote;
                }

                let gives_check = is_in_check(&next_board, current_player.opponent());

                let mut reduction = 0;
                if depth >= 3
                    && moves_searched >= 3
                    && !is_capture
                    && !is_promote_move
                    && !gives_check
                {
                    reduction = 1;
                    if moves_searched > 6 {
                        reduction = 2;
                    }
                }

                // Search with reduced depth, null window
                let d = (depth - 1).saturating_sub(reduction).max(1);
                score = -self.alpha_beta(
                    &next_board,
                    d,
                    -alpha - 1,
                    -alpha,
                    !is_hero_pov,
                    current_player.opponent(),
                );

                // Re-search if LMR failed
                if reduction > 0 && score > alpha {
                    score = -self.alpha_beta(
                        &next_board,
                        depth - 1,
                        -alpha - 1,
                        -alpha,
                        !is_hero_pov,
                        current_player.opponent(),
                    );
                }

                // PVS re-search full window if null window failed (and still promising)
                if score > alpha && score < beta {
                    score = -self.alpha_beta(
                        &next_board,
                        depth - 1,
                        -beta,
                        -alpha,
                        !is_hero_pov,
                        current_player.opponent(),
                    );
                }
            } else {
                // PV-Move or Standard Search (Light AI or First Move)
                score = -self.alpha_beta(
                    &next_board,
                    depth - 1,
                    -beta,
                    -alpha,
                    !is_hero_pov,
                    current_player.opponent(),
                );
            }

            // Minimax / Max logic handled by Negamax recursion (score is already negated above)
            // But wait, my previous logic had explicit Max/Min node handling inside loop?
            // "if is_hero_pov ..." block.
            // My PVS logic above assumes Negamax style (-alpha_beta).
            // The previous logic was:
            // if is_hero_pov { match max } else { match min }
            // But wait, if I use Negamax (-self.alpha_beta), I don't need is_hero_pov branching for alpha/beta update!
            // Negamax unifies it.
            // BUT, `is_hero_pov` is passed to recursive calls.
            // And in `search_root`, I call it with `is_hero_pov = true`.
            // If I switch to Pure Negamax, I can remove `is_hero_pov` logic inside the loop for alpha/beta update.
            // The score returned IS the score from current player's perspective.
            // So we just want to maximize it.

            // Let's stick to Pure Negamax behavior since I wrote `-self.alpha_beta`.
            // So:
            if score > best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }
            alpha = alpha.max(score);

            if alpha >= beta {
                break; // Beta Cutoff
            }
            moves_searched += 1;
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

    fn order_moves(&self, board: &Board, moves: &mut [Move], _player: PlayerId) {
        moves.sort_by_key(|mv| {
            let mut score = 0;
            match mv {
                Move::Normal { from, to, promote } => {
                    // 1. MVV-LVA (Most Valuable Victim - Least Valuable Aggressor)
                    if let Some(_target) = board.get_piece(*to) {
                        // 取る駒の価値が高いほど優先
                        score -= eval::evaluate(board) - 1000;
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
        self.search_root(board)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_local(&self) -> bool {
        true
    }
}
