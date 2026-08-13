use core::fmt;
use std::{fmt::Display, vec};

use crate::{
    castling::CastlingRights,
    enums::{
        Color::{self, BLACK, WHITE},
        PieceType::{self, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK},
        Square, get_squares_from_bitboard,
    },
    moves::Move,
    piece::Piece,
};

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pawns_white: u64,
    pawns_black: u64,
    knights_white: u64,
    knights_black: u64,
    bishops_white: u64,
    bishops_black: u64,
    rooks_white: u64,
    rooks_black: u64,
    queens_white: u64,
    queens_black: u64,
    king_white: u64,
    king_black: u64,

    en_passant: u64,

    pub white_kingside_castling: u64,
    pub white_queenside_castling: u64,
    pub black_kingside_castling: u64,
    pub black_queenside_castling: u64,

    pub rank_8: u64,
    pub rank_7: u64,
    pub rank_6: u64,
    pub rank_5: u64,
    pub rank_4: u64,
    pub rank_3: u64,
    pub rank_2: u64,
    pub rank_1: u64,

    pub file_a: u64,
    pub file_b: u64,
    pub file_c: u64,
    pub file_d: u64,
    pub file_e: u64,
    pub file_f: u64,
    pub file_g: u64,
    pub file_h: u64,
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FooWrite")
    }
}

impl Board {
    pub fn empty() -> Self {
        Self {
            pawns_white: 0,
            pawns_black: 0,
            bishops_white: 0,
            bishops_black: 0,
            knights_white: 0,
            knights_black: 0,
            rooks_white: 0,
            rooks_black: 0,
            queens_white: 0,
            queens_black: 0,
            king_white: 0,
            king_black: 0,
            en_passant: 0,

            white_kingside_castling:
                0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            white_queenside_castling:
                0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            black_kingside_castling:
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000,
            black_queenside_castling:
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01110000,

            rank_8: 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            rank_7: 0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000,
            rank_6: 0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000,
            rank_5: 0b00000000_00000000_00000000_11111111_00000000_00000000_00000000_00000000,
            rank_4: 0b00000000_00000000_00000000_00000000_11111111_00000000_00000000_00000000,
            rank_3: 0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000,
            rank_2: 0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000,
            rank_1: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111,

            file_a: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101,
            file_b: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101 << 1,
            file_c: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101 << 2,
            file_d: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101 << 3,
            file_e: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101 << 4,
            file_f: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101 << 5,
            file_g: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101 << 6,
            file_h: 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101 << 7,
        }
    }

    pub fn new() -> Self {
        Self::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"))
    }

    pub fn is_there_piece_at_square(&self, piece: &Piece, square: Square) -> bool {
        // Iterate over all bitboards from get_all_piece_bitboards
        let bitboard = self.get_piece_bitboard(piece);
        // Check if the specified square is set in this bitboard
        if (bitboard & (1u64 << square.to_index())) != 0 {
            return true;
        }
        return false;
    }

    pub fn get_piece_at_square(&self, square: Square) -> Option<Piece> {
        // Iterate over all bitboards from get_all_piece_bitboards
        for (piece_type_index, bitboard) in self.get_all_piece_bitboards().iter().enumerate() {
            // Check if the specified square is set in this bitboard
            if (bitboard & (1u64 << square.to_index())) != 0 {
                // Determine the color of the piece based on its position in the vector
                let color = if piece_type_index % 2 == 0 {
                    WHITE
                } else {
                    BLACK
                };

                // Determine the type of the piece based on its position in the vector
                let piece_type = match piece_type_index / 2 {
                    0 => PAWN,
                    1 => KNIGHT,
                    2 => BISHOP,
                    3 => ROOK,
                    4 => QUEEN,
                    5 => KING,
                    _ => unreachable!(), // Should never happen
                };

                return Some(Piece { color, piece_type });
            }
        }
        None // No piece at this square
    }

    pub fn get_piece_at_board_index(&self, index: u8) -> Option<Piece> {
        // Iterate over all bitboards from get_all_piece_bitboards
        for (piece_type_index, bitboard) in self.get_all_piece_bitboards().iter().enumerate() {
            // Check if the specified square is set in this bitboard
            if (bitboard & (1u64 << index)) != 0 {
                // Determine the color of the piece based on its position in the vector
                let color = if piece_type_index % 2 == 0 {
                    WHITE
                } else {
                    BLACK
                };

                // Determine the type of the piece based on its position in the vector
                let piece_type = match piece_type_index / 2 {
                    0 => PAWN,
                    1 => KNIGHT,
                    2 => BISHOP,
                    3 => ROOK,
                    4 => QUEEN,
                    5 => KING,
                    _ => unreachable!(), // Should never happen
                };

                return Some(Piece { color, piece_type });
            }
        }
        None // No piece at this square
    }

    pub fn is_occupied(&self, square: Square) -> bool {
        let piece = self.get_piece_at_square(square);

        return piece.is_none();
    }

    pub fn is_occupied_by_color(&self, square: Square, color: Color) -> bool {
        let piece = self.get_piece_at_square(square);

        if piece.is_none() {
            return false;
        }

        let square_piece_color = piece.unwrap().color;

        return square_piece_color == color;
    }

