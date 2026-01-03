use crate::core::{Board, Move, PlayerId};
use crate::logic::evaluate;
use crate::player::PlayerController;
use rand::prelude::*;
use std::f64;

pub struct WeightedRandomAI {
    pub player_id: PlayerId,
    pub name: String,
    pub temperature: f64,
}

impl WeightedRandomAI {
    pub fn new(player_id: PlayerId, name: &str) -> Self {
        Self {
            player_id,
            name: name.to_string(),
            temperature: 1.0,
        }
    }

    /// Softmax-like probability distribution from scores
    fn get_probabilities(&self, board: &Board, moves: &[Move]) -> Vec<f64> {
        let scores: Vec<f64> = moves
            .iter()
            .map(|mv| {
                let next_board = crate::logic::apply_move(board, mv, self.player_id);
                evaluate(&next_board, self.player_id) as f64
            })
            .collect();

        if scores.is_empty() {
            return vec![];
        }

        let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let exps: Vec<f64> = scores
            .iter()
            .map(|&s| ((s - max_score) / self.temperature).exp())
            .collect();
        let sum_exp: f64 = exps.iter().sum();

        exps.iter().map(|&e| e / sum_exp).collect()
    }
}

impl PlayerController for WeightedRandomAI {
    fn choose_move(&self, board: &Board, moves: &[Move]) -> Option<Move> {
        if moves.is_empty() {
            return None;
        }

        let probs = self.get_probabilities(board, moves);
        let mut rng = thread_rng();

        // Weighted selection
        let mut r = rng.gen::<f64>();
        for (i, &p) in probs.iter().enumerate() {
            if r < p {
                return Some(moves[i].clone());
            }
            r -= p;
        }

        Some(moves.last().unwrap().clone())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
