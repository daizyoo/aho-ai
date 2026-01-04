use super::types::PlayerId;
use serde::{Deserialize, Serialize};

/// 駒の種類
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceKind {
    // 将棋駒
    S_King,   // 王/玉
    S_Rook,   // 飛
    S_Bishop, // 角
    S_Gold,   // 金
    S_Silver, // 銀
    S_Knight, // 桂
    S_Lance,  // 香
    S_Pawn,   // 歩
    // 成り駒 (将棋)
    S_ProRook,   // 竜
    S_ProBishop, // 馬
    S_ProSilver, // 成銀
    S_ProKnight, // 成桂
    S_ProLance,  // 成香
    S_ProPawn,   // と金
    // チェス駒
    C_King,
    C_Queen,
    C_Rook,
    C_Bishop,
    C_Knight,
    C_Pawn,
}

impl PieceKind {
    pub fn display_char(&self) -> char {
        match self {
            PieceKind::S_King => '王',
            PieceKind::S_Rook => '飛',
            PieceKind::S_Bishop => '角',
            PieceKind::S_Gold => '金',
            PieceKind::S_Silver => '銀',
            PieceKind::S_Knight => '桂',
            PieceKind::S_Lance => '香',
            PieceKind::S_Pawn => '歩',
            PieceKind::S_ProRook => '竜',
            PieceKind::S_ProBishop => '馬',
            PieceKind::S_ProSilver => '全',
            PieceKind::S_ProKnight => '圭',
            PieceKind::S_ProLance => '杏',
            PieceKind::S_ProPawn => 'と',
            PieceKind::C_King => 'K',
            PieceKind::C_Queen => 'Q',
            PieceKind::C_Rook => 'R',
            PieceKind::C_Bishop => 'B',
            PieceKind::C_Knight => 'N',
            PieceKind::C_Pawn => 'P',
        }
    }
}

/// 移動の特性
#[derive(Debug, Clone)]
pub enum MoveStep {
    Step(i32, i32),  // 指定した相対座標へ1マス移動
    Slide(i32, i32), // 指定した方向へ障害物があるまで移動
}

/// 駒の定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Piece {
    pub kind: PieceKind,
    pub owner: PlayerId,
    pub is_shogi: bool, // 将棋系かチェス系か (表示や特殊ルール用)
}

impl Piece {
    pub fn new(kind: PieceKind, owner: PlayerId) -> Self {
        let is_shogi = matches!(
            kind,
            PieceKind::S_King
                | PieceKind::S_Rook
                | PieceKind::S_Bishop
                | PieceKind::S_Gold
                | PieceKind::S_Silver
                | PieceKind::S_Knight
                | PieceKind::S_Lance
                | PieceKind::S_Pawn
                | PieceKind::S_ProRook
                | PieceKind::S_ProBishop
                | PieceKind::S_ProSilver
                | PieceKind::S_ProKnight
                | PieceKind::S_ProLance
                | PieceKind::S_ProPawn
        );
        Piece {
            kind,
            owner,
            is_shogi,
        }
    }

