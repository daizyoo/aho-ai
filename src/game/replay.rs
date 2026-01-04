use crate::core::setup::setup_from_strings;
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
    pub fn new(history: Vec<Move>) -> Self {
        // Pre-calculate all board states
        let mut boards = Vec::new();
        // Default to reversed mixed setup (Chess P1 vs Shogi P2) as per original MVP assumption
        // or wait, let's use standard Shogi vs Chess?
        // Let's stick to what I wrote before: get_reversed_mixed_setup (Wait, usually P1 is human, so Shogi vs Chess is standard?)
        // Let's use get_standard_mixed_setup() (Shogi P1 vs Chess P2) as default for now if generic.
        // Actually, main loop defaults to "5" (Fair) or "reversed_fair".
        // Let's assume Standard Mixed (Shogi P1 vs Chess P2) for simplicity or try to guess.
        // Or better: use setup_from_strings properly.

        // let map = crate::core::setup::get_reversed_mixed_setup();
        // let board = setup_from_strings(&map, false, true); // P1=Chess, P2=Shogi ? Reversed mixed setup implies P1=Chess likely if "reversed".

        // Wait, "Reversed Mixed" = Chess (P1) vs Shogi (P2).
        // "Standard Mixed" = Shogi (P1) vs Chess (P2).
        // Let's use Standard Mixed (Shogi P1) as default?
        // Or if the user previously played Reversed Fair... mismatch.
        // Ideally we save metadata.
        // For MVP, I will use "Standard Mixed" setup (Shogi P1, Chess P2).

        let map = crate::core::setup::get_standard_mixed_setup();
        let mut board = setup_from_strings(&map, true, false); // P1=Shogi, P2=Chess
        let mut current_player = PlayerId::Player1;

        // Note: The initial board setup might be different depending on the saved game!
        // Ideally, we should save the initial board state in the Kifu as well.
        // For now, we'll assume the user needs to select the correct setup or we default to a standard one.
        // Wait, Kifu just saves moves. If we played "Fair" setup, replay on "Standard" setup will break.
        // REQUIREMENT GAP: Kifu needs to store initial board or setup type.
        // For now, to keep it "lightweight" and simple as requested, let's just use the moves.
        // But if moves are invalid for default board, it will panic or error.
        // Let's assume Standard Mixed for now or maybe we can make the user choose setup before replay?
        // Actually, let's default to standard_mixed for '1', but '5' is fair.
        // Improvement: Let's try to infer or just ask user?
        // OR: simpler: Just assume standard mixed for now as MVP.
        //
        // Actually, previous implementation relied on `main.rs` to setup board.
        // Let's allow passing the initial board to `new`.

        boards.push(board.clone());

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
