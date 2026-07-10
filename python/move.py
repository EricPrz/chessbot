from dataclasses import dataclass, field
from typing import Optional

from position import Position
from piece import Piece
from enums import PieceType

@dataclass
class Move:
    from_pos: Position
    to_pos: Position
    piece: Piece
    captured: Optional[Piece] = None
    promotion: Optional[PieceType] = None
    is_castle: bool = False
    is_en_passant: bool = False
    
    def to_uci(self) -> str:
        s = f"{self.from_pos.to_uci()}{self.to_pos.to_uci()}"
        if self.promotion:
            s += self.promotion.to_char().lower()
        return s
    
    def to_san(self, board: 'Board') -> str:
        """Simple SAN representation (not fully disambiguated)."""
        if self.is_castle:
            return "O-O" if self.to_pos.x > self.from_pos.x else "O-O-O"
        
        piece_char = "" if self.piece.type == PieceType.PAWN else self.piece.type.to_char().upper()
        
        capture = "x" if self.captured or self.is_en_passant else ""
        
        promotion = ""
        if self.promotion:
            promotion = f"={self.promotion.to_char().upper()}"
        
        # Handle pawn captures
        if self.piece.type == PieceType.PAWN and capture:
            piece_char = self.from_pos.to_uci()[0]
        
        check = ""
        # Check/checkmate would be determined from resulting position
        
        return f"{piece_char}{capture}{self.to_pos.to_uci()}{promotion}{check}"
    
    def copy(self) -> 'Move':
        return Move(
            self.from_pos,
            self.to_pos,
            self.piece.copy(),
            self.captured.copy() if self.captured else None,
            self.promotion,
            self.is_castle,
            self.is_en_passant
        )
    
    def __str__(self) -> str:
        return self.to_uci()

