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
        "l n s g k cq cb cn cr", // P2 starting line: Shogi Left (0-3), King (4), Chess Right (5-8)
        ". r . . . . . cb .",    // P2: Shogi Rook (1), Chess Bishop (7)
        "p p p p p cp cp cp cp", // P2 pawns: Shogi (0-4), Chess (5-8)
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "P P P P P CP CP CP CP", // P1 pawns: Shogi (0-4), Chess (5-8)
        ". B . . . . . CR .",    // P1: Shogi Bishop (1), Chess Rook (7)
        "L N S G K CR CN CB CQ", // P1 starting line: Shogi Left (0-3), King (4), Chess Right (5-8)
    ]
}

pub fn get_reversed_fair_setup() -> Vec<&'static str> {
    vec![
        "cr cn cb cq ck g s n l", // P2 starting line: Chess Left (0-3), King (4), Shogi Right (5-8)
        ". cr . . . . . b .",     // P2: Chess Rook (1), Shogi Bishop (7)
        "cp cp cp cp p p p p p",  // P2 pawns: Chess (0-3), Shogi (4-8)
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "CP CP CP CP P P P P P", // P1 pawns: Chess (0-3), Shogi (4-8)
        ". CB . . . . . R .",    // P1: Chess Bishop (1), Shogi Rook (7)
        "CR CN CB CQ K G S N L", // P1 starting line: Chess Left (0-3), King (4), Shogi Right (5-8)
    ]
}
