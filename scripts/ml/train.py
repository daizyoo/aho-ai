import argparse
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import h5py
import numpy as np
import onnx
from model import ShogiNet


class ShogiDataset(Dataset):
    """Dataset loader for HDF5 training data"""
    def __init__(self, h5_path):
        self.h5_path = h5_path
        with h5py.File(h5_path, 'r') as f:
            self.length = len(f['features'])
    
    def __len__(self):
        return self.length
    
    def __getitem__(self, idx):
        with h5py.File(self.h5_path, 'r') as f:
            features = torch.from_numpy(f['features'][idx])
            move = torch.tensor(f['moves'][idx], dtype=torch.long)
            outcome = torch.tensor(f['outcomes'][idx], dtype=torch.float32)
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
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    print(f"Using device: {device}")
    
    # Load dataset
    dataset = ShogiDataset(data_path)
    dataloader = DataLoader(dataset, batch_size=batch_size, shuffle=True, num_workers=0)
    print(f"Loaded {len(dataset)} training examples")
    
    # Initialize model
    model = ShogiNet().to(device)
    criterion_policy = nn.CrossEntropyLoss()
    criterion_value = nn.MSELoss()
    optimizer = optim.Adam(model.parameters(), lr=lr)
    
    # Training loop
    for epoch in range(epochs):
        avg_loss = train_epoch(model, dataloader, criterion_policy, criterion_value, optimizer, device)
        print(f"Epoch {epoch+1}/{epochs}, Loss: {avg_loss:.4f}")
    
    # Save model
    torch.save(model.state_dict(), model_path)
    print(f"Model saved to {model_path}")
    
    # Export to ONNX
    dummy_input = torch.randn(1, 2647).to(device)
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

    # Add version to ONNX metadata
    print(f"Adding version {version} to ONNX metadata...")
    onnx_model = onnx.load(onnx_path)
    meta = onnx_model.metadata_props.add()
    meta.key = "version"
    meta.value = version
    onnx.save(onnx_model, onnx_path)
    print("Metadata updated successfully.")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Train Shogi AI model')
    parser.add_argument('--data', type=str, default='models/training_data.h5', help='Path to h5 dataset')
    parser.add_argument('--output', type=str, default='models/shogi_model.pt', help='Output model path (.pt)')
    parser.add_argument('--epochs', type=int, default=10, help='Number of epochs')
    parser.add_argument('--batch-size', type=int, default=64, help='Batch size')
    parser.add_argument('--version', type=str, default='0.1.0', help='Model version')
    
    args = parser.parse_args()
    
    train(
        data_path=args.data,
        model_path=args.output,
        epochs=args.epochs,
        batch_size=args.batch_size,
        version=args.version
    )
