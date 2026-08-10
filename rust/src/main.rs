use crate::game::Game;

mod board;
mod castling;
mod enums;
mod game;
mod moves;
mod piece;
mod position;

fn main() {
    println!("Hello, world!");

    let game = Game::new();
    let legal_moves = game.get_legal_moves_uci();

    for move_ in legal_moves {
        println!("{}", move_);
    }
}
