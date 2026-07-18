from dataclasses import dataclass

@dataclass(frozen=True)
class Position:
    x: int  # 0-7 (file)
    y: int  # 0-7 (rank)
    
    # def __post_init__(self):
    #     if not (0 <= self.x < 8 and 0 <= self.y < 8):
    #         raise ValueError(f"Invalid position: ({self.x}, {self.y})")
    
    def to_uci(self) -> str:
        return f"{chr(ord('a') + self.x)}{self.y + 1}"

    def offset(self, dx: int, dy: int) -> 'Position | None':
        """Create a new position with offset, returns None if invalid."""
        new_pos = Position(self.x + dx, self.y + dy)
        if new_pos.is_valid():
            return new_pos
        return None
    
    @staticmethod
    def from_uci(s: str) -> 'Position':
        if len(s) != 2:
            raise ValueError(f"Invalid UCI position: {s}")
        x = ord(s[0].lower()) - ord('a')
        y = int(s[1]) - 1
        return Position(x, y)
    
    def is_valid(self) -> bool:
        return 0 <= self.x < 8 and 0 <= self.y < 8