    pub fn find_king(&self, color: Color) -> Square {
        let king_bitboard = match color {
            WHITE => self.get_white_king(),
            BLACK => self.get_black_king(),
        };

        get_squares_from_bitboard(&king_bitboard)
            .first()
            .copied()
            .expect("Exactly one king per side")
    }

    pub fn set_piece_at_square(&mut self, square: Square, piece: Piece) {
        let bitboard = self.get_mutable_piece_bitboard(&piece);
        let index = square.to_index();
        *bitboard |= 1 << index;
    }

    pub fn remove_piece_at_square(&mut self, square: Square, piece: Piece) {
        let bitboard = self.get_mutable_piece_bitboard(&piece);
        let index = square.to_index();
        *bitboard |= 0 << index;
    }

    pub fn move_piece(&mut self, from_pos: Square, to_pos: Square) {
        let from_piece = self.get_piece_at_square(from_pos);

        if from_piece.is_none() {
            panic!("No piece was found at from square.")
        }

        self.remove_piece_at_square(from_pos, from_piece.clone().unwrap());

        self.set_piece_at_square(to_pos, from_piece.unwrap());
    }

    pub fn get_piece_bitboard(&self, piece: &Piece) -> u64 {
        match piece {
            Piece {
                piece_type: PAWN,
                color: WHITE,
            } => self.get_white_pawns(),
            Piece {
                piece_type: PAWN,
                color: BLACK,
            } => self.get_black_pawns(),
            Piece {
                piece_type: BISHOP,
                color: WHITE,
            } => self.get_white_bishops(),
            Piece {
                piece_type: BISHOP,
                color: BLACK,
            } => self.get_black_bishops(),
            Piece {
                piece_type: KNIGHT,
                color: WHITE,
            } => self.get_white_knights(),
            Piece {
                piece_type: KNIGHT,
                color: BLACK,
            } => self.get_black_knights(),
            Piece {
                piece_type: ROOK,
                color: WHITE,
            } => self.get_white_rooks(),
            Piece {
                piece_type: ROOK,
                color: BLACK,
            } => self.get_black_rooks(),
            Piece {
                piece_type: QUEEN,
                color: WHITE,
            } => self.get_white_queens(),
            Piece {
                piece_type: QUEEN,
                color: BLACK,
            } => self.get_black_queens(),
            Piece {
                piece_type: KING,
                color: WHITE,
            } => self.get_white_king(),
            Piece {
                piece_type: KING,
                color: BLACK,
            } => self.get_black_king(),
        }
    }

    pub fn get_mutable_piece_bitboard(&mut self, piece: &Piece) -> &mut u64 {
        match piece {
            Piece {
                piece_type: PAWN,
                color: WHITE,
            } => self.get_mutable_white_pawns(),
            Piece {
                piece_type: PAWN,
                color: BLACK,
            } => self.get_mutable_black_pawns(),
            Piece {
                piece_type: BISHOP,
                color: WHITE,
            } => self.get_mutable_white_bishops(),
            Piece {
                piece_type: BISHOP,
                color: BLACK,
            } => self.get_mutable_black_bishops(),
            Piece {
                piece_type: KNIGHT,
                color: WHITE,
            } => self.get_mutable_white_knights(),
            Piece {
                piece_type: KNIGHT,
                color: BLACK,
            } => self.get_mutable_black_knights(),
            Piece {
                piece_type: ROOK,
                color: WHITE,
            } => self.get_mutable_white_rooks(),
            Piece {
                piece_type: ROOK,
                color: BLACK,
            } => self.get_mutable_black_rooks(),
            Piece {
                piece_type: QUEEN,
                color: WHITE,
            } => self.get_mutable_white_queens(),
            Piece {
                piece_type: QUEEN,
                color: BLACK,
            } => self.get_mutable_black_queens(),
            Piece {
                piece_type: KING,
                color: WHITE,
            } => self.get_mutable_white_king(),
            Piece {
                piece_type: KING,
                color: BLACK,
            } => self.get_mutable_black_king(),
        }
    }

    fn get_all_piece_bitboards(&self) -> Vec<u64> {
        vec![
            self.pawns_white,
            self.pawns_black,
            self.knights_white,
            self.knights_black,
            self.bishops_white,
            self.bishops_black,
            self.rooks_white,
            self.rooks_black,
            self.queens_white,
            self.queens_black,
            self.king_white,
            self.king_black,
        ]
    }
    pub fn get_whites(&self) -> u64 {
        self.pawns_white
            | self.knights_white
            | self.bishops_white
            | self.rooks_white
            | self.queens_white
            | self.king_white
    }

    pub fn get_blacks(&self) -> u64 {
        self.pawns_black
            | self.knights_black
            | self.bishops_black
            | self.rooks_black
            | self.queens_black
            | self.king_black
    }

    pub fn get_empty(&self) -> u64 {
        !(self.get_whites() | self.get_blacks())
    }

    pub fn get_all(&self) -> u64 {
        self.get_whites() | self.get_blacks()
    }

    pub fn get_en_passant_bitboard(&self) -> u64 {
        self.en_passant
    }

