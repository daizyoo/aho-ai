#!/usr/bin/env python3
"""
Kifu to Training Data Converter
Converts kifu files to machine learning training data (no numpy required)
"""

import json
from pathlib import Path
from typing import List, Dict, Tuple
import pickle

def piece_to_index(piece_kind: str) -> int:
    """Convert piece kind to index"""
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

def encode_move(move: Dict) -> List[float]:
    """Encode a move as a feature vector (list instead of numpy array)"""
    # Feature vector: [from_x, from_y, to_x, to_y, is_drop, is_promote, piece_type]
    features = [0.0] * 7
    
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
        piece = m.get('piece_kind') or m.get('kind', '')
        
        features[0] = 0
        features[1] = 0
        features[2] = to_x / 8.0
        features[3] = to_y / 8.0
        features[4] = 1  # Is a drop
        features[5] = 0
        features[6] = piece_to_index(piece) / 20.0
    
    return features

def extract_game_sequence(kifu: Dict) -> List[List[float]]:
    """Extract move sequence from a kifu"""
    return [encode_move(move) for move in kifu.get('moves', [])]

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
        try:
            with open(filepath, 'r') as f:
                kifu = json.load(f)
            
            sequence = extract_game_sequence(kifu)
            all_sequences.append(sequence)
            
            # Placeholder label (would need results file for actual labels)
            all_labels.append(0.5)
        except Exception as e:
            print(f"Warning: Failed to process {filepath.name}: {e}")
    
    print(f"Processed {len(all_sequences)} games")
    
    # Calculate average game length
    avg_length = sum(len(s) for s in all_sequences) / len(all_sequences) if all_sequences else 0
    
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
    print(f"Average game length: {avg_length:.1f} moves")

def main():
    import sys
    
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python ml_prepare_data.py <kifu_dir>")
        print("\nExample:")
        print("  python ml_prepare_data.py selfplay_kifu")
        sys.exit(1)
    
    kifu_dir = sys.argv[1]
    
    # Create basic training data
    create_training_data(kifu_dir)
    
    print("\nNote: This creates basic training data.")
    print("For ML training, install: pip install tensorflow numpy")

if __name__ == "__main__":
    main()
