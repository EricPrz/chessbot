use super::enums;
use super::piece;

pub struct Move {
    from_pos: enums::Square,
    to_pos: enums::Square,
    piece: piece::Piece,
    captured: Option<piece::Piece>,
    promotion: Option<enums::PieceType>,
    is_castle: bool,
    is_en_passant: bool,
}

impl Move {
    pub fn new(from_pos: enums::Square, to_pos: enums::Square, piece: piece::Piece, captured: Option<piece::Piece>, promotion: Option<enums::PieceType>, is_castle: bool, is_en_passant: bool) -> Move {
        Move {
            from_pos:  from_pos,
            to_pos:  to_pos,
            piece:  piece,
            captured:  captured,
            promotion:  promotion,
            is_castle:  is_castle,
            is_en_passant:  is_en_passant,
        }
    }

    fn to_uci(&self) -> String {
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
            if self.to_pos.x > self.from_pos.x {
                String::from("0-0")
            } else {
                String::from("0-0-0")
            };
        }

        let piece_char: char = if self.piece.type == PieceType::PAWN { "" } else self.piece.type.to_char().to_uppercase();

        let capture: char = if self.captured || self.is_en_passant { "x" } else { "" };

        let promotion = "";
        if Some(self.promotion) {
            let promotion = self.promotion.to_char().to_uppercase(); 
        }

        if self.piece.tipe == PieceType::PAWN && capture == "x" {
            let piece_char = self.from_pos.to_uci()[0];
        }

        let mut san: String = String::new();
        san += &piece_char.to_string();
        san += &capture.to_string();
        san += &self.to_pos.to_uci();
        san += promotion;

        return san;
    }
}
