#!/usr/bin/env python3
"""
Self-Play Results Analyzer
Analyzes JSON results from self-play games
"""

import json
import sys
from pathlib import Path
from typing import Dict, List
import statistics

def load_results(filepath: str) -> Dict:
    """Load self-play results from JSON file"""
    with open(filepath, 'r') as f:
        return json.load(f)

def analyze_basic_stats(data: Dict) -> None:
    """Print basic statistics"""
    print("=== Basic Statistics ===")
    print(f"Total Games: {data['total_games']}")
    print(f"Player 1 Wins: {data['p1_wins']} ({data['p1_wins']/data['total_games']*100:.1f}%)")
    print(f"Player 2 Wins: {data['p2_wins']} ({data['p2_wins']/data['total_games']*100:.1f}%)")
    print(f"Draws: {data['draws']} ({data['draws']/data['total_games']*100:.1f}%)")
    print(f"Average Moves: {data['avg_moves']:.1f}")
    print(f"Average Time: {data['avg_time_ms']/1000:.1f}s")
    print()

def analyze_move_distribution(games: List[Dict]) -> None:
    """Analyze move count distribution"""
    moves = [g['moves'] for g in games]
    
    print("=== Move Distribution ===")
    print(f"Min Moves: {min(moves)}")
    print(f"Max Moves: {max(moves)}")
    print(f"Median Moves: {statistics.median(moves):.1f}")
    print(f"Std Dev: {statistics.stdev(moves):.1f}" if len(moves) > 1 else "Std Dev: N/A")
    print()

def analyze_time_efficiency(games: List[Dict]) -> None:
    """Analyze time efficiency"""
    times = [g['time_ms']/1000 for g in games]
    
    print("=== Time Efficiency ===")
    print(f"Fastest Game: {min(times):.1f}s")
    print(f"Slowest Game: {max(times):.1f}s")
    print(f"Median Time: {statistics.median(times):.1f}s")
    print()

def analyze_win_patterns(games: List[Dict]) -> None:
    """Analyze winning patterns"""
    p1_moves = [g['moves'] for g in games if g.get('winner') == 'Player1']
    p2_moves = [g['moves'] for g in games if g.get('winner') == 'Player2']
    draw_moves = [g['moves'] for g in games if g.get('winner') is None]
    
    print("=== Win Patterns ===")
    if p1_moves:
        print(f"P1 Average Win Length: {statistics.mean(p1_moves):.1f} moves")
    if p2_moves:
        print(f"P2 Average Win Length: {statistics.mean(p2_moves):.1f} moves")
    if draw_moves:
        print(f"Draw Average Length: {statistics.mean(draw_moves):.1f} moves")
    print()

def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze_selfplay.py <results_file.json>")
        print("\nExample:")
        print("  python analyze_selfplay.py selfplay_results_20260104_222146.json")
        sys.exit(1)
    
    filepath = sys.argv[1]
    
    if not Path(filepath).exists():
        print(f"Error: File '{filepath}' not found")
        sys.exit(1)
    
    print(f"Analyzing: {filepath}\n")
    
    data = load_results(filepath)
    
    analyze_basic_stats(data)
    analyze_move_distribution(data['games'])
    analyze_time_efficiency(data['games'])
    analyze_win_patterns(data['games'])
    
    print("Analysis complete!")

if __name__ == "__main__":
    main()
