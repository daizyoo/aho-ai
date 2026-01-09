# Windows GPU 加速セットアップガイド

このガイドでは、Windows 環境で AMD Radeon RX 6900 XT を使って GPU 加速を有効にする方法を説明します。

## 目次

- [概要](#概要)
- [前提条件](#前提条件)
- [セットアップ手順](#セットアップ手順)
- [動作確認](#動作確認)
- [トラブルシューティング](#トラブルシューティング)
- [パフォーマンス](#パフォーマンス)

---

## 概要

このプロジェクトは**DirectML**を使用して Windows 上で GPU 加速をサポートしています。

### DirectML とは

- Microsoft 製の GPU 加速ライブラリ
- Windows 10/11 に標準搭載
- **AMD、NVIDIA、Intel**すべての GPU に対応
- ONNX Runtime と統合

### 対応 GPU

- ✅ AMD Radeon RX 6900 XT（検証済み）
- ✅ NVIDIA GeForce シリーズ
- ✅ Intel Arc / Iris Xe

---

## 前提条件

### システム要件

- **OS**: Windows 10 バージョン 1903 以降、または Windows 11
- **GPU**: DirectX 12 対応 GPU
- **ドライバー**: 最新の GPU ドライバー

### 確認方法

1. Windows バージョン確認:

   ```cmd
   winver
   ```

   → 1903 以降であることを確認

2. DirectX 12 確認:
   ```cmd
   dxdiag
   ```
   → 「DirectX Version: DirectX 12」と表示されることを確認

---

## セットアップ手順

### ステップ 1: 実行ファイルの準備

#### Mac でビルド（クロスコンパイル）

```bash
# ML機能を有効にしてWindows向けにビルド
cargo build --release --target x86_64-pc-windows-gnu --features ml
```

生成されるファイル:

```
target/x86_64-pc-windows-gnu/release/shogi-aho-ai.exe
```

#### または、Windows で直接ビルド

```cmd
cargo build --release --features ml
```

---

### ステップ 2: ONNX Runtime（DirectML 版）のダウンロード

1. **GitHub リリースページにアクセス**:

   ```
   https://github.com/microsoft/onnxruntime/releases
   ```

2. **最新版を探す**:

   - 「onnxruntime-win-x64-directml-{version}.zip」を探す
   - 例: `onnxruntime-win-x64-directml-1.19.2.zip`
   - ⚠️ 注意: `-directml-` が付いているものを選択

3. **ダウンロード**:
   - ファイルサイズは約 50-100MB

---

### ステップ 3: DLL ファイルの配置

#### 3.1. zip ファイルを解凍

ダウンロードした zip ファイルを解凍すると、以下の構造になっています:

```
onnxruntime-win-x64-directml-{version}/
├── lib/
│   ├── DirectML.dll
│   ├── onnxruntime.dll
│   ├── onnxruntime_providers_shared.dll
│   └── その他のファイル...
├── include/
└── ...
```

#### 3.2. 必要な DLL をコピー

`lib/`フォルダから以下の**3 つの DLL**を、`shogi-aho-ai.exe`と**同じフォルダ**にコピー:

```
✅ DirectML.dll
✅ onnxruntime.dll
✅ onnxruntime_providers_shared.dll
```

#### 3.3. 最終的なフォルダ構成

```
your_app_folder/
├── shogi-aho-ai.exe                      ← 実行ファイル
├── DirectML.dll                          ← ONNX Runtime DML拡張
├── onnxruntime.dll                       ← ONNX Runtime本体
├── onnxruntime_providers_shared.dll      ← プロバイダー共有ライブラリ
└── models/                               ← モデルフォルダ（ML使用時）
    └── ShogiOnly/
        └── v0.1.0/
            └── model.onnx
```

---

### ステップ 4: 設定ファイル（オプション）

GPU 加速を使用するには、`config/ai_config.json`でニューラルネットワーク評価を有効にする必要があります。

#### ai_config.json

```json
{
  "evaluation": {
    "evaluator_type": "NeuralNetwork",
    "nn_model_path": "models/ShogiOnly/v0.1.0/model.onnx"
  },
  "search": {
    "default_depth": 5,
    "time_per_move_ms": 60000
  }
}
```

**フォルダ構成**:

```
your_app_folder/
├── shogi-aho-ai.exe
├── DirectML.dll
├── onnxruntime.dll
├── onnxruntime_providers_shared.dll
├── config/
│   └── ai_config.json
└── models/
    └── ShogiOnly/
        └── v0.1.0/
            └── model.onnx
```

---

## 動作確認

### 起動メッセージの確認

アプリケーションを起動すると、以下のメッセージが表示されます:

#### ✅ GPU 加速が有効な場合

```
[ML] GPU acceleration enabled (DirectML)
[ML] Loaded model: ShogiOnly/v0.1.0 (v1.0)
```

#### ⚠️ CPU 動作の場合

```
[ML] Warning: DirectML not available, using CPU: ...
[ML] Loaded model: ShogiOnly/v0.1.0 (v1.0)
```

---

### タスクマネージャーで GPU 使用率を確認

1. **タスクマネージャー**を開く（`Ctrl + Shift + Esc`）
2. **パフォーマンス**タブを選択
3. 左側のメニューから**GPU**を選択
4. Self-Play を実行中に以下を確認:
   - **GPU 使用率**が上昇（通常 10-30%）
   - **専用 GPU メモリ**が使用されている

#### GPU 使用例

```
GPU 0 - AMD Radeon RX 6900 XT
┌─────────────────────────────┐
│ 使用率:    25%              │
│ メモリ:   1.2GB / 16GB      │
│ 温度:     65°C              │
└─────────────────────────────┘
```

---

## トラブルシューティング

### Q1: DirectML が有効にならない

**症状**:

```
[ML] Warning: DirectML not available, using CPU: ...
```

**原因と対策**:

1. **DLL ファイルが見つからない**

   - `DirectML.dll`、`onnxruntime.dll`、`onnxruntime_providers_shared.dll`が`.exe`と同じフォルダにあるか確認

2. **Windows バージョンが古い**

   - Windows 10 バージョン 1903 以降が必要
   - `winver`コマンドで確認

3. **GPU ドライバーが古い**

   - AMD Radeon ドライバーを最新版に更新
   - https://www.amd.com/support

4. **DirectX 12 が無効**
   - `dxdiag`で確認
   - Windows Update で最新化

---

### Q2: モデルファイルが見つからない

**症状**:

```
Error: No such file or directory (os error 2)
```

**対策**:

1. **パスを確認**:

   ```
   models/ShogiOnly/v0.1.0/model.onnx
   ```

2. **相対パスの基準**:

   - 実行ファイル（`.exe`）からの相対パス
   - または`ai_config.json`で絶対パスを指定

3. **ファイル存在確認**:
   ```cmd
   dir models\ShogiOnly\v0.1.0\model.onnx
   ```

---

### Q3: パフォーマンスが変わらない・向上しない

**原因**:

1. **Handcrafted 評価を使用している**

   - `ai_config.json`で`evaluator_type`が`"Handcrafted"`になっている
   - → `"NeuralNetwork"`に変更

2. **モデルが小さすぎる**

   - GPU のオーバーヘッドがある
   - 小さいモデルでは CPU の方が速い場合もある

3. **探索深度が浅い**
   - 探索深度が 2-3 程度だと GPU の利点が出にくい
   - depth 5-6 以上で効果が出る

---

### Q4: アプリケーションが起動しない

**症状**:

```
error loading onnxruntime.dll
```

**対策**:

1. **Visual C++ Redistributable をインストール**:

   ```
   https://aka.ms/vs/17/release/vc_redist.x64.exe
   ```

2. **DLL バージョンの不一致**:
   - ビルド時の ONNX Runtime バージョンと、配置した DLL のバージョンが一致しているか確認
   - Cargo.toml で`ort = "2.0.0-rc.10"`となっている場合、DLL も v1.19.x 系を使用

---

## パフォーマンス

### CPU vs GPU 比較（予想値）

#### CPU（Handcrafted 評価）

```
探索深度:   4-6
1手あたり:  10-60秒
ノード数:   10,000-100,000
評価方式:   ルールベース
```

#### GPU（DirectML + Neural Network）

```
探索深度:   5-7（より深く）
1手あたり:  1-10秒（5-10倍高速）
ノード数:   50,000-500,000
評価方式:   ニューラルネットワーク（高精度）
```

### AMD Radeon RX 6900 XT 性能

- **CU 数**: 80
- **メモリ**: 16GB GDDR6
- **理論性能**: 23 TFLOPS（FP32）

DirectML を使用することで、この強力な GPU を最大限活用できます。

---

## 関連ドキュメント

- [ML_USAGE.md](./ML_USAGE.md) - 機械学習モデルの使い方
- [ARCHITECTURE.md](./ARCHITECTURE.md) - アーキテクチャ概要
- [EVALUATION_IMPROVEMENTS.md](./EVALUATION_IMPROVEMENTS.md) - 評価関数の改善履歴

---

## 参考リンク

- [ONNX Runtime - DirectML](https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html)
- [Microsoft DirectML](https://github.com/microsoft/DirectML)
- [AMD Radeon ドライバー](https://www.amd.com/support)

---

## まとめ

✅ **3 つの DLL を配置するだけ**で、AMD Radeon RX 6900 XT の性能を活用できます

✅ **自動検出**: コード側で自動的に DirectML を使用

✅ **フォールバック**: DirectML が使えない環境では自動的に CPU を使用

より詳しい技術情報は、プロジェクトの Issue や Discussions でお気軽にお問い合わせください。
