# Evaluation Function Improvements

Implementation plan to strengthen the AI's position evaluation and playing ability.

## Background Context

Current evaluation function includes:

- Material counting with piece values
- Piece-Square Tables (PST) for positional bonuses
- Hand piece evaluation (Shogi drops)
- Pawn structure penalties (doubled, isolated)
- King safety (3x3 defender count)

**Self-Play Results** (ShogiOnly, depth=4→6):

- P1 wins: 56%, P2 wins: 44% (slight imbalance)
- Average game length: 64 moves
- No draws (good - games are decisive)

**Identified Weaknesses**:

1. No mobility evaluation (piece activity not considered)
2. No game phase awareness (same eval for opening/endgame)
3. Limited tactical pattern recognition
4. King safety is basic (only counts defenders)

---

## Proposed Improvements

### Priority 1: Mobility Evaluation (High Impact)

**Problem**: Current eval doesn't consider piece activity. A Bishop trapped in corner = Bishop in center.

**Solution**: Add mobility score based on legal moves

```rust
fn calculate_mobility(board: &Board, player: PlayerId) -> i32 {
    let moves = legal_moves(board, player);
    let base_mobility = moves.len() as i32;

    // Weight by move quality
    let mut weighted_mobility = 0;
    for mv in moves {
        match mv {
            Move::Normal { from, to, capture, promote } => {
                // Captures are valuable
                if capture.is_some() {
                    weighted_mobility += 3;
                } else if promote.is_some() {
                    weighted_mobility += 2;
                } else {
                    weighted_mobility += 1;
                }
            }
            Move::Drop { .. } => {
                weighted_mobility += 1;  // Drops are flexibility
            }
        }
    }

    // Scale down to ~50-200 CP range
    (weighted_mobility * 2).min(200)
}
```

**Expected Impact**: +100-150 Elo (major improvement)

---

### Priority 2: Game Phase Detection (Medium Impact)

**Problem**: Opening and endgame use same evaluation logic. King should be safe in opening, active in endgame.

**Solution**: Detect game phase by material count

```rust
#[derive(Debug, Clone, Copy)]
enum GamePhase {
    Opening,   // Material > 8000 (most pieces alive)
    Midgame,   // Material 4000-8000
    Endgame,   // Material < 4000 (few pieces left)
}

fn detect_game_phase(board: &Board) -> GamePhase {
    let total_material = count_total_material(board);

    if total_material > 8000 {
        GamePhase::Opening
    } else if total_material > 4000 {
        GamePhase::Midgame
    } else {
        GamePhase::Endgame
    }
}

// Adjust evaluation based on phase
fn phase_adjusted_eval(base_eval: i32, phase: GamePhase, board: &Board) -> i32 {
    match phase {
        GamePhase::Opening => {
            // Emphasize development, king safety
            base_eval + king_safety_bonus * 2 - undeveloped_piece_penalty
        }
        GamePhase::Midgame => {
            // Balanced, as is
            base_eval
        }
        GamePhase::Endgame => {
            // King activity, passed pawns matter more
            base_eval + king_activity_bonus + passed_pawn_bonus
        }
    }
}
```

**Expected Impact**: +50-100 Elo

---

### Priority 3: Enhanced King Safety (Medium Impact)

**Problem**: Only counts defenders in 3x3 area. Doesn't consider:

- Escape squares (mobility for king)
- Pawn shield quality
- Attacking pieces near king

**Solution**: Multi-factor king safety

```rust
fn enhanced_king_safety(board: &Board, king_pos: Position, owner: PlayerId, phase: GamePhase) -> i32 {
    let mut safety = 0;

    // 1. Defender count (existing)
    safety += count_defenders(board, king_pos, owner) * 40;

    // 2. Escape squares
    let escape_count = count_escape_squares(board, king_pos, owner);
    safety += escape_count * 15;  // Mobility for king

    // 3. Pawn shield (pieces in front of king)
    let shield = count_pawn_shield(board, king_pos, owner);
    safety += shield * 25;

    // 4. Attacker penalty
    let attackers = count_enemy_attacks_near(board, king_pos, owner);
    safety -= attackers * 30;

    // Phase adjustment: Safety less critical in endgame
    match phase {
        GamePhase::Opening | GamePhase::Midgame => safety,
        GamePhase::Endgame => safety / 2,  // Reduce weight
    }
}
```

