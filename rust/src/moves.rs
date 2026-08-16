use crate::enums::PieceType;
use crate::enums::PieceType::PAWN;
use crate::enums::Square;
use crate::game::Game;
use crate::piece::Piecee;

use log::{info, warn};
use regex::Regex;
use std::str::FromStr;

use super::enums;
use super::piece;

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from_pos: enums::Square,
    pub to_pos: enums::Square,
    pub piece: piece::Piecee,
    pub captured: Option<piece::Piecee>,
    pub promotion: Option<enums::PieceType>,
    pub is_castle: bool,
    pub is_en_passant: bool,
}

impl Move {
    pub fn new(
        from_pos: enums::Square,
        to_pos: enums::Square,
        piece: piece::Piecee,
        captured: Option<piece::Piecee>,
        promotion: Option<enums::PieceType>,
        is_castle: bool,
        is_en_passant: bool,
    ) -> Move {
        Move {
            from_pos: from_pos,
            to_pos: to_pos,
            piece: piece,
            captured: captured,
            promotion: promotion,
            is_castle: is_castle,
            is_en_passant: is_en_passant,
        }
    }

    pub fn get_to_pos(&self) -> Square {
        self.to_pos
    }

    pub fn get_piece(&self) -> Piecee {
        self.piece
    }

    pub fn to_uci(&self) -> String {
        let mut uci = String::new();
        uci += &self.from_pos.to_uci();
        uci += &self.to_pos.to_uci();

        if let Some(promo) = &self.promotion {
            uci += &promo.to_char().to_string();
        }

        return uci;
    }

