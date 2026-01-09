# Windows GPU 加速セットアップガイド（日本語版）

このガイドでは、Windows 環境で GPU 加速を有効にする詳しい手順を説明します。

詳細は [WINDOWS_GPU_SETUP.md](./WINDOWS_GPU_SETUP.md) をご覧ください。

## クイックスタート

### 必要なファイル

1. **実行ファイル**: `shogi-aho-ai.exe`
2. **DLL ファイル（3 つ）**:
   - `DirectML.dll`
   - `onnxruntime.dll`
   - `onnxruntime_providers_shared.dll`

### ダウンロード

ONNX Runtime DirectML 版:

```
https://github.com/microsoft/onnxruntime/releases
→ onnxruntime-win-x64-directml-{version}.zip
```

### 配置

すべてのファイルを同じフォルダに配置:

```
shogi-aho-ai.exe
DirectML.dll
onnxruntime.dll
onnxruntime_providers_shared.dll
```

### 確認

起動時に以下のメッセージが表示されれば成功:

```
[ML] GPU acceleration enabled (DirectML)
```

---

## 対応 GPU

- ✅ AMD Radeon シリーズ（RX 6900 XT 検証済み）
- ✅ NVIDIA GeForce シリーズ
- ✅ Intel Arc / Iris Xe

---

詳しい手順とトラブルシューティングは [WINDOWS_GPU_SETUP.md](./WINDOWS_GPU_SETUP.md) をご覧ください。
