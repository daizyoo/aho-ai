#!/usr/bin/env python3
"""
Deep analysis of ShogiOnly vs Fair board results
Provides statistical insights and hypothesis testing
"""

import json
from pathlib import Path
import math


class Colors:
    """ANSI color codes for terminal output"""
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    RESET = '\033[0m'


def calculate_confidence_interval(wins, total, confidence=0.95):
    """Calculate Wilson score confidence interval for win rate"""
    if total == 0:
        return 0, 0
    
    p = wins / total
    z = 1.96 if confidence == 0.95 else 2.576  # 95% or 99%
    
    denominator = 1 + z**2 / total
    center = (p + z**2 / (2 * total)) / denominator
    margin = z * math.sqrt(p * (1 - p) / total + z**2 / (4 * total**2)) / denominator
    
    return max(0, center - margin) * 100, min(1, center + margin) * 100


def chi_square_test(p1_wins, p2_wins, draws):
    """Perform chi-square test for balance"""
    total = p1_wins + p2_wins + draws
    if total == 0:
        return 0, "N/A"
    
    # Expected values (assuming perfect balance)
    expected_wins = (p1_wins + p2_wins) / 2
    
    # Chi-square statistic
    chi_square = ((p1_wins - expected_wins)**2 / expected_wins + 
                  (p2_wins - expected_wins)**2 / expected_wins)
    
    # Degrees of freedom = 1
    # Critical values: 3.841 (p=0.05), 6.635 (p=0.01), 10.828 (p=0.001)
    if chi_square < 3.841:
        significance = "Not significant (p > 0.05)"
    elif chi_square < 6.635:
        significance = "Significant (p < 0.05)"
    elif chi_square < 10.828:
        significance = "Very significant (p < 0.01)"
    else:
        significance = "Extremely significant (p < 0.001)"
    
    return chi_square, significance


