use super::piece::{Piece, PieceKind};
use super::types::{PlayerConfig, PlayerId, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the game board and state.
///
/// This struct holds all information necessary to describe the current position, including:
/// - Placement of pieces on the 2D grid.
/// - Captured pieces (hand) for each player.
/// - Configurations for player directions/regions (PlayerConfig).
/// - History and hashing for repetition detection (Zobrist/Sennichite).
///
/// The board dimensions are flexible (defined by `width` and `height`),
/// typically 9x9 for Shogi or custom for mixed variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// Width of the board (e.g., 9 for Shogi)
    pub width: usize,
    /// Height of the board (e.g., 9 for Shogi)
    pub height: usize,

    /// Active pieces on the board, keyed by their coordinate.
    /// sparse representation (only occupied squares are stored).
    #[serde(with = "crate::core::serialization")]
    pub pieces: HashMap<Position, Piece>,

    /// Captured pieces held by each player ("Mochigoma").
    /// Map: PlayerId -> (PieceKind -> Count)
    #[serde(
        serialize_with = "crate::core::serialization::serialize_hand",
        deserialize_with = "crate::core::serialization::deserialize_hand"
    )]
    pub hand: HashMap<PlayerId, HashMap<PieceKind, usize>>,

    /// Player-specific configurations (e.g., promotion zones, movement direction).
    #[serde(with = "crate::core::serialization")]
    pub player_configs: HashMap<PlayerId, PlayerConfig>,

    /// The move that led to this state (used for display/highlighting).
    pub last_move: Option<crate::core::Move>,

    /// Zobrist Hash of the current position.
    /// Used for Transposition Table lookups and repetition detection.
    #[serde(skip)]
    pub zobrist_hash: u64,

    /// History of Zobrist hashes leading to this state.
    /// Essential for detecting 4-fold repetition (Sennichite).
    #[serde(skip)]
    pub history: Vec<u64>,
}

impl Board {
    /// Creates a new empty board with specified dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Board {
            width,
            height,
            pieces: HashMap::new(),
            hand: HashMap::new(),
            player_configs: HashMap::new(),
            last_move: None,
            zobrist_hash: 0,
            history: Vec::new(),
        }
    }

    /// Places a piece at the specified position.
    /// Overwrites any existing piece at that location.
    pub fn place_piece(&mut self, pos: Position, piece: Piece) {
        self.pieces.insert(pos, piece);
    }

    /// Returns a reference to the piece at the specified position, if any.
    pub fn get_piece(&self, pos: Position) -> Option<&Piece> {
        self.pieces.get(&pos)
    }

    /// Removes and returns the piece at the specified position, if any.
    pub fn remove_piece(&mut self, pos: Position) -> Option<Piece> {
        self.pieces.remove(&pos)
    }

    /// Adds a captured piece to the player's hand.
    pub fn add_to_hand(&mut self, player: PlayerId, kind: PieceKind) {
        let hand = self.hand.entry(player).or_default();
        *hand.entry(kind).or_insert(0) += 1;
    }

    /// Attempts to remove a piece from the player's hand (e.g., for a drop move).
    /// Returns `true` if successful, `false` if the player didn't have that piece.
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

    /// Sets the configuration (direction, promotion zone) for a player.
    pub fn set_player_config(&mut self, player: PlayerId, config: PlayerConfig) {
        self.player_configs.insert(player, config);
    }

    /// retrieves the configuration for a player. Returns default if not set.
    pub fn get_player_config(&self, player: PlayerId) -> PlayerConfig {
        self.player_configs
            .get(&player)
            .cloned()
            .unwrap_or_default()
    }

    /// Locates the position of the King (or "King-like" piece) for a given player.
    /// Used for check/mate detection.
    pub fn find_king(&self, player: PlayerId) -> Option<Position> {
        self.pieces
            .iter()
            .find(|(_, p)| {
                p.owner == player && matches!(p.kind, PieceKind::S_King | PieceKind::C_King)
            })
            .map(|(pos, _)| *pos)
    }
}
