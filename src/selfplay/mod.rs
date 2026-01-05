use crate::core::PlayerId;
use crate::game::{Game, KifuData, PerspectiveMode};
use crate::player::ai::{AIStrength, AlphaBetaAI};
use crate::player::PlayerController;
use crossterm::{execute, terminal};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum BoardSetupType {
    StandardMixed, // Shogi P1 vs Chess P2
    ReversedMixed, // Chess P1 vs Shogi P2
    ShogiOnly,     // Shogi vs Shogi
    ChessOnly,     // Chess vs Chess
    Fair,          // Symmetric Mixed
    ReversedFair,  // Reversed Symmetric Mixed
}

pub struct SelfPlayConfig {
    pub num_games: usize,
    pub ai1_strength: AIStrength,
    pub ai2_strength: AIStrength,
    pub board_setup: BoardSetupType,
    pub save_kifus: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub winner: Option<PlayerId>,
    pub moves: usize,
    pub time_ms: u128,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelfPlayStats {
    pub total_games: usize,
    pub p1_wins: usize,
    pub p2_wins: usize,
    pub draws: usize,
    pub avg_moves: f64,
    pub avg_time_ms: f64,
    pub ai1_strength: String,
    pub ai2_strength: String,
    pub board_setup: String,
    pub games: Vec<GameResult>,
}

impl SelfPlayStats {
    pub fn new() -> Self {
        Self {
            total_games: 0,
            p1_wins: 0,
            p2_wins: 0,
            draws: 0,
            avg_moves: 0.0,
            avg_time_ms: 0.0,
            ai1_strength: String::new(),
            ai2_strength: String::new(),
            board_setup: String::new(),
            games: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: GameResult) {
        self.total_games += 1;
        match result.winner {
            Some(PlayerId::Player1) => self.p1_wins += 1,
            Some(PlayerId::Player2) => self.p2_wins += 1,
            None => self.draws += 1,
        }
        self.games.push(result);
        self.recalculate_averages();
    }

    fn recalculate_averages(&mut self) {
        if self.games.is_empty() {
            return;
        }
        let total_moves: usize = self.games.iter().map(|g| g.moves).sum();
        let total_time: u128 = self.games.iter().map(|g| g.time_ms).sum();
        self.avg_moves = total_moves as f64 / self.games.len() as f64;
        self.avg_time_ms = total_time as f64 / self.games.len() as f64;
    }
}

pub fn run_selfplay(config: SelfPlayConfig) -> anyhow::Result<SelfPlayStats> {
    let mut stats = SelfPlayStats::new();
    
    // Store AI configuration
    stats.ai1_strength = format!("{:?}", config.ai1_strength);
    stats.ai2_strength = format!("{:?}", config.ai2_strength);
    stats.board_setup = match config.board_setup {
        BoardSetupType::StandardMixed => "StandardMixed",
        BoardSetupType::ReversedMixed => "ReversedMixed",
        BoardSetupType::ShogiOnly => "ShogiOnly",
        BoardSetupType::ChessOnly => "ChessOnly",
        BoardSetupType::Fair => "Fair",
        BoardSetupType::ReversedFair => "ReversedFair",
    }.to_string();

    for game_num in 1..=config.num_games {
        let start_time = Instant::now();

        // Create board
        let board = match config.board_setup {
            BoardSetupType::StandardMixed => {
                let map = crate::core::setup::get_standard_mixed_setup();
                crate::core::setup::setup_from_strings(&map, true, false)
            }
            BoardSetupType::ReversedMixed => {
                let map = crate::core::setup::get_reversed_mixed_setup();
                crate::core::setup::setup_from_strings(&map, false, true)
            }
            BoardSetupType::ShogiOnly => {
                let map = crate::core::setup::get_shogi_setup();
                crate::core::setup::setup_from_strings(&map, true, true)
            }
            BoardSetupType::ChessOnly => {
                let map = crate::core::setup::get_chess_setup();
                crate::core::setup::setup_from_strings(&map, false, false)
            }
            BoardSetupType::Fair => {
                let map = crate::core::setup::get_fair_setup();
                crate::core::setup::setup_from_strings(&map, true, true)
            }
            BoardSetupType::ReversedFair => {
                let map = crate::core::setup::get_reversed_fair_setup();
                crate::core::setup::setup_from_strings(&map, false, false)
            }
        };

        // Create AI players
        let p1: Box<dyn PlayerController> = Box::new(AlphaBetaAI::new(
            PlayerId::Player1,
            "AI-P1",
            config.ai1_strength,
        ));
        let p2: Box<dyn PlayerController> = Box::new(AlphaBetaAI::new(
            PlayerId::Player2,
            "AI-P2",
            config.ai2_strength,
        ));

        // Run game silently
        let mut game = Game::new(board);
        game.perspective_mode = PerspectiveMode::Fixed(PlayerId::Player1);

        let (winner, move_count) = run_game_silent(&mut game, p1.as_ref(), p2.as_ref())?;

        let elapsed = start_time.elapsed();
        let result = GameResult {
            winner,
            moves: move_count,
            time_ms: elapsed.as_millis(),
        };
        stats.add_result(result);

        // Display detailed progress
        execute!(
            std::io::stdout(),
            terminal::Clear(terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;

        print!("=== Self-Play Progress ===\r\n\r\n");
        print!("Game {}/{} completed\r\n", game_num, config.num_games);
        print!(
            "Result: {} ({} moves, {:.1}s)\r\n\r\n",
            match winner {
                Some(PlayerId::Player1) => "P1 wins",
                Some(PlayerId::Player2) => "P2 wins",
                None => "Draw",
            },
            move_count,
            elapsed.as_secs_f64()
        );

        print!("--- Current Statistics ---\r\n");
        print!(
            "P1 Wins: {} ({:.1}%)\r\n",
            stats.p1_wins,
            stats.p1_wins as f64 / stats.total_games as f64 * 100.0
        );
        print!(
            "P2 Wins: {} ({:.1}%)\r\n",
            stats.p2_wins,
            stats.p2_wins as f64 / stats.total_games as f64 * 100.0
        );
        print!(
            "Draws: {} ({:.1}%)\r\n",
            stats.draws,
            stats.draws as f64 / stats.total_games as f64 * 100.0
        );
        print!("Avg Moves: {:.1}\r\n", stats.avg_moves);
        print!("Avg Time: {:.1}s\r\n\r\n", stats.avg_time_ms / 1000.0);

        std::io::Write::flush(&mut std::io::stdout())?;

        // Save kifu if requested
        if config.save_kifus {
            save_kifu(&game, game_num, &stats.board_setup)?;
        }
    }

    println!(); // New line after progress
    Ok(stats)
}

fn run_game_silent(
    game: &mut Game,
    p1: &dyn PlayerController,
    p2: &dyn PlayerController,
) -> anyhow::Result<(Option<PlayerId>, usize)> {
    let mut move_count = 0;
    let max_moves = 500; // Prevent infinite games

    loop {
        if move_count >= max_moves {
            return Ok((None, move_count)); // Draw by move limit
        }

        let current_player = game.current_player;
        let controller = match current_player {
            PlayerId::Player1 => p1,
            PlayerId::Player2 => p2,
        };

        let legal_moves = crate::logic::legal_moves(&game.board, current_player);

        if legal_moves.is_empty() {
            // Checkmate or stalemate
            let in_check = crate::logic::is_in_check(&game.board, current_player);
            if in_check {
                return Ok((Some(current_player.opponent()), move_count));
            } else {
                return Ok((None, move_count)); // Stalemate
            }
        }

        // Display current move being calculated
        print!(
            "\rMove {}: {:?} thinking...",
            move_count + 1,
            current_player
        );
        std::io::Write::flush(&mut std::io::stdout())?;

        if let Some(chosen_move) = controller.choose_move(&game.board, &legal_moves) {
            game.board = crate::logic::apply_move(&game.board, &chosen_move, current_player);
            game.history.push(chosen_move);
            game.current_player = current_player.opponent();
            move_count += 1;
        } else {
            return Ok((Some(current_player.opponent()), move_count)); // Resignation
        }
    }
}

fn save_kifu(game: &Game, game_num: usize, board_setup: &str) -> anyhow::Result<()> {
    let kifu_dir = "selfplay_kifu";
    std::fs::create_dir_all(kifu_dir)?;

    let filename = format!(
        "{}/game_{:04}_{}.json",
        kifu_dir,
        game_num,
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );

    let kifu_data = KifuData {
        board_setup: board_setup.to_string(),
        moves: game.history.clone(),
    };

    let file = std::fs::File::create(filename)?;
    serde_json::to_writer(file, &kifu_data)?;
    Ok(())
}
