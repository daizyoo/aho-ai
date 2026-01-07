#!/usr/bin/env python3
"""
Enhanced AI Thinking Visualizer

Visualizes AI thinking data from kifu files with:
- Player-specific evaluation tracking
- Critical moments highlighting
- Comprehensive game statistics
- Improved visual presentation
"""

import json
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
from typing import List, Dict, Tuple

def normalize_score(score: int) -> int:
    """Normalize checkmate scores for better visualization"""
    CHECKMATE_THRESHOLD = 100000
    NORMALIZED_CHECKMATE = 5000  # Visual representation of checkmate
    
    if score < -CHECKMATE_THRESHOLD:  # Checkmate for P2
        return -NORMALIZED_CHECKMATE
    elif score > CHECKMATE_THRESHOLD:  # Checkmate for P1
        return NORMALIZED_CHECKMATE
    return score

def is_checkmate(score: int) -> bool:
    """Check if score represents checkmate"""
    return abs(score) > 100000

def detect_critical_moments(scores: List[int], threshold: int = 200) -> List[int]:
    """Detect critical moments where evaluation swings significantly"""
    critical = []
    for i in range(1, len(scores)):
        # Ignore checkmate scores for swing detection
        if is_checkmate(scores[i]) or is_checkmate(scores[i-1]):
            continue
        if abs(scores[i] - scores[i-1]) > threshold:
            critical.append(i)
    return critical

