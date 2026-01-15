use super::eval::HandcraftedEvaluator;
use super::evaluator::Evaluator;
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
    killer_moves: RefCell<[[Option<Move>; 2]; 64]>,                // Ply indexed
    evaluator: RefCell<Box<dyn Evaluator>>,
}

const MAX_PLY: usize = 64;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AIStrength {
    Strong,
    Light,
}

impl AlphaBetaAI {
    #[allow(unused)]
    pub fn new(
        player_id: PlayerId,
        name: &str,
        strength: AIStrength,
        custom_model_path: Option<String>,
        silent: bool,
    ) -> Self {
        use crate::player::ai::config::AIConfig;

        // Create evaluator based on config
        let config = AIConfig::get();
        let evaluator: Box<dyn Evaluator> = match config.evaluation.evaluator_type.as_str() {
            "NeuralNetwork" => {
                #[cfg(feature = "ml")]
                {
                    use crate::ml::nn_evaluator::NNEvaluator;

                    let model_path =
                        custom_model_path.or_else(|| config.evaluation.nn_model_path.clone());

                    if let Some(ref path) = model_path {
                        let result = if silent {
                            NNEvaluator::load_silent(path)
                        } else {
                            NNEvaluator::load(path)
                        };

                        match result {
                            Ok(nn_eval) => {
                                // Successfully loaded
                                Box::new(nn_eval)
                            }
                            Err(_e) => {
                                // Failed to load, fallback
                                Box::new(HandcraftedEvaluator::new())
                            }
                        }
                    } else {
                        // No model path
                        Box::new(HandcraftedEvaluator::new())
                    }
                }
                #[cfg(not(feature = "ml"))]
                {
                    // ML feature not enabled
                    Box::new(HandcraftedEvaluator::new())
                }
            }
            _ => {
                // Default: Handcrafted evaluator
                Box::new(HandcraftedEvaluator::new())
            }
        };

        Self {
            player_id,
            name: name.to_string(),
            tt: RefCell::new(TranspositionTable::new(64)), // 64MB
            nodes_evaluated: RefCell::new(0),
            time_limit: Duration::from_secs(if strength == AIStrength::Strong { 3 } else { 1 }),
            strength,
            last_thinking: RefCell::new(None),
            killer_moves: RefCell::new([[None; 2]; MAX_PLY]),
            evaluator: RefCell::new(evaluator),
        }
    }

    /// Get the name of the evaluator being used
    pub fn evaluator_name(&self) -> String {
        self.evaluator.borrow().name().to_string()
    }

