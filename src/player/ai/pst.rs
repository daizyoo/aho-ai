use crate::core::{PieceKind, PlayerId};

// Scores are in centipawns (roughly).
// Perspective: Player 1 (Bottom moving Up).
// Index 0 is (0,0) -> top-left from white's perspective?
// Wait, our board is 9x9. (0,0) is usually top-left.
// Player 1 is at the bottom (rank 8, 9). Moving to rank 0.
// So:
// Rank 0 (Top) is promotion zone for P1.
// Rank 8 (Bottom) is home base for P1.
//
// Board index = y * 9 + x.
// y=0 is top. y=8 is bottom.
//
// So the table should be defined:
// [
//   Rank 0 (Top - Deep enemy territory)
//   Rank 1
//   ...
//   Rank 8 (Bottom - Home)
// ]

const MG_PAWN: [i32; 81] = [
    // Rank 0 (Promote!)
    15, 15, 15, 15, 15, 15, 15, 15, 15, // Rank 1
    10, 10, 10, 10, 10, 10, 10, 10, 10, // Rank 2 (Approaching)
    5, 5, 5, 5, 5, 5, 5, 5, 5, // Rank 3
    2, 2, 2, 10, 10, 2, 2, 2, 2, // Rank 4 (Center fight)
    1, 1, 2, 10, 10, 2, 1, 1, 1, // Rank 5
    0, 0, 0, 5, 5, 0, 0, 0, 0, // Rank 6
    0, 0, 0, -5, -5, 0, 0, 0, 0, // Rank 7
    0, 0, 0, -5, -5, 0, 0, 0, 0, // Rank 8 (Base)
    0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const MG_LANCE: [i32; 81] = [
    // Rank 0
    20, 20, 20, 20, 20, 20, 20, 20, 20, // Rank 1
    10, 10, 10, 10, 10, 10, 10, 10, 10, // Rank 2
    5, 5, 5, 5, 5, 5, 5, 5, 5, // ...
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, -5, -5, -5, -5, -5, -5, -5, -5, -5, // Don't block bottom
    0, 0, 0, 5, 5, 0, 0, 0, 0, // Center file is good
];

const MG_KNIGHT: [i32; 81] = [
    // Rank 0
    10, 10, 10, 10, 10, 10, 10, 10, 10, // Rank 1
    10, 10, 10, 10, 10, 10, 10, 10, 10, // Rank 2
    15, 15, 15, 15, 15, 15, 15, 15, 15, // Good square to jump into
    // Rank 3
    5, 5, 10, 10, 10, 10, 10, 5, 5, // Rank 4
    0, 0, 5, 5, 5, 5, 5, 0, 0, // Rank 5
    0, 0, 0, 0, 0, 0, 0, 0, 0, // Rank 6
    0, 0, 0, 0, 0, 0, 0, 0, 0, // Rank 7 (Start)
    0, 5, 0, 0, 0, 0, 0, 5, 0, // Rank 8
    0, -10, 0, 0, 0, 0, 0, -10, 0, // Stuck at bottom is bad
];

const MG_SILVER: [i32; 81] = [
    // Attacking piece, likes to be forward
    // Rank 0
    10, 10, 10, 10, 10, 10, 10, 10, 10, // Rank 1
    5, 5, 5, 5, 5, 5, 5, 5, 5, // Rank 2
    5, 5, 5, 5, 5, 5, 5, 5, 5, // Rank 3
    2, 2, 10, 10, 10, 10, 10, 2, 2, // Rank 4
    0, 2, 5, 5, 5, 5, 5, 2, 0, // ...
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -5, -5, -5,
    -5, -5, -5, -5, -5, -5,
];

const MG_GOLD: [i32; 81] = [
    // Defensive usually, but also good for mate
    // Rank 0
    10, 10, 10, 10, 10, 10, 10, 10, 10, // Rank 1
    5, 5, 5, 5, 5, 5, 5, 5, 5, // Rank 2
    0, 0, 0, 0, 0, 0, 0, 0, 0, // Rank 3
    0, 0, 2, 2, 2, 2, 2, 0, 0, // Rank 4
    0, 0, 2, 5, 5, 5, 2, 0, 0, // Rank 5
    0, 0, 1, 2, 2, 2, 1, 0, 0, // Rank 6
    0, 0, 0, 0, 0, 0, 0, 0, 0, // Rank 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, // Rank 8
    -5, -5, -5, -5, -5, -5, -5, -5, -5,
];

// King Safety Table (Middle Game / End Game hybrid)
// Prefer castles ( corners / side )
const MG_KING: [i32; 81] = [
    // Rank 0 - Dangerous
    -30, -40, -40, -40, -40, -40, -40, -40, -30, // Rank 1
    -30, -40, -40, -40, -40, -40, -40, -40, -30, // Rank 2
    -30, -40, -40, -40, -40, -40, -40, -40, -30, // Rank 3
    -30, -40, -40, -40, -40, -40, -40, -40, -30, // Rank 4
    -10, -20, -20, -20, -20, -20, -20, -20, -10, // Rank 5
    0, -10, -10, -10, -10, -10, -10, -10, 0, // Rank 6
    10, 0, 0, 0, 0, 0, 0, 0, 10, // Rank 7
    20, 10, 0, 0, 0, 0, 0, 10, 20, // Rank 8 (Castle)
    30, 40, 30, 10, 0, 10, 30, 40, 30,
];

// Generic fallback for others (Bishop, Rook, Queen, etc) - Centralize
const MG_GENERIC: [i32; 81] = [
    // Encouraging invasion (Rank 0-2) and Centralization
    10, 10, 10, 10, 10, 10, 10, 10, 10, // Rank 0 (Deep enemy territory)
    5, 5, 5, 5, 10, 5, 5, 5, 5, // Rank 1
    0, 0, 0, 5, 5, 5, 0, 0, 0, // Rank 2
    -5, 0, 5, 10, 10, 10, 5, 0, -5, // Rank 3
    -5, 0, 5, 10, 10, 10, 5, 0, -5, // Rank 4
    -5, 0, 0, 5, 5, 5, 0, 0, -5, // Rank 5
    -5, -5, 0, 0, 0, 0, 0, -5, -5, // Rank 6
    -5, -5, -5, -5, -5, -5, -5, -5, -5, // Rank 7
    -10, -10, -10, -10, -10, -10, -10, -10, -10, // Rank 8 (Back rank, bad for active pieces)
];

pub fn get_pst_value(kind: PieceKind, idx: usize, player: PlayerId) -> i32 {
    // If Player 2, we need to mirror the index
    // x = idx % 9
    // y = idx / 9
    // P2's y = 8 - y. x is typically also mirrored?
    // Wait, "Reversed" Mixed map has pieces at top.
    // If P2 is at top (Camp 0), attacking down (Camp 8).
    // Our tables are defined for P1 (Camp 8 -> Camp 0).
    // So for P2:
    // y_p2 = 8 - y_p1.
    // x_p2 = 8 - x_p1. (Mirror board completely)

    let table = match kind {
        PieceKind::S_Pawn | PieceKind::C_Pawn => &MG_PAWN,
        PieceKind::S_Lance => &MG_LANCE,
        PieceKind::S_Knight | PieceKind::C_Knight => &MG_KNIGHT,
        PieceKind::S_Silver => &MG_SILVER,
        PieceKind::S_Gold => &MG_GOLD,
        PieceKind::S_King | PieceKind::C_King => &MG_KING,
        // Major pieces
        PieceKind::S_Bishop
        | PieceKind::C_Bishop
        | PieceKind::S_Rook
        | PieceKind::C_Rook
        | PieceKind::S_ProBishop
        | PieceKind::S_ProRook
        | PieceKind::C_Queen => &MG_GENERIC,
        // Promoted pieces - treat as Gold
        PieceKind::S_ProPawn
        | PieceKind::S_ProLance
        | PieceKind::S_ProKnight
        | PieceKind::S_ProSilver => &MG_GOLD,
    };

    let lookup_idx = if player == PlayerId::Player1 {
        idx
    } else {
        // Mirror Logic:
        // P1(0) is Top-Left. P1(80) is Bottom-Right.
        // P2(0) should be ... ?
        // If P2 moves DOWN, then Rank 0 is "Home" and Rank 8 is "Enemy Base".
        // Our table: Rank 0 = Enemy Base, Rank 8 = Home.
        // So for P2: Rank 0 (Top) is Home.
        // We need to reverse the rows.
        // Index i -> (i / 9), (i % 9).
        // Row r -> 8 - r.
        // Col c -> 8 - c? (Left/Right symmetry).
        // Yes, 80 - idx does exactly this (mirrors both axes).
        80 - idx
    };

    // Boundary check
    if lookup_idx >= table.len() {
        return 0;
    }

    table[lookup_idx]
}
