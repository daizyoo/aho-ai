#!/usr/bin/env python3
"""
Bump version in Cargo.toml following semantic versioning
"""

import argparse
import re
import subprocess
from pathlib import Path


class Colors:
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    BOLD = '\033[1m'
    RESET = '\033[0m'


def parse_version(version_str):
    """Parse version string into (major, minor, patch)"""
    match = re.match(r'(\d+)\.(\d+)\.(\d+)', version_str)
    if not match:
        raise ValueError(f"Invalid version format: {version_str}")
    return tuple(map(int, match.groups()))


def bump_version(current_version, bump_type):
    """Bump version based on type (major, minor, patch)"""
    major, minor, patch = parse_version(current_version)
    
    if bump_type == 'major':
        return f"{major + 1}.0.0"
    elif bump_type == 'minor':
        return f"{major}.{minor + 1}.0"
    elif bump_type == 'patch':
        return f"{major}.{minor}.{patch + 1}"
    else:
        raise ValueError(f"Invalid bump type: {bump_type}")


def get_current_version(cargo_toml_path):
    """Extract current version from Cargo.toml"""
    with open(cargo_toml_path, 'r') as f:
        content = f.read()
    
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if not match:
        raise ValueError("Could not find version in Cargo.toml")
    
    return match.group(1)


def update_cargo_toml(cargo_toml_path, new_version):
    """Update version in Cargo.toml"""
    with open(cargo_toml_path, 'r') as f:
        content = f.read()
    
    updated_content = re.sub(
        r'^version\s*=\s*"[^"]+"',
        f'version = "{new_version}"',
        content,
        count=1,
        flags=re.MULTILINE
    )
    
    with open(cargo_toml_path, 'w') as f:
        f.write(updated_content)


def create_git_commit(new_version, message=None):
    """Create git commit with version bump"""
    try:
        subprocess.run(['git', 'add', 'Cargo.toml'], check=True)
        
        commit_msg = message or f"chore: Bump version to {new_version}"
        subprocess.run(['git', 'commit', '-m', commit_msg], check=True)
        
        return True
    except subprocess.CalledProcessError as e:
        print(f"{Colors.YELLOW}Warning: Failed to create git commit: {e}{Colors.RESET}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description='Bump version in Cargo.toml',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Bump patch version (0.3.1 → 0.3.2)
  python3 scripts/bump_version.py patch
  
  # Bump minor version (0.3.1 → 0.4.0)
  python3 scripts/bump_version.py minor
  
  # Bump major version (0.3.1 → 1.0.0)
  python3 scripts/bump_version.py major
  
  # Dry run (show what would change)
  python3 scripts/bump_version.py patch --dry-run
  
  # Create git commit with custom message
  python3 scripts/bump_version.py patch --commit -m "fix: Critical bug fix"
        """
    )
    
    parser.add_argument(
        'bump_type',
        choices=['major', 'minor', 'patch'],
        help='Type of version bump'
    )
    
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Show what would change without making changes'
    )
    
    parser.add_argument(
        '--commit',
        action='store_true',
        help='Create git commit after bumping version'
    )
    
    parser.add_argument(
        '-m', '--message',
        type=str,
        help='Custom commit message (requires --commit)'
    )
    
    args = parser.parse_args()
    
    # Find Cargo.toml
    cargo_toml = Path('Cargo.toml')
    if not cargo_toml.exists():
        print(f"{Colors.YELLOW}Error: Cargo.toml not found{Colors.RESET}")
        print("Run this script from the project root directory")
        return 1
    
    # Get current version
    try:
        current_version = get_current_version(cargo_toml)
    except ValueError as e:
        print(f"{Colors.YELLOW}Error: {e}{Colors.RESET}")
        return 1
    
    # Calculate new version
    try:
        new_version = bump_version(current_version, args.bump_type)
    except ValueError as e:
        print(f"{Colors.YELLOW}Error: {e}{Colors.RESET}")
        return 1
    
    # Display changes
    print(f"{Colors.CYAN}{'=' * 60}{Colors.RESET}")
    print(f"{Colors.BOLD}Version Bump: {args.bump_type.upper()}{Colors.RESET}")
    print(f"{Colors.CYAN}{'=' * 60}{Colors.RESET}\n")
    
    print(f"  Current version: {Colors.YELLOW}{current_version}{Colors.RESET}")
    print(f"  New version:     {Colors.GREEN}{new_version}{Colors.RESET}")
    print()
    
    if args.dry_run:
        print(f"{Colors.BLUE}[DRY RUN] No changes made{Colors.RESET}")
        return 0
    
    # Update Cargo.toml
    try:
        update_cargo_toml(cargo_toml, new_version)
        print(f"{Colors.GREEN}✓ Updated Cargo.toml{Colors.RESET}")
    except Exception as e:
        print(f"{Colors.YELLOW}Error updating Cargo.toml: {e}{Colors.RESET}")
        return 1
    
    # Create git commit if requested
    if args.commit:
        if create_git_commit(new_version, args.message):
            print(f"{Colors.GREEN}✓ Created git commit{Colors.RESET}")
        else:
            print(f"{Colors.YELLOW}⚠ Cargo.toml updated but commit failed{Colors.RESET}")
            print(f"  You can commit manually with:")
            print(f"  git add Cargo.toml")
            print(f"  git commit -m 'chore: Bump version to {new_version}'")
    else:
        print()
        print(f"{Colors.BLUE}Next steps:{Colors.RESET}")
        print(f"  git add Cargo.toml")
        print(f"  git commit -m 'chore: Bump version to {new_version}'")
    
    print()
    return 0


if __name__ == '__main__':
    exit(main())
