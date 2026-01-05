#!/usr/bin/env python3
"""
Visualize and analyze board setups for structural asymmetries
"""


class Colors:
    """ANSI color codes"""
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    BOLD = '\033[1m'
    RESET = '\033[0m'


def get_shogi_setup():
    """ShogiOnly board setup"""
    return [
        "l n s g k g s n l",
        ". r . . . . . b .",
        "p p p p p p p p p",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "P P P P P P P P P",
        ". B . . . . . R .",
        "L N S G K G S N L",
    ]


def get_fair_setup():
    """Fair board setup"""
    return [
        "cr cn cb cq ck g s n l",
        ". r . . . . . b .",
        "cp cp cp cp p p p p p",
        ". . . . . . . . .",
        ". . . . . . . . .",
        ". . . . . . . . .",
        "P P P P P CP CP CP CP",
        ". B . . . . . R .",
        "L N S G K CQ CB CN CR",
    ]


def visualize_board(setup, title):
    """Display board with colors"""
    print(f"\n{Colors.BOLD}{Colors.CYAN}{'=' * 60}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{title}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 60}{Colors.RESET}\n")
    
    # Column headers
    print("  ", end="")
    for i in range(9):
        print(f" {i} ", end="")
    print()
    
    for row_idx, row in enumerate(setup):
        print(f"{row_idx} ", end="")
        pieces = row.split()
        for piece in pieces:
            if piece == ".":
                print(f" {Colors.WHITE}.{Colors.RESET} ", end="")
            elif piece.isupper():
                # Player 1 (uppercase)
                if piece.startswith("C"):
                    # Chess piece
                    print(f"{Colors.GREEN}{piece}{Colors.RESET}", end="")
                else:
                    # Shogi piece
                    print(f"{Colors.BLUE}{piece}{Colors.RESET} ", end="")
            else:
                # Player 2 (lowercase)
                if piece.startswith("c"):
                    # Chess piece
                    print(f"{Colors.YELLOW}{piece}{Colors.RESET}", end="")
                else:
                    # Shogi piece
                    print(f"{Colors.RED}{piece}{Colors.RESET} ", end="")
        print()
    
    print(f"\n{Colors.GREEN}Player 1 (Upper){Colors.RESET}: Blue=Shogi, Green=Chess")
    print(f"{Colors.RED}Player 2 (Lower){Colors.RESET}: Red=Shogi, Yellow=Chess")
    print()


def analyze_piece_values(setup, title):
    """Analyze material and positional advantages"""
    print(f"{Colors.BOLD}{Colors.BLUE}ANALYSIS: {title}{Colors.RESET}")
    print(f"{Colors.CYAN}{'─' * 60}{Colors.RESET}\n")
    
    # Piece values (rough estimates)
    piece_values = {
        # Shogi pieces
        'k': 0, 'K': 0,  # King (invaluable)
        'r': 9, 'R': 9,  # Rook
        'b': 8, 'B': 8,  # Bishop
        'g': 6, 'G': 6,  # Gold
        's': 5, 'S': 5,  # Silver
        'n': 3, 'N': 3,  # Knight
        'l': 3, 'L': 3,  # Lance
        'p': 1, 'P': 1,  # Pawn
        # Chess pieces
        'cq': 9, 'CQ': 9,  # Queen
        'cr': 5, 'CR': 5,  # Rook
        'cb': 3, 'CB': 3,  # Bishop
        'cn': 3, 'CN': 3,  # Knight
        'cp': 1, 'CP': 1,  # Pawn
        'ck': 0, 'CK': 0,  # King
    }
    
    p1_material = 0
    p2_material = 0
    p1_pieces = {}
    p2_pieces = {}
    
    # King positions
    p1_king_pos = None
    p2_king_pos = None
    
    for row_idx, row in enumerate(setup):
        for col_idx, piece in enumerate(row.split()):
            if piece == ".":
                continue
            
            # Determine player
            is_p1 = piece[0].isupper()
            
            # Get piece value
            piece_lower = piece.lower()
            value = piece_values.get(piece_lower, 0)
            
            if is_p1:
                p1_material += value
                p1_pieces[piece_lower] = p1_pieces.get(piece_lower, 0) + 1
                if piece_lower in ['k', 'ck']:
                    p1_king_pos = (row_idx, col_idx)
            else:
                p2_material += value
                p2_pieces[piece_lower] = p2_pieces.get(piece_lower, 0) + 1
                if piece_lower in ['k', 'ck']:
                    p2_king_pos = (row_idx, col_idx)
    
    print(f"  {Colors.BOLD}Material Count:{Colors.RESET}")
    print(f"    Player 1: {Colors.GREEN}{p1_material}{Colors.RESET} points")
    print(f"    Player 2: {Colors.RED}{p2_material}{Colors.RESET} points")
    print(f"    Difference: {Colors.YELLOW}{abs(p1_material - p2_material)}{Colors.RESET} points")
    
    if p1_material != p2_material:
        print(f"    {Colors.YELLOW}⚠ Material imbalance detected!{Colors.RESET}")
    else:
        print(f"    {Colors.GREEN}✓ Material is balanced{Colors.RESET}")
    
    print()
    
    # King safety analysis
    print(f"  {Colors.BOLD}King Positions:{Colors.RESET}")
    if p1_king_pos:
        print(f"    Player 1 King: Row {p1_king_pos[0]}, Col {p1_king_pos[1]}")
    if p2_king_pos:
        print(f"    Player 2 King: Row {p2_king_pos[0]}, Col {p2_king_pos[1]}")
    
    # Check if kings are on same column (symmetry)
    if p1_king_pos and p2_king_pos:
        if p1_king_pos[1] == p2_king_pos[1]:
            print(f"    {Colors.GREEN}✓ Kings are vertically aligned (symmetric){Colors.RESET}")
        else:
            print(f"    {Colors.YELLOW}⚠ Kings are NOT vertically aligned!{Colors.RESET}")
            print(f"      P1 King Col: {p1_king_pos[1]}, P2 King Col: {p2_king_pos[1]}")
            print(f"      Offset: {abs(p1_king_pos[1] - p2_king_pos[1])} columns")
    
    print()
    
    # Piece composition
    print(f"  {Colors.BOLD}Piece Composition:{Colors.RESET}")
    
    all_piece_types = set(p1_pieces.keys()) | set(p2_pieces.keys())
    for piece_type in sorted(all_piece_types):
        p1_count = p1_pieces.get(piece_type, 0)
        p2_count = p2_pieces.get(piece_type, 0)
        
        if p1_count != p2_count:
            diff_marker = f"{Colors.YELLOW}⚠{Colors.RESET}"
        else:
            diff_marker = f"{Colors.GREEN}✓{Colors.RESET}"
        
        print(f"    {diff_marker} {piece_type:3s}: P1={p1_count}, P2={p2_count}")
    
    print()