def plot_thinking_data(kifu_path: str):
    """Plot enhanced AI thinking data from a kifu file"""
    with open(kifu_path) as f:
        data = json.load(f)
    
    thinking = data.get('thinking_data')
    if not thinking:
        print(f"No thinking data in {kifu_path}")
        return
    
    # Extract data
    moves = [t['move_number'] for t in thinking]
    scores = [t['score'] for t in thinking]
    depths = [t['depth'] for t in thinking]
    nodes = [t['nodes'] for t in thinking]
    
    # Detect checkmate positions
    checkmate_positions = [(i, moves[i], scores[i]) for i in range(len(scores)) if is_checkmate(scores[i])]
    
    # Normalize scores for visualization
    normalized_scores = [normalize_score(s) for s in scores]
    
    # Separate by player (odd moves = P1, even moves = P2)
    p1_moves = [m for m in moves if m % 2 == 1]
    p2_moves = [m for m in moves if m % 2 == 0]
    p1_scores = [normalized_scores[i] for i in range(len(normalized_scores)) if moves[i] % 2 == 1]
    p2_scores = [-normalized_scores[i] for i in range(len(normalized_scores)) if moves[i] % 2 == 0]  # Negate for P2 perspective
    
    # Detect critical moments (using original scores)
    critical_auto = detect_critical_moments(scores)
    critical_from_data = data.get('critical_moments', [])
    
    # Calculate statistics (using original scores for accuracy)
    avg_depth = np.mean(depths)
    max_depth = max(depths)
    total_nodes = sum(nodes)
    avg_nodes = np.mean(nodes)
    
    # Calculate max swing excluding checkmates
    non_checkmate_scores = [s for s in scores if not is_checkmate(s)]
    if len(non_checkmate_scores) > 1:
        max_score_swing = max(abs(non_checkmate_scores[i] - non_checkmate_scores[i-1]) 
                             for i in range(1, len(non_checkmate_scores)))
    else:
        max_score_swing = 0
    
    # Game result
    if checkmate_positions:
        last_idx, last_move, last_score = checkmate_positions[-1]
        game_result = f"P{'1' if last_score > 0 else '2'} wins (move {last_move})"
    else:
        game_result = "Ongoing/Draw"
    
    # Create figure with custom layout
    fig = plt.figure(figsize=(16, 10))
    gs = fig.add_gridspec(3, 2, height_ratios=[2, 1, 1], width_ratios=[3, 1])
    
    # Main title
    fig.suptitle(f"AI Analysis: {data['player1_name']} vs {data['player2_name']}\n"
                 f"Board: {data.get('board_setup', 'Unknown')} | Moves: {len(moves)}",
                 fontsize=14, fontweight='bold')
    
    # Plot 1: Evaluation scores (split by player)
    ax1 = fig.add_subplot(gs[0, :])
    ax1.plot(p1_moves, p1_scores, marker='o', linestyle='-', markersize=4, 
             color='#2E86DE', label=f'{data["player1_name"]} (P1)', alpha=0.8)
    ax1.plot(p2_moves, p2_scores, marker='s', linestyle='-', markersize=4, 
             color='#EE5A6F', label=f'{data["player2_name"]} (P2)', alpha=0.8)
    
    # Mark checkmate positions
    for idx, move_num, original_score in checkmate_positions:
        marker_color = '#2E86DE' if original_score > 0 else '#EE5A6F'
        marker_symbol = '★' if original_score > 0 else '✖'
        winner_text = 'P1' if original_score > 0 else 'P2'
        
        ax1.plot(move_num, normalized_scores[idx], marker='*', markersize=20, 
                color=marker_color, markeredgecolor='black', markeredgewidth=1.5, zorder=5)
        ax1.annotate(f'{marker_symbol} Checkmate ({winner_text} wins)', 
                    xy=(move_num, normalized_scores[idx]),
                    xytext=(10, 10), textcoords='offset points',
                    fontsize=10, fontweight='bold', color=marker_color,
                    bbox=dict(boxstyle='round,pad=0.5', facecolor='yellow', alpha=0.7),
                    arrowprops=dict(arrowstyle='->', color=marker_color, lw=2))
    
    # Highlight critical moments
    for cm in critical_auto:
        if cm < len(moves):
            ax1.axvline(x=moves[cm], color='orange', linestyle='--', alpha=0.4, linewidth=1.5)
    for cm in critical_from_data:
        if cm < len(moves):
            ax1.axvline(x=moves[cm], color='red', linestyle='--', alpha=0.6, linewidth=2)
    
    ax1.axhline(y=0, color='gray', linestyle='-', alpha=0.3, linewidth=1)
    ax1.fill_between(moves, 0, normalized_scores, where=[s > 0 for s in normalized_scores], 
                      color='#2E86DE', alpha=0.1, label='P1 Advantage')
    ax1.fill_between(moves, 0, normalized_scores, where=[s < 0 for s in normalized_scores], 
                      color='#EE5A6F', alpha=0.1, label='P2 Advantage')
    
    ax1.set_ylabel('Evaluation (centipawns)', fontsize=11, fontweight='bold')
    ax1.set_title('Evaluation Score Over Time (Checkmate scores normalized to ±5000)', 
                  fontsize=12, fontweight='bold')
    ax1.legend(loc='upper right', fontsize=9)
    ax1.grid(True, alpha=0.2)
    
    # Plot 2: Search depth
    ax2 = fig.add_subplot(gs[1, 0])
    ax2.plot(moves, depths, marker='s', linestyle='-', markersize=3, color='#10AC84', linewidth=1.5)
    ax2.axhline(y=avg_depth, color='red', linestyle='--', alpha=0.5, label=f'Avg: {avg_depth:.1f}')
    ax2.set_ylabel('Depth', fontsize=10, fontweight='bold')
    ax2.set_title('Search Depth', fontsize=11, fontweight='bold')
    ax2.legend(loc='upper right', fontsize=8)
    ax2.grid(True, alpha=0.2)
    
    # Plot 3: Nodes evaluated
    ax3 = fig.add_subplot(gs[2, 0])
    ax3.plot(moves, nodes, marker='^', linestyle='-', markersize=3, color='#F79F1F', linewidth=1.5)
    ax3.axhline(y=avg_nodes, color='red', linestyle='--', alpha=0.5, label=f'Avg: {avg_nodes:.0f}')
    ax3.set_ylabel('Nodes', fontsize=10, fontweight='bold')
    ax3.set_xlabel('Move Number', fontsize=10, fontweight='bold')
    ax3.set_title('Nodes Evaluated', fontsize=11, fontweight='bold')
    ax3.legend(loc='upper right', fontsize=8)
    ax3.grid(True, alpha=0.2)
    ax3.ticklabel_format(style='plain', axis='y')
    
    # Plot 4: Statistics panel
    ax_stats = fig.add_subplot(gs[1:, 1])
    ax_stats.axis('off')
    
    stats_text = f"""
    GAME STATISTICS
    ───────────────────
    
    Players:
    • P1: {data['player1_name']}
    • P2: {data['player2_name']}
    
    Board: {data.get('board_setup', 'N/A')}
    Total Moves: {len(moves)}
    Result: {game_result}
    
    Search Stats:
    • Avg Depth: {avg_depth:.1f}
    • Max Depth: {max_depth}
    • Avg Nodes: {avg_nodes:.0f}
    • Total Nodes: {total_nodes:,}
    
    Evaluation:
    • Max Swing: {max_score_swing} CP
    • Critical Moments: {len(critical_from_data)}
      (auto-detected: {len(critical_auto)})
    • Checkmates: {len(checkmate_positions)}
    
    Final Score: {scores[-1] if scores else 'N/A'} CP
    """
    
    ax_stats.text(0.05, 0.95, stats_text, transform=ax_stats.transAxes,
                  fontsize=9, verticalalignment='top', fontfamily='monospace',
                  bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.3))
    
    plt.tight_layout()
    
    # Save plot
    output_dir = Path("analysis_graphs")
    output_dir.mkdir(exist_ok=True)
    
    kifu_filename = Path(kifu_path).stem
    output_path = output_dir / f"{kifu_filename}_enhanced.png"
    plt.savefig(output_path, dpi=200, bbox_inches='tight')
    print(f"✓ Saved enhanced plot to: {output_path}")
    
    plt.show()

