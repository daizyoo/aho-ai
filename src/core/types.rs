use serde::{Deserialize, Serialize};
use std::fmt;

/// プレイヤーID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerId {
    Player1, // 先手 (通常)
    Player2, // 後手 (通常)
}

impl Default for PlayerId {
    fn default() -> Self {
        PlayerId::Player1
    }
}

impl PlayerId {
    pub fn opponent(self) -> PlayerId {
        match self {
            PlayerId::Player1 => PlayerId::Player2,
            PlayerId::Player2 => PlayerId::Player1,
        }
    }
}

/// プレイヤーの能力設定
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub can_capture: bool,   // 駒を取れるか
    pub can_promote: bool,   // 成れるか
    pub can_drop: bool,      // 持ち駒を打てるか
    pub keep_captured: bool, // 取った駒を持ち駒にするか (将棋 = true, チェス = false)
}

impl Default for PlayerConfig {
    fn default() -> Self {
        PlayerConfig {
            can_capture: true,
            can_promote: true,
            can_drop: true,
            keep_captured: true,
        }
    }
}

impl PlayerConfig {
    pub fn shogi() -> Self {
        Self::default()
    }

    pub fn chess() -> Self {
        PlayerConfig {
            can_capture: true,
            can_promote: false, // チェスのPawnプロモーションは駒個別の能力として扱う方が汎用的
            can_drop: false,
            keep_captured: false,
        }
    }
}

/// 盤面座標 (0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Position { x, y }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
