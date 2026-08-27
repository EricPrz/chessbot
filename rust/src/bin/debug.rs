use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use chess_engine::game::Game;
use chess_engine::moves::Move;
use chess_engine::search::{TranspositionTable, Zobrist, iterative_deepening_threaded};
use flexi_logger::{Duplicate, FileSpec, Logger};
use nnue_rs::{Accumulator, Network};

const MAX_DEPTH: i32 = 15;

fn main() {
    let logger = Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default())
        .duplicate_to_stdout(Duplicate::Info)
        .start()
        .unwrap();

    let mut game = Game::new();
    let zobrist = Zobrist::new();

    let moves = vec!["c2c3", "a7a6", "d1a4", "d7d6"];

    for move_ in moves {
        game.board.print();
        let moves_from_pos = &game.get_legal_moves();

        for move_pos in moves_from_pos {
            log::info!("Moves generated at {}: {:?}", move_, move_pos);
        }

        game._apply_move(Move::from_uci(move_, &game).unwrap(), &zobrist);
    }
}
