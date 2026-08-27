use crate::enums::Colorr::WHITE;
use crate::game::Game;
use crate::moves::Move;
use nnue_rs::Color::White;
use nnue_rs::{Accumulator, Board, Network};
use rand::{Rng, RngExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::time::Instant;

const NNUE_SORT: bool = false;
const QUIESCENCE_MAX_DEPTH: usize = 20;
const TT_SIZE: usize = 1 << 22; // ~4 million entries

#[derive(PartialEq, Eq, Clone)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: usize,
    pub score: i32,
    pub best_move: Option<Move>,
    pub flag: TTFlag,
    pub is_pv: bool,
}

#[derive(Clone)]
pub struct TranspositionTable {
    pub entries: Vec<TTEntry>,
}
impl TranspositionTable {
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (TT_SIZE - 1)
    }

    fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let entry = &self.entries[self.index(hash)];
        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }

    fn store(&mut self, hash: u64, entry: TTEntry) {
        let idx = self.index(hash);
        // replacement: always store if new depth >= existing depth
        if entry.depth >= self.entries[idx].depth {
            self.entries[idx] = entry;
        }
    }
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            entries: vec![
                TTEntry {
                    hash: 0,
                    depth: 0,
                    score: 0,
                    best_move: None,
                    flag: TTFlag::Exact,
                    is_pv: false
                };
                TT_SIZE
            ],
        }
    }
}

pub fn sort_moves(
    game: &mut Game,
    moves_: Vec<Move>,
    parent: &Game,
    parent_acc: &Accumulator,
    tt_move: Option<Move>,
    net: &Network,
    zobrist: &Zobrist,
    killers: &KillerMoves,
    history: &HistoryTable,
    ply: usize,
) -> Vec<Move> {
    let mut move_scores: Vec<(usize, i32)> = Vec::with_capacity(moves_.len());

    for (i, &move_) in moves_.iter().enumerate() {
        // If TT entry exists and has a best move, try it first
        if Some(move_) == tt_move {
            // Give TT move maximum possible priority to ensure it sorts first
            move_scores.push((i, i32::MAX));
            continue;
        }

        // Killer Moves
        // First killer
        if Some(move_) == killers.killers[ply][0] {
            move_scores.push((i, 500_000));
            continue;
        }

        // Second killer
        if Some(move_) == killers.killers[ply][1] {
            move_scores.push((i, 400_000));
            continue;
        }

        // History heuristic (for non-captures)
        if move_.captured.is_none() {
            let color = game.side_to_move() as usize;
            let from = move_.from_pos.to_index() as usize;
            let to = move_.to_pos.to_index() as usize;

            let score = 100_000 + history.get(color, from, to);
            move_scores.push((i, score));
            continue;
        }

        // MVV-LVA
        if let Some(captured_piece) = move_.captured {
            let victim = captured_piece.piece_type.get_value() * 100;
            let attacker = move_.piece.piece_type.get_value() * 100;
            let mvv_lva_score = victim as i32 * 10 - attacker as i32;
            move_scores.push((i, mvv_lva_score));
            continue;
        }

        // If not capture NNUE
        if NNUE_SORT {
            game._apply_move(move_, zobrist);
            let mut acc = net.empty_accumulator();
            net.update(parent, game, parent_acc, &mut acc);

            let score = net.evaluate_accumulator(&acc, parent.side_to_move());
            move_scores.push((i, score));

            game.unmake_move();
            continue;
        }

        move_scores.push((i, 0));
    }

    move_scores.sort_by(|a, b| b.1.cmp(&a.1));

    // Reorder moves based on sorted indices
    let sorted_moves: Vec<Move> = move_scores.iter().map(|&i| moves_[i.0]).collect();
    sorted_moves
}

pub fn sort_moves_nnue(
    game: &mut Game,
    moves_: Vec<Move>,
    parent: &Game,
    parent_acc: &Accumulator,
    tt_move: Option<Move>,
    net: &Network,
    zobrist: &Zobrist,
) -> Vec<Move> {
    let mut move_scores: Vec<(usize, i32)> = Vec::with_capacity(moves_.len());

    for (i, &move_) in moves_.iter().enumerate() {
        if Some(move_) == tt_move {
            // Give TT move maximum possible priority to ensure it sorts first
            move_scores.push((i, i32::MAX));
            continue;
        }

        game._apply_move(move_, zobrist);
        let mut acc = net.empty_accumulator();
        net.update(parent, game, parent_acc, &mut acc);

        let score = net.evaluate_accumulator(&acc, parent.side_to_move());
        move_scores.push((i, score));

        game.unmake_move();
    }

    move_scores.sort_by(|a, b| b.1.cmp(&a.1));

    // Reorder moves based on sorted indices
    let sorted_moves: Vec<Move> = move_scores.iter().map(|&i| moves_[i.0]).collect();
    sorted_moves
}

