mod board;
mod castling;
mod enums;
mod game;
mod moves;
mod piece;
mod position;

fn main() {
    println!("Hello, world!");
    let position = position::Position { x: 0, y: 0 };
    let ay = position;
    let ay = ay.offset(3, 2).unwrap().to_uci();
    println!("{ay}");
}
