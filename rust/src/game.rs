use std::vec;

use crate::{
    board::Board,
    castling::CastlingRights,
    enums::{
        Color::{self, BLACK, WHITE},
        PieceType::{self, KING, PAWN, ROOK},
        Square, get_bitboard_from_square,
    },
    moves::Move,
    piece::Piece,
};

pub struct Game {
    pub board: Board,
    turn: Color,
    castling: CastlingRights,
    half_move_clock: u8,
    fullmove_clock: u8,
    moves: vec::Vec<Move>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            turn: WHITE,
            castling: CastlingRights::new(),
            half_move_clock: 0,
            fullmove_clock: 1,
            moves: Vec::new(),
        }
    }

    // to do
    pub fn from_fen(fen: String) -> Self {
        let mut game = Self::new();

        let fen_string: Vec<&str> = fen.split(" ").collect();

        game.board = Board::from_fen(
            fen_string
                .get(0)
                .expect("FEN should have board info")
                .to_string(),
        );

        let color_char = fen_string.get(1).expect("FEN should have turn info");
        game.turn = Color::from_char(color_char);

        let castling_string = fen_string
            .get(2)
            .expect("FEN should have castling rights info");
        game.castling = CastlingRights::from_string(castling_string.to_string());

        // to do
        // let en_passant = fen_string
        //     .get(4)
        //     .expect("FEN should have info about en passant");
        // if en_passant != &"-" {
        //     let en_passant_square = Square::from_uci(en_passant);
        // }

        game.half_move_clock = fen_string
            .get(4)
            .expect("FEN should have info about half move clock")
            .to_string()
            .parse()
            .expect("Half move clock in FEN should be a number");
        game.fullmove_clock = fen_string
            .get(5)
            .expect("FEN should have info about fullmove clock")
            .to_string()
            .parse()
            .expect("Full move clock in FEN should be a number");

        game
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        self._generate_pseudo_legal_moves()
            .into_iter()
            .filter(|m| self._is_legal_move(m))
            .collect()
    }

    pub fn get_legal_moves_uci(&self) -> Vec<String> {
        self.get_legal_moves()
            .into_iter()
            .map(|m| m.to_uci())
            .collect()
    }

    pub fn _generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut pseudo_legal_moves = Vec::new();

        for piece_type in PieceType::iter() {
            let piece = Piece::new(piece_type, self.turn);
            let moves = piece.get_pseudo_legal_moves(&self.board);
            for move_ in &moves {
                println!("Move: {}", move_.to_uci());
            }
            pseudo_legal_moves.extend(moves);
        }

        pseudo_legal_moves
    }

    // to do
    fn _is_legal_move(&self, move_: &Move) -> bool {
        if !move_.from_pos.is_valid() || !move_.to_pos.is_valid() {
            return false;
        }

        // Create a copy of the board
        let mut board_copy = self.board.clone();

        // Make the move on the copy
        let piece = board_copy.get_piece_at_square(move_.from_pos);
        if piece.is_none() {
            return false;
        }

        // Special handling for castling
        if move_.is_castle {
            // Move king
            board_copy.move_piece(move_.from_pos, move_.to_pos);

            // Move rook
            if move_.to_pos.get_col_index() > move_.from_pos.get_col_index() {
                board_copy.move_piece(
                    Square::new(7, move_.from_pos.get_row_index()),
                    Square::new(5, move_.from_pos.get_row_index()),
                );
            } else {
                board_copy.move_piece(
                    Square::new(0, move_.from_pos.get_row_index()),
                    Square::new(3, move_.from_pos.get_row_index()),
                );
            }
        } else {
            // Move piece
            board_copy.move_piece(move_.from_pos, move_.to_pos);

            // Handle en passant capture
            if move_.is_en_passant {
                let captured_pos =
                    Square::new(move_.to_pos.get_col_index(), move_.from_pos.get_row_index());
                board_copy.remove_piece_at_square(captured_pos, Piece::new(PAWN, self.turn));
                // board_copy.set_piece(captured_pos, None)
            }
        }

        // Check if our king is in check after this move
        let king_pos = board_copy.find_king(self.turn);

        return !board_copy.is_attacked(king_pos, self.turn.opposite());
    }

    pub fn _apply_move(&mut self, move_: Move) {
        // Update halfmove clock
        if move_.get_piece().piece_type == PAWN || move_.captured.is_some() {
            self.half_move_clock = 0
        } else {
            self.half_move_clock += 1
        }

        // Update en passant
        if move_.get_piece().piece_type == PAWN
            && (move_
                .to_pos
                .get_row_index()
                .abs_diff(move_.from_pos.get_row_index())
                == 2)
        {
            let en_passant_square = Square::new(
                move_.from_pos.get_col_index(),
                (move_.from_pos.get_row_index() + move_.to_pos.get_row_index()) / 2,
            );
            *self.board.get_mutable_en_passant() = get_bitboard_from_square(en_passant_square);
        } else {
            *self.board.get_mutable_en_passant() = 0;
        }

        // Handle castling
        if move_.is_castle {
            // Move king
            self.board.move_piece(move_.from_pos, move_.to_pos);

            // Move rook
            // Kingside
            if move_.to_pos.get_col_index() > move_.from_pos.get_col_index() {
                self.board.move_piece(
                    Square::new(7, move_.from_pos.get_row_index()),
                    Square::new(5, move_.from_pos.get_row_index()),
                );
                // self.board.move_piece(Position(7, move.from_pos.y), Position(5, move.from_pos.y))
            } else {
                self.board.move_piece(
                    Square::new(0, move_.from_pos.get_row_index()),
                    Square::new(3, move_.from_pos.get_row_index()),
                );
                // self.board.move_piece(Position(0, move.from_pos.y), Position(3, move.from_pos.y))
            }

            // Update castling rights
            if move_.piece.color == WHITE {
                self.castling.white_kingside = false;
                self.castling.white_queenside = false;
            } else {
                self.castling.black_kingside = false;
                self.castling.black_queenside = false;
            }
        } else {
            // Handle en passant capture
            if move_.is_en_passant {
                let captured_pos =
                    Square::new(move_.to_pos.get_col_index(), move_.from_pos.get_row_index());
                // captured_pos = Position(move.to_pos.x, move.from_pos.y)

                // self.board.set_piece(captured_pos, None);
                self.board.remove_piece_at_square(
                    captured_pos,
                    Piece::new(PAWN, move_.piece.color.opposite()),
                );
            }

            // Move piece
            self.board.move_piece(move_.from_pos, move_.to_pos);

            // Handle promotion
            if move_.promotion.is_some() {
                self.board.set_piece_at_square(
                    move_.to_pos,
                    Piece::new(move_.promotion.expect("Promotion is some"), self.turn),
                );
                // self.board.set_piece(move.to_pos, Piece(move.promotion, self.turn))
            }

            // Update castling rights
            if move_.piece.piece_type == KING {
                if self.turn == WHITE {
                    self.castling.white_kingside = false;
                    self.castling.white_queenside = false;
                } else {
                    self.castling.black_kingside = false;
                    self.castling.black_queenside = false;
                }
            } else if move_.piece.piece_type == ROOK {
                // Queenside rook
                if move_.from_pos.get_col_index() == 0 {
                    if self.turn == WHITE && move_.from_pos.get_row_index() == 0 {
                        self.castling.white_queenside = false;
                    } else if self.turn == BLACK && move_.from_pos.get_row_index() == 7 {
                        self.castling.black_queenside = false;
                    }
                } else if move_.from_pos.get_col_index() == 7 {
                    // Kingside rook
                    if self.turn == WHITE && move_.from_pos.get_row_index() == 0 {
                        self.castling.white_kingside = false;
                    } else if self.turn == BLACK && move_.from_pos.get_row_index() == 7 {
                        self.castling.black_kingside = false;
                    }
                }
            }
        }

        // Switch turn
        self.turn = self.turn.opposite();

        // Update fullmove number
        if self.turn == WHITE {
            self.fullmove_clock += 1
        }
    }

    pub fn is_check(&self) -> bool {
        let king_pos = self.board.find_king(self.turn);

        return self.board.is_attacked(king_pos, self.turn.opposite());
    }

    pub fn is_checkmate(&self) -> bool {
        if !self.is_check() {
            return false;
        }
        return self.get_legal_moves().len() == 0;
    }

    pub fn is_stalemate(&self) -> bool {
        if self.is_check() {
            return false;
        }
        return self.get_legal_moves().len() == 0;
    }

    pub fn is_draw(&self) -> bool {
        self.half_move_clock >= 100 || self.is_stalemate()
    }

    // pub fn get_fen(&self) -> String {}

    // pub fn get_pgn(&self) -> String {}

    // pub fn get_nnue_encoding(&self) {}
}
