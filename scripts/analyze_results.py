#!/usr/bin/env python3
"""
Analyze self-play game results and display statistics
Aggregates all results grouped by game type
"""

import json
import argparse
from pathlib import Path
from collections import defaultdict


class Colors:
    """ANSI color codes for terminal output"""
    # Basic colors
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    
    # Styles
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    
    # Reset
    RESET = '\033[0m'
    
    @staticmethod
    def header(text):
        return f"{Colors.BOLD}{Colors.CYAN}{text}{Colors.RESET}"
    
    @staticmethod
    def subheader(text):
        return f"{Colors.BOLD}{Colors.BLUE}{text}{Colors.RESET}"
    
    @staticmethod
    def success(text):
        return f"{Colors.GREEN}{text}{Colors.RESET}"
    
    @staticmethod
    def warning(text):
        return f"{Colors.YELLOW}{text}{Colors.RESET}"
    
    @staticmethod
    def error(text):
        return f"{Colors.RED}{text}{Colors.RESET}"
    
    @staticmethod
    def info(text):
        return f"{Colors.CYAN}{text}{Colors.RESET}"
    
    @staticmethod
    def highlight(text):
        return f"{Colors.BOLD}{Colors.MAGENTA}{text}{Colors.RESET}"

def load_all_results():
    """Load all results files and group by game type"""
    results_dir = Path("selfplay_results")
    if not results_dir.exists():
        print(Colors.error("selfplay_results directory not found"))
        return {}
        
    results_files = list(results_dir.glob("selfplay_results_*.json"))
    
    if not results_files:
        print(Colors.warning("No results files found in selfplay_results/"))
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

def analyze_single_file(filepath):
    """Analyze a single results file"""
    file_path = Path(filepath)
    
    if not file_path.exists():
        print(Colors.error(f"File not found: {filepath}"))
        return None
    
    try:
        with open(file_path) as f:
            data = json.load(f)
    except json.JSONDecodeError:
        print(Colors.error(f"Invalid JSON file: {filepath}"))
        return None
    
    # Display file header
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print(Colors.header(f"SINGLE FILE ANALYSIS: {file_path.name}"))
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print()
    
    # Display configuration
    print(Colors.subheader("Configuration:"))
    print(f"  Board Setup:  {Colors.MAGENTA}{data['board_setup']}{Colors.RESET}")
    print(f"  AI 1 Strength: {Colors.MAGENTA}{data['ai1_strength']}{Colors.RESET}")
    print(f"  AI 2 Strength: {Colors.MAGENTA}{data['ai2_strength']}{Colors.RESET}")
    print()
    
    # Convert to format expected by display_game_type_statistics
    game_type = f"{data['board_setup']} ({data['ai1_strength']} vs {data['ai2_strength']})"
    formatted_data = {
        'total_games': data['total_games'],
        'p1_wins': data['p1_wins'],
        'p2_wins': data['p2_wins'],
        'draws': data['draws'],
        'total_moves': data['avg_moves'] * data['total_games'],
        'total_time_ms': data['avg_time_ms'] * data['total_games'],
        'files': [file_path.name],
        'board_setup': data['board_setup'],
        'ai1_strength': data['ai1_strength'],
        'ai2_strength': data['ai2_strength']
    }
    
    display_game_type_statistics(game_type, formatted_data)
    
    return data

def display_game_type_statistics(game_type, data):
    """Display statistics for a specific game type"""
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print(Colors.header(f"GAME TYPE: {game_type}"))
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print()
    
    total = data['total_games']
    if total == 0:
        print(Colors.warning("No games played"))
        return
    
    # Calculate averages
    avg_moves = data['total_moves'] / total
    avg_time_s = data['total_time_ms'] / total / 1000
    
    # Win rates
    p1_rate = (data['p1_wins'] / total * 100) if total > 0 else 0
    p2_rate = (data['p2_wins'] / total * 100) if total > 0 else 0
    draw_rate = (data['draws'] / total * 100) if total > 0 else 0
    
    print(Colors.info(f"Total Games: {Colors.BOLD}{total}{Colors.RESET}"))
    print(Colors.info(f"Source Files: {len(data['files'])}"))
    print()
    
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    print(Colors.subheader("WIN RATES"))
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    
    # Color code win rates based on percentage
    p1_color = Colors.GREEN if p1_rate > p2_rate else (Colors.YELLOW if p1_rate == p2_rate else Colors.WHITE)
    p2_color = Colors.GREEN if p2_rate > p1_rate else (Colors.YELLOW if p2_rate == p1_rate else Colors.WHITE)
    
    print(f"{p1_color}Player 1: {data['p1_wins']:4d} wins ({p1_rate:5.1f}%){Colors.RESET}")
    print(f"{p2_color}Player 2: {data['p2_wins']:4d} wins ({p2_rate:5.1f}%){Colors.RESET}")
    print(f"{Colors.WHITE}Draws:    {data['draws']:4d}      ({draw_rate:5.1f}%){Colors.RESET}")
    print()
    
    # Balance analysis
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    print(Colors.subheader("BALANCE ANALYSIS"))
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    win_diff = abs(p1_rate - p2_rate)
    if win_diff < 5:
        status = Colors.success("✓ Excellent balance")
        diff_color = Colors.GREEN
    elif win_diff < 10:
        status = Colors.success("✓ Good balance")
        diff_color = Colors.GREEN
    elif win_diff < 20:
        status = Colors.warning("⚠ Slight imbalance")
        diff_color = Colors.YELLOW
    else:
        status = Colors.error("⚠ Significant imbalance")
        diff_color = Colors.RED
    
    print(f"Win rate difference: {diff_color}{win_diff:.1f}%{Colors.RESET}")
    print(f"Status: {status}")
    print()
    
    # Game characteristics
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    print(Colors.subheader("GAME CHARACTERISTICS"))
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    print(f"Average moves:  {Colors.MAGENTA}{avg_moves:.1f}{Colors.RESET}")
    print(f"Average time:   {Colors.MAGENTA}{avg_time_s:.1f}s{Colors.RESET}")
    print()
    
    # Sample size assessment
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    print(Colors.subheader("SAMPLE SIZE"))
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    if total < 10:
        print(Colors.warning("⚠ Very small sample (<10 games)"))
        print(Colors.YELLOW + "  → Run more games for reliable statistics" + Colors.RESET)
    elif total < 50:
        print(Colors.warning("⚠ Small sample (<50 games)"))
        print(Colors.YELLOW + "  → Consider running more games" + Colors.RESET)
    elif total < 100:
        print(Colors.success("✓ Moderate sample size"))
        print(Colors.GREEN + "  → Results are fairly reliable" + Colors.RESET)
    else:
        print(Colors.success("✓ Large sample size"))
        print(Colors.GREEN + "  → Results are statistically significant" + Colors.RESET)
    
    if win_diff > 15 and total >= 10:
        print()
        print(Colors.error("⚠ Win rate imbalance detected"))
        print(Colors.RED + "  → Consider investigating AI behavior" + Colors.RESET)
    
    print()

