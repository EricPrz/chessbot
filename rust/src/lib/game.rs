use crate::search::Zobrist;
use nnue_rs::{Board, Color, FenBoard, Network, Piece};
use std::vec;

use crate::{
    board::{self, Boardd},
    castling::CastlingRights,
    enums::{
        Colorr::{self, BLACK, WHITE},
        PieceType::{self, KING, PAWN, ROOK},
        Square, get_bitboard_from_square, get_squares_from_bitboard,
    },
    moves::Move,
    piece::Piecee,
};

#[derive(Debug, Clone)]
pub struct Game {
    pub board: Boardd,
    pub turn: Colorr,
    pub castling: CastlingRights,
    pub half_move_clock: u8,
    pub fullmove_clock: u8,
    pub moves: vec::Vec<Move>,
    pub hash_history: Vec<u64>,
    pub hash: Option<u64>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Boardd::new(),
            turn: WHITE,
            castling: CastlingRights::new(),
            half_move_clock: 0,
            fullmove_clock: 1,
            moves: Vec::new(),
            hash_history: Vec::new(),
            hash: None,
        }
    }

    pub fn init_hash(&mut self, zobrist: &Zobrist) {
        self.hash = Some(self.compute_hash(zobrist));
    }

    pub fn compute_hash(&self, zobrist: &Zobrist) -> u64 {
        let mut h = 0;
        for sq in 0..64 {
            if let Some(piece) = self
                .board
                .get_piece_at_square(Square::square_from_number(sq as u8))
            {
                let idx = piece.piece_index();
                h ^= zobrist.piece_square[idx][sq];
            }
        }
        if self.turn == BLACK {
            // use self.turn, not self.side_to_move()
            h ^= zobrist.side_to_move;
        }
        // castling rights
        let mask = self.castling_mask();
        h ^= zobrist.castling[mask as usize];
        // en passant square
        if let Some(ep_sq) = self.en_passant_square_index() {
            h ^= zobrist.en_passant[ep_sq as usize];
        }
        h
    }
    pub fn last_move(&self) -> Option<&Move> {
        self.moves.last()
    }

    fn castling_mask(&self) -> u8 {
        let mut mask = 0;
        if self.castling.white_kingside {
            mask |= 1;
        }
        if self.castling.white_queenside {
            mask |= 2;
        }
        if self.castling.black_kingside {
            mask |= 4;
        }
        if self.castling.black_queenside {
            mask |= 8;
        }
        mask
    }

    /// Returns the square index (0‑63) of the en‑passant target square, or `None` if none exists.
    fn en_passant_square_index(&self) -> Option<u8> {
        let ep_bb = self.board.get_en_passant_bitboard();
        if ep_bb == 0 {
            None
        } else {
            Some(ep_bb.trailing_zeros() as u8) // correct bit index
        }
    }

    pub fn is_last_move_capture(&self) -> bool {
        if let Some(move_) = self.moves.last() {
            return move_.captured.is_some();
        }
        false
    }

    pub fn from_fen(fen: String) -> Self {
        let mut game = Self::new();

        let fen_string: Vec<&str> = fen.split(" ").collect();

        game.board = Boardd::from_fen(
            fen_string
                .get(0)
                .expect("FEN should have board info")
                .to_string(),
        );

        let color_char = fen_string.get(1).expect("FEN should have turn info");
        game.turn = Colorr::from_char(color_char);

        let castling_string = fen_string
            .get(2)
            .expect("FEN should have castling rights info");
        game.castling = CastlingRights::from_string(castling_string.to_string());

        let en_passant = fen_string
            .get(3)
            .expect("FEN should have info about en passant");
        if en_passant != &"-" {
            let en_passant_square = Square::from_uci(en_passant);
            // println!("En passant square: {}", en_passant_square.to_uci());
            let en_passant_square_num = en_passant_square.to_index();
            let mut en_passant_bitboard = game.board.get_mutable_en_passant();
            *en_passant_bitboard = 1u64 << (en_passant_square_num);
        }

        game.half_move_clock = fen_string
            .get(4)
            .expect("FEN should have info about half move clock")
            .to_string()
            .parse()
            .expect("Half move clock in FEN should be a number");
        game.fullmove_clock = fen_string
            .get(5)
            .expect("FEN should have info about fullmove clock")
            .to_string()
            .parse()
            .expect("Full move clock in FEN should be a number");

        game
    }

    pub fn get_legal_captures(&self) -> Vec<Move> {
        self._generate_pseudo_legal_moves()
            .into_iter()
            .filter(|m| self._is_legal_move(m) && m.captured.is_some())
            .collect()
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        self._generate_pseudo_legal_moves()
            .into_iter()
            .filter(|m| self._is_legal_move(m))
            .collect()
    }

    pub fn get_legal_moves_uci(&self) -> Vec<String> {
        self.get_legal_moves()
            .into_iter()
            .map(|m| m.to_uci())
            .collect()
    }

    pub fn _generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut pseudo_legal_moves = Vec::new();

        for piece_type in PieceType::iter() {
            // println!("Generating Moves for Piece: {}", piece_type.to_char());
            let piece = Piecee::new(piece_type, self.turn);
            let moves = piece.get_pseudo_legal_moves(&self);
            // for move_ in &moves {
            // println!("Move: {}", move_.to_uci());
            // }
            pseudo_legal_moves.extend(moves);
        }

        pseudo_legal_moves
    }

    fn _is_legal_move(&self, move_: &Move) -> bool {
        if !move_.from_pos.is_valid() || !move_.to_pos.is_valid() {
            return false;
        }

        // Create a copy of the board
        let mut game_copy = self.clone();

        // Make the move on the copy
        let piece = game_copy.board.get_piece_at_square(move_.from_pos);
        if piece.is_none() {
            return false;
        }

        // Special handling for castling
        if move_.is_castle {
            if game_copy.board.is_attacked(
                game_copy.board.find_king(self.turn),
                self.turn.opposite(),
                &self,
            ) {
                return false;
            }
            // Move king
            game_copy.board.move_piece(move_.from_pos, move_.to_pos);

            // Move rook
            if move_.to_pos.get_col_index() > move_.from_pos.get_col_index() {
                game_copy.board.move_piece(
                    Square::new(7, move_.from_pos.get_row_index()),
                    Square::new(5, move_.from_pos.get_row_index()),
                );
            } else {
                game_copy.board.move_piece(
                    Square::new(0, move_.from_pos.get_row_index()),
                    Square::new(3, move_.from_pos.get_row_index()),
                );
            }
        } else {
            // Move piece
            game_copy.board.move_piece(move_.from_pos, move_.to_pos);

            // Handle en passant capture
            if move_.is_en_passant {
                let captured_pos =
                    Square::new(move_.to_pos.get_col_index(), move_.from_pos.get_row_index());
                game_copy
                    .board
                    .remove_piece_at_square(captured_pos, Piecee::new(PAWN, self.turn));
                // board_copy.set_piece(captured_pos, None)
            }
        }

        // Check if our king is in check after this move
        let king_pos = game_copy.board.find_king(self.turn);

        // return !board_copy.is_attacked(king_pos, self.turn.opposite(), &self);
        return !game_copy
            .board
            .is_attacked(king_pos, self.turn.opposite(), &game_copy);
    }

    pub fn get_children(&self, zobrist: &Zobrist) -> Vec<Game> {
        let moves = self.get_legal_moves();
        let mut children: Vec<Game> = Vec::new();

        for move_ in moves {
            let mut child = self.clone();
            child._apply_move(move_, zobrist);
            children.push(child);
        }

        children
    }

    pub fn unmake_move(&mut self) {
        // Safely pop the last move
        let last_move = match self.moves.pop() {
            Some(m) => m,
            None => return,
        };

        // Store the color of the player who made the move
        let mover_color = last_move.piece.color;

        // 1. Revert half-move clock using stored value
        self.half_move_clock = last_move.old_half_move_clock;

        // 2. Revert fullmove clock if the move was made by Black
        //    (fullmove clock increments after Black moves)
        if mover_color == BLACK {
            self.fullmove_clock -= 1;
        }

        // 3. Restore board state
        if last_move.is_castle {
            // Move king back
            self.board.move_piece(last_move.to_pos, last_move.from_pos);
            // Move rook back
            if last_move.to_pos.get_col_index() > last_move.from_pos.get_col_index() {
                // Kingside rook: from f-file (5) back to h-file (7)
                self.board.move_piece(
                    Square::new(5, last_move.from_pos.get_row_index()),
                    Square::new(7, last_move.from_pos.get_row_index()),
                );
            } else {
                // Queenside rook: from d-file (3) back to a-file (0)
                self.board.move_piece(
                    Square::new(3, last_move.from_pos.get_row_index()),
                    Square::new(0, last_move.from_pos.get_row_index()),
                );
            }
        } else {
            // Handle regular moves, promotions, captures, and en passant

            // First, handle promotion (if any)
            if let Some(promo_type) = last_move.promotion {
                // Remove the promoted piece (which is at to_pos)
                let promo_piece = Piecee::new(promo_type, mover_color);
                self.board
                    .remove_piece_at_square(last_move.to_pos, promo_piece);
                // Put the pawn back at from_pos
                self.board
                    .set_piece_at_square(last_move.from_pos, Piecee::new(PAWN, mover_color));
            } else {
                // Standard move: shift the piece back from to_pos to from_pos
                self.board.move_piece(last_move.to_pos, last_move.from_pos);
            }

            // Restore captured pieces (normal captures and en passant)
            if last_move.is_en_passant {
                // The captured pawn is on the square adjacent to the destination
                let captured_pos = Square::new(
                    last_move.to_pos.get_col_index(),
                    last_move.from_pos.get_row_index(),
                );
                // Restore the captured pawn (it was the opponent's pawn)
                self.board
                    .set_piece_at_square(captured_pos, Piecee::new(PAWN, mover_color.opposite()));
            } else if let Some(captured_piece) = last_move.captured {
                // Normal capture: put the captured piece back on the destination square
                self.board
                    .set_piece_at_square(last_move.to_pos, captured_piece);
            }
        }

        // 4. Restore castling rights and en passant square
        self.castling = last_move.old_castling_rights;
        *self.board.get_mutable_en_passant() = last_move.old_en_passant;

        // 5. Finally, switch the turn back to the player who moved before
        self.turn = self.turn.opposite();

        self.hash = self.hash_history.pop();
    }

    pub fn _apply_move(&mut self, move_: Move, zobrist: &Zobrist) {
        // Ensure hash is initialised
        if self.hash.is_none() {
            self.hash = Some(self.compute_hash(zobrist));
        }
        self.hash_history.push(self.hash.unwrap());

        let mut h = self.hash.unwrap();

        // --- Save old state for hash update ---
        let old_castling_mask = self.castling_mask();
        let old_ep = self.en_passant_square_index();

        // --- Piece movement deltas (based on the state BEFORE the move) ---

        let from_piece = self.board.get_piece_at_square(move_.from_pos);
        match from_piece {
            Some(p) => log::info!("Found {:?}", p),
            None => {
                log::info!("Didnt found piece at sqaure {}", move_.from_pos.to_uci());
                self.board.print();
            }
        }

        let from_piece = self.board.get_piece_at_square(move_.from_pos).unwrap();
        let from_idx = from_piece.piece_index();
        let from_sq = move_.from_pos.to_index() as usize;
        let to_sq = move_.to_pos.to_index() as usize;

        // Remove moving piece from source
        h ^= zobrist.piece_square[from_idx][from_sq];

        // Handle capture (regular or en‑passant)
        if let Some(captured) = move_.captured {
            let cap_idx = captured.piece_index();
            if move_.is_en_passant {
                // Captured pawn is on the square adjacent to the destination
                let ep_capture_sq =
                    Square::new(move_.to_pos.get_col_index(), move_.from_pos.get_row_index());
                let ep_cap_idx = ep_capture_sq.to_index() as usize;
                h ^= zobrist.piece_square[cap_idx][ep_cap_idx];
            } else {
                h ^= zobrist.piece_square[cap_idx][to_sq];
            }
        }

        // Handle castling rook movement
        if move_.is_castle {
            let rook_piece = Piecee::new(ROOK, self.turn);
            let rook_idx = rook_piece.piece_index();
            let (rook_from, rook_to) =
                if move_.to_pos.get_col_index() > move_.from_pos.get_col_index() {
                    // Kingside
                    (
                        Square::new(7, move_.from_pos.get_row_index()),
                        Square::new(5, move_.from_pos.get_row_index()),
                    )
                } else {
                    // Queenside
                    (
                        Square::new(0, move_.from_pos.get_row_index()),
                        Square::new(3, move_.from_pos.get_row_index()),
                    )
                };
            let rook_from_sq = rook_from.to_index() as usize;
            let rook_to_sq = rook_to.to_index() as usize;
            h ^= zobrist.piece_square[rook_idx][rook_from_sq];
            h ^= zobrist.piece_square[rook_idx][rook_to_sq];
        }

        // Add moving piece to destination (or promoted piece)
        let dest_idx = if let Some(promo) = move_.promotion {
            Piecee::new(promo, self.turn).piece_index()
        } else {
            from_idx
        };
        h ^= zobrist.piece_square[dest_idx][to_sq];

        // ----------------------------------------------------------------
        // --- Now apply the move to the board and state (original logic) ---
        // ----------------------------------------------------------------
        // Update halfmove clock
        if move_.get_piece().piece_type == PAWN || move_.captured.is_some() {
            self.half_move_clock = 0
        } else {
            self.half_move_clock += 1
        }

        // Update en passant
        if move_.get_piece().piece_type == PAWN
            && (move_
                .to_pos
                .get_row_index()
                .abs_diff(move_.from_pos.get_row_index())
                == 2)
        {
            let en_passant_square = Square::new(
                move_.from_pos.get_col_index(),
                (move_.from_pos.get_row_index() + move_.to_pos.get_row_index()) / 2,
            );
            *self.board.get_mutable_en_passant() = get_bitboard_from_square(en_passant_square);
        } else {
            *self.board.get_mutable_en_passant() = 0;
        }

        // Handle castling
        if move_.is_castle {
            // Move king
            self.board.move_piece(move_.from_pos, move_.to_pos);

            // Move rook
            // Kingside
            if move_.to_pos.get_col_index() > move_.from_pos.get_col_index() {
                self.board.move_piece(
                    Square::new(7, move_.from_pos.get_row_index()),
                    Square::new(5, move_.from_pos.get_row_index()),
                );
            } else {
                self.board.move_piece(
                    Square::new(0, move_.from_pos.get_row_index()),
                    Square::new(3, move_.from_pos.get_row_index()),
                );
            }

            // Update castling rights
            if move_.piece.color == WHITE {
                self.castling.white_kingside = false;
                self.castling.white_queenside = false;
            } else {
                self.castling.black_kingside = false;
                self.castling.black_queenside = false;
            }
        } else {
            // Handle en passant capture
            if move_.is_en_passant {
                let captured_pos =
                    Square::new(move_.to_pos.get_col_index(), move_.from_pos.get_row_index());
                self.board.remove_piece_at_square(
                    captured_pos,
                    Piecee::new(PAWN, move_.piece.color.opposite()),
                );
            }

            // Move piece
            self.board.move_piece(move_.from_pos, move_.to_pos);

            // Handle promotion
            if move_.promotion.is_some() {
                self.board.remove_piece_at_square(move_.to_pos, move_.piece);
                self.board.set_piece_at_square(
                    move_.to_pos,
                    Piecee::new(move_.promotion.expect("Promotion is some"), self.turn),
                );
            }

            // Update castling rights
            if move_.piece.piece_type == KING {
                if self.turn == WHITE {
                    self.castling.white_kingside = false;
                    self.castling.white_queenside = false;
                } else {
                    self.castling.black_kingside = false;
                    self.castling.black_queenside = false;
                }
            } else if move_.piece.piece_type == ROOK {
                // Queenside rook
                if move_.from_pos.get_col_index() == 0 {
                    if self.turn == WHITE && move_.from_pos.get_row_index() == 7 {
                        self.castling.white_queenside = false;
                    } else if self.turn == BLACK && move_.from_pos.get_row_index() == 0 {
                        self.castling.black_queenside = false;
                    }
                } else if move_.from_pos.get_col_index() == 7 {
                    // Kingside rook
                    if self.turn == WHITE && move_.from_pos.get_row_index() == 7 {
                        self.castling.white_kingside = false;
                    } else if self.turn == BLACK && move_.from_pos.get_row_index() == 0 {
                        self.castling.black_kingside = false;
                    }
                }
            }
        }

        // Switch turn
        self.turn = self.turn.opposite();

        // Update fullmove number
        if self.turn == WHITE {
            self.fullmove_clock += 1
        }

        self.moves.push(move_);

        // --- Now update hash for castling rights, en-passant, and side to move ---
        let new_castling_mask = self.castling_mask();
        let new_ep = self.en_passant_square_index();

        h ^= zobrist.castling[old_castling_mask as usize];
        h ^= zobrist.castling[new_castling_mask as usize];

        if let Some(ep) = old_ep {
            h ^= zobrist.en_passant[ep as usize];
        }
        if let Some(ep) = new_ep {
            h ^= zobrist.en_passant[ep as usize];
        }

        // Toggle side to move (this matches the switch above)
        h ^= zobrist.side_to_move;

        self.hash = Some(h);
    }

    pub fn is_threefold_repetition(&self) -> bool {
        let current_hash = match self.hash {
            Some(h) => h,
            None => return false,
        };

        let mut count = 1; // Current position counts as 1

        for &hash in self.hash_history.iter().rev() {
            if hash == current_hash {
                count += 1;
                // Not actually 3, due to problems on zobrist hashing on apply / unmake moves
                if count >= 2 {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_check(&self) -> bool {
        let king_pos = self.board.find_king(self.turn);

        return self
            .board
            .is_attacked(king_pos, self.turn.opposite(), &self);
    }

    pub fn is_checkmate(&self) -> bool {
        if !self.is_check() {
            return false;
        }
        return self.get_legal_moves().len() == 0;
    }

    pub fn is_stalemate(&self) -> bool {
        if self.is_check() {
            return false;
        }
        return self.get_legal_moves().len() == 0;
    }

    pub fn is_draw(&self) -> bool {
        self.half_move_clock >= 100 || self.is_stalemate() || self.is_threefold_repetition()
    }

    pub fn get_fen(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Board part
        parts.push(self.board.to_fen());

        // turn
        parts.push(self.turn.to_char().to_string());

        // castling rights
        parts.push(self.castling.to_string());

        // en passant
        let en_passant_bit = self.board.get_en_passant_bitboard();
        let square = get_squares_from_bitboard(&en_passant_bit);
        if square.len() == 1 {
            parts.push(
                square
                    .get(0)
                    .expect("Should be one en passant sqr")
                    .to_uci(),
            );
        } else {
            parts.push("-".to_string());
        }

        // half move
        parts.push(self.half_move_clock.to_string());

        // full move
        parts.push(self.fullmove_clock.to_string());

        parts.join(" ")
    }

    // pub fn get_pgn(&self) -> String {}

    // pub fn get_nnue_encoding(&self) {}
}

impl Board for Game {
    fn side_to_move(&self) -> Color {
        // whose turn it is
        if self.turn == Colorr::WHITE {
            Color::White
        } else {
            Color::Black
        }
    }

    fn king_square(&self, color: Color) -> u8 {
        // square (0-63) of `color`'s king
        let colorr = match color {
            Color::White => Colorr::WHITE,
            Color::Black => Colorr::BLACK,
        };

        let sqr = self.board.find_king(colorr).to_index();

        if sqr > 63 {
            log::info!("Sqr index greater than 63: {}", sqr);
            panic!("Error at puting sqr to network");
        }

        // println!("Idx: {}", self.board.find_king(colorr).to_index());
        63 - sqr
    }

    fn for_each_piece(&self, f: &mut dyn FnMut(u8, Piece)) {
        // call `f(square, piece)` for every piece on the board
        for piece_type in PieceType::iter() {
            for colorr in [Colorr::WHITE, Colorr::BLACK] {
                let piece = Piecee::new(piece_type, colorr);
                let bitboard = self.board.get_piece_bitboard(&piece);

                let sqrs = get_squares_from_bitboard(&bitboard);
                for sqr in sqrs {
                    if sqr.to_index() > 63 {
                        log::info!("Sqr index greater than 63: {}", sqr.to_index());
                        panic!("Error at puting sqr to network");
                    }
                    f(63 - sqr.to_index(), piece.to_nnue_piece())
                }
            }
        }
    }
}
