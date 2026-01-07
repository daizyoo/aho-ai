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


def is_position_symmetric(board_setup: str) -> bool:
    """
    Check if board setup allows horizontal flipping.
    
    Symmetric setups (Fair, StandardMixed) can be safely flipped.
    Asymmetric setups (ShogiOnly, ChessOnly) should not be flipped
    as they have directional pieces and asymmetric promotion zones.
    """
    symmetric_setups = ['Fair', 'StandardMixed', 'ReversedFair']
    return board_setup in symmetric_setups


def flip_horizontal(features: np.ndarray) -> np.ndarray:
    """
    Flip board features horizontally (x -> 8-x).
    
    Args:
        features: Feature vector of shape (3344,)
                  Layout: [board: 9x9x41=3321] + [hand: 22] + [turn: 1]
    
    Returns:
        Flipped feature vector
    """
    features = features.copy()
    
    # Constants from features.rs
    BOARD_SIZE = 9
    NUM_PIECE_TYPES = 41
    BOARD_FEATURES = BOARD_SIZE * BOARD_SIZE * NUM_PIECE_TYPES  # 3321
    HAND_FEATURES = 22
    TURN_FEATURE = 1
    
    # Extract components
    board_features = features[:BOARD_FEATURES]
    hand_features = features[BOARD_FEATURES:BOARD_FEATURES + HAND_FEATURES]
    turn_feature = features[BOARD_FEATURES + HAND_FEATURES:]
    
    # Reshape board to (9, 9, 41)
    board_3d = board_features.reshape(BOARD_SIZE, BOARD_SIZE, NUM_PIECE_TYPES)
    
    # Flip horizontally (along x-axis, which is axis=1 in our layout)
    # Note: In the feature extraction, we iterate y then x, so flipping x means flipping axis=1
    board_3d_flipped = np.flip(board_3d, axis=1)
    
    # Flatten back
    board_flipped = board_3d_flipped.reshape(-1)
    
    # Hand pieces and turn indicator remain unchanged (they're not positional)
    flipped_features = np.concatenate([board_flipped, hand_features, turn_feature])
    
    return flipped_features


def augment_position(features: np.ndarray, board_setup: str) -> List[np.ndarray]:
    """
    Apply valid transformations to board position.
    
    For symmetric setups: add horizontal flip
    For asymmetric setups: return only original
    
    Returns: List of augmented feature vectors (including original)
    """
    augmented = [features]  # Always include original
    
    # Only augment if board setup allows it
    if is_position_symmetric(board_setup):
        flipped = flip_horizontal(features)
        augmented.append(flipped)
    
    return augmented



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
    all_game_lengths = []
    all_material_diffs = []
    all_augmented_flags = []
    
    total_games_processed = 0
    total_augmented_samples = 0
    
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
        
        # Determine board type from file path
        board_setup = None
        for board in boards:
            if board in file_path:
                board_setup = board
                break
        
        if not board_setup:
            board_setup = boards[0]  # Fallback
        
        # Parse outcome from kifu metadata
        # This is a placeholder - actual implementation depends on kifu format
        outcome = 0.0  # Draw as default
        game_length = len(examples)  # Number of moves
        material_diff = 0  # Placeholder - would need final board state
        
        total_games_processed += 1
        
        for example in examples:
            features = np.array(example['features'], dtype=np.float32)
            move_idx = encode_move_to_index(example['move'])
            
            # Apply data augmentation
            augmented_samples = augment_position(features, board_setup)
            
            for aug_idx, aug_features in enumerate(augmented_samples):
                is_augmented = (aug_idx > 0)  # First sample is  original
                
                all_features.append(aug_features)
                all_moves.append(move_idx)
                all_outcomes.append(outcome)
                all_game_lengths.append(game_length)
                all_material_diffs.append(material_diff)
                all_augmented_flags.append(is_augmented)
            
            if len(augmented_samples) > 1:
                total_augmented_samples += (len(augmented_samples) - 1)
    
    if not all_features:
        print("Warning: No training data extracted! Check kifu files and binary.")
        return
    
    # Convert to numpy arrays
    features_array = np.array(all_features, dtype=np.float32)
    moves_array = np.array(all_moves, dtype=np.int32)
    outcomes_array = np.array(all_outcomes, dtype=np.float32)
    game_lengths_array = np.array(all_game_lengths, dtype=np.int32)
    material_diffs_array = np.array(all_material_diffs, dtype=np.int32)
    augmented_flags_array = np.array(all_augmented_flags, dtype=np.bool_)
    
    print(f"\nDataset Summary:")
    print(f"  Games Processed: {total_games_processed}")
    print(f"  Total Samples: {len(all_features)}")
    print(f"  Original Samples: {len(all_features) - total_augmented_samples}")
    print(f"  Augmented Samples: {total_augmented_samples}")
    print(f"  Augmentation Rate: {total_augmented_samples / len(all_features) * 100:.1f}%")
    print(f"  Feature Shape: {features_array.shape}")
    
    # Ensure output directory exists
    output_dir = Path(output_path).parent
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Save to HDF5 with enhanced schema
    with h5py.File(output_path, 'w') as f:
        f.create_dataset('features', data=features_array, compression='gzip')
        f.create_dataset('moves', data=moves_array, compression='gzip')
        f.create_dataset('outcomes', data=outcomes_array, compression='gzip')
        f.create_dataset('game_lengths', data=game_lengths_array, compression='gzip')
        f.create_dataset('material_diffs', data=material_diffs_array, compression='gzip')
        f.create_dataset('augmented', data=augmented_flags_array, compression='gzip')
        
        # Store metadata
        f.attrs['version'] = '0.2.0'
        f.attrs['num_games'] = total_games_processed
        f.attrs['num_samples'] = len(all_features)
        f.attrs['augmentation_enabled'] = True
    
    print(f"\nDataset saved to {output_path}")
    return len(all_features)