**Expected Impact**: +30-50 Elo

---

### Priority 4: Tactical Patterns (Low-Medium Impact)

** Problem**: No recognition of common tactical motifs

**Solution**: Basic pattern detection

```rust
fn detect_tactical_patterns(board: &Board, player: PlayerId) -> i32 {
    let mut bonus = 0;

    // 1. Piece trades when ahead
    if is_material_ahead(board, player) {
        bonus += 20;  // Minor incentive to trade
    }

    // 2. Passed pawns (no enemy pawns blocking path)
    let passed_pawns = count_passed_pawns(board, player);
    bonus += passed_pawns * 50;

    // 3. Bishop pair bonus
    if has_bishop_pair(board, player) {
        bonus += 30;  // Strong in open positions
    }

    // 4. Rook on open file
    let rooks_on_open = count_rooks_on_open_files(board, player);
    bonus += rooks_on_open * 40;

    bonus
}
```

**Expected Impact**: +20-40 Elo

---

### Priority 5: Tempo & Development (Low Impact, Opening Only)

**Problem**: No incentive to develop pieces in opening

**Solution**: Penalize pieces on starting squares in opening

```rust
fn development_score(board: &Board, player: PlayerId, phase: GamePhase) -> i32 {
    if !matches!(phase, GamePhase::Opening) {
        return 0;  // Only relevant in opening
    }

    let mut undeveloped = 0;

    // Check if pieces are still on starting ranks
    let start_rank = if player == PlayerId::Player1 { 8 } else { 0 };

    for (&pos, piece) in &board.pieces {
        if piece.owner == player && pos.y == start_rank {
            // Major pieces undeveloped
            if matches!(
                piece.kind,
                PieceKind::S_Bishop
                    | PieceKind::C_Bishop
                    | PieceKind::S_Rook
                    | PieceKind::C_Rook
                    | PieceKind::C_Knight
            ) {
                undeveloped += 1;
            }
        }
    }

    -undeveloped * 10  // Small penalty
}
```

**Expected Impact**: +10-20 Elo

---

## Updated `evaluate()` Function Structure

```rust
pub fn evaluate(board: &Board) -> i32 {
    let config = AIConfig::get();
    let mut score = 0;

    // === 1. Game Phase Detection ===
    let phase = detect_game_phase(board);

    // === 2. Material & PST (Existing) ===
    score += evaluate_material_and_pst(board);

    // === 3. Hand Pieces (Existing) ===
    score += evaluate_hand_pieces(board, config.evaluation.hand_piece_bonus_multiplier);

    // === 4. Pawn Structure (Existing) ===
    score += evaluate_pawn_structure(board);

    // === 5. King Safety (Enhanced) ===
    score += evaluate_king_safety_enhanced(board, phase);

    // === 6. NEW: Mobility ===
    let p1_mobility = calculate_mobility(board, PlayerId::Player1);
    let p2_mobility = calculate_mobility(board, PlayerId::Player2);
    score += p1_mobility - p2_mobility;

    // === 7. NEW: Tactical Patterns ===
    let p1_tactical = detect_tactical_patterns(board, PlayerId::Player1);
    let p2_tactical = detect_tactical_patterns(board, PlayerId::Player2);
    score += p1_tactical - p2_tactical;

    // === 8. NEW: Development (Opening only) ===
    let p1_dev = development_score(board, PlayerId::Player1, phase);
    let p2_dev = development_score(board, PlayerId::Player2, phase);
    score += p1_dev - p2_dev;

    score
}
```

---

## Implementation Order

**Phase 1**: Core Enhancements (Most Impact)

1. ✅ Add `calculate_mobility()`
2. ✅ Add `detect_game_phase()`
3. ✅ Integrate into main `evaluate()`

