use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::{env, thread};

use chess_engine::game::Game;
use chess_engine::moves::Move;
use chess_engine::piece::KingMoves;
use chess_engine::search::{
    HistoryTable, KillerMoves, TranspositionTable, Zobrist, iterative_deepening_threaded,
};
use flexi_logger::{Duplicate, FileSpec, Logger};
use nnue_rs::{Accumulator, Network};
use reqwest::Response;

const MAX_DEPTH: usize = 6;

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
    table: Arc<RwLock<TranspositionTable>>,
    on_search: &Arc<AtomicBool>,
    zobrist: &Zobrist,
    killers: Arc<RwLock<KillerMoves>>,
    history: Arc<RwLock<HistoryTable>>,
) -> Option<Move> {
    log::info!("Search started in thread");

    // let mut search_node = SearchNode::new_root(game, net, table);
    let mut table = table.write().unwrap();
    let mut killers = killers.write().unwrap();
    let mut history = history.write().unwrap();
    // Convert AtomicBool to a mutable reference for the search
    // We'll use a wrapper to check the flag
    // let final_move = search_node.iterative_deepening_threaded(MAX_DEPTH, net, table, on_search);
    let acc = net.accumulator(game);
    let final_move = iterative_deepening_threaded(
        game,
        &acc,
        MAX_DEPTH,
        net,
        &mut *table,
        on_search,
        zobrist,
        &mut *killers,
        &mut *history,
    );

    log::info!("Ended go");

    match final_move {
        Some(m) => {
            log::info!("Found Move: {:?}", m);
            log::info!("Move uci: {:?}", m.to_uci());
            println!("bestmove {}", m.to_uci());
            Some(m)
        }
        None => {
            // println!("Dindnt found a moveeeeeeeeeeeeee");
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut nnue_filename = env::current_exe().expect("Failed to get current executable path");
    println!("env: {}", nnue_filename.to_str().unwrap());
    nnue_filename.pop();
    nnue_filename.push("nn-47fc8b7fff06.nnue");

    let nnue_filename = nnue_filename.to_str().unwrap();

    // Download the specific version nnue-rs expects if it isn't here yet
    if !Path::new(nnue_filename).exists() {
        log::info!("NNUE file not found, downloading...");
        let mut res =
            reqwest::blocking::get("https://tests.stockfishchess.org/api/nn/nn-47fc8b7fff06.nnue")?;
        let mut file = File::create(nnue_filename)?;
        res.copy_to(&mut file)?;
    }

    let net = Arc::new(Network::from_file(nnue_filename).unwrap());
    // let logger = Logger::try_with_str(spec)
    let logger = Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default())
        .duplicate_to_stdout(Duplicate::Info)
        .start()
        .unwrap();
    log::info!("ENgine started");
    let net = Arc::new(
        Network::from_file("/home/eric/Projects/chessbot/rust/src/nn-47fc8b7fff06.nnue").unwrap(),
    );
    let table = Arc::new(RwLock::new(TranspositionTable::new()));
    let killers = Arc::new(RwLock::new(KillerMoves::new()));
    let history = Arc::new(RwLock::new(HistoryTable::new()));
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

                let table_clone = Arc::clone(&table);
                let killers_clone = Arc::clone(&killers);
                let history_clone = Arc::clone(&history);
                let on_search_clone = Arc::clone(&on_search);

                let zobrist_clone = Arc::clone(&zobrist); // clone for thread
                // Spawn search thread
                search_thread = Some(thread::spawn(move || {
                    let result = handle_go_threaded(
                        &mut game_clone,
                        &net_clone,
                        table_clone,
                        &on_search_clone,
                        &*zobrist_clone,
                        killers_clone,
                        history_clone,
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
                return Ok(());
            }
            _ => {} // Ignore unknown commands
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Adjust imports based on where you place this
    use chess_engine::game::Game;
    use nnue_rs::{Board, Network};

    #[test]
    fn test_nnue_perspective() {
        // 1. Load your network
        let net = Network::from_file("/home/eric/Projects/chessbot/rust/src/nn-47fc8b7fff06.nnue")
            .unwrap();

        // 2. Set up a FEN where Black is completely winning (White is missing their Queen)
        // Notice the 'w' - it is White's turn to move.
        let mut game_white_to_move = Game::from_fen(String::from(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1",
        ));
        let acc_w = net.accumulator(&game_white_to_move);
        let eval_w = net.evaluate_accumulator(&acc_w, game_white_to_move.side_to_move());

        // 3. Set up the exact same FEN, but change the side to move to Black ('b')
        let mut game_black_to_move = Game::from_fen(String::from(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR b KQkq - 0 1",
        ));
        let acc_b = net.accumulator(&game_black_to_move);
        let eval_b = net.evaluate_accumulator(&acc_b, game_black_to_move.side_to_move());

        println!("--- NNUE Perspective Test ---");
        println!("Black is up a Queen in both positions.");
        println!("Raw Eval (White's turn): {}", eval_w);
        println!("Raw Eval (Black's turn): {}", eval_b);
        println!("-----------------------------");

        // We can safely assume Black is winning here.
        // Let's analyze what the numbers mean:
        if eval_w < 0 && eval_b < 0 {
            println!("Result: Your NNUE uses WHITE'S PERSPECTIVE.");
            println!("(You MUST flip the score in Negamax when it is Black's turn)");
        } else if eval_w < 0 && eval_b > 0 {
            println!("Result: Your NNUE uses SIDE-TO-MOVE PERSPECTIVE.");
            println!("(Do not flip the evaluation score in Negamax)");
        } else {
            println!("Result: Something is deeply wrong with the evaluation function!");
        }
    }

    #[test]
    fn test_zobrist_side_to_move() {
        let zobrist = Zobrist::new();

        // 1. Starting position (White's turn)
        let mut game_w = Game::from_fen(String::from(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ));
        let hash_w = game_w.compute_hash(&zobrist);

        // 2. Exact same board, but Black's turn
        let mut game_b = Game::from_fen(String::from(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        ));
        let hash_b = game_b.compute_hash(&zobrist);

        println!("--- Zobrist Hash Test ---");
        println!("White to move hash: {}", hash_w);
        println!("Black to move hash: {}", hash_b);
        println!("-------------------------");

        if hash_w == hash_b {
            println!("CRITICAL BUG: Your hashes are identical!");
            println!("Black is using White's evaluations from the Transposition Table.");
        } else {
            println!("PASS: Hashes are different. Side-to-move is working in your FEN parser.");

            // Step 3: Let's test if make/unmake move correctly toggles the hash
            let original_hash = game_w.compute_hash(&zobrist);
            let moves = game_w.get_legal_moves();

            // Just apply and unmake the first legal move
            game_w._apply_move(moves[0], &zobrist);
            let hash_after_move = game_w.hash.unwrap();

            game_w.unmake_move();
            let hash_after_unmake = game_w.hash.unwrap_or(game_w.compute_hash(&zobrist));

            if original_hash != hash_after_unmake {
                println!("CRITICAL BUG: unmake_move() does not restore the original hash!");
            } else {
                println!("PASS: make/unmake move correctly restores the hash.");
            }
        }
    }
}
