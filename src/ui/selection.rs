use crate::core::{Board, PlayerId};
use crate::game::PerspectiveMode;
use crate::player::PlayerController;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub fn create_player_controllers(
    choice: &str,
) -> anyhow::Result<(
    Box<dyn PlayerController>,
    Box<dyn PlayerController>,
    PerspectiveMode,
)> {
    match choice {
        "1" => Ok((
            Box::new(crate::player::TuiController::new(
                PlayerId::Player1,
                "Player1",
            )),
            Box::new(crate::player::TuiController::new(
                PlayerId::Player2,
                "Player2",
            )),
            PerspectiveMode::AutoFlip,
        )),
        "2" => Ok((
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
        )),
        "3" => Ok((
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
        )),
        "4" => Ok((
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
        )),
        _ => Err(anyhow::anyhow!("Invalid selection")),
    }
}

fn ask_hand_config(player_name: &str) -> anyhow::Result<Option<bool>> {
    print!(
        "\r\nEnable held pieces (mochigoma) for {}? (y: Yes, n: No, Enter: Default): ",
        player_name
    );
    use std::io::Write;
    std::io::stdout().flush()?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('y') => {
                        println!("Yes\r");
                        return Ok(Some(true));
                    }
                    KeyCode::Char('n') => {
                        println!("No\r");
                        return Ok(Some(false));
                    }
                    KeyCode::Enter | KeyCode::Char('d') => {
                        println!("Default\r");
                        return Ok(None);
                    }
                    KeyCode::Char('q') => return Err(anyhow::anyhow!("Canceled")),
                    _ => {}
                }
            }
        }
    }
}

pub fn select_board_setup() -> anyhow::Result<(Board, String)> {
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
                    KeyCode::Char('q') => return Err(anyhow::anyhow!("Canceled")),
                    _ => {}
                }
            }
        }
    };

    println!("\r"); // New line after selection

    let p1_hand = ask_hand_config("Player 1")?;
    let p2_hand = ask_hand_config("Player 2")?;

    Ok(match b_choice {
        "1" => (
            crate::core::setup::setup_from_strings(
                &crate::core::setup::get_standard_mixed_setup(),
                true,
                false,
                p1_hand,
                p2_hand,
            ),
            "StandardMixed".to_string(),
        ),
        "2" => (
            crate::core::setup::setup_from_strings(
                &crate::core::setup::get_reversed_mixed_setup(),
                false,
                true,
                p1_hand,
                p2_hand,
            ),
            "ReversedMixed".to_string(),
        ),
        "3" => (
            crate::core::setup::setup_from_strings(
                &crate::core::setup::get_shogi_setup(),
                true,
                true,
                p1_hand,
                p2_hand,
            ),
            "ShogiOnly".to_string(),
        ),
        "4" => (
            crate::core::setup::setup_from_strings(
                &crate::core::setup::get_chess_setup(),
                false,
                false,
                p1_hand,
                p2_hand,
            ),
            "ChessOnly".to_string(),
        ),
        "5" => (
            crate::core::setup::setup_from_strings(
                &crate::core::setup::get_fair_setup(),
                true,
                true,
                p1_hand,
                p2_hand,
            ),
            "Fair".to_string(),
        ),
        _ => (
            crate::core::setup::setup_from_strings(
                &crate::core::setup::get_reversed_fair_setup(),
                true,
                true,
                p1_hand,
                p2_hand,
            ),
            "ReversedFair".to_string(),
        ),
    })
}
