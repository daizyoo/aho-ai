#!/usr/bin/env python3
"""
Generate comprehensive analysis report from selfplay data
Works with any available game types
"""

import json
from pathlib import Path


class Colors:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    BOLD = '\033[1m'
    RESET = '\033[0m'


def generate_summary_report():
    """Generate a comprehensive summary of findings"""
    
    # Load data
    results_file = Path("scripts/analyze_results.json")
    if not results_file.exists():
        print(f"{Colors.RED}Error: analyze_results.json not found{Colors.RESET}")
        print(f"{Colors.YELLOW}Run 'python scripts/analyze_results.py' first{Colors.RESET}")
        return
    
    with open(results_file) as f:
        data = json.load(f)
    
    if len(data) == 0:
        print(f"{Colors.RED}Error: No game types found{Colors.RESET}")
        return
    
    # Get total stats
    total_games = sum(d['total_games'] for d in data.values())
    game_types = list(data.keys())
    
    # Header
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}COMPREHENSIVE ANALYSIS REPORT{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 80}{Colors.RESET}\n")
    
    # Executive Summary
    print(f"{Colors.BOLD}{Colors.MAGENTA}📊 EXECUTIVE SUMMARY{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}\n")
    
    print(f"Total Games Analyzed: {Colors.BOLD}{total_games}{Colors.RESET}")
    print(f"Game Types: {Colors.BOLD}{len(game_types)}{Colors.RESET}")
    print()
    for gt in game_types:
        print(f"  • {gt}: {data[gt]['total_games']} games")
    print()
    
    # Key Findings
    print(f"{Colors.BOLD}{Colors.MAGENTA}🔍 KEY FINDINGS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}\n")
    
    for i, gt in enumerate(game_types, 1):
        gt_data = data[gt]
        p1_rate = gt_data['p1_win_rate']
        p2_rate = gt_data['p2_win_rate']
        draw_rate = gt_data['draw_rate']
        diff = abs(p1_rate - p2_rate)
        
        balance_status = (
            f"{Colors.GREEN}Well-balanced{Colors.RESET}" if diff < 10 else
            f"{Colors.YELLOW}Moderate imbalance{Colors.RESET}" if diff < 20 else
            f"{Colors.RED}Significant imbalance{Colors.RESET}"
        )
        
        winner = "Player 1" if p1_rate > p2_rate else "Player 2" if p2_rate > p1_rate else "Balanced"
        
        print(f"{Colors.BOLD}{i}. {gt}{Colors.RESET}")
        print(f"   Games: {gt_data['total_games']} | "
              f"P1: {p1_rate:.1f}% | P2: {p2_rate:.1f}% | Draws: {draw_rate:.1f}%")
        print(f"   Win difference: {Colors.BOLD}{diff:.1f}%{Colors.RESET}")
        print(f"   Favors: {Colors.BOLD}{winner}{Colors.RESET}")
        print(f"   Balance status: {balance_status}")
        print(f"   Avg moves: {gt_data['avg_moves']:.1f} | Avg time: {gt_data['avg_time_s']:.1f}s")
        print()
    
    # Balance Analysis
    print(f"{Colors.BOLD}{Colors.MAGENTA}⚖️  BALANCE ANALYSIS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}\n")
    
    # Check if different game types favor different players
    p1_favored = []
    p2_favored = []
    balanced = []
    
    for gt in game_types:
        gt_data = data[gt]
        if gt_data['p1_win_rate'] > gt_data['p2_win_rate'] + 5:
            p1_favored.append(gt)
        elif gt_data['p2_win_rate'] > gt_data['p1_win_rate'] + 5:
            p2_favored.append(gt)
        else:
            balanced.append(gt)
    
    if p1_favored:
        print(f"{Colors.GREEN}Game types favoring Player 1:{Colors.RESET}")
        for gt in p1_favored:
            print(f"  • {gt} ({data[gt]['p1_win_rate']:.1f}% vs {data[gt]['p2_win_rate']:.1f}%)")
        print()
    
    if p2_favored:
        print(f"{Colors.RED}Game types favoring Player 2:{Colors.RESET}")
        for gt in p2_favored:
            print(f"  • {gt} ({data[gt]['p2_win_rate']:.1f}% vs {data[gt]['p1_win_rate']:.1f}%)")
        print()
    
    if balanced:
        print(f"{Colors.GREEN}Well-balanced game types:{Colors.RESET}")
        for gt in balanced:
            print(f"  • {gt} ({data[gt]['p1_win_rate']:.1f}% vs {data[gt]['p2_win_rate']:.1f}%)")
        print()
    
    if p1_favored and p2_favored:
        print(f"{Colors.BOLD}{Colors.YELLOW}⚠ Different game types favor opposite players!{Colors.RESET}")
        print(f"{Colors.YELLOW}This suggests board-specific structural differences.{Colors.RESET}\n")
    
    # Game Complexity
    print(f"{Colors.BOLD}{Colors.MAGENTA}📈 GAME COMPLEXITY{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}\n")
    
    game_types_sorted = sorted(game_types, key=lambda gt: data[gt]['avg_moves'], reverse=True)
    
    print(f"Game types ranked by average move count:\n")
    for i, gt in enumerate(game_types_sorted, 1):
        gt_data = data[gt]
        print(f"  {i}. {gt}: {gt_data['avg_moves']:.1f} moves "
              f"({gt_data['avg_time_s']:.1f}s)")
    print()
    
    # Recommendations
    print(f"{Colors.BOLD}{Colors.MAGENTA}🚀 RECOMMENDATIONS{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}\n")
    
    # Check sample sizes
    small_samples = [gt for gt in game_types if data[gt]['total_games'] < 20]
    if small_samples:
        print(f"{Colors.BOLD}Priority 1: Increase Sample Size{Colors.RESET}")
        print(f"  The following game types have limited data (\u003c20 games):")
        for gt in small_samples:
            print(f"    • {gt}: {data[gt]['total_games']} games")
        print(f"  {Colors.YELLOW}→ Run more games for reliable statistics{Colors.RESET}\n")
    
    # Check for imbalances
    imbalanced = [gt for gt in game_types if abs(data[gt]['p1_win_rate'] - data[gt]['p2_win_rate']) > 20]
    if imbalanced:
        print(f"{Colors.BOLD}Priority 2: Investigate Imbalances{Colors.RESET}")
        print(f"  Significant win rate differences detected in:")
        for gt in imbalanced:
            diff = abs(data[gt]['p1_win_rate'] - data[gt]['p2_win_rate'])
            print(f"    • {gt}: {diff:.1f}% difference")
        print(f"  {Colors.YELLOW}→ Run deep_analysis.py to identify root causes{Colors.RESET}\n")
    
    print(f"{Colors.BOLD}Priority 3: Visualization{Colors.RESET}")
    print(f"  {Colors.GREEN}→ Use visualize_thinking.py to analyze game trajectories{Colors.RESET}")
    print(f"  {Colors.GREEN}→ Identify critical moments and tactical patterns{Colors.RESET}\n")
    
    # Conclusion
    print(f"{Colors.BOLD}{Colors.MAGENTA}📝 CONCLUSION{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 80}{Colors.RESET}\n")
    
    if len(imbalanced) > 0:
        print(f"Analysis of {total_games} games reveals {Colors.RED}balance issues{Colors.RESET}")
        print(f"in {len(imbalanced)} game type(s). Further investigation recommended.")
    elif len(balanced) == len(game_types):
        print(f"Analysis of {total_games} games shows {Colors.GREEN}good overall balance{Colors.RESET}")
        print(f"across all {len(game_types)} game type(s).")
    else:
        print(f"Analysis of {total_games} games shows {Colors.YELLOW}mixed results{Colors.RESET}.")
        print(f"Some game types are balanced, others show bias.")
    
    print()
    print(f"{Colors.CYAN}{'=' * 80}{Colors.RESET}\n")


if __name__ == "__main__":
    generate_summary_report()
