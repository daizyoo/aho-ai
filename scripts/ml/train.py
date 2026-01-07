import argparse
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import h5py
import numpy as np
import onnx
import time
from model import ShogiNet


class ShogiDataset(Dataset):
    """Dataset loader for HDF5 training data"""
    def __init__(self, h5_path, use_enhanced_labels=False):
        self.h5_path = h5_path
        self.use_enhanced_labels = use_enhanced_labels
        
        with h5py.File(h5_path, 'r') as f:
            self.length = len(f['features'])
            self.input_size = f['features'].shape[1]
            
            # Check if enhanced labels are available
            self.has_enhanced = 'game_lengths' in f and 'material_diffs' in f
            
            if use_enhanced_labels and not self.has_enhanced:
                print(f"Warning: Enhanced labels requested but not found in {h5_path}")
                print(f"Available datasets: {list(f.keys())}")
                self.use_enhanced_labels = False
    
    def __len__(self):
        return self.length
    
    def __getitem__(self, idx):
        with h5py.File(self.h5_path, 'r') as f:
            features = torch.from_numpy(f['features'][idx])
            move = torch.tensor(f['moves'][idx], dtype=torch.long)
            outcome = torch.tensor(f['outcomes'][idx], dtype=torch.float32)
            
            # Load enhanced labels if available and requested
            if self.use_enhanced_labels and self.has_enhanced:
                game_length = torch.tensor(f['game_lengths'][idx], dtype=torch.float32)
                material_diff = torch.tensor(f['material_diffs'][idx], dtype=torch.float32)
                return features, move, outcome, game_length, material_diff
            
        return features, move, outcome


def train_epoch(model, dataloader, criterion_policy, criterion_value, optimizer, device):
    """Train for one epoch"""
    model.train()
    total_loss = 0.0
    
    for features, moves, outcomes in dataloader:
        features = features.to(device)
        moves = moves.to(device)
        outcomes = outcomes.to(device)
        
        optimizer.zero_grad()
        
        # Forward pass
        policy_logits, value_pred = model(features)
        
        # Loss
        policy_loss = criterion_policy(policy_logits, moves)
        value_loss = criterion_value(value_pred.squeeze(), outcomes)
        loss = policy_loss + value_loss
        
        # Backward pass
        loss.backward()
        optimizer.step()
        
        total_loss += loss.item()
    
    return total_loss / len(dataloader)


def train(data_path, model_path, epochs=10, batch_size=64, lr=0.001, version='0.1.0'):
    """Main training loop"""
    start_time = time.time()
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    print(f"Using device: {device}")
    
    # Load dataset
    dataset = ShogiDataset(data_path)
    dataloader = DataLoader(dataset, batch_size=batch_size, shuffle=True, num_workers=0)
    print(f"Loaded {len(dataset)} training examples")
    
    # Initialize model
    model = ShogiNet(input_size=dataset.input_size).to(device)
    criterion_policy = nn.CrossEntropyLoss()
    criterion_value = nn.MSELoss()
    optimizer = optim.Adam(model.parameters(), lr=lr)
    
    # Training loop
    final_loss = 0.0
    for epoch in range(epochs):
        avg_loss = train_epoch(model, dataloader, criterion_policy, criterion_value, optimizer, device)
        final_loss = avg_loss
        print(f"Epoch {epoch+1}/{epochs}, Loss: {avg_loss:.4f}")
    
    # Save model
    torch.save(model.state_dict(), model_path)
    print(f"Model saved to {model_path}")
    
    # Export to ONNX
    dummy_input = torch.randn(1, dataset.input_size).to(device)
    onnx_path = model_path.replace('.pt', '.onnx')
    torch.onnx.export(
        model,
        dummy_input,
        onnx_path,
        input_names=['board_features'],
        output_names=['policy', 'value'],
        dynamic_axes={'board_features': {0: 'batch_size'}}
    )
    print(f"ONNX model exported to {onnx_path}")

    training_time = time.time() - start_time
    
    return {
        'final_loss': final_loss,
        'training_time': training_time,
        'num_samples': len(dataset),
        'input_size': dataset.input_size
    }


