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

use nnue_rs::{Board, Network};

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
    // fn main() -> io::Result<()> {
    let cli = Args::parse();

    if cli.mode == "SEARCH" {
        let start = "r1bq1rk1/Npp2ppp/3p4/2b5/2K1P3/4P3/PPPP2PP/RNBQ1B1R b - - 0 9";
        let mut game = Game::from_fen(start.to_string());

        let net = Network::from_file("/home/eric/Projects/chessbot/rust/src/nn-37f18f62d772.nnue")
            .expect("ay");

        let root_acc = net.accumulator(&game);
        let score = net.evaluate_accumulator(&root_acc, game.side_to_move());
        println!("Acc Score: {}", score);

        // let net = Network::from_file("net.nnue")?;
        //
        // // Compute the accumulator once for the root position.
        // let root_acc = net.accumulator(&parent);
        //
        // // For each child, advance into a fresh accumulator slot.
        // let mut child_acc = net.empty_accumulator();
        // net.update(&parent, &child, &root_acc, &mut child_acc);
        //
        // // Evaluate. Side to move is passed separately so the same accumulator can be
        // // reused across a null move.
        // let score = net.evaluate_accumulator(&child_acc, child.side_to_move());
    }

    if cli.mode == "NNUE" {
        let net = Network::from_file("/home/eric/Projects/chessbot/rust/src/nn-37f18f62d772.nnue")
            .expect("ay");
        let start = "r1bq1rk1/Npp2ppp/3p4/2b5/2K1P3/4P3/PPPP2PP/RNBQ1B1R b - - 0 9";
        let mut game = Game::from_fen(start.to_string());
        // let move_ = Move::from_uci("e4", &mut game);
        // game._apply_move(move_.unwrap());

        let fen = game.get_fen();
        println!("{}", fen);
        let score = net.evaluate_fen(start).expect("ay");
        println!("score: {score}");
    }

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
            game.get_fen();

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
                let all_mvs = game._generate_pseudo_legal_moves();
                for mv in all_mvs {
                    println!("Pseudo Legal Mv: {:?}", mv);
                }

                let all_mvs = game.get_legal_moves();
                for mv in all_mvs {
                    println!("Legal Mv: {:?}", mv);
                }

                let mv = Move::from_uci(move_, &mut game);
                if mv.is_some() {
                    println!("Selected move: {:?}", mv);
                    game._apply_move(mv.unwrap());
                    println!("Turn after _apply_move: {}", game.turn.to_char());
                    println!("Internal FEN: {}", game.board.to_fen());
                    game.board.print();
                } else {
                    if move_ == "1-0" || move_ == "0-1" || move_ == "1/2-1/2" {
                        println!("Game finished: {}", move_);
                    } else {
                        println!("Game: {}", line_game);
                        panic!("Move not found: {}", move_);
                    }
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
