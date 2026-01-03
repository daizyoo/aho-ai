use crate::core::{Board, PlayerId, Position};
use crossterm::{cursor, execute, style::Stylize, terminal};
use std::io::stdout;

#[derive(Default)]
pub struct DisplayState {
    pub cursor: Position,
    pub selected: Option<Position>,
    pub highlights: Vec<Position>,
    pub status_msg: Option<String>,
    pub hand_mode: bool,
    pub hand_index: usize,
    pub last_move: Option<crate::core::Move>,
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
    for x in 0..board.width {
        print!("  {} ", x + 1);
    }
    print!("\r\n");

    print!("   +{}+\r\n", "----".repeat(board.width));

    for y in 0..board.height {
        // --- Line 1: Piece Content ---
        print!("{:2} |", y + 1);
        for x in 0..board.width {
            let pos = Position::new(x, y);
            let piece = board.get_piece(pos);

            let is_cursor = state.cursor == pos && !state.hand_mode;
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

            // 描画文字列の組み立て ([ ], ( ), or " ")
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

            // 色付け
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

        // --- Line 2: Vertical Padding for Square Ratio (skip for last row) ---
        if y < board.height - 1 {
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
                    let is_selected_hand = state.hand_mode
                        && player == PlayerId::Player1 // Currently only P1 supports interactive hand selection
                        && state.hand_index == i;

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
