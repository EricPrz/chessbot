from typing import List, Tuple
from stockfish import Stockfish
import chess
from time import time

class Node:
    def __init__(self, fen: str, stockfish: Stockfish, move_made: chess.Move | None, evaluation: float | None):
        self.fen = fen
        self.evaluation = None
        self.move_made = move_made
        self.children = None

    def evaluate(self, stockfish: Stockfish) -> float:
        stockfish.set_fen_position(fen_position=self.fen)
        evaluation = stockfish.get_static_eval()
        self.evaluation = evaluation

        if evaluation is None:
            return 0

        return evaluation

    def create_children(self, stockfish: Stockfish):
        board = chess.Board(self.fen)
        moves = board.generate_legal_moves()

        children = []
        for move in moves:
            temp_board = board.copy()

            temp_board.push(move)
            fen = temp_board.fen()

            child = Node(fen, stockfish, move_made=move, evaluation=None)
            children.append(child)

        # Sort children by evaluation
        # Check parent turn
        # turn = board.turn
        # is_white = turn == chess.WHITE
        #
        # self.children = sorted(children, key = lambda c: c.evaluation, reverse=is_white)
        self.children = children

    def get_children(self, stockfish: Stockfish) -> List['Node'] | None:
        if self.children is None:
            self.create_children(stockfish)
        return self.children
        

def alpha_beta_with_generation(node: Node, stockfish: Stockfish, depth: int, 
                               alpha: float, beta: float, is_maximizing: bool) -> Tuple[Node | None, float]:
    """Alpha-beta search that generates children as needed"""
    
    # Terminal condition
    if depth == 0:
        return None, node.evaluate(stockfish)
    
    # Generate children ONLY when needed
    if node.get_children(stockfish) is None:
        return None, node.evaluate(stockfish)

    children = node.get_children(stockfish)

    if is_maximizing:
        best_move = None
        max_eval = -float('inf')
        for child in children:
            _, eval = alpha_beta_with_generation(child, stockfish, depth - 1, alpha, beta, False)
            if eval > max_eval:
                max_eval = eval
                best_move = child
            alpha = max(alpha, eval)
            if beta <= alpha:
                break  # Beta cutoff
        return best_move, max_eval
    else:
        best_move = None
        min_eval = float('inf')
        for child in children:
            _, eval = alpha_beta_with_generation(child, stockfish, depth - 1, alpha, beta, True)
            if eval < min_eval:
                min_eval = eval
                best_move = child
            beta = min(beta, eval)
            if beta <= alpha:
                break  # Alpha cutoff
        return best_move, min_eval

# Usage
stockfish = Stockfish("/home/eric/Projects/chessbot/stockfish/stockfish-ubuntu-x86-64-avx2")
board = chess.Board()
root = Node(board.fen(), stockfish, move_made=None, evaluation=None)

while True:
    inp = input("Enter move: ")
    init = time()

    if len(inp.split()) > 1 and inp.split()[1] == "p":
        move, command = inp.split()
        try:
            board.push_san(move)
            new_node = None
            if root.children is not None:
                for child in root.get_children(stockfish):
                    if child.move_made == move:
                        new_node = child
            if new_node == None:
                root = Node(board.fen(), stockfish, move_made=None, evaluation=None)
            else:
                root = new_node
        except:
            print("Wrong Move, try again")
            continue
        fen = board.fen()

        turn = board.turn
        is_white = turn == chess.WHITE

        print("Thinking...")
        move, best_score = alpha_beta_with_generation(root, stockfish, depth=4, alpha=-float('inf'), beta=float('inf'), is_maximizing=is_white)
        print(move.move_made, best_score)

        root = move

        board.push(move.move_made)
    elif inp == "p":
        fen = board.fen()

        turn = board.turn
        is_white = turn == chess.WHITE

        print("Thinking...")
        move, best_score = alpha_beta_with_generation(root, stockfish, depth=4, alpha=-float('inf'), beta=float('inf'), is_maximizing=is_white)
        print(move.move_made, best_score)

        root = move

        board.push(move.move_made)
    else:
        move = inp
        try:
            board.push_san(move)
        except:
            print("Wrong Move, try again")
            continue
        
    print(f"Took {time() - init} seconds.")

