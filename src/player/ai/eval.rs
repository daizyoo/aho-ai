//! # Evaluation Module
//!
//! This module implements the static evaluation function for the game state.
//! It converts a given `Board` state into a single integer score from the perspective
//! of `Player1` (positive = Player1 advantage, negative = Player2 advantage).
//!
//! ## Scoring Strategy
//! The score is composed of:
//! 1. **Material Balance**: Sum of values of all pieces on the board.
//! 2. **Piece-Square Tables (PST)**: Positional bonuses for pieces (e.g., King safety, advancing pawns).
//! 3. **Hand Material**: Value of captured pieces (drops) with a multiplier bonus.
//!
//! ## Values
//! - Material values are tuned for a mixed Shogi/Chess environment.
//! - Hand pieces are valued slightly higher (1.2x) to encourage efficient reuse/drops.

use super::config::AIConfig;
use super::evaluator::Evaluator;
use crate::core::{Board, PieceKind, PlayerId};
use crate::player::ai::pst::get_pst_value;

/// Handcrafted evaluation function (rule-based)
pub struct HandcraftedEvaluator;

impl Evaluator for HandcraftedEvaluator {
    fn evaluate(&mut self, board: &Board) -> i32 {
        evaluate(board)
    }

    fn name(&self) -> String {
        "Handcrafted".to_string()
    }
}

impl HandcraftedEvaluator {
    pub fn new() -> Self {
        Self
    }
}

// Material Values (in centipawns, CP)
// Adjusted for Mixed Shogi/Chess environment
const VAL_PAWN: i32 = 100;
const VAL_LANCE: i32 = 300;
const VAL_S_KNIGHT: i32 = 400; // Knight is strong in Shogi due to jumping
const VAL_C_KNIGHT: i32 = 500; // Chess Knight moves in 8 directions (Silver-class)
const VAL_SILVER: i32 = 500;
const VAL_GOLD: i32 = 600;
const VAL_BISHOP: i32 = 800;
const VAL_ROOK: i32 = 1000;
/// King value is effectively infinite for checkmate search, but finite here to allow pruning.
const VAL_KING: i32 = 20000;

// Promoted Pieces (Shogi)
const VAL_PRO_PAWN: i32 = 700; // Tokin is as valuable as a Gold
const VAL_PRO_LANCE: i32 = 700;
const VAL_PRO_KNIGHT: i32 = 700;
const VAL_PRO_SILVER: i32 = 700;
const VAL_PRO_BISHOP: i32 = 1200; // Horse
const VAL_PRO_ROOK: i32 = 1500; // Dragon

// Chess defaults
const VAL_QUEEN: i32 = 1800; // Strongest sliding piece

/// Returns the static material value of a piece kind.
///
/// These values represent the inherent worth of a piece type, independent of its position.
pub fn piece_val(k: PieceKind) -> i32 {
    match k {
        PieceKind::S_Pawn | PieceKind::C_Pawn => VAL_PAWN,
        PieceKind::S_Lance => VAL_LANCE,
        PieceKind::S_Knight => VAL_S_KNIGHT,
        PieceKind::C_Knight => VAL_C_KNIGHT,
        PieceKind::S_Silver => VAL_SILVER,
        PieceKind::S_Gold => VAL_GOLD,
        PieceKind::S_Bishop | PieceKind::C_Bishop => VAL_BISHOP,
        PieceKind::S_Rook | PieceKind::C_Rook => VAL_ROOK,
        PieceKind::S_King | PieceKind::C_King => VAL_KING,
        PieceKind::C_Queen => VAL_QUEEN,

        // Promoted Shogi
        PieceKind::S_ProPawn => VAL_PRO_PAWN,
        PieceKind::S_ProLance => VAL_PRO_LANCE,
        PieceKind::S_ProKnight => VAL_PRO_KNIGHT,
        PieceKind::S_ProSilver => VAL_PRO_SILVER,
        PieceKind::S_ProBishop => VAL_PRO_BISHOP,
        PieceKind::S_ProRook => VAL_PRO_ROOK,
    }
}

