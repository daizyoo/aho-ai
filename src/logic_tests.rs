#[cfg(test)]
mod tests {
    use crate::core::{Board, Move, Piece, PieceKind, PlayerId, Position};
    use crate::logic::legal_moves;

    #[test]
    fn test_shogi_forced_promotion() {
        // Setup a board where Shogi Pawn, Lance, Knight are about to move to the last rank
        let mut board = Board::new(9, 9);

        let p1_pawn = Piece::new(PieceKind::S_Pawn, PlayerId::Player1);
        let p1_lance = Piece::new(PieceKind::S_Lance, PlayerId::Player1);
        let p1_knight = Piece::new(PieceKind::S_Knight, PlayerId::Player1);

        board.place_piece(Position::new(1, 1), p1_pawn);
        board.place_piece(Position::new(2, 1), p1_lance);
        board.place_piece(Position::new(3, 2), p1_knight); // Knight at y=2 jumps to y=0

        // Get moves
        let moves = legal_moves(&board, PlayerId::Player1);

        // Check Pawn moves (from 1,1 to 1,0)
        let pawn_moves: Vec<&Move> = moves
            .iter()
            .filter(|m| {
                if let Move::Normal { from, .. } = m {
                    *from == Position::new(1, 1)
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(pawn_moves.len(), 1);
        if let Move::Normal { promote, .. } = pawn_moves[0] {
            assert!(promote.is_some()); // Must promote
        } else {
            panic!("Not normal move");
        }

        // Check Lance moves (from 2,1 to 2,0)
        let lance_moves: Vec<&Move> = moves
            .iter()
            .filter(|m| {
                if let Move::Normal { from, to, .. } = m {
                    *from == Position::new(2, 1) && *to == Position::new(2, 0)
                } else {
                    false
                }
            })
            .collect();
        assert_eq!(lance_moves.len(), 1);
        if let Move::Normal { promote, .. } = lance_moves[0] {
            assert!(promote.is_some());
        }

        // Check Knight moves (from 3,2 to 2,0 or 4,0)
        let knight_moves: Vec<&Move> = moves
            .iter()
            .filter(|m| {
                if let Move::Normal { from, .. } = m {
                    *from == Position::new(3, 2)
                } else {
                    false
                }
            })
            .collect();
        // Knight can jump to two spots if clear? 3,2 -> 2,0 and 4,0
        // Both are y=0, so MUST promote.
        for m in knight_moves {
            if let Move::Normal { promote, .. } = m {
                assert!(promote.is_some(), "Knight must promote at y=0");
            }
        }
    }

    #[test]
    fn test_chess_pawn_promotion() {
        let mut board = Board::new(9, 9);
        let c_pawn = Piece::new(PieceKind::C_Pawn, PlayerId::Player1);
        // Chess pawn moves forward direction. P1 forward is -1?
        // P1 Chess Pawn starts at y=? In mixed game?
        // Logic says: P1 Promo Y is 0.
        board.place_piece(Position::new(5, 1), c_pawn);

        let moves = legal_moves(&board, PlayerId::Player1);
        let pawn_moves: Vec<&Move> = moves
            .iter()
            .filter(|m| {
                if let Move::Normal { from, .. } = m {
                    *from == Position::new(5, 1)
                } else {
                    false
                }
            })
            .collect();

        // Should have 4 moves (Queen, Rook, Bishop, Knight)
        // And NO non-promote move
        assert_eq!(pawn_moves.len(), 4, "Should have 4 promotion options");

        for m in pawn_moves {
            if let Move::Normal { promote, .. } = m {
                assert!(promote.is_some(), "Chess pawn at end must promote");
                let k = promote.unwrap();
                assert!(matches!(
                    k,
                    PieceKind::C_Queen
                        | PieceKind::C_Rook
                        | PieceKind::C_Bishop
                        | PieceKind::C_Knight
                ));
            }
        }
    }
}
