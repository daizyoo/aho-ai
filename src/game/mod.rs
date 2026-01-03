use crate::core::{Board, PlayerId};
use crate::logic::{apply_move, legal_moves};
use crate::player::PlayerController;

pub struct Game {
    pub board: Board,
    pub current_player: PlayerId,
}

impl Game {
    pub fn new(board: Board) -> Self {
        Game {
            board,
            current_player: PlayerId::Player1,
        }
    }

    pub fn play(&mut self, p1: &dyn PlayerController, p2: &dyn PlayerController) {
        loop {
            // 合法手生成
            let moves = legal_moves(&self.board, self.current_player);

            if moves.is_empty() {
                let mut state = crate::display::DisplayState::default();
                state.status_msg = Some(
                    if crate::logic::is_checkmate(&self.board, self.current_player) {
                        format!("Checkmate! {:?} wins!", self.current_player.opponent())
                    } else {
                        format!("No more moves! {:?} wins!", self.current_player.opponent())
                    },
                );
                crate::display::render_board(&self.board, &state);
                std::thread::sleep(std::time::Duration::from_secs(10));
                break;
            }

            let controller = match self.current_player {
                PlayerId::Player1 => p1,
                PlayerId::Player2 => p2,
            };

            if controller.name().contains("AI") {
                let mut state = crate::display::DisplayState::default();
                state.last_move = self.board.last_move.clone();
                let check_msg = if crate::logic::is_in_check(&self.board, self.current_player) {
                    " (CHECK)"
                } else {
                    ""
                };
                state.status_msg = Some(format!(
                    "AI ({:?}) is thinking{}...",
                    self.current_player, check_msg
                ));
                crate::display::render_board(&self.board, &state);
                std::thread::sleep(std::time::Duration::from_millis(600));
            }

            if let Some(mv) = controller.choose_move(&self.board, &moves) {
                self.board = apply_move(&self.board, &mv, self.current_player);
                self.current_player = self.current_player.opponent();
            } else {
                println!(
                    "Player resigned. {:?} wins!\r",
                    self.current_player.opponent()
                );
                break;
            }
        }
    }
}
