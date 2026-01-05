#!/usr/bin/env python3
"""
Analyze self-play game results and display statistics
"""

import json
import sys
from pathlib import Path
from datetime import datetime

def load_latest_results():
    """Load the most recent results file"""
    results_files = sorted(Path("selfplay_results").glob("selfplay_results_*.json"), 
                          key=lambda p: p.stat().st_mtime, reverse=True)
    
    if not results_files:
        print("No results files found")
        return None
    
    results_file = results_files[0]
    print(f"Loading: {results_file.name}\n")
    
    with open(results_file) as f:
        return json.load(f)

def display_statistics(results):
    """Display comprehensive statistics"""
    print("=" * 60)
    print("SELF-PLAY RESULTS ANALYSIS")
    print("=" * 60)
    print()
    
    # Basic stats
    print(f"Total Games: {results['total_games']}")
    print(f"Board Setup: {results['board_setup']}")
    print(f"AI Strength: {results['ai1_strength']} vs {results['ai2_strength']}")
    print()
    
    # Win rates
    print("-" * 60)
    print("WIN RATES")
    print("-" * 60)
    total = results['total_games']
    p1_rate = (results['p1_wins'] / total * 100) if total > 0 else 0
    p2_rate = (results['p2_wins'] / total * 100) if total > 0 else 0
    draw_rate = (results['draws'] / total * 100) if total > 0 else 0
    
    print(f"Player 1: {results['p1_wins']:3d} wins ({p1_rate:5.1f}%)")
    print(f"Player 2: {results['p2_wins']:3d} wins ({p2_rate:5.1f}%)")
    print(f"Draws:    {results['draws']:3d}      ({draw_rate:5.1f}%)")
    print()
    
    # Balance analysis
    print("-" * 60)
    print("BALANCE ANALYSIS")
    print("-" * 60)
    win_diff = abs(p1_rate - p2_rate)
    if win_diff < 5:
        status = "✓ Excellent balance"
    elif win_diff < 10:
        status = "✓ Good balance"
    elif win_diff < 20:
        status = "⚠ Slight imbalance"
    else:
        status = "⚠ Significant imbalance"
    
    print(f"Win rate difference: {win_diff:.1f}%")
    print(f"Status: {status}")
    print()
    
    # Game characteristics
    print("-" * 60)
    print("GAME CHARACTERISTICS")
    print("-" * 60)
    print(f"Average moves:  {results['avg_moves']:.1f}")
    print(f"Average time:   {results['avg_time_ms']/1000:.1f}s")
    print()
    
    # Recommendations
    print("-" * 60)
    print("RECOMMENDATIONS")
    print("-" * 60)
    if total < 10:
        print("⚠ Sample size too small (< 10 games)")
        print("  → Run more games for reliable statistics")
    elif total < 50:
        print("⚠ Small sample size (< 50 games)")
        print("  → Consider running more games")
    elif total < 100:
        print("✓ Moderate sample size")
        print("  → Results are fairly reliable")
    else:
        print("✓ Large sample size")
        print("  → Results are statistically significant")
    
    if win_diff > 15:
        print("\n⚠ Win rate imbalance detected")
        print("  → Consider investigating AI behavior")
        print("  → Check evaluation function symmetry")
    
    print()

def compare_results():
    """Compare multiple result files"""
    results_files = sorted(Path("selfplay_results").glob("selfplay_results_*.json"), 
                          key=lambda p: p.stat().st_mtime, reverse=True)[:5]
    
    if len(results_files) < 2:
        print("Not enough result files to compare")
        return
    
    print("=" * 60)
    print("HISTORICAL COMPARISON (Last 5 runs)")
    print("=" * 60)
    print()
    
    print(f"{'Date':<20} {'Games':>6} {'P1 Win%':>8} {'P2 Win%':>8} {'Avg Moves':>10}")
    print("-" * 60)
    
    for results_file in results_files:
        with open(results_file) as f:
            data = json.load(f)
        
        # Extract timestamp from filename
        timestamp = results_file.stem.split('_')[-2:]
        date_str = f"{timestamp[0]} {timestamp[1]}"
        
        total = data['total_games']
        p1_rate = (data['p1_wins'] / total * 100) if total > 0 else 0
        p2_rate = (data['p2_wins'] / total * 100) if total > 0 else 0
        
        print(f"{date_str:<20} {total:6d} {p1_rate:7.1f}% {p2_rate:7.1f}% {data['avg_moves']:10.1f}")
    
    print()

if __name__ == "__main__":
    results = load_latest_results()
    
    if results:
        display_statistics(results)
        
        # Show comparison if available
        if len(list(Path("../selfplay_results").glob("selfplay_results_*.json"))) > 1:
            compare_results()
