mod core;

mod game;
mod logic;
#[cfg(test)]
mod logic_tests;
mod ml;
mod network;
mod player;
mod selfplay;
mod ui;

use crate::core::PlayerId;
use crate::player::{PlayerController, TuiController};
use crossterm::{execute, terminal};
use std::io::{self};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Check for CLI arguments first
    if args.len() >= 2 {
        let mode = args[1].as_str();

        match mode {
            "server" => {
                let addr = if args.len() >= 3 {
                    &args[2]
                } else {
                    "127.0.0.1:8080"
                };
                crate::network::server::start_server(addr).await?;
                return Ok(());
            }
            "client" => {
                let addr = if args.len() >= 3 {
                    &args[2]
                } else {
                    "127.0.0.1:8080"
                };
                terminal::enable_raw_mode()?;
                execute!(io::stdout(), terminal::EnterAlternateScreen)?;
                let res = run_client(addr).await;
                execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
                terminal::disable_raw_mode()?;
                return res;
            }
            "local" => {
                terminal::enable_raw_mode()?;
                execute!(io::stdout(), terminal::EnterAlternateScreen)?;
                let res = run_local().await;
                execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
                terminal::disable_raw_mode()?;
                return res;
            }
            "replay" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} replay <kifu_file_path>", args[0]);
                    eprintln!(
                        "Example: {} replay selfplay_kifu/ShogiOnly/20260108_124038/game_0034.json",
                        args[0]
                    );
                    return Ok(());
                }
                let kifu_path = std::path::Path::new(&args[2]);
                if !kifu_path.exists() {
                    eprintln!("Error: File not found: {}", kifu_path.display());
                    return Ok(());
                }

                terminal::enable_raw_mode()?;
                execute!(io::stdout(), terminal::EnterAlternateScreen)?;
                let res = run_replay_file(kifu_path);
                execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
                terminal::disable_raw_mode()?;
                return res;
            }
            "selfplay" => {
                return run_selfplay_cli(&args[2..]).await;
            }
            _ => {} // Fall back to menu if mode is invalid
        }
    }

    // Interactive Menu
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let res = run_menu().await;

    // ターミナル復帰
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    res
}

async fn run_menu() -> anyhow::Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    use std::time::Duration;

    print!("=== Unified Board Game Engine (Shogi x Chess) ===\r\n");

    print!("\r\nSelect mode:\r\n");
    print!("1. Local Play\r\n");
    print!("2. Start Server\r\n");
    print!("3. Connect to Server\r\n");
    print!("4. Self-Play (Batch AI vs AI)\r\n");

    let mode = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => break "local",
                    KeyCode::Char('2') => break "server",
                    KeyCode::Char('3') => break "client",
                    KeyCode::Char('4') => break "selfplay",
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    };

    match mode {
        "server" => {
            let addr = crate::ui::read_input_raw(
                "127.0.0.1:8080",
                "Enter server BIND address (e.g. 0.0.0.0:8080)",
            )
            .await?;

            // ユーザーがngrokのホスト名などを入力してしまった場合のガイド
            if addr.contains("ngrok")
                || (addr.contains(".")
                    && !addr.chars().next().unwrap().is_ascii_digit()
                    && !addr.starts_with("localhost"))
            {
                print!("\r\n[!] Warning: It looks like you entered a hostname instead of a local IP.\r\n");
                print!(
                    "    To use ngrok, bind the server to '0.0.0.0:8080' or '127.0.0.1:8080'\r\n"
                );
                print!("    and then run 'ngrok http 8080' in a separate terminal.\r\n\r\n");
            }

            print!("Starting server on {}...\r\n", addr);
            if let Err(e) = crate::network::server::start_server(&addr).await {
                eprintln!("Failed to start server: {}\r\n", e);
                eprintln!("Try binding to '0.0.0.0:8080' instead.\r\n");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
            Ok(())
        }
        "client" => {
            let addr =
                crate::ui::read_input_raw("127.0.0.1:8080", "Enter server address to connect")
                    .await?;
            run_client(&addr).await
        }
        "selfplay" => run_selfplay().await,
        _ => run_local().await,
    }
}

