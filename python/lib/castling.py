from dataclasses import dataclass

from .enums import Color

@dataclass
class CastlingRights:
    white_kingside: bool = True
    white_queenside: bool = True
    black_kingside: bool = True
    black_queenside: bool = True
    
    def copy(self) -> 'CastlingRights':
        return CastlingRights(
            self.white_kingside,
            self.white_queenside,
            self.black_kingside,
            self.black_queenside
        )
    
    def get_for_color(self, color: Color):
        if color == Color.WHITE:
            return (self.white_kingside, self.white_queenside)
        else:
            return (self.black_kingside, self.black_queenside)
    
    def to_string(self) -> str:
        s = ""
        if self.white_kingside: s += "K"
        if self.white_queenside: s += "Q"
        if self.black_kingside: s += "k"
        if self.black_queenside: s += "q"
        return s if s else "-"
    
    @staticmethod
    def from_string(s: str) -> 'CastlingRights':
        cr = CastlingRights(False, False, False, False)
        if s == "-":
            return cr
        cr.white_kingside = 'K' in s
        cr.white_queenside = 'Q' in s
        cr.black_kingside = 'k' in s
        cr.black_queenside = 'q' in s
        return cr

