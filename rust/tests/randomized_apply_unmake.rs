use chess_engine::game::Game;
use chess_engine::search::Zobrist;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[test]
fn randomized_apply_unmake() {
    let zobrist = Zobrist::new();
    let mut rng = StdRng::seed_from_u64(0xC0FFEE_u64);

    // Run many random sequences of moves and ensure apply->unmake restores state
    for iter in 0..200 {
        let mut game = Game::new();
        let initial_fen = game.get_fen();
        let seq_len = (rng.next_u64() % 8) as usize + 1;
        let mut applied = Vec::new();
        let mut fens_after_apply: Vec<String> = Vec::new();

        for _ in 0..seq_len {
            let moves = game.get_legal_moves();
            if moves.is_empty() { break; }
            let idx = rng.next_u64() as usize % moves.len();
            let m = moves[idx];
            game._apply_move(m, &zobrist);
            applied.push(m);
            fens_after_apply.push(game.get_fen());
        }

        // unmake in LIFO order
        for _ in 0..applied.len() {
            game.unmake_move();
        }

        let final_fen = game.get_fen();
        if final_fen != initial_fen {
            eprintln!("---- Randomized failure ----");
            eprintln!("seed: 0xC0FFEE");
            eprintln!("iter: {}", iter);
            eprintln!("seq_len requested: {}", seq_len);
            eprintln!("initial_fen: {}", initial_fen);
            eprintln!("final_fen:   {}", final_fen);
            eprintln!("Applied moves (in order):");
            for (i, m) in applied.iter().enumerate() {
                eprintln!("  {}: {} -> {}    fen_after: {}", i, m.from_pos.to_uci(), m.to_pos.to_uci(), fens_after_apply.get(i).unwrap_or(&String::from("")));
            }

            // Replay the sequence separately and dump per-square piece info
            eprintln!("\nReplaying moves to inspect board state:");
            let mut replay = Game::new();
            for (i, m) in applied.iter().enumerate() {
                replay._apply_move(*m, &zobrist);
                eprintln!("After applying {}: FEN: {}", i, replay.get_fen());
            }

            eprintln!("\nBoard squares after apply (index: uci -> piece)");
            for idx in 0u8..64u8 {
                let sq = chess_engine::enums::Square::square_from_number(idx);
                let p = replay.board.get_piece_at_square(sq);
                if let Some(pc) = p {
                    eprintln!("  {}: {} -> {}", idx, sq.to_uci(), pc.to_char());
                }
            }

            eprintln!("\nNow unmaking on replay and dumping board:");
            for i in 0..applied.len() {
                replay.unmake_move();
                eprintln!("After unmake {}: FEN: {}", i, replay.get_fen());
            }

            panic!("FEN mismatch after random seq {}", iter);
        }
    }
}
