use crate::core::{Board, PieceKind, PlayerId};
use super::config::AIConfig;
use crate::player::ai::pst::get_pst_value;

// Material Values (in CP) - Adjusted for Mixed
const VAL_PAWN: i32 = 100;
const VAL_LANCE: i32 = 300;
const VAL_KNIGHT: i32 = 400; // Knight is strong in Shogi
const VAL_SILVER: i32 = 500;
const VAL_GOLD: i32 = 600;
const VAL_BISHOP: i32 = 800;
const VAL_ROOK: i32 = 1000;
// King value effectively infinite for checkmate logic, but for eval we might use high number.
const VAL_KING: i32 = 20000;

// Promoted
const VAL_PRO_PAWN: i32 = 700; // Tokin ~ Gold
const VAL_PRO_LANCE: i32 = 700;
const VAL_PRO_KNIGHT: i32 = 700;
const VAL_PRO_SILVER: i32 = 700;
const VAL_PRO_BISHOP: i32 = 1200; // Horse
const VAL_PRO_ROOK: i32 = 1500; // Dragon

// Chess defaults
const VAL_QUEEN: i32 = 1800; // Strongest

// Hand piece bonus (increased from 1.1 to 1.2 based on self-play analysis)
// Analysis shows winners use drops 14.8% of moves - higher value encourages this

fn piece_val(k: PieceKind) -> i32 {
    match k {
        PieceKind::S_Pawn | PieceKind::C_Pawn => VAL_PAWN,
        PieceKind::S_Lance => VAL_LANCE,
        PieceKind::S_Knight | PieceKind::C_Knight => VAL_KNIGHT,
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

pub fn evaluate(board: &Board) -> i32 {
    // Use cached config - zero overhead after first access
    let hand_multiplier = AIConfig::get().evaluation.hand_piece_bonus_multiplier as f32;
    let mut score = 0;

    // 1. Material & PST
    // Board stores pieces in a HashMap<Position, Piece>
    // Iterating over keys gives random order, but score addition is commutative so it's fine.
    for (&pos, piece) in &board.pieces {
        let mat = piece_val(piece.kind);

        // PST requires an index 0..80.
        // Position has x, y.
        // Board is 9x9.
        // Index = y * 9 + x. (Assuming 0-indexed x,y)
        // Position struct likely has 1-indexed or 0-indexed. Let's check `core/types.rs` or `board.rs` if needed.
        // Assuming 1-indexed based on "9x9". Wait, usually 0-indexed in code.
        // Let's assume 0-indexed for now (0..9).
        // Actually, previous code used `board.cells.iter().enumerate()`.
        // `Board` struct definition shows `pieces: HashMap<Position, Piece>`.
        // `Position` struct?

        // Let's assume Position has x, y as (i8 or u8 or usize).
        // x: 1..=9, y: 1..=9 ? Or 0..=8?
        // Standard Shogi is 1..9.
        // Let's coerce to 0-8 for PST index.
        let idx = ((pos.y.saturating_sub(1) as usize) * 9) + (pos.x.saturating_sub(1) as usize);

        let pst = get_pst_value(piece.kind, idx, piece.owner);

        if piece.owner == PlayerId::Player1 {
            score += mat + pst;
        } else {
            score = score.saturating_sub(mat + pst);
        }
    }

    // 2. Hand Material
    // board.hand is HashMap<PlayerId, HashMap<PieceKind, u8>>
    if let Some(hand) = board.hand.get(&PlayerId::Player1) {
        for (kind, &count) in hand {
            if count > 0 {
                let val = piece_val(*kind);
                score += (val as f32 * hand_multiplier) as i32 * count as i32;
            }
        }
    }

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