    /// その駒が本来持っている「動きの定義」を返す
    pub fn movement_rules(&self) -> Vec<MoveStep> {
        let forward = if self.owner == PlayerId::Player1 {
            -1
        } else {
            1
        };

        match self.kind {
            PieceKind::S_King | PieceKind::C_King => vec![
                MoveStep::Step(-1, -1),
                MoveStep::Step(0, -1),
                MoveStep::Step(1, -1),
                MoveStep::Step(-1, 0),
                MoveStep::Step(1, 0),
                MoveStep::Step(-1, 1),
                MoveStep::Step(0, 1),
                MoveStep::Step(1, 1),
            ],
            PieceKind::S_Rook | PieceKind::C_Rook => vec![
                MoveStep::Slide(0, -1),
                MoveStep::Slide(0, 1),
                MoveStep::Slide(-1, 0),
                MoveStep::Slide(1, 0),
            ],
            PieceKind::S_Bishop | PieceKind::C_Bishop => vec![
                MoveStep::Slide(-1, -1),
                MoveStep::Slide(-1, 1),
                MoveStep::Slide(1, -1),
                MoveStep::Slide(1, 1),
            ],
            PieceKind::C_Queen => vec![
                MoveStep::Slide(0, -1),
                MoveStep::Slide(0, 1),
                MoveStep::Slide(-1, 0),
                MoveStep::Slide(1, 0),
                MoveStep::Slide(-1, -1),
                MoveStep::Slide(-1, 1),
                MoveStep::Slide(1, -1),
                MoveStep::Slide(1, 1),
            ],
            PieceKind::S_Gold
            | PieceKind::S_ProSilver
            | PieceKind::S_ProKnight
            | PieceKind::S_ProLance
            | PieceKind::S_ProPawn => vec![
                MoveStep::Step(-1, forward),
                MoveStep::Step(0, forward),
                MoveStep::Step(1, forward),
                MoveStep::Step(-1, 0),
                MoveStep::Step(1, 0),
                MoveStep::Step(0, -forward),
            ],
            PieceKind::S_Silver => vec![
                MoveStep::Step(-1, forward),
                MoveStep::Step(0, forward),
                MoveStep::Step(1, forward),
                MoveStep::Step(-1, -forward),
                MoveStep::Step(1, -forward),
            ],
            PieceKind::S_Knight => vec![
                MoveStep::Step(-1, 2 * forward),
                MoveStep::Step(1, 2 * forward),
            ],
            PieceKind::S_Lance => vec![MoveStep::Slide(0, forward)],
            PieceKind::S_Pawn => vec![MoveStep::Step(0, forward)],
            PieceKind::C_Knight => vec![
                MoveStep::Step(-2, -1),
                MoveStep::Step(-2, 1),
                MoveStep::Step(2, -1),
                MoveStep::Step(2, 1),
                MoveStep::Step(-1, -2),
                MoveStep::Step(-1, 2),
                MoveStep::Step(1, -2),
                MoveStep::Step(1, 2),
            ],
            PieceKind::S_ProRook => vec![
                MoveStep::Slide(0, -1),
                MoveStep::Slide(0, 1),
                MoveStep::Slide(-1, 0),
                MoveStep::Slide(1, 0),
                MoveStep::Step(-1, -1),
                MoveStep::Step(-1, 1),
                MoveStep::Step(1, -1),
                MoveStep::Step(1, 1),
            ],
            PieceKind::S_ProBishop => vec![
                MoveStep::Slide(-1, -1),
                MoveStep::Slide(-1, 1),
                MoveStep::Slide(1, -1),
                MoveStep::Slide(1, 1),
                MoveStep::Step(0, -1),
                MoveStep::Step(0, 1),
                MoveStep::Step(-1, 0),
                MoveStep::Step(1, 0),
            ],
            PieceKind::C_Pawn => {
                // チェスのPawnは「移動」と「取り」が違うため、エンジンの合法手生成側で特殊処理する
                vec![]
            }
        }
    }

    pub fn display_char(&self) -> char {
        match self.kind {
            PieceKind::S_King => '王',
            PieceKind::S_Rook => '飛',
            PieceKind::S_Bishop => '角',
            PieceKind::S_Gold => '金',
            PieceKind::S_Silver => '銀',
            PieceKind::S_Knight => '桂',
            PieceKind::S_Lance => '香',
            PieceKind::S_Pawn => '歩',
            PieceKind::S_ProRook => '竜',
            PieceKind::S_ProBishop => '馬',
            PieceKind::S_ProSilver => '全',
            PieceKind::S_ProKnight => '圭',
            PieceKind::S_ProLance => '杏',
            PieceKind::S_ProPawn => 'と',
            PieceKind::C_King => 'K',
            PieceKind::C_Queen => 'Q',
            PieceKind::C_Rook => 'R',
            PieceKind::C_Bishop => 'B',
            PieceKind::C_Knight => 'N',
            PieceKind::C_Pawn => 'P',
        }
    }

    pub fn promotable_kind(&self) -> Option<PieceKind> {
        match self.kind {
            PieceKind::S_Rook => Some(PieceKind::S_ProRook),
            PieceKind::S_Bishop => Some(PieceKind::S_ProBishop),
            PieceKind::S_Silver => Some(PieceKind::S_ProSilver),
            PieceKind::S_Knight => Some(PieceKind::S_ProKnight),
            PieceKind::S_Lance => Some(PieceKind::S_ProLance),
            PieceKind::S_Pawn => Some(PieceKind::S_ProPawn),
            PieceKind::C_Pawn => Some(PieceKind::C_Queen), // チェスはQueen固定（簡略化）
            _ => None,
        }
    }

    pub fn unpromoted_kind(&self) -> PieceKind {
        match self.kind {
            PieceKind::S_ProRook => PieceKind::S_Rook,
            PieceKind::S_ProBishop => PieceKind::S_Bishop,
            PieceKind::S_ProSilver => PieceKind::S_Silver,
            PieceKind::S_ProKnight => PieceKind::S_Knight,
            PieceKind::S_ProLance => PieceKind::S_Lance,
            PieceKind::S_ProPawn => PieceKind::S_Pawn,
            _ => self.kind,
        }
    }
}
