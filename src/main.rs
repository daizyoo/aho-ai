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
    // ターミナル初期化
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let res = run().await;

    // ターミナル復帰
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    res
}

async fn run() -> anyhow::Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    use std::time::Duration;

    print!("=== Unified Board Game Engine (Shogi x Chess) ===\r\n");

    print!("\r\nSelect mode:\r\n");
    print!("1. Local Play\r\n");
    print!("2. Start Server (127.0.0.1:8080)\r\n");
    print!("3. Connect to Server (127.0.0.1:8080)\r\n");

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
            crate::network::server::start_server("127.0.0.1:8080").await?;
            return Ok(());
        }
        "client" => {
            return run_client().await;
        }
        _ => run_local().await,
    }
}

async fn run_client() -> anyhow::Result<()> {
    use crate::core::{Board, Move};
    use crate::game::Game;
    use crate::network::client::NetworkClient;
    use crate::player::network::NetworkController;
    use std::sync::mpsc;
    use tokio::sync::mpsc as tokio_mpsc;

    print!("Connecting to server...\r\n");
    let client = NetworkClient::connect("127.0.0.1:8080").await?;
    println!("Connected!");

    let (player_id_tx, player_id_rx) = mpsc::channel::<PlayerId>();
    let (board_tx, board_rx) = mpsc::channel::<Board>();
    let (remote_move_tx, remote_move_rx) = mpsc::channel::<Move>();
    let (local_move_tx, local_move_rx) = tokio_mpsc::unbounded_channel::<Move>();

    let mut client_handle = client;
    tokio::spawn(async move {
        if let Err(e) = client_handle
            .run(player_id_tx, board_tx, remote_move_tx, local_move_rx)
            .await
        {
            eprintln!("Client networking error: {}", e);
        }
    });

    // Wait for initial data
    print!("Waiting for opponent...\r\n");
    let my_id = player_id_rx.recv()?;
    let board = board_rx.recv()?;

    let mut game = Game::new(board);
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
    use crate::game::Game;
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

    let (p1, p2): (Box<dyn PlayerController>, Box<dyn PlayerController>) = match p_choice {
        "1" => (
            Box::new(TuiController::new(PlayerId::Player1, "Player 1")),
            Box::new(TuiController::new(PlayerId::Player2, "Player 2")),
        ),
        "2" => (
            Box::new(TuiController::new(PlayerId::Player1, "Human")),
            Box::new(WeightedRandomAI::new(PlayerId::Player2, "Weighted AI")),
        ),
        "3" => (
            Box::new(WeightedRandomAI::new(PlayerId::Player1, "Sente AI")),
            Box::new(WeightedRandomAI::new(PlayerId::Player2, "Gote AI")),
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

    let board = match b_choice {
        "1" => setup_from_strings(&crate::core::setup::get_standard_mixed_setup(), true, false),
        "2" => setup_from_strings(&crate::core::setup::get_reversed_mixed_setup(), false, true),
        "3" => setup_from_strings(&crate::core::setup::get_shogi_setup(), true, true),
        _ => setup_from_strings(&crate::core::setup::get_chess_setup(), false, false),
    };

    let mut game = Game::new(board);
    game.play(p1.as_ref(), p2.as_ref(), |_| {});

    Ok(())
}
