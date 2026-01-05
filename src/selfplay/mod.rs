use crate::core::PlayerId;
use crate::game::{Game, KifuData, PerspectiveMode, ThinkingInfo};
use crate::player::ai::{AIStrength, AlphaBetaAI};
use crate::player::PlayerController;
use crossterm::{execute, terminal};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum BoardSetupType {
    StandardMixed,
    ReversedMixed,
    ShogiOnly,
    ChessOnly,
    Fair,
    ReversedFair,
}

impl BoardSetupType {
    fn to_string(&self) -> String {
        match self {
            BoardSetupType::StandardMixed => "StandardMixed",
            BoardSetupType::ReversedMixed => "ReversedMixed",
            BoardSetupType::ShogiOnly => "ShogiOnly",
            BoardSetupType::ChessOnly => "ChessOnly",
            BoardSetupType::Fair => "Fair",
            BoardSetupType::ReversedFair => "ReversedFair",
        }
        .to_string()
    }

    fn create_board(&self) -> crate::core::Board {
        match self {
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
        }
    }
}

#[derive(Clone, Copy)]
pub struct SelfPlayConfig {
    pub num_games: usize,
    pub board_setup: BoardSetupType,
    pub ai1_strength: AIStrength,
    pub ai2_strength: AIStrength,
    pub save_kifus: bool,
    pub use_parallel: bool, // Enable/disable parallel execution
}

#[derive(Serialize, Deserialize)]
struct GameResult {
    winner: Option<PlayerId>,
    moves: usize,
    time_ms: u128,
}

#[derive(Serialize)]
pub struct SelfPlayStats {
    pub total_games: usize,
    pub p1_wins: usize,
    pub p2_wins: usize,
    pub draws: usize,
    pub avg_moves: f64,
    pub avg_time_ms: f64,
    pub board_setup: String,
    pub ai1_strength: String,
    pub ai2_strength: String,
}

impl SelfPlayStats {
    fn new(board_setup: String, ai1_strength: AIStrength, ai2_strength: AIStrength) -> Self {
        Self {
            total_games: 0,
            p1_wins: 0,
            p2_wins: 0,
            draws: 0,
            avg_moves: 0.0,
            avg_time_ms: 0.0,
            board_setup,
            ai1_strength: format!("{:?}", ai1_strength),
            ai2_strength: format!("{:?}", ai2_strength),
        }
    }

    fn add_result(&mut self, result: GameResult) {
        self.total_games += 1;
        match result.winner {
            Some(PlayerId::Player1) => self.p1_wins += 1,
            Some(PlayerId::Player2) => self.p2_wins += 1,
            None => self.draws += 1,
        }

        // Update averages
        let n = self.total_games as f64;
        self.avg_moves = (self.avg_moves * (n - 1.0) + result.moves as f64) / n;
        self.avg_time_ms = (self.avg_time_ms * (n - 1.0) + result.time_ms as f64) / n;
    }
}

