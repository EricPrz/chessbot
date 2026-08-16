use std::collections::HashMap;

use crate::enums::Colorr::{BLACK, WHITE};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Square {
    A8 = 0,
    B8 = 1,
    C8 = 2,
    D8 = 3,
    E8 = 4,
    F8 = 5,
    G8 = 6,
    H8 = 7,
    A7 = 8,
    B7 = 9,
    C7 = 10,
    D7 = 11,
    E7 = 12,
    F7 = 13,
    G7 = 14,
    H7 = 15,
    A6 = 16,
    B6 = 17,
    C6 = 18,
    D6 = 19,
    E6 = 20,
    F6 = 21,
    G6 = 22,
    H6 = 23,
    A5 = 24,
    B5 = 25,
    C5 = 26,
    D5 = 27,
    E5 = 28,
    F5 = 29,
    G5 = 30,
    H5 = 31,
    A4 = 32,
    B4 = 33,
    C4 = 34,
    D4 = 35,
    E4 = 36,
    F4 = 37,
    G4 = 38,
    H4 = 39,
    A3 = 40,
    B3 = 41,
    C3 = 42,
    D3 = 43,
    E3 = 44,
    F3 = 45,
    G3 = 46,
    H3 = 47,
    A2 = 48,
    B2 = 49,
    C2 = 50,
    D2 = 51,
    E2 = 52,
    F2 = 53,
    G2 = 54,
    H2 = 55,
    A1 = 56,
    B1 = 57,
    C1 = 58,
    D1 = 59,
    E1 = 60,
    F1 = 61,
    G1 = 62,
    H1 = 63,
}

impl Square {
    pub fn new(x: u8, y: u8) -> Square {
        let number = x + y * 8;
        Square::square_from_number(number)
    }
    pub fn square_from_number(num: u8) -> Square {
        // println!("Square from number: {}", num);
        match num {
            0 => Square::A8,
            1 => Square::B8,
            2 => Square::C8,
            3 => Square::D8,
            4 => Square::E8,
            5 => Square::F8,
            6 => Square::G8,
            7 => Square::H8,
            8 => Square::A7,
            9 => Square::B7,
            10 => Square::C7,
            11 => Square::D7,
            12 => Square::E7,
            13 => Square::F7,
            14 => Square::G7,
            15 => Square::H7,
            16 => Square::A6,
            17 => Square::B6,
            18 => Square::C6,
            19 => Square::D6,
            20 => Square::E6,
            21 => Square::F6,
            22 => Square::G6,
            23 => Square::H6,
            24 => Square::A5,
            25 => Square::B5,
            26 => Square::C5,
            27 => Square::D5,
            28 => Square::E5,
            29 => Square::F5,
            30 => Square::G5,
            31 => Square::H5,
            32 => Square::A4,
            33 => Square::B4,
            34 => Square::C4,
            35 => Square::D4,
            36 => Square::E4,
            37 => Square::F4,
            38 => Square::G4,
            39 => Square::H4,
            40 => Square::A3,
            41 => Square::B3,
            42 => Square::C3,
            43 => Square::D3,
            44 => Square::E3,
            45 => Square::F3,
            46 => Square::G3,
            47 => Square::H3,
            48 => Square::A2,
            49 => Square::B2,
            50 => Square::C2,
            51 => Square::D2,
            52 => Square::E2,
            53 => Square::F2,
            54 => Square::G2,
            55 => Square::H2,
            56 => Square::A1,
            57 => Square::B1,
            58 => Square::C1,
            59 => Square::D1,
            60 => Square::E1,
            61 => Square::F1,
            62 => Square::G1,
            63 => Square::H1,
            _ => panic!("Invalid Square. num: {}", num),
        }
    }

