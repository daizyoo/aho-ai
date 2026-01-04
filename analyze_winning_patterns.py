#!/usr/bin/env python3
"""
Feature Analysis for Evaluation Function Improvement
Analyzes winning patterns from self-play data
"""

import json
from pathlib import Path
from collections import defaultdict

def load_data():
    """Load results and kifus"""
    # Load results
    results_files = list(Path('selfplay_results').glob('*.json'))
    if not results_files:
        print("Error: No results files found in selfplay_results/")
        return None, None
    
    with open(results_files[0]) as f:
        results = json.load(f)
    
    # Load kifus
    kifus = []
    for kifu_file in sorted(Path('selfplay_kifu').glob('*.json')):
        with open(kifu_file) as f:
            kifus.append(json.load(f))
    
    return results, kifus

def extract_features(kifu):
    """Extract features from a game"""
    moves = kifu.get('moves', [])
    if not moves:
        return {}
    
    features = {}
    
    # Promotion rate
    promotions = sum(1 for m in moves 
                    if isinstance(m, dict) and 'Normal' in m 
                    and m['Normal'].get('promote', False))
    features['promotion_rate'] = promotions / len(moves)
    
    # Drop rate
    drops = sum(1 for m in moves 
               if isinstance(m, dict) and 'Drop' in m)
    features['drop_rate'] = drops / len(moves)
    
    # Game length
    features['game_length'] = len(moves)
    
    # Early game aggression (first 10 moves)
    early_moves = moves[:min(10, len(moves))]
    early_captures = sum(1 for m in early_moves 
                        if isinstance(m, dict) and 'Normal' in m)
    features['early_aggression'] = early_captures / len(early_moves) if early_moves else 0
    
    return features

def analyze_winning_patterns():
    """Analyze patterns in winning games"""
    results, kifus = load_data()
    
    if not results or not kifus:
        return
    
    # Separate by winner
    p1_features = defaultdict(list)
    p2_features = defaultdict(list)
    draw_features = defaultdict(list)
    
    for game_result, kifu in zip(results['games'], kifus):
        winner = game_result.get('winner')
        features = extract_features(kifu)
        
        if not features:
            continue
        
        if winner == 'Player1':
            for key, value in features.items():
                p1_features[key].append(value)
        elif winner == 'Player2':
            for key, value in features.items():
                p2_features[key].append(value)
        else:
            for key, value in features.items():
                draw_features[key].append(value)
    
    # Calculate averages
    def avg(lst):
        return sum(lst) / len(lst) if lst else 0
    
    print("=== Winning Patterns Analysis ===\n")
    print(f"Total Games: {len(results['games'])}")
    print(f"P1 Wins: {len(p1_features['game_length'])}")
    print(f"P2 Wins: {len(p2_features['game_length'])}")
    print(f"Draws: {len(draw_features['game_length'])}\n")
    
    # Compare features
    feature_names = ['promotion_rate', 'drop_rate', 'game_length', 'early_aggression']
    
    for feature in feature_names:
        p1_avg = avg(p1_features[feature])
        p2_avg = avg(p2_features[feature])
        draw_avg = avg(draw_features[feature])
        
        print(f"{feature.replace('_', ' ').title()}:")
        print(f"  P1 Winners: {p1_avg:.3f}")
        print(f"  P2 Winners: {p2_avg:.3f}")
        if draw_features[feature]:
            print(f"  Draws: {draw_avg:.3f}")
        
        # Calculate importance
        diff = abs(p1_avg - p2_avg)
        print(f"  Difference: {diff:.3f}")
        
        if diff > 0.1:
            print(f"  ⚠️  HIGH IMPORTANCE - Consider adjusting evaluation weight")
        print()
    
    # Recommendations
    print("=== Recommendations ===\n")
    
    p1_prom = avg(p1_features['promotion_rate'])
    p2_prom = avg(p2_features['promotion_rate'])
    if abs(p1_prom - p2_prom) > 0.05:
        winner_prom = "P1" if p1_prom > p2_prom else "P2"
        print(f"1. Promotion Strategy: {winner_prom} wins more with higher promotion rate")
        print(f"   → Consider increasing promotion bonus in eval.rs\n")
    
    p1_drop = avg(p1_features['drop_rate'])
    p2_drop = avg(p2_features['drop_rate'])
    if abs(p1_drop - p2_drop) > 0.05:
        winner_drop = "P1" if p1_drop > p2_drop else "P2"
        print(f"2. Drop Usage: {winner_drop} wins more with higher drop rate")
        print(f"   → Consider adjusting hand piece value in eval.rs\n")
    
    p1_len = avg(p1_features['game_length'])
    p2_len = avg(p2_features['game_length'])
    if abs(p1_len - p2_len) > 10:
        winner_len = "P1" if p1_len < p2_len else "P2"
        print(f"3. Game Length: {winner_len} wins in shorter games")
        print(f"   → Consider adjusting aggression parameters\n")

def main():
    analyze_winning_patterns()

if __name__ == "__main__":
    main()
