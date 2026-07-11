from .game import Game

def test_initial_position():
    """Test that initial position is correct."""
    game = Game()
    print("Initial position:")
    print(game)
    
    # Test FEN
    expected_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    actual_fen = game.get_fen()
    print(f"FEN: {actual_fen}")
    assert actual_fen == expected_fen, f"FEN mismatch: {actual_fen} vs {expected_fen}"
    
    # Test move generation
    moves = game.get_legal_moves_uci()
    print(f"Initial moves: {len(moves)}")
    print(f"First 10 moves: {moves[:10]}")
    
    print("✓ All tests passed!")


def test_game_tree():
    """Test that game tree generation works."""
    game = Game()
    
    # Get all legal moves from start position
    next_states = game.get_legal_moves()
    print(f"Generated {len(next_states)} next states")
    
    # Test a few moves
    for i, state in enumerate(next_states[:5]):
        assert state.move_made is not None
        print(f"Move {i+1}: {state.move_made.to_uci()} -> FEN: {state.get_fen()}")
    
    print("✓ Game tree generation works!")


def test_nnue_encoding():
    """Test NNUE encoding."""
    game = Game()
    encoding = game.get_nnue_encoding()
    print(f"NNUE encoding length: {len(encoding)}")
    print(f"Number of active features: {sum(encoding)}")
    assert len(encoding) == 896, f"Encoding length should be 896, got {len(encoding)}"
    print("✓ NNUE encoding works!")


if __name__ == "__main__":
    print("Testing Chess Engine...\n")
    test_initial_position()
    print()
    test_game_tree()
    print()
    test_nnue_encoding()
    print("\n✅ All tests passed! Engine is ready for NNUE training!")