/// Game phase for phase-dependent evaluation
#[derive(Debug, Clone, Copy, PartialEq)]
enum GamePhase {
    Opening, // Most pieces alive
    Midgame, // Normal play
    Endgame, // Few pieces left
}

/// Count total material on board (both players)
fn count_total_material(board: &Board) -> i32 {
    let mut total = 0;
    for piece in board.pieces.values() {
        total += piece_val(piece.kind);
    }
    total
}

/// Detect current game phase based on material
fn detect_game_phase(board: &Board) -> GamePhase {
    let total_material = count_total_material(board);

    // Thresholds tuned for 9x9 board with mixed pieces
    if total_material > 8000 {
        GamePhase::Opening
    } else if total_material > 4000 {
        GamePhase::Midgame
    } else {
        GamePhase::Endgame
    }
}

/// Calculate mobility score (piece activity)
fn calculate_mobility(board: &Board, player: PlayerId) -> i32 {
    let moves = crate::logic::legal_moves(board, player);
    let mut weighted_mobility = 0;

    for mv in &moves {
        match mv {
            crate::core::Move::Normal { to, promote, .. } => {
                // Check if destination square has an enemy piece (capture)
                let is_capture = board.get_piece(*to).is_some();

                if is_capture {
                    weighted_mobility += 3;
                } else if promote.is_some() {
                    weighted_mobility += 2;
                } else {
                    weighted_mobility += 1;
                }
            }
            crate::core::Move::Drop { .. } => {
                // Drops provide flexibility
                weighted_mobility += 1;
            }
        }
    }

    // Scale to reasonable range (0-200 CP)
    // Typical position has 30-80 legal moves
    (weighted_mobility * 2).min(200)
}

