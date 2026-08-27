use crate::enums::Colorr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub black_kingside: bool,
    pub white_queenside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        Self {
            white_kingside: true,
            black_kingside: true,
            white_queenside: true,
            black_queenside: true,
        }
    }

    pub fn castling_mask(&self) -> u8 {
        let mut mask = 0;

        if self.white_kingside {
            mask |= 1;
        } // Bit 0
        if self.white_queenside {
            mask |= 2;
        } // Bit 1
        if self.black_kingside {
            mask |= 4;
        } // Bit 2
        if self.black_queenside {
            mask |= 8;
        } // Bit 3

        mask
    }

    pub fn get_for_color(&self, color: &Colorr) -> (bool, bool) {
        if color == &Colorr::WHITE {
            return (self.white_kingside, self.white_queenside);
        } else {
            return (self.black_kingside, self.black_queenside);
        }
    }

    pub fn to_string(&self) -> String {
        let mut castlings = String::new();

        if self.white_kingside {
            castlings += "K"
        }

        if self.white_queenside {
            castlings += "Q"
        }

        if self.black_kingside {
            castlings += "k"
        }

        if self.black_queenside {
            castlings += "q"
        }

        if castlings.is_empty() {
            return String::from("-");
        }

        return castlings;
    }

    pub fn from_string(string: String) -> CastlingRights {
        CastlingRights {
            white_kingside: string.find('K').is_some(),
            black_kingside: string.find('k').is_some(),
            white_queenside: string.find('Q').is_some(),
            black_queenside: string.find('q').is_some(),
        }
    }
}
