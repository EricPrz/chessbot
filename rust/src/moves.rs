use crate::enums::PieceType::PAWN;
use crate::enums::Square;

use super::enums;
use super::piece;

pub struct Move {
    pub from_pos: enums::Square,
    pub to_pos: enums::Square,
    pub piece: piece::Piece,
    pub captured: Option<piece::Piece>,
    pub promotion: Option<enums::PieceType>,
    pub is_castle: bool,
    pub is_en_passant: bool,
}

impl Move {
    pub fn new(
        from_pos: enums::Square,
        to_pos: enums::Square,
        piece: piece::Piece,
        captured: Option<piece::Piece>,
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

    pub fn get_piece(&self) -> Piece {
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
