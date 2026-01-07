# Model Version v0.1.0

## Dataset Information
- **Generated**: 2026-01-07 03:54:33
- **Version**: 0.1.0
- **Board Types**: StandardMixed
- **Total Samples**: 3,344
- **Feature Size**: 2647

## Files
- `training_data.h5` - Training dataset (features, moves, outcomes)

## Usage
```bash
# Train model with this dataset
python scripts/ml/train.py --version 0.1.0
```

## Notes
This dataset was automatically generated from self-play kifu files.
Each sample contains board features, the move played, and the game outcome.

## Model Training
- **Trained**: 2026-01-07 04:12:49
- **Data Source**: `models/StandardMixed/v0.1.0/training_data.h5`
- **Training Samples**: 4,688
- **Epochs**: 20
- **Batch Size**: 64
- **Learning Rate**: 0.001
- **Final Loss**: 0.0000
- **Training Time**: 56.7s
- **Architecture**:
  - Input: 3344 features
  - Hidden: 256 units
  - ResBlocks: 4
  - Policy Head: 7290 actions
  - Value Head: Tanh output [-1, 1]

## Model Files
- `model.pt` - PyTorch model weights
- `model.onnx` - ONNX export for Rust inference (with embedded version metadata)
