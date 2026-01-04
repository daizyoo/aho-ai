#!/usr/bin/env python3
"""
Kifu Analyzer
Analyzes game records (kifu) from self-play
"""

import json
import sys
from pathlib import Path
from typing import Dict, List
from collections import Counter

def load_kifu(filepath: str) -> Dict:
    """Load kifu from JSON file"""
    with open(filepath, 'r') as f:
        return json.load(f)

def analyze_move_types(moves: List[Dict]) -> None:
    """Analyze types of moves"""
    move_types = []
    
    for move in moves:
        if 'Normal' in move:
            move_types.append('Normal')
        elif 'Drop' in move:
            move_types.append('Drop')
    
    counter = Counter(move_types)
    
    print("=== Move Types ===")
    for move_type, count in counter.items():
        print(f"{move_type}: {count} ({count/len(moves)*100:.1f}%)")
    print()

def analyze_promotions(moves: List[Dict]) -> None:
    """Analyze promotion frequency"""
    promotions = 0
    
    for move in moves:
        if 'Normal' in move and move['Normal'].get('promote', False):
            promotions += 1
    
    print("=== Promotions ===")
    print(f"Total Promotions: {promotions}")
    print(f"Promotion Rate: {promotions/len(moves)*100:.1f}%")
    print()

def analyze_drops(moves: List[Dict]) -> None:
    """Analyze piece drops"""
    drops = [m for m in moves if 'Drop' in m]
    
    if not drops:
        print("=== Drops ===")
        print("No drops in this game")
        print()
        return
    
    drop_pieces = []
    for drop in drops:
        piece = drop['Drop']['piece_kind']
        drop_pieces.append(piece)
    
    counter = Counter(drop_pieces)
    
    print("=== Drops ===")
    print(f"Total Drops: {len(drops)}")
    for piece, count in counter.most_common():
        print(f"  {piece}: {count}")
    print()

def analyze_opening(moves: List[Dict], n: int = 10) -> None:
    """Analyze opening moves"""
    opening = moves[:min(n, len(moves))]
    
    print(f"=== Opening ({n} moves) ===")
    for i, move in enumerate(opening, 1):
        if 'Normal' in move:
            m = move['Normal']
            promote = " (promote)" if m.get('promote', False) else ""
            print(f"{i}. {m['from']} -> {m['to']}{promote}")
        elif 'Drop' in move:
            m = move['Drop']
            print(f"{i}. Drop {m['piece_kind']} at {m['to']}")
    print()

def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze_kifu.py <kifu_file.json>")
        print("\nExample:")
        print("  python analyze_kifu.py selfplay_kifu/game_0001_20260104_220000.json")
        sys.exit(1)
    
    filepath = sys.argv[1]
    
    if not Path(filepath).exists():
        print(f"Error: File '{filepath}' not found")
        sys.exit(1)
    
    print(f"Analyzing: {filepath}\n")
    
    data = load_kifu(filepath)
    moves = data['moves']
    
    print(f"Total Moves: {len(moves)}\n")
    
    analyze_move_types(moves)
    analyze_promotions(moves)
    analyze_drops(moves)
    analyze_opening(moves)
    
    print("Analysis complete!")

if __name__ == "__main__":
    main()
