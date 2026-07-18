from dataclasses import dataclass
from typing import List, Optional, TYPE_CHECKING

from .enums import PieceType, Color
from .position import Position

if TYPE_CHECKING:
    from board import Board

@dataclass
class Piece:
    type: PieceType
    color: Color
    
    def get_pseudo_legal_moves(self, pos: Position, board: 'Board') -> List[Position]:
        """Generate pseudo-legal moves for this piece at given position."""
        moves = []
        
        if self.type == PieceType.PAWN:
            moves = self._get_pawn_moves(pos, board)
        elif self.type == PieceType.KNIGHT:
            moves = self._get_knight_moves(pos, board)
        elif self.type == PieceType.BISHOP:
            moves = self._get_sliding_moves(pos, board, [(1, 1), (1, -1), (-1, 1), (-1, -1)])
        elif self.type == PieceType.ROOK:
            moves = self._get_sliding_moves(pos, board, [(1, 0), (-1, 0), (0, 1), (0, -1)])
        elif self.type == PieceType.QUEEN:
            moves = self._get_sliding_moves(pos, board, [
                (1, 0), (-1, 0), (0, 1), (0, -1),
                (1, 1), (1, -1), (-1, 1), (-1, -1)
            ])
        elif self.type == PieceType.KING:
            moves = self._get_king_moves(pos, board)
        
        return moves
    
    def _get_pawn_moves(self, pos: Position, board: 'Board') -> List[Position]:
        moves = []
        direction = 1 if self.color == Color.WHITE else -1
        start_rank = 1 if self.color == Color.WHITE else 6
        
        # Forward one square
        new_pos = Position(pos.x, pos.y + direction)
        if new_pos.is_valid() and not board.is_occupied(new_pos):
            moves.append(new_pos)
            
            # Forward two squares from starting position
            new_pos2 = Position(pos.x, pos.y + 2 * direction)
            if pos.y == start_rank and not board.is_occupied(new_pos2):
                moves.append(new_pos2)
        
        # Captures
        for dx in [-1, 1]:
            new_pos = Position(pos.x + dx, pos.y + direction)
            if new_pos.is_valid():
                if board.is_occupied_by_color(new_pos, self.color.opposite()):
                    moves.append(new_pos)
                # En passant handled in legal move generation
        
        return moves
    
    def _get_knight_moves(self, pos: Position, board: 'Board') -> List[Position]:
        moves = []
        knight_moves = [
            (2, 1), (2, -1), (-2, 1), (-2, -1),
            (1, 2), (1, -2), (-1, 2), (-1, -2)
        ]
        for dx, dy in knight_moves:
            new_pos = Position(pos.x + dx, pos.y + dy)
            if new_pos.is_valid() and not board.is_occupied_by_color(new_pos, self.color):
                moves.append(new_pos)
        return moves
    
    def _get_sliding_moves(self, pos: Position, board: 'Board', directions: List[tuple]) -> List[Position]:
        moves = []
        for dx, dy in directions:
            current = Position(pos.x + dx, pos.y + dy)
            while current.is_valid():
                if board.is_occupied_by_color(current, self.color):
                    break
                moves.append(current)
                if board.is_occupied_by_color(current, self.color.opposite()):
                    break
                current = Position(current.x + dx, current.y + dy)

        return moves
    
    def _get_king_moves(self, pos: Position, board: 'Board') -> List[Position]:
        moves = []
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                if dx == 0 and dy == 0:
                    continue
                new_pos = Position(pos.x + dx, pos.y + dy)
                if new_pos.is_valid() and not board.is_occupied_by_color(new_pos, self.color):
                    moves.append(new_pos)
        return moves
    
    def copy(self) -> 'Piece':
        return Piece(self.type, self.color)
    
    def to_char(self) -> str:
        c = self.type.to_char()
        return c.upper() if self.color == Color.WHITE else c
    
    def __str__(self) -> str:
        return self.to_char()