// Parallel self-play implementation
pub fn run_selfplay(config: SelfPlayConfig) -> anyhow::Result<SelfPlayStats> {
    // Configure thread pool for optimal performance
    // Using 6 threads on 8-core system leaves headroom for OS and background tasks
    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build_global()
        .ok(); // Ignore error if already initialized

    let mut stats = SelfPlayStats::new(
        config.board_setup.to_string(),
        config.ai1_strength,
        config.ai2_strength,
    );

    let mode = if config.use_parallel {
        "parallel (6 threads)"
    } else {
        "sequential"
    };
    println!("Starting {} games ({} mode)...\r", config.num_games, mode);
    println!(
        "AI Strength: {:?} vs {:?}\r",
        config.ai1_strength, config.ai2_strength
    );
    println!();

    let results: Vec<_> = if config.use_parallel {
        // Parallel execution with progress display
        let game_status = Arc::new(Mutex::new(vec![None; config.num_games]));

        (1..=config.num_games)
            .into_par_iter()
            .map(|game_num| {
                let result = run_single_game(game_num, &config, true); // silent mode

                // Update progress
                {
                    let mut status = game_status.lock().unwrap();
                    status[game_num - 1] = Some(true);
                    display_progress(&status, config.num_games);
                }
                result
            })
            .collect()
    } else {
        // Sequential execution with simple counter
        (1..=config.num_games)
            .map(|game_num| {
                println!("Running game {}/{}...", game_num, config.num_games);
                run_single_game(game_num, &config, false) // verbose mode
            })
            .collect()
    };

    println!("\n\nProcessing results...");

    // Process results sequentially
    for (idx, result) in results.into_iter().enumerate() {
        let game_num = idx + 1;
        let (game, winner, move_count, thinking_data, elapsed) = result?;

        let game_result = GameResult {
            winner,
            moves: move_count,
            time_ms: elapsed.as_millis(),
        };

        stats.add_result(game_result);

        // Save kifu if requested
        if config.save_kifus {
            save_kifu(
                &game,
                game_num,
                &stats.board_setup,
                config.ai1_strength,
                config.ai2_strength,
                thinking_data,
            )?;
        }
    }

    // Display final statistics
    execute!(
        std::io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;

    println!("=== Self-Play Complete ===\n");
    println!("Total Games: {}", stats.total_games);
    println!(
        "P1 Wins: {} ({:.1}%)",
        stats.p1_wins,
        stats.p1_wins as f64 / stats.total_games as f64 * 100.0
    );
    println!(
        "P2 Wins: {} ({:.1}%)",
        stats.p2_wins,
        stats.p2_wins as f64 / stats.total_games as f64 * 100.0
    );
    println!(
        "Draws: {} ({:.1}%)",
        stats.draws,
        stats.draws as f64 / stats.total_games as f64 * 100.0
    );
    println!("Avg Moves: {:.1}", stats.avg_moves);
    println!("Avg Time: {:.1}s\n", stats.avg_time_ms / 1000.0);

    Ok(stats)
}

fn run_single_game(
    _game_num: usize,
    config: &SelfPlayConfig,
    silent: bool,
) -> anyhow::Result<(
    Game,
    Option<PlayerId>,
    usize,
    Vec<ThinkingInfo>,
    std::time::Duration,
)> {
    let start_time = Instant::now();

    // Create board
    let board = config.board_setup.create_board();

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

    // Run game
    let mut game = Game::new(board);
    game.perspective_mode = PerspectiveMode::Fixed(PlayerId::Player1);

    let (winner, move_count, thinking_data) =
        run_game_silent(&mut game, p1.as_ref(), p2.as_ref(), silent)?;

    let elapsed = start_time.elapsed();

    Ok((game, winner, move_count, thinking_data, elapsed))
}
fn run_game_silent(
    game: &mut Game,
    p1: &dyn PlayerController,
    p2: &dyn PlayerController,
    silent: bool,
) -> anyhow::Result<(Option<PlayerId>, usize, Vec<ThinkingInfo>)> {
    let mut move_count = 0;
    let mut thinking_data = Vec::new();
    let max_moves = 500; // Prevent infinite games

    loop {
        if move_count >= max_moves {
            return Ok((None, move_count, thinking_data.clone())); // Draw by move limit
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
                return Ok((
                    Some(current_player.opponent()),
                    move_count,
                    thinking_data.clone(),
                ));
            } else {
                return Ok((None, move_count, thinking_data.clone())); // Stalemate
            }
        }

        // Display current move being calculated (only in verbose mode)
        if !silent {
            print!(
                "\r\x1B[KMove {}: {:?} thinking...",
                move_count + 1,
                current_player
            );
            std::io::Write::flush(&mut std::io::stdout())?;
        }

        if let Some(chosen_move) = controller.choose_move(&game.board, &legal_moves) {
            // Collect thinking data from AI
            let ai_ptr =
                controller as *const dyn crate::player::PlayerController as *const AlphaBetaAI;
            if let Some((depth, score, nodes, time_ms)) =
                unsafe { (*ai_ptr).last_thinking.borrow().clone() }
            {
                // IMPORTANT: score is from the AI's perspective (always positive = good for AI)
                // We need to convert to Player1's perspective for consistent analysis
                // Player1: keep score as-is
                // Player2: negate score (because it's from P2's perspective, we want P1's perspective)
                let normalized_score = if current_player == crate::core::PlayerId::Player1 {
                    score
                } else {
                    -score
                };

                thinking_data.push(ThinkingInfo {
                    move_number: move_count + 1,
                    player: format!("{:?}", current_player),
                    depth,
                    score: normalized_score,
                    nodes,
                    time_ms,
                });
            }

            game.board = crate::logic::apply_move(&game.board, &chosen_move, current_player);
            game.history.push(chosen_move);
            game.current_player = current_player.opponent();
            move_count += 1;
        } else {
            return Ok((
                Some(current_player.opponent()),
                move_count,
                thinking_data.clone(),
            )); // Resignation
        }
    }
}

fn save_kifu(
    game: &Game,
    game_num: usize,
    board_setup: &str,
    ai1_strength: AIStrength,
    ai2_strength: AIStrength,
    thinking_data: Vec<ThinkingInfo>,
) -> anyhow::Result<()> {
    // Create board-type-specific directory
    let base_dir = "selfplay_kifu";
    let board_dir = format!("{}/{}", base_dir, board_setup);
    std::fs::create_dir_all(&board_dir)?;

    let filename = format!(
        "{}/game_{:04}_{}.json",
        board_dir,
        game_num,
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );

    let kifu_data = KifuData {
        board_setup: board_setup.to_string(),
        player1_name: format!("AI ({:?})", ai1_strength),
        player2_name: format!("AI ({:?})", ai2_strength),
        moves: game.history.clone(),
        thinking_data: Some(thinking_data),
    };

    let file = std::fs::File::create(&filename)?;
    serde_json::to_writer_pretty(file, &kifu_data)?;

    Ok(())
}

// Display progress as simple counter (reliable in parallel execution)
fn display_progress(status: &[Option<bool>], total: usize) {
    let completed = status.iter().filter(|&&s| s == Some(true)).count();
    let running = status.iter().filter(|&&s| s == Some(false)).count();

    // Clear line and show simple progress
    print!("\r\x1B[K");
    print!(
        "Progress: {} running, {} completed / {} total ({:.1}%)",
        running,
        completed,
        total,
        (completed as f64 / total as f64) * 100.0
    );
    std::io::Write::flush(&mut std::io::stdout()).ok();
}
