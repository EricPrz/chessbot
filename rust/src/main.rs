use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use chess_engine::game::Game;
use chess_engine::moves::Move;
use chess_engine::search::{TranspositionTable, Zobrist, iterative_deepening_threaded};
use flexi_logger::{FileSpec, Logger};
use nnue_rs::{Accumulator, Network};

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

fn handle_position(args: &Vec<&str>, game: &mut Game, zobrist: &Zobrist) {
    log::info!("position");

    for arg in args {
        log::info!("Arg: {}", arg);
    }

    if args[1] == "fen" {
        let fen = args.get(2..8).unwrap().join(" ");
        log::info!("FEN parsed: {}", fen);

        *game = Game::from_fen(fen);
        game.hash = Some(game.compute_hash(zobrist));

        if let Some(m) = args.get(8)
            && m == &"moves"
        {
            // let num_moves = args.len() - 9;
            for move_uci in args[9..].iter() {
                log::info!("UCI Move: {}", move_uci);

                let move_ = Move::from_uci(move_uci, &mut *game);

                match move_ {
                    Some(m) => {
                        game._apply_move(m, zobrist);
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
    game: &mut Game,
    net: &Network,
    table: &mut TranspositionTable,
    on_search: &Arc<AtomicBool>,
    zobrist: &Zobrist,
) -> Option<Move> {
    log::info!("Search started in thread");

    // let mut search_node = SearchNode::new_root(game, net, table);

    // Convert AtomicBool to a mutable reference for the search
    // We'll use a wrapper to check the flag
    // let final_move = search_node.iterative_deepening_threaded(MAX_DEPTH, net, table, on_search);
    let acc = net.accumulator(game);
    let final_move =
        iterative_deepening_threaded(game, &acc, MAX_DEPTH, net, table, on_search, zobrist);

    println!("Ended go");

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
    let logger = Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default())
        .start()
        .unwrap();
    let net = Arc::new(
        Network::from_file("/home/eric/Projects/chessbot/rust/src/nn-47fc8b7fff06.nnue").unwrap(),
    );
    let mut table = TranspositionTable::new();
    let mut game = Game::new();

    let on_search = Arc::new(AtomicBool::new(false));
    let mut search_thread: Option<thread::JoinHandle<Option<Move>>> = None;

    let zobrist = Arc::new(Zobrist::new());

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.split_whitespace().collect();

        match args[0] {
            "uci" => handle_uci(),
            "isready" => println!("readyok"), // Engine is ready
            "setoption" => handle_setoption(&args),
            "ucinewgame" => handle_ucinewgame(),
            "position" => handle_position(&args, &mut game, &zobrist),
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
                let mut game_clone = game.clone();
                let mut table_clone = TranspositionTable::new();
                let on_search_clone = Arc::clone(&on_search);

                let zobrist_clone = Arc::clone(&zobrist); // clone for thread
                // Spawn search thread
                search_thread = Some(thread::spawn(move || {
                    let result = handle_go_threaded(
                        &mut game_clone,
                        &net_clone,
                        &mut table_clone,
                        &on_search_clone,
                        &*zobrist_clone,
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
