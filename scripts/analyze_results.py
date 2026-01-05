#!/usr/bin/env python3
"""
Analyze self-play game results and display statistics
Aggregates all results grouped by game type
"""

import json
from pathlib import Path
from collections import defaultdict

def load_all_results():
    """Load all results files and group by game type"""
    results_dir = Path("selfplay_results")
    if not results_dir.exists():
        print("selfplay_results directory not found")
        return {}
        
    results_files = list(results_dir.glob("selfplay_results_*.json"))
    
    if not results_files:
        print("No results files found in selfplay_results/")
        return {}
    
    # Group by game type (board_setup + AI strengths)
    grouped = defaultdict(lambda: {
        'total_games': 0,
        'p1_wins': 0,
        'p2_wins': 0,
        'draws': 0,
        'total_moves': 0,
        'total_time_ms': 0,
        'files': []
    })
    
    for results_file in results_files:
        with open(results_file) as f:
            data = json.load(f)
        
        # Create game type key
        game_type = f"{data['board_setup']} ({data['ai1_strength']} vs {data['ai2_strength']})"
        
        # Aggregate data
        group = grouped[game_type]
        group['total_games'] += data['total_games']
        group['p1_wins'] += data['p1_wins']
        group['p2_wins'] += data['p2_wins']
        group['draws'] += data['draws']
        group['total_moves'] += data['avg_moves'] * data['total_games']
        group['total_time_ms'] += data['avg_time_ms'] * data['total_games']
        group['files'].append(results_file.name)
        group['board_setup'] = data['board_setup']
        group['ai1_strength'] = data['ai1_strength']
        group['ai2_strength'] = data['ai2_strength']
    
    return grouped

def display_game_type_statistics(game_type, data):
    """Display statistics for a specific game type"""
    print("=" * 70)
    print(f"GAME TYPE: {game_type}")
    print("=" * 70)
    print()
    
    total = data['total_games']
    if total == 0:
        print("No games played")
        return
    
    # Calculate averages
    avg_moves = data['total_moves'] / total
    avg_time_s = data['total_time_ms'] / total / 1000
    
    # Win rates
    p1_rate = (data['p1_wins'] / total * 100) if total > 0 else 0
    p2_rate = (data['p2_wins'] / total * 100) if total > 0 else 0
    draw_rate = (data['draws'] / total * 100) if total > 0 else 0
    
    print(f"Total Games: {total}")
    print(f"Source Files: {len(data['files'])}")
    print()
    
    print("-" * 70)
    print("WIN RATES")
    print("-" * 70)
    print(f"Player 1: {data['p1_wins']:4d} wins ({p1_rate:5.1f}%)")
    print(f"Player 2: {data['p2_wins']:4d} wins ({p2_rate:5.1f}%)")
    print(f"Draws:    {data['draws']:4d}      ({draw_rate:5.1f}%)")
    print()
    
    # Balance analysis
    print("-" * 70)
    print("BALANCE ANALYSIS")
    print("-" * 70)
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
    print("-" * 70)
    print("GAME CHARACTERISTICS")
    print("-" * 70)
    print(f"Average moves:  {avg_moves:.1f}")
    print(f"Average time:   {avg_time_s:.1f}s")
    print()
    
    # Sample size assessment
    print("-" * 70)
    print("SAMPLE SIZE")
    print("-" * 70)
    if total < 10:
        print("⚠ Very small sample (< 10 games)")
        print("  → Run more games for reliable statistics")
    elif total < 50:
        print("⚠ Small sample (< 50 games)")
        print("  → Consider running more games")
    elif total < 100:
        print("✓ Moderate sample size")
        print("  → Results are fairly reliable")
    else:
        print("✓ Large sample size")
        print("  → Results are statistically significant")
    
    if win_diff > 15 and total >= 10:
        print("\n⚠ Win rate imbalance detected")
        print("  → Consider investigating AI behavior")
    
    print()

def display_summary(grouped_data):
    """Display overall summary"""
    if not grouped_data:
        return
    
    print("=" * 70)
    print("OVERALL SUMMARY")
    print("=" * 70)
    print()
    
    total_games = sum(data['total_games'] for data in grouped_data.values())
    total_types = len(grouped_data)
    
    print(f"Total Game Types: {total_types}")
    print(f"Total Games Played: {total_games}")
    print()
    
    print("Game Types:")
    for game_type, data in sorted(grouped_data.items()):
        print(f"  • {game_type}: {data['total_games']} games")
    print()

if __name__ == "__main__":
    grouped_data = load_all_results()
    
    if not grouped_data:
        print("No game results found")
    else:
        # Display summary first
        display_summary(grouped_data)
        
        # Display statistics for each game type
        for game_type in sorted(grouped_data.keys()):
            display_game_type_statistics(game_type, grouped_data[game_type])