def update_readme(output_dir: str, version: str, data_path: str, epochs: int, batch_size: int, 
                  lr: float, stats: dict):
    """Update or create README.md with training information"""
    from datetime import datetime
    from pathlib import Path
    
    readme_path = Path(output_dir) / "README.md"
    
    # Read existing content if it exists
    existing_content = ""
    if readme_path.exists():
        with open(readme_path, 'r') as f:
            existing_content = f.read()
    
    # Append training information
    training_info = f"""
## Model Training
- **Trained**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
- **Data Source**: `{data_path}`
- **Training Samples**: {stats.get('num_samples', 'N/A'):,}
- **Epochs**: {epochs}
- **Batch Size**: {batch_size}
- **Learning Rate**: {lr}
- **Final Loss**: {stats.get('final_loss', 0.0):.4f}
- **Training Time**: {stats.get('training_time', 0.0):.1f}s
- **Architecture**:
  - Input: {stats.get('input_size', 'Unknown')} features
  - Hidden: 256 units
  - ResBlocks: 4
  - Policy Head: 7290 actions
  - Value Head: Tanh output [-1, 1]

## Model Files
- `model.pt` - PyTorch model weights
- `model.onnx` - ONNX export for Rust inference (with embedded version metadata)
"""
    
    # If README exists, replace or append training section
    if "## Model Training" in existing_content:
        # Replace existing training section
        parts = existing_content.split("## Model Training")
        updated_content = parts[0] + training_info
    else:
        # Append to existing content
        updated_content = existing_content + training_info
    
    with open(readme_path, 'w') as f:
        f.write(updated_content)
    
    print(f"README updated at {readme_path}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Train Shogi AI model')
    parser.add_argument('--data', type=str, help='Path to h5 dataset (optional, defaults to models/{board_type}/v{version}/training_data.h5)')
    parser.add_argument('--output', type=str, help='Output model path (optional, overrides version-based path)')
    parser.add_argument('--epochs', type=int, default=10, help='Number of epochs')
    parser.add_argument('--batch-size', type=int, default=64, help='Batch size')
    parser.add_argument('--version', type=str, default='0.1.0', help='Model version')
    parser.add_argument('--board-type', type=str, default='Fair', help='Board type (Fair, ChessOnly, ShogiOnly, ALL, etc.)')
    
    args = parser.parse_args()
    
    
    # Determine data path
    if args.data:
        data_path = args.data
    else:
        # Default: models/{board_type}/v{version}/training_data.h5
        data_path = f"models/{args.board_type}/v{args.version}/training_data.h5"
    
    # Determine output path
    if args.output:
        model_path = args.output
    else:
        # Default: models/{board_type}/v{version}/model.pt
        import os
        output_dir = f"models/{args.board_type}/v{args.version}"
        os.makedirs(output_dir, exist_ok=True)
        model_path = f"{output_dir}/model.pt"
        
        # Also check if data path needs to be updated relative to version?
        # For now, we assume data might be shared or specific.
        # If the user wants specific data, they should pass --data
    
    # Ensure directory exists for explicit output path too
    output_dir = os.path.dirname(model_path)
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)

    print(f"Training version {args.version}")
    print(f"Board type: {args.board_type}")
    print(f"Data path: {data_path}")
    print(f"Output path: {model_path}")
    
    stats = train(
        data_path=data_path,
        model_path=model_path,
        epochs=args.epochs,
        batch_size=args.batch_size,
        lr=0.001,
        version=args.version
    )
    
    # Update README if using version-based directory
    if not args.output:  # Only for auto-generated version directories
        output_dir_path = os.path.dirname(model_path)
        update_readme(output_dir_path, args.version, data_path, args.epochs, args.batch_size, 0.001, stats)
