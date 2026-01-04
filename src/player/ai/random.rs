use crate::core::{Board, Move, PlayerId};
use crate::player::PlayerController;
use rand::seq::SliceRandom;

pub struct RandomAI {
    pub name: String,
}

impl RandomAI {
    pub fn new(_player_id: PlayerId, name: &str) -> Self {
        RandomAI {
            name: name.to_string(),
        }
    }
}

impl PlayerController for RandomAI {
    fn name(&self) -> &str {
        &self.name
    }

    fn choose_move(&self, _board: &Board, legal_moves: &[Move]) -> Option<Move> {
        let mut rng = rand::thread_rng();
        legal_moves.choose(&mut rng).cloned()
    }

    fn is_local(&self) -> bool {
        true
    }
}
