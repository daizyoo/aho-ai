# Shogi-Aho-AI Documentation

プロジェクトの全ドキュメントは`docs/`フォルダに整理されています。

## 📚 ドキュメント一覧

### メインドキュメント

- **[docs/INDEX.md](./docs/INDEX.md)** - 全体の索引とクイックスタート

### 最新の改善

#### 2026-01-10: Self-Play 品質改善

- **千日手ペナルティ修正**

  - 探索崩壊を防止（-15000/-5000 → 0/-100）
  - 探索深度を改善（1-2 → 3-4）

- **SEE (Static Exchange Evaluation) 実装**

  - 不利な交換を回避
  - ゲーム長を改善（13 手 → 38 手平均）

- **診断・モニタリング機能**
  - ゲーム終了診断ログ自動保存
  - 進捗表示に統計追加（平均手数、Termination 回数）
  - 実行モード選択（並列/順次）

#### 2026-01-07: 評価関数・ML 改善

- **[docs/improvements/EVALUATION_IMPROVEMENTS.md](./docs/improvements/EVALUATION_IMPROVEMENTS.md)**

  - 評価関数の全改善（Priority 1-5）
  - 期待 Elo: +210-360

- **[docs/improvements/ML_DATA_IMPROVEMENTS.md](./docs/improvements/ML_DATA_IMPROVEMENTS.md)**

  - ML データ品質改善
  - データ拡張とラベル強化

- **[docs/improvements/EVALUATION_PLAN.md](./docs/improvements/EVALUATION_PLAN.md)**
  - 評価関数改善の実装計画

### アーキテクチャ・仕様

- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - システム設計
- **[ML_USAGE.md](./ML_USAGE.md)** - 機械学習の使用方法
- **[WINDOWS_GPU_SETUP.md](./docs/WINDOWS_GPU_SETUP.md)** - Windows GPU 設定

## 🚀 クイックリンク

```bash
# ドキュメントを開く
open docs/INDEX.md

# 評価関数の改善を見る
open docs/improvements/EVALUATION_IMPROVEMENTS.md

# MLデータ改善を見る
open docs/improvements/ML_DATA_IMPROVEMENTS.md
```

## 📊 実装状況

**評価関数**: ✅ 完了 (+210-360 Elo)  
**SEE 実装**: ✅ 完了 (ゲーム長 3 倍改善)  
**千日手対策**: ✅ 修正済み（探索深度改善）  
**診断機能**: ✅ 実装済み  
**ML データ品質**: ✅ 完了 (2x data, 5x labels)

## ⚠️ 既知の問題

**Self-Play 詰み達成率**: 🔴 0% (要調査)

- 全ゲームが途中終了
- 診断ログ機能の検証が必要

---

詳細は [docs/INDEX.md](./docs/INDEX.md) をご覧ください。
