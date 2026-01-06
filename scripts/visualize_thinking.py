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
    
    
    # Save plot to analysis_graphs directory
    output_dir = Path("analysis_graphs")
    output_dir.mkdir(exist_ok=True)
    
    kifu_filename = Path(kifu_path).stem
    output_path = output_dir / f"{kifu_filename}_analysis.png"
    plt.savefig(output_path, dpi=150)
    print(f"Saved plot to: {output_path}")
    
    plt.show()

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1:
        kifu_path = sys.argv[1]
    else:
        # Find kifu files with thinking data
        kifu_dir = Path("selfplay_kifu")
        if not kifu_dir.exists():
            print(f"Directory {kifu_dir} not found")
            sys.exit(1)
            
        kifus = sorted(kifu_dir.rglob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)
        
        # Filter files with thinking data
        kifus_with_thinking = []
        for kifu_file in kifus:
            try:
                with open(kifu_file) as f:
                    data = json.load(f)
                if data.get('thinking_data'):
                    kifus_with_thinking.append(kifu_file)
            except:
                continue
        
        if not kifus_with_thinking:
            print("No kifu files with thinking data found")
            sys.exit(1)
        
        # Display menu
        print("Available kifu files with thinking data:\n")
        for idx, kifu_file in enumerate(kifus_with_thinking[:20], 1):  # Show max 20
            with open(kifu_file) as f:
                data = json.load(f)
            print(f"{idx:2}. {kifu_file.name}")
            print(f"    {data['player1_name']} vs {data['player2_name']}")
            print(f"    Moves: {len(data['moves'])}, Thinking data: {len(data.get('thinking_data', []))}")
            print()
        
        # Get user selection
        try:
            choice = int(input(f"Select file (1-{len(kifus_with_thinking[:20])}): "))
            if 1 <= choice <= len(kifus_with_thinking[:20]):
                kifu_path = str(kifus_with_thinking[choice - 1])
            else:
                print("Invalid selection")
                sys.exit(1)
        except (ValueError, KeyboardInterrupt):
            print("\nCancelled")
            sys.exit(1)
    
    print(f"\nAnalyzing: {kifu_path}")
    plot_thinking_data(kifu_path)