/// Evaluates the current board state and returns a score from Player1's perspective.
///
/// Positive score indicates Player1 advantage.
/// Negative score indicates Player2 advantage.
///
/// # Metrics
/// - **Material**: Sum of pieces on board + PST bonuses.
/// - **Hand**: Sum of captured pieces * multiplier (from config).
pub fn evaluate(board: &Board) -> i32 {
    // Use cached config - zero overhead after first access
    let hand_multiplier = AIConfig::get().evaluation.hand_piece_bonus_multiplier as f32;
    let mut score = 0;

    // === 0. Detect Game Phase ===
    let phase = detect_game_phase(board);

    // 1. Material & PST (Piece-Square Tables)
    let mut p1_king = None;
    let mut p2_king = None;
    let mut p1_pawn_cols = vec![0; board.width];
    let mut p2_pawn_cols = vec![0; board.width];

    // Board stores pieces in a HashMap<Position, Piece>.
    // Iteration order doesn't matter as addition is commutative.
    for (&pos, piece) in &board.pieces {
        let mat = piece_val(piece.kind);

        // Calculate PST index (y * 9 + x) assuming 9x9 board and 0-indexed Position
        let idx = (pos.y * 9) + pos.x;
        let pst = get_pst_value(piece.kind, idx, piece.owner);

        if piece.owner == PlayerId::Player1 {
            score += mat + pst;
            if matches!(piece.kind, PieceKind::S_King | PieceKind::C_King) {
                p1_king = Some(pos);
            }
            if matches!(piece.kind, PieceKind::S_Pawn | PieceKind::C_Pawn) {
                p1_pawn_cols[pos.x] += 1;
            }
        } else {
            // Player2's pieces count negatively against Player1
            score = score.saturating_sub(mat + pst);
            if matches!(piece.kind, PieceKind::S_King | PieceKind::C_King) {
                p2_king = Some(pos);
            }
            if matches!(piece.kind, PieceKind::S_Pawn | PieceKind::C_Pawn) {
                p2_pawn_cols[pos.x] += 1;
            }
        }
    }

    // Pawn Structure (Doubled & Isolated)
    const PENALTY_DOUBLED: i32 = 20;
    const PENALTY_ISOLATED: i32 = 20;

    for x in 0..board.width {
        // Player 1
        if p1_pawn_cols[x] > 1 {
            score -= PENALTY_DOUBLED * (p1_pawn_cols[x] - 1) as i32;
        }
        if p1_pawn_cols[x] > 0 {
            let left = if x > 0 { p1_pawn_cols[x - 1] } else { 0 };
            let right = if x < board.width - 1 {
                p1_pawn_cols[x + 1]
            } else {
                0
            };
            if left == 0 && right == 0 {
                score -= PENALTY_ISOLATED;
            }
        }

        // Player 2 (Symmetric, subtract from score means adding to their advantage)
        if p2_pawn_cols[x] > 1 {
            score += PENALTY_DOUBLED * (p2_pawn_cols[x] - 1) as i32;
        }
        if p2_pawn_cols[x] > 0 {
            let left = if x > 0 { p2_pawn_cols[x - 1] } else { 0 };
            let right = if x < board.width - 1 {
                p2_pawn_cols[x + 1]
            } else {
                0
            };
            if left == 0 && right == 0 {
                score += PENALTY_ISOLATED;
            }
        }
    }

    // 2. King Safety Bonus (Enhanced with escape squares & attackers)
    if let Some(kpos) = p1_king {
        score += enhanced_king_safety(board, kpos, PlayerId::Player1, phase);
    }
    if let Some(kpos) = p2_king {
        score -= enhanced_king_safety(board, kpos, PlayerId::Player2, phase);
    }

    // 2. Hand Material
    // Evaluation favors having pieces in hand slightly more than raw material
    // to account for drop flexibility.

    // Add Player1's hand value
    if let Some(hand) = board.hand.get(&PlayerId::Player1) {
        for (kind, &count) in hand {
            if count > 0 {
                let val = piece_val(*kind);
                score += (val as f32 * hand_multiplier) as i32 * count as i32;
            }
        }
    }

    // Subtract Player2's hand value
    if let Some(hand) = board.hand.get(&PlayerId::Player2) {
        for (kind, &count) in hand {
            if count > 0 {
                let val = piece_val(*kind);
                score -= (val as f32 * hand_multiplier) as i32 * count as i32;
            }
        }
    }

    // NEW: Mobility Evaluation (piece activity)
    let p1_mobility = calculate_mobility(board, PlayerId::Player1);
    let p2_mobility = calculate_mobility(board, PlayerId::Player2);
    score += p1_mobility - p2_mobility;

    // NEW: Tactical Patterns (passed pawns, bishop pair, rooks on open files)
    let p1_tactical = detect_tactical_patterns(board, PlayerId::Player1);
    let p2_tactical = detect_tactical_patterns(board, PlayerId::Player2);
    score += p1_tactical - p2_tactical;

    // NEW: Development (opening only)
    let p1_dev = development_score(board, PlayerId::Player1, phase);
    let p2_dev = development_score(board, PlayerId::Player2, phase);
    score += p1_dev - p2_dev;

    score
}

// Helper for King Safety
// Checks 3x3 area around king for friendly defenders
// Now phase-aware: more important in opening, less in endgame
fn calc_king_safety(
    board: &Board,
    kpos: crate::core::Position,
    owner: PlayerId,
    phase: GamePhase,
) -> i32 {
    let mut safety = 0;
    // Offsets for neighbors
    let offsets = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for (dx, dy) in offsets {
        let nx = kpos.x as i32 + dx;
        let ny = kpos.y as i32 + dy;

        if nx >= 0 && nx < board.width as i32 && ny >= 0 && ny < board.height as i32 {
            let npos = crate::core::Position {
                x: nx as usize,
                y: ny as usize,
            };
            if let Some(piece) = board.get_piece(npos) {
                if piece.owner == owner {
                    // Bonus for defenders
                    safety += match piece.kind {
                        PieceKind::S_Gold
                        | PieceKind::S_Silver
                        | PieceKind::S_ProPawn
                        | PieceKind::S_ProLance
                        | PieceKind::S_ProKnight
                        | PieceKind::S_ProSilver
                        | PieceKind::S_ProBishop
                        | PieceKind::S_ProRook => 40, // Strong defenders
                        PieceKind::S_Lance | PieceKind::S_Knight => 20,
                        PieceKind::S_Pawn | PieceKind::C_Pawn => 15, // Wall
                        _ => 10,
                    };
                }
            }
        }
    }

    // Phase adjustment: King safety more critical in opening
    match phase {
        GamePhase::Opening => safety * 2, // 2x weight: protect king early
        GamePhase::Midgame => safety,     // Normal weight
        GamePhase::Endgame => safety / 2, // 0.5x weight: king can be active
    }
}

