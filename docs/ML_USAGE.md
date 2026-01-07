# ML 評価関数の学習と使用ガイド

Shogi-Aho-AI でニューラルネットワーク評価関数を訓練・使用するための完全ガイド。

---

## クイックスタート

```bash
# 1. 訓練データ生成 (Self-Play)
cargo run --release -- selfplay --num-games 5000 --board Fair

# 2. データセット準備
python scripts/ml/prepare_dataset.py --boards Fair --version 0.3.0

# 3. モデル訓練
python scripts/ml/train.py --board Fair --version 0.3.0 --epochs 50

# 4. ゲームで使用
# ai_config.json を編集: evaluator_type を "NeuralNetwork" に設定
cargo run --release --features ml
```

---

## 前提条件

### 1. 特徴抽出バイナリのビルド

```bash
cargo build --release --bin extract_features
```

### 2. Python 依存関係のインストール

```bash
cd scripts/ml
pip install -r requirements.txt
```

**必要なパッケージ**: PyTorch, h5py, numpy, onnx, onnxruntime

---

## ステップ 1: 訓練データの収集

Self-Play でゲーム記録(棋譜)を生成：

```bash
cargo run --release -- selfplay \\\n  --num-games 5000 \\\n  --board Fair \\\n  --parallel 6
```

**オプション**:

- `--num-games`: 生成するゲーム数 (1,000+ 推奨)
- `--board`: 盤面設定 (Fair, ShogiOnly, ChessOnly, StandardMixed など)
- `--parallel`: 並列ゲーム数 (デフォルト: CPU コア数)

**出力**: `selfplay_kifu/{BoardType}/*.json` に棋譜ファイルを保存

**推奨データ量**:

- **テスト**: 100-500 ゲーム
- **基本モデル**: 1,000-5,000 ゲーム
- **強いモデル**: 10,000+ ゲーム

---

## ステップ 2: データセットの準備

棋譜ファイルから特徴量を抽出し、HDF5 形式のデータセットを作成。

### 単一の盤面タイプ (推奨)

```bash
python scripts/ml/prepare_dataset.py --boards Fair --version 0.3.0
```

### 複数の盤面タイプ

```bash
python scripts/ml/prepare_dataset.py --boards \"Fair,ShogiOnly\" --version 0.3.0
```

### 全盤面タイプ

```bash
python scripts/ml/prepare_dataset.py --boards all --version 0.3.0
```

**出力**: `models/{board_type}/v{version}/training_data.h5`

**機能**:

- ✅ **データ拡張**: 対称盤面での水平反転 (Fair, StandardMixed)
- ✅ **強化ラベル**: ゲーム長、駒差、評価値軌跡、決定的局面
- ✅ **圧縮**: gzip 圧縮 (~50% サイズ削減)

**データセットスキーマ** (v0.3.0):

```python
{
    'features': (N, 3344),         # 盤面特徴量
    'moves': (N,),                 # 手のインデックス
    'outcomes': (N,),              # ゲーム結果 (0/0.5/1)
    'game_lengths': (N,),          # 新: ゲーム長
    'material_diffs': (N,),        # 新: 最終駒差
    'augmented': (N,),             # 新: 拡張フラグ
}
```

---

## ステップ 3: モデルの訓練

準備したデータセットでニューラルネットワークモデルを訓練。

### 基本の訓練

```bash
python scripts/ml/train.py \\\n  --board Fair \\\n  --version 0.3.0 \\\n  --epochs 50
```

### 高度な訓練

```bash
python scripts/ml/train.py \\\n  --board Fair \\\n  --version 0.3.0 \\\n  --epochs 50 \\\n  --batch-size 128 \\\n  --learning-rate 0.001 \\\n  --use-enhanced-labels  # 新ラベルフィールドを使用
```

**パラメータ**:

- `--epochs`: 訓練エポック数 (20-100 推奨)
- `--batch-size`: バッチサイズ (デフォルト: 64)
- `--learning-rate`: 学習率 (デフォルト: 0.001)
- `--use-enhanced-labels`: 新ラベルで補助タスクを有効化

**出力ファイル**:

- `models/{board}/v{version}/model.pt` - PyTorch 重み
- `models/{board}/v{version}/model.onnx` - Rust 用 ONNX
- `models/{board}/v{version}/README.md` - 訓練統計

**モデルアーキテクチャ**:

- ResNet スタイル (8 残差ブロック)
- 入力: 3344 特徴量
- Policy ヘッド: 9072 手
- Value ヘッド: 勝率予測

---

## ステップ 4: 訓練済みモデルの使用

### 1. AI 設定

`ai_config.json` を編集:

```json
{
  \"version\": \"1.0\",
  \"evaluation\": {
    \"evaluator_type\": \"NeuralNetwork\",
    \"nn_model_path\": \"models/Fair/v0.3.0/model.onnx\"
  }
}
```

**評価器タイプ**:

- `\"Handcrafted\"` - 従来の評価関数 (デフォルト)
- `\"NeuralNetwork\"` - ML ベース評価

### 2. ML でゲームを実行

```bash
cargo run --release --features ml
```

**重要**: `--features ml` フラグが必須！

### 3. モデル選択メニュー

NeuralNetwork 評価器使用時、アプリケーションはモデル選択メニューを表示:

```
ML モデル選択:
1. Fair/v0.3.0 (最新)
2. ShogiOnly/v0.2.0
3. [カスタムパス]
```

---

## 反復改善サイクル

### Self-Play 訓練ループ

