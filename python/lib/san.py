# import re
# from .board import Board
# from .enums import PieceType, Color
# from .move import Move, Piece
# from .position import Position
# from .game import Game
#
# DEBUG = True
# def printd(msg: str):
#     if DEBUG:
#         print(msg)
#
# def san_to_uci(san: str, game: Game) -> bool:
#     """ Returns boolean indicating succes."""
#     printd(f"\n\nFEN: {game.get_fen()}")
#     match = re.match(r"[a-z][0-9][a-z][0-9]", san)
#     printd(f"Is UCI: {match} in move: {san}")
#
#     board = game.board
#     turn = game.turn
#     printd(f"Turn: {turn}")
#
#     # Checking Enroques
#     is_castling = False
#     if san == "0-0" or san == "O-O" or san == "0-0-0" or san == "O-O-O":
#         printd("Enroque!!")
#         is_castling = True
#         moves = game._get_castling_moves()
#         moves_ = [move.to_uci() for move in moves]
#         printd(f"Castling Moves: {moves_}")
#         if len(moves) == 1:
#             game._apply_move(moves[0])
#             return True
#
#         king_side = len(san) == 3
#         printd(f"Casling on King Side: {king_side}")
#
#         for move in moves:
#             if king_side:
#                 if move.to_pos.x == 6:
#                     game._apply_move(move)
#                     return True
#
#             if move.to_pos.x == 2:
#                 game._apply_move(move)
#                 return True
#
#
#     # Checking result
#     if san == "0-1" or san == "1-0":
#         printd(f"Result: {san}")
#         return True
#
#
#     # Getting piece that makes move
#     piece_map = {
#         "Q": PieceType.QUEEN,
#         "K": PieceType.KING,
#         "R": PieceType.ROOK,
#         "B": PieceType.BISHOP,
#         "N": PieceType.KNIGHT
#     }
#     piece_type = piece_map.get(san[0], PieceType.PAWN)
#     printd(f"Piece Type: {piece_type}")
#     piece = Piece(piece_type, turn)
#     printd(f"Piece: {piece}")
#
#     # Checking if promotion
#     promoted_piece: PieceType | None = None
#     if "=" in san:
#         #exf8=Q
#         # Find promoted piece_type
#         piece_letter = san[san.find("=") + 1]
#         promoted_piece = piece_map.get(piece_letter, None)
#
#     printd(f"Promoted Piece: {promoted_piece}")
#
#     san = re.sub(r"\=[A-Z]", "", san)
#     printd(f"Cleaned promotion: {san}")
#
#
#     # Getting destination square
#     clean_san = re.sub(r"[+*!?#x]", "", san)
#     printd(f"Clean San: {clean_san}")
#     destination_square = clean_san[-2:]
#     printd(f"Destination Square: {destination_square}")
#     destination_position = Position.from_uci(destination_square)
#     printd(f"Destination Position: {destination_position}")
#
#     # Getting if on check or on mate
#     is_check = "+" in san
#     is_mate = "#" in san
#     printd(f"Is check: {is_check}")
#     printd(f"Is mate: {is_mate}")
#
#     # Checking if capture
#     is_capture = "x" in san
#     printd(f"Is capture: {is_capture}")
#     captured_piece = None
#     if is_capture:
#         captured_piece = board.get_piece(destination_position)
#
#
#     # Get origin square
#     from_position = get_origin_square(game, piece, san, clean_san, destination_position, turn, is_capture, promoted_piece)
#     printd(f"From Position: {from_position}")
#     if from_position is None:
#         return False
#
#     # Get en is_en_passant
#     is_en_passant = False
#     if piece_type == PieceType.PAWN and from_position.x != destination_position.x and board.get_piece(destination_position) == None:
#         is_en_passant = True
#
#     # Apply move
#     move = Move(from_position, destination_position, piece, captured_piece, promoted_piece, is_castling, is_en_passant)
#     game._apply_move(move)
#
#     return True
#
#
#
# def get_origin_square(game: Game, piece: Piece, og_san: str, clean_san: str, destination_position: Position, turn: Color, is_capture: bool, promoted_piece: Piece | None) -> Position:
#     # san = clean_san[:-2]
#     san = re.sub(r"[A-Z]", "", clean_san)
#     san = san[:-2]
#     printd(f"OOOOO SAN: {san}")
#
#     # If disambiguation
#     disambiguation_letter_idx = 0
#
#     disambiguations = {}
#
#     if san:
#
#         if san[disambiguation_letter_idx] in "abcdefgh":
#             file_idx = ord(san[disambiguation_letter_idx]) - ord('a')
#             disambiguations["file"] = file_idx
#             printd(f"Found file disambiguation: {file_idx}")
#
#             if len(san) >= disambiguation_letter_idx + 2 and san[disambiguation_letter_idx + 1] in "12345678":
#                 rank_idx = ord(san[disambiguation_letter_idx + 1]) - ord('1')
#                 printd(f"Found rank disambiguation: {rank_idx}")
#                 return Position(x=file_idx, y=rank_idx)
#
#         if san[disambiguation_letter_idx] in "12345678":
#             rank_idx = ord(san[disambiguation_letter_idx]) - ord('1')
#             disambiguations["rank"] = rank_idx
#             printd(f"Found rank disambiguation: {rank_idx}")
#
#     printd(f"disambiguations: {disambiguations}")
#
#     all_moves = game.get_legal_moves()
#     possible_moves: list[Game] = []
#     printd("Possible Moves:")
#     for move in all_moves:
#         if move.move_made.is_castle:
#             continue
#
#         if move.board.get_piece(destination_position) is None:
#             continue
#
#         if move.board.get_piece(destination_position) != piece:
#             if promoted_piece is None:
#                 continue
#             elif move.board.get_piece(destination_position) is None or (move.board.get_piece(destination_position) and move.board.get_piece(destination_position).type != promoted_piece):
#                 continue
#         printd(f"\t {move.move_made}")
#         possible_moves.append(move)
#
#
#     if len(possible_moves) == 1:
#         # Return only move
#         printd("Found only one move")
#         assert possible_moves[0].move_made is not None
#         return possible_moves[0].move_made.from_pos
#
#     if disambiguations.get("rank") is not None:
#         printd("Found rank diss")
#         rank = disambiguations.get("rank")
#         printd(f"Disss {rank}")
#         for move in possible_moves:
#             if move.move_made.from_pos.y == rank:
#                 return move.move_made.from_pos
#
#     if disambiguations.get("file") is not None:
#         printd("Found file diss")
#         file = disambiguations.get("file")
#         printd(f"Disss {file}")
#         for move in possible_moves:
#             if move.move_made.from_pos.x == file:
#                 return move.move_made.from_pos
#
#     print(game.get_fen())
#     print(game)
#     print(og_san)
#     print("Unable to find origin position")
#     # raise Exception("Unable to find origin position")


