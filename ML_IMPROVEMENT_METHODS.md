# Self-Play データを使った機械学習・評価関数改善の方法

## 概要

`selfplay_kifu/`と`selfplay_results/`のデータを使って、AI の評価関数を改善する方法をいくつか提案します。

---

## 方法 1: 教師あり学習による局面評価モデル

### 概要

勝ったゲームの局面は「良い」、負けたゲームの局面は「悪い」として学習

### 実装手順

1. **データ準備**

```python
# 勝敗情報を含むデータセット作成
for game_result, kifu in zip(results['games'], kifus):
    winner = game_result['winner']
    for move_idx, move in enumerate(kifu['moves']):
        position = encode_position(move)
        # 勝者側の局面 = +1, 敗者側 = -1
        value = 1 if is_winner_move(move_idx, winner) else -1
        dataset.append((position, value))
```

2. **モデル訓練**

```python
# ニューラルネットワークで局面価値を予測
model = build_value_network()
model.fit(positions, values)
```

3. **Rust への統合**

- 訓練済みモデルを ONNX 形式でエクスポート
- Rust で`tract`クレートを使って推論
- 現在の`eval.rs`の評価値と組み合わせ

### メリット

- 実装が比較的簡単
- 既存の AI の知識を活用

### デメリット

- 局面の良し悪しが勝敗だけでは不正確

---

## 方法 2: 時間差学習（TD Learning）

### 概要

連続する局面の評価値の差から学習（AlphaZero の基礎）

### 実装手順

1. **TD 誤差の計算**

```python
# V(s_t) ≈ r + γ * V(s_{t+1})
for t in range(len(game) - 1):
    current_value = model.predict(position_t)
    next_value = model.predict(position_t+1)
    td_error = reward + gamma * next_value - current_value
    # td_errorを最小化するように学習
```

2. **Self-Play ループ**

```
Self-Play → データ収集 → モデル更新 → Self-Play → ...
```

### メリット

- より正確な評価値学習
- 強化学習の基礎

### デメリット

- 実装が複雑
- 収束に時間がかかる

---

## 方法 3: 特徴量抽出と重み調整

### 概要

勝ちゲームと負けゲームの特徴を分析し、評価関数の重みを調整

### 実装手順

1. **特徴量抽出**

```python
def extract_features(game):
    return {
        'material_advantage': calculate_material(game),
        'king_safety': evaluate_king_safety(game),
        'piece_activity': count_active_pieces(game),
        'center_control': evaluate_center(game),
        'promotion_rate': count_promotions(game),
        'drop_usage': count_drops(game)
    }
```

2. **統計分析**

```python
# 勝ちゲームと負けゲームの特徴を比較
win_features = [extract_features(g) for g in winning_games]
loss_features = [extract_features(g) for g in losing_games]

# 差が大きい特徴を重視
for feature in features:
    importance = abs(mean(win_features[feature]) - mean(loss_features[feature]))
    adjust_weight(feature, importance)
```

3. **eval.rs の更新**

```rust
// 分析結果に基づいて重みを調整
const KING_SAFETY_WEIGHT: i32 = 150;  // 分析で重要と判明
const CENTER_CONTROL_WEIGHT: i32 = 80;
```

### メリット

- 実装が簡単
- 解釈可能
- すぐに効果が出る

### デメリット

- 手動調整が必要
- 局所最適に陥りやすい

---

## 方法 4: Piece-Square Table (PST) の最適化

### 概要

勝ちゲームでの駒の位置分布を分析し、PST を更新

### 実装手順

1. **位置分布の分析**

```python
# 各マスでの勝率を計算
position_stats = {}
for game in winning_games:
    for move in game['moves']:
        if 'Normal' in move:
            pos = move['Normal']['to']
            position_stats[pos] = position_stats.get(pos, 0) + 1

# ヒートマップ生成
heatmap = generate_heatmap(position_stats)
```

2. **PST の更新**

```rust
// pst.rsを自動生成
const PAWN_PST: [i32; 81] = [
    // 分析結果に基づいた値
    10, 10, 10, 10, 10, 10, 10, 10, 10,
    20, 20, 20, 20, 20, 20, 20, 20, 20,
    // ...
];
```

### メリット

- データドリブン
- 直接的な改善

### デメリット

- 過学習のリスク
- データ量が必要

---

## 方法 5: Monte Carlo Tree Search (MCTS) + Neural Network

### 概要

AlphaZero 風のアプローチ（最も強力だが複雑）

### 実装手順

1. **Policy Network（方策ネットワーク）**

```python
# 局面から次の手の確率分布を予測
policy_net = build_policy_network()
policy_net.fit(positions, move_probabilities)
```

