"""
Dataset Preparation for Shogi AI

Loads kifu JSON files and converts them into training data for supervised learning.
Uses Rust binary to extract features from actual board states.
"""

import json
import glob
import numpy as np
import h5py
import subprocess
from pathlib import Path
from typing import List, Tuple, Optional
import argparse

def extract_features_from_kifu(kifu_path: str, binary_path: str) -> Optional[List[dict]]:
    """
    Extract features from a kifu file using Rust binary.
    
    Returns:
        List of dicts with {features, move, player, move_idx}
    """
    try:
        result = subprocess.run(
            [binary_path, kifu_path],
            capture_output=True,
            text=True,
            check=True
        )
        
        # Parse JSON lines output
        examples = []
        for line in result.stdout.strip().split('\n'):
            if line:
                examples.append(json.loads(line))
        
        return examples
    except subprocess.CalledProcessError as e:
        print(f"Error processing {kifu_path}: {e.stderr}")
        return None
    except Exception as e:
        print(f"Unexpected error with {kifu_path}: {e}")
        return None


def encode_move_to_index(move_data: dict) -> int:
    """
    Encode a move as action index.
    
    For simplicity, we use a placeholder encoding.
    Real implementation needs full action space mapping.
    """
    # TODO: Implement proper move encoding
    # For now, return 0 as placeholder
    return 0


def prepare_dataset(kifu_dir: str, output_path: str, boards: List[str], binary_path: str):
    """
    Prepare HDF5 dataset from kifu files.
    
    Args:
        kifu_dir: Directory containing kifu JSON files
        output_path: Output HDF5 file path
        boards: List of board types to include (e.g., ['Fair', 'ShogiOnly'])
        binary_path: Path to extract_features binary
    """
    # Find kifu files for selected boards
    kifu_files = []
    for board in boards:
        pattern = f"{kifu_dir}/{board}/**/*.json"
        kifu_files.extend(glob.glob(pattern, recursive=True))
    
    print(f"Found {len(kifu_files)} kifu files for boards: {', '.join(boards)}")
    
    if not kifu_files:
        print("No kifu files found! Run SelfPlay first.")
        return
    
    all_features = []
    all_moves = []
    all_outcomes = []
    
    for i, file_path in enumerate(kifu_files):
        if i % 10 == 0:
            print(f"Processing {i}/{len(kifu_files)}...")
        
        # Load kifu metadata for outcome
        with open(file_path, 'r') as f:
            kifu_meta = json.load(f)
        
        # Extract features using Rust binary
        examples = extract_features_from_kifu(file_path, binary_path)
        if not examples:
            continue
        
        # Determine outcome (placeholder - need to parse from kifu)
        # In real implementation, derive from game result
        outcome = 0.0  # Draw as default
        
        for example in examples:
            features = example['features']
            move_idx = encode_move_to_index(example['move'])
            
            all_features.append(features)
            all_moves.append(move_idx)
            all_outcomes.append(outcome)
    
    if not all_features:
        print("Warning: No training data extracted! Check kifu files and binary.")
        return
    
    # Convert to numpy arrays
    features_array = np.array(all_features, dtype=np.float32)
    moves_array = np.array(all_moves, dtype=np.int32)
    outcomes_array = np.array(all_outcomes, dtype=np.float32)
    
    print(f"Extracted {len(all_features)} training examples")
    print(f"Feature shape: {features_array.shape}")
    
    # Save to HDF5
    with h5py.File(output_path, 'w') as f:
        f.create_dataset('features', data=features_array)
        f.create_dataset('moves', data=moves_array)
        f.create_dataset('outcomes', data=outcomes_array)
    
    print(f"Dataset saved to {output_path}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Prepare training dataset from kifu files')
    parser.add_argument('--kifu-dir', default='selfplay_kifu', help='Directory containing kifu files')
    parser.add_argument('--output', default='models/training_data.h5', help='Output HDF5 file')
    parser.add_argument('--boards', default='Fair', help='Comma-separated board types (e.g., Fair,ShogiOnly) or "all"')
    parser.add_argument('--binary', default='target/release/extract_features', help='Path to extract_features binary')
    
    args = parser.parse_args()
    
    # Parse board selection
    if args.boards.lower() == 'all':
        boards = ['StandardMixed', 'ReversedMixed', 'ShogiOnly', 'ChessOnly', 'Fair', 'ReversedFair']
    else:
        boards = [b.strip() for b in args.boards.split(',')]
    
    # Ensure binary exists
    binary_path = Path(args.binary)
    if not binary_path.exists():
        print(f"Error: Binary not found at {binary_path}")
        print("Please build it first: cargo build --release --bin extract_features")
        exit(1)
    
    prepare_dataset(args.kifu_dir, args.output, boards, str(binary_path))
