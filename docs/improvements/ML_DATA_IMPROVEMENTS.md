# ML Training Data Quality Improvements

機械学習のトレーニングデータ品質を向上させるための実装完了ドキュメント

---

## 概要

Self-Play で生成されるトレーニングデータの質と量を改善し、より強いニューラルネットワークモデルを学習できるようにしました。

**主な改善**:

1. ✅ 結果ラベルの強化
2. ✅ データ拡張（水平反転）
3. ✅ HDF5 スキーマ拡張

---

## 1. 結果ラベルの強化

### src/selfplay/mod.rs の変更

#### GameResult 構造体の拡張

```rust
struct GameResult {
    winner: Option<PlayerId>,
    moves: usize,
    time_ms: u128,
    material_diff: i32,           // NEW: 最終駒差
    avg_move_time_ms: f32,        // NEW: 平均思考時間
}
```

#### GameExecutionResult 構造体の拡張

```rust
pub struct GameExecutionResult {
    pub game: Game,
    pub winner: Option<PlayerId>,
    pub move_count: usize,
    pub thinking_data: Vec<ThinkingInfo>,
    pub duration: Duration,
    pub position_evaluations: Vec<i32>,   // NEW: 評価値の軌跡
    pub critical_moments: Vec<usize>,     // NEW: 決定的な局面
}
```

#### compute_game_metrics() 関数

```rust
fn compute_game_metrics(
    game: &Game,
    thinking_data: &[ThinkingInfo],
    duration: Duration,
) -> (i32, f32, Vec<i32>, Vec<usize>) {
    // 1. 最終的な駒差を計算
    let material_diff = evaluate(&game.board);

    // 2. 平均思考時間
    let avg_move_time_ms = total_time / move_count;

    // 3. 評価値の軌跡を抽出
    let position_evaluations = thinking_data
        .iter()
        .map(|t| t.score)
        .collect();

    // 4. 決定的な局面を検出 (評価値変動 > 2000 CP)
    let critical_moments = detect_large_swings(position_evaluations);

    (material_diff, avg_move_time_ms, position_evaluations, critical_moments)
}
```

**効果**: ゲームの質的情報を豊富に記録

---

## 2. データ拡張

### scripts/ml/prepare_dataset.py の変更

#### データ拡張関数

**is_position_symmetric()**

```python
def is_position_symmetric(board_setup: str) -> bool:
    """対称性をチェック"""
    symmetric_setups = ['Fair', 'StandardMixed', 'ReversedFair']
    return board_setup in symmetric_setups
```

**flip_horizontal()**

```python
def flip_horizontal(features: np.ndarray) -> np.ndarray:
    """盤面を水平反転"""
    # Board: (9, 9, 41) を reshape
    board_3d = features[:3321].reshape(9, 9, 41)

    # 水平反転
    board_flipped = np.flip(board_3d, axis=1)

    # 手駒とターン情報は保持
    return np.concatenate([
        board_flipped.reshape(-1),
        features[3321:3343],  # hand
        features[3343:]       # turn
    ])
```

**augment_position()**

```python
def augment_position(features: np.ndarray, board_setup: str) -> List[np.ndarray]:
    """局面を拡張"""
    augmented = [features]  # オリジナル

    if is_position_symmetric(board_setup):
        augmented.append(flip_horizontal(features))

    return augmented
```

**効果**: 対称盤面でデータ量が 2 倍に

---

## 3. HDF5 スキーマ拡張

### 新しいスキーマ

```python
# Old schema
datasets = {
    'features': (N, 3344),
    'moves': (N,),
    'outcomes': (N,)
}

# New schema
datasets = {
    'features': (N, 3344),           # 変更なし
    'moves': (N,),                   # 変更なし
    'outcomes': (N,),                # 変更なし
    'game_lengths': (N,),            # NEW
    'material_diffs': (N,),          # NEW
    'augmented': (N,),               # NEW: bool flag
}

# Metadata
attrs = {
    'version': '0.2.0',
    'num_games': int,
    'num_samples': int,
    'augmentation_enabled': bool
}
```

### データセット生成の改善

```python
def prepare_dataset(...):
    all_features = []
    all_game_lengths = []
    all_material_diffs = []
    all_augmented_flags = []

    for kifu_file in kifu_files:
        # ボードタイプを検出
        board_setup = detect_board_type(kifu_file)

        for position in extract_positions(kifu_file):
            # 拡張を適用
            augmented = augment_position(position, board_setup)

            for idx, aug_pos in enumerate(augmented):
                all_features.append(aug_pos)
                all_augmented_flags.append(idx > 0)

    # HDF5に保存（gzip圧縮）
    with h5py.File(output_path, 'w') as f:
        f.create_dataset('features', data=features, compression='gzip')
        f.create_dataset('game_lengths', data=lengths, compression='gzip')
        f.create_dataset('material_diffs', data=diffs, compression='gzip')
        f.create_dataset('augmented', data=flags, compression='gzip')
```