def list_and_select_kifu() -> str:
    """List available kifu files and let user select one"""
    kifu_dir = Path("selfplay_kifu")
    if not kifu_dir.exists():
        print(f"✗ Directory {kifu_dir} not found")
        return None
        
    kifus = sorted(kifu_dir.rglob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)
    
    # Filter files with thinking data
    kifus_with_thinking = []
    for kifu_file in kifus:
        try:
            with open(kifu_file) as f:
                data = json.load(f)
            if data.get('thinking_data'):
                kifus_with_thinking.append((kifu_file, data))
        except:
            continue
    
    if not kifus_with_thinking:
        print("✗ No kifu files with thinking data found")
        return None
    
    # Display menu
    print("═" * 70)
    print("  Available Kifu Files with Thinking Data")
    print("═" * 70)
    print()
    
    display_limit = min(20, len(kifus_with_thinking))
    for idx, (kifu_file, data) in enumerate(kifus_with_thinking[:display_limit], 1):
        evaluator = data.get('evaluator', 'Unknown')
        board = data.get('board_setup', 'Unknown')
        
        print(f" {idx:2}. {kifu_file.name}")
        print(f"     Players: {data['player1_name']} vs {data['player2_name']}")
        print(f"     Board: {board} | Moves: {len(data['moves'])} | Evaluator: {evaluator}")
        if idx < display_limit:
            print()
    
    print("─" * 70)
    
    # Get user selection
    try:
        choice = int(input(f"Select file (1-{display_limit}): "))
        if 1 <= choice <= display_limit:
            return str(kifus_with_thinking[choice - 1][0])
        else:
            print("✗ Invalid selection")
            return None
    except (ValueError, KeyboardInterrupt):
        print("\n✗ Cancelled")
        return None

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1:
        kifu_path = sys.argv[1]
    else:
        kifu_path = list_and_select_kifu()
        if not kifu_path:
            sys.exit(1)
    
    print(f"\n⚙ Analyzing: {kifu_path}")
    print("─" * 70)
    plot_thinking_data(kifu_path)
