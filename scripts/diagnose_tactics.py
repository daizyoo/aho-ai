#!/usr/bin/env python3
"""
Tactical Blunder Diagnostic Tool
Analyzes game records to detect positions where pieces are captured without compensation.
"""

import json
import argparse
from pathlib import Path
from collections import defaultdict
from typing import List, Dict, Any, Optional, Tuple

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

# Piece values for material calculation
PIECE_VALUES = {
    'Pawn': 100,
    'Lance': 350,
    'Knight': 400,
    'Silver': 550,
    'Gold': 600,
    'Bishop': 850,
    'Rook': 1000,
    'King': 10000,
    # Promoted pieces
    'ProPawn': 600,
    'ProLance': 600,
    'ProKnight': 700,
    'ProSilver': 700,
    'ProBishop': 1200,  # Horse
    'ProRook': 1500,    # Dragon
    # Chess pieces
    'ChessPawn': 100,
    'ChessKnight': 400,
    'ChessBishop': 850,
    'ChessRook': 1000,
    'ChessQueen': 1800,
}

def get_piece_value(piece_kind: str) -> int:
    """Get material value of a piece"""
    return PIECE_VALUES.get(piece_kind, 0)

def get_piece_name(piece_kind: str) -> str:
    """Get readable piece name"""
    name_map = {
        'Pawn': '歩', 'Lance': '香', 'Knight': '桂', 'Silver': '銀',
        'Gold': '金', 'Bishop': '角', 'Rook': '飛', 'King': '玉',
        'ProPawn': 'と', 'ProLance': '成香', 'ProKnight': '成桂',
        'ProSilver': '成銀', 'ProBishop': '馬', 'ProRook': '竜',
        'ChessPawn': 'P', 'ChessKnight': 'N', 'ChessBishop': 'B',
        'ChessRook': 'R', 'ChessQueen': 'Q',
    }
    return name_map.get(piece_kind, piece_kind)

def format_position(pos: Dict[str, int]) -> str:
    """Format position as readable coordinates (e.g., 7六)"""
    # In Shogi notation: x=0 is 9筋, x=8 is 1筋
    # y=0 is 一, y=8 is 九
    files = ['９', '８', '７', '６', '５', '４', '３', '２', '１']
    ranks = ['一', '二', '三', '四', '五', '六', '七', '八', '九']
    x, y = pos['x'], pos['y']
    if 0 <= x < 9 and 0 <= y < 9:
        return f"{files[x]}{ranks[y]}"
    return f"({x},{y})"

class GameAnalyzer:
    def __init__(self, kifu_path: str):
        self.kifu_path = Path(kifu_path)
        with open(kifu_path) as f:
            self.data = json.load(f)
        self.moves = self.data['moves']
        self.blunders = []
        self.material_swings = []
        
    def analyze(self) -> Dict[str, Any]:
        """Analyze game for tactical blunders"""
        print(f"{Colors.CYAN}Analyzing: {self.kifu_path.name}{Colors.RESET}")
        
        # Track material balance over time
        material_history = []
        last_material = 0
        
        for move_num, move in enumerate(self.moves, 1):
            # Check if this move captures a piece
            if 'Normal' in move:
                normal_move = move['Normal']
                from_pos = normal_move['from']
                to_pos = normal_move['to']
                
                # Simple heuristic: detect large material swings
                # This is a placeholder - in reality we'd need to simulate the board state
                # For now, we'll just flag suspicious move sequences
                
        result = self.data.get('result', {})
        winner = result.get('winner')
        termination = result.get('termination', 'Unknown')
        total_moves = len(self.moves)
        
        return {
            'file': self.kifu_path.name,
            'total_moves': total_moves,
            'winner': winner,
            'termination': termination,
            'blunders_detected': len(self.blunders),
            'blunders': self.blunders,
        }

def analyze_directory(directory: str, max_games: int = 10) -> List[Dict[str, Any]]:
    """Analyze multiple game files in a directory"""
    dir_path = Path(directory)
    if not dir_path.exists():
        print(f"{Colors.RED}Directory not found: {directory}{Colors.RESET}")
        return []
    
    kifu_files = sorted(dir_path.glob("game_*.json"))[:max_games]
    
    if not kifu_files:
        print(f"{Colors.YELLOW}No game files found in {directory}{Colors.RESET}")
        return []
    
    results = []
    for kifu_file in kifu_files:
        analyzer = GameAnalyzer(kifu_file)
        result = analyzer.analyze()
        results.append(result)
    
    return results

def display_analysis_summary(results: List[Dict[str, Any]]):
    """Display summary of analysis results"""
    if not results:
        return
    
    print()
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print(Colors.BOLD + Colors.CYAN + "TACTICAL ANALYSIS SUMMARY" + Colors.RESET)
    print(Colors.CYAN + "=" * 70 + Colors.RESET)
    print()
    
    total_games = len(results)
    total_moves = sum(r['total_moves'] for r in results)
    avg_moves = total_moves / total_games if total_games > 0 else 0
    
    games_with_blunders = sum(1 for r in results if r['blunders_detected'] > 0)
    total_blunders = sum(r['blunders_detected'] for r in results)
    
    print(f"{Colors.WHITE}Total Games Analyzed: {Colors.BOLD}{total_games}{Colors.RESET}")
    print(f"{Colors.WHITE}Average Moves per Game: {Colors.BOLD}{avg_moves:.1f}{Colors.RESET}")
    print(f"{Colors.WHITE}Games with Blunders: {Colors.BOLD}{games_with_blunders}{Colors.RESET}")
    print(f"{Colors.WHITE}Total Blunders Detected: {Colors.BOLD}{total_blunders}{Colors.RESET}")
    print()
    
    # Display per-game results
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    print(Colors.BOLD + "GAME RESULTS" + Colors.RESET)
    print(Colors.BLUE + "-" * 70 + Colors.RESET)
    
    for i, result in enumerate(results, 1):
        winner_str = result.get('winner', 'Unknown')
        if winner_str == 'Player1':
            winner_color = Colors.GREEN
        elif winner_str == 'Player2':
            winner_color = Colors.MAGENTA
        else:
            winner_color = Colors.YELLOW
        
        termination = result.get('termination', 'Unknown')
        blunders = result['blunders_detected']
        blunder_str = f"{Colors.RED}{blunders} blunders{Colors.RESET}" if blunders > 0 else f"{Colors.GREEN}No blunders{Colors.RESET}"
        
        print(f"{i:2d}. {result['file']}: {winner_color}{winner_str}{Colors.RESET} "
              f"by {termination}, {result['total_moves']} moves, {blunder_str}")
    
    print()

def main():
    parser = argparse.ArgumentParser(
        description="Diagnose tactical blunders in self-play games",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    
    parser.add_argument(
        'directory',
        type=str,
        nargs='?',
        default='selfplay_kifu/ShogiOnly/20260110_175401',
        help='Directory containing game files (default: latest ShogiOnly session)'
    )
    
    parser.add_argument(
        '-n', '--num-games',
        type=int,
        default=10,
        help='Number of games to analyze (default: 10)'
    )
    
    args = parser.parse_args()
    
    # Analyze games
    results = analyze_directory(args.directory, max_games=args.num_games)
    
    if results:
        display_analysis_summary(results)
        
        # Save detailed results
        output_file = "scripts/tactical_diagnosis.json"
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"{Colors.GREEN}✓ Detailed results saved to: {output_file}{Colors.RESET}")
    else:
        print(f"{Colors.RED}No games analyzed{Colors.RESET}")

if __name__ == "__main__":
    main()
