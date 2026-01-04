mod core;
mod display;
mod game;
mod logic;
mod network;
mod player;

use crate::core::PlayerId;
use crate::player::{PlayerController, TuiController};
use crossterm::{execute, terminal};
use std::io;

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

    print!("Connecting to {}...\r\n", addr);
    let client = NetworkClient::connect(addr).await?;
    println!("Connected!");

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
    use crate::player::ai::weighted::WeightedRandomAI;
    use crossterm::event::{self, Event, KeyCode};
    use std::time::Duration;

    print!("\r\nSelect players:\r\n");
    print!("1. Human vs Human (TUI)\r\n");
    print!("2. Human vs Weighted AI\r\n");
    print!("3. Weighted AI vs Weighted AI\r\n");

    let p_choice = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => break "1",
                    KeyCode::Char('2') => break "2",
                    KeyCode::Char('3') => break "3",
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    };

    let (p1, p2, perspective): (
        Box<dyn PlayerController>,
        Box<dyn PlayerController>,
        PerspectiveMode,
    ) = match p_choice {
        "1" => (
            Box::new(TuiController::new(PlayerId::Player1, "Player 1")),
            Box::new(TuiController::new(PlayerId::Player2, "Player 2")),
            PerspectiveMode::AutoFlip,
        ),
        "2" => (
            Box::new(TuiController::new(PlayerId::Player1, "Human")),
            Box::new(WeightedRandomAI::new(PlayerId::Player2, "Weighted AI")),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        "3" => (
            Box::new(WeightedRandomAI::new(PlayerId::Player1, "Sente AI")),
            Box::new(WeightedRandomAI::new(PlayerId::Player2, "Gote AI")),
            PerspectiveMode::Fixed(PlayerId::Player1),
        ),
        _ => unreachable!(),
    };

    print!("\r\nSelect board setup:\r\n");
    print!("1. Shogi (P1) vs Chess (P2)\r\n");
    print!("2. Chess (P1) vs Shogi (P2)\r\n");
    print!("3. Shogi vs Shogi\r\n");
    print!("4. Chess vs Chess\r\n");

    let b_choice = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => break "1",
                    KeyCode::Char('2') => break "2",
                    KeyCode::Char('3') => break "3",
                    KeyCode::Char('4') => break "4",
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    };

    let (p1_shogi, p2_shogi) = match b_choice {
        "1" => (true, false),
        "2" => (false, true),
        "3" => (true, true),
        _ => (false, false),
    };

    let board = match b_choice {
        "1" => setup_from_strings(
            &crate::core::setup::get_standard_mixed_setup(),
            p1_shogi,
            p2_shogi,
        ),
        "2" => setup_from_strings(
            &crate::core::setup::get_reversed_mixed_setup(),
            p1_shogi,
            p2_shogi,
        ),
        "3" => setup_from_strings(&crate::core::setup::get_shogi_setup(), p1_shogi, p2_shogi),
        _ => setup_from_strings(&crate::core::setup::get_chess_setup(), p1_shogi, p2_shogi),
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
