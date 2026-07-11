from lib.game import Game

# Create a game
game = Game(fen="rnbqkbnr/pppp1ppp/8/4p3/4P3/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 2")

# Generate all legal moves as new states
legal_states = game.get_legal_moves()
print(f"Found {len(legal_states)} legal moves")

# Play a move
next_game = legal_states[0]

assert next_game.move_made is not None

print(f"Played: {next_game.move_made.to_uci()}")
print(f"New position: {next_game.get_fen()}")

# Get NNUE encoding for training
encoding = next_game.get_nnue_encoding()
print(f"Encoding length: {len(encoding)}")

# Check game state
if next_game.is_check():
    print("Check!")
if next_game.is_checkmate():
    print("Checkmate!")
if next_game.is_draw():
    print("Draw!")
