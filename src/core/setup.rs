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

            let kind = parse_piece_kind(kind_str);
            if let Some(k) = kind {
                board.place_piece(Position::new(x, y), Piece::new(k, owner));
            }
        }
    }
    board
}

fn parse_piece_kind(s: &str) -> Option<PieceKind> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        // 将棋系 (デフォルト)
        "k" => {
            if s.chars().next().unwrap().is_uppercase() {
                Some(PieceKind::S_King)
            } else {
                Some(PieceKind::S_King)
            }
        }
        "r" => Some(PieceKind::S_Rook),
        "b" => Some(PieceKind::S_Bishop),
        "g" => Some(PieceKind::S_Gold),
        "s" => Some(PieceKind::S_Silver),
        "n" => Some(PieceKind::S_Knight),
        "l" => Some(PieceKind::S_Lance),
        "p" => Some(PieceKind::S_Pawn),
        // チェス系 (明示的または種類で判別)
        "ck" => Some(PieceKind::C_King),
        "cq" => Some(PieceKind::C_Queen),
        "cr" => Some(PieceKind::C_Rook),
        "cb" => Some(PieceKind::C_Bishop),
        "cn" => Some(PieceKind::C_Knight),
        "cp" => Some(PieceKind::C_Pawn),
        _ => {
            // 将棋駒の1文字パース
            match s.chars().next().unwrap().to_ascii_uppercase() {
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
