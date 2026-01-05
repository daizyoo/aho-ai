use crate::core::{Board, Move, PlayerId};
use crate::display::{render_board, DisplayState};
use crate::logic::apply_move;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub struct ReplayViewer {
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

        Self {
            history,
            boards,
            current_index: 0,
        }
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

            let mut state = DisplayState::default();
            state.perspective = PlayerId::Player1; // Fixed perspective for replay? Or auto? Let's use P1.
            state.last_move = last_move.clone();
            state.status_msg = Some(format!(
                "Replay: Move {}/{} ({:?}) - [Only Standard/Fair supported?]",
                self.current_index,
                self.history.len(),
                next_player
            ));

            render_board(board, &state);
            println!("\r\nControls: [<-/p] Prev, [->/n] Next, [q] Quit");

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
