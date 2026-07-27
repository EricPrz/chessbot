use crate::enums::Color;
use crate::enums::Color::BLACK;
use crate::enums::Color::WHITE;
use crate::enums::PieceType;
use crate::moves;
use crate::piece;

use super::board;
use super::enums;
use super::position;

use std::vec;

#[derive(Clone, Copy)]
pub struct Piece {
    pub piece_type: enums::PieceType,
    pub color: enums::Color,
}

impl Piece {
    fn new(piece_type: enums::PieceType, color: enums::Color) -> Piece {
        Piece {
            piece_type: piece_type,
            color: color,
        }
    }

    fn get_pseudo_legal_moves(&self, board: &board::Board) -> vec::Vec<moves::Move> {
        match self.piece_type {
            enums::PieceType::PAWN => self.get_pawn_moves(board),
            enums::PieceType::KNIGHT => self.get_knight_moves(pos, board),
            enums::PieceType::BISHOP => self.get_sliding_moves(pos, board),
            enums::PieceType::ROOK => self.get_sliding_moves(pos, board),
            enums::PieceType::QUEEN => self.get_sliding_moves(pos, board),
            enums::PieceType::KING => self.get_king_moves(pos, board),
        }
    }

    fn get_pawn_moves(&self, board: &board::Board) -> Vec<moves::Move> {
        let color = self.color;

        let pawns = match color {
            WHITE => board.get_white_pawns(),
            BLACK => board.get_black_pawns(),
        };
        let enemy_pieces = match color {
            WHITE => board.get_blacks(),
            BLACK => board.get_whites(),
        };

        let mut moves: Vec<moves::Move> = vec::Vec::new();

        // One forward
        let mut one_forward = match color {
            WHITE => pawns << 8,
            BLACK => pawns >> 8,
        };
        one_forward = one_forward & board.get_empty();
        while one_forward != 0 {
            // Isolate the MSB
            let msb = one_forward & one_forward.wrapping_neg();

            // Find the square index (0-63)
            let to_square_num = 63 - msb.leading_zeros() as u8;
            let from_square_num = match color {
                WHITE => to_square_num + 8 as u8,
                BLACK => to_square_num - 8 as u8,
            };

            let to_square = enums::Square::square_from_number(to_square_num);
            let from_square = enums::Square::square_from_number(from_square_num);

            let piece = piece::Piece::new(enums::PieceType::PAWN, color);

            let promotion_row: u8 = match color {
                WHITE => 8,
                BLACK => 1,
            };

            if to_square.get_row_index() == promotion_row {
                // Handle promotions (excluding KING)
                for piece_type in PieceType::iter().filter(|&p| p != PieceType::KING) {
                    let _move = moves::Move::new(
                        from_square,
                        to_square,
                        piece,
                        None,             // No captured piece
                        Some(piece_type), // Promotion piece
                        true,             // is_promotion
                        false,            // is_castling
                    );
                    moves.push(_move);
                }
            } else {
                // Regular move
                let _move = moves::Move::new(
                    from_square,
                    to_square,
                    piece,
                    None,  // No captured piece
                    None,  // No promotion
                    false, // is_promotion
                    false, // is_castling
                );
                moves.push(_move);
            }

            // Remove the processed knight
            one_forward ^= msb;
        }

        // Two forward
        let mut two_forward = match color {
            WHITE => pawns & board.rank_2 << 8 & board.get_empty() << 8 & board.get_empty(),
            BLACK => pawns & board.rank_6 >> 8 & board.get_empty() >> 8 & board.get_empty(),
        };
        while two_forward != 0 {
            // Isolate the MSB
            let msb = two_forward & two_forward.wrapping_neg();

            // Find the square index (0-63)
            let to_square_num = 63 - msb.leading_zeros() as u8;
            let from_square_num = match color {
                WHITE => to_square_num + 16 as u8,
                BLACK => to_square_num - 16 as u8,
            };

            let to_square = enums::Square::square_from_number(to_square_num);
            let from_square = enums::Square::square_from_number(from_square_num);

            let piece = piece::Piece::new(enums::PieceType::PAWN, color);

            let _move = moves::Move::new(
                from_square,
                to_square,
                piece,
                None,  // No captured piece
                None,  // No promotion
                false, // is_promotion
                true,  // is_castling
            );
            moves.push(_move);

            two_forward ^= msb;
        }

        // Capture
        let pawn_capture_moves = PawnCaptureMoves::new(color);
        let mut pawns = pawns;

        while pawns != 0 {
            // Isolate the MSB
            let msb = pawns & pawns.wrapping_neg();

            // Find the square index (0-63)
            let from_square_num = 63 - msb.leading_zeros() as usize;
            let from_square = enums::Square::square_from_number(from_square_num as u8);

            // Get precomputed moves for this square
            let mut pawn_captures_from_square =
                pawn_capture_moves.moves[from_square_num] & enemy_pieces;

            let piece = piece::Piece::new(enums::PieceType::PAWN, color.clone());

            while pawn_captures_from_square != 0 {
                let msb2 = pawn_captures_from_square & pawn_captures_from_square.wrapping_neg();
                let to_square_num = 63 - msb2.leading_zeros() as usize;
                let to_square = enums::Square::square_from_number(to_square_num as u8);

                let captured = board.get_piece_at_square(to_square);

                let promotion_row: u8 = match color {
                    WHITE => 8,
                    BLACK => 1,
                };

                if to_square.get_row_index() == promotion_row {
                    // Handle promotions (excluding KING)
                    for piece_type in PieceType::iter().filter(|&p| p != PieceType::KING) {
                        let _move = moves::Move::new(
                            from_square,
                            to_square,
                            piece,
                            captured,         // No captured piece
                            Some(piece_type), // Promotion piece
                            true,             // is_promotion
                            false,            // is_castling
                        );
                        moves.push(_move);
                    }
                } else {
                    // Regular move
                    let _move = moves::Move::new(
                        from_square,
                        to_square,
                        piece.clone(),
                        captured, // No captured piece
                        None,     // No promotion
                        false,    // is_promotion
                        false,    // is_castling
                    );
                    moves.push(_move);
                }

                pawn_captures_from_square ^= msb2;
            }

            // Remove the processed knight
            pawns ^= msb;
        }

        // En Passant
        if board.get_en_passant_bitboard() != 0 {
            // let en_passant_from = if color == WHITE {
            //     en_passant_square >> 8
            // } else {
            //     en_passant_square << 8
            // };
            //
            // // Check if the pawn is adjacent to the en_passant square
            // let can_capture_en_passant = match color {
            //     WHITE => (pawns & (en_passant_from << 1 | en_passant_from >> 1)) != 0,
            //     BLACK => (pawns & (en_passant_from << 1 | en_passant_from >> 1)) != 0,
            // };
            //
            // if can_capture_en_passant {
            //     moves.push(moves::Move::new(
            //         pos,
            //         en_passant_square.trailing_zeros() as usize, // Convert bitboard to square index
            //         moves::MoveType::EnPassant,
            //     ));
            // }
            panic!("We should compute en passant captures.")
        }

        moves
    }