async fn run_client(addr: &str) -> anyhow::Result<()> {
    use crate::core::{Board, Move};
    use crate::game::Game;
    use crate::network::client::NetworkClient;
    use crate::player::network::NetworkController;
    use std::sync::mpsc;
    use tokio::sync::mpsc as tokio_mpsc;

    let sanitized = NetworkClient::sanitize_addr(addr);
    print!("Connecting to {}... (Original: {})\r\n", sanitized, addr);

    let client_res = NetworkClient::connect(&sanitized).await;
    let client = match client_res {
        Ok(c) => c,
        Err(e) => {
            print!("\r\n[!] Connection Failed: {}\r\n", e);
            print!("    Hint: If using ngrok, try 'ngrok tcp 8080' instead of 'ngrok http'.\r\n");
            print!("    Wait 5s to return to menu...\r\n");
            std::thread::sleep(std::time::Duration::from_secs(5));
            return Err(e);
        }
    };
    print!("Connected!\r\n");

    let (player_id_tx, player_id_rx) = mpsc::channel::<PlayerId>();
    let (remote_move_tx, remote_move_rx) = mpsc::channel::<Move>();
    let (local_move_tx, local_move_rx) = tokio_mpsc::unbounded_channel::<Move>();

    // 盤面更新同期用
    let (board_sync_tx, board_sync_rx) = mpsc::channel::<(Board, PlayerId)>();

    let mut client_handle = client;
    tokio::spawn(async move {
        // board_tx の代わりに board_sync_tx を渡す
        if let Err(e) = client_handle
            .run(player_id_tx, board_sync_tx, remote_move_tx, local_move_rx)
            .await
        {
            eprintln!("Client networking error: {}", e);
        }
    });

    // Wait for initial data
    print!("Waiting for opponent...\r\n");
    let my_id = player_id_rx.recv()?;
    // 初期盤面を同期チャネルから受け取る
    let (board, _next_player) = board_sync_rx.recv()?;

    let mut game = Game::new(board);
    game.board_sync_rx = Some(board_sync_rx);
    game.perspective_mode = crate::game::PerspectiveMode::Fixed(my_id);

    let p1: Box<dyn PlayerController>;
    let p2: Box<dyn PlayerController>;

    if my_id == PlayerId::Player1 {
        p1 = Box::new(TuiController::new(PlayerId::Player1, "You"));
        p2 = Box::new(NetworkController::new(
            PlayerId::Player2,
            "Remote",
            remote_move_rx,
        ));
    } else {
        p1 = Box::new(NetworkController::new(
            PlayerId::Player1,
            "Remote",
            remote_move_rx,
        ));
        p2 = Box::new(TuiController::new(PlayerId::Player2, "You"));
    }

    game.play(p1.as_ref(), p2.as_ref(), |mv| {
        let _ = local_move_tx.send(mv.clone());
    });

    Ok(())
}

