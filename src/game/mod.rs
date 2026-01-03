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

    pub fn play<F>(&mut self, p1: &dyn PlayerController, p2: &dyn PlayerController, mut on_move: F)
    where
        F: FnMut(&crate::core::Move),
    {
        loop {
            // 現状をまず描画 (リモートプレイヤーも待機画面が見えるように)
            let mut state = crate::display::DisplayState::default();
            state.last_move = self.board.last_move.clone();
            state.status_msg = Some(format!(
                "{}'s turn ({:?})",
                if self.current_player == PlayerId::Player1 {
                    p1.name()
                } else {
                    p2.name()
                },
                self.current_player
            ));
            crate::display::render_board(&self.board, &state);

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
                // AI thinking message used to be here, but we now render at start of turn.
                // Re-rendering with AI specific message if needed.
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
                on_move(&mv);
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
