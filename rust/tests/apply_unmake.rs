use chess_engine::game::Game;
use chess_engine::moves::Move;
use chess_engine::search::Zobrist;

#[test]
fn apply_unmake_all_legal_moves() {
    let zobrist = Zobrist::new();
    let mut game = Game::new();

    let initial_fen = game.get_fen();
    let initial_moves_len = game.moves.len();

    let legal = game.get_legal_moves();

    for m in legal {
        // apply
        game._apply_move(m, &zobrist);
        // unapply
        game.unmake_move();

        // after unmake, state should match initial
        assert_eq!(game.get_fen(), initial_fen, "FEN mismatch after apply/unmake for move {}", m.to_uci());
        assert_eq!(game.moves.len(), initial_moves_len, "Move stack length mismatch after apply/unmake");
    }
}
