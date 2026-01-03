use super::piece::{Piece, PieceKind};
use super::types::{PlayerConfig, PlayerId, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 盤面
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    /// 駒の位置
    #[serde(with = "crate::core::serialization")]
    pub pieces: HashMap<Position, Piece>,
    /// 持ち駒
    #[serde(with = "crate::core::serialization")]
    pub hand: HashMap<PlayerId, HashMap<PieceKind, usize>>,
    /// プレイヤー設定
    #[serde(with = "crate::core::serialization")]
    pub player_configs: HashMap<PlayerId, PlayerConfig>,
    pub last_move: Option<crate::core::Move>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Board {
            width,
            height,
            pieces: HashMap::new(),
            hand: HashMap::new(),
            player_configs: HashMap::new(),
            last_move: None,
        }
    }

    pub fn place_piece(&mut self, pos: Position, piece: Piece) {
        self.pieces.insert(pos, piece);
    }

    pub fn get_piece(&self, pos: Position) -> Option<&Piece> {
        self.pieces.get(&pos)
    }

    pub fn remove_piece(&mut self, pos: Position) -> Option<Piece> {
        self.pieces.remove(&pos)
    }

    pub fn add_to_hand(&mut self, player: PlayerId, kind: PieceKind) {
        let hand = self.hand.entry(player).or_insert_with(HashMap::new);
        *hand.entry(kind).or_insert(0) += 1;
    }

    pub fn remove_from_hand(&mut self, player: PlayerId, kind: PieceKind) -> bool {
        if let Some(hand) = self.hand.get_mut(&player) {
            if let Some(count) = hand.get_mut(&kind) {
                if *count > 0 {
                    *count -= 1;
                    if *count == 0 {
                        hand.remove(&kind);
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn set_player_config(&mut self, player: PlayerId, config: PlayerConfig) {
        self.player_configs.insert(player, config);
    }

    pub fn get_player_config(&self, player: PlayerId) -> PlayerConfig {
        self.player_configs
            .get(&player)
            .cloned()
            .unwrap_or_default()
    }

    pub fn find_king(&self, player: PlayerId) -> Option<Position> {
        self.pieces
            .iter()
            .find(|(_, p)| {
                p.owner == player && matches!(p.kind, PieceKind::S_King | PieceKind::C_King)
            })
            .map(|(pos, _)| *pos)
    }
}
