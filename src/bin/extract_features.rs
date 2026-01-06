//! CLI tool for extracting board features from kifu files
//!
//! This binary is called by Python scripts to convert kifu games into training data.

use shogi_aho_ai::core::{Board, PlayerId};
use shogi_aho_ai::game::KifuData;
use shogi_aho_ai::logic::apply_move;
use shogi_aho_ai::ml::features::BoardFeatureExtractor;
use std::env;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: extract_features <kifu_file.json>");
        std::process::exit(1);
    }

    let kifu_path = &args[1];

    // Load kifu
    let file = File::open(kifu_path)?;
    let kifu: KifuData = serde_json::from_reader(file)?;

    // Reconstruct board setup
    let board = match kifu.board_setup.as_str() {
        "StandardMixed" => {
            let map = shogi_aho_ai::core::setup::get_standard_mixed_setup();
            shogi_aho_ai::core::setup::setup_from_strings(&map, true, true, None, None)
        }
        "ReversedMixed" => {
            let map = shogi_aho_ai::core::setup::get_reversed_mixed_setup();
            shogi_aho_ai::core::setup::setup_from_strings(&map, true, true, None, None)
        }
        "ShogiOnly" => {
            let map = shogi_aho_ai::core::setup::get_shogi_setup();
            shogi_aho_ai::core::setup::setup_from_strings(&map, true, true, None, None)
        }
        "ChessOnly" => {
            let map = shogi_aho_ai::core::setup::get_chess_setup();
            shogi_aho_ai::core::setup::setup_from_strings(&map, false, false, None, None)
        }
        "Fair" => {
            let map = shogi_aho_ai::core::setup::get_fair_setup();
            shogi_aho_ai::core::setup::setup_from_strings(&map, true, true, None, None)
        }
        "ReversedFair" => {
            let map = shogi_aho_ai::core::setup::get_reversed_fair_setup();
            shogi_aho_ai::core::setup::setup_from_strings(&map, false, false, None, None)
        }
        _ => {
            eprintln!("Unknown board setup: {}", kifu.board_setup);
            std::process::exit(1);
        }
    };

    // Replay game and extract features
    let mut current_board = board;
    let mut current_player = PlayerId::Player1;

    for (move_idx, mv) in kifu.moves.iter().enumerate() {
        // Extract features from current position
        let features = BoardFeatureExtractor::extract(&current_board, current_player);

        // Output as JSON line
        let output = serde_json::json!({
            "move_idx": move_idx,
            "features": features,
            "move": mv,
            "player": format!("{:?}", current_player),
        });

        println!("{}", serde_json::to_string(&output)?);

        // Apply move
        current_board = apply_move(&current_board, mv, current_player);
        current_player = current_player.opponent();
    }

    Ok(())
}
