#!/usr/bin/env python3
"""
Visualize AI thinking data from kifu files
"""

import json
import matplotlib.pyplot as plt
from pathlib import Path

def plot_thinking_data(kifu_path):
    """Plot AI thinking data from a kifu file"""
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
    
    # Create subplots
    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(12, 10))
    fig.suptitle(f"AI Thinking Analysis: {data['player1_name']} vs {data['player2_name']}")
    
    # Plot 1: Evaluation scores
    ax1.plot(moves, scores, marker='o', linestyle='-', markersize=3)
    ax1.axhline(y=0, color='r', linestyle='--', alpha=0.3)
    ax1.set_ylabel('Score')
    ax1.set_title('Evaluation Score Over Time')
    ax1.grid(True, alpha=0.3)
    
    # Plot 2: Search depth
    ax2.plot(moves, depths, marker='s', linestyle='-', markersize=3, color='green')
    ax2.set_ylabel('Depth')
    ax2.set_title('Search Depth')
    ax2.grid(True, alpha=0.3)
    
    # Plot 3: Nodes evaluated
    ax3.plot(moves, nodes, marker='^', linestyle='-', markersize=3, color='orange')
    ax3.set_ylabel('Nodes')
    ax3.set_xlabel('Move Number')
    ax3.set_title('Nodes Evaluated')
    ax3.grid(True, alpha=0.3)
    
    plt.tight_layout()
    
    # Save plot
    output_path = kifu_path.replace('.json', '_analysis.png')
    plt.savefig(output_path, dpi=150)
    print(f"Saved plot to: {output_path}")
    
    plt.show()

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1:
        kifu_path = sys.argv[1]
    else:
        # Find most recent kifu with thinking data
        kifu_dir = Path("selfplay_kifu")
        kifus = sorted(kifu_dir.glob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)
        
        for kifu_file in kifus:
            with open(kifu_file) as f:
                data = json.load(f)
            if data.get('thinking_data'):
                kifu_path = str(kifu_file)
                break
        else:
            print("No kifu files with thinking data found")
            sys.exit(1)
    
    print(f"Analyzing: {kifu_path}")
    plot_thinking_data(kifu_path)