```
┌─────────────────────────────────────────┐
│ 1. ゲーム生成 (Self-Play)              │
│    - 現在最良の評価関数を使用           │
│    - 5,000+ ゲーム                      │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 2. データセット準備                      │
│    - 特徴量抽出                          │
│    - 拡張適用                            │
│    - HDF5作成                           │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 3. 新モデル訓練                          │
│    - 50+ エポック                        │
│    - Loss監視                           │
│    - ONNXエクスポート                    │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 4. 新モデル評価                          │
│    - 旧 vs 新 (100+ ゲーム)             │
│    - 勝率測定                            │
│    - Elo比較                            │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 5. より良ければデプロイ                  │
│    - ai_config.json 更新                │
│    - 新モデルが「最良」に                │
└──────────────┬──────────────────────────┘
               ↓
               └──────> ステップ1に戻る
```

### コマンド例

```bash
# サイクル1: 初期モデル (v0.1.0)
cargo run --release -- selfplay --num-games 5000 --board Fair
python scripts/ml/prepare_dataset.py --boards Fair --version 0.1.0
python scripts/ml/train.py --board Fair --version 0.1.0 --epochs 50

# サイクル2: 改善評価器 + NN (v0.2.0)
# ai_config.json を編集して v0.1.0 モデルを使用
cargo run --release --features ml -- selfplay --num-games 5000 --board Fair
python scripts/ml/prepare_dataset.py --boards Fair --version 0.2.0
python scripts/ml/train.py --board Fair --version 0.2.0 --epochs 50

# サイクル3: さらに改善 (v0.3.0)
# 継続...
```

---

## 強化された評価関数 (v0.3.0)

**最新の改善**: 手作り評価関数が大幅強化 (+210-360 Elo)

**新機能**:

1. **モビリティ** - 駒の活動性スコアリング
2. **フェーズ検出** - 序盤/中盤/終盤の認識
3. **強化キング安全性** - 逃げ場所、攻撃駒検出
4. **戦術パターン** - パスポーン、ビショップペア、オープンファイル
5. **駒組み** - 序盤の駒の展開

**ML への影響**:

- より良い Self-Play ゲーム = より良い訓練データ
- v0.3.0 評価器で訓練したモデルは +100-200 Elo 強くなる見込み

詳細は [docs/improvements/EVALUATION_IMPROVEMENTS.md](./improvements/EVALUATION_IMPROVEMENTS.md) 参照。

---

## トラブルシューティング

### FileNotFoundError: training_data.h5

**問題**: prepare と train で盤面タイプが不一致

```
FileNotFoundError: 'models/Fair/v0.3.0/training_data.h5'
```

**解決策**: 両ステップで `--board` を一致させる:

```bash
# これらを一致させる:
python scripts/ml/prepare_dataset.py --boards Fair --version 0.3.0
python scripts/ml/train.py --board Fair --version 0.3.0
```

### ML 機能が有効でない

**問題**: `--features ml` なしで実行

```
error: ML feature not enabled. Rebuild with --features ml
```

**解決策**: 常にフィーチャーフラグを使用:

```bash
cargo run --release --features ml
```

### ONNX モデル読み込みエラー

**問題**: モデルバージョン不一致または破損

**解決策**:

1. ONNX を再エクスポート: `python scripts/ml/train.py --board Fair --version X.Y.Z --export-only`
2. `ai_config.json` でモデルパスを確認
3. ONNX ランタイムの互換性を確認

### モデル性能が低い

**考えられる原因**:

- 訓練データ不足 (< 1,000 ゲーム)
- 過学習 (エポック数が多すぎ)
- Self-Play の元評価器が弱い

**解決策**:

1. より多くのゲームを生成
2. 訓練 Loss を監視 (減少すべき)
3. 改善された手作り評価器を使用 (v0.3.0+)

---

## ヒントとベストプラクティス

### データ収集

- **小さく始める**: まず 100 ゲームでテスト
- **スケールアップ**: 本格的訓練には 5,000+
- **並列使用**: 8 コア CPU で `--parallel 6`
- **局面の混合**: 異なる盤面設定を使用

### 訓練

- **Loss を監視**: 着実に減少すべき
- **早期停止**: Loss が平らになったら停止
- **学習率**: 0.001 から開始、不安定なら減少
- **バッチサイズ**: 大きいほど高速だがメモリ使用量増

### バージョン管理

- **セマンティックバージョニング**: v0.1.0, v0.2.0, v0.3.0 など
- **変更を追跡**: バージョンフォルダに改善を記録
- **古いモデルを保持**: 以前のバージョンと比較

### 評価

- **Elo テスト**: 100+ゲームでモデルを比較
- **勝率**: 前バージョンに対し 55%+を目標
- **多様なテスト**: 複数の盤面設定でテスト

---

## 性能指標

### 期待される改善

| バージョン  | 元            | 期待 Elo     | 前バージョン勝率 |
| ----------- | ------------- | ------------ | ---------------- |
| v0.1.0      | 基本評価      | ベースライン | -                |
| v0.2.0      | depth=6       | +100         | ~60%             |
| v0.3.0      | 強化評価      | +250         | ~70%             |
| v0.4.0 (NN) | v0.3.0 データ | +100         | ~55%             |

### 訓練時間

**概算時間** (RTX 3060, 10k サンプル):

| エポック | 時間   | 備考           |
| -------- | ------ | -------------- |
| 10       | ~5 分  | クイックテスト |
| 50       | ~20 分 | 推奨           |
| 100      | ~40 分 | 徹底訓練       |

---

## 関連ドキュメント

- [評価関数改善](./improvements/EVALUATION_IMPROVEMENTS.md) - 強化された手作り評価器
- [ML データ改善](./improvements/ML_DATA_IMPROVEMENTS.md) - データ拡張とラベル
- [索引](./INDEX.md) - ドキュメント概要

---

**最終更新**: 2026-01-07  
**現在バージョン**: 0.5.0  
**ML データスキーマ**: v0.3.0
