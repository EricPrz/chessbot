from enum import Enum
from typing import Optional

class Color(Enum):
    WHITE = 1
    BLACK = -1
    
    def opposite(self) -> 'Color':
        return Color.WHITE if self == Color.BLACK else Color.BLACK
    
    def to_char(self) -> str:
        return 'w' if self == Color.WHITE else 'b'
    
    @staticmethod
    def from_char(c: str) -> 'Color':
        return Color.WHITE if c == 'w' else Color.BLACK


class PieceType(Enum):
    PAWN = 1
    KNIGHT = 2
    BISHOP = 3
    ROOK = 4
    QUEEN = 5
    KING = 6
    
    def to_char(self) -> str:
        return {
            PieceType.PAWN: 'p',
            PieceType.KNIGHT: 'n',
            PieceType.BISHOP: 'b',
            PieceType.ROOK: 'r',
            PieceType.QUEEN: 'q',
            PieceType.KING: 'k'
        }[self]
    
    def get_value(self) -> int:
        return {
            PieceType.PAWN: 1,
            PieceType.KNIGHT: 3,
            PieceType.BISHOP: 3,
            PieceType.ROOK: 5,
            PieceType.QUEEN: 9,
            PieceType.KING: 0
        }[self]
    
    @staticmethod
    def from_char(c: str) -> Optional['PieceType']:
        mapping = {
            'p': PieceType.PAWN, 'n': PieceType.KNIGHT,
            'b': PieceType.BISHOP, 'r': PieceType.ROOK,
            'q': PieceType.QUEEN, 'k': PieceType.KING
        }
        return mapping.get(c.lower())

