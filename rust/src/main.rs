use crate::game::Game;
use crate::moves::Move;
use crate::search::{SearchNode, TranspositionTable};
use std::time;

use log2::*;

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
mod search;

use nnue_rs::{Board, Network};

// Pseudocode for the 'uci' command response
fn handle_uci() {
    println!("id name MyEngine");
    println!("id author YourName");
    // Optional: announce options
    // println!("option name Hash type spin default 64 min 1 max 1024");
    println!("uciok");
    log::info!("uci");
}

fn handle_setoption(args: &Vec<&str>) {
    log::info!("setoption");

    for arg in args {
        log::info!("Arg: {}", arg);
    }
}

fn handle_position(args: &Vec<&str>, game: &mut Game) {
    log::info!("position");

    for arg in args {
        log::info!("Arg: {}", arg);
    }

    if args[1] == "fen" {
        let fen = args.get(2..8).unwrap().join(" ");
        log::info!("FEN parsed: {}", fen);

        *game = Game::from_fen(fen);

        if let Some(m) = args.get(8)
            && m == &"moves"
        {
            // let num_moves = args.len() - 9;
            for move_uci in args[9..].iter() {
                log::info!("UCI Move: {}", move_uci);

                let move_ = Move::from_uci(move_uci, &mut *game);

                match move_ {
                    Some(m) => {
                        game._apply_move(m);
                        log::info!("Applied move {}", m.to_uci());
                    }
                    None => {
                        log::info!("Move wasnt found for {}", move_uci);
                        panic!("Move wasnt found");
                    }
                }
            }
        }
    }

    log::info!("Applied FEN: {}", game.get_fen());
}

fn handle_go(args: &Vec<&str>, game: &Game, net: &Network, table: &mut TranspositionTable) {
    log::info!("go");

    for arg in args {
        log::info!("Arg: {}", arg);
    }

    let mut search_node = SearchNode::new_root(game, net, table);
    let final_move = search_node.iterative_deepening(MAX_DEPTH, net, table);

    match final_move {
        Some(m) => {
            log::info!("Found Move: {:?}", m);
            log::info!("Move uci: {:?}", m.to_uci());
            log::info!("CastlingRights: {:?}", game.castling);
            println!("bestmove {}", m.to_uci());
        }
        None => {
            log::info!("No move found on search");
            panic!("F");
        }
    }
}

fn handle_ucinewgame() {
    log::info!("ucinewgame");
}

fn handle_ponderhit() {
    log::info!("ponderhit");
}

fn handle_stop() {
    log::info!("stop");
}

const MAX_DEPTH: i32 = 8;

fn main() {
    // Initialize essentials
    let _log2 = log2::open("logs/my_engine.txt").start();
    let net =
        Network::from_file("/home/eric/Projects/chessbot/rust/src/nn-47fc8b7fff06.nnue").unwrap();
    let mut table = TranspositionTable::new();
    let mut game = Game::new();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.split_whitespace().collect();

        match args[0] {
            "uci" => handle_uci(),
            "isready" => println!("readyok"), // Engine is ready
            "setoption" => handle_setoption(&args),
            "ucinewgame" => handle_ucinewgame(),
            "position" => handle_position(&args, &mut game),
            "ponderhit" => handle_ponderhit(),
            "go" => handle_go(&args, &game, &net, &mut table), // Calls your alpha-beta search!
            "stop" => handle_stop(),
            "quit" => break,
            _ => {} // Ignore unknown commands
        }
    }
}