/// Count escape squares for king (empty or capturable squares)
fn count_king_escape_squares(
    board: &Board,
    king_pos: crate::core::Position,
    owner: PlayerId,
) -> i32 {
    let offsets = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut escape_count = 0;
    for (dx, dy) in offsets {
        let nx = king_pos.x as i32 + dx;
        let ny = king_pos.y as i32 + dy;

        if nx >= 0 && nx < board.width as i32 && ny >= 0 && ny < board.height as i32 {
            let npos = crate::core::Position {
                x: nx as usize,
                y: ny as usize,
            };

            match board.get_piece(npos) {
                None => escape_count += 1,                                // Empty square
                Some(piece) if piece.owner != owner => escape_count += 1, // Can capture
                _ => {}                                                   // Blocked by own piece
            }
        }
    }

    escape_count
}

/// Count enemy attackers near king (within 2 squares)
fn count_enemy_attackers(board: &Board, king_pos: crate::core::Position, owner: PlayerId) -> i32 {
    let mut attackers = 0;

    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = king_pos.x as i32 + dx;
            let ny = king_pos.y as i32 + dy;

            if nx >= 0 && nx < board.width as i32 && ny >= 0 && ny < board.height as i32 {
                let npos = crate::core::Position {
                    x: nx as usize,
                    y: ny as usize,
                };

                if let Some(piece) = board.get_piece(npos) {
                    if piece.owner != owner {
                        attackers += match piece.kind {
                            PieceKind::S_Rook
                            | PieceKind::S_ProRook
                            | PieceKind::C_Rook
                            | PieceKind::C_Queen => 3,
                            PieceKind::S_Bishop | PieceKind::S_ProBishop | PieceKind::C_Bishop => 2,
                            _ => 1,
                        };
                    }
                }
            }
        }
    }

    attackers
}

/// Enhanced king safety with multiple factors
fn enhanced_king_safety(
    board: &Board,
    king_pos: crate::core::Position,
    owner: PlayerId,
    phase: GamePhase,
) -> i32 {
    let mut safety = 0;

    // 1. Basic defender count (existing logic)
    safety += calc_king_safety(board, king_pos, owner, phase);

    // 2. Escape squares (important to avoid checkmate)
    let escapes = count_king_escape_squares(board, king_pos, owner);
    safety += escapes * 15;

    // 3. Enemy attackers penalty
    let attackers = count_enemy_attackers(board, king_pos, owner);
    safety -= attackers * 30;

    safety
}

/// Detect tactical patterns and return bonus score
fn detect_tactical_patterns(board: &Board, player: PlayerId) -> i32 {
    let mut bonus = 0;

    // 1. Passed pawns
    bonus += count_passed_pawns(board, player) * 50;

    // 2. Bishop pair
    if has_bishop_pair(board, player) {
        bonus += 30;
    }

    // 3. Rooks on open files
    bonus += count_rooks_on_open_files(board, player) * 40;

    bonus
}

/// Count passed pawns (simplified: no enemy pawns ahead in column)
fn count_passed_pawns(board: &Board, player: PlayerId) -> i32 {
    let mut passed = 0;
    let forward_dir = if player == PlayerId::Player1 { -1 } else { 1 };

    for (&pos, piece) in &board.pieces {
        if piece.owner == player && matches!(piece.kind, PieceKind::S_Pawn | PieceKind::C_Pawn) {
            let mut is_passed = true;

            // Check ahead in this column for enemy pawns
            let mut check_y = pos.y as i32 + forward_dir;
            while check_y >= 0 && check_y < 9 {
                let check_pos = crate::core::Position {
                    x: pos.x,
                    y: check_y as usize,
                };

                if let Some(p) = board.get_piece(check_pos) {
                    if p.owner != player && matches!(p.kind, PieceKind::S_Pawn | PieceKind::C_Pawn)
                    {
                        is_passed = false;
                        break;
                    }
                }

                check_y += forward_dir;
            }

            if is_passed {
                passed += 1;
            }
        }
    }

    passed
}

