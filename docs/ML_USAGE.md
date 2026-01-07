# Machine Learning Usage Guide

Complete guide for training and using neural network evaluators in Shogi-Aho-AI.

---

## Quick Start

```bash
# 1. Generate training data (Self-Play)
cargo run --release -- selfplay --num-games 5000 --board Fair

# 2. Prepare dataset
python scripts/ml/prepare_dataset.py --boards Fair --version 0.3.0

# 3. Train model
python scripts/ml/train.py --board Fair --version 0.3.0 --epochs 50

# 4. Use in game
# Edit ai_config.json: set evaluator_type to "NeuralNetwork"
cargo run --release --features ml
```

---

## Prerequisites

### 1. Build Feature Extraction Binary

```bash
cargo build --release --bin extract_features
```

### 2. Install Python Dependencies

```bash
cd scripts/ml
pip install -r requirements.txt
```

**Required packages**: PyTorch, h5py, numpy, onnx, onnxruntime

---

## Step 1: Collect Training Data

Generate game records (kifu) through Self-Play:

```bash
cargo run --release -- selfplay \
  --num-games 5000 \
  --board Fair \
  --parallel 6
```

**Options**:

- `--num-games`: Number of games to generate (recommend 1,000+)
- `--board`: Board setup (Fair, ShogiOnly, ChessOnly, StandardMixed, etc.)
- `--parallel`: Number of parallel games (default: CPU cores)

**Output**: Kifu files saved to `selfplay_kifu/{BoardType}/*.json`

**Recommended data amounts**:

- **Testing**: 100-500 games
- **Basic model**: 1,000-5,000 games
- **Strong model**: 10,000+ games

---

## Step 2: Prepare Dataset

Extract features from kifu files and create HDF5 dataset.

### Single Board Type (Recommended)

```bash
python scripts/ml/prepare_dataset.py --boards Fair --version 0.3.0
```

### Multiple Board Types

```bash
python scripts/ml/prepare_dataset.py --boards "Fair,ShogiOnly" --version 0.3.0
```

### All Board Types

```bash
python scripts/ml/prepare_dataset.py --boards all --version 0.3.0
```

**Output**: `models/{board_type}/v{version}/training_data.h5`

**Features**:

- ✅ **Data Augmentation**: Horizontal flip for symmetric boards (Fair, StandardMixed)
- ✅ **Enhanced Labels**: Game length, material difference, evaluations, critical moments
- ✅ **Compression**: gzip compression (~50% size reduction)

**Dataset Schema** (v0.3.0):

```python
{
    'features': (N, 3344),         # Board features
    'moves': (N,),                 # Move indices
    'outcomes': (N,),              # Game outcomes (0/0.5/1)
    'game_lengths': (N,),          # NEW: Game duration
    'material_diffs': (N,),        # NEW: Final material difference
    'augmented': (N,),             # NEW: Augmentation flag
}
```

---

## Step 3: Train Model

Train a neural network model using the prepared dataset.

### Basic Training

```bash
python scripts/ml/train.py \
  --board Fair \
  --version 0.3.0 \
  --epochs 50
```

### Advanced Training

```bash
python scripts/ml/train.py \
  --board Fair \
  --version 0.3.0 \
  --epochs 50 \
  --batch-size 128 \
  --learning-rate 0.001 \
  --use-enhanced-labels  # Use new label fields
```

**Parameters**:

- `--epochs`: Training epochs (recommend 20-100)
- `--batch-size`: Batch size (default: 64)
- `--learning-rate`: Learning rate (default: 0.001)
- `--use-enhanced-labels`: Enable auxiliary tasks with new labels

**Output Files**:

- `models/{board}/v{version}/model.pt` - PyTorch weights
- `models/{board}/v{version}/model.onnx` - ONNX for Rust
- `models/{board}/v{version}/README.md` - Training stats

**Model Architecture**:

- ResNet-style with 8 residual blocks
- Input: 3344 features
- Policy head: 9072 moves
- Value head: Win probability

---

## Step 4: Use Trained Model

### 1. Configure AI

Edit `ai_config.json`:

```json
{
  "version": "1.0",
  "evaluation": {
    "evaluator_type": "NeuralNetwork",
    "nn_model_path": "models/Fair/v0.3.0/model.onnx"
  }
}
```

**Evaluator Types**:

- `"Handcrafted"` - Traditional evaluation function (default)
- `"NeuralNetwork"` - ML-based evaluation

### 2. Run Game with ML

```bash
cargo run --release --features ml
```

**Important**: The `--features ml` flag is required!

### 3. Model Selection Menu

When using NeuralNetwork evaluator, the application will show a model selection menu:

```
Select ML Model:
1. Fair/v0.3.0 (Latest)
2. ShogiOnly/v0.2.0
3. [Custom Path]
```

---

## Iterative Improvement Cycle

### The Self-Play Training Loop