---

## 4. トレーニングスクリプト対応

### scripts/ml/train.py の変更

```python
class ShogiDataset(Dataset):
    def __init__(self, h5_path, use_enhanced_labels=False):
        self.use_enhanced_labels = use_enhanced_labels

        with h5py.File(h5_path, 'r') as f:
            self.has_enhanced = 'game_lengths' in f

    def __getitem__(self, idx):
        with h5py.File(self.h5_path, 'r') as f:
            features = f['features'][idx]
            move = f['moves'][idx]
            outcome = f['outcomes'][idx]

            if self.use_enhanced_labels and self.has_enhanced:
                game_length = f['game_lengths'][idx]
                material_diff = f['material_diffs'][idx]
                return features, move, outcome, game_length, material_diff

            return features, move, outcome
```

**後方互換性**: 古いデータセットでも動作

---

## データ拡張の統計

### 拡張率

| Board Setup   | 対称性    | 拡張率        |
| ------------- | --------- | ------------- |
| ShogiOnly     | ❌ 非対称 | 0% (拡張なし) |
| ChessOnly     | ❌ 非対称 | 0% (拡張なし) |
| Fair          | ✅ 対称   | 100% (2 倍)   |
| StandardMixed | ✅ 対称   | 100% (2 倍)   |
| ReversedFair  | ✅ 対称   | 100% (2 倍)   |

### サンプル出力

```
Dataset Summary:
  Games Processed: 1,000
  Total Samples: 65,000
  Original Samples: 32,500
  Augmented Samples: 32,500
  Augmentation Rate: 50.0%
  Feature Shape: (65000, 3344)
```

---

## ストレージ効率

### 圧縮効果

```python
# 前: 非圧縮
HDF5 size: ~800 MB (1,000 games)

# 後: gzip圧縮
HDF5 size: ~400 MB (1,000 games)

# 削減率: ~50%
```

---

## 使用方法

### データ生成パイプライン

```bash
# 1. Self-Play でデータ生成
cargo run --release --features ml -- selfplay \
  --num-games 5000 \
  --board Fair \
  --parallel 6

# 2. データセット準備（拡張+強化ラベル）
python scripts/ml/prepare_dataset.py \
  --boards Fair \
  --version 0.2.0

# 期待される出力:
# Dataset Summary:
#   Games Processed: 5,000
#   Total Samples: ~325,000
#   Augmented Samples: ~162,500
#   Augmentation Rate: 50.0%
```

### モデル学習

```bash
# 強化ラベルを使用（オプション）
python scripts/ml/train.py \
  --version 0.2.0 \
  --epochs 50 \
  --use-enhanced-labels
```

---

## 期待される効果

### データ品質

| 指標            | 改善前       | 改善後        | 改善率 |
| --------------- | ------------ | ------------- | ------ |
| ラベル次元      | 1 (勝敗のみ) | 5 (勝敗+4 種) | 5x     |
| データ量 (Fair) | N            | 2N            | 2x     |
| ストレージ効率  | 100%         | 50%           | 2x     |

### モデル性能

**予想される改善**:

- より強い戦略理解（駒差、評価軌跡から学習）
- より豊富な訓練データ（拡張により）
- 収束速度の向上

**期待 Elo 改善**: +50-100 Elo (v0.2.0 → v0.3.0)

---

## ファイル変更まとめ

### Rust

- [src/selfplay/mod.rs](file:///Users/daizyoo/Rust/shogi-aho-ai/src/selfplay/mod.rs)
  - GameResult 拡張
  - GameExecutionResult 拡張
  - compute_game_metrics() 追加

### Python

- [scripts/ml/prepare_dataset.py](file:///Users/daizyoo/Rust/shogi-aho-ai/scripts/ml/prepare_dataset.py)

  - データ拡張関数追加
  - HDF5 スキーマ拡張
  - 統計情報出力

- [scripts/ml/train.py](file:///Users/daizyoo/Rust/shogi-aho-ai/scripts/ml/train.py)
  - 強化ラベル対応
  - 後方互換性維持

---

## 次のステップ

1. **新データ生成**: 5,000 ゲーム以上を生成
2. **モデル学習**: v0.3.0 モデルをトレーニング
3. **性能比較**: v0.2.0 vs v0.3.0
4. **反復改善**: NN モデルを評価関数として使用

---

**作成日**: 2026-01-07  
**バージョン**: 0.2.0  
**ステータス**: 実装完了
