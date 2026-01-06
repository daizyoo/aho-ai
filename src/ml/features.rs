//! Board Feature Extraction for Neural Network Input
//!
//! Converts the game board state into a fixed-size feature vector suitable for ML models.

use crate::core::{Board, PieceKind, PlayerId, Position};

/// Number of piece types (including empty squares)
/// 1 (empty) + 20 (own pieces) + 20 (opponent pieces) = 41
const NUM_PIECE_TYPES: usize = 41;

/// Feature extractor for board states
pub struct BoardFeatureExtractor;

impl BoardFeatureExtractor {
    /// Extract features from a board state
    ///
    /// Returns a flattened feature vector representing:
    /// - Piece positions (9x9 grid, one-hot encoded per piece type)
    /// - Hand pieces for both players
    /// - Side to move
    pub fn extract(board: &Board, current_player: PlayerId) -> Vec<f32> {
        let mut features = Vec::new();

        // 1. Board state (9x9x41 = 3321 features)
        // Each square has one-hot encoding for piece type
        for y in 0..board.height {
            for x in 0..board.width {
                let pos = Position::new(x, y);
                let piece_features = Self::encode_square(board, pos, current_player);
                features.extend(piece_features);
            }
        }

        // 2. Hand pieces (26 piece types x 2 players x max_count)
        // For simplicity, we use count normalized by max expected (e.g., max 18 pieces in hand)
        let hand_features = Self::encode_hand(board, current_player);
        features.extend(hand_features);

        // 3. Side to move (1 feature: 1.0 if current_player, 0.0 otherwise)
        features.push(1.0);

        features
    }

    /// Encode a single square as one-hot vector
    fn encode_square(board: &Board, pos: Position, perspective: PlayerId) -> Vec<f32> {
        let mut encoding = vec![0.0; NUM_PIECE_TYPES];

        if let Some(piece) = board.get_piece(pos) {
            let idx = Self::piece_to_index(piece.kind, piece.owner, perspective);
            encoding[idx] = 1.0;
        } else {
            encoding[0] = 1.0; // Empty square
        }

        encoding
    }

    /// Encode hand pieces as normalized counts
    fn encode_hand(board: &Board, perspective: PlayerId) -> Vec<f32> {
        let mut hand_features = Vec::new();
        const MAX_HAND_COUNT: f32 = 18.0;

        for &player in &[PlayerId::Player1, PlayerId::Player2] {
            if let Some(hand) = board.hand.get(&player) {
                for &kind in &[
                    PieceKind::S_Pawn,
                    PieceKind::S_Lance,
                    PieceKind::S_Knight,
                    PieceKind::S_Silver,
                    PieceKind::S_Gold,
                    PieceKind::S_Bishop,
                    PieceKind::S_Rook,
                    PieceKind::C_Pawn,
                    PieceKind::C_Knight,
                    PieceKind::C_Bishop,
                    PieceKind::C_Rook,
                ] {
                    let count = hand.get(&kind).copied().unwrap_or(0) as f32;
                    let normalized = (count / MAX_HAND_COUNT).min(1.0);

                    // Flip perspective if needed
                    let value = if (player == perspective) {
                        normalized
                    } else {
                        -normalized
                    };
                    hand_features.push(value);
                }
            } else {
                // No hand for this player, push zeros
                hand_features.extend(vec![0.0; 11]);
            }
        }

        hand_features
    }

    /// Map piece kind and owner to feature index
    fn piece_to_index(kind: PieceKind, owner: PlayerId, perspective: PlayerId) -> usize {
        let base_idx = match kind {
            PieceKind::S_Pawn => 1,
            PieceKind::S_Lance => 2,
            PieceKind::S_Knight => 3,
            PieceKind::S_Silver => 4,
            PieceKind::S_Gold => 5,
            PieceKind::S_Bishop => 6,
            PieceKind::S_Rook => 7,
            PieceKind::S_King => 8,
            PieceKind::S_ProPawn => 9,
            PieceKind::S_ProLance => 10,
            PieceKind::S_ProKnight => 11,
            PieceKind::S_ProSilver => 12,
            PieceKind::S_ProBishop => 13,
            PieceKind::S_ProRook => 14,
            PieceKind::C_Pawn => 15,
            PieceKind::C_Knight => 16,
            PieceKind::C_Bishop => 17,
            PieceKind::C_Rook => 18,
            PieceKind::C_Queen => 19,
            PieceKind::C_King => 20,
        };

        // Offset by 20 if piece belongs to opponent (from perspective)
        if owner == perspective {
            base_idx
        } else {
            base_idx + 20
        }
    }

    /// Get the expected feature vector size
    pub fn feature_size() -> usize {
        // Board: 9*9*41 + Hand: 2*11 + Turn: 1
        9 * 9 * NUM_PIECE_TYPES + 2 * 11 + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_size() {
        let expected = 9 * 9 * 41 + 22 + 1;
        assert_eq!(BoardFeatureExtractor::feature_size(), expected);
    }

    #[test]
    fn test_extract_empty_board() {
        let board = Board::new(9, 9);
        let features = BoardFeatureExtractor::extract(&board, PlayerId::Player1);
        assert_eq!(features.len(), BoardFeatureExtractor::feature_size());
    }
}