```
┌─────────────────────────────────────────┐
│ 1. Generate Games (Self-Play)          │
│    - Use current best evaluator         │
│    - 5,000+ games                       │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 2. Prepare Dataset                      │
│    - Extract features                   │
│    - Apply augmentation                 │
│    - Create HDF5                        │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 3. Train New Model                      │
│    - 50+ epochs                         │
│    - Monitor loss                       │
│    - Export to ONNX                     │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 4. Evaluate New Model                   │
│    - Old vs New (100+ games)            │
│    - Measure win rate                   │
│    - Compare Elo                        │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│ 5. Deploy if Better                     │
│    - Update ai_config.json              │
│    - New model becomes "best"           │
└──────────────┬──────────────────────────┘
               ↓
               └──────> Back to Step 1
```

### Example Commands

```bash
# Cycle 1: Initial model (v0.1.0)
cargo run --release -- selfplay --num-games 5000 --board Fair
python scripts/ml/prepare_dataset.py --boards Fair --version 0.1.0
python scripts/ml/train.py --board Fair --version 0.1.0 --epochs 50

# Cycle 2: Improved evaluator + NN (v0.2.0)
# Edit ai_config.json to use v0.1.0 model
cargo run --release --features ml -- selfplay --num-games 5000 --board Fair
python scripts/ml/prepare_dataset.py --boards Fair --version 0.2.0
python scripts/ml/train.py --board Fair --version 0.2.0 --epochs 50

# Cycle 3: Even better (v0.3.0)
# Continue...
```

---

## Enhanced Evaluation Function (v0.3.0)

**Latest improvement**: Handcrafted evaluator significantly enhanced (+210-360 Elo)

**New features**:

1. **Mobility** - Piece activity scoring
2. **Phase Detection** - Opening/Midgame/Endgame awareness
3. **Enhanced King Safety** - Escape squares, attacker detection
4. **Tactical Patterns** - Passed pawns, bishop pair, open files
5. **Development** - Opening piece development

**Impact on ML**:

- Better self-play games = better training data
- Models trained with v0.3.0 evaluator expected to be +100-200 Elo stronger

See [docs/improvements/EVALUATION_IMPROVEMENTS.md](./improvements/EVALUATION_IMPROVEMENTS.md) for details.

---

## Troubleshooting

### FileNotFoundError: training_data.h5

**Problem**: Board type mismatch between prepare and train

```
FileNotFoundError: 'models/Fair/v0.3.0/training_data.h5'
```

**Solution**: Ensure `--board` matches in both steps:

```bash
# These must match:
python scripts/ml/prepare_dataset.py --boards Fair --version 0.3.0
python scripts/ml/train.py --board Fair --version 0.3.0
```

### ML Feature Not Enabled

**Problem**: Running without `--features ml`

```
error: ML feature not enabled. Rebuild with --features ml
```

**Solution**: Always use the feature flag:

```bash
cargo run --release --features ml
```

### ONNX Model Loading Error

**Problem**: Model version mismatch or corruption

**Solution**:

1. Re-export ONNX: `python scripts/ml/train.py --board Fair --version X.Y.Z --export-only`
2. Verify model path in `ai_config.json`
3. Check ONNX runtime compatibility

### Poor Model Performance

**Possible causes**:

- Insufficient training data (< 1,000 games)
- Overfitting (too many epochs)
- Weak source evaluator for self-play

**Solutions**:

1. Generate more games
2. Monitor training loss (should decrease)
3. Use improved handcrafted evaluator (v0.3.0+)

---

## Tips & Best Practices

### Data Collection

- **Start small**: Test with 100 games first
- **Scale up**: Increase to 5,000+ for serious training
- **Use parallel**: `--parallel 6` on 8-core CPU
- **Mix positions**: Use different board setups

### Training

- **Monitor loss**: Should decrease steadily
- **Early stopping**: Stop if loss plateaus
- **Learning rate**: Start with 0.001, decrease if unstable
- **Batch size**: Larger = faster but more memory

### Versioning

- **Semantic versioning**: v0.1.0, v0.2.0, v0.3.0, etc.
- **Track changes**: Document improvements in version folders
- **Keep old models**: Compare with previous versions

### Evaluation

- **Elo testing**: Compare models with 100+ games
- **Win rate**: Target 55%+ against previous version
- **Diverse testing**: Test on multiple board setups

---

## Performance Metrics

### Expected Improvements

| Version     | Source        | Expected Elo | Win Rate vs Prev |
| ----------- | ------------- | ------------ | ---------------- |
| v0.1.0      | Basic Eval    | Baseline     | -                |
| v0.2.0      | depth=6       | +100         | ~60%             |
| v0.3.0      | Enhanced Eval | +250         | ~70%             |
| v0.4.0 (NN) | v0.3.0 data   | +100         | ~55%             |

### Training Time

**Approximate times** (RTX 3060, 10k samples):

| Epochs | Time    | Notes             |
| ------ | ------- | ----------------- |
| 10     | ~5 min  | Quick test        |
| 50     | ~20 min | Recommended       |
| 100    | ~40 min | Thorough training |

---

## Related Documentation

- [Evaluation Improvements](./improvements/EVALUATION_IMPROVEMENTS.md) - Enhanced handcrafted evaluator
- [ML Data Improvements](./improvements/ML_DATA_IMPROVEMENTS.md) - Data augmentation & labels
- [Index](./INDEX.md) - Documentation overview

---

**Last Updated**: 2026-01-07  
**Current Version**: 0.5.0  
**ML Data Schema**: v0.3.0
