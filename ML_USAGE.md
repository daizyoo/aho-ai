# ML 評価関数の学習と使用ガイド

## 前提条件

1. **特徴抽出バイナリのビルド:**

```bash
cargo build --release --bin extract_features
```

2. **Python 依存関係のインストール:**

```bash
cd scripts/ml
pip install -r requirements.txt
```

## ステップ 1: 学習データの収集

SelfPlay を実行して kifu ファイルを生成します:

```bash
cargo run --release
# "4. Self-Play"を選択
# 設定例: 100ゲーム、Fairボード
```

これにより `selfplay_kifu/Fair/*.json` ファイルが作成されます。

## ステップ 2: データセットの準備

### 単一の盤

```bash
python scripts/ml/prepare_dataset.py --boards Fair
```

### 複数の盤

```bash
python scripts/ml/prepare_dataset.py --boards "Fair,ShogiOnly,ChessOnly"
```

### すべての盤

```bash
python scripts/ml/prepare_dataset.py --boards all
```

出力: `models/training_data.h5`

## ステップ 3: モデルの訓練

```bash
python scripts/ml/train.py --epochs 20
```

以下が生成されます:

- `models/shogi_model.pt` (PyTorch の重み)
- `models/shogi_model.onnx` (Rust 推論用)

## ステップ 4: 訓練済みモデルの使用

### モデルの配置

訓練した ONNX モデルを`models/`ディレクトリに配置します:

```bash
# 既にtrain.pyで生成されている場合
ls models/shogi_model.onnx
```

### 設定ファイルの編集

`ai_config.json`を編集してニューラルネットワーク評価関数を使用するように設定:

```json
{
  "version": "1.0",
  "evaluation": {
    "hand_piece_bonus_multiplier": 1.1,
    "material_values": { ... },
    "pst_enabled": true,
    "evaluator_type": "NeuralNetwork",
    "nn_model_path": "models/shogi_model.onnx"
  },
  "search": {
    "max_depth_light": 2,
    "max_depth_strong": 4
  }
}
```

### ゲームの実行

ML 機能を有効にしてビルド・実行:

```bash
cargo run --release --features ml
```

ゲーム開始時に使用中の evaluator が表示されます:

```
[AI Configuration]
Evaluator: NeuralNetwork
```

## マルチモデルサポート

### アーキテクチャ

このプロジェクトは複数の ML モデルをサポートする設計になっています:

- **モデルレジストリ** (`src/ml/model_registry.rs`): モデルの管理・検出
- **動的 evaluator ロード** (`src/player/ai/alpha_beta.rs`): 設定に基づいて評価関数を選択
- **設定ベースの切り替え**: `ai_config.json`で簡単に evaluator を変更可能

### サポートされているモデルタイプ

現在:

- ✅ **ONNX**: ONNX ランタイムを使用した推論 ( `--features ml`でビルド)
- ✅ **Handcrafted**: 手作り評価関数 (デフォルト、ML 機能不要)

将来:

- 🚧 PyTorch
- 🚧 TensorFlow

### モデルの自動検出

モデルレジストリは`models/`ディレクトリを自動的にスキャンし、利用可能なモデルを検出します。

## トラブルシューティング

### ML 機能なしでビルドしている

エラー: `"ML feature not enabled. Rebuild with --features ml"`

解決策:

```bash
cargo build --release --features ml
cargo run --release --features ml
```

### モデルファイルが見つからない

確認項目:

1. `models/shogi_model.onnx`ファイルが存在するか
2. `ai_config.json`の`nn_model_path`が正しいパスを指しているか
3. 相対パスはプロジェクトルートから指定

## ヒント

- まずは 100 ゲームで素早く実験してみましょう
- 集中的な訓練には単一の盤(`Fair`など)を使用
- データが多いほど良いモデルになります(1,000 ゲーム以上を目指しましょう)
- CPU での訓練は 10 エポックで約 5〜10 分かかります
- モデルを切り替えるには、`ai_config.json`の`evaluator_type`を変更して再起動
