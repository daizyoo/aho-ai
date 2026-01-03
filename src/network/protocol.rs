use crate::core::{Board, Move, PlayerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    // Client -> Server
    Join {
        name: String,
    },
    MakeMove {
        mv: Move,
    },
    Resign,

    // Server -> Client
    Welcome {
        player_id: PlayerId,
        board: Board,
    },
    MatchFound {
        opponent_name: String,
    },
    Update {
        board: Board,
        last_move: Option<Move>,
        next_player: PlayerId,
    },
    GameOver {
        winner: PlayerId,
        reason: String,
    },
    Error {
        message: String,
    },
}
