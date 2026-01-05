#!/usr/bin/env python3
"""
Auto-update ai_config.json based on analysis results
"""

import json
import shutil
from datetime import datetime
from pathlib import Path

def update_ai_config(analysis_results):
    """Update ai_config.json based on analysis recommendations"""
    config_path = "ai_config.json"
    
    if not Path(config_path).exists():
        print(f"Error: {config_path} not found")
        return
    
    # Backup current config
    backup_path = f"ai_config.backup.{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    shutil.copy(config_path, backup_path)
    print(f"\n✓ Backed up config to: {backup_path}")
    
    # Load current config
    with open(config_path, 'r') as f:
        config = json.load(f)
    
    # Load analysis results
    with open('scripts/analysis_results.json', 'r') as f:
        analysis = json.load(f)
    
    # Apply recommendations based on analysis
    changes_made = []
    
    # Check drop rate difference
    drop_diff = analysis['features']['drop_rate']['difference']
    if drop_diff > 0.10:  # Significant drop rate difference
        old_value = config['evaluation']['hand_piece_bonus_multiplier']
        new_value = min(old_value + 0.1, 1.5)  # Increase by 0.1, max 1.5
        config['evaluation']['hand_piece_bonus_multiplier'] = new_value
        changes_made.append(f"hand_piece_bonus_multiplier: {old_value} → {new_value}")
    
    if not changes_made:
        print("\nNo significant changes recommended based on current analysis.")
        return
    
    # Save updated config
    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2)
    
    print("\n" + "="*60)
    print("AI CONFIG UPDATED")
    print("="*60)
    for change in changes_made:
        print(f"✓ {change}")
    print(f"\nConfig saved to: {config_path}")
    print(f"To revert: cp {backup_path} {config_path}")

if __name__ == "__main__":
    # First run analysis
    import subprocess
    print("Running analysis...")
    subprocess.run(["python3", "scripts/phase1_analyze.py"])
    
    # Then update config
    print("\nChecking for config updates...")
    update_ai_config(None)