pub fn iterative_deepening_threaded(
    game: &mut Game,
    acc: &Accumulator,
    max_depth: usize,
    net: &Network,
    table: &mut TranspositionTable,
    on_search: &Arc<AtomicBool>,
    zobrist: &Zobrist,
    killers: &mut KillerMoves,
    history: &mut HistoryTable,
) -> Option<Move> {
    // Ensure hash is initialised
    if game.hash.is_none() {
        game.hash = Some(game.compute_hash(zobrist));
    }

    let mut best_move = None;
    let mut best_score = -std::i32::MAX;

    let initial_search_time = Instant::now();
    let mut nodes: usize = 0;
    let mut max_sel_depth = 0;

    for depth in 1..=max_depth {
        // Check if we should stop
        if !on_search.load(Ordering::SeqCst) {
            log::info!("Search stopped at depth {}", depth);
            break;
        }

        let mut sel_depth = 0;
        let score = alpha_beta_threaded(
            game,
            acc,
            depth,
            0,
            -std::i32::MAX,
            std::i32::MAX,
            net,
            table,
            &mut nodes,
            &mut sel_depth,
            on_search,
            zobrist,
            true,
            killers,
            history,
        );

        if !on_search.load(Ordering::SeqCst) {
            break;
        }

        if sel_depth > max_sel_depth {
            max_sel_depth = sel_depth;
        }

        let hash = game.hash.unwrap(); // must be Some
        if let Some(entry) = table.probe(hash) {
            if entry.depth >= depth {
                best_score = score;
                best_move = entry.best_move;
                log::info!(
                    "Depth {}: score={}, move={:?}",
                    depth,
                    best_score,
                    best_move
                );

                let elapsed = initial_search_time.elapsed().as_millis() as usize;
                let elapsed_fixed = if elapsed == 0 { 1 } else { elapsed };
                let nps = (nodes * 1000) / elapsed_fixed;
                println!(
                    "info depth {} seldepth {} score cp {} nodes {} nps {} time {} pv {}",
                    depth,
                    max_sel_depth,
                    score,
                    nodes,
                    nps,
                    elapsed,
                    best_move.unwrap().to_uci()
                );
            }
        }
    }

    best_move
}

