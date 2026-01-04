use crate::core::{Board, PlayerId};
use crate::logic::{apply_move, legal_moves};
use crate::player::PlayerController;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerspectiveMode {
    Fixed(PlayerId),
    AutoFlip,
}

pub struct Game {
    pub board: Board,
    pub current_player: PlayerId,
    pub board_sync_rx: Option<std::sync::mpsc::Receiver<Board>>,
    pub perspective_mode: PerspectiveMode,
}

impl Game {
    pub fn new(board: Board) -> Self {
        Game {
            board,
            current_player: PlayerId::Player1,
            board_sync_rx: None,
            perspective_mode: PerspectiveMode::AutoFlip,
        }
    }

    pub fn play<F>(&mut self, p1: &dyn PlayerController, p2: &dyn PlayerController, mut on_move: F)
    where
        F: FnMut(&crate::core::Move),
    {
        loop {
            // 外部（ネットワーク等）からの盤面更新があれば反映
            if let Some(ref rx) = self.board_sync_rx {
                while let Ok(new_board) = rx.try_recv() {
                    self.board = new_board;
                }
            }

            // 現状をまず描画 (リモートプレイヤーも待機画面が見えるように)
            let mut state = crate::display::DisplayState::default();
            state.perspective = match self.perspective_mode {
                PerspectiveMode::Fixed(p) => p,
                PerspectiveMode::AutoFlip => self.current_player,
            };
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
                // 移動適用前の最終同期チェック
                if let Some(ref rx) = self.board_sync_rx {
                    while let Ok(new_board) = rx.try_recv() {
                        self.board = new_board;
                    }
                }

                // ローカルで移動を適用
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
