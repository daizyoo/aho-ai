use crate::core::PlayerId;
use crate::game::{Game, KifuData, PerspectiveMode, ThinkingInfo};
use crate::player::ai::{AIStrength, AlphaBetaAI};
use crate::player::PlayerController;
use crossterm::{execute, terminal};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
