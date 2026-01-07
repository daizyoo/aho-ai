use crate::core::{Board, PlayerId};
use crate::game::PerspectiveMode;
use crate::player::PlayerController;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub fn create_player_controllers(
    choice: &str,
    model_path: Option<String>,
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
                model_path,
                false,
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
                model_path.clone(),
                false,
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        )),
        "4" => Ok((
            Box::new(crate::player::ai::AlphaBetaAI::new(
                PlayerId::Player1,
                "AlphaBeta-Strong-1",
                crate::player::ai::AIStrength::Strong,
                model_path.clone(),
                false,
            )),
            Box::new(crate::player::ai::AlphaBetaAI::new(
                PlayerId::Player2,
                "AlphaBeta-Strong-2",
                crate::player::ai::AIStrength::Strong,
                model_path,
                false,
            )),
            PerspectiveMode::Fixed(PlayerId::Player1),
        )),
        _ => Err(anyhow::anyhow!("Invalid selection")),
    }
}

pub fn select_model() -> anyhow::Result<Option<String>> {
    use crate::ml::model_registry::ModelRegistry;
    use std::io::Write;

    let mut registry = ModelRegistry::new();
    registry.discover_models("models")?;

    let models = registry.list();
    if models.is_empty() {
        println!("\r\n[!] No ML models found in 'models/' directory.\r");
        println!("Using default model if configured in ai_config.json.\r");
        std::thread::sleep(Duration::from_secs(2));
        return Ok(None);
    }

    // Sort models by name for consistency
    let mut models: Vec<_> = models.into_iter().collect();
    models.sort_by(|a, b| a.name.cmp(&b.name));

    let mut selected_idx = 0;

    print!("\r\nSelect ML Model (Use ↑/↓ and Enter):\r\n");

    loop {
        // Render list
        for (i, model) in models.iter().enumerate() {
            let prefix = if i == selected_idx { "> " } else { "  " };
            let version = model
                .version
                .as_ref()
                .map(|v| format!(" (v{})", v))
                .unwrap_or_default();
            print!("\r\x1B[K{}{}{}\r\n", prefix, model.name, version);
        }

        std::io::stdout().flush()?;

        // Move cursor back up
        print!("\x1B[{}A", models.len());

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected_idx > 0 {
                            selected_idx -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_idx < models.len() - 1 {
                            selected_idx += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Clear the menu display area
                        for _ in 0..models.len() {
                            print!("\r\x1B[K\r\n");
                        }
                        print!("\x1B[{}A", models.len());

                        let selected_model = models[selected_idx];
                        println!("Selected Model: {}\r", selected_model.path.display());
                        return Ok(Some(selected_model.path.to_string_lossy().to_string()));
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Clear the menu
                        for _ in 0..models.len() {
                            print!("\r\x1B[K\r\n");
                        }
                        print!("\x1B[{}A", models.len());
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
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

    let (p1_hand, p2_hand) = match b_choice {
        "3" | "5" | "6" => (Some(true), Some(true)), // Shogi/Fair modes always enable hands
        "4" => (Some(false), Some(false)),           // ChessOnly always disables hands
        _ => (ask_hand_config("Player 1")?, ask_hand_config("Player 2")?),
    };

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
