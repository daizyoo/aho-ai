# Shogi-Aho-AI

将棋とチェスのハイブリッドゲームに対応した、機械学習機能を備えた AI 対戦システム

[![Rust](https://img.shields.io/badge/Rust-1.x-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.6.0-green.svg)](Cargo.toml)

## 📖 概要

Shogi-Aho-AI は、将棋とチェスのルールを融合させたボードゲームの AI 対戦システムです。以下の特徴を持ちます：

- 🎮 **3 つのボードタイプ**: ShogiOnly、StandardMixed、Fair
- 🤖 **強力な AI**: Alpha-Beta 探索 + 高度な評価関数
- 🧠 **機械学習**: PyTorch ベースのニューラルネットワーク評価器
- 🔄 **Self-Play**: 並列/順次実行に対応した自己対戦システム
- 🌐 **Web インターフェース**: Next.js 製のブラウザ UI（開発中）

## 🚀 クイックスタート

### 前提条件

- Rust 1.x 以降
- Python 3.8 以降（機械学習機能を使用する場合）
- Cargo

### インストール

```bash
# リポジトリをクローン
git clone https://github.com/daizyoo/aho-ai.git
cd aho-ai

# ビルド
cargo build --release

# ビルド（機械学習機能付き）
cargo build --release --features ml
```

### 基本的な使い方

#### 1. 対戦プレイ

```bash
# ローカル対戦（人 vs 人 or AI vs AI）
cargo run --release

# ボードタイプを指定
cargo run --release -- local --board ShogiOnly
```

#### 2. Self-Play（AI 自己対戦）

```bash
# 100ゲームの自己対戦を並列実行
cargo run --release -- selfplay --num-games 100 --board Fair --parallel 6

# 順次実行（デバッグ用）
cargo run --release -- selfplay --num-games 10 --board Fair --sequential
```

#### 3. 機械学習パイプライン

```bash
# データセット準備
python scripts/ml/prepare_dataset.py --boards Fair --version 0.3.0

# モデル学習
python scripts/ml/train.py --version 0.3.0 --epochs 50 --batch-size 128

# ONNXエクスポート
python scripts/ml/export_to_onnx.py --version 0.3.0
```

## 🎯 主要機能

### AI 評価関数

- ✅ **基本評価**: 駒の価値、位置評価（PST）、手駒評価
- ✅ **モビリティ評価**: 駒の活動性（+100-150 Elo）
- ✅ **ゲームフェーズ検出**: 序盤/中盤/終盤の適応（+50-100 Elo）
- ✅ **キング安全性**: 逃げ場所と攻撃駒の評価（+30-50 Elo）
- ✅ **戦術パターン**: パスポーン、ビショップペア、開いたファイル（+20-40 Elo）
- ✅ **SEE**: Static Exchange Evaluation（不利な交換回避）

**総合効果**: +210-360 Elo

### 探索アルゴリズム

- Alpha-Beta pruning with optimizations
- Transposition Table（局面ハッシュテーブル）
- Null Move Pruning
- Late Move Reduction
- 千日手検出と対策

### 機械学習

- **モデル**: ResNet-style（8 ブロック）
- **フレームワーク**: PyTorch
- **推論**: ONNX Runtime（DirectML GPU 対応）
- **データ拡張**: 対称盤面での水平反転
- **強化ラベル**: 勝敗、駒差、評価軌跡、決定的局面

## 📁 プロジェクト構造

```
shogi-aho-ai/
├── src/
│   ├── core/              # コアゲームロジック
│   ├── logic/             # ゲームルールとバリデーション
│   ├── player/            # プレイヤーコントローラー
│   │   └── ai/            # AI実装
│   │       ├── eval.rs    # 評価関数
│   │       ├── alpha_beta.rs  # 探索アルゴリズム
│   │       ├── see.rs     # Static Exchange Evaluation
│   │       └── pst.rs     # Piece-Square Tables
│   ├── selfplay/          # Self-Playシステム
│   ├── ml/                # 機械学習モジュール
│   ├── ui/                # ターミナルUI
│   └── main.rs            # エントリーポイント
├── scripts/
│   ├── ml/                # Python機械学習スクリプト
│   │   ├── prepare_dataset.py
│   │   ├── train.py
│   │   └── model.py
│   └── analyze_results.py # Self-Play結果分析
├── web/                   # Webインターフェース（Next.js）
├── docs/                  # ドキュメント
└── models/                # 学習済みモデル（.gitignore）
```

## 📚 ドキュメント

詳細なドキュメントは `docs/` ディレクトリにあります：

- **[DOCS.md](./DOCS.md)** - ドキュメント索引
- **[docs/INDEX.md](./docs/INDEX.md)** - 全体の索引とクイックスタート
- **[docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)** - システムアーキテクチャ
- **[docs/ML_USAGE.md](./docs/ML_USAGE.md)** - 機械学習の使用方法
- **[docs/VERSIONING.md](./docs/VERSIONING.md)** - バージョン管理ガイド
- **[docs/WINDOWS_GPU_SETUP.md](./docs/WINDOWS_GPU_SETUP.md)** - Windows GPU 設定

### 最新の改善

- **[評価関数改善](./docs/improvements/EVALUATION_IMPROVEMENTS.md)** - Priority 1-5 の詳細実装
- **[ML データ品質改善](./docs/improvements/ML_DATA_IMPROVEMENTS.md)** - データ拡張とラベル強化

## 🛠️ 設定

### AI 設定

`ai_config.json` で AI の動作をカスタマイズ：

```json
{
  "search_depth": 6,
  "use_transposition_table": true,
  "use_null_move_pruning": true,
  "use_late_move_reduction": true,
  "time_per_move_ms": 5000
}
```

### ボードタイプ

- **ShogiOnly**: 完全な将棋ルール
- **StandardMixed**: 将棋とチェスの混合ルール
- **Fair**: バランス調整版

## 📊 性能指標

### 評価関数の進化

| バージョン | 主要機能      | 期待 Elo     | ビルド時間 |
| ---------- | ------------- | ------------ | ---------- |
| v0.1.0     | 基本評価関数  | Baseline     | -          |
| v0.2.0     | depth=6       | +100         | -          |
| **v0.3.0** | **Full 改善** | **+210-360** | **5.97s**  |

### Self-Play 統計

- **ゲーム長**: 平均 38 手（SEE 実装後、13 手から改善）
- **探索深度**: 平均 3-4（千日手対策後、1-2 から改善）
- **並列処理**: 最大 8 スレッド対応

## 🧪 開発

### テスト実行

```bash
# 単体テスト
cargo test

# 特定のテスト
cargo test --test logic_tests

# リリースビルドでテスト
cargo test --release
```

### Self-Play 診断

```bash
# 診断ログ付きでSelf-Play実行
cargo run --release -- selfplay --num-games 10 --board Fair --sequential

# 診断ログは selfplay_termination_logs/ に保存
```

### 結果分析

```bash
# Self-Play結果の統計分析
python scripts/analyze_results.py

# 戦術診断
python scripts/diagnose_tactics.py
```

## 🌐 Web インターフェース

Next.js ベースの WebUI を開発中：

```bash
cd web
npm install
npm run dev
```

詳細は [web/README.md](./web/README.md) を参照。

## 🤝 コントリビューション

プルリクエストを歓迎します！

1. このリポジトリをフォーク
2. フィーチャーブランチを作成（`git checkout -b feature/amazing-feature`）
3. 変更をコミット（`git commit -m 'Add amazing feature'`）
4. ブランチにプッシュ（`git push origin feature/amazing-feature`）
5. プルリクエストを作成

## 📝 ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。

## 🙏 謝辞

- Rust コミュニティ
- PyTorch チーム
- ONNX Runtime プロジェクト

## 📧 連絡先

質問や提案がある場合は、Issue を作成してください。

---

**最終更新**: 2026-01-15  
**バージョン**: v0.6.0  
**ステータス**: アクティブ開発中
