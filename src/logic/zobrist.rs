use crate::core::{Board, PieceKind, PlayerId};
use rand::Rng;
use std::sync::OnceLock;

// 定数
const WIDTH: usize = 9;
const HEIGHT: usize = 9;
const PIECE_TYPES: usize = 20; // PieceKindの数
const PLAYERS: usize = 2;

// Zobrist Hash用の乱数テーブル
struct ZobristTable {
    pieces: [[[u64; PIECE_TYPES]; HEIGHT]; WIDTH],
    hand: [[u64; PIECE_TYPES]; PLAYERS],
    side_to_move: u64,
}

static ZOBRIST_TABLE: OnceLock<ZobristTable> = OnceLock::new();

fn get_zobrist_table() -> &'static ZobristTable {
    ZOBRIST_TABLE.get_or_init(|| {
        let mut rng = rand::thread_rng();
        let mut table = ZobristTable {
            pieces: [[[0; PIECE_TYPES]; HEIGHT]; WIDTH],
            hand: [[0; PIECE_TYPES]; PLAYERS],
            side_to_move: rng.gen(),
        };

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                for k in 0..PIECE_TYPES {
                    table.pieces[x][y][k] = rng.gen();
                }
            }
        }

        for p in 0..PLAYERS {
            for k in 0..PIECE_TYPES {
                table.hand[p][k] = rng.gen();
            }
        }

        table
    })
}

fn piece_kind_to_index(kind: PieceKind) -> usize {
    match kind {
        PieceKind::S_King => 0,
        PieceKind::S_Rook => 1,
        PieceKind::S_Bishop => 2,
        PieceKind::S_Gold => 3,
        PieceKind::S_Silver => 4,
        PieceKind::S_Knight => 5,
        PieceKind::S_Lance => 6,
        PieceKind::S_Pawn => 7,
        PieceKind::S_ProRook => 8,
        PieceKind::S_ProBishop => 9,
        PieceKind::S_ProSilver => 10,
        PieceKind::S_ProKnight => 11,
        PieceKind::S_ProLance => 12,
        PieceKind::S_ProPawn => 13,
        PieceKind::C_King => 14,
        PieceKind::C_Queen => 15,
        PieceKind::C_Rook => 16,
        PieceKind::C_Bishop => 17,
        PieceKind::C_Knight => 18,
        PieceKind::C_Pawn => 19,
    }
}

pub struct ZobristHasher;

impl ZobristHasher {
    pub fn compute_hash(board: &Board, current_player: PlayerId) -> u64 {
        let table = get_zobrist_table();
        let mut hash = 0;

        // 盤上の駒
        for (&pos, piece) in board.pieces.iter() {
            let k_idx = piece_kind_to_index(piece.kind);
            // 敵味方の区別が必要だが、PieceKindには所有者情報がないため
            // 自分の駒と相手の駒で別の乱数を使うか、あるいは
            // 所有者を考慮したインデックスにする必要がある。
            // ここでは簡易的に、Player2の駒はZobrist的には「別の場所にある」わけではないので
            // 本来は [Player][Piece][Pos] にするか、 [Piece][Pos] + owner乱数 にする。
            // 修正案: PieceKind自体は共通だが、盤面上の意味はPlayerによって異なる
            // なので、tableを [2][PieceType][Pos] に拡張するのが正しいが、
            // 今回は [PieceType][Pos] しかないので、
            // PlayerId をインデックスに含めるよう拡張する。

            // 修正: ZobristTableは今回 [Width][Height][PieceType] なので、
            // 実際は PlayerId も区別しないとハッシュが衝突してしまう（敵の歩と自分の歩が同じハッシュになる）。
            // そこで、XORする際に PlayerId に応じてビットを反転させるなどの工夫もできるが、
            // 専用の乱数を用意するのが定石。
            // ここでは table.pieces を使うが、Player2の場合はハッシュ値を bit rotate させて使う簡易実装にする。

            let mut val = table.pieces[pos.x][pos.y][k_idx];
            if piece.owner == PlayerId::Player2 {
                val = val.rotate_left(32); // 簡易的な区別
            }
            hash ^= val;
        }

        // 持ち駒
        for (&p_id, hand) in board.hand.iter() {
            let p_idx = if p_id == PlayerId::Player1 { 0 } else { 1 };
            for (&kind, &count) in hand.iter() {
                if count > 0 {
                    let k_idx = piece_kind_to_index(kind);
                    // 持ち駒の個数分だけXORするのは一般的ではない（同じ状態に戻った時にずれる可能性がある）
                    // 本来は個数ごとの乱数を用意するが、
                    // 簡易的に「個数 * 乱数」をXORする（衝突耐性は下がるが）
                    // あるいは Zobrist Hash としては「各個数」に対応する乱数を持つのが正しい。
                    // 今回は簡易版として、count回XORするのではなく、(count as u64).rotate_left(...) をXORする

                    let mut val = table.hand[p_idx][k_idx];
                    // 個数を区別するためにカウント分回す
                    val = val.rotate_left(count as u32);
                    hash ^= val;
                }
            }
        }

        // 手番
        if current_player == PlayerId::Player2 {
            hash ^= table.side_to_move;
        }

        hash
    }
}
