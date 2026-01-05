#!/usr/bin/env python3
"""
Test if evaluation function is symmetric for both players
"""

import json
from pathlib import Path

def analyze_first_moves():
    """Analyze if both players make similar first moves"""
    kifu_dir = Path("selfplay_kifu")
    
    p1_first_moves = {}
    p2_first_moves = {}
    
    for kifu_file in kifu_dir.glob("*.json"):
        with open(kifu_file) as f:
            data = json.load(f)
            moves = data.get("moves", [])
            
            if len(moves) >= 2:
                # P1's first move
                p1_move = moves[0]
                if "Normal" in p1_move:
                    from_pos = (p1_move["Normal"]["from"]["x"], p1_move["Normal"]["from"]["y"])
                    to_pos = (p1_move["Normal"]["to"]["x"], p1_move["Normal"]["to"]["y"])
                    key = f"{from_pos} -> {to_pos}"
                    p1_first_moves[key] = p1_first_moves.get(key, 0) + 1
                
                # P2's first move
                p2_move = moves[1]
                if "Normal" in p2_move:
                    from_pos = (p2_move["Normal"]["from"]["x"], p2_move["Normal"]["from"]["y"])
                    to_pos = (p2_move["Normal"]["to"]["x"], p2_move["Normal"]["to"]["y"])
                    key = f"{from_pos} -> {to_pos}"
                    p2_first_moves[key] = p2_first_moves.get(key, 0) + 1
    
    print("="*60)
    print("FIRST MOVE ANALYSIS")
    print("="*60)
    print("\nPlayer 1 First Moves:")
    for move, count in sorted(p1_first_moves.items(), key=lambda x: -x[1])[:5]:
        print(f"  {move}: {count} times")
    
    print("\nPlayer 2 First Moves:")
    for move, count in sorted(p2_first_moves.items(), key=lambda x: -x[1])[:5]:
        print(f"  {move}: {count} times")
    
    # Check if moves are symmetric
    print("\n" + "="*60)
    if len(p1_first_moves) == 1 and len(p2_first_moves) > 3:
        print("⚠️  WARNING: P1 always makes the same move, P2 varies")
        print("This suggests evaluation is NOT symmetric!")
    elif len(p2_first_moves) == 1 and len(p1_first_moves) > 3:
        print("⚠️  WARNING: P2 always makes the same move, P1 varies")
        print("This suggests evaluation is NOT symmetric!")
    else:
        print("✓ Both players show move variety")

if __name__ == "__main__":
    analyze_first_moves()
