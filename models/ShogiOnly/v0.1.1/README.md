# Model Version v0.1.1

## Dataset Information
- **Generated**: 2026-01-07 21:13:45
- **Version**: 0.1.1
- **Board Types**: ShogiOnly
- **Total Samples**: 3,344
- **Feature Size**: 2647

## Files
- `training_data.h5` - Training dataset (features, moves, outcomes)

## Usage
```bash
# Train model with this dataset
python scripts/ml/train.py --version 0.1.1
```

## Notes
This dataset was automatically generated from self-play kifu files.
Each sample contains board features, the move played, and the game outcome.

## Model Training
- **Trained**: 2026-01-07 21:18:15
- **Data Source**: `models/ShogiOnly/v0.1.1/training_data.h5`
- **Training Samples**: 11,345
- **Epochs**: 20
- **Batch Size**: 64
- **Learning Rate**: 0.001
- **Final Loss**: 0.0000
- **Training Time**: 70.9s
- **Architecture**:
  - Input: 3344 features
  - Hidden: 256 units
  - ResBlocks: 4
  - Policy Head: 7290 actions
  - Value Head: Tanh output [-1, 1]

## Model Files
- `model.pt` - PyTorch model weights
- `model.onnx` - ONNX export for Rust inference (with embedded version metadata)
