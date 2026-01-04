use crate::core::{Board, Move, PlayerId};
use crate::player::PlayerController;
use std::sync::mpsc;

pub struct NetworkController {
    player_id: PlayerId,
    name: String,
    // Channel to receive moves from the network thread
    rx: mpsc::Receiver<Move>,
}

impl NetworkController {
    pub fn new(player_id: PlayerId, name: &str, rx: mpsc::Receiver<Move>) -> Self {
        Self {
            player_id,
            name: name.to_string(),
            rx,
        }
    }
}

impl PlayerController for NetworkController {
    fn name(&self) -> &str {
        &self.name
    }

    fn choose_move(&self, _board: &Board, _moves: &[Move]) -> Option<Move> {
        // Block until a move is received from the server
        self.rx.recv().ok()
    }

    fn is_local(&self) -> bool {
        false
    }
}
