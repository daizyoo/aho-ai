#!/usr/bin/env python3
"""
Deep analysis of board results with flexible game type selection
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
    
    if expected_wins == 0:
        return 0, "N/A (no decisive games)"
    
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


def analyze_board_comparison(name1, data1, name2, data2):
    """Compare two game type results"""
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}DEEP ANALYSIS: {name1} vs {name2}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}\n")
    
    # Basic stats
    print(f"{Colors.BOLD}{Colors.BLUE}1. SAMPLE SIZE COMPARISON{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    print(f"  {name1}: {Colors.BOLD}{data1['total_games']}{Colors.RESET} games")
    print(f"  {name2}: {Colors.BOLD}{data2['total_games']}{Colors.RESET} games")
    print()
    
    # Win rate analysis
    print(f"{Colors.BOLD}{Colors.BLUE}2. WIN RATE ANALYSIS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    for name, data in [(name1, data1), (name2, data2)]:
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
    
    diff1 = abs(data1['p1_win_rate'] - data1['p2_win_rate'])
    diff2 = abs(data2['p1_win_rate'] - data2['p2_win_rate'])
    
    print(f"  {name1}: {Colors.RED if diff1 > 20 else Colors.YELLOW}{diff1:.1f}%{Colors.RESET} difference")
    print(f"  {name2}: {Colors.RED if diff2 > 20 else Colors.YELLOW}{diff2:.1f}%{Colors.RESET} difference")
    print()
    
    # Direction analysis
    print(f"{Colors.BOLD}{Colors.BLUE}4. DIRECTION OF BIAS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    winner1 = "Player 1" if data1['p1_win_rate'] > data1['p2_win_rate'] else "Player 2"
    winner2 = "Player 1" if data2['p1_win_rate'] > data2['p2_win_rate'] else "Player 2"
    
    print(f"  {name1} favors: {Colors.BOLD}{Colors.GREEN if winner1 == 'Player 1' else Colors.RED}{winner1}{Colors.RESET}")
    print(f"  {name2} favors: {Colors.BOLD}{Colors.GREEN if winner2 == 'Player 1' else Colors.RED}{winner2}{Colors.RESET}")
    
    if winner1 != winner2:
        print(f"\n  {Colors.BOLD}{Colors.YELLOW}⚠ CRITICAL: Game types favor OPPOSITE players!{Colors.RESET}")
        print(f"  {Colors.YELLOW}This suggests setting-specific structural imbalance.{Colors.RESET}")
    else:
        print(f"\n  {Colors.BOLD}{Colors.GREEN}✓ Both favor the same player{Colors.RESET}")
        print(f"  {Colors.GREEN}This suggests consistent evaluation or search bias.{Colors.RESET}")
    print()
    
    # Game length analysis
    print(f"{Colors.BOLD}{Colors.BLUE}5. GAME COMPLEXITY{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    moves1 = data1['avg_moves']
    moves2 = data2['avg_moves']
    moves_diff_pct = ((moves1 - moves2) / moves2) * 100 if moves2 > 0 else 0
    
    print(f"  {name1} avg moves: {Colors.MAGENTA}{moves1:.1f}{Colors.RESET}")
    print(f"  {name2} avg moves: {Colors.MAGENTA}{moves2:.1f}{Colors.RESET}")
    print(f"  Difference:       {Colors.BOLD}{moves_diff_pct:+.1f}%{Colors.RESET}")
    
    time1 = data1['avg_time_s']
    time2 = data2['avg_time_s']
    time_diff_pct = ((time1 - time2) / time2) * 100 if time2 > 0 else 0
    
    print(f"\n  {name1} avg time:  {Colors.MAGENTA}{time1:.1f}s{Colors.RESET}")
    print(f"  {name2} avg time:  {Colors.MAGENTA}{time2:.1f}s{Colors.RESET}")
    print(f"  Difference:        {Colors.BOLD}{time_diff_pct:+.1f}%{Colors.RESET}")
    print()
    
    # Hypotheses
    print(f"{Colors.BOLD}{Colors.BLUE}6. POTENTIAL ROOT CAUSES{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    print(f"\n  {Colors.BOLD}Hypothesis 1: Initial Position Asymmetry{Colors.RESET}")
    print(f"    • Different board setups have inherent advantages")
    print(f"    • Each setup favors different piece configurations")
    print(f"    • Evidence: {Colors.YELLOW}Opposite bias directions{Colors.RESET}" if winner1 != winner2 else f"    • Evidence: {Colors.GREEN}Consistent bias{Colors.RESET}")
    
    print(f"\n  {Colors.BOLD}Hypothesis 2: Evaluation Function Bias{Colors.RESET}")
    print(f"    • AI evaluation may favor certain patterns")
    print(f"    • Different boards expose different biases")
    print(f"    • Evidence: {Colors.YELLOW}Consistent bias within each type{Colors.RESET}")
    
    print(f"\n  {Colors.BOLD}Hypothesis 3: Search Depth Interaction{Colors.RESET}")
    print(f"    • Longer games may amplify small biases")
    print(f"    • More complex positions → more bias opportunities")
    print(f"    • Evidence: {Colors.YELLOW}{abs(moves_diff_pct):.1f}% move difference{Colors.RESET}")
    
    print()
    
    # Statistical power
    print(f"{Colors.BOLD}{Colors.BLUE}7. STATISTICAL CONFIDENCE{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}")
    
    for name, data in [(name1, data1), (name2, data2)]:
        total = data['total_games']
        decisive = data['p1_wins'] + data['p2_wins']
        
        if decisive > 0:
            observed_ratio = max(data['p1_wins'], data['p2_wins']) / decisive
            moe = 1.96 * math.sqrt(0.5 * 0.5 / decisive) * 100
            
            confidence_level = "High" if total >= 50 else "Moderate" if total >= 20 else "Low"
            confidence_color = Colors.GREEN if total >= 50 else Colors.YELLOW if total >= 20 else Colors.RED
            
            print(f"\n  {Colors.BOLD}{name}:{Colors.RESET}")
            print(f"    Observed win ratio: {observed_ratio:.1%}")
            print(f"    Margin of error (95% CI): ±{moe:.1f}%")
            print(f"    Confidence: {confidence_color}{confidence_level}{Colors.RESET} (n={total})")
    
    print()
    print(f"{Colors.CYAN}{'=' * 80}{Colors.RESET}\n")


def main():
    # Load analysis results
    results_file = Path("scripts/analyze_results.json")
    
    if not results_file.exists():
        print(f"{Colors.RED}Error: {results_file} not found{Colors.RESET}")
        print(f"{Colors.YELLOW}Run 'python scripts/analyze_results.py' first to generate the data{Colors.RESET}")
        return
    
    with open(results_file) as f:
        data = json.load(f)
    
    if len(data) == 0:
        print(f"{Colors.RED}Error: No game types found in analyze_results.json{Colors.RESET}")
        return
    
    if len(data) < 2:
        print(f"{Colors.YELLOW}Warning: Only one game type found{Colors.RESET}")
        print(f"Found: {list(data.keys())[0]}")
        print(f"\n{Colors.CYAN}Need at least 2 game types for comparison.{Colors.RESET}")
        print(f"{Colors.CYAN}Run more selfplay games with different settings.{Colors.RESET}")
        return
    
    # Display available game types
    game_types = list(data.keys())
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}Available Game Types for Comparison{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}\n")
    
    for i, gt in enumerate(game_types, 1):
        gt_data = data[gt]
        print(f"  {Colors.BOLD}{i}.{Colors.RESET} {gt}")
        print(f"      Games: {gt_data['total_games']} | "
              f"P1: {gt_data['p1_win_rate']:.1f}% | "
              f"P2: {gt_data['p2_win_rate']:.1f}% | "
              f"Draws: {gt_data['draw_rate']:.1f}%")
        print()
    
    # Auto-select first two, or let user choose
    if len(game_types) == 2:
        idx1, idx2 = 0, 1
        print(f"{Colors.GREEN}Auto-selecting both game types for comparison.{Colors.RESET}\n")
    else:
        print(f"{Colors.YELLOW}Enter two numbers to compare (e.g., '1 2'), or press Enter for first two:{Colors.RESET}")
        try:
            user_input = input("> ").strip()
            if user_input:
                choices = [int(x) - 1 for x in user_input.split()]
                if len(choices) != 2 or not all(0 <= c < len(game_types) for c in choices):
                    print(f"{Colors.RED}Invalid selection. Using first two.{Colors.RESET}")
                    idx1, idx2 = 0, 1
                else:
                    idx1, idx2 = choices
            else:
                idx1, idx2 = 0, 1
        except (ValueError, KeyboardInterrupt):
            print(f"{Colors.RED}Invalid input. Using first two.{Colors.RESET}")
            idx1, idx2 = 0, 1
        print()
    
    # Perform deep analysis
    name1 = game_types[idx1]
    name2 = game_types[idx2]
    analyze_board_comparison(name1, data[name1], name2, data[name2])


if __name__ == "__main__":
    main()
