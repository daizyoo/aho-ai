# Machine Learning with Self-Play Data

Self-play で生成された棋譜データを使って機械学習モデルを訓練するためのガイドです。

## 概要

Self-play で生成されたデータを使って以下のことができます:

1. 次の手を予測するモデル（Move Prediction）
2. 局面評価モデル（Position Evaluation）
3. 勝率予測モデル（Win Rate Prediction）

## セットアップ

### 必要なパッケージ

```bash
# 基本的な分析のみ（標準ライブラリのみ）
python analyze_*.py

# 機械学習を使う場合
pip install tensorflow numpy
```

## ワークフロー

### 1. Self-Play でデータ生成

```bash
# Rustアプリで実行（save_kifus: trueに設定済み）
cargo run --release
# オプション4を選択してSelf-Playを実行
```

生成されるデータ:

- `selfplay_results_*.json` - 統計データ
- `selfplay_kifu/game_*.json` - 個別の棋譜

### 2. データ準備

```bash
# 棋譜を機械学習用データに変換
python ml_prepare_data.py selfplay_kifu selfplay_results_*.json
```

生成されるファイル:

- `training_data.pkl` - 手の予測用データ
- `position_values.pkl` - 局面評価用データ

### 3. モデル訓練

```bash
# 50エポック訓練（デフォルト）
python ml_train_model.py

# カスタムエポック数
python ml_train_model.py 100
```

生成されるファイル:

- `move_prediction_model.h5` - 訓練済みモデル

## データ形式

### Move Encoding (7 次元ベクトル)

各手は以下の特徴量でエンコードされます:

```
[from_x, from_y, to_x, to_y, is_drop, is_promote, piece_type]
```

- `from_x, from_y`: 移動元座標 (0-1 に正規化)
- `to_x, to_y`: 移動先座標 (0-1 に正規化)
- `is_drop`: 駒打ちかどうか (0 or 1)
- `is_promote`: 成るかどうか (0 or 1)
- `piece_type`: 駒の種類 (0-1 に正規化)

### Training Data Structure

```python
{
    'sequences': List[List[np.ndarray]],  # ゲームごとの手のシーケンス
    'labels': List[float],                # ゲーム結果 (1=P1勝, 0=P2勝, 0.5=引分)
    'num_games': int
}
```

## モデルアーキテクチャ

### Move Prediction Model

```
Input (35次元) -> Dense(128, ReLU) -> Dropout(0.2)
-> Dense(128, ReLU) -> Dropout(0.2)
-> Dense(64, ReLU) -> Dense(7, Sigmoid)
```

- **入力**: 過去 5 手のシーケンス (5 × 7 = 35 次元)
- **出力**: 次の手の予測 (7 次元)

## 活用例

### 1. AI 評価関数の改善

訓練したモデルを使って局面評価を行い、現在の AI の評価関数と比較:

```python
# モデルで局面評価
predicted_value = model.predict(position_features)

# 現在のAIの評価
current_eval = alpha_beta_ai.evaluate(board)

# 差分を分析
```

### 2. 開局データベース構築

```bash
# 全棋譜から開局パターンを抽出
python analyze_batch_kifu.py > opening_database.txt
```

### 3. 戦略パターン発見

```python
# 勝ちゲームと負けゲームの手の違いを分析
# 成功パターンを抽出
```

## 今後の拡張

- **強化学習**: Self-play の結果を使って AI を改善
- **AlphaZero 風**: MCTS + Neural Network
- **転移学習**: 既存のチェス/将棋モデルから転移
- **アンサンブル**: 複数モデルの組み合わせ

## パフォーマンス

- **データ量**: 100 ゲーム ≈ 8,000 手 ≈ 十分な初期訓練データ
- **訓練時間**: 50 エポック ≈ 5-10 分 (CPU)
- **推論速度**: 1 手 ≈ 1ms (CPU)

## トラブルシューティング

### TensorFlow がインストールできない

```bash
# 軽量版を使用
pip install tensorflow-cpu
```

### メモリ不足

```python
# バッチサイズを減らす
model.fit(..., batch_size=16)  # デフォルトは32
```

### 精度が低い

- データ量を増やす（1000 ゲーム以上推奨）
- エポック数を増やす
- モデルを複雑化（層を追加）
