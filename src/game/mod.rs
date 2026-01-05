use crate::core::{Board, Move, PlayerId};
use crate::logic::{apply_move, legal_moves};
use crate::player::PlayerController;
use serde::{Deserialize, Serialize};

pub mod replay;

#[derive(Serialize, Deserialize)]
pub struct KifuData {
    pub board_setup: String,
    pub moves: Vec<Move>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerspectiveMode {
    Fixed(PlayerId),
    AutoFlip,
}

pub struct Game {
    pub board: Board,
    pub current_player: PlayerId,
    pub board_sync_rx: Option<std::sync::mpsc::Receiver<(Board, PlayerId)>>,
    pub perspective_mode: PerspectiveMode,
    pub history: Vec<Move>,
    pub board_setup: String,
}

impl Game {
    pub fn new(board: Board) -> Self {
        Game {
            board,
            current_player: PlayerId::Player1,
            board_sync_rx: None,
            perspective_mode: PerspectiveMode::AutoFlip,
            history: Vec::new(),
            board_setup: "Unknown".to_string(),
        }
    }
    
    pub fn with_setup(board: Board, board_setup: String) -> Self {
        Game {
            board,
            current_player: PlayerId::Player1,
            board_sync_rx: None,
            perspective_mode: PerspectiveMode::AutoFlip,
            history: Vec::new(),
            board_setup,
        }
    }

    pub fn play<F>(&mut self, p1: &dyn PlayerController, p2: &dyn PlayerController, mut on_move: F)
    where
        F: FnMut(&crate::core::Move),
    {
        loop {
            // 外部（ネットワーク等）からの盤面・手番更新があれば反映
            if let Some(ref rx) = self.board_sync_rx {
                while let Ok((new_board, next_player)) = rx.try_recv() {
                    self.board = new_board;
                    self.current_player = next_player;
                }
            }

            // 現状をまず描画 (リモートプレイヤーも待機画面が見えるように)
            let mut state = crate::display::DisplayState::new();
            state.perspective = match self.perspective_mode {
                PerspectiveMode::Fixed(p) => p,
                PerspectiveMode::AutoFlip => self.current_player,
            };
            state.last_move = self.board.last_move.clone();

            // AI vs AIの場合はカーソルを表示しない
            let current_controller = match self.current_player {
                PlayerId::Player1 => p1,
                PlayerId::Player2 => p2,
            };
            state.show_cursor =
                current_controller.is_local() && !current_controller.name().contains("AI");

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

                // 思考ウェイト中に終了判定
                let timeout = std::time::Duration::from_millis(100);
                if crossterm::event::poll(timeout).unwrap_or(false) {
                    if let crossterm::event::Event::Key(key) =
                        crossterm::event::read().unwrap_or(crossterm::event::Event::Key(
                            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Null),
                        ))
                    {
                        if key.code == crossterm::event::KeyCode::Char('q') {
                            println!("Interrupted by user.");
                            break;
                        }
                    }
                }
            }

            if let Some(mv) = controller.choose_move(&self.board, &moves) {
                // 移動適用前の最終同期チェック
                if let Some(ref rx) = self.board_sync_rx {
                    while let Ok((new_board, next_player)) = rx.try_recv() {
                        self.board = new_board;
                        self.current_player = next_player;
                    }
                }

                // サーバーからの同期によって手番が変わっていたら、この指し手は無効（または既に適用済み）
                let current_controller = match self.current_player {
                    PlayerId::Player1 => p1,
                    PlayerId::Player2 => p2,
                };
                if !std::ptr::eq(controller, current_controller) {
                    continue;
                }

                // ローカルで移動を適用
                if controller.is_local() {
                    on_move(&mv);
                }
                self.board = apply_move(&self.board, &mv, self.current_player);
                self.history.push(mv.clone());
                self.current_player = self.current_player.opponent();
            } else {
                println!(
                    "Player resigned. {:?} wins!\r",
                    self.current_player.opponent()
                );
                break;
            }
        }

        self.ask_save_kifu();
    }

    fn ask_save_kifu(&self) {
        use std::io::Write;

        // 端末設定を一度戻す（入力のため）
        let _ = crossterm::terminal::disable_raw_mode();

        print!("\r\nSave game record? (y/N) > ");
        let _ = std::io::stdout().flush();

        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);

        if input.trim().eq_ignore_ascii_case("y") {
            // Create kifu directory if it doesn't exist
            let kifu_dir = "kifu";
            if let Err(e) = std::fs::create_dir_all(kifu_dir) {
                println!("Failed to create kifu directory: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(2));
                return;
            }

            let default_name = "game.json";
            print!("Filename (default: {}) > ", default_name);
            let _ = std::io::stdout().flush();

            let mut filename_input = String::new();
            let _ = std::io::stdin().read_line(&mut filename_input);
            let mut filename = filename_input.trim().to_string();

            if filename.is_empty() {
                filename = default_name.to_string();
            }

            // Ensure .json extension
            if !filename.ends_with(".json") {
                filename.push_str(".json");
            }

            // Save to kifu directory
            let filepath = std::path::Path::new(kifu_dir).join(&filename);

            match std::fs::File::create(&filepath) {
                Ok(file) => {
                    let kifu_data = KifuData {
                        board_setup: "Unknown".to_string(), // TODO: Track board setup
                        moves: self.history.clone(),
                    };
                    // Minified JSON (not pretty) to keep it lightweight
                    if let Err(e) = serde_json::to_writer(file, &kifu_data) {
                        println!("Failed to write kifu: {}", e);
                    } else {
                        println!("Kifu saved to {}", filepath.display());
                    }
                }
                Err(e) => println!("Failed to create file: {}", e),
            }
            // Wait user to see message
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        // Re-enable raw mode for next game
        let _ = crossterm::terminal::enable_raw_mode();
    }
}