    pub fn to_uci(&self) -> String {
        match self {
            Square::A8 => "a8".to_string(),
            Square::B8 => "b8".to_string(),
            Square::C8 => "c8".to_string(),
            Square::D8 => "d8".to_string(),
            Square::E8 => "e8".to_string(),
            Square::F8 => "f8".to_string(),
            Square::G8 => "g8".to_string(),
            Square::H8 => "h8".to_string(),
            Square::A7 => "a7".to_string(),
            Square::B7 => "b7".to_string(),
            Square::C7 => "c7".to_string(),
            Square::D7 => "d7".to_string(),
            Square::E7 => "e7".to_string(),
            Square::F7 => "f7".to_string(),
            Square::G7 => "g7".to_string(),
            Square::H7 => "h7".to_string(),
            Square::A6 => "a6".to_string(),
            Square::B6 => "b6".to_string(),
            Square::C6 => "c6".to_string(),
            Square::D6 => "d6".to_string(),
            Square::E6 => "e6".to_string(),
            Square::F6 => "f6".to_string(),
            Square::G6 => "g6".to_string(),
            Square::H6 => "h6".to_string(),
            Square::A5 => "a5".to_string(),
            Square::B5 => "b5".to_string(),
            Square::C5 => "c5".to_string(),
            Square::D5 => "d5".to_string(),
            Square::E5 => "e5".to_string(),
            Square::F5 => "f5".to_string(),
            Square::G5 => "g5".to_string(),
            Square::H5 => "h5".to_string(),
            Square::A4 => "a4".to_string(),
            Square::B4 => "b4".to_string(),
            Square::C4 => "c4".to_string(),
            Square::D4 => "d4".to_string(),
            Square::E4 => "e4".to_string(),
            Square::F4 => "f4".to_string(),
            Square::G4 => "g4".to_string(),
            Square::H4 => "h4".to_string(),
            Square::A3 => "a3".to_string(),
            Square::B3 => "b3".to_string(),
            Square::C3 => "c3".to_string(),
            Square::D3 => "d3".to_string(),
            Square::E3 => "e3".to_string(),
            Square::F3 => "f3".to_string(),
            Square::G3 => "g3".to_string(),
            Square::H3 => "h3".to_string(),
            Square::A2 => "a2".to_string(),
            Square::B2 => "b2".to_string(),
            Square::C2 => "c2".to_string(),
            Square::D2 => "d2".to_string(),
            Square::E2 => "e2".to_string(),
            Square::F2 => "f2".to_string(),
            Square::G2 => "g2".to_string(),
            Square::H2 => "h2".to_string(),
            Square::A1 => "a1".to_string(),
            Square::B1 => "b1".to_string(),
            Square::C1 => "c1".to_string(),
            Square::D1 => "d1".to_string(),
            Square::E1 => "e1".to_string(),
            Square::F1 => "f1".to_string(),
            Square::G1 => "g1".to_string(),
            Square::H1 => "h1".to_string(),
        }
    }

    pub fn from_uci(uci: &str) -> Self {
        if uci.len() != 2 {
            panic!("UCI position is not 2 char long")
        }

        let file = uci.chars().next().expect("UCI should have file");
        let rank = uci.chars().nth(1).expect("UCI should have rank");

        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            panic!("UCI is not valid")
        }

        // Konvertiere den File-Char in einen Index (0-7)
        let file_index = (file as u8 - b'a') as usize;
        // Konvertiere den Rank-Char in einen Index (0-7)
        let rank_index = (rank as u8 - b'1') as usize;

        // Berechne den Index des Squares (0 = A1, 1 = B1, ..., 63 = H8)
        let index = (7 - rank_index) * 8 + file_index;

        // Konvertiere den Index zurück in das Square-Enum
        let sqr = Square::square_from_number(index as u8);

        println!(
            "From uci: {} {}, {} {}, {}",
            file,
            rank,
            file_index,
            rank_index,
            sqr.to_uci()
        );

        sqr
    }

    pub fn square_to_number(square: Square) -> u8 {
        square as u8
    }

    pub fn to_index(&self) -> u8 {
        self.clone() as u8
    }

    pub fn get_row_index(&self) -> u8 {
        let rev_index = self.clone() as u8 / 8;
        rev_index
    }

    pub fn get_col_index(&self) -> u8 {
        let rev_index = self.clone() as u8 % 8;
        rev_index
    }

    pub fn is_valid(&self) -> bool {
        self.to_index() < 64 && self.to_index() >= 0
    }
}

