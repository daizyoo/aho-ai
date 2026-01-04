use crate::core::{Board, PieceKind, PlayerId};

pub struct Evaluator;

impl Evaluator {
    // 評価値の定数
    const VAL_KING: i32 = 100000;
    const VAL_QUEEN: i32 = 1000;
    const VAL_ROOK: i32 = 900;
    const VAL_BISHOP: i32 = 800;

    const VAL_GOLD: i32 = 600;
    const VAL_SILVER: i32 = 500;
    const VAL_KNIGHT: i32 = 400;
    const VAL_LANCE: i32 = 300;
    const VAL_PAWN: i32 = 100;

    // 持ち駒の係数（1.1倍などは計算コストがかかるので、整数加算で調整）
    const HAND_BONUS: i32 = 10;

    pub fn evaluate(board: &Board, player: PlayerId) -> i32 {
        let mut score = 0;
        let _opponent = player.opponent();

        // 1. 盤上の駒 (Material) & King Safety (簡易的)
        for (&pos, piece) in board.pieces.iter() {
            let mut val = Self::get_piece_value(piece.kind);

            // 玉の安全性ボーナス/ペナルティ
            // 玉の周りに自分の駒があればプラス、なければマイナス（簡易）
            if matches!(piece.kind, PieceKind::S_King | PieceKind::C_King) {
                // 王の座標に基づく評価（端にいるほうが安全な場合が多いが、将棋とチェスで違うので中央回避のみ評価）
                if pos.x == 4 && (pos.y == 4 || pos.y == 3 || pos.y == 5) {
                    // 中央付近は危険
                    val -= 50;
                }
            }

            if piece.owner == player {
                score += val;
            } else {
                score -= val;
            }
        }

        // 2. 持ち駒 (Hand)
        for (&p_id, hand) in board.hand.iter() {
            for (&kind, &count) in hand.iter() {
                if count > 0 {
                    let val = Self::get_piece_value(kind) + Self::HAND_BONUS;
                    if p_id == player {
                        score += val * count as i32;
                    } else {
                        score -= val * count as i32;
                    }
                }
            }
        }

        // 3. 機動力 (Mobility)
        // 重い処理なので、Quiescence Searchなどでは省略することもあるが、
        // ここでは PseudoLegalMoves の数などを加味すると強くなる。
        // ただし毎ノード計算は遅いので、評価関数としては Material+Hand+Positioning を主とする。
        // ここでは全合法手生成は重すぎるためスキップ。

        score
    }

    fn get_piece_value(kind: PieceKind) -> i32 {
        match kind {
            PieceKind::S_King | PieceKind::C_King => Self::VAL_KING,
            PieceKind::C_Queen => Self::VAL_QUEEN,
            PieceKind::S_Rook | PieceKind::C_Rook => Self::VAL_ROOK,
            PieceKind::S_Bishop | PieceKind::C_Bishop => Self::VAL_BISHOP,
            PieceKind::S_ProRook | PieceKind::S_ProBishop => 1100, // 成った大駒

            PieceKind::S_Gold
            | PieceKind::S_ProSilver
            | PieceKind::S_ProKnight
            | PieceKind::S_ProLance
            | PieceKind::S_ProPawn => Self::VAL_GOLD,

            PieceKind::S_Silver | PieceKind::C_Knight => Self::VAL_SILVER,
            PieceKind::S_Knight => Self::VAL_KNIGHT,
            PieceKind::S_Lance => Self::VAL_LANCE,
            PieceKind::S_Pawn | PieceKind::C_Pawn => Self::VAL_PAWN,
        }
    }
}