def compare_boards():
    """Compare ShogiOnly and Fair boards"""
    print(f"\n{Colors.BOLD}{Colors.MAGENTA}{'=' * 60}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.MAGENTA}BOARD COMPARISON: ShogiOnly vs Fair{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.MAGENTA}{'=' * 60}{Colors.RESET}\n")
    
    print(f"{Colors.BOLD}Key Differences:{Colors.RESET}\n")
    
    print(f"  {Colors.CYAN}1. ShogiOnly:{Colors.RESET}")
    print(f"     • Both players use identical Shogi pieces")
    print(f"     • Perfect mirror symmetry")
    print(f"     • Traditional Shogi starting position")
    
    print(f"\n  {Colors.CYAN}2. Fair:{Colors.RESET}")
    print(f"     • Each player has a MIX of Chess and Shogi pieces")
    print(f"     • Player 1: Shogi left side, Chess right side")
    print(f"     • Player 2: Chess left side, Shogi right side")
    print(f"     • {Colors.YELLOW}Kings are NOT on the same column!{Colors.RESET}")
    
    print(f"\n  {Colors.BOLD}{Colors.RED}CRITICAL ASYMMETRY IN FAIR BOARD:{Colors.RESET}")
    print(f"     • P1 King at column 4 (center)")
    print(f"     • P2 King at column 4 (center)")
    print(f"     • BUT: Piece arrangements around kings are DIFFERENT")
    print(f"     • P1 has Shogi pieces on left, Chess on right")
    print(f"     • P2 has Chess pieces on left, Shogi on right")
    print(f"     • This creates ASYMMETRIC tactical opportunities!")
    
    print(f"\n  {Colors.BOLD}{Colors.YELLOW}HYPOTHESIS:{Colors.RESET}")
    print(f"     • Different piece types have different mobility patterns")
    print(f"     • Chess pieces (Queen, Rook, Bishop) have longer range")
    print(f"     • Shogi pieces (Gold, Silver) have shorter range but can drop")
    print(f"     • The POSITION of these pieces relative to the king matters!")
    print(f"     • Fair board's asymmetry may favor one player's piece arrangement")
    
    print()


def main():
    shogi_setup = get_shogi_setup()
    fair_setup = get_fair_setup()
    
    # Visualize boards
    visualize_board(shogi_setup, "ShogiOnly Board Setup")
    analyze_piece_values(shogi_setup, "ShogiOnly")
    
    print("\n" + "=" * 60 + "\n")
    
    visualize_board(fair_setup, "Fair Board Setup")
    analyze_piece_values(fair_setup, "Fair")
    
    # Compare
    compare_boards()
    
    # Final recommendations
    print(f"\n{Colors.BOLD}{Colors.GREEN}RECOMMENDATIONS:{Colors.RESET}\n")
    
    print(f"  {Colors.CYAN}1. Test with Reversed Boards:{Colors.RESET}")
    print(f"     Run games with players swapped to confirm bias direction")
    
    print(f"\n  {Colors.CYAN}2. Analyze Piece Mobility:{Colors.RESET}")
    print(f"     Count average legal moves per turn for each player")
    
    print(f"\n  {Colors.CYAN}3. Examine Opening Advantage:{Colors.RESET}")
    print(f"     Analyze first 10 moves to see if one player gains early advantage")
    
    print(f"\n  {Colors.CYAN}4. Evaluation Function Audit:{Colors.RESET}")
    print(f"     Test if AI evaluates initial position as equal (should be ~0)")
    
    print()


if __name__ == "__main__":
    main()
