use super::eval;
use super::tt::{Bound, TranspositionTable};
use crate::core::{Board, Move, PlayerId};
use crate::logic::ZobristHasher;
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
    pub last_thinking: RefCell<Option<(usize, i32, usize, u128)>>, // (depth, score, nodes, time_ms)
}

#[derive(Clone, Copy, PartialEq, Debug)]
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
            last_thinking: RefCell::new(None),
        }
    }

    // --- Search Root (Iterative Deepening) ---
    fn search_root(&self, board: &Board) -> Option<Move> {
        self.tt.borrow_mut().clear(); // Clear TT for new search
        *self.nodes_evaluated.borrow_mut() = 0;
        let start_time = Instant::now();

        let mut best_move = None;
        let alpha = -200000;
        let beta = 200000;

        // Reduced max depth to prevent stack overflow
        let max_depth = if self.strength == AIStrength::Strong {
            6
        } else {
            4
        };

        let mut final_depth = 0;
        let mut final_score = 0;
        for depth in 1..=max_depth {
            let score = self.negamax(board, depth, alpha, beta, self.player_id);

            // Check time
            if start_time.elapsed() > self.time_limit {
                break;
            }

            // Get best move from TT for this depth
            let hash = ZobristHasher::compute_hash(board, self.player_id);
            if let Some((_entry, Some(m))) = self.tt.borrow().get(hash) {
                best_move = Some(m);
                final_depth = depth;
                final_score = score;

                // Display thinking info
                if std::env::var("VERBOSE_AI").is_ok() {
                    print!(
                        " [d={} s={} n={}]",
                        depth,
                        score,
                        self.nodes_evaluated.borrow()
                    );
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                }
            }
        }

        // Save thinking data
        let elapsed = start_time.elapsed();
        *self.last_thinking.borrow_mut() = Some((
            final_depth,
            final_score,
            *self.nodes_evaluated.borrow(),
            elapsed.as_millis(),
        ));

        best_move
    }

    // --- Negamax with Alpha-Beta Pruning ---
    fn negamax(
        &self,
        board: &Board,
        depth: usize,
        mut alpha: i32,
        beta: i32,
        current_player: PlayerId,
    ) -> i32 {
        *self.nodes_evaluated.borrow_mut() += 1;

        let alpha_orig = alpha;
        let hash = ZobristHasher::compute_hash(board, current_player);

        // Repetition Check (Sennichite)
        // If this position has appeared 3 times before (total 4), it's a draw.
        // board.history includes the current hash if this is a leaf/node we just moved to?
        // Wait, apply_move adds hash to history. So board.history ALREADY contains 'hash'.
        // So we count how many times 'hash' is in 'board.history'.
        // If count >= 4, return 0.
        let rep_count = board.history.iter().filter(|&&h| h == hash).count();
        if rep_count >= 4 {
            return 0; // Draw score
        }

        // TT Lookup
        if let Some((entry, _)) = self.tt.borrow().get(hash) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => {
                        if entry.score < beta {
                            return entry.score;
                        }
                    }
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        // Leaf node
        if depth == 0 {
            let score = eval::evaluate(board);
            // Negamax: always return score from current player's perspective
            // evaluate() returns score from Player1's perspective
            return if current_player == PlayerId::Player1 {
                score
            } else {
                -score
            };
        }

        let in_check = is_in_check(board, current_player);
        let mut moves = legal_moves(board, current_player);

        if moves.is_empty() {
            if in_check {
                return -200000 + (100 - depth) as i32; // Checkmate
            } else {
                return 0; // Stalemate
            }
        }

        // --- Null Move Pruning (NMP) --- (Only for Strong AI)
        if self.strength == AIStrength::Strong && !in_check && depth >= 3 && beta.abs() < 100000 {
            let r = 2;
            if depth > r {
                let score = -self.negamax(
                    board,
                    depth - r - 1,
                    -beta,
                    -beta + 1,
                    current_player.opponent(),
                );

                if score >= beta {
                    return beta;
                }
            }
        }

        // Move Ordering
        self.order_moves(board, &mut moves, current_player);

        let mut best_score = -200000;
        let mut best_move = None;

        for (moves_searched, mv) in moves.iter().enumerate() {
            let next_board = apply_move(board, mv, current_player);
            let mut score;

            // PVS & LMR (Only for Strong AI and non-first moves)
            if self.strength == AIStrength::Strong && moves_searched > 0 && depth >= 2 {
                // Determine if this is a tactical move
                let mut is_capture = false;
                let mut is_promote_move = false;
                if let Move::Normal { to, promote, .. } = mv {
                    is_capture = board.get_piece(*to).is_some();
                    is_promote_move = *promote;
                }
                let gives_check = is_in_check(&next_board, current_player.opponent());

                // LMR
                let mut reduction = 0;
                if depth >= 3
                    && moves_searched >= 4
                    && !is_capture
                    && !is_promote_move
                    && !gives_check
                {
                    reduction = 1;
                }

                // Try reduced search first
                let search_depth = if depth > reduction + 1 {
                    depth - reduction - 1
                } else {
                    1
                };

                // Null window search
                score = -self.negamax(
                    &next_board,
                    search_depth,
                    -alpha - 1,
                    -alpha,
                    current_player.opponent(),
                );

                // Re-search if necessary
                if score > alpha && (score < beta || reduction > 0) {
                    score = -self.negamax(
                        &next_board,
                        depth - 1,
                        -beta,
                        -alpha,
                        current_player.opponent(),
                    );
                }
            } else {
                // First move or Light AI: full window search
                score = -self.negamax(
                    &next_board,
                    depth - 1,
                    -beta,
                    -alpha,
                    current_player.opponent(),
                );
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }

            alpha = alpha.max(score);

            if alpha >= beta {
                break; // Beta cutoff
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

    fn order_moves(&self, board: &Board, moves: &mut [Move], _player: PlayerId) {
        moves.sort_by_key(|mv| {
            let mut score = 0;
            if let Move::Normal { to, promote, .. } = mv {
                if board.get_piece(*to).is_some() {
                    score -= 10000; // Capture bonus
                }
                if *promote {
                    score -= 5000; // Promote bonus
                }
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
