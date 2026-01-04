use crate::core::{Board, PlayerId, Position};
use crossterm::{cursor, execute, style::Stylize, terminal};
use std::io::stdout;

pub struct DisplayState {
    pub cursor: Position,
    pub selected: Option<Position>,
    pub highlights: Vec<Position>,
    pub status_msg: Option<String>,
    pub hand_mode: bool,
    pub hand_index: usize,
    pub last_move: Option<crate::core::Move>,
    pub perspective: PlayerId,
    pub show_cursor: bool,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            cursor: Position::default(),
            selected: None,
            highlights: Vec::new(),
            status_msg: None,
            hand_mode: false,
            hand_index: 0,
            last_move: None,
            perspective: PlayerId::default(),
            show_cursor: true, // Default to showing cursor
        }
    }
}

impl DisplayState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn render_board(board: &Board, state: &DisplayState) {
    let mut out = stdout();

    // 画面クリア（スクロール防止）
    execute!(
        out,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    print!("=== Unified Board Game Engine ===\r\n");
    if let Some(msg) = &state.status_msg {
        print!("{}\r\n", msg.clone().bold().yellow());
    } else {
        print!("\r\n");
    }
    print!("\r\n");

    // X軸ラベル
    print!("    ");
    for i in 0..board.width {
        let x = if state.perspective == PlayerId::Player1 {
            i
        } else {
            board.width - 1 - i
        };
        print!("  {} ", x + 1);
    }
    print!("\r\n");

    print!("   +{}+\r\n", "----".repeat(board.width));

    for i in 0..board.height {
        let y = if state.perspective == PlayerId::Player1 {
            i
        } else {
            board.height - 1 - i
        };

        // --- Line 1: Piece Content ---
        print!("{:2} |", y + 1);
        for j in 0..board.width {
            let x = if state.perspective == PlayerId::Player1 {
                j
            } else {
                board.width - 1 - j
            };

            let pos = Position::new(x, y);
            let piece = board.get_piece(pos);
            // ... (rest of the logic remains same, but using x, y derived from perspective)
            let is_cursor = state.show_cursor && state.cursor == pos && !state.hand_mode;
            let is_selected = state.selected == Some(pos);
            let is_highlight = state.highlights.contains(&pos);

            let char_str = if let Some(p) = piece {
                p.display_char().to_string()
            } else {
                ".".to_string()
            };

            let is_shogi = piece.map(|p| p.is_shogi).unwrap_or(false);

            let is_last_move = if let Some(mv) = &state.last_move {
                match mv {
                    crate::core::Move::Normal { from, to, .. } => *from == pos || *to == pos,
                    crate::core::Move::Drop { to, .. } => *to == pos,
                }
            } else {
                false
            };

            let (prefix, suffix) = if is_cursor {
                ("[", "]")
            } else if is_selected {
                ("|", "|")
            } else if is_highlight {
                ("(", ")")
            } else if is_last_move {
                ("{", "}")
            } else {
                (" ", " ")
            };

            let cell_text = if is_shogi {
                format!("{}{}{}", prefix, char_str, suffix)
            } else {
                format!("{} {}{}", prefix, char_str, suffix)
            };

            if is_cursor {
                print!("{}", cell_text.yellow());
            } else if is_selected {
                print!("{}", cell_text.blue());
            } else if is_highlight {
                print!("{}", cell_text.green());
            } else if is_last_move {
                print!("{}", cell_text.red());
            } else if let Some(p) = piece {
                if p.owner == PlayerId::Player1 {
                    print!("{}", cell_text.cyan());
                } else {
                    print!("{}", cell_text.magenta());
                }
            } else {
                print!("{}", cell_text);
            }
        }
        print!("|\r\n");

        // --- Line 2: Vertical Padding ---
        if i < board.height - 1 {
            print!("   |");
            for _ in 0..board.width {
                print!("    ");
            }
            print!("|\r\n");
        }
    }
    print!("   +{}+\r\n", "----".repeat(board.width));

    // 持ち駒表示
    render_hands(board, state);
}

fn render_hands(board: &Board, state: &DisplayState) {
    for player in [PlayerId::Player1, PlayerId::Player2] {
        print!("{:?} Hand:\r\n", player);
        if let Some(hand) = board.hand.get(&player) {
            let mut items: Vec<_> = hand.iter().filter(|(_, &c)| c > 0).collect();
            items.sort_by_key(|(k, _)| format!("{:?}", k));

            if items.is_empty() {
                print!("  (None)\r\n");
            } else {
                print!("  ");
                for (i, (&kind, &count)) in items.iter().enumerate() {
                    let is_selected_hand =
                        state.hand_mode && player == state.perspective && state.hand_index == i;

                    let char_str = kind.display_char();
                    let label = format!("[{} x{}]", char_str, count);

                    if is_selected_hand {
                        print!("{} ", label.black().on_yellow());
                    } else if player == PlayerId::Player1 {
                        print!("{} ", label.cyan());
                    } else {
                        print!("{} ", label.magenta());
                    }
                }
                print!("\r\n");
            }
        }
        print!("\r\n");
    }
}