**Phase 2**: Safety & Tactics

1. ✅ Enhance `enhanced_king_safety()`
2. ✅ Add passed pawn detection
3. ✅ Add bishop pair / rook on open file

**Phase 3**: Polish

1. ✅ Add development scoring
2. ✅ Tune weights based on self-play
3. ✅ Document changes

---

## Testing Plan

### Baseline Measurement

```bash
# Run 100 games with OLD evaluator
cargo run --release -- selfplay --num-games 100 --board ShogiOnly

# Record: P1 wins, P2 wins, avg moves
```

### Test New Evaluator

```bash
# Run 100 games with NEW evaluator
cargo run --release -- selfplay --num-games 100 --board ShogiOnly

# Compare: Win rates, game lengths
```

### Head-to-Head

```bash
# Old evaluator vs New evaluator (50 games each side)
# This gives true Elo difference estimate
```

**Expected Results**:

- New evaluator should win 60-70% vs old
- Games may be slightly longer (better play = more complex positions)
- More decisive games (better evaluation = clearer advantages)

---

## Weight Tuning

After initial implementation, tune these weights:

| Component             | Initial Weight | Tunable Range |
| --------------------- | -------------- | ------------- |
| Mobility              | 2 per move     | 1-4           |
| King Safety (Opening) | 1x             | 0.5x-2x       |
| King Safety (Endgame) | 0.5x           | 0.25x-1x      |
| Passed Pawn           | 50 CP          | 30-100        |
| Bishop Pair           | 30 CP          | 20-50         |
| Development Penalty   | -10 CP         | -20 to -5     |

Use self-play results to iterate.

---

## Risk Mitigation

> [!WARNING] > **Performance Risk**: Adding mobility calculation means calling `legal_moves()` twice per evaluation.
>
> **Mitigation**:
>
> - Cache legal moves if called multiple times in same position
> - Use simplified mobility (pseudo-legal moves) for speed
> - Profile before/after to ensure < 20% slowdown

> [!IMPORTANT] > **Balance Risk**: Too much weight on one component can create imbalanced play.
>
> **Mitigation**:
>
> - Start with conservative weights
> - Test each component individually
> - Revert if win rate drops

---

## Success Criteria

✅ **Minimum** (Phase 1 only):

- New evaluator wins ≥ 55% vs old
- Average game quality improves (fewer blunders in position eval)

✅ **Target** (All phases):

- New evaluator wins ≥ 65% vs old (~100 Elo gain)
- More varied tactical play
- Better opening development

✅ **Stretch**:

- New evaluator wins ≥ 75% vs old (~200 Elo gain)
- Can beat old evaluator at lower search depth (depth=4 new vs depth=6 old)

---

## Files to Modify

### Core Implementation

- [src/player/ai/eval.rs](file:///Users/daizyoo/Rust/shogi-aho-ai/src/player/ai/eval.rs)
  - Add mobility, phase detection, tactical patterns
  - Refactor into helper functions

### Optional Enhancements

- [src/player/ai/pst.rs](file:///Users/daizyoo/Rust/shogi-aho-ai/src/player/ai/pst.rs)
  - Add endgame-specific PST tables (if phase-dependent)

### Configuration

- [src/player/ai/config.rs](file:///Users/daizyoo/Rust/shogi-aho-ai/src/player/ai/config.rs)
  - Add evaluation weights for tuning

---

## Estimated Impact Summary

| Improvement          | Implementation Effort | Expected Elo Gain |
| -------------------- | --------------------- | ----------------- |
| Mobility             | 2-3 hours             | +100-150          |
| Game Phase           | 1-2 hours             | +50-100           |
| Enhanced King Safety | 2-3 hours             | +30-50            |
| Tactical Patterns    | 2-4 hours             | +20-40            |
| Development          | 1 hour                | +10-20            |
| **Total**            | **8-13 hours**        | **+210-360 Elo**  |

This represents a **significant strengthening** of the AI's positional understanding.
