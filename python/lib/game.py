from typing import List, Optional, Tuple

from .enums import Color, PieceType
from .castling import CastlingRights
from .position import Position
from .move import Move
from .piece import Piece
from .board import Board

class Game:
    def __init__(self, board: Optional[Board] = None, fen: Optional[str] = None):
        self.board = board or Board()
        self.turn = Color.WHITE
        self.castling = CastlingRights()
        self.en_passant: Optional[Position] = None
        self.halfmove_clock = 0
        self.fullmove_number = 1
        self.parent: Optional['Game'] = None
        self.move_made: Optional[Move] = None
        
        if fen:
            self._load_from_fen(fen)
        elif not board:
            self._setup_initial_position()
    
    def _setup_initial_position(self):
        """Set up the initial chess position."""
        self.board = Board()
        
        # Setup pawns
        for x in range(8):
            self.board.set_piece(Position(x, 1), Piece(PieceType.PAWN, Color.WHITE))
            self.board.set_piece(Position(x, 6), Piece(PieceType.PAWN, Color.BLACK))
        
        # Setup pieces
        back_rank = [PieceType.ROOK, PieceType.KNIGHT, PieceType.BISHOP,
                     PieceType.QUEEN, PieceType.KING, PieceType.BISHOP,
                     PieceType.KNIGHT, PieceType.ROOK]
        
        for x, pt in enumerate(back_rank):
            self.board.set_piece(Position(x, 0), Piece(pt, Color.WHITE))
            self.board.set_piece(Position(x, 7), Piece(pt, Color.BLACK))
        
        self.turn = Color.WHITE
        self.castling = CastlingRights()
        self.en_passant = None
        self.halfmove_clock = 0
        self.fullmove_number = 1
    
    def _load_from_fen(self, fen: str):
        """Load game state from FEN string."""
        parts = fen.split()
        
        # Parse board position
        self.board = Board.from_fen(parts[0])
        
        # Parse active color
        if len(parts) > 1:
            self.turn = Color.from_char(parts[1])
        
        # Parse castling rights
        if len(parts) > 2:
            self.castling = CastlingRights.from_string(parts[2])
        
        # Parse en passant
        if len(parts) > 3 and parts[3] != '-':
            self.en_passant = Position.from_uci(parts[3])
        else:
            self.en_passant = None
        
        # Parse halfmove clock
        if len(parts) > 4:
            self.halfmove_clock = int(parts[4])
        
        # Parse fullmove number
        if len(parts) > 5:
            self.fullmove_number = int(parts[5])
    
    def get_legal_moves(self) -> List['Game']:
        """Generate all legal moves as new Game states."""
        legal_moves = []
        
        # Generate pseudo-legal moves
        for move in self._generate_pseudo_legal_moves():
            # Check if move is legal (doesn't leave king in check)
            if self._is_legal_move(move):
                # Create new game state
                new_game = self.copy()
                new_game._apply_move(move)
                new_game.parent = self
                new_game.move_made = move
                legal_moves.append(new_game)
        
        return legal_moves
    
    def get_legal_moves_uci(self) -> List[str]:
        """Get legal moves as UCI strings."""
        return [move.to_uci() for move in self._generate_pseudo_legal_moves()
                if self._is_legal_move(move)]
    
    def _generate_pseudo_legal_moves(self) -> List[Move]:
        """Generate all pseudo-legal moves for current position."""
        moves = []
        
        # Iterate through all pieces on board
        for y in range(8):
            for x in range(8):
                pos = Position(x, y)
                piece = self.board.get_piece(pos)
                if piece and piece.color == self.turn:
                    # Get pseudo-legal moves for this piece
                    for target in piece.get_pseudo_legal_moves(pos, self.board):
                        moves.append(Move(
                            from_pos=pos,
                            to_pos=target,
                            piece=piece.copy(),
                            captured=self.board.get_piece(target).copy() if self.board.get_piece(target) else None
                        ))
        
        # Add special moves (castling, en passant, promotions)
        moves.extend(self._get_castling_moves())
        moves.extend(self._get_en_passant_moves())
        
        # Handle promotions
        promotion_moves = []
        for move in moves:
            if move.piece.type == PieceType.PAWN:
                if (move.to_pos.y == 7 and move.piece.color == Color.WHITE) or \
                   (move.to_pos.y == 0 and move.piece.color == Color.BLACK):
                    # Generate promotion moves (queen, rook, bishop, knight)
                    for pt in [PieceType.QUEEN, PieceType.ROOK, 
                              PieceType.BISHOP, PieceType.KNIGHT]:
                        promo_move = move.copy()
                        promo_move.promotion = pt
                        promotion_moves.append(promo_move)

        if promotion_moves:
            # Replace pawn moves with promotion moves
            moves = [m for m in moves if not 
                    ((m.to_pos.y == 7 and m.piece.color == Color.WHITE and m.piece.type == PieceType.PAWN) or
                     (m.to_pos.y == 0 and m.piece.color == Color.BLACK and m.piece.type == PieceType.PAWN))]
            moves.extend(promotion_moves)

        return moves
    
    def _get_castling_moves(self) -> List[Move]:
        """Generate castling moves if available."""
        moves = []
        if self.turn == Color.WHITE:
            king_pos = self.board.find_king(Color.WHITE)
            if not king_pos:
                return moves
            
            # Kingside castling
            if self.castling.white_kingside:
                if (not self.board.get_piece(Position(5, 0)) and 
                    not self.board.get_piece(Position(6, 0))):
                    if (not self.board.is_attacked(Position(4, 0), Color.BLACK) and
                        not self.board.is_attacked(Position(5, 0), Color.BLACK) and
                        not self.board.is_attacked(Position(6, 0), Color.BLACK)):
                        # King moves from e1 to g1
                        move = Move(
                            from_pos=Position(4, 0),
                            to_pos=Position(6, 0),
                            piece=Piece(PieceType.KING, Color.WHITE),
                            is_castle=True
                        )
                        moves.append(move)
            
            # Queenside castling
            if self.castling.white_queenside:
                if (not self.board.get_piece(Position(3, 0)) and 
                    not self.board.get_piece(Position(2, 0)) and
                    not self.board.get_piece(Position(1, 0))):
                    if (not self.board.is_attacked(Position(4, 0), Color.BLACK) and
                        not self.board.is_attacked(Position(3, 0), Color.BLACK) and
                        not self.board.is_attacked(Position(2, 0), Color.BLACK)):
                        # King moves from e1 to c1
                        move = Move(
                            from_pos=Position(4, 0),
                            to_pos=Position(2, 0),
                            piece=Piece(PieceType.KING, Color.WHITE),
                            is_castle=True
                        )
                        moves.append(move)
        else:
            # Similar for black
            king_pos = self.board.find_king(Color.BLACK)
            if not king_pos:
                return moves
            
            if self.castling.black_kingside:
                if (not self.board.get_piece(Position(5, 7)) and 
                    not self.board.get_piece(Position(6, 7))):
                    if (not self.board.is_attacked(Position(4, 7), Color.WHITE) and
                        not self.board.is_attacked(Position(5, 7), Color.WHITE) and
                        not self.board.is_attacked(Position(6, 7), Color.WHITE)):
                        move = Move(
                            from_pos=Position(4, 7),
                            to_pos=Position(6, 7),
                            piece=Piece(PieceType.KING, Color.BLACK),
                            is_castle=True
                        )
                        moves.append(move)
            
            if self.castling.black_queenside:
                if (not self.board.get_piece(Position(3, 7)) and 
                    not self.board.get_piece(Position(2, 7)) and
                    not self.board.get_piece(Position(1, 7))):
                    if (not self.board.is_attacked(Position(4, 7), Color.WHITE) and
                        not self.board.is_attacked(Position(3, 7), Color.WHITE) and
                        not self.board.is_attacked(Position(2, 7), Color.WHITE)):
                        move = Move(
                            from_pos=Position(4, 7),
                            to_pos=Position(2, 7),
                            piece=Piece(PieceType.KING, Color.BLACK),
                            is_castle=True
                        )
                        moves.append(move)
        
        return moves
    
    def _get_en_passant_moves(self) -> List[Move]:
        """Generate en passant moves."""
        moves = []
        if not self.en_passant:
            return moves
        
        # Check if there's a pawn that can capture en passant
        for dx in [-1, 1]:
            pawn_pos = Position(self.en_passant.x + dx, self.en_passant.y + (-1 if self.turn == Color.WHITE else 1))
            if pawn_pos.is_valid():
                piece = self.board.get_piece(pawn_pos)
                if piece and piece.color == self.turn and piece.type == PieceType.PAWN:
                    move = Move(
                        from_pos=pawn_pos,
                        to_pos=self.en_passant,
                        piece=piece.copy(),
                        captured=Piece(PieceType.PAWN, self.turn.opposite()),
                        is_en_passant=True
                    )
                    moves.append(move)
        
        return moves
    
    def _is_legal_move(self, move: Move) -> bool:
        """Check if a move is legal (doesn't leave king in check)."""
        if not move.from_pos.is_valid() or not move.to_pos.is_valid():
            return False

        # Create a copy of the board
        board_copy = self.board.copy()
        
        # Make the move on the copy
        piece = board_copy.get_piece(move.from_pos)
        if not piece:
            return False
        
        # Special handling for castling
        if move.is_castle:
            # Move king
            board_copy.move_piece(move.from_pos, move.to_pos)
            
            # Move rook
            if move.to_pos.x > move.from_pos.x:  # Kingside
                board_copy.move_piece(Position(7, move.from_pos.y), Position(5, move.from_pos.y))
            else:  # Queenside
                board_copy.move_piece(Position(0, move.from_pos.y), Position(3, move.from_pos.y))
        else:
            # Move piece
            board_copy.move_piece(move.from_pos, move.to_pos)
            
            # Handle en passant capture
            if move.is_en_passant:
                captured_pos = Position(move.to_pos.x, move.from_pos.y)
                board_copy.set_piece(captured_pos, None)
        
        # Check if our king is in check after this move
        king_pos = board_copy.find_king(self.turn)
        if not king_pos:
            return False
        
        return not board_copy.is_attacked(king_pos, self.turn.opposite())
    
    def _apply_move(self, move: Move):
        """Apply a move to the current board state."""
        # Update halfmove clock
        if move.piece.type == PieceType.PAWN or move.captured:
            self.halfmove_clock = 0
        else:
            self.halfmove_clock += 1
        
        # Update en passant
        if move.piece.type == PieceType.PAWN and abs(move.to_pos.y - move.from_pos.y) == 2:
            self.en_passant = Position(move.from_pos.x, (move.from_pos.y + move.to_pos.y) // 2)
        else:
            self.en_passant = None
        
        # Handle castling
        if move.is_castle:
            # Move king
            self.board.move_piece(move.from_pos, move.to_pos)
            # Move rook
            if move.to_pos.x > move.from_pos.x:  # Kingside
                self.board.move_piece(Position(7, move.from_pos.y), Position(5, move.from_pos.y))
            else:  # Queenside
                self.board.move_piece(Position(0, move.from_pos.y), Position(3, move.from_pos.y))
            
            # Update castling rights
            if move.piece.color == Color.WHITE:
                self.castling.white_kingside = False
                self.castling.white_queenside = False
            else:
                self.castling.black_kingside = False
                self.castling.black_queenside = False
        else:
            # Handle en passant capture
            if move.is_en_passant:
                captured_pos = Position(move.to_pos.x, move.from_pos.y)
                self.board.set_piece(captured_pos, None)
            
            # Move piece
            self.board.move_piece(move.from_pos, move.to_pos)
            
            # Handle promotion
            if move.promotion:
                self.board.set_piece(move.to_pos, Piece(move.promotion, self.turn))
            
            # Update castling rights
            if move.piece.type == PieceType.KING:
                if self.turn == Color.WHITE:
                    self.castling.white_kingside = False
                    self.castling.white_queenside = False
                else:
                    self.castling.black_kingside = False
                    self.castling.black_queenside = False
            elif move.piece.type == PieceType.ROOK:
                if move.from_pos.x == 0:  # Queenside rook
                    if self.turn == Color.WHITE and move.from_pos.y == 0:
                        self.castling.white_queenside = False
                    elif self.turn == Color.BLACK and move.from_pos.y == 7:
                        self.castling.black_queenside = False
                elif move.from_pos.x == 7:  # Kingside rook
                    if self.turn == Color.WHITE and move.from_pos.y == 0:
                        self.castling.white_kingside = False
                    elif self.turn == Color.BLACK and move.from_pos.y == 7:
                        self.castling.black_kingside = False
        
        # Switch turn
        self.turn = self.turn.opposite()
        
        # Update fullmove number
        if self.turn == Color.WHITE:
            self.fullmove_number += 1
    
    def is_check(self) -> bool:
        """Check if current player is in check."""
        king_pos = self.board.find_king(self.turn)
        if not king_pos:
            return False
        return self.board.is_attacked(king_pos, self.turn.opposite())
    
    def is_checkmate(self) -> bool:
        """Check if current player is in checkmate."""
        if not self.is_check():
            return False
        return not any(self.get_legal_moves())
    
    def is_stalemate(self) -> bool:
        """Check if current player is in stalemate."""
        if self.is_check():
            return False
        return not any(self.get_legal_moves())
    
    def is_draw(self) -> bool:
        """Check for draw conditions."""
        return self.halfmove_clock >= 100 or self.is_stalemate()
    
    def get_fen(self) -> str:
        """Get FEN representation of current position."""
        board_fen = self.board.to_fen()
        turn_fen = self.turn.to_char()
        castling_fen = self.castling.to_string()
        en_passant_fen = self.en_passant.to_uci() if self.en_passant else "-"
        
        return f"{board_fen} {turn_fen} {castling_fen} {en_passant_fen} {self.halfmove_clock} {self.fullmove_number}"
    
    def get_pgn(self) -> str:
        """Generate PGN of the game (basic version)."""
        if not self.parent:
            return ""
        
        # Reconstruct game tree
        game_history = []
        current = self
        while current.parent:
            game_history.append(current)
            current = current.parent
        
        # Generate PGN from move history
        pgn_moves = []
        game_number = 1
        for i, state in enumerate(reversed(game_history)):
            if not state.move_made:
                continue
            
            move_san = state.move_made.to_san(state.parent.board)
            if i % 2 == 0:
                pgn_moves.append(f"{game_number}. {move_san}")
                game_number += 1
            else:
                pgn_moves.append(move_san)
        
        return " ".join(pgn_moves)
    
    def get_nnue_encoding(self) -> List[int]:
        """
        Simple NNUE-like encoding (HalfKP).
        This is a basic version - for production NNUE you'd want more optimized encoding.
        """
        # 6 piece types × 2 colors × 64 squares = 768 features
        # Plus king position (64 × 2) for HalfKP = 768 + 128 = 896 features
        features = [0] * 896  # 768 + 128
        
        # Encode piece positions (6 × 2 × 64)
        for y in range(8):
            for x in range(8):
                pos = Position(x, y)
                piece = self.board.get_piece(pos)
                if piece:
                    # Piece type index (0-5)
                    pt_idx = piece.type.value - 1
                    # Color offset (0 for white, 6 for black)
                    color_offset = 0 if piece.color == Color.WHITE else 6
                    # Square index (0-63)
                    square_idx = y * 8 + x
                    # Feature index: (color_offset + pt_idx) * 64 + square_idx
                    features[(color_offset + pt_idx) * 64 + square_idx] = 1
        
        # Encode king positions (HalfKP)
        # White king
        white_king = self.board.find_king(Color.WHITE)
        if white_king:
            features[768 + (white_king.y * 8 + white_king.x)] = 1
        
        # Black king
        black_king = self.board.find_king(Color.BLACK)
        if black_king:
            features[768 + 64 + (black_king.y * 8 + black_king.x)] = 1
        
        return features
    
    def copy(self) -> 'Game':
        """Create a deep copy of the game."""
        new_game = Game()
        new_game.board = self.board.copy()
        new_game.turn = self.turn
        new_game.castling = self.castling.copy()
        new_game.en_passant = self.en_passant
        new_game.halfmove_clock = self.halfmove_clock
        new_game.fullmove_number = self.fullmove_number
        new_game.parent = self.parent
        new_game.move_made = self.move_made.copy() if self.move_made else None
        return new_game
    
    def __str__(self) -> str:
        """Print board for debugging."""
        result = "  a b c d e f g h\n"
        for y in range(7, -1, -1):
            result += f"{y+1} "
            for x in range(8):
                piece = self.board.get_piece(Position(x, y))
                result += f"{piece if piece else '.'} "
            result += f"{y+1}\n"
        result += "  a b c d e f g h\n"
        result += f"Turn: {self.turn.name}\n"
        result += f"Castling: {self.castling.to_string()}\n"
        result += f"En passant: {self.en_passant.to_uci() if self.en_passant else '-'}\n"
        result += f"Halfmove: {self.halfmove_clock}, Fullmove: {self.fullmove_number}\n"
        return result