    fn get_knight_moves(&self, board: &board::Board) -> Vec<moves::Move> {
        let knight_moves = KnightMoves::new();

        let mut moves: Vec<moves::Move> = vec::Vec::new();

        let color = &self.color;
        let enemy_bitboard = match color {
            enums::Color::WHITE => board.get_blacks(),
            enums::Color::BLACK => board.get_whites(),
        };
        let mut knights = match color {
            enums::Color::WHITE => board.get_white_knights(),
            enums::Color::BLACK => board.get_black_knights(),
        };

        while knights != 0 {
            // Isolate the MSB
            let msb = knights & knights.wrapping_neg();

            // Find the square index (0-63)
            let from_square_num = 63 - msb.leading_zeros() as usize;
            let from_square = enums::Square::square_from_number(from_square_num as u8);

            // Get precomputed moves for this square
            let mut knight_moves_from_square =
                knight_moves.moves[from_square_num] & (enemy_bitboard | board.get_empty());

            let piece = piece::Piece::new(enums::PieceType::KNIGHT, color.clone());

            while knight_moves_from_square != 0 {
                let msb2 = knight_moves_from_square & knight_moves_from_square.wrapping_neg();
                let to_square_num = 63 - msb2.leading_zeros() as usize;
                let to_square = enums::Square::square_from_number(to_square_num as u8);

                let captured = board.get_piece_at_square(to_square);

                let _move = moves::Move::new(
                    from_square.clone(),
                    to_square,
                    piece.clone(),
                    captured,
                    None,
                    false,
                    false,
                );
                moves.push(_move);

                knight_moves_from_square ^= msb2;
            }

            // Remove the processed knight
            knights ^= msb;
        }

        moves
    }

    fn get_king_moves(&self, board: &board::Board) -> u64 {}
}

pub struct KnightMoves {
    pub moves: [u64; 64],
}

impl KnightMoves {
    pub const fn new() -> Self {
        let mut moves = [0u64; 64];
        let mut square = 0;

        while square < 64 {
            moves[square] = Self::calculate_knight_moves(square);
            square += 1;
        }

        Self { moves }
    }

    /// Calculate all knight moves from a given square (0-63).
    const fn calculate_knight_moves(square: usize) -> u64 {
        let mut result = 0u64;
        let row = square / 8;
        let col = square % 8;

        // All 8 possible knight moves (L-shaped: 2 squares in one direction, 1 in the other)
        let knight_moves = [
            (row + 2, col + 1),
            (row + 2, col - 1),
            (row - 2, col + 1),
            (row - 2, col - 1),
            (row + 1, col + 2),
            (row + 1, col - 2),
            (row - 1, col + 2),
            (row - 1, col - 2),
        ];

        let mut i = 0;
        while i < 8 {
            let (new_row, new_col) = knight_moves[i];
            if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                let new_square = (new_row * 8 + new_col) as usize;
                result |= 1u64 << new_square;
            }
            i += 1;
        }

        result
    }
}

pub struct PawnCaptureMoves {
    pub moves: [u64; 64],
}

impl PawnCaptureMoves {
    pub const fn new(color: Color) -> Self {
        let mut moves = [0u64; 64];
        let mut square = 0;

        while square < 64 {
            moves[square] = Self::calculate_pawn_capture_moves(square, color);
            square += 1;
        }

        Self { moves }
    }

    const fn calculate_pawn_capture_moves(square: usize, color: Color) -> u64 {
        let mut result = 0u64;
        let row = square / 8;
        let col = square % 8;

        let pawn_capture_moves = match color {
            WHITE => [(row - 1, col + 1), (row - 1, col - 1)],
            BLACK => [(row + 1, col + 1), (row + 1, col - 1)],
        };

        let mut i = 0;
        while i < 2 {
            let (new_row, new_col) = pawn_capture_moves[i];
            if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                let new_square = (new_row * 8 + new_col) as usize;
                result |= 1u64 << new_square;
            }
            i += 1;
        }

        result
    }
}
