use crate::core::{Board, Move, PlayerId};
use crate::logic::apply_move;
use crate::ui::display::{render_board, DisplayState};
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
                setup::setup_from_strings(&map, true, false, None, None)
            }
            "ReversedMixed" => {
                let map = setup::get_reversed_mixed_setup();
                setup::setup_from_strings(&map, false, true, None, None)
            }
            "ShogiOnly" => {
                let map = setup::get_shogi_setup();
                setup::setup_from_strings(&map, true, true, None, None)
            }
            "ChessOnly" => {
                let map = setup::get_chess_setup();
                setup::setup_from_strings(&map, false, false, None, None)
            }
            "Fair" => {
                let map = setup::get_fair_setup();
                setup::setup_from_strings(&map, true, true, None, None)
            }
            "ReversedFair" => {
                let map = setup::get_reversed_fair_setup();
                setup::setup_from_strings(&map, false, false, None, None)
            }
            _ => {
                // Default to Fair if unknown
                let map = setup::get_fair_setup();
                setup::setup_from_strings(&map, true, true, None, None)
            }
        }
    }

    #[allow(dead_code)]
    pub fn from_kifu_path(path: &std::path::Path) -> anyhow::Result<Self> {
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let kifu_data: crate::game::KifuData = serde_json::from_reader(reader)?;
        Ok(Self::new(kifu_data))
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            // Get current board state
            let board = &self.boards[self.current_index];
            let last_move = if self.current_index > 0 {
                Some(self.history[self.current_index - 1].clone())
            } else {
                None
            };

            // Clear screen
            execute!(
                io::stdout(),
                terminal::Clear(terminal::ClearType::All),
                crossterm::cursor::MoveTo(0, 0)
            )?;

            // Render board
            let state = DisplayState {
                perspective: PlayerId::Player1,
                last_move,
                status_msg: None,
                ..Default::default()
            };

            render_board(board, &state);

            // Display game info AFTER board
            print!("\r\n");
            print!("=== Kifu Replay ===\r\n");
            print!("Setup: {}\r\n", self.kifu.board_setup);
            print!(
                "{} vs {}\r\n",
                self.kifu.player1_name, self.kifu.player2_name
            );

            // Display winner
            let total_moves = self.kifu.moves.len();
            if total_moves > 0 {
                let (winner_name, winner_id) = if total_moves % 2 == 1 {
                    (&self.kifu.player1_name, "Player1")
                } else {
                    (&self.kifu.player2_name, "Player2")
                };
                print!("Winner: {} ({})\r\n", winner_name, winner_id);
            }

            println!(
                "\rMove {}/{} | [←/→] Navigate | [q] Quit",
                self.current_index + 1,
                total_moves
            );

            // Input handling
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