def analyze_board_comparison(shogi_data, fair_data):
    """Compare ShogiOnly and Fair boards"""
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}DEEP ANALYSIS: ShogiOnly vs Fair Board Comparison{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}\n")
    
    # Basic stats
    print(f"{Colors.BOLD}{Colors.BLUE}1. SAMPLE SIZE COMPARISON{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    print(f"  ShogiOnly: {Colors.BOLD}{shogi_data['total_games']}{Colors.RESET} games")
    print(f"  Fair:      {Colors.BOLD}{fair_data['total_games']}{Colors.RESET} games")
    print()
    
    # Win rate analysis
    print(f"{Colors.BOLD}{Colors.BLUE}2. WIN RATE ANALYSIS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    for name, data in [("ShogiOnly", shogi_data), ("Fair", fair_data)]:
        p1_rate = data['p1_win_rate']
        p2_rate = data['p2_win_rate']
        draw_rate = data['draw_rate']
        
        # Calculate confidence intervals
        p1_ci_low, p1_ci_high = calculate_confidence_interval(
            data['p1_wins'], data['total_games'])
        p2_ci_low, p2_ci_high = calculate_confidence_interval(
            data['p2_wins'], data['total_games'])
        
        print(f"\n  {Colors.BOLD}{name}:{Colors.RESET}")
        print(f"    Player 1: {p1_rate:5.1f}% (95% CI: {p1_ci_low:.1f}% - {p1_ci_high:.1f}%)")
        print(f"    Player 2: {p2_rate:5.1f}% (95% CI: {p2_ci_low:.1f}% - {p2_ci_high:.1f}%)")
        print(f"    Draws:    {draw_rate:5.1f}%")
        
        # Chi-square test
        chi_sq, sig = chi_square_test(data['p1_wins'], data['p2_wins'], data['draws'])
        print(f"    χ² statistic: {chi_sq:.2f} - {sig}")
    
    print()
    
    # Imbalance comparison
    print(f"{Colors.BOLD}{Colors.BLUE}3. IMBALANCE MAGNITUDE{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    shogi_diff = abs(shogi_data['p1_win_rate'] - shogi_data['p2_win_rate'])
    fair_diff = abs(fair_data['p1_win_rate'] - fair_data['p2_win_rate'])
    
    print(f"  ShogiOnly: {Colors.RED if shogi_diff > 20 else Colors.YELLOW}{shogi_diff:.1f}%{Colors.RESET} difference")
    print(f"  Fair:      {Colors.RED if fair_diff > 20 else Colors.YELLOW}{fair_diff:.1f}%{Colors.RESET} difference")
    print()
    
    # Direction analysis
    print(f"{Colors.BOLD}{Colors.BLUE}4. DIRECTION OF BIAS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    shogi_winner = "Player 1" if shogi_data['p1_win_rate'] > shogi_data['p2_win_rate'] else "Player 2"
    fair_winner = "Player 1" if fair_data['p1_win_rate'] > fair_data['p2_win_rate'] else "Player 2"
    
    print(f"  ShogiOnly favors: {Colors.BOLD}{Colors.GREEN if shogi_winner == 'Player 1' else Colors.RED}{shogi_winner}{Colors.RESET}")
    print(f"  Fair favors:      {Colors.BOLD}{Colors.GREEN if fair_winner == 'Player 1' else Colors.RED}{fair_winner}{Colors.RESET}")
    
    if shogi_winner != fair_winner:
        print(f"\n  {Colors.BOLD}{Colors.YELLOW}⚠ CRITICAL: Boards favor OPPOSITE players!{Colors.RESET}")
        print(f"  {Colors.YELLOW}This suggests board-specific structural imbalance.{Colors.RESET}")
    print()
    
    # Game length analysis
    print(f"{Colors.BOLD}{Colors.BLUE}5. GAME COMPLEXITY{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    shogi_moves = shogi_data['avg_moves']
    fair_moves = fair_data['avg_moves']
    moves_diff_pct = ((shogi_moves - fair_moves) / fair_moves) * 100
    
    print(f"  ShogiOnly avg moves: {Colors.MAGENTA}{shogi_moves:.1f}{Colors.RESET}")
    print(f"  Fair avg moves:      {Colors.MAGENTA}{fair_moves:.1f}{Colors.RESET}")
    print(f"  Difference:          {Colors.BOLD}{moves_diff_pct:+.1f}%{Colors.RESET}")
    
    shogi_time = shogi_data['avg_time_s']
    fair_time = fair_data['avg_time_s']
    time_diff_pct = ((shogi_time - fair_time) / fair_time) * 100
    
    print(f"\n  ShogiOnly avg time:  {Colors.MAGENTA}{shogi_time:.1f}s{Colors.RESET}")
    print(f"  Fair avg time:       {Colors.MAGENTA}{fair_time:.1f}s{Colors.RESET}")
    print(f"  Difference:          {Colors.BOLD}{time_diff_pct:+.1f}%{Colors.RESET}")
    print()
    
    # Hypotheses
    print(f"{Colors.BOLD}{Colors.BLUE}6. POTENTIAL ROOT CAUSES{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    print(f"\n  {Colors.BOLD}Hypothesis 1: Initial Board Asymmetry{Colors.RESET}")
    print(f"    • ShogiOnly and Fair boards have different initial setups")
    print(f"    • Each setup has inherent structural advantages for one player")
    print(f"    • Evidence: {Colors.YELLOW}Opposite bias directions{Colors.RESET}")
    
    print(f"\n  {Colors.BOLD}Hypothesis 2: Evaluation Function Bias{Colors.RESET}")
    print(f"    • AI evaluation may favor certain piece configurations")
    print(f"    • Different boards expose different evaluation biases")
    print(f"    • Evidence: {Colors.YELLOW}Consistent bias within each board type{Colors.RESET}")
    
    print(f"\n  {Colors.BOLD}Hypothesis 3: Search Depth Interaction{Colors.RESET}")
    print(f"    • Longer games (ShogiOnly) may amplify small biases")
    print(f"    • More complex positions → more opportunities for bias")
    print(f"    • Evidence: {Colors.YELLOW}ShogiOnly has {moves_diff_pct:+.1f}% more moves{Colors.RESET}")
    
    print()
    
    # Recommendations
    print(f"{Colors.BOLD}{Colors.BLUE}7. RECOMMENDED INVESTIGATIONS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    print(f"\n  {Colors.GREEN}Priority 1: Initial Position Analysis{Colors.RESET}")
    print(f"    → Examine the exact piece placement in both board setups")
    print(f"    → Look for asymmetries in material, mobility, or king safety")
    
    print(f"\n  {Colors.GREEN}Priority 2: Evaluation Function Audit{Colors.RESET}")
    print(f"    → Test evaluation scores from both player perspectives")
    print(f"    → Check for coordinate-based biases (e.g., file/rank preferences)")
    
    print(f"\n  {Colors.GREEN}Priority 3: Move Distribution Analysis{Colors.RESET}")
    print(f"    → Analyze which pieces move most frequently for each player")
    print(f"    → Check if certain strategies dominate for winning players")
    
    print(f"\n  {Colors.GREEN}Priority 4: Increase Sample Size{Colors.RESET}")
    print(f"    → Run 100+ games for each board type")
    print(f"    → Current ShogiOnly sample (n={shogi_data['total_games']}) is moderate")
    print(f"    → Current Fair sample (n={fair_data['total_games']}) is small")
    
    print()
    
    # Statistical power
    print(f"{Colors.BOLD}{Colors.BLUE}8. STATISTICAL CONFIDENCE{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    # For ShogiOnly
    shogi_total = shogi_data['total_games']
    shogi_decisive = shogi_data['p1_wins'] + shogi_data['p2_wins']
    if shogi_decisive > 0:
        shogi_observed_ratio = max(shogi_data['p1_wins'], shogi_data['p2_wins']) / shogi_decisive
        # Margin of error for 95% CI
        shogi_moe = 1.96 * math.sqrt(0.5 * 0.5 / shogi_decisive) * 100
        
        print(f"\n  {Colors.BOLD}ShogiOnly:{Colors.RESET}")
        print(f"    Observed win ratio: {shogi_observed_ratio:.1%}")
        print(f"    Margin of error (95% CI): ±{shogi_moe:.1f}%")
        print(f"    Confidence: {Colors.GREEN}High{Colors.RESET} (n={shogi_total})")
    
    # For Fair
    fair_total = fair_data['total_games']
    fair_decisive = fair_data['p1_wins'] + fair_data['p2_wins']
    if fair_decisive > 0:
        fair_observed_ratio = max(fair_data['p1_wins'], fair_data['p2_wins']) / fair_decisive
        fair_moe = 1.96 * math.sqrt(0.5 * 0.5 / fair_decisive) * 100
        
        print(f"\n  {Colors.BOLD}Fair:{Colors.RESET}")
        print(f"    Observed win ratio: {fair_observed_ratio:.1%}")
        print(f"    Margin of error (95% CI): ±{fair_moe:.1f}%")
        print(f"    Confidence: {Colors.YELLOW}Moderate{Colors.RESET} (n={fair_total})")
    
    print()
    print(f"{Colors.CYAN}{'=' * 80}{Colors.RESET}\n")


def main():
    # Load analysis results
    results_file = Path("scripts/analyze_results.json")
    
    if not results_file.exists():
        print(f"{Colors.RED}Error: {results_file} not found{Colors.RESET}")
        print(f"{Colors.YELLOW}Run analyze_results.py first to generate the data{Colors.RESET}")
        return
    
    with open(results_file) as f:
        data = json.load(f)
    
    # Extract ShogiOnly and Fair data
    shogi_key = "ShogiOnly (Light vs Light)"
    fair_key = "Fair (Light vs Light)"
    
    if shogi_key not in data or fair_key not in data:
        print(f"{Colors.RED}Error: Missing required game types{Colors.RESET}")
        print(f"Expected: {shogi_key} and {fair_key}")
        print(f"Found: {list(data.keys())}")
        return
    
    shogi_data = data[shogi_key]
    fair_data = data[fair_key]
    
    # Perform deep analysis
    analyze_board_comparison(shogi_data, fair_data)


if __name__ == "__main__":
    main()