    pub fn from_uci(san: &str, game: &mut Game) -> Option<Move> {
        log::debug!("FEN: {}", game.get_fen());
        let color = game.turn;

        // 1. Castling Checks
        let mut san_clean = san.replace(|c: char| ['+', '#', '!', '?', '*', ' '].contains(&c), "");
        log::debug!("Clean SAN: {}", san_clean);

        let is_castling = matches!(san_clean.as_str(), "0-0" | "O-O" | "0-0-0" | "O-O-O");
        if is_castling {
            log::debug!("Castling!!");
            println!("UCI Castling!!");
            println!("Castling Rights: {:?}", game.castling);
            let king_square = game.board.find_king(color);
            let king = game
                .board
                .get_piece_at_square(king_square)
                .expect("Always should be a king");

            let moves = king.get_pseudo_legal_moves(&game.board, &game.castling);
            log::debug!("King moves: {:?}", moves);
            log::debug!("Castling Moves: {:?}", moves);

            let king_side = san_clean.len() <= 3;
            for mv in moves {
                if king_side && mv.to_pos.get_col_index() == 6 {
                    // game._apply_move(mv);
                    return Some(mv);
                } else if !king_side && mv.to_pos.get_col_index() == 2 {
                    // game._apply_move(mv);
                    return Some(mv);
                }
            }

            log::debug!("No castling");
            return None;
        }

        // 2. Game result strings
        if matches!(san_clean.as_str(), "0-1" | "1-0" | "1/2-1/2") {
            log::debug!("Result: {}", san_clean);
            return None; // oder ein spezielles `MoveResult`-Enum, falls benötigt
        }

        let piece_map: std::collections::HashMap<char, PieceType> = [
            ('Q', PieceType::QUEEN),
            ('K', PieceType::KING),
            ('R', PieceType::ROOK),
            ('B', PieceType::BISHOP),
            ('N', PieceType::KNIGHT),
        ]
        .iter()
        .cloned()
        .collect();

        // 3. Promotion
        let mut promoted_piece: Option<PieceType> = None;
        if let Some(pos) = san_clean.find('=') {
            let parts: Vec<&str> = san_clean.split('=').collect();
            if let Some(promotion_part) = parts.get(1) {
                let promoted_letter = promotion_part.chars().next()?;
                promoted_piece = piece_map.get(&promoted_letter).cloned();
                log::debug!("Promoted Piece: {:?}", promoted_piece);

                let _eq_sign_idx = san_clean.find("=");
                san_clean.remove(_eq_sign_idx.unwrap());
                san_clean.remove(_eq_sign_idx.unwrap());
            }
        }

        if san_clean.len() < 2 {
            log::debug!("Failed to parse SAN (too short): {}", san);
            return None;
        }

        // Target square is always the final two characters
        let dest_str = &san_clean[san_clean.len() - 2..];
        let remainder = &san_clean[..san_clean.len() - 2];

        if !Regex::new(r"^[a-h][1-8]$").unwrap().is_match(dest_str) {
            log::debug!("Failed to parse SAN (invalid destination): {}", san);
            println!("Failed to parse SAN (invalid destination): {}", san);
            return None;
        }

        println!("Promoted Piece: {:?}", &promoted_piece);

        let destination_position = Square::from_uci(dest_str);
        log::debug!("Destination Position: {:?}", destination_position);

        let is_capture = remainder.contains('x');
        let remainder = if is_capture {
            remainder.replace('x', "")
        } else {
            remainder.to_string()
        };
        log::debug!("Is Capture: {}", is_capture);

        // Identify moving piece
        let mut piece_type = PAWN;
        let mut remainder_chars = remainder.chars().peekable();
        if let Some(c) = remainder_chars.peek() {
            if let Some(&pt) = piece_map.get(c) {
                piece_type = pt;
                remainder_chars.next();
            }
        }
        let remainder: String = remainder_chars.collect();
        log::debug!("Piece Type: {:?}", piece_type);

        // Remaining characters represent disambiguation (e.g., 'a', '1', or 'a1')
        let mut dis_file: Option<char> = None;
        let mut dis_rank: Option<char> = None;

        let remainder_chars: Vec<char> = remainder.chars().collect();
        for char in &remainder_chars {
            println!("Remainder char: {}", char);
        }
        match remainder_chars.len() {
            1 => {
                let c = remainder_chars[0];
                if ('a'..='h').contains(&c) {
                    dis_file = Some(c);
                } else if ('1'..='8').contains(&c) {
                    dis_rank = Some(c);
                }
            }
            2 => {
                let c1 = remainder_chars[0];
                let c2 = remainder_chars[1];
                if ('a'..='h').contains(&c1) && ('1'..='8').contains(&c2) {
                    dis_file = Some(c1);
                    dis_rank = Some(c2);
                }
            }
            _ => {}
        }

        if dis_file.is_some() {
            println!("Dis file: {}", dis_file.unwrap());
        }
        if dis_rank.is_some() {
            println!("Dis rank: {}", dis_rank.unwrap());
        }

        let req_file = dis_file.map(|c| c as u8 - b'a');
        let req_rank = dis_rank.map(|c| c as u8 - b'1');

        if req_file.is_some() {
            println!("Req file: {}", req_file.unwrap());
        }
        if req_rank.is_some() {
            println!("Req rank: {}", req_rank.unwrap());
        }

        let turn = game.turn;
        let piece = Piecee::new(piece_type, turn);
        log::debug!("Piece: {:?}", piece);

        // 4. Fetch Legal Moves
        let all_moves_raw = game.get_legal_moves();
        let mut possible_moves = Vec::new();

        println!("There are {} raw moves", all_moves_raw.len());

        for item in all_moves_raw {
            let mv = item;

            println!("Move Raw: {:?}", item);
            println!("Col Idx: {:?}", mv.from_pos.get_col_index());
            println!("Row Idx: {:?}", mv.from_pos.get_row_index());

            if mv.is_castle {
                continue;
            }

            if let Some(target_promo) = promoted_piece {
                if mv.promotion != Some(target_promo) {
                    continue;
                }
            } else {
                // If the user didn't specify a promotion, skip moves that require one
                if mv.promotion.is_some() {
                    continue;
                }
            }

            if mv.to_pos != destination_position {
                continue;
            }

            if mv.piece.piece_type != piece_type || mv.piece.color != turn {
                continue;
            }

            if let Some(req_file) = req_file {
                if mv.from_pos.get_col_index() != req_file {
                    continue;
                }
            }

            if let Some(req_rank) = req_rank {
                if mv.from_pos.get_row_index() != 7 - req_rank {
                    continue;
                }
            }

            possible_moves.push(mv);
        }

        // let chosen_move = if possible_moves.len() >= 1 {
        //     possible_moves
        // } else {
        //     log::debug!(
        //         "Unable to find origin position for move: {}\nFEN: {}",
        //         san,
        //         game.get_fen()
        //     );
        //     return None;
        // };

        let chosen_move = if possible_moves.len() == 1 {
            possible_moves[0]
        } else {
            println!(
                "Unable to find origin position for move: {}\nFEN: {}",
                san,
                game.get_fen()
            );
            for mv in possible_moves {
                println!("Possible move: {:?}", mv);
            }
            return None;
        };

        let mut is_en_passant = false;
        if piece_type == PAWN
            && chosen_move.from_pos.get_col_index() != destination_position.get_col_index()
        {
            if game
                .board
                .get_piece_at_square(destination_position)
                .is_none()
            {
                is_en_passant = true;
            }
        }

        let captured_piece = game.board.get_piece_at_square(destination_position);

        let final_move = Move::new(
            chosen_move.from_pos,
            destination_position,
            piece,
            captured_piece,
            promoted_piece,
            is_castling,
            is_en_passant,
        );

        Some(final_move)
    }

    fn to_san(&self) -> String {
        if self.is_castle {
            if self.to_pos.get_col_index() > self.from_pos.get_col_index() {
                String::from("0-0")
            } else {
                String::from("0-0-0")
            };
        }

        let mut piece_char: String = if self.piece.piece_type == PAWN {
            String::new()
        } else {
            self.piece
                .piece_type
                .to_char()
                .to_ascii_uppercase()
                .to_string()
        };

        let capture: String = if self.captured.is_some() || self.is_en_passant {
            'x'.to_string()
        } else {
            String::new()
        };

        let promotion: String = match self.promotion {
            Some(p) => p.to_char().to_ascii_uppercase().to_string(),
            None => String::new(),
        };

        if self.piece.piece_type == PAWN && capture == "x" {
            piece_char = self
                .from_pos
                .to_uci()
                .get(..1)
                .expect("There should be something when returning uci move")
                .to_string();
        }

        let mut san: String = String::new();
        san += &piece_char;
        san += &capture.to_string();
        san += &self.to_pos.to_uci();
        san += &promotion;

        return san;
    }
}
