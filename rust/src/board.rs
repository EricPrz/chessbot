use std::vec;

use crate::{enums, piece::Piece};

#[derive(Debug)]
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

    pub rank_8: u64 = 0bFF00000000000000u64,
    pub rank_7: u64 = 0b00FF000000000000u64,
    pub rank_6: u64 = 0b0000FF0000000000u64,
    pub rank_5: u64 = 0b000000FF00000000u64,
    pub rank_4: u64 = 0bFF000000u64,
    pub rank_3: u64 = 0b00FF0000u64,
    pub rank_2: u64 = 0b0000FF00u64,
    pub rank_1: u64 = 0b000000FFu64,

    pub file_a: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64,
    pub file_b: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64 << 1,
    pub file_c: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64 << 2,
    pub file_d: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64 << 3,
    pub file_e: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64 << 4,
    pub file_f: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64 << 5,
    pub file_g: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64 << 6,
    pub file_h: u64 = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101u64 << 7,
}

impl Board {
    pub fn get_piece_at_square(&self, square: enums::Square) -> Option<Piece> {
        // Iterate over all bitboards from get_all_piece_bitboards
        for (piece_type_index, bitboard) in self.get_all_piece_bitboards().iter().enumerate() {
            // Check if the specified square is set in this bitboard
            if (bitboard & (1u64 << square.to_index())) != 0 {
                // Determine the color of the piece based on its position in the vector
                let color = if piece_type_index % 2 == 0 {
                    enums::Color::WHITE
                } else {
                    enums::Color::BLACK
                };

                // Determine the type of the piece based on its position in the vector
                let piece_type = match piece_type_index / 2 {
                    0 => enums::PieceType::PAWN,
                    1 => enums::PieceType::KNIGHT,
                    2 => enums::PieceType::BISHOP,
                    3 => enums::PieceType::ROOK,
                    4 => enums::PieceType::QUEEN,
                    5 => enums::PieceType::KING,
                    _ => unreachable!(), // Should never happen
                };

                return Some(Piece {
                    color,
                    piece_type,
                });
            }
        }
        None // No piece at this square
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
        self.pawns_white | self.knights_white | self.bishops_white | self.rooks_white | self.queens_white | self.king_white
    }

    pub fn get_blacks(&self) -> u64 {
        self.pawns_black | self.knights_black | self.bishops_black | self.rooks_black | self.queens_black | self.king_black
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

    }
