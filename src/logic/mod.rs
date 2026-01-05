use crate::core::{Board, Move, MoveStep, Piece, PieceKind, PlayerId, Position};

/// 合法手生成 (自殺手を排除)
pub fn legal_moves(board: &Board, player: PlayerId) -> Vec<Move> {
    let pseudo = pseudo_legal_moves(board, player);
    pseudo
        .into_iter()
        .filter(|mv| {
            let next_board = apply_move(board, mv, player);
            !is_in_check(&next_board, player)
        })
        .collect()
}

/// 疑似合法手生成 (王手放置などは考慮しない)
pub fn pseudo_legal_moves(board: &Board, player: PlayerId) -> Vec<Move> {
    let mut moves = Vec::new();
    let config = board.get_player_config(player);

    for (&pos, piece) in board.pieces.iter() {
        if piece.owner == player {
            moves.extend(get_piece_moves(board, pos, piece));
        }
    }

    if config.can_drop {
        if let Some(hand) = board.hand.get(&player) {
            for (&kind, &count) in hand.iter() {
                if count > 0 {
                    for y in 0..board.height {
                        for x in 0..board.width {
                            let to = Position::new(x, y);
                            if board.get_piece(to).is_none() {
                                if kind == PieceKind::S_Pawn && has_pawn_in_column(board, player, x)
                                {
                                    continue;
                                }
                                moves.push(Move::Drop { kind, to });
                            }
                        }
                    }
                }
            }
        }
    }

    moves
}

/// 王が取られる状態か判定
pub fn is_in_check(board: &Board, player: PlayerId) -> bool {
    // 自玉の位置を探す
    let king_pos = board.pieces.iter().find_map(|(&pos, p)| {
        if p.owner == player && matches!(p.kind, PieceKind::S_King | PieceKind::C_King) {
            Some(pos)
        } else {
            None
        }
    });

    let king_pos = match king_pos {
        Some(pos) => pos,
        None => return false, // 王がいない（既に取られた）場合はチェックではないとする（通常は起こらない）
    };

    // 相手の全駒の「疑似合法手」で玉が取られるか確認
    let opponent = if player == PlayerId::Player1 {
        PlayerId::Player2
    } else {
        PlayerId::Player1
    };

    for (&pos, piece) in board.pieces.iter() {
        if piece.owner == opponent {
            let moves = get_piece_moves(board, pos, piece);
            if moves.iter().any(|mv| match mv {
                Move::Normal { to, .. } => *to == king_pos,
                _ => false,
            }) {
                return true;
            }
        }
    }

    false
}

/// 詰み（または投了状態）か判定
pub fn is_checkmate(board: &Board, player: PlayerId) -> bool {
    is_in_check(board, player) && legal_moves(board, player).is_empty()
}

fn get_piece_moves(board: &Board, from: Position, piece: &Piece) -> Vec<Move> {
    let mut moves = Vec::new();
    let config = board.get_player_config(piece.owner);

    if piece.kind == PieceKind::C_Pawn {
        return get_chess_pawn_moves(board, from, piece);
    }

    for step in piece.movement_rules() {
        match step {
            MoveStep::Step(dx, dy) => {
                if let Some(to) = offset_pos(from, dx, dy, board) {
                    if let Some(target) = board.get_piece(to) {
                        if config.can_capture && target.owner != piece.owner {
                            add_normal_moves(&mut moves, from, to, piece, config.can_promote);
                        }
                    } else {
                        add_normal_moves(&mut moves, from, to, piece, config.can_promote);
                    }
                }
            }
            MoveStep::Slide(dx, dy) => {
                let mut curr = from;
                while let Some(to) = offset_pos(curr, dx, dy, board) {
                    if let Some(target) = board.get_piece(to) {
                        if config.can_capture && target.owner != piece.owner {
                            add_normal_moves(&mut moves, from, to, piece, config.can_promote);
                        }
                        break;
                    } else {
                        add_normal_moves(&mut moves, from, to, piece, config.can_promote);
                        curr = to;
                    }
                }
            }
        }
    }
    moves
}

