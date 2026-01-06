use crate::core::PlayerId;
use crate::game::{Game, KifuData, PerspectiveMode, ThinkingInfo};
use crate::player::ai::{AIStrength, AlphaBetaAI};
use crate::player::PlayerController;
use crossterm::{execute, terminal};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub enum BoardSetupType {
    StandardMixed,
    ReversedMixed,
    ShogiOnly,
    ChessOnly,
    Fair,
    ReversedFair,
}

impl std::fmt::Display for BoardSetupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BoardSetupType::StandardMixed => "StandardMixed",
            BoardSetupType::ReversedMixed => "ReversedMixed",
            BoardSetupType::ShogiOnly => "ShogiOnly",
            BoardSetupType::ChessOnly => "ChessOnly",
            BoardSetupType::Fair => "Fair",
            BoardSetupType::ReversedFair => "ReversedFair",
        };
        write!(f, "{}", s)
    }
}

impl BoardSetupType {
    fn create_board(&self) -> crate::core::Board {
        match self {
            BoardSetupType::StandardMixed => {
                let map = crate::core::setup::get_standard_mixed_setup();
                crate::core::setup::setup_from_strings(&map, true, true, None, None)
            }
            BoardSetupType::ReversedMixed => {
                let map = crate::core::setup::get_reversed_mixed_setup();
                crate::core::setup::setup_from_strings(&map, true, true, None, None)
            }
            BoardSetupType::ShogiOnly => {
                let map = crate::core::setup::get_shogi_setup();
                crate::core::setup::setup_from_strings(&map, true, true, None, None)
            }
            BoardSetupType::ChessOnly => {
                let map = crate::core::setup::get_chess_setup();
                crate::core::setup::setup_from_strings(&map, false, false, None, None)
            }
            BoardSetupType::Fair => {
                let map = crate::core::setup::get_fair_setup();
                crate::core::setup::setup_from_strings(&map, true, true, None, None)
            }
            BoardSetupType::ReversedFair => {
                let map = crate::core::setup::get_reversed_fair_setup();
                crate::core::setup::setup_from_strings(&map, false, false, None, None)
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
    pub use_parallel: bool,
    pub update_interval_moves: usize, // How often workers update shared state
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

pub struct GameExecutionResult {
    pub game: Game,
    pub winner: Option<PlayerId>,
    pub move_count: usize,
    pub thinking_data: Vec<ThinkingInfo>,
    pub duration: std::time::Duration,
}

// State for a single worker slot
struct WorkerState {
    status: String,
    game_id: Option<usize>,
}

struct SharedProgress {
    workers: Mutex<Vec<WorkerState>>,
    completed_games: AtomicUsize,
    p1_wins: AtomicUsize,
    p2_wins: AtomicUsize,
    draws: AtomicUsize,
    total_games: usize,
    is_running: AtomicBool,
}

// Parallel self-play implementation
pub fn run_selfplay(config: SelfPlayConfig) -> anyhow::Result<SelfPlayStats> {
    let num_threads = if config.use_parallel { 6 } else { 1 }; // Default to 6 for parallel

    // Configure thread pool if parallel
    if config.use_parallel {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .ok();
    }

    let mut stats = SelfPlayStats::new(
        config.board_setup.to_string(),
        config.ai1_strength,
        config.ai2_strength,
    );

    let mode = if config.use_parallel {
        if rayon::current_num_threads() > 0 {
            format!("parallel ({} threads)", rayon::current_num_threads())
        } else {
            "parallel".to_string()
        }
    } else {
        "sequential".to_string()
    };

    println!("\n=== Self-Play Configuration Details ===\r");
    println!("Total Games: {}\r", config.num_games);
    println!("Execution Mode: {}\r", mode);
    println!("Board Setup: {}\r", config.board_setup);
    println!("Update Interval: {} moves\r", config.update_interval_moves);

    // Determine promotion status based on board setup
    // For Mixed/Fair, both can promote. For others it might vary, but in this codebase promotion is generally enabled.
    // We'll display "Enabled" for both unless we have logic to say otherwise.
    let p1_promo = "Enabled";
    let p2_promo = "Enabled";
    println!("Promotion Rules:\r");
    println!("  Player 1: {}\r", p1_promo);
    println!("  Player 2: {}\r", p2_promo);

    println!("Player 1 (AI): {:?}\r", config.ai1_strength);
    println!("Player 2 (AI): {:?}\r", config.ai2_strength);
    println!("Save Kifu: {}\r", config.save_kifus);
    println!("=======================================\n\r");

    println!("Starting execution...\r");

    let num_display_slots = if config.use_parallel { num_threads } else { 1 };

    // Reserve space for UI
    println!("\r"); // Header line for Overall Progress
    for _ in 0..num_display_slots {
        println!("\r");
    }

    // Initialize shared state
    let shared_state = Arc::new(SharedProgress {
        workers: Mutex::new(
            (0..num_display_slots)
                .map(|_| WorkerState {
                    status: "Waiting...".to_string(),
                    game_id: None,
                })
                .collect(),
        ),
        completed_games: AtomicUsize::new(0),
        p1_wins: AtomicUsize::new(0),
        p2_wins: AtomicUsize::new(0),
        draws: AtomicUsize::new(0),
        total_games: config.num_games,
        is_running: AtomicBool::new(true),
    });

    // Start UI thread
    let ui_state = Arc::clone(&shared_state);
    let ui_handle = thread::spawn(move || {
        let mut stdout = std::io::stdout();
        use std::io::Write;

        while ui_state.is_running.load(Ordering::Relaxed)
            || ui_state.completed_games.load(Ordering::Relaxed) < ui_state.total_games
        {
            // Draw
            let completed = ui_state.completed_games.load(Ordering::Relaxed);
            let total = ui_state.total_games;
            let percent = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            // Move cursor to top of reserved area
            // Area:
            // [Progress Header]
            // [Slot 0]
            // ...
            // [Slot N-1]
            // Cursor is currently at N+1 lines down (logically)

            let lines_total = num_display_slots + 1;
            write!(stdout, "\x1B[{}A", lines_total).ok();

            // Draw Header
            let p1_w = ui_state.p1_wins.load(Ordering::Relaxed);
            let p2_w = ui_state.p2_wins.load(Ordering::Relaxed);
            let d = ui_state.draws.load(Ordering::Relaxed);

            let (p1_pct, p2_pct, d_pct) = if completed > 0 {
                (
                    (p1_w as f64 / completed as f64) * 100.0,
                    (p2_w as f64 / completed as f64) * 100.0,
                    (d as f64 / completed as f64) * 100.0,
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            write!(
                stdout,
                "\r\x1B[KProgress: {}/{} ({:.1}%) - P1: {} ({:.1}%), P2: {} ({:.1}%), Draw: {} ({:.1}%)\r\n",
                completed, total, percent, p1_w, p1_pct, p2_w, p2_pct, d, d_pct
            )
            .ok();

            // Draw Slots
            {
                let workers = ui_state.workers.lock().unwrap();
                for w in workers.iter() {
                    write!(stdout, "\r\x1B[K{}\r\n", w.status).ok();
                }
            }

            stdout.flush().ok();

            if ui_state.is_running.load(Ordering::Relaxed) == false && completed >= total {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    let results: Vec<_> = if config.use_parallel {
        (1..=config.num_games)
            .into_par_iter()
            .map(|game_num| execute_game_with_monitoring(game_num, &config, &shared_state))
            .collect()
    } else {
        (1..=config.num_games)
            .map(|game_num| execute_game_with_monitoring(game_num, &config, &shared_state))
            .collect()
    };

    // Signal UI to stop
    shared_state.is_running.store(false, Ordering::Relaxed);
    ui_handle.join().ok();

    println!("\r\n\r\nProcessing results...\r");

    // Process results sequentially
    for (idx, result) in results.into_iter().enumerate() {
        let game_num = idx + 1;
        let exec_result = result?;

        let game_result = GameResult {
            winner: exec_result.winner,
            moves: exec_result.move_count,
            time_ms: exec_result.duration.as_millis(),
        };

        stats.add_result(game_result);

        if config.save_kifus {
            save_kifu(
                &exec_result.game,
                game_num,
                &stats.board_setup,
                config.ai1_strength,
                config.ai2_strength,
                exec_result.thinking_data,
            )?;
        }
    }

    // Final Stats Display
    execute!(
        std::io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;

    println!("=== Self-Play Complete ===\r\n");
    println!("Total Games: {}\r", stats.total_games);
    println!(
        "P1 Wins: {} ({:.1}%)\r",
        stats.p1_wins,
        stats.p1_wins as f64 / stats.total_games as f64 * 100.0
    );
    println!(
        "P2 Wins: {} ({:.1}%)\r",
        stats.p2_wins,
        stats.p2_wins as f64 / stats.total_games as f64 * 100.0
    );
    println!(
        "Draws: {} ({:.1}%)\r",
        stats.draws,
        stats.draws as f64 / stats.total_games as f64 * 100.0
    );
    println!("Avg Moves: {:.1}\r", stats.avg_moves);
    println!("Avg Time: {:.1}s\r\n", stats.avg_time_ms / 1000.0);

    Ok(stats)
}

fn execute_game_with_monitoring(
    game_num: usize,
    config: &SelfPlayConfig,
    shared: &Arc<SharedProgress>,
) -> anyhow::Result<GameExecutionResult> {
    // Allocate slot
    let slot_idx = {
        let mut workers = shared.workers.lock().unwrap();
        if let Some(idx) = workers.iter().position(|w| w.game_id.is_none()) {
            workers[idx].game_id = Some(game_num);
            workers[idx].status = format!("Game {}: Starting...", game_num);
            idx
        } else {
            // Fallback: use id modulo slots (collision possible in display but logic safe)
            game_num % workers.len()
        }
    };

    // Callback
    let update_interval = config.update_interval_moves;
    let shared_clone = Arc::clone(shared);
    let on_progress = move |moves: usize, player: PlayerId| {
        if moves % update_interval == 0 {
            let mut workers = shared_clone.workers.lock().unwrap();
            if slot_idx < workers.len() {
                workers[slot_idx].status =
                    format!("Game {}: Move {} ({:?})", game_num, moves + 1, player);
            }
        }
    };

    let result = run_single_game(game_num, config, true, Some(Box::new(on_progress)));

    // Completion update
    {
        // Update stats
        if let Ok(ref res) = result {
            match res.winner {
                Some(PlayerId::Player1) => {
                    shared.p1_wins.fetch_add(1, Ordering::Relaxed);
                }
                Some(PlayerId::Player2) => {
                    shared.p2_wins.fetch_add(1, Ordering::Relaxed);
                }
                None => {
                    shared.draws.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        let mut workers = shared.workers.lock().unwrap();
        if slot_idx < workers.len() {
            workers[slot_idx].status = format!("Game {}: Finished", game_num);
            workers[slot_idx].game_id = None; // Free slot
        }
        shared.completed_games.fetch_add(1, Ordering::Relaxed);
    }

    result
}

fn run_single_game(
    _game_num: usize,
    config: &SelfPlayConfig,
    silent: bool,
    on_progress: Option<Box<dyn Fn(usize, PlayerId) + Send + Sync>>,
) -> anyhow::Result<GameExecutionResult> {
    let start_time = Instant::now();

    let board = config.board_setup.create_board();

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

    let mut game = Game::new(board);
    game.perspective_mode = PerspectiveMode::Fixed(PlayerId::Player1);

    let (winner, move_count, thinking_data) =
        run_game_silent(&mut game, p1.as_ref(), p2.as_ref(), silent, on_progress)?;

    let elapsed = start_time.elapsed();

    Ok(GameExecutionResult {
        game,
        winner,
        move_count,
        thinking_data,
        duration: elapsed,
    })
}

fn run_game_silent(
    game: &mut Game,
    p1: &dyn PlayerController,
    p2: &dyn PlayerController,
    silent: bool,
    on_progress: Option<Box<dyn Fn(usize, PlayerId) + Send + Sync>>,
) -> anyhow::Result<(Option<PlayerId>, usize, Vec<ThinkingInfo>)> {
    let mut move_count = 0;
    let mut thinking_data = Vec::new();
    let max_moves = 500;

    loop {
        if move_count >= max_moves {
            return Ok((None, move_count, thinking_data.clone()));
        }

        let current_player = game.current_player;
        let controller = match current_player {
            PlayerId::Player1 => p1,
            PlayerId::Player2 => p2,
        };

        if let Some(ref cb) = on_progress {
            cb(move_count, current_player);
        }

        let hash_count = game
            .board
            .history
            .iter()
            .filter(|&&h| h == game.board.zobrist_hash)
            .count();
        if hash_count >= 4 {
            return Ok((None, move_count, thinking_data.clone()));
        }

        let legal_moves = crate::logic::legal_moves(&game.board, current_player);

        if legal_moves.is_empty() {
            let in_check = crate::logic::is_in_check(&game.board, current_player);
            if in_check {
                return Ok((
                    Some(current_player.opponent()),
                    move_count,
                    thinking_data.clone(),
                ));
            } else {
                return Ok((None, move_count, thinking_data.clone()));
            }
        }

        if !silent {
            print!(
                "\r\x1B[KMove {}: {:?} thinking...",
                move_count + 1,
                current_player
            );
            std::io::Write::flush(&mut std::io::stdout())?;
        }

        if let Some(chosen_move) = controller.choose_move(&game.board, &legal_moves) {
            let ai_ptr =
                controller as *const dyn crate::player::PlayerController as *const AlphaBetaAI;
            if let Some((depth, score, nodes, time_ms)) =
                unsafe { *(*ai_ptr).last_thinking.borrow() }
            {
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
            ));
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

    if let Ok(abs_path) = std::fs::canonicalize(&filename) {
        println!("Saved kifu to: {}\r", abs_path.display());
    } else {
        println!("Saved kifu to: {}\r", filename);
    }

    Ok(())
}
