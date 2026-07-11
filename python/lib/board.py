from typing import Optional, Dict, List

from .enums import Color, PieceType
from .position import Position
from .piece import Piece


class Board:
    def __init__(self):
        self.grid = [[None for _ in range(8)] for _ in range(8)]
        self.kings: Dict[Color, Optional[Position]] = {
            Color.WHITE: None,
            Color.BLACK: None
        }
    
    def get_piece(self, pos: Position) -> Optional[Piece]:
        if not pos.is_valid():
            return None
        return self.grid[pos.y][pos.x]
    
    def set_piece(self, pos: Position, piece: Optional[Piece]) -> None:
        if not pos.is_valid():
            return
        self.grid[pos.y][pos.x] = piece
        if piece and piece.type == PieceType.KING:
            self.kings[piece.color] = pos
    
    def move_piece(self, from_pos: Position, to_pos: Position) -> Optional[Piece]:
        """Move piece from from_pos to to_pos. Returns captured piece."""
        piece = self.get_piece(from_pos)
        if not piece:
            return None
        
        captured = self.get_piece(to_pos)
        self.set_piece(to_pos, piece)
        self.set_piece(from_pos, None)
        
        return captured
    
    def is_occupied(self, pos: Position) -> bool:
        return self.get_piece(pos) is not None
    
    def is_occupied_by_color(self, pos: Position, color: Color) -> bool:
        piece = self.get_piece(pos)
        return piece is not None and piece.color == color
    
    def find_king(self, color: Color) -> Optional[Position]:
        return self.kings.get(color)
    
    def is_attacked(self, pos: Position, by_color: Color) -> bool:
        """Check if position is attacked by any piece of given color."""
        # Check for pawn attacks
        pawn_direction = 1 if by_color == Color.WHITE else -1
        for dx in [-1, 1]:
            attack_pos = Position(pos.x + dx, pos.y + pawn_direction)
            if attack_pos.is_valid():
                piece = self.get_piece(attack_pos)
                if piece and piece.color == by_color and piece.type == PieceType.PAWN:
                    return True
        
        # Check for knight attacks
        knight_moves = [
            (2, 1), (2, -1), (-2, 1), (-2, -1),
            (1, 2), (1, -2), (-1, 2), (-1, -2)
        ]
        for dx, dy in knight_moves:
            attack_pos = Position(pos.x + dx, pos.y + dy)
            if attack_pos.is_valid():
                piece = self.get_piece(attack_pos)
                if piece and piece.color == by_color and piece.type == PieceType.KNIGHT:
                    return True
        
        # Check for sliding attacks (bishop, rook, queen)
        # Bishop directions
        for dx, dy in [(1, 1), (1, -1), (-1, 1), (-1, -1)]:
            current = Position(pos.x + dx, pos.y + dy)
            while current.is_valid():
                piece = self.get_piece(current)
                if piece:
                    if piece.color == by_color and piece.type in [PieceType.BISHOP, PieceType.QUEEN]:
                        return True
                    break
                current = Position(current.x + dx, current.y + dy)
        
        # Rook directions
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            current = Position(pos.x + dx, pos.y + dy)
            while current.is_valid():
                piece = self.get_piece(current)
                if piece:
                    if piece.color == by_color and piece.type in [PieceType.ROOK, PieceType.QUEEN]:
                        return True
                    break
                current = Position(current.x + dx, current.y + dy)
        
        # Check for king attacks
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                if dx == 0 and dy == 0:
                    continue
                attack_pos = Position(pos.x + dx, pos.y + dy)
                if attack_pos.is_valid():
                    piece = self.get_piece(attack_pos)
                    if piece and piece.color == by_color and piece.type == PieceType.KING:
                        return True
        
        return False
    
    def copy(self) -> 'Board':
        """Create a deep copy of the board."""
        new_board = Board()
        for y in range(8):
            for x in range(8):
                pos = Position(x, y)
                piece = self.get_piece(pos)
                if piece:
                    new_board.set_piece(pos, piece.copy())
        return new_board
    
    def to_fen(self) -> str:
        """Convert board to FEN string (piece placement only)."""
        fen = ""
        for y in range(7, -1, -1):
            empty = 0
            for x in range(8):
                piece = self.grid[y][x]
                if piece:
                    if empty > 0:
                        fen += str(empty)
                        empty = 0
                    fen += piece.to_char()
                else:
                    empty += 1
            if empty > 0:
                fen += str(empty)
            if y > 0:
                fen += "/"
        return fen
    
    @staticmethod
    def from_fen(fen: str) -> 'Board':
        """Create board from FEN string."""
        board = Board()
        parts = fen.split()
        fen_pieces = parts[0]
        
        y = 7
        x = 0
        for char in fen_pieces:
            if char == '/':
                y -= 1
                x = 0
            elif char.isdigit():
                x += int(char)
            else:
                color = Color.WHITE if char.isupper() else Color.BLACK
                piece_type = PieceType.from_char(char.lower())
                if piece_type:
                    board.set_piece(Position(x, y), Piece(piece_type, color))
                x += 1
        
        return board

