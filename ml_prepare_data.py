#!/usr/bin/env python3
"""
Kifu to Training Data Converter
Converts kifu files to machine learning training data
"""

import json
import numpy as np
from pathlib import Path
from typing import List, Dict, Tuple
import pickle

# Board representation constants
BOARD_SIZE = 9
PIECE_TYPES = 32  # Different piece types (Shogi + Chess pieces)

def piece_to_index(piece_kind: str) -> int:
    """Convert piece kind to index for one-hot encoding"""
    pieces = [
        'S_Pawn', 'S_Lance', 'S_Knight', 'S_Silver', 'S_Gold', 'S_Bishop', 'S_Rook', 'S_King',
        'S_ProPawn', 'S_ProLance', 'S_ProKnight', 'S_ProSilver', 'S_ProBishop', 'S_ProRook',
        'C_Pawn', 'C_Knight', 'C_Bishop', 'C_Rook', 'C_Queen', 'C_King'
    ]
    try:
        return pieces.index(piece_kind)
    except ValueError:
        return -1

def position_to_coords(pos: Dict) -> Tuple[int, int]:
    """Convert position dict to (x, y) coordinates"""
    return (pos['x'] - 1, pos['y'] - 1)  # Convert to 0-indexed

def encode_move(move: Dict) -> np.ndarray:
    """Encode a move as a feature vector"""
    # Feature vector: [from_x, from_y, to_x, to_y, is_drop, is_promote, piece_type]
    features = np.zeros(7)
    
    if 'Normal' in move:
        m = move['Normal']
        from_x, from_y = position_to_coords(m['from'])
        to_x, to_y = position_to_coords(m['to'])
        
        features[0] = from_x / 8.0  # Normalize to [0, 1]
        features[1] = from_y / 8.0
        features[2] = to_x / 8.0
        features[3] = to_y / 8.0
        features[4] = 0  # Not a drop
        features[5] = 1 if m.get('promote', False) else 0
        
    elif 'Drop' in move:
        m = move['Drop']
        to_x, to_y = position_to_coords(m['to'])
        
        features[0] = 0
        features[1] = 0
        features[2] = to_x / 8.0
        features[3] = to_y / 8.0
        features[4] = 1  # Is a drop
        features[5] = 0
        features[6] = piece_to_index(m['piece_kind']) / 20.0
    
    return features

def extract_game_sequence(kifu: Dict) -> List[np.ndarray]:
    """Extract move sequence from a kifu"""
    return [encode_move(move) for move in kifu['moves']]

def create_training_data(kifu_dir: str, output_file: str = 'training_data.pkl'):
    """Create training dataset from all kifu files"""
    kifu_path = Path(kifu_dir)
    
    if not kifu_path.exists():
        print(f"Error: Directory '{kifu_dir}' not found")
        return
    
    all_sequences = []
    all_labels = []  # 1 for P1 win, 0 for P2 win, 0.5 for draw
    
    print(f"Loading kifus from: {kifu_dir}")
    
    for filepath in sorted(kifu_path.glob("*.json")):
        with open(filepath, 'r') as f:
            kifu = json.load(f)
        
        sequence = extract_game_sequence(kifu)
        all_sequences.append(sequence)
        
        # Note: We don't have winner info in kifu, would need to get from results
        # For now, use placeholder
        all_labels.append(0.5)
    
    print(f"Processed {len(all_sequences)} games")
    
    # Save as pickle
    data = {
        'sequences': all_sequences,
        'labels': all_labels,
        'num_games': len(all_sequences)
    }
    
    with open(output_file, 'wb') as f:
        pickle.dump(data, f)
    
    print(f"Training data saved to: {output_file}")
    print(f"Total games: {len(all_sequences)}")
    print(f"Average game length: {np.mean([len(s) for s in all_sequences]):.1f} moves")

def create_position_value_dataset(kifu_dir: str, results_file: str, output_file: str = 'position_values.pkl'):
    """Create dataset for position evaluation (value network)"""
    # Load results to get winners
    with open(results_file, 'r') as f:
        results = json.load(f)
    
    kifu_path = Path(kifu_dir)
    kifus = []
    
    for filepath in sorted(kifu_path.glob("*.json")):
        with open(filepath, 'r') as f:
            kifus.append(json.load(f))
    
    # Match kifus with results
    positions = []
    values = []
    
    for i, (kifu, game_result) in enumerate(zip(kifus, results['games'])):
        winner = game_result.get('winner')
        
        # Value: 1 for P1 win, -1 for P2 win, 0 for draw
        if winner == 'Player1':
            final_value = 1.0
        elif winner == 'Player2':
            final_value = -1.0
        else:
            final_value = 0.0
        
        # Each position in the game gets a value based on outcome
        for move_idx, move in enumerate(kifu['moves']):
            # Alternate between players
            player = 1 if move_idx % 2 == 0 else -1
            
            # Encode position (simplified: just the move)
            pos_features = encode_move(move)
            
            # Value from current player's perspective
            value = final_value * player
            
            positions.append(pos_features)
            values.append(value)
    
    data = {
        'positions': np.array(positions),
        'values': np.array(values),
        'num_positions': len(positions)
    }
    
    with open(output_file, 'wb') as f:
        pickle.dump(data, f)
    
    print(f"Position value dataset saved to: {output_file}")
    print(f"Total positions: {len(positions)}")

def main():
    import sys
    
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python ml_prepare_data.py <kifu_dir> [results_file]")
        print("\nExamples:")
        print("  python ml_prepare_data.py selfplay_kifu")
        print("  python ml_prepare_data.py selfplay_kifu selfplay_results_*.json")
        sys.exit(1)
    
    kifu_dir = sys.argv[1]
    
    # Create basic training data
    create_training_data(kifu_dir)
    
    # If results file provided, create position value dataset
    if len(sys.argv) >= 3:
        results_file = sys.argv[2]
        create_position_value_dataset(kifu_dir, results_file)
        print("\nBoth datasets created!")
    else:
        print("\nNote: Provide results file to create position value dataset")

if __name__ == "__main__":
    main()
