use crate::core::{Board, Move, PlayerId, Position};
use crate::player::PlayerController;
use crate::ui::display::{render_board, DisplayState};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::cell::RefCell;
use std::time::Duration;

pub struct TuiController {
    player_id: PlayerId,
    name: String,
    last_cursor: RefCell<Option<Position>>,
}

impl TuiController {
    pub fn new(player_id: PlayerId, name: &str) -> Self {
        Self {
            player_id,
            name: name.to_string(),
            last_cursor: RefCell::new(None),
        }
    }
}

impl PlayerController for TuiController {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_local(&self) -> bool {
        true
    }

    fn choose_move(&self, board: &Board, legal_moves_list: &[Move]) -> Option<Move> {
        let mut state = DisplayState {
            perspective: self.player_id,
            last_move: board.last_move.clone(),
            status_msg: Some(format!("{}'s turn ({:?})", self.name, self.player_id)),
            ..Default::default()
        };

        // 前回位置があれば復元、なければキングに合わせる
        let cached_pos = *self.last_cursor.borrow();
        if let Some(pos) = cached_pos {
            state.cursor = pos;
        } else if let Some(king_pos) = board.find_king(self.player_id) {
            state.cursor = king_pos;
            *self.last_cursor.borrow_mut() = Some(king_pos);
        }

        loop {
            // 描画
            render_board(board, &state);
            print!(
                "[Arrows]: Move | [Enter]: Select | [Esc]: Cancel | [p]: Hand | [q]: Resign\r\n"
            );

            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                    match code {
                        KeyCode::Char('q') => return None,
                        KeyCode::Esc => {
                            state.selected = None;
                            state.highlights.clear();
                            state.hand_mode = false;
                        }
                        KeyCode::Up => {
                            if state.perspective == PlayerId::Player1 {
                                if state.cursor.y > 0 {
                                    state.cursor.y -= 1;
                                }
                            } else if state.cursor.y < board.height - 1 {
                                state.cursor.y += 1;
                            }
                            *self.last_cursor.borrow_mut() = Some(state.cursor);
                        }
                        KeyCode::Down => {
                            if state.perspective == PlayerId::Player1 {
                                if state.cursor.y < board.height - 1 {
                                    state.cursor.y += 1;
                                }
                            } else if state.cursor.y > 0 {
                                state.cursor.y -= 1;
                            }
                            *self.last_cursor.borrow_mut() = Some(state.cursor);
                        }
                        KeyCode::Left => {
                            if state.hand_mode {
                                if let Some(hand) = board.hand.get(&self.player_id) {
                                    let items_count = hand.values().filter(|&&c| c > 0).count();
                                    if items_count > 0 {
                                        state.hand_index =
                                            (state.hand_index + items_count - 1) % items_count;
                                    }
                                }
                            } else {
                                if state.perspective == PlayerId::Player1 {
                                    if state.cursor.x > 0 {
                                        state.cursor.x -= 1;
                                    }
                                } else if state.cursor.x < board.width - 1 {
                                    state.cursor.x += 1;
                                }
                                *self.last_cursor.borrow_mut() = Some(state.cursor);
                            }
                        }
                        KeyCode::Right => {
                            if state.hand_mode {
                                if let Some(hand) = board.hand.get(&self.player_id) {
                                    let items_count = hand.values().filter(|&&c| c > 0).count();
                                    if items_count > 0 {
                                        state.hand_index = (state.hand_index + 1) % items_count;
                                    }
                                }
                            } else {
                                if state.perspective == PlayerId::Player1 {
                                    if state.cursor.x < board.width - 1 {
                                        state.cursor.x += 1;
                                    }
                                } else if state.cursor.x > 0 {
                                    state.cursor.x -= 1;
                                }
                                *self.last_cursor.borrow_mut() = Some(state.cursor);
                            }
                        }
                        KeyCode::Char('p') => {
                            if state.hand_mode {
                                state.hand_mode = false;
                            } else if let Some(hand) = board.hand.get(&self.player_id) {
                                if !hand.values().all(|&v| v == 0) {
                                    state.hand_mode = true;
                                    state.hand_index = 0;
                                }
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if state.hand_mode {
                                // 持ち駒の確定
                                if let Some(hand) = board.hand.get(&self.player_id) {
                                    let mut items: Vec<_> =
                                        hand.iter().filter(|(_, &c)| c > 0).collect();
                                    items.sort_by_key(|(k, _)| format!("{:?}", k));
                                    if let Some(&(kind, _)) = items.get(state.hand_index) {
                                        let h_kind = *kind;
                                        state.hand_mode = false;
                                        // 全位置からドロップ可能個所を選択
                                        // 簡易的に現在のカーソル位置に落とす
                                        let mv = Move::Drop {
                                            kind: h_kind,
                                            to: state.cursor,
                                        };
                                        if legal_moves_list.contains(&mv) {
                                            return Some(mv);
                                        }
                                    }
                                }
                            } else if let Some(from) = state.selected {
                                // 移動先確定
                                let to = state.cursor;
                                if from == to {
                                    state.selected = None;
                                    state.highlights.clear();
                                } else {
                                    let found_moves: Vec<Move> = legal_moves_list
                                        .iter()
                                        .filter(|m| match m {
                                            Move::Normal { from: f, to: t, .. } => {
                                                *f == from && *t == to
                                            }
                                            _ => false,
                                        })
                                        .cloned()
                                        .collect();

                                    if !found_moves.is_empty() {
                                        if found_moves.len() > 1 {
                                            // 成り選択
                                            render_board(board, &state);

                                            // Check if we have both promote=Some and promote=None
                                            let has_non_promote = found_moves.iter().any(|m| {
                                                matches!(m, Move::Normal { promote: None, .. })
                                            });

                                            // Create a list of (PromotedKind, Move)
                                            let promote_options: Vec<_> = found_moves
                                                .iter()
                                                .filter_map(|m| {
                                                    if let Move::Normal {
                                                        promote: Some(k), ..
                                                    } = m
                                                    {
                                                        Some((k, m.clone()))
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect();

                                            if !promote_options.is_empty()
                                                && has_non_promote
                                                && promote_options.len() == 1
                                            {
                                                // Shogi-style simple question
                                                println!("\nPromote? [y] Yes / [n] No");
                                                loop {
                                                    if let Event::Key(KeyEvent {
                                                        code: KeyCode::Char(c),
                                                        ..
                                                    }) = event::read().unwrap()
                                                    {
                                                        if c == 'y' {
                                                            return Some(
                                                                promote_options[0].1.clone(),
                                                            );
                                                        } else if c == 'n' {
                                                            return found_moves.into_iter().find(
                                                                |m| {
                                                                    matches!(
                                                                        m,
                                                                        Move::Normal {
                                                                            promote: None,
                                                                            ..
                                                                        }
                                                                    )
                                                                },
                                                            );
                                                        }
                                                    }
                                                }
                                            } else {
                                                // Chess-style: Multiple promotion options OR mandatory choice
                                                println!("\nSelect promotion:");
                                                for (i, (k, _)) in
                                                    promote_options.iter().enumerate()
                                                {
                                                    println!("[{}] Promote to {:?}", i + 1, k);
                                                }
                                                if has_non_promote {
                                                    println!("[0] Don't promote");
                                                }

                                                loop {
                                                    if let Event::Key(KeyEvent {
                                                        code: KeyCode::Char(c),
                                                        ..
                                                    }) = event::read().unwrap()
                                                    {
                                                        if let Some(digit) = c.to_digit(10) {
                                                            if digit == 0 && has_non_promote {
                                                                return found_moves
                                                                    .into_iter()
                                                                    .find(|m| {
                                                                        matches!(
                                                                            m,
                                                                            Move::Normal {
                                                                                promote: None,
                                                                                ..
                                                                            }
                                                                        )
                                                                    });
                                                            }
                                                            if digit > 0
                                                                && (digit as usize)
                                                                    <= promote_options.len()
                                                            {
                                                                return Some(
                                                                    promote_options
                                                                        [digit as usize - 1]
                                                                        .1
                                                                        .clone(),
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        return Some(found_moves[0].clone());
                                    }
                                }
                            } else {
                                // 移動元選択
                                if let Some(piece) = board.get_piece(state.cursor) {
                                    if piece.owner == self.player_id {
                                        state.selected = Some(state.cursor);
                                        state.highlights = legal_moves_list
                                            .iter()
                                            .filter_map(|m| match m {
                                                Move::Normal { from, to, .. }
                                                    if *from == state.cursor =>
                                                {
                                                    Some(*to)
                                                }
                                                _ => None,
                                            })
                                            .collect();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
