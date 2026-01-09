/// Static Exchange Evaluation (SEE)
///
/// Evaluates the material outcome of a capture sequence at a given square.
/// Returns the expected material gain/loss from the perspective of the moving player.
///
/// Example:
/// - Pawn takes Bishop defended by Rook: pawn(100) captures bishop(800)
///   then rook takes pawn → net: -100 (lose pawn, gain bishop, lose bishop-equivalent)
///   Actually: +800 (bishop) - 100 (pawn lost) = +700
pub fn static_exchange_eval(
    board: &crate::core::Board,
    mv: &crate::core::Move,
    player: crate::core::PlayerId,
) -> i32 {
    use crate::core::Move;
    use crate::player::ai::eval::piece_val;

    // Only evaluate capture moves
    let (from, to, _promote) = match mv {
        Move::Normal { from, to, promote } => (from, to, promote),
        Move::Drop { .. } => return 0, // Drops don't have SEE
    };

    // Get the captured piece value
    let captured_piece = match board.get_piece(*to) {
        Some(p) if p.owner != player => piece_val(p.kind),
        _ => return 0, // No capture
    };

    // Get the attacking piece value
    let attacker = match board.get_piece(*from) {
        Some(p) => piece_val(p.kind),
        None => return 0,
    };

    // Simulate the exchange sequence
    let mut gain = vec![captured_piece];
    let mut current_side = player.opponent();
    let mut piece_value = attacker;

    // Simplified SEE: just check if we lose the attacker
    // Full SEE would simulate the entire exchange sequence

    // For now, simple heuristic:
    // gain[0] = value of captured piece
    // If we capture and can be recaptured, we might lose our piece

    // Check if the square is defended
    let defenders = count_defenders_simple(board, *to, current_side);

    if defenders > 0 {
        // We will likely lose our piece
        gain.push(-piece_value);
    }

    // Net gain from player's perspective
    gain.iter().sum()
}

/// Simple defender count (without expensive move generation)
fn count_defenders_simple(
    board: &crate::core::Board,
    target: crate::core::Position,
    defender: crate::core::PlayerId,
) -> usize {
    // Simplified: just count pieces in attacking range
    let mut count = 0;

    for (&pos, piece) in &board.pieces {
        if piece.owner != defender {
            continue;
        }

        // Simple distance check (not accurate but fast)
        let dx = (pos.x as i32 - target.x as i32).abs();
        let dy = (pos.y as i32 - target.y as i32).abs();

        // If within reasonable range, might be a defender
        if dx <= 2 && dy <= 2 {
            count += 1;
        }
    }

    count
}
