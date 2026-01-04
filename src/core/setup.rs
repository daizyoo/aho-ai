use crate::core::{Board, Piece, PieceKind, PlayerConfig, PlayerId, Position};

/// 文字列配列から盤面とプレイヤー設定を初期化する
pub fn setup_from_strings(setup: &[&str], p1_shogi: bool, p2_shogi: bool) -> Board {
    let height = setup.len();
    let width = if height > 0 {
        setup[0].split_whitespace().count()
    } else {
        0
    };
    let mut board = Board::new(width, height);

    board.set_player_config(
        PlayerId::Player1,
        if p1_shogi {
            PlayerConfig::shogi()
        } else {
            PlayerConfig::chess()
        },
    );
    board.set_player_config(
        PlayerId::Player2,
        if p2_shogi {
            PlayerConfig::shogi()
        } else {
            PlayerConfig::chess()
        },
    );

    for (y, row) in setup.iter().enumerate() {
        for (x, s) in row.split_whitespace().enumerate() {
            if s == "." {
                continue;
            }

            let (kind_str, owner) = if s.chars().next().unwrap().is_uppercase() {
                (s, PlayerId::Player1)
            } else {
                (s, PlayerId::Player2)
            };

            let is_shogi_hint = if owner == PlayerId::Player1 {
                p1_shogi
            } else {
                p2_shogi
            };

            let kind = parse_piece_kind(kind_str, is_shogi_hint);
            if let Some(k) = kind {
                board.place_piece(Position::new(x, y), Piece::new(k, owner));
            }
        }
    }
    board
}

fn parse_piece_kind(s: &str, is_shogi_hint: bool) -> Option<PieceKind> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        // 成り駒などの2文字表記はそのまま
        "cp" => Some(PieceKind::C_Pawn),
        "ck" => Some(PieceKind::C_King),
        "cq" => Some(PieceKind::C_Queen),
        "cr" => Some(PieceKind::C_Rook),
        "cb" => Some(PieceKind::C_Bishop),
        "cn" => Some(PieceKind::C_Knight),
        _ => {
            // 1文字表記のパース
            let ch = s.chars().next().unwrap().to_ascii_uppercase();
            if is_shogi_hint {
                match ch {
                    'K' => Some(PieceKind::S_King),
                    'R' => Some(PieceKind::S_Rook),
                    'B' => Some(PieceKind::S_Bishop),
                    'G' => Some(PieceKind::S_Gold),
                    'S' => Some(PieceKind::S_Silver),
                    'N' => Some(PieceKind::S_Knight),
                    'L' => Some(PieceKind::S_Lance),
                    'P' => Some(PieceKind::S_Pawn),
                    _ => None,
                }
            } else {
                match ch {
                    'K' => Some(PieceKind::C_King),
                    'Q' => Some(PieceKind::C_Queen),
                    'R' => Some(PieceKind::C_Rook),
                    'B' => Some(PieceKind::C_Bishop),
                    'N' => Some(PieceKind::C_Knight),
                    'P' => Some(PieceKind::C_Pawn),
                    _ => None,
                }
            }
        }
    }
}

pub fn get_standard_mixed_setup() -> Vec<&'static str> {
    vec![
        "cr cn cb cq ck cq cb cn cr", // チェス後手 (P2)
        "cp cp cp cp cp cp cp cp cp",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "P P P P P P P P P", // 将棋先手 (P1)
        ". B . . . . . R .",
        "L N S G K G S N L",
    ]
}

pub fn get_reversed_mixed_setup() -> Vec<&'static str> {
    vec![
        "l n s g k g s n l", // 将棋後手 (P2)
        ". r . . . . . b .",
        "p p p p p p p p p",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "CP CP CP CP CP CP CP CP CP", // チェス先手 (P1)
        "CR CN CB CQ CK CQ CB CN CR",
    ]
}

pub fn get_shogi_setup() -> Vec<&'static str> {
    vec![
        "l n s g k g s n l",
        ". r . . . . . b .",
        "p p p p p p p p p",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "P P P P P P P P P",
        ". B . . . . . R .",
        "L N S G K G S N L",
    ]
}

pub fn get_chess_setup() -> Vec<&'static str> {
    vec![
        "cr cn cb cq ck cq cb cn cr",
        "cp cp cp cp cp cp cp cp cp",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "CP CP CP CP CP CP CP CP CP",
        "CR CN CB CQ CK CQ CB CN CR",
    ]
}

pub fn get_fair_setup() -> Vec<&'static str> {
    vec![
        "l n s g ck cq cb cn cr", // P2 starting line: Shogi Left, King Middle, Chess Right
        ". r . . . . . cb .",     // P2: Shogi Rook (Col 2), Chess Bishop (Col 8)
        "p p p p p cp cp cp cp",  // P2 pawns
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "P P P P P CP CP CP CP",  // P1 pawns
        ". R . . . . . CB .",     // P1: Shogi Rook (Col 2), Chess Bishop (Col 8)
        "L N S G CK CQ CB CN CR", // P1 starting line: Shogi Left, King Middle, Chess Right
    ]
}

pub fn get_reversed_fair_setup() -> Vec<&'static str> {
    vec![
        "cr cn cb cq ck g s n l", // P2 starting line: Chess Left, King Middle, Shogi Right
        ". cb . . . . . r .",     // P2: Chess Bishop (Col 2), Shogi Rook (Col 8)
        "cp cp cp cp p p p p p",  // P2 pawns
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "CP CP CP CP P P P P P",  // P1 pawns
        ". CB . . . . . R .",     // P1: Chess Bishop (Col 2), Shogi Rook (Col 8)
        "CR CN CB CQ CK G S N L", // P1 starting line: Chess Left, King Middle, Shogi Right
    ]
}