pub fn get_positions_from_bitboard(_bitboard: &u64) -> Vec<u8> {
    let mut bitboard = _bitboard.clone();
    let mut positions: Vec<u8> = Vec::new();

    while bitboard != 0 {
        // Isolate the MSB
        let msb = bitboard & bitboard.wrapping_neg();

        // Find the square index (0-63)
        let position = 63 - msb.leading_zeros() as u8;
        positions.push(position);

        // Remove the processed bit
        bitboard ^= msb;
    }

    positions
}

pub fn get_bitboard_from_square(square: Square) -> u64 {
    // Convert the Square to a position number (0-63)
    let position = square.to_index();

    // Set the bit at the corresponding position
    1u64 << position
}

pub fn get_squares_from_bitboard(_bitboard: &u64) -> Vec<Square> {
    let positions = get_positions_from_bitboard(_bitboard);
    let squares = positions
        .into_iter()
        .filter(|position| (0..64).contains(position))
        .map(|position| Square::square_from_number(position))
        .collect();

    squares
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Colorr {
    WHITE,
    BLACK,
}

impl Colorr {
    pub fn opposite(&self) -> Colorr {
        if self == &Colorr::WHITE {
            return Colorr::BLACK;
        }

        if self == &Colorr::BLACK {
            return Colorr::WHITE;
        }

        panic!("No Color matched.")
    }

    pub fn to_char(&self) -> char {
        if self == &Colorr::WHITE { 'w' } else { 'b' }
    }

    pub fn from_char(char: &str) -> Colorr {
        match char {
            "w" => WHITE,
            "b" => BLACK,
            _ => panic!("Char {} doesn't match with a color", char),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum PieceType {
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6,
}

impl PieceType {
    pub fn from_char(char: char) -> Self {
        match char.to_ascii_lowercase() {
            'p' => Self::PAWN,
            'n' => Self::KNIGHT,
            'b' => Self::BISHOP,
            'r' => Self::ROOK,
            'q' => Self::QUEEN,
            'k' => Self::KING,
            _ => panic!("Invalid piece character: {}", char),
        }
    }

    pub fn to_char(&self) -> char {
        let piece_map: HashMap<PieceType, char> = HashMap::from([
            (PieceType::PAWN, 'p'),
            (PieceType::KNIGHT, 'n'),
            (PieceType::BISHOP, 'b'),
            (PieceType::ROOK, 'r'),
            (PieceType::QUEEN, 'q'),
            (PieceType::KING, 'k'),
        ]);

        piece_map.get(self).expect("No Piece matched.").to_owned()
    }

    fn get_value(&self) -> u8 {
        let piece_map: HashMap<PieceType, u8> = HashMap::from([
            (PieceType::PAWN, 1),
            (PieceType::KNIGHT, 3),
            (PieceType::BISHOP, 3),
            (PieceType::ROOK, 5),
            (PieceType::QUEEN, 9),
            (PieceType::KING, 0),
        ]);

        piece_map.get(self).expect("No Piece matched.").to_owned()
    }

    /// Returns an iterator over all `PieceType` variants.
    pub fn iter() -> PieceTypeIterator {
        PieceTypeIterator { index: 0 }
    }
}

pub struct PieceTypeIterator {
    index: usize,
}

impl Iterator for PieceTypeIterator {
    type Item = PieceType;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index {
            0 => {
                self.index += 1;
                Some(PieceType::PAWN)
            }
            1 => {
                self.index += 1;
                Some(PieceType::KNIGHT)
            }
            2 => {
                self.index += 1;
                Some(PieceType::BISHOP)
            }
            3 => {
                self.index += 1;
                Some(PieceType::ROOK)
            }
            4 => {
                self.index += 1;
                Some(PieceType::QUEEN)
            }
            5 => {
                self.index += 1;
                Some(PieceType::KING)
            }
            _ => None,
        }
    }
}
