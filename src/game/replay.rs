use crate::core::{Board, Move, PlayerId};
use crate::display::{render_board, DisplayState};
use crate::logic::apply_move;
use crossterm::event::{self, Event, KeyCode};
use crossterm::{execute, terminal};
use std::io;
use std::time::Duration;

pub struct ReplayViewer {
    kifu: crate::game::KifuData,
    history: Vec<Move>,
    boards: Vec<Board>,
    current_index: usize,
}

impl ReplayViewer {
    pub fn new(kifu_data: crate::game::KifuData) -> Self {
        // Reconstruct initial board from board_setup string
        let board = Self::board_from_setup(&kifu_data.board_setup);

        // Pre-calculate all board states
        let mut boards = Vec::new();
        let mut current_board = board;
        let mut current_player = PlayerId::Player1;

        boards.push(current_board.clone());

        for mv in &kifu_data.moves {
            current_board = apply_move(&current_board, mv, current_player);
            boards.push(current_board.clone());
            current_player = current_player.opponent();
        }

        Self {
            kifu: kifu_data.clone(),
            history: kifu_data.moves,
            boards,
            current_index: 0,
        }
    }

    fn board_from_setup(setup: &str) -> Board {
        use crate::core::setup;

        match setup {
            "StandardMixed" => {
                let map = setup::get_standard_mixed_setup();
                setup::setup_from_strings(&map, true, false)
            }
            "ReversedMixed" => {
                let map = setup::get_reversed_mixed_setup();
                setup::setup_from_strings(&map, false, true)
            }
            "ShogiOnly" => {
                let map = setup::get_shogi_setup();
                setup::setup_from_strings(&map, true, true)
            }
            "ChessOnly" => {
                let map = setup::get_chess_setup();
                setup::setup_from_strings(&map, false, false)
            }
            "Fair" => {
                let map = setup::get_fair_setup();
                setup::setup_from_strings(&map, true, true)
            }
            "ReversedFair" => {
                let map = setup::get_reversed_fair_setup();
                setup::setup_from_strings(&map, false, false)
            }
            _ => {
                // Default to Fair if unknown
                let map = setup::get_fair_setup();
                setup::setup_from_strings(&map, true, true)
            }
        }
    }

    #[allow(dead_code)]
    pub fn with_initial_board(history: Vec<Move>, initial_board: Board) -> Self {
        let mut boards = Vec::new();
        let mut board = initial_board;
        boards.push(board.clone());
        let mut current_player = PlayerId::Player1;

        for mv in &history {
            board = apply_move(&board, mv, current_player);
            boards.push(board.clone());
            current_player = current_player.opponent();
        }

        // This function cannot initialize the `kifu` field correctly without `KifuData`.
        // It's likely intended for testing or specific scenarios where `KifuData` isn't available.
        // For now, we'll use a dummy KifuData.
        Self {
            kifu: crate::game::KifuData { board_setup: "Unknown".to_string(), player1_name: "?".to_string(), player2_name: "?".to_string(), moves: vec![] }, // Placeholder
            history,
            boards,
            current_index: 0,
        }
    }

    pub fn from_kifu_path(path: &std::path::Path) -> anyhow::Result<Self> {
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let kifu_data: crate::game::KifuData = serde_json::from_reader(reader)?;
        Ok(Self::new(kifu_data))
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let _current_player = PlayerId::Player1; // Tracks who moved to reach CURRENT state (approx)
                                                 // Actually, state[0] is initial. state[1] is after P1 move.
                                                 // So at index i, the next move is by P1 if i is even, P2 if i is odd.
                                                 // (Assuming P1 starts).

        loop {
            // Render
            let board = &self.boards[self.current_index];
            let last_move = if self.current_index > 0 {
                Some(self.history[self.current_index - 1].clone())
            } else {
                None
            };

            let next_player = if self.current_index % 2 == 0 {
                PlayerId::Player1
            } else {
                PlayerId::Player2
            };

            execute!(
                io::stdout(),
                terminal::Clear(terminal::ClearType::All),
                crossterm::cursor::MoveTo(0, 0)
            )?;

            // Display game info
            print!("=== Kifu Replay ===\r\n");
            print!("Setup: {}\r\n", self.kifu.board_setup);
            print!(
                "{} vs {}\r\n",
                self.kifu.player1_name, self.kifu.player2_name
            );

            // Display winner
            let total_moves = self.kifu.moves.len();
            if total_moves > 0 {
                let winner = if total_moves % 2 == 1 {
                    &self.kifu.player1_name
                } else {
                    &self.kifu.player2_name
                };
                print!("Winner: {}\r\n", winner);
            }
            print!("\r\n");

            print!(
                "Move {}/{} | [←/→] Navigate | [q] Quit\r\n",
                self.current_index, // Changed from current_move_index + 1
                self.kifu.moves.len()
            );
            print!("\r\n");

            let mut state = DisplayState::default();
            state.perspective = PlayerId::Player1; // Fixed perspective for replay? Or auto? Let's use P1.
            state.last_move = last_move.clone();
            state.status_msg = Some(format!(
                "Replay: Move {}/{} ({:?}) - [Only Standard/Fair supported?]",
                self.current_index,
                self.kifu.moves.len(), // Changed from self.history.len()
                next_player
            ));

            render_board(board, &state);
            // The original println!("\r\nControls: ...") is now replaced by the new print! statement above.
            // If it was intended to be kept, it would be redundant.
            // Based on the diff, it seems the new print! statement replaces the old status_msg and controls line.

            // Input
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Right | KeyCode::Char('n') => {
                            if self.current_index < self.history.len() {
                                self.current_index += 1;
                            }
                        }
                        KeyCode::Left | KeyCode::Char('p') => {
                            if self.current_index > 0 {
                                self.current_index -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
