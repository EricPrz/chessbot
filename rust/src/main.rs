use crate::game::Game;
use crate::moves::Move;

use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

mod board;
mod castling;
mod enums;
mod game;
mod moves;
mod piece;
mod position;

#[derive(PartialEq, Clone)]
enum Mode {
    PGN = 0,
    UCI = 1,
    DEBUG = 2,
}

#[derive(Parser)]
struct Args {
    mode: String,
}

fn main() -> io::Result<()> {
    let cli = Args::parse();

    if cli.mode == "PGN" {
        println!("PGN");

        let file = File::open("/run/media/eric/FAEF-F582/lichess_db_standard_rated_2026-06.pgn")?;
        let reader = BufReader::new(file);

        let re = Regex::new(r"(\{[^}]*\})|(\d*\.{1,3})").unwrap();

        let mut line = String::new();
        for line in reader.lines() {
            let line = line?;
            if line.starts_with("[") {
                continue;
            }

            let mut game = Game::new();

            let line_game = re.replace_all(&line, "");
            for move_ in line_game.split(" ") {
                if move_.is_empty() {
                    continue;
                } else {
                    println!(
                        "--------------------------------------------- UCI Move: {}",
                        move_
                    );
                }

                println!("Turn: {}", game.turn.to_char());

                let all_mvs = game.get_legal_moves();
                for mv in all_mvs {
                    println!("Mv: {:?}", mv);
                }

                let mv = Move::from_uci(move_, &mut game);
                if mv.is_some() {
                    println!("Selected move: {:?}", mv);
                    game._apply_move(mv.unwrap());
                    println!("Turn after _apply_move: {}", game.turn.to_char());
                } else {
                    println!("Game: {}", line_game);
                    panic!("Move not found");
                }
            }
        }
    }

    if cli.mode == "DEBUG" {
        let game = Game::from_fen(String::from(
            "rnbqk2r/ppppbppp/4pn2/8/8/4PN2/PPPPBPPP/RNBQ1RK1 b kq - 3 4",
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

    Ok(())
}