async fn run_local() -> anyhow::Result<()> {
    use crate::game::Game;
    use crossterm::event::{self, Event, KeyCode};
    use std::time::Duration;

    print!("\r\nSelect game mode:\r\n");
    print!("1. Human vs Human (TUI)\r\n");
    print!("\r\n");
    print!("--- Player vs AI ---\r\n");
    print!("2. Player vs Alpha-Beta AI (Light)\r\n");
    print!("3. Player vs Alpha-Beta AI (Strong)\r\n");
    print!("\r\n");
    print!("--- AI vs AI ---\r\n");
    print!("4. Alpha-Beta AI (Strong) vs Alpha-Beta AI (Strong)\r\n");
    print!("\r\n");
    print!("5. Replay Game Record (Kifu)\r\n");

    let p_choice = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => break "1",
                    KeyCode::Char('2') => break "2",
                    KeyCode::Char('3') => break "3",
                    KeyCode::Char('4') => break "4",
                    KeyCode::Char('5') => break "5",
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    };

    if p_choice == "5" {
        // Replay - Use new scrollable selector
        use crate::ui::kifu_selector::KifuSelector;
        use std::path::PathBuf;

        // Scan multiple directories including selfplay_kifu
        let dirs = vec![
            PathBuf::from("kifu"),
            PathBuf::from("selfplay_results"),
            PathBuf::from("selfplay_kifu"),
        ];

        let mut selector = KifuSelector::scan_directories(&dirs)?;

        loop {
            match selector.run()? {
                Some(selected_path) => {
                    // User selected a file
                    let mut viewer =
                        crate::game::replay::ReplayViewer::from_kifu_path(&selected_path)?;
                    viewer.run()?;
                    // After replay, return to file selection
                }
                None => {
                    // User quit
                    break;
                }
            }
        }

        return Ok(());
    }

    let mut model_path = None;
    if p_choice == "2" || p_choice == "3" || p_choice == "4" {
        use crate::player::ai::config::AIConfig;
        let config = AIConfig::get();
        if config.evaluation.evaluator_type == "NeuralNetwork" {
            model_path = crate::ui::selection::select_model()?;
        }
    }

    let (p1, p2, perspective) =
        crate::ui::selection::create_player_controllers(p_choice, model_path.clone())?;

    // Display evaluator if AI is involved
    if p_choice == "2" || p_choice == "3" || p_choice == "4" {
        use crate::core::PlayerId;
        use crate::player::ai::alpha_beta::AlphaBetaAI;
        let temp_ai = AlphaBetaAI::new(
            PlayerId::Player1,
            "Display",
            crate::player::ai::alpha_beta::AIStrength::Strong,
            model_path,
            true,
        );
        print!("\r\n[AI Configuration]\r\n");
        print!("Evaluator: {}\r\n\r\n", temp_ai.evaluator_name());
        std::thread::sleep(std::time::Duration::from_millis(500)); // Brief pause to show message
    }

    let (board, setup_name) = crate::ui::selection::select_board_setup()?;

    let mut game = Game::with_setup(board, setup_name);
    game.perspective_mode = perspective;
    game.play(p1.as_ref(), p2.as_ref(), |_| {});

    Ok(())
}

