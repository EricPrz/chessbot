use crate::game::Game;
use crate::moves::Move;
use crate::search::{SearchNode, TranspositionTable};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

mod board;
mod castling;
mod enums;
mod game;
mod moves;
mod piece;
mod search;

use nnue_rs::Network;

const MAX_DEPTH: i32 = 15;

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

fn handle_go_threaded(
    game: &Game,
    net: &Network,
    table: &mut TranspositionTable,
    on_search: &Arc<AtomicBool>,
) -> Option<Move> {
    log::info!("Search started in thread");

    let mut search_node = SearchNode::new_root(game, net, table);

    // Convert AtomicBool to a mutable reference for the search
    // We'll use a wrapper to check the flag
    let final_move = search_node.iterative_deepening_threaded(MAX_DEPTH, net, table, on_search);

    match final_move {
        Some(m) => {
            log::info!("Found Move: {:?}", m);
            log::info!("Move uci: {:?}", m.to_uci());
            println!("bestmove {}", m.to_uci());
            Some(m)
        }
        None => {
            log::info!("No move found on search");
            None
        }
    }
}
fn handle_ucinewgame() {
    log::info!("ucinewgame");
}

fn handle_ponderhit() {
    log::info!("ponderhit");
}

fn handle_stop(on_search: &mut Arc<AtomicBool>) {
    *on_search = Arc::new(AtomicBool::new(false));
    log::info!("stop");
}

fn main() {
    // Initialize essentials
    let _log2 = log2::open("logs/my_engine.txt").start();
    let net = Arc::new(
        Network::from_file("/home/eric/Projects/chessbot/rust/src/nn-47fc8b7fff06.nnue").unwrap(),
    );
    let mut table = TranspositionTable::new();
    let mut game = Game::new();

    let on_search = Arc::new(AtomicBool::new(false));
    let mut search_thread: Option<thread::JoinHandle<Option<Move>>> = None;

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
            "go" => {
                // If there's a previous search thread, wait for it
                if let Some(thread) = search_thread.take() {
                    let _ = thread.join();
                }

                // Reset the flag
                on_search.store(true, Ordering::SeqCst);

                // Clone data for the thread
                let net_clone = net.clone(); // Assuming Network implements Clone
                let game_clone = game.clone();
                let mut table_clone = TranspositionTable::new();
                let on_search_clone = Arc::clone(&on_search);

                // Spawn search thread
                search_thread = Some(thread::spawn(move || {
                    let result = handle_go_threaded(
                        &game_clone,
                        &net_clone,
                        &mut table_clone,
                        &on_search_clone,
                    );
                    result
                }));
            }
            "stop" => {
                on_search.store(false, Ordering::SeqCst);
                // Wait for search thread to finish
                if let Some(thread) = search_thread.take() {
                    let _ = thread.join();
                }
            }
            "quit" => {
                on_search.store(false, Ordering::SeqCst);
                if let Some(thread) = search_thread.take() {
                    let _ = thread.join();
                }
                break;
            }
            _ => {} // Ignore unknown commands
        }
    }
}
