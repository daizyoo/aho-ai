#!/usr/bin/env python3
"""
Phase 1: Feature Analysis
Analyzes winning patterns and suggests evaluation function improvements
"""

import json
from pathlib import Path
from collections import defaultdict

def load_selfplay_data():
    """Load all self-play results and kifus"""
    # Load results
    results_dir = Path('selfplay_results')
    if not results_dir.exists():
        print("Error: selfplay_results/ directory not found")
        print("Run self-play first: cargo run --release → option 4")
        return None, None
    
    results_files = list(results_dir.glob('*.json'))
    if not results_files:
        print("Error: No results files found")
        return None, None
    
    # Use most recent results file
    latest_results = max(results_files, key=lambda p: p.stat().st_mtime)
    with open(latest_results) as f:
        results = json.load(f)
    
    # Load kifus
    kifu_dir = Path('selfplay_kifu')
    if not kifu_dir.exists():
        print("Error: selfplay_kifu/ directory not found")
        return None, None
    
    kifus = []
    for kifu_file in sorted(kifu_dir.glob('*.json')):
        with open(kifu_file) as f:
            kifus.append(json.load(f))
    
    print(f"Loaded: {latest_results.name}")
    print(f"Games: {len(kifus)}\n")
    
    return results, kifus

def extract_features(kifu):
    """Extract strategic features from a game"""
    moves = kifu.get('moves', [])
    if not moves:
        return None
    
    features = {}
    
    # Promotion rate
    promotions = sum(1 for m in moves 
                    if isinstance(m, dict) and 'Normal' in m 
                    and m['Normal'].get('promote', False))
    features['promotion_rate'] = promotions / len(moves)
    
    # Drop rate
    drops = sum(1 for m in moves if isinstance(m, dict) and 'Drop' in m)
    features['drop_rate'] = drops / len(moves)
    
    # Game length
    features['game_length'] = len(moves)
    
    # Early aggression (first 10 moves)
    early_moves = moves[:min(10, len(moves))]
    early_normal = sum(1 for m in early_moves 
                      if isinstance(m, dict) and 'Normal' in m)
    features['early_aggression'] = early_normal / len(early_moves) if early_moves else 0
    
    # Piece types dropped
    drop_types = defaultdict(int)
    for m in moves:
        if isinstance(m, dict) and 'Drop' in m:
            piece = m['Drop'].get('kind', '')
            drop_types[piece] += 1
    features['drop_types'] = dict(drop_types)
    
    return features

def analyze_features():
    """Main analysis function"""
    results, kifus = load_selfplay_data()
    if not results or not kifus:
        return
    
    # Separate by winner
    p1_features = defaultdict(list)
    p2_features = defaultdict(list)
    
    for game_result, kifu in zip(results['games'], kifus):
        winner = game_result.get('winner')
        features = extract_features(kifu)
        
        if not features:
            continue
        
        target = p1_features if winner == 'Player1' else p2_features
        for key, value in features.items():
            if key != 'drop_types':
                target[key].append(value)
    
    # Calculate statistics
    def avg(lst):
        return sum(lst) / len(lst) if lst else 0
    
    def std_dev(lst):
        if len(lst) < 2:
            return 0
        mean = avg(lst)
        variance = sum((x - mean) ** 2 for x in lst) / len(lst)
        return variance ** 0.5
    
    print("=" * 60)
    print("FEATURE ANALYSIS RESULTS")
    print("=" * 60)
    print()
    
    print(f"Total Games: {results['total_games']}")
    print(f"P1 Wins: {results['p1_wins']} ({results['p1_wins']/results['total_games']*100:.1f}%)")
    print(f"P2 Wins: {results['p2_wins']} ({results['p2_wins']/results['total_games']*100:.1f}%)")
    print(f"Draws: {results['draws']}")
    print()
    
    # Analyze each feature
    features_to_analyze = ['promotion_rate', 'drop_rate', 'game_length', 'early_aggression']
    recommendations = []
    
    for feature in features_to_analyze:
        p1_avg = avg(p1_features[feature])
        p2_avg = avg(p2_features[feature])
        p1_std = std_dev(p1_features[feature])
        p2_std = std_dev(p2_features[feature])
        
        diff = abs(p1_avg - p2_avg)
        
        print(f"{feature.replace('_', ' ').title()}:")
        print(f"  P1 Winners: {p1_avg:.3f} (±{p1_std:.3f})")
        print(f"  P2 Winners: {p2_avg:.3f} (±{p2_std:.3f})")
        print(f"  Difference: {diff:.3f}")
        
        # Determine importance
        if diff > 0.1 or (feature == 'game_length' and diff > 10):
            print(f"  ⚠️  HIGH IMPORTANCE")
            
            # Generate recommendation
            if feature == 'promotion_rate' and diff > 0.05:
                winner = "P1" if p1_avg > p2_avg else "P2"
                recommendations.append({
                    'feature': 'Promotion Bonus',
                    'current': 'Promotion adds piece value difference',
                    'suggestion': f'Increase promotion bonus (winners promote {max(p1_avg, p2_avg)*100:.1f}% of moves)',
                    'code': 'eval.rs: Increase PRO_* piece values'
                })
            
            elif feature == 'drop_rate' and diff > 0.05:
                winner = "P1" if p1_avg > p2_avg else "P2"
                recommendations.append({
                    'feature': 'Hand Piece Value',
                    'current': 'Hand pieces valued at piece_value * 1.1',
                    'suggestion': f'Increase hand piece bonus (winners drop {max(p1_avg, p2_avg)*100:.1f}% of moves)',
                    'code': 'eval.rs: Change HAND_PIECE_BONUS_MULTIPLIER from 1.1 to 1.2'
                })
        
        print()
    
    # Print recommendations
    if recommendations:
        print("=" * 60)
        print("RECOMMENDED CHANGES TO eval.rs")
        print("=" * 60)
        print()
        
        for i, rec in enumerate(recommendations, 1):
            print(f"{i}. {rec['feature']}")
            print(f"   Current: {rec['current']}")
            print(f"   Suggest: {rec['suggestion']}")
            print(f"   Code: {rec['code']}")
            print()
    else:
        print("=" * 60)
        print("No significant differences found.")
        print("Collect more data (100+ games recommended)")
        print("=" * 60)
        print()
    
    # Save analysis results
    output = {
        'total_games': results['total_games'],
        'p1_wins': results['p1_wins'],
        'p2_wins': results['p2_wins'],
        'features': {
            feature: {
                'p1_avg': avg(p1_features[feature]),
                'p2_avg': avg(p2_features[feature]),
                'difference': abs(avg(p1_features[feature]) - avg(p2_features[feature]))
            }
            for feature in features_to_analyze
        },
        'recommendations': recommendations
    }
    
    output_file = 'scripts/analysis_results.json'
    Path('scripts').mkdir(exist_ok=True)
    with open(output_file, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"Analysis saved to: {output_file}")

if __name__ == "__main__":
    analyze_features()
