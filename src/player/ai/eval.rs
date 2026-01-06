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

    fn name(&self) -> &str {
        "Handcrafted"
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
fn piece_val(k: PieceKind) -> i32 {
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

    // 2. King Safety Bonus (Castle)
    // Add bonus for friendly pieces around the king.
    if let Some(kpos) = p1_king {
        score += calc_king_safety(board, kpos, PlayerId::Player1);
    }
    if let Some(kpos) = p2_king {
        score -= calc_king_safety(board, kpos, PlayerId::Player2);
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

    score
}

// Helper for King Safety
// Checks 3x3 area around king for friendly defenders
fn calc_king_safety(board: &Board, kpos: crate::core::Position, owner: PlayerId) -> i32 {
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
    safety
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
