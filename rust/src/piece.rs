use crate::board::Boardd;
use crate::castling::CastlingRights;
use crate::enums::Colorr;
use crate::enums::Colorr::BLACK;
use crate::enums::Colorr::WHITE;
use crate::enums::PieceType;
use crate::enums::PieceType::BISHOP;
use crate::enums::PieceType::KING;
use crate::enums::PieceType::KNIGHT;
use crate::enums::PieceType::PAWN;
use crate::enums::PieceType::QUEEN;
use crate::enums::PieceType::ROOK;
use crate::enums::Square;
use crate::enums::get_squares_from_bitboard;
use crate::moves;
use crate::moves::Move;
use crate::piece;

use super::board;
use super::enums;

use nnue_rs::Color;
use nnue_rs::{Piece, PieceKind};

use std::vec;

#[derive(Clone, Copy, Debug)]
pub struct Piecee {
    pub piece_type: enums::PieceType,
    pub color: enums::Colorr,
}

impl Piecee {
    pub fn new(piece_type: enums::PieceType, color: enums::Colorr) -> Piecee {
        Piecee {
            piece_type: piece_type,
            color: color,
        }
    }

    pub fn from_char(c: char) -> Piecee {
        let color = if c.is_ascii_uppercase() {
            enums::Colorr::WHITE
        } else {
            enums::Colorr::BLACK
        };

        let piece_type = PieceType::from_char(c.to_ascii_lowercase());
        Piecee::new(piece_type, color)
    }

    pub fn get_pseudo_legal_moves(
        &self,
        board: &board::Boardd,
        castling_rights: &CastlingRights,
    ) -> vec::Vec<moves::Move> {
        match self.piece_type {
            enums::PieceType::PAWN => self.get_pawn_moves(board),
            enums::PieceType::KNIGHT => self.get_knight_moves(board),
            enums::PieceType::BISHOP => {
                self.get_sliding_moves(board, &[(1, 1), (1, -1), (-1, 1), (-1, -1)])
            }
            enums::PieceType::ROOK => {
                self.get_sliding_moves(board, &[(1, 0), (-1, 0), (0, 1), (0, -1)])
            }
            enums::PieceType::QUEEN => self.get_sliding_moves(
                board,
                &[
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ],
            ),
            enums::PieceType::KING => self.get_king_moves(board, castling_rights),
        }
    }