fn add_normal_moves(
    moves: &mut Vec<Move>,
    from: Position,
    to: Position,
    piece: &Piece,
    can_promote: bool,
) {
    if can_promote && piece.promotable_kind().is_some() {
        let is_promotion_zone = if piece.owner == PlayerId::Player1 {
            to.y <= 2
        } else {
            to.y >= 6
        };
        let from_promotion_zone = if piece.owner == PlayerId::Player1 {
            from.y <= 2
        } else {
            from.y >= 6
        };

        if is_promotion_zone || from_promotion_zone {
            moves.push(Move::Normal {
                from,
                to,
                promote: true,
            });
            let must_promote = matches!(piece.kind, PieceKind::S_Pawn | PieceKind::S_Lance)
                && ((piece.owner == PlayerId::Player1 && to.y == 0)
                    || (piece.owner == PlayerId::Player2 && to.y == 8));
            if !must_promote {
                moves.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
            return;
        }
    }
    moves.push(Move::Normal {
        from,
        to,
        promote: false,
    });
}

fn get_chess_pawn_moves(board: &Board, from: Position, piece: &Piece) -> Vec<Move> {
    let mut moves = Vec::new();
    let forward = if piece.owner == PlayerId::Player1 {
        -1
    } else {
        1
    };
    let start_y = if piece.owner == PlayerId::Player1 {
        6
    } else {
        1
    };
    let promo_y = if piece.owner == PlayerId::Player1 {
        0
    } else {
        7
    };

    if let Some(to) = offset_pos(from, 0, forward, board) {
        if board.get_piece(to).is_none() {
            let promote = to.y == promo_y;
            moves.push(Move::Normal { from, to, promote });
            if from.y == start_y {
                if let Some(to2) = offset_pos(from, 0, forward * 2, board) {
                    if board.get_piece(to2).is_none() {
                        moves.push(Move::Normal {
                            from,
                            to: to2,
                            promote: false,
                        });
                    }
                }
            }
        }
    }

    for dx in [-1, 1] {
        if let Some(to) = offset_pos(from, dx, forward, board) {
            if let Some(target) = board.get_piece(to) {
                if target.owner != piece.owner {
                    let promote = to.y == promo_y;
                    moves.push(Move::Normal { from, to, promote });
                }
            }
        }
    }
    moves
}

fn offset_pos(pos: Position, dx: i32, dy: i32, board: &Board) -> Option<Position> {
    let x = pos.x as i32 + dx;
    let y = pos.y as i32 + dy;
    if x >= 0 && x < board.width as i32 && y >= 0 && y < board.height as i32 {
        Some(Position::new(x as usize, y as usize))
    } else {
        None
    }
}

fn has_pawn_in_column(board: &Board, player: PlayerId, x: usize) -> bool {
    for y in 0..board.height {
        if let Some(p) = board.get_piece(Position::new(x, y)) {
            if p.owner == player && p.kind == PieceKind::S_Pawn {
                return true;
            }
        }
    }
    false
}

/// 移動適用
pub fn apply_move(board: &Board, mv: &Move, player: PlayerId) -> Board {
    let mut next = board.clone();
    next.last_move = Some(mv.clone());

    match mv {
        Move::Normal { from, to, promote } => {
            if let Some(mut piece) = next.remove_piece(*from) {
                let config = next.get_player_config(piece.owner);

                if let Some(captured) = next.remove_piece(*to) {
                    if config.keep_captured {
                        next.add_to_hand(piece.owner, captured.unpromoted_kind());
                    }
                }

                if *promote {
                    if let Some(kind) = piece.promotable_kind() {
                        piece.kind = kind;
                    }
                }

                next.place_piece(*to, piece);
            }
        }
        Move::Drop { kind, to } => {
            if next.remove_from_hand(player, *kind) {
                next.place_piece(*to, Piece::new(*kind, player));
            }
        }
    }

    next
}

/// 評価関数
pub fn evaluate(board: &Board, player: PlayerId) -> i32 {
    let mut score = 0;

    // 盤上の駒の評価
    for piece in board.pieces.values() {
        let val = get_piece_value(piece.kind);
        if piece.owner == player {
            score += val;
        } else {
            score -= val;
        }
    }

    // 持ち駒の評価
    for (&p_id, hand) in board.hand.iter() {
        for (&kind, &count) in hand.iter() {
            if count > 0 {
                // 持ち駒は盤上にあるより少し価値を高く見積もる（再配置の柔軟性）
                // ただし王は持ち駒にならないので除外
                let val = (get_piece_value(kind) as f32 * 1.1) as i32;
                if p_id == player {
                    score += val * count as i32;
                } else {
                    score -= val * count as i32;
                }
            }
        }
    }

    score
}

fn get_piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::S_King | PieceKind::C_King => 100000,
        PieceKind::S_Rook | PieceKind::C_Rook => 900,
        PieceKind::S_Bishop | PieceKind::C_Bishop => 800,
        PieceKind::C_Queen => 1000,
        PieceKind::S_Gold
        | PieceKind::S_ProSilver
        | PieceKind::S_ProKnight
        | PieceKind::S_ProLance
        | PieceKind::S_ProPawn => 600,
        PieceKind::S_Silver | PieceKind::C_Knight => 500,
        PieceKind::S_Knight => 400,
        PieceKind::S_Lance => 300,
        PieceKind::S_Pawn | PieceKind::C_Pawn => 100,
        PieceKind::S_ProRook | PieceKind::S_ProBishop => 1100,
    }
}
