mod core;
mod display;
mod game;
mod logic;
mod player;

use crate::core::{setup_from_strings, PlayerId};
use crate::game::Game;
use crate::player::{ai::RandomAI, PlayerController, TuiController};
use crossterm::{execute, terminal};
use std::io;

fn main() -> anyhow::Result<()> {
    // ターミナル初期化
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let res = run();

    // ターミナル復帰
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    res
}

fn run() -> anyhow::Result<()> {
    use crossterm::event::{self, Event, KeyCode};

    print!("=== Unified Board Game Engine (Shogi x Chess) ===\r\n");

    print!("\r\nSelect players:\r\n");
    print!("1. Human vs Human (TUI)\r\n");
    print!("2. Human vs Weighted AI\r\n");
    print!("3. Weighted AI vs Weighted AI\r\n");
    print!("4. Random AI vs Random AI\r\n");

    let p_choice = loop {
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
    };

    let (p1, p2): (Box<dyn PlayerController>, Box<dyn PlayerController>) = match p_choice {
        "1" => (
            Box::new(TuiController::new(PlayerId::Player1, "Player 1")),
            Box::new(TuiController::new(PlayerId::Player2, "Player 2")),
        ),
        "2" => (
            Box::new(TuiController::new(PlayerId::Player1, "Human")),
            Box::new(crate::player::ai::weighted::WeightedRandomAI::new(
                PlayerId::Player2,
                "Weighted AI",
            )),
        ),
        "3" => (
            Box::new(crate::player::ai::weighted::WeightedRandomAI::new(
                PlayerId::Player1,
                "Weighted AI 1",
            )),
            Box::new(crate::player::ai::weighted::WeightedRandomAI::new(
                PlayerId::Player2,
                "Weighted AI 2",
            )),
        ),
        _ => (
            Box::new(RandomAI::new(PlayerId::Player1, "Random AI 1")),
            Box::new(RandomAI::new(PlayerId::Player2, "Random AI 2")),
        ),
    };

    print!("\r\nSelect board setup:\r\n");
    print!("1. Shogi (P1) vs Chess (P2)\r\n");
    print!("2. Chess (P1) vs Shogi (P2)\r\n");
    print!("3. Shogi vs Shogi\r\n");
    print!("4. Chess vs Chess\r\n");

    let b_choice = loop {
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
    };

    let board = match b_choice {
        "1" => setup_from_strings(&crate::core::setup::get_standard_mixed_setup(), true, false),
        "2" => setup_from_strings(&crate::core::setup::get_reversed_mixed_setup(), false, true),
        "3" => setup_from_strings(&crate::core::setup::get_shogi_setup(), true, true),
        _ => setup_from_strings(&crate::core::setup::get_chess_setup(), false, false),
    };

    let mut game = Game::new(board);
    game.play(p1.as_ref(), p2.as_ref());

    Ok(())
}
