#!/usr/bin/env python3
"""
Batch Kifu Analyzer
Analyzes all kifu files in a directory
"""

import json
from pathlib import Path
from typing import List, Dict
from collections import Counter

def load_all_kifus(directory: str) -> List[Dict]:
    """Load all kifu files from directory"""
    kifu_dir = Path(directory)
    
    if not kifu_dir.exists():
        print(f"Error: Directory '{directory}' not found")
        return []
    
    kifus = []
    for filepath in sorted(kifu_dir.glob("*.json")):
        try:
            with open(filepath, 'r') as f:
                data = json.load(f)
                data['filename'] = filepath.name
                kifus.append(data)
        except Exception as e:
            print(f"Warning: Failed to load {filepath.name}: {e}")
    
    return kifus

def calculate_average(values: List[float]) -> float:
    """Calculate average, handling empty lists"""
    return sum(values) / len(values) if values else 0.0

def analyze_all_games(kifus: List[Dict]) -> None:
    """Analyze patterns across all games"""
    all_moves = []
    game_lengths = []
    promotion_rates = []
    drop_rates = []
    
    for kifu in kifus:
        moves = kifu.get('moves', [])
        all_moves.extend(moves)
        game_lengths.append(len(moves))
        
        # Promotion rate
        promotions = sum(1 for m in moves 
                        if isinstance(m, dict) and 'Normal' in m 
                        and m['Normal'].get('promote', False))
        promotion_rates.append(promotions / len(moves) * 100 if moves else 0)
        
        # Drop rate
        drops = sum(1 for m in moves if isinstance(m, dict) and 'Drop' in m)
        drop_rates.append(drops / len(moves) * 100 if moves else 0)
    
    if not game_lengths:
        print("No games found!")
        return
    
    print("=== Aggregate Statistics ===")
    print(f"Total Games Analyzed: {len(kifus)}")
    print(f"Total Moves: {len(all_moves)}")
    print(f"Average Game Length: {calculate_average(game_lengths):.1f} moves")
    print(f"Shortest Game: {min(game_lengths)} moves")
    print(f"Longest Game: {max(game_lengths)} moves")
    print()
    
    print("=== Average Rates ===")
    print(f"Average Promotion Rate: {calculate_average(promotion_rates):.1f}%")
    print(f"Average Drop Rate: {calculate_average(drop_rates):.1f}%")
    print()

def analyze_piece_usage(kifus: List[Dict]) -> None:
    """Analyze which pieces are used most"""
    dropped_pieces = []
    
    for kifu in kifus:
        for move in kifu.get('moves', []):
            if isinstance(move, dict) and 'Drop' in move:
                drop_data = move['Drop']
                # Handle both 'piece_kind' and 'kind' field names
                piece = drop_data.get('piece_kind') or drop_data.get('kind')
                if piece:
                    dropped_pieces.append(piece)
    
    if not dropped_pieces:
        print("=== Piece Drop Usage ===")
        print("No drops found across all games")
        print()
        return
    
    counter = Counter(dropped_pieces)
    
    print("=== Piece Drop Usage (All Games) ===")
    for piece, count in counter.most_common():
        print(f"{piece}: {count} drops")
    print()

def main():
    directory = "selfplay_kifu"
    
    print(f"Loading kifus from: {directory}\n")
    
    kifus = load_all_kifus(directory)
    
    if not kifus:
        print("No kifu files found!")
        return
    
    analyze_all_games(kifus)
    analyze_piece_usage(kifus)
    
    print("Batch analysis complete!")

if __name__ == "__main__":
    main()
