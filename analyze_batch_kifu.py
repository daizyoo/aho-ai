#!/usr/bin/env python3
"""
Batch Kifu Analyzer
Analyzes all kifu files in a directory
"""

import json
from pathlib import Path
from typing import List, Dict
from collections import Counter
import statistics

def load_all_kifus(directory: str) -> List[Dict]:
    """Load all kifu files from directory"""
    kifu_dir = Path(directory)
    
    if not kifu_dir.exists():
        print(f"Error: Directory '{directory}' not found")
        return []
    
    kifus = []
    for filepath in sorted(kifu_dir.glob("*.json")):
        with open(filepath, 'r') as f:
            data = json.load(f)
            data['filename'] = filepath.name
            kifus.append(data)
    
    return kifus

def analyze_all_games(kifus: List[Dict]) -> None:
    """Analyze patterns across all games"""
    all_moves = []
    game_lengths = []
    promotion_rates = []
    drop_rates = []
    
    for kifu in kifus:
        moves = kifu['moves']
        all_moves.extend(moves)
        game_lengths.append(len(moves))
        
        # Promotion rate
        promotions = sum(1 for m in moves if 'Normal' in m and m['Normal'].get('promote', False))
        promotion_rates.append(promotions / len(moves) * 100 if moves else 0)
        
        # Drop rate
        drops = sum(1 for m in moves if 'Drop' in m)
        drop_rates.append(drops / len(moves) * 100 if moves else 0)
    
    print("=== Aggregate Statistics ===")
    print(f"Total Games Analyzed: {len(kifus)}")
    print(f"Total Moves: {len(all_moves)}")
    print(f"Average Game Length: {statistics.mean(game_lengths):.1f} moves")
    print(f"Shortest Game: {min(game_lengths)} moves")
    print(f"Longest Game: {max(game_lengths)} moves")
    print()
    
    print("=== Average Rates ===")
    print(f"Average Promotion Rate: {statistics.mean(promotion_rates):.1f}%")
    print(f"Average Drop Rate: {statistics.mean(drop_rates):.1f}%")
    print()

def analyze_piece_usage(kifus: List[Dict]) -> None:
    """Analyze which pieces are used most"""
    dropped_pieces = []
    
    for kifu in kifus:
        for move in kifu['moves']:
            if 'Drop' in move:
                dropped_pieces.append(move['Drop']['piece_kind'])
    
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