async fn run_selfplay() -> anyhow::Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    use std::time::Duration;

    // Configuration UI
    execute!(
        std::io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;

    print!("=== Self-Play Configuration ===\r\n\r\n");

    // Number of games
    print!("Number of games (default: 10): ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut num_input = String::new();
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => break,
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        num_input.push(c);
                        print!("{}", c);
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                    KeyCode::Backspace => {
                        if !num_input.is_empty() {
                            num_input.pop();
                            print!("\u{0008} \u{0008}");
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let num_games = if num_input.is_empty() {
        10
    } else {
        num_input.parse().unwrap_or(10)
    };

    // AI1 Strength Selection
    print!("\r\nPlayer 1 AI Strength:\r\n");
    print!("1. Light (Depth 4, 1s)\r\n");
    print!("2. Strong (Depth 6, 3s)\r\n");
    print!("Select (default: 2): ");

    let ai1_strength = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => {
                        print!("1\r\n");
                        break crate::player::ai::AIStrength::Light;
                    }
                    KeyCode::Char('2') | KeyCode::Enter => {
                        print!("2\r\n");
                        break crate::player::ai::AIStrength::Strong;
                    }
                    _ => {}
                }
            }
        }
    };

    // AI2 Strength Selection
    print!("\r\nPlayer 2 AI Strength:\r\n");
    print!("1. Light (Depth 4, 1s)\r\n");
    print!("2. Strong (Depth 6, 3s)\r\n");
    print!("Select (default: 2): ");

    let ai2_strength = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => {
                        print!("1\r\n");
                        break crate::player::ai::AIStrength::Light;
                    }
                    KeyCode::Char('2') | KeyCode::Enter => {
                        print!("2\r\n");
                        break crate::player::ai::AIStrength::Strong;
                    }
                    _ => {}
                }
            }
        }
    };

    // Board Setup Selection
    print!("\r\nBoard Setup:\r\n");
    print!("1. Standard Mixed (Shogi P1 vs Chess P2)\r\n");
    print!("2. Reversed Mixed (Chess P1 vs Shogi P2)\r\n");
    print!("3. Shogi Only\r\n");
    print!("4. Chess Only\r\n");
    print!("5. Fair (Symmetric Mixed)\r\n");
    print!("6. Reversed Fair\r\n");
    print!("Select (default: 3): "); // Changed default from 5 to 3

    let board_setup = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => {
                        print!("1\r\n");
                        break crate::selfplay::BoardSetupType::StandardMixed;
                    }
                    KeyCode::Char('2') => {
                        print!("2\r\n");
                        break crate::selfplay::BoardSetupType::ReversedMixed;
                    }
                    KeyCode::Char('3') | KeyCode::Enter => {
                        // Changed: Enter now selects ShogiOnly
                        print!("3\r\n");
                        break crate::selfplay::BoardSetupType::ShogiOnly;
                    }
                    KeyCode::Char('4') => {
                        print!("4\r\n");
                        break crate::selfplay::BoardSetupType::ChessOnly;
                    }
                    KeyCode::Char('5') => {
                        // Removed KeyCode::Enter from here
                        print!("5\r\n");
                        break crate::selfplay::BoardSetupType::Fair;
                    }
                    KeyCode::Char('6') => {
                        print!("6\r\n");
                        break crate::selfplay::BoardSetupType::ReversedFair;
                    }
                    _ => {}
                }
            }
        }
    };

    // Execution Mode Selection
    print!("\r\nExecution Mode:\r\n");
    print!("1. Parallel (Multi-threaded)\r\n");
    print!("2. Sequential (Single-threaded)\r\n");
    print!("Select (default: 1): ");

    let use_parallel = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') | KeyCode::Enter => {
                        print!("1\r\n");
                        break true;
                    }
                    KeyCode::Char('2') => {
                        print!("2\r\n");
                        break false;
                    }
                    _ => {}
                }
            }
        }
    };
    let mut model_path = None;
    {
        use crate::player::ai::config::AIConfig;
        let config = AIConfig::get();
        if config.evaluation.evaluator_type == "NeuralNetwork" {
            model_path = crate::ui::selection::select_model()?;
        }
    }

    print!("\r\n\r\nRunning {} games...\r\n\r\n", num_games);

    // Run self-play
    let config = crate::selfplay::SelfPlayConfig {
        num_games,
        board_setup,
        ai1_strength,
        ai2_strength,
        use_parallel, // Use the selected mode
        save_kifus: true,
        update_interval_moves: 1,
        model_path: model_path.clone(),
    };

    // Display configuration before starting
    use crate::core::PlayerId;
    use crate::player::ai::alpha_beta::AlphaBetaAI;
    let temp_ai = AlphaBetaAI::new(
        PlayerId::Player1,
        "Display",
        config.ai1_strength,
        model_path,
        true,
    );

    print!("\r\n");
    print!("=== Self-Play Configuration ===\r\n");
    print!("Games: {}\r\n", num_games);
    print!("Board: {:?}\r\n", board_setup);
    print!("AI Strength: {:?}\r\n", ai1_strength);
    print!("Evaluator: {}\r\n", temp_ai.evaluator_name());
    print!("==============================\r\n\r\n");
    std::thread::sleep(std::time::Duration::from_secs(2));

    let stats = crate::selfplay::run_selfplay(config)?;

    // Display results
    print!("\r\n\r\n=== Self-Play Results ===\r\n");
    print!("Evaluator: {}\r\n", temp_ai.evaluator_name());
    print!("Total Games: {}\r\n", stats.total_games);
    print!(
        "Player 1 Wins: {} ({:.1}%)\r\n",
        stats.p1_wins,
        stats.p1_wins as f64 / stats.total_games as f64 * 100.0
    );
    print!(
        "Player 2 Wins: {} ({:.1}%)\r\n",
        stats.p2_wins,
        stats.p2_wins as f64 / stats.total_games as f64 * 100.0
    );
    print!(
        "Draws: {} ({:.1}%)\r\n",
        stats.draws,
        stats.draws as f64 / stats.total_games as f64 * 100.0
    );
    print!("Average Moves: {:.1}\r\n", stats.avg_moves);
    print!(
        "Average Time: {:.1}s per game\r\n",
        stats.avg_time_ms / 1000.0
    );

    // Save results to JSON
    let results_dir = "selfplay_results";
    std::fs::create_dir_all(results_dir)?;

    let results_file = format!(
        "{}/selfplay_results_{}.json",
        results_dir,
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let file = std::fs::File::create(&results_file)?;
    serde_json::to_writer_pretty(file, &stats)?;

    if let Ok(abs_path) = std::fs::canonicalize(&results_file) {
        print!("\r\nResults saved to {}\r\n", abs_path.display());
    } else {
        print!("\r\nResults saved to {}\r\n", results_file);
    }

    print!("\r\nPress any key to return to menu...\r\n");
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }
    }

    Ok(())
}

