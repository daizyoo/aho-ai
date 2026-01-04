# Self-Play Data Analysis

このディレクトリには、Self-play で生成されたデータを分析するための Python スクリプトが含まれています。

## スクリプト一覧

### 1. analyze_selfplay.py

Self-play の結果 JSON(`selfplay_results_*.json`)を分析します。

**使い方:**

```bash
python analyze_selfplay.py selfplay_results_20260104_222146.json
```

**出力例:**

```
=== Basic Statistics ===
Total Games: 10
Player 1 Wins: 6 (60.0%)
Player 2 Wins: 3 (30.0%)
Draws: 1 (10.0%)
Average Moves: 87.3
Average Time: 12.5s

=== Move Distribution ===
Min Moves: 45
Max Moves: 132
Median Moves: 85.0
Std Dev: 23.4
```

### 2. analyze_kifu.py

個別の棋譜ファイル(`selfplay_kifu/*.json`)を分析します。

**使い方:**

```bash
python analyze_kifu.py selfplay_kifu/game_0001_20260104_220000.json
```

**出力例:**

```
Total Moves: 87

=== Move Types ===
Normal: 75 (86.2%)
Drop: 12 (13.8%)

=== Promotions ===
Total Promotions: 8
Promotion Rate: 9.2%

=== Drops ===
Total Drops: 12
  S_Pawn: 5
  S_Silver: 3
  S_Knight: 4
```

### 3. analyze_batch_kifu.py

`selfplay_kifu/`ディレクトリ内の全棋譜を一括分析します。

**使い方:**

```bash
python analyze_batch_kifu.py
```

**出力例:**

```
=== Aggregate Statistics ===
Total Games Analyzed: 10
Total Moves: 873
Average Game Length: 87.3 moves
Shortest Game: 45 moves
Longest Game: 132 moves

=== Average Rates ===
Average Promotion Rate: 9.1%
Average Drop Rate: 13.5%

=== Piece Drop Usage (All Games) ===
S_Pawn: 45 drops
S_Silver: 23 drops
S_Knight: 18 drops
```

## 必要な環境

Python 3.6 以上（標準ライブラリのみ使用、追加パッケージ不要）

## データの場所

- **結果 JSON**: `selfplay_results_*.json`（プロジェクトルート）
- **棋譜**: `selfplay_kifu/game_*.json`（`save_kifus: true`の場合）

## 活用例

### AI 性能比較

```bash
# Light vs Light
python analyze_selfplay.py results_light_vs_light.json

# Strong vs Strong
python analyze_selfplay.py results_strong_vs_strong.json

# 結果を比較
```

### 盤面バランス分析

```bash
# 異なる盤面設定で実行した結果を比較
python analyze_selfplay.py results_fair.json
python analyze_selfplay.py results_standard.json
```

### 戦略パターン分析

```bash
# 全棋譜から戦略パターンを抽出
python analyze_batch_kifu.py
```
