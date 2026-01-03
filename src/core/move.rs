use super::piece::PieceKind;
use super::types::Position;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    Normal {
        from: Position,
        to: Position,
        promote: bool,
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
                if *promote {
                    write!(f, "{} -> {} (promote)", from, to)
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
