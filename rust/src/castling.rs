mod board;

struct CastlingRights {
    white_kingside: bool,
    black_kingside: bool,
    white_queenside: bool,
    black_queenside: bool,
}

impl CastlingRights {
    fn get_for_color(&self, color: &Color) -> (bool, bool) {
        if color == Color::WHITE {
            return (self.white_kingside, self.white_queenside);
        } else {
            return (self.black_kingside, self.black_queenside);
        }
    }

    fn to_string(&self) -> String {
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
}
