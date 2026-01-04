mod core;
mod display;
mod game;
mod logic;
mod network;
mod player;

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
        let addr = if args.len() >= 3 {
            &args[2]
        } else {
            "127.0.0.1:8080"
        };

        match mode {
            "server" => {
                crate::network::server::start_server(addr).await?;
                return Ok(());
            }
            "client" => {
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

    let mode = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => break "local",
                    KeyCode::Char('2') => break "server",
                    KeyCode::Char('3') => break "client",
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    };

    match mode {
        "server" => {
            let addr = read_input_raw(
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
            let addr = read_input_raw("127.0.0.1:8080", "Enter server address to connect").await?;
            run_client(&addr).await
        }
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
    use crate::core::setup_from_strings;
    use crate::game::{Game, PerspectiveMode};
    use crossterm::event::{self, Event, KeyCode};
    use std::time::Duration;

    print!("\r\nSelect players:\r\n");
    print!("1. Human vs Human (TUI)\r\n");
    print!("\r\n");
    print!("--- Player vs AI ---\r\n");
    print!("2. Player vs Random AI\r\n");
    print!("3. Player vs Weighted Random AI\r\n");
    print!("4. Player vs Minimax AI (Depth 2)\r\n");
    print!("7. Player vs Alpha-Beta AI (Light)\r\n");
    print!("8. Player vs Alpha-Beta AI (Strong)\r\n");
    print!("\r\n");
    print!("--- AI vs AI ---\r\n");
    print!("5. Weighted AI vs Weighted AI\r\n");
    print!("9. Alpha-Beta AI (Strong) vs Alpha-Beta AI (Strong)\r\n");
    print!("\r\n");
    print!("10. Replay Game Record (Kifu)\r\n");

    let p_choice = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => break "1",
                    KeyCode::Char('2') => break "2",
                    KeyCode::Char('3') => break "3",
                    KeyCode::Char('4') => break "4",
                    KeyCode::Char('5') => break "5",
                    KeyCode::Char('6') => break "6",
                    KeyCode::Char('7') => break "7",
                    KeyCode::Char('8') => break "8",
                    KeyCode::Char('9') => break "9",
                    KeyCode::Char('0') => break "10", // For option 10
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    };

    if p_choice == "10" {
        let kifu_dir = "kifu";
        if std::fs::read_dir(kifu_dir).is_err() {
            std::fs::create_dir_all(kifu_dir)?;
        }

        if let Some(path) = select_kifu_file(kifu_dir)? {
            let file = std::fs::File::open(path)?;
            let history: Vec<crate::core::Move> = serde_json::from_reader(file)?;

            // Viewer uses alternate screen, we are already in it or should ensure it.
            // (Assuming main entered it)
            let mut viewer = crate::game::replay::ReplayViewer::new(history);
            viewer.run()?;
        }
        return Ok(());
    }

    let (mut p1, mut p2, perspective): (
        Box<dyn PlayerController>,
        Box<dyn PlayerController>,
        PerspectiveMode,
    ) = match p_choice {
        "1" => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::TuiController::new(
                PlayerId::Player2,
                "Player2",
            )),
            PerspectiveMode::AutoFlip,
        ),
        "2" => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::ai::WeightedRandomAI::new(
                PlayerId::Player2,
                "WeightedAI",
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "3" => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::ai::WeightedRandomAI::new(
                PlayerId::Player2,
                "WeightedAI",
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "4" => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::ai::MinimaxAI::new(
                PlayerId::Player2,
                "MinimaxAI",
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "5" => (
            Box::new(crate::player::ai::WeightedRandomAI::new(
                PlayerId::Player1,
                "WeightedAI1",
            )),
            Box::new(crate::player::ai::WeightedRandomAI::new(
                PlayerId::Player2,
                "WeightedAI2",
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "7" => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::ai::AlphaBetaAI::new(
                PlayerId::Player2,
                "AlphaBeta-Light",
                crate::player::ai::AIStrength::Light,
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "8" => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::ai::AlphaBetaAI::new(
                PlayerId::Player2,
                "AlphaBeta-Strong",
                crate::player::ai::AIStrength::Strong,
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "9" => (
            Box::new(crate::player::ai::AlphaBetaAI::new(
                PlayerId::Player1,
                "AlphaBeta-Strong-1",
                crate::player::ai::AIStrength::Strong,
            )),
            Box::new(crate::player::ai::AlphaBetaAI::new(
                PlayerId::Player2,
                "AlphaBeta-Strong-2",
                crate::player::ai::AIStrength::Strong,
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "10" => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::TuiController::new(
                PlayerId::Player2,
                "Player2",
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        _ => (
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::TuiController::new(
                PlayerId::Player2,
                "Player2",
            )),
            PerspectiveMode::AutoFlip,
        ),
    };
    // This line was part of the original code's structure, but the new block
    // already returns the tuple directly from the match statement.
    // Removing it to avoid a syntax error.
    // (p1, p2, perspective)
    // The match statement now directly assigns to (p1, p2, perspective)
    // so this line is no longer needed.
    // The original code had `let (mut p1, mut p2, perspective) = { ... };`
    // where the block returned the tuple.
    // The new code has `let (mut p1, mut p2, perspective) = match p_choice { ... };`
    // where each match arm returns the tuple.
    // So the final `(p1, p2, perspective)` is not needed.
    // The instruction provided `(p1, p2, perspective)` at the end, which would be a syntax error.
    // I will remove it to make the code syntactically correct.
    // The instruction also had an extra `}` at the end of the `let` assignment, which I've corrected.
    // The instruction's provided code block ends with `};` which is correct for the `let = match` syntax.
    // The instruction's provided code block also had `(p1, p2, perspective)` after the closing `};`
    // which is incorrect. I will remove that.

    print!("\r\nSelect board setup:\r\n");
    print!("1. Shogi (P1) vs Chess (P2)\r\n");
    print!("2. Chess (P1) vs Shogi (P2)\r\n");
    print!("3. Shogi vs Shogi\r\n");
    print!("4. Chess vs Chess\r\n");
    print!("5. Fair (Mixed Shogi/Chess)\r\n");
    print!("6. Reversed Fair\r\n");

    let b_choice = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => break "1",
                    KeyCode::Char('2') => break "2",
                    KeyCode::Char('3') => break "3",
                    KeyCode::Char('4') => break "4",
                    KeyCode::Char('5') => break "5",
                    KeyCode::Char('6') => break "6",
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    };

    // p1_shogi / p2_shogi determine piece parsing when 1-char symbols are used
    // in fair setup, both are used, so we just set them based on majority or just true/false
    let board = match b_choice {
        "1" => setup_from_strings(&crate::core::setup::get_standard_mixed_setup(), true, false),
        "2" => setup_from_strings(&crate::core::setup::get_reversed_mixed_setup(), false, true),
        "3" => setup_from_strings(&crate::core::setup::get_shogi_setup(), true, true),
        "4" => setup_from_strings(&crate::core::setup::get_chess_setup(), false, false),
        "5" => setup_from_strings(&crate::core::setup::get_fair_setup(), true, true), // Mixed: use S hint for both
        _ => setup_from_strings(&crate::core::setup::get_reversed_fair_setup(), true, true), // Mixed: use S hint for both
    };

    let mut game = Game::new(board);
    game.perspective_mode = perspective;
    game.play(p1.as_ref(), p2.as_ref(), |_| {});

    Ok(())
}

async fn read_input_raw(default: &str, prompt: &str) -> anyhow::Result<String> {
    use crossterm::event::{self, Event, KeyCode};
    use std::io::Write;
    use std::time::Duration;

    print!("{}: (Default: {}) \r\n> ", prompt, default);
    io::stdout().flush()?;

    let mut input = String::new();
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        print!("\r\n");
                        if input.is_empty() {
                            return Ok(default.to_string());
                        } else {
                            return Ok(input);
                        }
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        print!("{}", c);
                        io::stdout().flush()?;
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            print!("\u{0008} \u{0008}");
                            io::stdout().flush()?;
                        }
                    }
                    KeyCode::Esc => {
                        return Err(anyhow::anyhow!("Canceled"));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn select_kifu_file(dir: &str) -> anyhow::Result<Option<std::path::PathBuf>> {
    use crossterm::event::{self, Event, KeyCode};
    use std::fs;
    use std::time::Duration;

    // Ensure dir exists
    if fs::metadata(dir).is_err() {
        return Ok(None);
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            files.push(path);
        }
    }

    // Sort by modified time (descending)
    files.sort_by_key(|p| {
        fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    files.reverse();

    if files.is_empty() {
        println!("No kifu files found in '{}'. Press any key to return.", dir);
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }
        return Ok(None);
    }

    let mut selected_index = 0;

    loop {
        execute!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        println!("Select Kifu to Replay (Arrow Keys / Enter / q):");
        println!("------------------------------------------------");

        for (i, file) in files.iter().enumerate() {
            let name = file.file_name().and_then(|s| s.to_str()).unwrap_or("???");
            if i == selected_index {
                println!("> {}", name);
            } else {
                println!("  {}", name);
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected_index > 0 {
                            selected_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_index < files.len() - 1 {
                            selected_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        return Ok(Some(files[selected_index].clone()));
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
    }
}
