use crate::core::{Board, Move, PlayerId};
use crate::display::{render_board, DisplayState};
use crate::player::PlayerController;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

pub struct TuiController {
    player_id: PlayerId,
    name: String,
}

impl TuiController {
    pub fn new(player_id: PlayerId, name: &str) -> Self {
        Self {
            player_id,
            name: name.to_string(),
        }
    }
}

impl PlayerController for TuiController {
    fn name(&self) -> &str {
        &self.name
    }

    fn choose_move(&self, board: &Board, legal_moves_list: &[Move]) -> Option<Move> {
        let mut state = DisplayState::default();
        state.last_move = board.last_move.clone();
        state.status_msg = Some(format!("{}'s turn ({:?})", self.name, self.player_id));

        // 初期カーソル位置をキングに合わせる
        if let Some(king_pos) = board.find_king(self.player_id) {
            state.cursor = king_pos;
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
                            if state.cursor.y > 0 {
                                state.cursor.y -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if state.cursor.y < board.height - 1 {
                                state.cursor.y += 1;
                            }
                        }
                        KeyCode::Left => {
                            if state.cursor.x > 0 {
                                state.cursor.x -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if state.cursor.x < board.width - 1 {
                                state.cursor.x += 1;
                            }
                        }
                        KeyCode::Char('p') => {
                            if state.hand_mode {
                                state.hand_mode = false;
                            } else {
                                if let Some(hand) = board.hand.get(&self.player_id) {
                                    if !hand.values().all(|&v| v == 0) {
                                        state.hand_mode = true;
                                        state.hand_index = 0;
                                    }
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
                                    let found_moves: Vec<_> = legal_moves_list
                                        .iter()
                                        .filter(|m| match m {
                                            Move::Normal { from: f, to: t, .. } => {
                                                *f == from && *t == to
                                            }
                                            _ => false,
                                        })
                                        .collect();

                                    if !found_moves.is_empty() {
                                        if found_moves.len() > 1 {
                                            // 成り選択
                                            render_board(board, &state);
                                            println!("\nPromote? [y] Yes / [n] No");
                                            loop {
                                                if let Event::Key(KeyEvent {
                                                    code: KeyCode::Char(c),
                                                    ..
                                                }) = event::read().unwrap()
                                                {
                                                    if c == 'y' {
                                                        return found_moves
                                                            .iter()
                                                            .find(|m| {
                                                                matches!(
                                                                    m,
                                                                    Move::Normal {
                                                                        promote: true,
                                                                        ..
                                                                    }
                                                                )
                                                            })
                                                            .cloned()
                                                            .cloned();
                                                    } else if c == 'n' {
                                                        return found_moves
                                                            .iter()
                                                            .find(|m| {
                                                                matches!(
                                                                    m,
                                                                    Move::Normal {
                                                                        promote: false,
                                                                        ..
                                                                    }
                                                                )
                                                            })
                                                            .cloned()
                                                            .cloned();
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