    pub fn get_pawn_moves(&self, board: &board::Boardd) -> Vec<moves::Move> {
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
            WHITE => pawns >> 8,
            BLACK => pawns << 8,
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

            let piece = piece::Piecee::new(enums::PieceType::PAWN, color);

            let promotion_row: u8 = match color {
                WHITE => 0,
                BLACK => 7,
            };

            if to_square.get_row_index() == promotion_row {
                // Handle promotions (excluding KING)
                for piece_type in PieceType::iter().filter(|&p| p != PieceType::KING && p != PAWN) {
                    let _move = moves::Move::new(
                        from_square,
                        to_square,
                        piece,
                        None,             // No captured piece
                        Some(piece_type), // Promotion piece
                        false,            // is_promotion
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
        let single_step = match color {
            WHITE => ((pawns & board.rank_2) >> 8) & board.get_empty(),
            BLACK => ((pawns & board.rank_7) << 8) & board.get_empty(),
        };

        let mut two_forward = match color {
            WHITE => (single_step >> 8) & board.get_empty(),
            BLACK => (single_step << 8) & board.get_empty(),
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

            let piece = piece::Piecee::new(enums::PieceType::PAWN, color);

            let _move = moves::Move::new(
                from_square,
                to_square,
                piece,
                None,  // No captured piece
                None,  // is promotion
                false, // is_castle
                true,  // is_en_passant
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

            let piece = piece::Piecee::new(enums::PieceType::PAWN, color.clone());

            while pawn_captures_from_square != 0 {
                let msb2 = pawn_captures_from_square & pawn_captures_from_square.wrapping_neg();
                let to_square_num = 63 - msb2.leading_zeros() as usize;
                let to_square = enums::Square::square_from_number(to_square_num as u8);

                let captured = board.get_piece_at_square(to_square);

                let promotion_row: u8 = match color {
                    WHITE => 0,
                    BLACK => 7,
                };

                if to_square.get_row_index() == promotion_row {
                    // Handle promotions (excluding KING)
                    for piece_type in
                        PieceType::iter().filter(|&p| p != PieceType::KING && p != PAWN)
                    {
                        let _move = moves::Move::new(
                            from_square,
                            to_square,
                            piece,
                            captured,         // No captured piece
                            Some(piece_type), // Promotion piece
                            false,            // is_promotion
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
        let en_passant_bitboard = board.get_en_passant_bitboard().clone();
        if en_passant_bitboard != 0 {
            let msb = en_passant_bitboard & en_passant_bitboard.wrapping_neg();
            let to_square_num = 63 - msb.leading_zeros() as usize;
            let to_pos = enums::Square::square_from_number(to_square_num as u8);

            let from_pos_num: usize = match color {
                WHITE => to_square_num + 8,
                BLACK => to_square_num - 8,
            };

            let piece = piece::Piecee::new(enums::PieceType::PAWN, color.clone());
            let captured = Piecee::new(PAWN, color.opposite());

            if board.is_there_piece_at_square(
                &piece,
                Square::square_from_number(from_pos_num as u8 + 1),
            ) {
                let from_pos = Square::square_from_number(from_pos_num as u8 + 1);
                let move_ = Move::new(from_pos, to_pos, piece, Some(captured), None, false, true);
                moves.push(move_);
            }

            if board.is_there_piece_at_square(
                &piece,
                Square::square_from_number(from_pos_num as u8 - 1),
            ) {
                let from_pos = Square::square_from_number(from_pos_num as u8 - 1);
                let move_ = Move::new(from_pos, to_pos, piece, Some(captured), None, false, true);
                moves.push(move_);
            }
        }

        moves
    }

    fn get_knight_moves(&self, board: &board::Boardd) -> Vec<moves::Move> {
        let knight_moves = KnightMoves::new();

        let mut moves: Vec<moves::Move> = vec::Vec::new();

        let color = &self.color;
        let enemy_bitboard = match color {
            enums::Colorr::WHITE => board.get_blacks(),
            enums::Colorr::BLACK => board.get_whites(),
        };
        let mut knights = match color {
            enums::Colorr::WHITE => board.get_white_knights(),
            enums::Colorr::BLACK => board.get_black_knights(),
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

            let piece = piece::Piecee::new(enums::PieceType::KNIGHT, color.clone());

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

    pub fn get_king_moves(
        &self,
        board: &board::Boardd,
        castling_rights: &CastlingRights,
    ) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let king_moves = KingMoves::new();

        let color = &self.color;
        let enemy_bitboard = match color {
            enums::Colorr::WHITE => board.get_blacks(),
            enums::Colorr::BLACK => board.get_whites(),
        };
        let king = match color {
            enums::Colorr::WHITE => board.get_white_king(),
            enums::Colorr::BLACK => board.get_black_king(),
        };

        let squares = get_squares_from_bitboard(&king);
        let from_square = squares.get(0).expect("There should be only one king.");

        let king_moves_bit = king_moves.moves[from_square.to_index() as usize];
        let mut king_moves = king_moves_bit & (enemy_bitboard | board.get_empty());

        let piece = piece::Piecee::new(enums::PieceType::KING, color.clone());

        while king_moves != 0 {
            let msb = king_moves & king_moves.wrapping_neg();
            let to_square_num = 63 - msb.leading_zeros() as usize;
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

            king_moves ^= msb;
        }

        // Castling
        match color {
            WHITE => {
                if board.white_kingside_castling & board.get_empty()
                    == board.white_kingside_castling
                    && board.find_king(WHITE) == Square::E1
                    && castling_rights.white_kingside
                {
                    let _move = moves::Move::new(
                        from_square.clone(),
                        Square::new(6, 7),
                        piece.clone(),
                        None,
                        None,
                        true,
                        false,
                    );
                    moves.push(_move);
                }

                if board.white_queenside_castling & board.get_empty()
                    == board.white_queenside_castling
                    && board.find_king(WHITE) == Square::E1
                    && castling_rights.white_queenside
                {
                    let _move = moves::Move::new(
                        from_square.clone(),
                        Square::new(2, 7),
                        piece.clone(),
                        None,
                        None,
                        true,
                        false,
                    );
                    moves.push(_move);
                }
            }
            BLACK => {
                if board.black_kingside_castling & board.get_empty()
                    == board.black_kingside_castling
                    && board.find_king(BLACK) == Square::E8
                    && castling_rights.black_kingside
                {
                    let _move = moves::Move::new(
                        from_square.clone(),
                        Square::new(6, 0),
                        piece.clone(),
                        None,
                        None,
                        true,
                        false,
                    );
                    moves.push(_move);
                }

                if board.black_queenside_castling & board.get_empty()
                    == board.black_queenside_castling
                    && board.find_king(BLACK) == Square::E8
                    && castling_rights.black_queenside
                {
                    let _move = moves::Move::new(
                        from_square.clone(),
                        Square::new(2, 0),
                        piece.clone(),
                        None,
                        None,
                        true,
                        false,
                    );
                    moves.push(_move);
                }
            }
        }

        moves
    }

    fn get_sliding_moves(&self, board: &Boardd, directions: &[(i32, i32)]) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let color = self.color;

        let bitboard = board.get_piece_bitboard(self);
        let numbers = enums::get_positions_from_bitboard(&bitboard);

        for number in numbers {
            let from_pos = Square::square_from_number(number);
            let start_rank = (number / 8) as i32;
            let start_file = (number % 8) as i32;

            for dir in directions {
                let dx = dir.0;
                let dy = dir.1;
                // println!("dir: {} {}", dir.0, dir.1);

                let mut current_rank = start_rank + dy;
                let mut current_file = start_file + dx;

                // Check boundaries explicitly to avoid board wrapping
                while current_rank >= 0 && current_rank < 8 && current_file >= 0 && current_file < 8
                {
                    let target_index = (current_rank * 8 + current_file) as u8;
                    let to_pos = Square::square_from_number(target_index);
                    let piece_at_target = board.get_piece_at_square(to_pos);
                    // println!("Start sqr: {} {}", start_file, start_rank);
                    // println!("Current sqr: {} {}", current_file, current_rank);
                    // println!(
                    //     "To sqr: {} {}",
                    //     to_pos.get_col_index(),
                    //     to_pos.get_row_index()
                    // );
                    // if piece_at_target.clone().is_some() {
                    //     println!("Piece at target: {}", piece_at_target.unwrap().to_char());
                    // } else {
                    //     println!("Piece at target: None");
                    // }
                    //
                    // println!("\n");

                    match piece_at_target {
                        None => {
                            // Empty square move (moving piece is self, target piece is None)
                            let _move = Move::new(
                                from_pos,
                                to_pos,
                                self.clone(), // Moving piece
                                None,         // No captured piece on an empty square
                                None,
                                false,
                                false,
                            );
                            moves.push(_move);
                        }
                        Some(captured_piece) => {
                            if captured_piece.color == color.opposite() {
                                // Enemy piece capture
                                let _move = Move::new(
                                    from_pos,
                                    to_pos,
                                    self.clone(),
                                    Some(captured_piece),
                                    None,
                                    false,
                                    false,
                                );
                                moves.push(_move);
                            }
                            // Blocked by any piece (friendly or enemy) -> stop sliding along this ray
                            break;
                        }
                    }

                    // Advance further along the ray direction
                    current_rank += dy;
                    current_file += dx;
                }
            }
        }

        // println!("Printing sliding moves...");
        // for move_ in &moves {
        //     println!("Sliding Move: {:?}", move_);
        // }

        moves
    }

    // fn get_sliding_moves(&self, board: &Boardd, directions: &[(i32, i32)]) -> Vec<Move> {
    //     let mut moves: Vec<Move> = vec::Vec::new();
    //     let color = self.color;
    //
    //     let bitboard = board.get_piece_bitboard(self);
    //     let numbers = enums::get_positions_from_bitboard(&bitboard);
    //
    //     for number in numbers {
    //         let from_pos = Square::square_from_number(number);
    //         for (dx, dy) in directions {
    //             let mut to_pos = Square::square_from_number((number as i32 - dy * 8 + dx) as u8);
    //
    //             while to_pos.is_valid() {
    //                 let piece_at_target = board.get_piece_at_square(to_pos);
    //
    //                 if piece_at_target.is_none() {
    //                     let _move = Move::new(
    //                         from_pos,
    //                         to_pos,
    //                         board
    //                             .get_piece_at_square(to_pos)
    //                             .expect("F at getting piece at a square it was supposed to be one"),
    //                         None,
    //                         None,
    //                         false,
    //                         false,
    //                     );
    //                     moves.push(_move);
    //
    //                     let temp_number = to_pos.to_index() as i32;
    //                     to_pos = Square::square_from_number((temp_number - dy * 8 + dx) as u8);
    //
    //                     continue;
    //                 }
    //
    //                 if piece_at_target
    //                     .expect("Previous if that contradicts none")
    //                     .color
    //                     == color.opposite()
    //                 {
    //                     let _move = Move::new(
    //                         from_pos,
    //                         to_pos,
    //                         board
    //                             .get_piece_at_square(to_pos)
    //                             .expect("F at getting piece at a square it was supposed to be one"),
    //                         piece_at_target,
    //                         None,
    //                         false,
    //                         false,
    //                     );
    //                     moves.push(_move);
    //
    //                     break;
    //                 }
    //
    //                 if piece_at_target
    //                     .expect("Previous if that contradicts none")
    //                     .color
    //                     == color
    //                 {
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    //     moves
    // }

    pub fn to_char(&self) -> char {
        let c = self.piece_type.to_char();
        match self.color {
            WHITE => c.to_ascii_uppercase(),
            BLACK => c.to_ascii_lowercase(),
        }
    }

    pub fn to_nnue_piece(&self) -> Piece {
        let color = match self.color {
            WHITE => Color::White,
            BLACK => Color::Black,
        };

        let piece_type = match self.piece_type {
            PAWN => PieceKind::Pawn,
            BISHOP => PieceKind::Bishop,
            KNIGHT => PieceKind::Knight,
            ROOK => PieceKind::Rook,
            QUEEN => PieceKind::Queen,
            KING => PieceKind::King,
        };

        Piece::new(color, piece_type)
    }
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
        let row = (square / 8) as isize;
        let col = (square % 8) as isize;

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
    pub const fn new(color: Colorr) -> Self {
        let mut moves = [0u64; 64];
        let mut square = 0;

        while square < 64 {
            moves[square] = Self::calculate_pawn_capture_moves(square, color);
            square += 1;
        }

        Self { moves }
    }

    const fn calculate_pawn_capture_moves(square: usize, color: Colorr) -> u64 {
        let mut result = 0u64;
        let row = (square / 8) as isize;
        let col = (square % 8) as isize;

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

pub struct KingMoves {
    pub moves: [u64; 64],
}

impl KingMoves {
    pub const fn new() -> Self {
        let mut moves = [0u64; 64];
        let mut square = 0;

        while square < 64 {
            moves[square] = Self::calculate_king_moves(square);
            square += 1;
        }

        Self { moves }
    }

    const fn calculate_king_moves(square: usize) -> u64 {
        let mut result = 0u64;
        let row = (square / 8) as isize;
        let col = (square % 8) as isize;

        let king_moves = [
            (row + 1, col + 1),
            (row + 1, col),
            (row + 1, col - 1),
            (row, col + 1),
            (row, col - 1),
            (row - 1, col + 1),
            (row - 1, col),
            (row - 1, col - 1),
        ];

        let mut i = 0;
        while i < 8 {
            let (new_row, new_col) = king_moves[i];
            if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                let new_square = (new_row * 8 + new_col) as usize;
                result |= 1u64 << new_square;
            }
            i += 1;
        }

        result
    }
}
