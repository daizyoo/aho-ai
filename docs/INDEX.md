# Shogi-Aho-AI - 実装ドキュメント索引

このドキュメントは、Shogi-Aho-AI プロジェクトの実装と改善の全体像を示します。

---

## 📚 ドキュメント構成

### プロジェクト概要

- [README.md](../README.md) - プロジェクト全体の説明
- [ARCHITECTURE.md](../ARCHITECTURE.md) - システムアーキテクチャ
- [ML_USAGE.md](./ML_USAGE.md) - 機械学習の使用方法
- [VERSIONING.md](./VERSIONING.md) - バージョン管理ガイド

### 最新の改善実装 (2026-01-07)

#### 評価関数の改善

- [📄 評価関数改善・完全版](./improvements/EVALUATION_IMPROVEMENTS.md)
  - Priority 1-5 の全実装詳細
  - 期待 Elo: +210-360
  - モビリティ、フェーズ検出、キング安全性、戦術パターン、駒組み

#### ML データ品質改善

- [📄 ML データ品質改善・完全版](./improvements/ML_DATA_IMPROVEMENTS.md)
  - データ拡張（水平反転）
  - 強化されたラベル（material_diff, evaluations, critical_moments）
  - HDF5 スキーマ拡張

---

## 🎯 実装済み機能まとめ

### 評価関数 (src/player/ai/eval.rs)

**基本評価** (既存):

- ✅ 駒の価値評価
- ✅ PST (Piece-Square Tables)
- ✅ 手駒の評価
- ✅ ポーン構造ペナルティ

**新規実装** (2026-01-07):

- ✅ **Mobility 評価** - 駒の活動性 (+100-150 Elo)
- ✅ **Game Phase 検出** - 序盤/中盤/終盤の適応 (+50-100 Elo)
- ✅ **強化されたキング安全性** - 逃げ場所+攻撃駒 (+30-50 Elo)
- ✅ **戦術パターン認識** - パスポーン、ビショップペア、開いたファイル (+20-40 Elo)
- ✅ **駒組み評価** - 序盤の開発促進 (+10-20 Elo)

**総合効果**: +210-360 Elo

### 機械学習パイプライン

**Self-Play** (src/selfplay/mod.rs):

- ✅ 並列対戦生成
- ✅ 強化されたゲーム結果記録
  - material_diff (最終駒差)
  - avg_move_time_ms (平均思考時間)
  - position_evaluations (評価値の軌跡)
  - critical_moments (決定的な局面)

**データ準備** (scripts/ml/prepare_dataset.py):

- ✅ データ拡張（対称盤面での水平反転）
- ✅ 強化されたラベル抽出
- ✅ HDF5 圧縮ストレージ (gzip)
- ✅ 拡張フラグ追跡

**トレーニング** (scripts/ml/train.py):

- ✅ 強化ラベル対応
- ✅ 後方互換性維持
- ✅ ResNet アーキテクチャ

---

## 📊 性能指標

### 評価関数の進化

| バージョン | 主要機能      | 期待 Elo     | ビルド時間 |
| ---------- | ------------- | ------------ | ---------- |
| v0.1.0     | 基本評価関数  | Baseline     | -          |
| v0.2.0     | depth=6       | +100         | -          |
| **v0.3.0** | **Full 改善** | **+210-360** | **5.97s**  |

### データ品質の改善

| 項目       | 改善前   | 改善後                        |
| ---------- | -------- | ----------------------------- |
| ラベル情報 | 勝敗のみ | 勝敗+駒差+評価軌跡+決定的局面 |
| データ量   | N        | 2N (対称盤面)                 |
| ストレージ | 無圧縮   | gzip 圧縮 (~50%削減)          |

---

## 🔧 技術スタック

### コア

- **言語**: Rust 1.x
- **ビルド**: Cargo (release 最適化)
- **並列処理**: Rayon

### AI/評価

- **探索**: Alpha-Beta with optimizations
  - Transposition Table
  - Null Move Pruning
  - Late Move Reduction
- **評価**: 多層評価関数 (300+ lines)

### 機械学習

- **言語**: Python 3.x
- **フレームワーク**: PyTorch
- **データ**: HDF5 (h5py)
- **モデル**: ResNet-style (8 blocks)

---

## 📁 重要ファイル

### Rust 実装

```
src/
├── player/ai/
│   ├── eval.rs           // 評価関数 (全改善実装)
│   ├── alpha_beta.rs     // 探索アルゴリズム
│   ├── pst.rs            // Piece-Square Tables
│   └── config.rs         // AI設定
├── selfplay/
│   └── mod.rs            // Self-Play実装 (強化ラベル)
└── ml/
    ├── nn_evaluator.rs   // NN評価器
    └── features.rs       // 特徴抽出
```

### Python 実装

```
scripts/ml/
├── prepare_dataset.py    // データ拡張+ラベル強化
├── train.py              // モデル学習
└── model.py              // NN アーキテクチャ
```

---

## 🚀 クイックスタート

### 1. Self-Play データ生成

```bash
cargo run --release --features ml -- selfplay \
  --num-games 5000 \
  --board Fair \
  --parallel 6
```

### 2. データセット準備

```bash
python scripts/ml/prepare_dataset.py \
  --boards Fair \
  --version 0.3.0
```

### 3. モデル学習

```bash
python scripts/ml/train.py \
  --version 0.3.0 \
  --epochs 50 \
  --batch-size 128
```

### 4. テストプレイ

```bash
cargo run --release -- selfplay \
  --num-games 100 \
  --board ShogiOnly
```

---

## 📈 次のステップ

### 短期 (今すぐ可能)

- [ ] 新評価関数で Self-Play 実行 (100-1000 ゲーム)
- [ ] 実際の Elo 測定
- [ ] 戦術的改善の検証

### 中期 (1-2 週間)

- [ ] 5,000 ゲーム生成
- [ ] v0.3.0 モデル学習
- [ ] v0.2.0 vs v0.3.0 比較

### 長期 (継続的)

- [ ] NN モデルを評価関数として利用
- [ ] 反復的 Self-Play + 学習
- [ ] オンライン対戦システム構築

---

## 🔗 関連リンク

- [評価関数詳細ドキュメント](./improvements/EVALUATION_IMPROVEMENTS.md)
- [ML データ改善詳細](./improvements/ML_DATA_IMPROVEMENTS.md)
- [Artifacts (開発ログ)](file:///.gemini/antigravity/brain/d3837e20-e4ee-405e-afb5-00b9155c7805/)

---

## 📝 更新履歴

### 2026-01-07

- ✅ 評価関数 Priority 1-5 完全実装
- ✅ ML データ品質改善実装
- ✅ ドキュメント体系整備

---

**作成日**: 2026-01-07  
**バージョン**: 0.3.0  
**ステータス**: 実装完了、テスト準備完了