    pub fn get_white_pawns(&self) -> u64 {
        self.pawns_white
    }

    pub fn get_black_pawns(&self) -> u64 {
        self.pawns_black
    }

    pub fn get_white_knights(&self) -> u64 {
        self.knights_white
    }

    pub fn get_black_knights(&self) -> u64 {
        self.knights_black
    }

    pub fn get_white_bishops(&self) -> u64 {
        self.bishops_white
    }

    pub fn get_black_bishops(&self) -> u64 {
        self.bishops_black
    }

    pub fn get_white_rooks(&self) -> u64 {
        self.rooks_white
    }

    pub fn get_black_rooks(&self) -> u64 {
        self.rooks_black
    }

    pub fn get_white_queens(&self) -> u64 {
        self.queens_white
    }

    pub fn get_black_queens(&self) -> u64 {
        self.queens_black
    }

    pub fn get_white_king(&self) -> u64 {
        self.king_white
    }

    pub fn get_black_king(&self) -> u64 {
        self.king_black
    }

    pub fn get_mutable_white_pawns(&mut self) -> &mut u64 {
        &mut self.pawns_white
    }

    pub fn get_mutable_black_pawns(&mut self) -> &mut u64 {
        &mut self.pawns_black
    }

    pub fn get_mutable_white_knights(&mut self) -> &mut u64 {
        &mut self.knights_white
    }

    pub fn get_mutable_black_knights(&mut self) -> &mut u64 {
        &mut self.knights_black
    }

    pub fn get_mutable_white_bishops(&mut self) -> &mut u64 {
        &mut self.bishops_white
    }

    pub fn get_mutable_black_bishops(&mut self) -> &mut u64 {
        &mut self.bishops_black
    }

    pub fn get_mutable_white_rooks(&mut self) -> &mut u64 {
        &mut self.rooks_white
    }

    pub fn get_mutable_black_rooks(&mut self) -> &mut u64 {
        &mut self.rooks_black
    }

    pub fn get_mutable_white_queens(&mut self) -> &mut u64 {
        &mut self.queens_white
    }

    pub fn get_mutable_black_queens(&mut self) -> &mut u64 {
        &mut self.queens_black
    }

    pub fn get_mutable_white_king(&mut self) -> &mut u64 {
        &mut self.king_white
    }

    pub fn get_mutable_black_king(&mut self) -> &mut u64 {
        &mut self.king_black
    }

    pub fn get_mutable_en_passant(&mut self) -> &mut u64 {
        &mut self.en_passant
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in 0..8 {
            let mut empty = 0;
            for file in 0..8 {
                let index = rank * 8 + file;
                let piece = self.get_piece_at_board_index(index);

                match piece {
                    Some(p) => {
                        if empty > 0 {
                            fen += &empty.to_string();
                            empty = 0;
                        }
                        fen += &p.to_char().to_string();
                    }
                    None => empty += 1,
                }
            }
            fen += "/"
        }

        fen
    }

    // pub fn from_fen(fen: String) -> Board {
    //     let mut board = Board::empty();
    //
    //     let mut ranks = fen.split("/");
    //
    //     for rank_index in 0..8 {
    //         let mut empty = 0;
    //         let rank = ranks.next().expect("Should be 8 parts of position fen");
    //
    //         for letter in rank.chars() {
    //             if letter.is_numeric() {
    //                 empty += letter.to_digit(10).unwrap();
    //                 continue;
    //             }
    //
    //             let piece = Piece::from_char(letter);
    //
    //             let square_index = rank_index * 8 + empty as u8;
    //             let square = Square::square_from_number(square_index);
    //
    //             board.set_piece_at_square(square, piece);
    //         }
    //     }
    //
    //     board
    // }

    pub fn from_fen(fen: String) -> Board {
        let mut board = Board::empty();

        // Only parse the position component of the FEN string
        let position_part = fen.split_whitespace().next().unwrap_or(&fen);
        let mut ranks = position_part.split('/');

        for rank_index in 0..8 {
            let rank = ranks.next().expect("Should be 8 parts of position fen");
            let mut file_index = 0u8;

            for letter in rank.chars() {
                if let Some(digit) = letter.to_digit(10) {
                    file_index += digit as u8;
                } else {
                    let piece = Piece::from_char(letter);
                    let square_index = rank_index * 8 + file_index;
                    let square = Square::square_from_number(square_index);

                    board.set_piece_at_square(square, piece);
                    file_index += 1; // Increment file for every placed piece!
                }
            }
        }

        board
    }

    pub fn is_attacked(
        &self,
        square: Square,
        by_color: Color,
        castling_rights: &CastlingRights,
    ) -> bool {
        for piece_type in PieceType::iter() {
            if piece_type == KING {
                continue;
            }

            let piece = Piece::new(piece_type, by_color);
            let moves = piece.get_pseudo_legal_moves(self, castling_rights);

            let attacking_moves: Vec<Move> = moves
                .into_iter()
                .filter(|m| m.get_to_pos() == square)
                .collect();

            if attacking_moves.len() > 0 {
                return true;
            }
        }

        false
    }
}