async fn run_selfplay_cli(args: &[String]) -> anyhow::Result<()> {
    // Parse command line arguments
    let mut num_games = 10;
    let mut board_setup = crate::selfplay::BoardSetupType::ShogiOnly;
    let mut ai1_strength = crate::player::ai::AIStrength::Strong;
    let mut ai2_strength = crate::player::ai::AIStrength::Strong;
    let mut use_parallel = true;
    let mut num_threads: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--num-games" => {
                if i + 1 < args.len() {
                    num_games = args[i + 1].parse().unwrap_or(10);
                    i += 1;
                }
            }
            "--board" => {
                if i + 1 < args.len() {
                    board_setup = match args[i + 1].as_str() {
                        "StandardMixed" => crate::selfplay::BoardSetupType::StandardMixed,
                        "ReversedMixed" => crate::selfplay::BoardSetupType::ReversedMixed,
                        "ShogiOnly" => crate::selfplay::BoardSetupType::ShogiOnly,
                        "ChessOnly" => crate::selfplay::BoardSetupType::ChessOnly,
                        "Fair" => crate::selfplay::BoardSetupType::Fair,
                        "ReversedFair" => crate::selfplay::BoardSetupType::ReversedFair,
                        _ => {
                            eprintln!("Unknown board type: {}, using ShogiOnly", args[i + 1]);
                            crate::selfplay::BoardSetupType::ShogiOnly
                        }
                    };
                    i += 1;
                }
            }
            "--ai1-strength" => {
                if i + 1 < args.len() {
                    ai1_strength = match args[i + 1].as_str() {
                        "Light" => crate::player::ai::AIStrength::Light,
                        "Strong" => crate::player::ai::AIStrength::Strong,
                        _ => {
                            eprintln!("Unknown AI strength: {}, using Strong", args[i + 1]);
                            crate::player::ai::AIStrength::Strong
                        }
                    };
                    i += 1;
                }
            }
            "--ai2-strength" => {
                if i + 1 < args.len() {
                    ai2_strength = match args[i + 1].as_str() {
                        "Light" => crate::player::ai::AIStrength::Light,
                        "Strong" => crate::player::ai::AIStrength::Strong,
                        _ => {
                            eprintln!("Unknown AI strength: {}, using Strong", args[i + 1]);
                            crate::player::ai::AIStrength::Strong
                        }
                    };
                    i += 1;
                }
            }
            "--parallel" => {
                use_parallel = true;
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        num_threads = Some(n);
                        i += 1;
                    }
                }
            }
            "--sequential" => {
                use_parallel = false;
            }
            "--help" | "-h" => {
                println!("Self-Play Mode Usage:");
                println!("  cargo run --release -- selfplay [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --num-games <N>          Number of games to play (default: 10)");
                println!("  --board <TYPE>           Board type: StandardMixed, ReversedMixed, ShogiOnly, ChessOnly, Fair, ReversedFair (default: ShogiOnly)");
                println!("  --ai1-strength <STR>     Player 1 AI strength: Light, Strong (default: Strong)");
                println!("  --ai2-strength <STR>     Player 2 AI strength: Light, Strong (default: Strong)");
                println!("  --parallel [N]           Enable parallel execution with optional thread count");
                println!("  --sequential             Enable sequential execution");
                println!("  --help, -h               Show this help message");
                println!();
                println!("Examples:");
                println!(
                    "  cargo run --release -- selfplay --num-games 100 --board Fair --parallel 6"
                );
                println!("  cargo run --release -- selfplay --num-games 10 --board ShogiOnly --sequential");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
            }
        }
        i += 1;
    }

    // Set thread count if specified
    if let Some(n) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok();
    }

    let mut model_path = None;
    {
        use crate::player::ai::config::AIConfig;
        let config = AIConfig::get();
        if config.evaluation.evaluator_type == "NeuralNetwork" {
            // For CLI mode, try to auto-detect model or skip
            // Since we can't use interactive UI here
            println!("Note: NeuralNetwork evaluator detected in config.");
            println!("To use a specific model, please set it in ai_config.json");
        }
    }

    println!();
    println!("=== Self-Play Configuration ===");
    println!("Games: {}", num_games);
    println!("Board: {:?}", board_setup);
    println!("AI1 Strength: {:?}", ai1_strength);
    println!("AI2 Strength: {:?}", ai2_strength);
    println!(
        "Mode: {}",
        if use_parallel {
            "Parallel"
        } else {
            "Sequential"
        }
    );
    if let Some(n) = num_threads {
        println!("Threads: {}", n);
    }
    println!("==============================");
    println!();

    // Run self-play
    let config = crate::selfplay::SelfPlayConfig {
        num_games,
        board_setup,
        ai1_strength,
        ai2_strength,
        use_parallel,
        save_kifus: true,
        update_interval_moves: 1,
        model_path: model_path.clone(),
    };

    // Display evaluator
    use crate::core::PlayerId;
    use crate::player::ai::alpha_beta::AlphaBetaAI;
    let temp_ai = AlphaBetaAI::new(
        PlayerId::Player1,
        "Display",
        config.ai1_strength,
        model_path,
        true,
    );
    println!("Evaluator: {}", temp_ai.evaluator_name());
    println!();

    let stats = crate::selfplay::run_selfplay(config)?;

    // Display results
    println!();
    println!("=== Self-Play Results ===");
    println!("Evaluator: {}", temp_ai.evaluator_name());
    println!("Total Games: {}", stats.total_games);
    println!(
        "Player 1 Wins: {} ({:.1}%)",
        stats.p1_wins,
        stats.p1_wins as f64 / stats.total_games as f64 * 100.0
    );
    println!(
        "Player 2 Wins: {} ({:.1}%)",
        stats.p2_wins,
        stats.p2_wins as f64 / stats.total_games as f64 * 100.0
    );
    println!(
        "Draws: {} ({:.1}%)",
        stats.draws,
        stats.draws as f64 / stats.total_games as f64 * 100.0
    );
    println!("Average Moves: {:.1}", stats.avg_moves);
    println!("Average Time: {:.1}s per game", stats.avg_time_ms / 1000.0);

    // Save results to JSON
    let results_dir = "selfplay_results";
    std::fs::create_dir_all(results_dir)?;

    let results_file = format!(
        "{}/selfplay_results_{}.json",
        results_dir,
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let file = std::fs::File::create(&results_file)?;
    serde_json::to_writer_pretty(file, &stats)?;

    if let Ok(abs_path) = std::fs::canonicalize(&results_file) {
        println!();
        println!("Results saved to {}", abs_path.display());
    } else {
        println!();
        println!("Results saved to {}", results_file);
    }

    Ok(())
}

fn run_replay_file(kifu_path: &std::path::Path) -> anyhow::Result<()> {
    let mut viewer = crate::game::replay::ReplayViewer::from_kifu_path(kifu_path)?;
    viewer.run()?;
    Ok(())
}
