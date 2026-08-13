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

    let game = Game::from_fen(String::from(
        // "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        // "r1bqkbnr/ppp1pppp/n7/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
        "r1bqkbnr/ppppp1pp/n7/5pP1/8/8/PPPPPP1P/RNBQKBNR w KQkq f6 0 3",
    ));
    // let game = Game::new();
    println!("Board: {}", game.board);
    let legal_moves = game.get_legal_moves();
    println!("There are {} moves", legal_moves.len());

    for move_ in legal_moves {
        println!(
            "{} {} {}",
            move_.piece.to_char(),
            move_.from_pos.to_uci(),
            move_.to_pos.to_uci()
        );
    }
}
