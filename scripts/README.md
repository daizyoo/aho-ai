# ML Improvement Scripts

このディレクトリには、Self-play データを使って AI を改善するためのスクリプトが含まれています。

## ディレクトリ構成

```
scripts/
├── phase1_analyze.py       # Phase 1: 特徴量分析
├── phase2_optimize_pst.py  # Phase 2: PST最適化（100ゲーム後）
├── phase3_train_nn.py      # Phase 3: NN訓練（1000ゲーム後）
└── analysis_results.json   # 分析結果（自動生成）
```

## Phase 1: 特徴量分析（現在のフェーズ）

### 目的

Self-play の結果から勝ちパターンを分析し、評価関数の改善点を提案

### 使い方

```bash
# 分析実行
python3 scripts/phase1_analyze.py
```

### 出力例

```
FEATURE ANALYSIS RESULTS
========================================
Total Games: 5
P1 Wins: 5 (100.0%)
P2 Wins: 0 (0.0%)

Drop Rate:
  P1 Winners: 0.148 (±0.052)
  P2 Winners: 0.000 (±0.000)
  Difference: 0.148
  ⚠️  HIGH IMPORTANCE

RECOMMENDED CHANGES TO eval.rs
========================================
1. Hand Piece Value
   Current: Hand pieces valued at piece_value * 1.1
   Suggest: Increase hand piece bonus
   Code: eval.rs: Change multiplier from 1.1 to 1.2
```

### 次のステップ

1. `src/player/ai/eval.rs`を推奨に従って修正
2. 新しい Self-Play で効果を検証
3. データを 100 ゲーム以上に増やす

---

## Phase 2: PST 最適化（100 ゲーム後に実行）

### 目的

勝ちゲームでの駒の位置分布を分析し、Piece-Square Table を最適化

### 使い方

```bash
# 100ゲーム以上のデータが必要
python3 scripts/phase2_optimize_pst.py
```

### 出力

- `src/player/ai/pst_optimized.rs` - 最適化された PST

---

## Phase 3: NN 訓練（1000 ゲーム後に実行）

### 目的

ニューラルネットワークで局面評価を学習

### 必要なパッケージ

```bash
pip install tensorflow numpy
```

### 使い方

```bash
# 1000ゲーム以上のデータが必要
python3 scripts/phase3_train_nn.py
```

### 出力

- `model.onnx` - 訓練済みモデル

---

## ワークフロー

```
1. Self-Play実行 (Rust)
   ↓
2. Phase 1: 特徴量分析 (Python)
   ↓
3. eval.rs 更新 (Rust)
   ↓
4. 検証 Self-Play (Rust)
   ↓
5. データ100ゲーム達成
   ↓
6. Phase 2: PST最適化 (Python)
   ↓
7. pst.rs 更新 (Rust)
   ↓
8. データ1000ゲーム達成
   ↓
9. Phase 3: NN訓練 (Python)
   ↓
10. NN推論統合 (Rust)
```

---

## トラブルシューティング

### データが見つからない

```bash
# Self-Playを実行
cargo run --release
# オプション4を選択
```

### 分析結果が不安定

- データ量を増やす（最低 10 ゲーム、推奨 100 ゲーム）
- 異なる AI 設定で対戦させる

### Python エラー

```bash
# Python 3.6以上が必要
python3 --version

# 標準ライブラリのみ使用（Phase 1, 2）
# Phase 3のみTensorFlowが必要
```
