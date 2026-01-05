use crate::core::Move;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Bound {
    Exact,
    Lower, // Beta cut (これ以上の値があるかもしれない)
    Upper, // Alpha cut (これ以下の値しかない)
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: usize,
    pub score: i32,
    pub bound: Bound,
    // MoveはCopyではない（Stringなどを含まないが、構造体定義による）
    // Copyを実装していない場合、TTに保存するのは難しい。
    // しかし Move は Clone 実装済み。
    // ここではリファレンスではなく実際のデータを持ちたいが、
    // RustのMove定義を確認すると、Drop { kind, to } などで String は無い。
    // core/move.rs を見ると Move は Clone, PartialEq, Eq はあるが Copy はない。
    // 省メモリ化のため、Option<Move> ではなく index 等で持ちたいが、
    // ここでは簡易的に Option<Move> を持つ。Clone コストは許容する。
    // (実際には AlphaBetaAI 内で Clone して使う)
}

// Move を TTEntry に含めるためのラッパー。実際には Clone して使う。
// 64MB 程度のサイズを確保したい。
// Entryサイズが大きくなるとキャッシュ効率が落ちるが、今回は簡易実装優先。

pub struct TranspositionTable {
    entries: Vec<Option<(TTEntry, Option<Move>)>>,
    size: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        // Entryのおおよそのサイズ: u64 + usize + i32 + Bound(1) + Option<Move>(~32?) ~ 64bytes
        // 1MB = 1024 * 1024 bytes
        // 要素数 = (size_mb * 1024 * 1024) / 64
        // あくまで概算。
        let num_entries = (size_mb * 1024 * 1024) / 80; // 安全側に倒して少し大きめに見積もる
        Self {
            entries: std::iter::repeat_n(None, num_entries).collect(),
            size: num_entries,
        }
    }

    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = None;
        }
    }

    pub fn get(&self, hash: u64) -> Option<(TTEntry, Option<Move>)> {
        let idx = (hash as usize) % self.size;
        if let Some((entry, mv)) = &self.entries[idx] {
            if entry.hash == hash {
                return Some((*entry, mv.clone()));
            }
        }
        None
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: usize,
        score: i32,
        bound: Bound,
        best_move: Option<Move>,
    ) {
        let idx = (hash as usize) % self.size;

        // 既存のエントリと衝突した場合の置換戦略（Deepest優先）
        if let Some((entry, _)) = &self.entries[idx] {
            if entry.hash != hash {
                // 衝突: 深さが深い方を優先、または新しい方を優先
                // ここでは「深さが同じか深い」場合、または「ハッシュが違う（上書き）」場合に保存
                if depth >= entry.depth {
                    // overwrite
                } else {
                    // keep existing (don't overwrite deep results with shallow ones)
                    return;
                }
            } else {
                // 同じハッシュ: 深さが深ければ更新
                if depth < entry.depth {
                    return;
                }
            }
        }

        self.entries[idx] = Some((
            TTEntry {
                hash,
                depth,
                score,
                bound,
            },
            best_move,
        ));
    }
}