pub fn alpha_beta_threaded(
    game: &mut Game,
    parent_acc: &Accumulator,
    depth: usize,
    ply: usize,
    mut alpha: i32,
    beta: i32,
    net: &Network,
    table: &mut TranspositionTable,
    nodes: &mut usize,
    sel_depth: &mut usize,
    on_search: &Arc<AtomicBool>,
    zobrist: &Zobrist,
    is_pv: bool,
    killers: &mut KillerMoves,
    history: &mut HistoryTable,
) -> i32 {
    let alpha_orig = alpha;
    log::debug!(
        "alpha_beta called: depth={}, alpha={}, beta={}",
        depth,
        alpha,
        beta
    );
    // Check if we should stop
    if !on_search.load(Ordering::SeqCst) {
        // return self.score;
        return 0;
    }

    log::info!("Node searched, depth: {}", depth);

    *nodes += 1;

    let hash = game.hash.unwrap(); // must be Some

    if depth == 0 {
        let score = if game.is_last_move_capture() {
            *sel_depth = 0;

            quiscence_threaded(
                game, parent_acc, alpha, beta, net, table, nodes, sel_depth, on_search, zobrist,
            )
        } else {
            log::info!("Evaluating accumulator");
            net.evaluate_accumulator(parent_acc, game.side_to_move())
        };

        let ttentry = TTEntry {
            hash: hash,
            depth: 0,
            score: score,
            best_move: None,
            flag: TTFlag::Exact,
            is_pv: is_pv,
        };

        table.store(hash, ttentry);
        return score;
    }

    if game.is_threefold_repetition() || game.is_stalemate() {
        return 0;
    }

    log::info!("Checking hash");
    // Check TT
    let mut tt_move: Option<Move> = None;
    if let Some(entry) = table.probe(hash) {
        tt_move = entry.best_move; // Extract cached best move for ordering

        if entry.depth >= depth {
            if entry.flag == TTFlag::Exact {
                return entry.score;
            }
            if entry.flag == TTFlag::LowerBound && entry.score >= beta {
                return beta;
            }
            if entry.flag == TTFlag::UpperBound && entry.score <= alpha {
                return alpha;
            }
        }
    }

    log::info!("Checked hash");

    // Check again before generating children
    if !on_search.load(Ordering::SeqCst) {
        // return self.score;
        return 0;
    }

    log::info!("Move gens...");
    let parent = game.clone();
    let mut best_score = -i32::MAX;
    let mut best_move: Option<Move> = None;

    // Sort moves
    // let sorted_moves = if NNUE_SORT {
    //     sort_moves_nnue(game, moves_, &parent, parent_acc, tt_move, net, zobrist)
    // } else {
    //     sort_moves(game, moves_, &parent, parent_acc, tt_move, net, zobrist)
    // };

    let sorted_moves = sort_moves(
        game,
        game.get_legal_moves(),
        &parent,
        parent_acc,
        tt_move,
        net,
        zobrist,
        killers,
        history,
        ply,
    );
    let side_before = parent.turn;

    let mut first_move = true;
    for move_ in sorted_moves {
        log::info!("Move: {}", &move_.to_uci());
        log::info!(
            "Move history: {:?}",
            game.moves
                .iter()
                .map(|m| m.to_uci())
                .collect::<Vec<String>>()
        );
        log::info!("Started Move applying");
        // Check before each recursive call
        if !on_search.load(Ordering::SeqCst) {
            break;
        }

        log::info!("Before move: {}", move_.to_uci());
        game.board.print();

        game._apply_move(move_, zobrist);

        let mut acc = net.empty_accumulator();
        net.update(&parent, game, parent_acc, &mut acc);

        let mut score: i32;

        if first_move {
            // First move: full window search (PV)
            score = -alpha_beta_threaded(
                game,
                &acc,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                net,
                table,
                nodes,
                sel_depth,
                on_search,
                zobrist,
                true,
                killers,
                history,
            );
            first_move = false;
        } else {
            // Try a null-window search first (faster)
            score = -alpha_beta_threaded(
                game,
                &acc,
                depth - 1,
                ply + 1,
                -alpha - 1, // Null window: (alpha, alpha+1)
                -alpha,
                net,
                table,
                nodes,
                sel_depth,
                on_search,
                zobrist,
                false,
                killers,
                history,
            );

            // If the null-window search fails (score > alpha), re-search with full window
            if score > alpha && score < beta {
                score = -alpha_beta_threaded(
                    game,
                    &acc,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    net,
                    table,
                    nodes,
                    sel_depth,
                    on_search,
                    zobrist,
                    true,
                    killers,
                    history,
                );
            }
        }

        log::info!("After move: {:?}", move_);
        game.board.print();

        game.unmake_move();

        log::info!("After unmake move: {}", move_.to_uci());
        game.board.print();

        if score > best_score {
            best_score = score;
            // best_move = parent.last_move().cloned();
            best_move = Some(move_);
        }

        if score >= beta {
            if move_.captured.is_none() {
                let color = side_before as usize;
                let from = move_.from_pos.to_index() as usize;
                let to = move_.to_pos.to_index() as usize;
                let bonus = depth * depth; // Deeper cuts get bigger bonus
                history.update(color, from, to, bonus as i32);
            }

            // Update killers
            killers.store(ply, move_);

            let ttentry = TTEntry {
                hash: hash,
                depth: depth,
                score: beta,
                best_move: best_move,
                flag: TTFlag::LowerBound,
                is_pv: is_pv,
            };
            table.store(hash, ttentry);
            return beta;
        }

        if score > alpha {
            alpha = score;
        }

        log::info!("Ended Move applying");
    }

    if best_move.is_none() {
        let score = if parent.is_checkmate() {
            -30000 - depth as i32
        } else if parent.is_draw() {
            0
        } else {
            0
        };

        let ttentry = TTEntry {
            hash: hash,
            depth: depth,
            score: score,
            best_move: None,
            flag: TTFlag::Exact,
            is_pv: is_pv,
        };

        table.store(hash, ttentry);
        return score;
    }

    let flag = if best_score <= alpha_orig {
        TTFlag::UpperBound // Failed low: score is at most best_score
    } else if best_score >= beta {
        TTFlag::LowerBound // Failed high: score is at least best_score
    } else {
        TTFlag::Exact // True minimax score within (alpha, beta)
    };

    let ttentry = TTEntry {
        hash: hash,
        depth: depth,
        score: best_score,
        best_move: best_move,
        flag: flag,
        is_pv: is_pv,
    };

    table.store(hash, ttentry);
    best_score
}