    // --- Search Root (Iterative Deepening) ---
    fn search_root(&self, board: &Board) -> Option<Move> {
        self.tt.borrow_mut().clear(); // Clear TT for new search
        *self.nodes_evaluated.borrow_mut() = 0;
        *self.killer_moves.borrow_mut() = [[None; 2]; MAX_PLY];
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
            // Clear killer moves for new ID iteration? No, keep them within same search.
            // But we should probably clear them between searches (done in search_root start)
            let score = self.negamax(board, depth, alpha, beta, self.player_id, 0);

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

        // Fallback: if no move found (time ran out before completing depth=1),
        // select first legal move
        if best_move.is_none() {
            let legal_moves = legal_moves(board, self.player_id);
            if !legal_moves.is_empty() {
                best_move = Some(legal_moves[0].clone());
                final_depth = 0;
                final_score = 0;
                eprintln!("⚠️ AI time limit exceeded, using fallback move");
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
        ply: usize,
    ) -> i32 {
        *self.nodes_evaluated.borrow_mut() += 1;

        let alpha_orig = alpha;
        let hash = ZobristHasher::compute_hash(board, current_player);

        // Repetition Check (Sennichite)
        // Count how many times this position appeared in history
        let rep_count = board.history.iter().filter(|&&h| h == hash).count();

        if rep_count >= 4 {
            // 4-fold repetition: Return draw score (0)
            // The game loop will handle this as a loss for the player who caused it
            return 0;
        }

        // For 3-fold repetition, apply small penalty only at root (ply == 0)
        // This gently discourages repetition without breaking search
        if rep_count >= 3 && ply == 0 {
            return -100; // Small penalty to prefer non-repetitive moves
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
            return self.qsearch(board, alpha, beta, current_player);
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
                    ply + 1 + r, // Approximate ply
                );

                if score >= beta {
                    return beta;
                }
            }
        }

        // Move Ordering
        self.order_moves(board, &mut moves, current_player, ply);

        let mut best_score = -200000;
        let mut best_move = None;

        for (moves_searched, mv) in moves.iter().enumerate() {
            // SEE Pruning: Skip obviously bad captures
            // Only prune in non-PV nodes (moves_searched > 0) and when not in check
            if !in_check && moves_searched > 0 && depth >= 2 {
                // Only call SEE for capture moves
                if let Move::Normal { from, to, .. } = mv {
                    if let Some(captured_piece) = board.get_piece(*to) {
                        // Quick heuristic: skip SEE for obviously good captures
                        if let Some(our_piece) = board.get_piece(*from) {
                            use crate::player::ai::eval::piece_val;
                            let our_value = piece_val(our_piece.kind);
                            let their_value = piece_val(captured_piece.kind);

                            // If we're capturing a more valuable piece with decent margin, likely good
                            // Skip expensive SEE evaluation
                            if their_value > our_value + 200 {
                                // Obviously good capture (e.g., pawn takes bishop)
                                // Skip SEE
                            } else {
                                // Questionable capture - evaluate with SEE
                                let see_value = crate::player::ai::see::static_exchange_eval(
                                    board,
                                    mv,
                                    current_player,
                                );
                                // If we lose more than a pawn (>150) in the exchange, skip this move
                                if see_value < -150 {
                                    continue; // Prune this move
                                }
                            }
                        }
                        // If we can't get our piece, skip SEE (shouldn't happen)
                    }
                    // If it's not a capture, no SEE needed
                }
                // If it's a drop, no SEE needed
            }

            let next_board = apply_move(board, mv, current_player);
            let mut score;

            // PVS & LMR (Only for Strong AI and non-first moves)
            if self.strength == AIStrength::Strong && moves_searched > 0 && depth >= 2 {
                // Determine if this is a tactical move
                let mut is_capture = false;
                let mut is_promote_move = None;
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
                    && !is_promote_move.is_some()
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
                    ply + 1,
                );

                // Re-search if necessary
                if score > alpha && (score < beta || reduction > 0) {
                    score = -self.negamax(
                        &next_board,
                        depth - 1,
                        -beta,
                        -alpha,
                        current_player.opponent(),
                        ply + 1,
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
                    ply + 1,
                );
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }

            alpha = alpha.max(score);

            if alpha >= beta {
                // Beta Cutoff
                let is_capture = match mv {
                    Move::Normal { to, .. } => board.get_piece(*to).is_some(),
                    _ => false,
                };

                // Store Killer Move (if not capture and valid ply)
                if !is_capture && ply < MAX_PLY {
                    let mut killers = self.killer_moves.borrow_mut();
                    if killers[ply][0].as_ref() != Some(mv) {
                        killers[ply][1] = killers[ply][0].clone();
                        killers[ply][0] = Some(mv.clone());
                    }
                }

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

    fn order_moves(&self, board: &Board, moves: &mut [Move], _player: PlayerId, ply: usize) {
        // Collect killer moves for this ply
        let killers = if ply < MAX_PLY {
            self.killer_moves.borrow()[ply].clone()
        } else {
            [None, None]
        };

        moves.sort_by_key(|mv| {
            let mut score = 0;
            // Killer Move Bonus
            if Some(mv) == killers[0].as_ref() {
                score -= 20000; // Priority over captures (captures are usually checked by QS anyway, but here we prioritize quiet refutations)
            } else if Some(mv) == killers[1].as_ref() {
                score -= 19000;
            }

            if let Move::Normal { to, promote, .. } = mv {
                if board.get_piece(*to).is_some() {
                    score -= 10000; // Capture bonus
                }
                if promote.is_some() {
                    score -= 5000; // Promote bonus
                }
            }
            score
        });
    }

    // --- Quiescence Search ---
    // Searches only captures, promotions, and checks to reach a stable state.
    fn qsearch(&self, board: &Board, alpha: i32, beta: i32, current_player: PlayerId) -> i32 {
        self.qsearch_depth(board, alpha, beta, current_player, 0)
    }

    fn qsearch_depth(
        &self,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        current_player: PlayerId,
        depth: usize,
    ) -> i32 {
        const MAX_QSEARCH_DEPTH: usize = 8;

        *self.nodes_evaluated.borrow_mut() += 1;

        // 1. Stand-pat (Static Evaluation)
        let eval_score = self.evaluator.borrow_mut().evaluate(board);
        let stand_pat = if current_player == PlayerId::Player1 {
            eval_score
        } else {
            -eval_score
        };

        if stand_pat >= beta {
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Depth limit to prevent runaway recursion
        if depth >= MAX_QSEARCH_DEPTH {
            return alpha;
        }

        // 2. Generate and filter tactical moves
        let moves = legal_moves(board, current_player);
        let mut tactical_moves: Vec<_> = moves
            .into_iter()
            .filter(|mv| {
                match mv {
                    Move::Normal { to, promote, .. } => {
                        // Include captures and promotions
                        board.get_piece(*to).is_some() || promote.is_some()
                    }
                    Move::Drop { .. } => {
                        // Include drops that give check
                        // This is important for Shogi tactics
                        if depth < 4 {
                            // Only check for drops in early QS levels
                            let next_board = apply_move(board, mv, current_player);
                            is_in_check(&next_board, current_player.opponent())
                        } else {
                            false
                        }
                    }
                }
            })
            .collect();

        if tactical_moves.is_empty() {
            return alpha;
        }

        self.order_moves(board, &mut tactical_moves, current_player, MAX_PLY);

        for mv in tactical_moves {
            let next_board = apply_move(board, &mv, current_player);
            let score = -self.qsearch_depth(
                &next_board,
                -beta,
                -alpha,
                current_player.opponent(),
                depth + 1,
            );

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
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
