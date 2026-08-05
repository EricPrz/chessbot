use std::vec;

use crate::{
    board::Board,
    castling::CastlingRights,
    enums::Color::{self, WHITE},
    moves::Move,
};

struct Game {
    board: Board,
    turn: Color,
    castling: CastlingRights,
    half_move_clock: u8,
    fullmove_clock: u8,
    moves: vec::Vec<Move>,
}

impl Game {
    fn new() -> Self {
        Self {
            board: Board::new(),
            turn: WHITE,
            castling: CastlingRights::new(),
            half_move_clock: 0,
            fullmove_clock: 0,
            moves: Vec::new(),
        }
    }

    // to do
    fn from_fen(fen: String) -> Self {
        Self::new()
    }

    fn get_legal_moves(&self) -> Vec<Move> {}

    fn get_legal_moves_uci(&self) -> Vec<String> {}

    fn _generate_pseudo_legal_moves(&self) -> Vec<Move> {}

    fn _is_legal_move(&self) -> bool {}

    fn _apply_move(&mut self, move: Move) {}

    fn is_check(&self) -> bool {}

    fn is_checkmate(&self) -> bool {}

    fn is_stalemate(&self) -> bool {}

    fn is_draw(&self) -> bool {}

    fn get_fen(&self) -> String {}

    fn get_pgn(&self) -> String {}

    fn get_nnue_encoding(&self) {}
}