pub fn quiscence_threaded(
    game: &mut Game,
    parent_acc: &Accumulator,
    mut alpha: i32,
    beta: i32,
    net: &Network,
    table: &mut TranspositionTable,
    nodes: &mut usize,
    sel_depth: &mut usize,
    on_search: &Arc<AtomicBool>,
    zobrist: &Zobrist,
) -> i32 {
    if *sel_depth > QUIESCENCE_MAX_DEPTH {
        return net.evaluate_accumulator(parent_acc, game.side_to_move());
    }

    // Check if we should stop
    if !on_search.load(Ordering::SeqCst) {
        return 0;
    }

    log::info!("Quiscence");
    log::info!("Quiscence Depth: {}", sel_depth);

    *nodes += 1;

    let stand_pat = net.evaluate_accumulator(&parent_acc, game.side_to_move());

    if stand_pat >= beta {
        return beta;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    if !on_search.load(Ordering::SeqCst) {
        return alpha;
    }

    let parent_game = game.clone();
    let children = game.get_legal_captures();

    for capture in children {
        if !on_search.load(Ordering::SeqCst) {
            break;
        }

        *sel_depth += 1;

        game._apply_move(capture, zobrist);
        let mut child_acc = net.empty_accumulator();
        net.update(&parent_game, game, parent_acc, &mut child_acc);

        let score = -quiscence_threaded(
            game, &child_acc, -beta, -alpha, net, table, nodes, sel_depth, on_search, zobrist,
        );

        game.unmake_move();

        *sel_depth -= 1;

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

pub struct Zobrist {
    // [piece_index][square_index] – you can flatten or use 2D
    pub piece_square: [[u64; 64]; 12], // 12 piece types (6 white + 6 black)
    pub side_to_move: u64,
    pub castling: [u64; 16],   // 4 bits -> 16 combinations
    pub en_passant: [u64; 64], // one per file (or per square, but only file matters)
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut table = Zobrist {
            piece_square: [[0; 64]; 12],
            side_to_move: rng.random(),
            castling: [0; 16],
            en_passant: [0; 64],
        };
        for piece in 0..12 {
            for sq in 0..64 {
                table.piece_square[piece][sq] = rng.random();
            }
        }
        for i in 0..16 {
            table.castling[i] = rng.random();
        }
        for i in 0..64 {
            table.en_passant[i] = rng.random();
        }
        table
    }
}

pub struct KillerMoves {
    // Store 2 killer moves for each ply (depth)
    // ply 0 = root, ply 1 = first move, ply 2 = second move, etc.
    killers: [[Option<Move>; 2]; 128], // Max depth 128
}

impl KillerMoves {
    pub fn new() -> Self {
        KillerMoves {
            killers: [[None, None]; 128],
        }
    }

    // Store a move that caused a beta cutoff
    pub fn store(&mut self, ply: usize, move_: Move) {
        // Don't store if it's already the first killer
        if Some(move_) == self.killers[ply][0] {
            return;
        }

        // Shift existing killers: new move becomes first
        self.killers[ply][1] = self.killers[ply][0];
        self.killers[ply][0] = Some(move_);
    }

    // Get killer moves for a ply
    pub fn get(&self, ply: usize) -> [Option<Move>; 2] {
        self.killers[ply]
    }

    // Check if a move is a killer
    pub fn is_killer(&self, ply: usize, move_: Move) -> bool {
        Some(move_) == self.killers[ply][0] || Some(move_) == self.killers[ply][1]
    }
}

pub struct HistoryTable {
    // [color][from_square][to_square]
    // Color: 0 = white, 1 = black
    table: [[[i32; 64]; 64]; 2],
}

impl HistoryTable {
    pub fn new() -> Self {
        HistoryTable {
            table: [[[0; 64]; 64]; 2],
        }
    }

    // Update when a move causes a beta cutoff
    pub fn update(&mut self, color: usize, from: usize, to: usize, bonus: i32) {
        // Bonus is usually depth * depth (deeper cuts get bigger bonuses)
        let new_score = self.table[color][from][to] + bonus;

        // Prevent overflow
        if new_score > 1_000_000 {
            // Decay all history values when they get too large
            self.decay();
        } else {
            self.table[color][from][to] = new_score;
        }
    }

    // Get history score for a move
    pub fn get(&self, color: usize, from: usize, to: usize) -> i32 {
        self.table[color][from][to]
    }

    // Decay all values to prevent overflow and give recent moves more weight
    fn decay(&mut self) {
        for color in 0..2 {
            for from in 0..64 {
                for to in 0..64 {
                    self.table[color][from][to] /= 2;
                }
            }
        }
    }
}