import re
from .board import Board
from .enums import PieceType, Color
from .move import Move, Piece
from .position import Position
from .game import Game

DEBUG = False
def printd(msg: str):
    if DEBUG:
        print(msg)

def san_to_uci(san: str, game: Game) -> bool:
    """ Returns boolean indicating success."""
    printd(f"\n\nFEN: {game.get_fen()}")
    
    # Strip any check, checkmate, or annotation marks immediately
    san_clean = re.sub(r"[+#!?* ]", "", san)
    printd(f"CLean SAN: {san_clean}")
    
    # 1. Castling Checks
    is_castling = False
    if san_clean in ("0-0", "O-O", "0-0-0", "O-O-O"):
        printd("Enroque!!")
        is_castling = True
        moves = game._get_castling_moves()
        printd(f"Enroque Moves: {moves}")
        
        king_side = len(san_clean) <= 3
        for move in moves:
            if king_side and move.to_pos.x == 6:
                game._apply_move(move)
                return True
            elif not king_side and move.to_pos.x == 2:
                game._apply_move(move)
                return True

        printd("No enroque")
        return False

    # 2. Game result strings
    if san_clean in ("0-1", "1-0", "1/2-1/2"):
        printd(f"Result: {san_clean}")
        return True

    piece_map = {
        "Q": PieceType.QUEEN,
        "K": PieceType.KING,
        "R": PieceType.ROOK,
        "B": PieceType.BISHOP,
        "N": PieceType.KNIGHT
    }

    # 3. Promotion
    promoted_piece: PieceType | None = None
    if "=" in san_clean:
        parts = san_clean.split("=")
        san_clean = parts[0]
        promoted_letter = parts[1][0]
        promoted_piece = piece_map.get(promoted_letter)
        printd(f"Promoted Piece: {promoted_piece}")

    if len(san_clean) < 2:
        printd(f"Failed to parse SAN (too short): {san}")
        return False

    # Target square is always the final two characters
    dest = san_clean[-2:]
    remainder = san_clean[:-2]

    if not re.match(r"^[a-h][1-8]$", dest):
        printd(f"Failed to parse SAN (invalid destination): {san}")
        return False

    destination_position = Position.from_uci(dest)
    printd(f"Destination Position: {destination_position}")

    is_capture = "x" in remainder
    if is_capture:
        remainder = remainder.replace("x", "")
    printd(f"Is Capture: {is_capture}")

    # Identify moving piece
    piece_type = PieceType.PAWN
    if remainder and remainder[0] in piece_map:
        piece_type = piece_map[remainder[0]]
        remainder = remainder[1:]
    printd(f"Piece Type: {piece_type}")

    # Remaining characters represent disambiguation (e.g., 'a', '1', or 'a1')
    dis_file = None
    dis_rank = None

    if len(remainder) == 1:
        if remainder in "abcdefgh":
            dis_file = remainder
        elif remainder in "12345678":
            dis_rank = remainder
    elif len(remainder) == 2:
        if remainder[0] in "abcdefgh" and remainder[1] in "12345678":
            dis_file = remainder[0]
            dis_rank = remainder[1]

    req_file = ord(dis_file) - ord('a') if dis_file else None
    req_rank = ord(dis_rank) - ord('1') if dis_rank else None

    turn = game.turn
    piece = Piece(piece_type, turn)
    printd(f"Piece: {piece}")

    # 4. Fetch Legal Moves safely handling both Move objects and MoveState wraps
    all_moves_raw = game.get_legal_moves()
    possible_moves = []

    for item in all_moves_raw:
        # Check if legal moves yields Move directly or wrapped in MoveState
        if hasattr(item, 'move_made'):
            mv = item.move_made
        else:
            mv = item
        printd(f"Raw Move: {item.move_made}")

        if mv is None:
            continue
            
        if mv.is_castle:
            continue

        if mv.to_pos != destination_position:
            continue

        if mv.piece.type != piece_type or mv.piece.color != turn:
            continue

        if req_file is not None and mv.from_pos.x != req_file:
            continue

        if req_rank is not None and mv.from_pos.y != req_rank:
            continue

        possible_moves.append(mv)

    if len(possible_moves) >= 1:
        # If ambiguous and no disambiguation helper was provided, default to the first matching legal move
        chosen_move = possible_moves[0]
    else:
        print(game.get_fen())
        print(game)
        print(f"Unable to find origin position for move: {san}")
        return False

    is_en_passant = False
    if piece_type == PieceType.PAWN and chosen_move.from_pos.x != destination_position.x:
        if game.board.get_piece(destination_position) is None:
            is_en_passant = True

    captured_piece = game.board.get_piece(destination_position)

    # 5. Apply the completed move
    final_move = Move(
        from_pos=chosen_move.from_pos,
        to_pos=destination_position,
        piece=piece,
        captured=captured_piece,
        promotion=promoted_piece,
        is_castle=is_castling,
        is_en_passant=is_en_passant
    )
    game._apply_move(final_move)
    return True