/// Check if player has bishop pair
fn has_bishop_pair(board: &Board, player: PlayerId) -> bool {
    let mut bishop_count = 0;

    for piece in board.pieces.values() {
        if piece.owner == player && matches!(piece.kind, PieceKind::S_Bishop | PieceKind::C_Bishop)
        {
            bishop_count += 1;
            if bishop_count >= 2 {
                return true;
            }
        }
    }

    false
}

/// Count rooks on open files
fn count_rooks_on_open_files(board: &Board, player: PlayerId) -> i32 {
    let mut rooks_on_open = 0;

    for (&pos, piece) in &board.pieces {
        if piece.owner == player
            && matches!(
                piece.kind,
                PieceKind::S_Rook | PieceKind::C_Rook | PieceKind::S_ProRook
            )
        {
            let mut has_pawn = false;
            for y in 0..9 {
                let check_pos = crate::core::Position { x: pos.x, y };
                if let Some(p) = board.get_piece(check_pos) {
                    if matches!(p.kind, PieceKind::S_Pawn | PieceKind::C_Pawn) {
                        has_pawn = true;
                        break;
                    }
                }
            }

            if !has_pawn {
                rooks_on_open += 1;
            }
        }
    }

    rooks_on_open
}

/// Penalize undeveloped pieces in opening
fn development_score(board: &Board, player: PlayerId, phase: GamePhase) -> i32 {
    if !matches!(phase, GamePhase::Opening) {
        return 0; // Only active in opening
    }

    let mut undeveloped = 0;
    let start_rank = if player == PlayerId::Player1 { 8 } else { 0 };

    for (&pos, piece) in &board.pieces {
        if piece.owner == player && pos.y == start_rank {
            // Major pieces should be developed
            if matches!(
                piece.kind,
                PieceKind::S_Bishop
                    | PieceKind::C_Bishop
                    | PieceKind::S_Rook
                    | PieceKind::C_Rook
                    | PieceKind::C_Knight
                    | PieceKind::S_Knight
            ) {
                undeveloped += 1;
            }
        }
    }

    -undeveloped * 10 // Small penalty per undeveloped piece
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_values() {
        // Verify relative values
        assert!(piece_val(PieceKind::S_Rook) > piece_val(PieceKind::S_Gold));
        assert!(piece_val(PieceKind::S_Gold) > piece_val(PieceKind::S_Pawn));
        assert!(piece_val(PieceKind::C_Queen) > piece_val(PieceKind::S_Rook));
    }

    #[test]
    fn test_eval_material_balance() {
        let mut board = Board::new(9, 9);
        // Empty board score should be 0
        assert_eq!(evaluate(&board), 0);

        // Add P1 Pawn
        board.place_piece(
            crate::core::Position { x: 0, y: 0 },
            crate::core::Piece {
                kind: PieceKind::S_Pawn,
                owner: PlayerId::Player1,
                is_shogi: true,
            },
        );
        let score_p1 = evaluate(&board);
        assert!(score_p1 > 0);

        // Add P2 Pawn (offsetting)
        // Note: PST values differ by position, so it might not be exactly 0
        // unless positions are symmetric relative to PST.
        // Let's just check sign.
        board.place_piece(
            crate::core::Position { x: 0, y: 8 },
            crate::core::Piece {
                kind: PieceKind::S_Pawn,
                owner: PlayerId::Player2,
                is_shogi: true,
            },
        );
        // Should be roughly balanced
        let score_balanced = evaluate(&board);
        assert!(score_balanced.abs() < 200); // PST diff is small
    }
}
