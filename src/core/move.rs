use super::piece::PieceKind;
use super::types::Position;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Move {
    Normal {
        from: Position,
        to: Position,
        promote: Option<PieceKind>,
    },
    Drop {
        kind: PieceKind,
        to: Position,
    },
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Move::Normal { from, to, promote } => {
                if let Some(p) = promote {
                    write!(f, "{} -> {} (promote to {:?})", from, to, p)
                } else {
                    write!(f, "{} -> {}", from, to)
                }
            }
            Move::Drop { kind, to } => {
                write!(f, "Drop {:?} at {}", kind, to)
            }
        }
    }
}