def generate_readme(output_dir: str, version: str, boards: list, num_samples: int, feature_size: int = 2647):
    """Generate README.md for the dataset version"""
    from datetime import datetime
    
    readme_path = Path(output_dir) / "README.md"
    
    readme_content = f"""# Model Version v{version}

## Dataset Information
- **Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
- **Version**: {version}
- **Board Types**: {', '.join(boards)}
- **Total Samples**: {num_samples:,}
- **Feature Size**: {feature_size}

## Files
- `training_data.h5` - Training dataset (features, moves, outcomes)

## Usage
```bash
# Train model with this dataset
python scripts/ml/train.py --version {version}
```

## Notes
This dataset was automatically generated from self-play kifu files.
Each sample contains board features, the move played, and the game outcome.
"""
    
    with open(readme_path, 'w') as f:
        f.write(readme_content)
    
    print(f"README generated at {readme_path}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Prepare training dataset from kifu files')
    parser.add_argument('--kifu-dir', default='selfplay_kifu', help='Directory containing kifu files')
    parser.add_argument('--output', help='Output HDF5 file (optional, overrides version-based path)')
    parser.add_argument('--boards', default='Fair', help='Comma-separated board types (e.g., Fair,ShogiOnly) or "all"')
    parser.add_argument('--binary', default='target/release/extract_features', help='Path to extract_features binary')
    parser.add_argument('--version', default='0.1.0', help='Dataset version')
    
    args = parser.parse_args()
    
    
    # Parse board selection
    if args.boards.lower() == 'all':
        boards = ['StandardMixed', 'ReversedMixed', 'ShogiOnly', 'ChessOnly', 'Fair', 'ReversedFair']
        board_type_name = "ALL"
    else:
        boards = [b.strip() for b in args.boards.split(',')]
        if len(boards) == 1:
            board_type_name = boards[0]
        else:
            board_type_name = "_".join(boards)

    # Determine output path
    if args.output:
        output_path = args.output
    else:
        # Default: models/{board_type}/v{version}/training_data.h5
        import os
        output_dir = f"models/{board_type_name}/v{args.version}"
        os.makedirs(output_dir, exist_ok=True)
        output_path = f"{output_dir}/training_data.h5"
    
    # Ensure directory exists for explicit output path too
    output_dir = Path(output_path).parent
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Ensure binary exists
    binary_path = Path(args.binary)
    if not binary_path.exists():
        print(f"Error: Binary not found at {binary_path}")
        print("Please build it first: cargo build --release --bin extract_features")
        exit(1)
    
    print(f"Dataset version: {args.version}")
    print(f"Output path: {output_path}")
    
    # Prepare dataset
    num_samples = prepare_dataset(args.kifu_dir, output_path, boards, str(binary_path))
    
    # Generate README if using version-based directory
    if not args.output:  # Only for auto-generated version directories
        output_dir_path = Path(output_path).parent
        generate_readme(str(output_dir_path), args.version, boards, num_samples if num_samples else 0)