def display_summary(grouped_data):
    """Display overall summary"""
    if not grouped_data:
        return
    
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print(Colors.header("OVERALL SUMMARY"))
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print()
    
    total_games = sum(data['total_games'] for data in grouped_data.values())
    total_types = len(grouped_data)
    
    print(Colors.highlight(f"Total Game Types: {total_types}"))
    print(Colors.highlight(f"Total Games Played: {total_games}"))
    print()
    
    print(Colors.subheader("Game Types:"))
    for game_type, data in sorted(grouped_data.items()):
        print(f"  {Colors.CYAN}•{Colors.RESET} {game_type}: {Colors.BOLD}{data['total_games']}{Colors.RESET} games")
    print()

def main():
    parser = argparse.ArgumentParser(
        description="Analyze self-play game results",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Analyze all results (aggregated by game type)
  python3 scripts/analyze_results.py
  
  # Analyze a specific results file
  python3 scripts/analyze_results.py -f selfplay_results/selfplay_results_20260105_212034.json
  
  # List all available results files
  python3 scripts/analyze_results.py -l
        """
    )
    
    parser.add_argument(
        '-f', '--file',
        type=str,
        help='Analyze a specific results file'
    )
    
    parser.add_argument(
        '-l', '--list',
        action='store_true',
        help='List all available results files'
    )
    
    args = parser.parse_args()
    
    # List mode
    if args.list:
        results_dir = Path("selfplay_results")
        if not results_dir.exists():
            print(Colors.error("selfplay_results directory not found"))
            return
        
        results_files = sorted(results_dir.glob("selfplay_results_*.json"))
        if not results_files:
            print(Colors.warning("No results files found"))
            return
        
        print(Colors.header("Available Results Files:"))
        print()
        for i, file_path in enumerate(results_files, 1):
            # Load file to show metadata
            try:
                with open(file_path) as f:
                    data = json.load(f)
                print(f"{Colors.CYAN}{i:2d}.{Colors.RESET} {file_path.name}")
                print(f"    {Colors.WHITE}Board: {data['board_setup']}, "
                      f"AI: {data['ai1_strength']} vs {data['ai2_strength']}, "
                      f"Games: {data['total_games']}{Colors.RESET}")
            except:
                print(f"{Colors.CYAN}{i:2d}.{Colors.RESET} {file_path.name}")
        print()
        return
    
    # Single file mode
    if args.file:
        analyze_single_file(args.file)
        return
    
    # Default: aggregate mode
    grouped_data = load_all_results()
    
    if not grouped_data:
        print(Colors.error("No game results found"))
    else:
        # Display summary first
        display_summary(grouped_data)
        
        # Display statistics for each game type
        for game_type in sorted(grouped_data.keys()):
            display_game_type_statistics(game_type, grouped_data[game_type])
        
        # Save to JSON
        output_data = {}
        for game_type, data in grouped_data.items():
            total = data['total_games']
            output_data[game_type] = {
                'total_games': total,
                'p1_wins': data['p1_wins'],
                'p2_wins': data['p2_wins'],
                'draws': data['draws'],
                'p1_win_rate': (data['p1_wins'] / total * 100) if total > 0 else 0,
                'p2_win_rate': (data['p2_wins'] / total * 100) if total > 0 else 0,
                'draw_rate': (data['draws'] / total * 100) if total > 0 else 0,
                'avg_moves': data['total_moves'] / total if total > 0 else 0,
                'avg_time_s': data['total_time_ms'] / total / 1000 if total > 0 else 0,
                'source_files': data['files'],
                'board_setup': data['board_setup'],
                'ai1_strength': data['ai1_strength'],
                'ai2_strength': data['ai2_strength'],
            }
        
        output_file = "scripts/analyze_results.json"
        with open(output_file, 'w') as f:
            json.dump(output_data, f, indent=2)
        
        print(Colors.success(f"\n✓ Results saved to: {output_file}"))

if __name__ == "__main__":
    main()
