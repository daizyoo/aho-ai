pub mod alpha_beta;
pub mod config;
pub mod eval;
pub mod minimax;
pub mod pst;
pub mod tt;
pub mod weighted;

pub use alpha_beta::{AIStrength, AlphaBetaAI};
pub use minimax::MinimaxAI;
pub use weighted::WeightedRandomAI;