2. **Value Network（価値ネットワーク）**

```python
# 局面の勝率を予測
value_net = build_value_network()
value_net.fit(positions, game_outcomes)
```

3. **MCTS の実装**

```rust
// Rustでの実装
struct MCTSNode {
    position: Board,
    visits: u32,
    value: f32,
    policy: Vec<(Move, f32)>,
}
```

4. **Self-Play ループ**

```
MCTS Self-Play → データ収集 → NN訓練 → MCTS更新 → ...
```

### メリット

- 最強のアプローチ
- 人間の知識不要

### デメリット

- 実装が非常に複雑
- 計算コストが高い
- 大量のデータが必要

---

## 方法 6: 遺伝的アルゴリズムによるパラメータ最適化

### 概要

評価関数のパラメータを進化的に最適化

### 実装手順

1. **パラメータのエンコード**

```python
# 評価関数のパラメータを遺伝子として表現
genome = {
    'pawn_value': 100,
    'knight_value': 400,
    'king_safety_weight': 150,
    # ...
}
```

2. **適応度評価**

```python
# Self-Playで勝率を測定
fitness = run_selfplay_with_params(genome)
```

3. **進化**

```python
# 選択、交叉、突然変異
population = evolve(population, fitness_scores)
```

### メリット

- 自動最適化
- 局所最適を回避しやすい

### デメリット

- 時間がかかる
- 不安定

---

## 推奨アプローチ（難易度順）

### 初級: 方法 3（特徴量抽出と重み調整）

**実装時間**: 1-2 日  
**効果**: 中  
**次のステップ**: すぐに実装可能、効果が見えやすい

### 中級: 方法 4（PST 最適化）

**実装時間**: 2-3 日  
**効果**: 中-高  
**次のステップ**: データが増えたら実施

### 上級: 方法 1（教師あり学習）

**実装時間**: 1 週間  
**効果**: 高  
**次のステップ**: TensorFlow/PyTorch の経験があれば

### 最上級: 方法 5（MCTS + NN）

**実装時間**: 1-2 ヶ月  
**効果**: 最高  
**次のステップ**: 長期プロジェクトとして

---

## 実装例: 方法 3（特徴量分析）

```python
#!/usr/bin/env python3
"""
Feature Analysis for Evaluation Function Improvement
"""

import json
from pathlib import Path
from collections import defaultdict

def analyze_winning_patterns():
    # Load results
    results_file = list(Path('selfplay_results').glob('*.json'))[0]
    with open(results_file) as f:
        results = json.load(f)

    # Load kifus
    kifus = []
    for kifu_file in sorted(Path('selfplay_kifu').glob('*.json')):
        with open(kifu_file) as f:
            kifus.append(json.load(f))

    # Analyze features
    p1_features = defaultdict(list)
    p2_features = defaultdict(list)

    for game_result, kifu in zip(results['games'], kifus):
        winner = game_result.get('winner')

        # Extract features
        promotion_rate = sum(1 for m in kifu['moves']
                           if 'Normal' in m and m['Normal'].get('promote')) / len(kifu['moves'])
        drop_rate = sum(1 for m in kifu['moves']
                       if 'Drop' in m) / len(kifu['moves'])
        game_length = len(kifu['moves'])

        if winner == 'Player1':
            p1_features['promotion_rate'].append(promotion_rate)
            p1_features['drop_rate'].append(drop_rate)
            p1_features['game_length'].append(game_length)
        elif winner == 'Player2':
            p2_features['promotion_rate'].append(promotion_rate)
            p2_features['drop_rate'].append(drop_rate)
            p2_features['game_length'].append(game_length)

    # Compare
    print("=== Winning Patterns Analysis ===\n")
    for feature in ['promotion_rate', 'drop_rate', 'game_length']:
        p1_avg = sum(p1_features[feature]) / len(p1_features[feature]) if p1_features[feature] else 0
        p2_avg = sum(p2_features[feature]) / len(p2_features[feature]) if p2_features[feature] else 0

        print(f"{feature}:")
        print(f"  P1 (winners): {p1_avg:.3f}")
        print(f"  P2 (winners): {p2_avg:.3f}")
        print(f"  Difference: {abs(p1_avg - p2_avg):.3f}\n")

if __name__ == "__main__":
    analyze_winning_patterns()
```

---

## まとめ

現在のデータ量（5 ゲーム）では:

1. **方法 3（特徴量分析）** から始めるのが最適
2. データを 100 ゲーム以上に増やす
3. **方法 4（PST 最適化）** を追加
4. さらにデータが増えたら **方法 1（教師あり学習）** へ

段階的に進めることで、着実に強くなります！
