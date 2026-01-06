# Scripts Directory

このディレクトリには、Self-play データの分析、可視化、および ML 訓練のための Python スクリプトが含まれています。

## ディレクトリ構成

```
scripts/
├── analyze_results.py      # セルフプレイ結果の統計分析
├── deep_analysis.py         # 詳細な戦略分析
├── summary_report.py        # サマリーレポート生成
├── visualize_boards.py      # ボード構成の可視化と分析
├── visualize_thinking.py    # AI思考過程の可視化
├── bump_version.py          # バージョン管理ユーティリティ
└── ml/                      # 機械学習関連スクリプト
    ├── model.py             # ニューラルネットワークモデル定義
    ├── train.py             # モデル訓練スクリプト
    ├── prepare_dataset.py   # データセット準備
    └── requirements.txt     # Python依存関係
```

## 分析スクリプト

### analyze_results.py

セルフプレイの結果を集計し、統計情報を表示します。

**使い方:**

```bash
python scripts/analyze_results.py
```

**機能:**

- `selfplay_results/`ディレクトリ内の全結果ファイルを集計
- ゲームタイプ別(ボード設定、AI 強度)に統計を表示
- 勝率、平均手数、平均時間などを分析
- カラー出力で見やすく表示

**出力例:**

```
===========================================
GAME TYPE: ShogiOnly (Strong vs Strong)
===========================================
Total Games:     80
Player 1 Wins:   37 (46.2%)
Player 2 Wins:   31 (38.8%)
Draws:           12 (15.0%)
Average Moves:   178.1
Average Time:    29m 43s
```

### deep_analysis.py

より詳細な戦略分析を実行します。

**使い方:**

```bash
python scripts/deep_analysis.py
```

**機能:**

- 駒の価値分析
- ポジショナルアドバンテージの評価
- 勝利パターンの特定

### summary_report.py

簡潔なサマリーレポートを生成します。

**使い方:**

```bash
python scripts/summary_report.py
```

## 可視化スクリプト

### visualize_boards.py

ボード構成を可視化し、構造的な非対称性を分析します。

**使い方:**

```bash
python scripts/visualize_boards.py
```

**機能:**

- ShogiOnly と Fair ボードの可視化
- 駒の配置と価値の分析
- ボード間の比較

**依存関係:**

- 標準ライブラリのみ (外部パッケージ不要)

### visualize_thinking.py

AI の思考データ(評価値、探索深さ、ノード数)をグラフ化します。

**使い方:**

```bash
# 特定のkifuファイルを指定
python scripts/visualize_thinking.py selfplay_kifu/Fair/game_0001.json

# インタラクティブモード(ファイル選択)
python scripts/visualize_thinking.py
```

**機能:**

- 評価値の推移をプロット
- 探索深さの変化を表示
- 評価ノード数の可視化
- グラフを`analysis_graphs/`に保存

**依存関係:**

```bash
pip install matplotlib
```

**出力:**

- `analysis_graphs/{kifu_filename}_analysis.png`

## ML スクリプト

ML 関連の詳細は [ML_USAGE.md](../ML_USAGE.md) を参照してください。

### 概要

1. **prepare_dataset.py** - kifu ファイルから訓練データを準備
2. **model.py** - ニューラルネットワークアーキテクチャの定義
3. **train.py** - モデルの訓練と ONNX エクスポート

**依存関係のインストール:**

```bash
cd scripts/ml
pip install -r requirements.txt
```

## ユーティリティ

### bump_version.py

プロジェクトのバージョンを管理します。詳細は [VERSIONING.md](../VERSIONING.md) を参照。

**使い方:**

```bash
python scripts/bump_version.py patch  # 0.3.2 -> 0.3.3
python scripts/bump_version.py minor  # 0.3.2 -> 0.4.0
python scripts/bump_version.py major  # 0.3.2 -> 1.0.0
```

## ワークフロー例

### 1. セルフプレイデータの分析

```bash
# 1. セルフプレイを実行
cargo run --release
# "4. Self-Play" を選択し、100ゲーム実行

# 2. 結果を分析
python scripts/analyze_results.py

# 3. より詳細な分析
python scripts/deep_analysis.py

# 4. 思考データを可視化
python scripts/visualize_thinking.py
```

### 2. ML モデルの訓練

```bash
# 1. データセット準備
python scripts/ml/prepare_dataset.py --boards Fair

# 2. モデル訓練
python scripts/ml/train.py --epochs 20

# 3. 訓練済みモデルでゲーム実行
# ai_config.json を編集して evaluator_type を "NeuralNetwork" に設定
cargo run --release --features ml
```

## トラブルシューティング

### データが見つからない

```bash
# セルフプレイを実行してデータを生成
cargo run --release
# "4. Self-Play" を選択
```

### Python 依存関係エラー

```bash
# 分析スクリプトは標準ライブラリのみ使用
python3 --version  # 3.6以上が必要

# 可視化用
pip install matplotlib

# ML訓練用
cd scripts/ml
pip install -r requirements.txt
```

### グラフが表示されない

`visualize_thinking.py`でグラフが表示されない場合:

- グラフは `analysis_graphs/` に保存されます
- GUI なし環境の場合、保存された PNG ファイルを確認してください
