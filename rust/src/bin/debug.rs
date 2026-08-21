use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use chess_engine::game::Game;
use chess_engine::moves::Move;
use chess_engine::search::{TranspositionTable, Zobrist, iterative_deepening_threaded};
use flexi_logger::{FileSpec, Logger};
use nnue_rs::{Accumulator, Network};

fn main() {
    let zobrist = Zobrist::new();

    let mut game = Game::new();
    game.board.print();

    for move_ in game.get_legal_moves() {
        println!("Legal Move: {}", move_.to_uci());
    }

    let move_ = Move::from_uci("e2e4", &mut game).unwrap();
    game._apply_move(move_, &zobrist);

    game.board.print();

    game.unmake_move();

    game.board.print();

    // Run a simple depth-2 DFS (no NNUE) to reproduce illegal move generation
    fn dfs(game: &mut Game, depth: i32, zobrist: &Zobrist) {
        if depth == 0 {
            return;
        }

        let moves = game.get_legal_moves();
        for m in moves {
            // Quick sanity check: piece must exist at from_pos
            let piece_at_from = game.board.get_piece_at_square(m.from_pos);
            if piece_at_from.is_none() {
                println!("ILLEGAL GENERATED MOVE (no piece at from): {}", m.to_uci());
                continue;
            }

            let from_piece = piece_at_from.unwrap();
            if from_piece.piece_type != m.piece.piece_type || from_piece.color != m.piece.color {
                println!(
                    "MISMATCH GENERATED MOVE {} : expected piece {} at from, but move has {}",
                    m.to_uci(),
                    from_piece.to_char(),
                    m.piece.to_char()
                );
                continue;
            }

            println!("Applying move {} : piece at from = {}", m.to_uci(), from_piece.to_char());

            // Apply and recurse
            game._apply_move(m, zobrist);
            dfs(game, depth - 1, zobrist);
            game.unmake_move();
        }
    }

    // Check specifically for the illegal move "e1d2" at depth 2
    fn find_move(game: &mut Game, depth: i32, zobrist: &Zobrist, target: &str) -> bool {
        if depth == 0 {
            return false;
        }

        for m in game.get_legal_moves() {
            if m.to_uci() == target && m.from_pos.to_uci() == "e1" {
                println!("FOUND illegal move: {}", m.to_uci());
                return true;
            }

            game._apply_move(m, zobrist);
            let found = find_move(game, depth - 1, zobrist, target);
            game.unmake_move();
            if found {
                return true;
            }
        }

        false
    }

    let found = find_move(&mut game, 2, &zobrist, "d2");
    println!("Found e1d2 at depth 2? {}", found);
}
