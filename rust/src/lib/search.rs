use crate::enums::Colorr::WHITE;
use crate::game::Game;
use crate::moves::Move;
use nnue_rs::{Accumulator, Board, Network};
use rand::{Rng, RngExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::time::Instant;

// pub fn get_children(&mut self, net: &Network, table: &mut TranspositionTable) -> Vec<SearchNode> {
//     let mut children = Vec::new();
//
//     for game_child in self.game.get_children() {
//         let fen = game_child.get_fen();
//         let mut child_acc = net.empty_accumulator();
//         net.update(&self.game, &game_child, &self.acc, &mut child_acc);
//
//         let score: i32 = if let Some(entry) = table.map.get(&fen) {
//             entry.score
//         } else {
//             let acc_eval = net.evaluate_accumulator(&child_acc, game_child.side_to_move());
//             // println!("Acc Eval: {}", acc_eval);
//             acc_eval
//         };
//
//         let new_node = SearchNode {
//             game: game_child,
//             acc: child_acc,
//             score: score,
//             depth: self.depth + 1,
//         };
//
//         if table.map.get(&fen).is_none() {
//             table.map.insert(fen.clone(), TTEntry::new(&new_node));
//         }
//
//         children.push(new_node);
//     }
//
//     let is_maxing = self.game.turn == WHITE;
//     match is_maxing {
//         true => children.sort_by(|a, b| b.score.cmp(&a.score)),
//         false => children.sort_by(|a, b| a.score.cmp(&b.score)),
//     }
//
//     // children.sort_by(|a, b| {
//     //     // Use move ordering heuristics instead
//     //     let a_score = self.move_ordering_score(&a.game.last_move().unwrap());
//     //     let b_score = self.move_ordering_score(&b.game.last_move().unwrap());
//     //     b_score.cmp(&a_score)
//     // });
//
//     children
// }
//
// pub fn get_children_captures(
//     &mut self,
//     net: &Network,
//     table: &mut TranspositionTable,
// ) -> Vec<SearchNode> {
//     let mut children = Vec::new();
//
//     for game_child in self.game.get_children() {
//         if let Some(l) = game_child.last_move() {
//             if l.captured.is_none() {
//                 continue;
//             }
//         }
//
//         let fen = game_child.get_fen();
//
//         let mut child_acc = net.empty_accumulator();
//         net.update(&self.game, &game_child, &self.acc, &mut child_acc);
//
//         let score: i32 = if let Some(entry) = table.map.get(&fen) {
//             entry.score
//         } else {
//             let acc_eval = net.evaluate_accumulator(&child_acc, game_child.side_to_move());
//             // println!("Acc Eval: {}", acc_eval);
//             acc_eval
//         };
//
//         let new_node = SearchNode {
//             game: game_child,
//             acc: child_acc,
//             score: score,
//             depth: self.depth + 1,
//         };
//
//         if table.map.get(&fen).is_none() {
//             table.map.insert(fen.clone(), TTEntry::new(&new_node));
//         }
//
//         children.push(new_node);
//     }
//
//     // children.sort_by(|a, b| {
//     //     // Use move ordering heuristics instead
//     //     let a_score = self.move_ordering_score(&a.game.last_move().unwrap());
//     //     let b_score = self.move_ordering_score(&b.game.last_move().unwrap());
//     //     b_score.cmp(&a_score)
//     // });
//
//     let is_maxing = self.game.turn == WHITE;
//     match is_maxing {
//         true => children.sort_by(|a, b| b.score.cmp(&a.score)),
//         false => children.sort_by(|a, b| a.score.cmp(&b.score)),
//     }
//
//     children
// }

#[derive(PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

pub struct TTEntry {
    pub hash: u64,
    pub depth: i32,
    pub score: i32,
    pub best_move: Option<Move>,
    pub flag: TTFlag,
}

impl TTEntry {
    // pub fn new(search_node: &SearchNode) -> TTEntry {
    //     TTEntry {
    //         hash: search_node.game.get_fen(),
    //         depth: search_node.depth,
    //         score: search_node.score,
    //         best_move: search_node.game.last_move().cloned(),
    //         flag: TTFlag::Exact,
    //     }
    // }
}

pub struct TranspositionTable {
    pub map: HashMap<u64, TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            map: HashMap::new(),
        }
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: i32,
        score: i32,
        best_move: Option<Move>,
        flag: TTFlag,
    ) {
        // Replacement policy: replace if deeper or if same depth with better flag type
        if let Some(entry) = self.map.get_mut(&hash) {
            if depth >= entry.depth {
                entry.depth = depth;
                entry.score = score;
                entry.best_move = best_move;
                entry.flag = flag;
            }
        } else {
            self.map.insert(
                hash,
                TTEntry {
                    hash,
                    depth,
                    score,
                    best_move,
                    flag,
                },
            );
        }
    }
}

pub fn iterative_deepening_threaded(
    game: &mut Game,
    acc: &Accumulator,
    max_depth: i32,
    net: &Network,
    table: &mut TranspositionTable,
    on_search: &Arc<AtomicBool>,
    zobrist: &Zobrist,
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
            -std::i32::MAX,
            std::i32::MAX,
            net,
            table,
            &mut nodes,
            &mut sel_depth,
            on_search,
            zobrist,
        );

        if !on_search.load(Ordering::SeqCst) {
            break;
        }

        if sel_depth > max_sel_depth {
            max_sel_depth = sel_depth;
        }

        let hash = game.hash.unwrap(); // must be Some
        if let Some(entry) = table.map.get(&hash) {
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
    depth: i32,
    mut alpha: i32,
    beta: i32,
    net: &Network,
    table: &mut TranspositionTable,
    nodes: &mut usize,
    sel_depth: &mut usize,
    on_search: &Arc<AtomicBool>,
    zobrist: &Zobrist,
) -> i32 {
    // Check if we should stop
    if !on_search.load(Ordering::SeqCst) {
        // return self.score;
        return 0;
    }

    println!("Node searched, depth: {}", depth);

    *nodes += 1;

    if depth == 0 {
        let score = if game.is_last_move_capture() {
            *sel_depth += 1;
            // let mut acc = net.empty_accumulator();
            // let parent = game.clone();
            // net.update(&parent, game, parent_acc, &mut acc);

            0
            // quiscence_threaded(
            //     game, parent_acc, alpha, beta, net, table, nodes, sel_depth, on_search, zobrist,
            // )
        } else {
            net.evaluate_accumulator(parent_acc, game.side_to_move())
        };

        // table.store(self, self.game.get_fen(), 0, score, None, TTFlag::Exact);
        return score;
    }

    println!("Checking hash");
    // Check TT
    let hash = game.hash.unwrap(); // must be Some
    if let Some(entry) = table.map.get(&hash) {
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

    println!("Checked hash");

    // Check again before generating children
    if !on_search.load(Ordering::SeqCst) {
        // return self.score;
        return 0;
    }

    println!("Move gens...");
    let moves_ = game.get_legal_moves();
    let parent = game.clone();
    let mut best_score = -i32::MAX;
    let mut best_move: Option<Move> = None;

    for move_ in &moves_ {
        println!("Legal Move: {}", move_.to_uci());
    }

    for move_ in moves_ {
        println!("Move: {}", &move_.to_uci());
        println!(
            "Move history: {:?}",
            game.moves
                .iter()
                .map(|m| m.to_uci())
                .collect::<Vec<String>>()
        );
        println!("Started Move applying");
        // Check before each recursive call
        if !on_search.load(Ordering::SeqCst) {
            break;
        }

        println!("Before move: {}", move_.to_uci());
        game.board.print();

        let old_hash = game.hash;
        game._apply_move(move_, zobrist);

        let mut acc = net.empty_accumulator();
        net.update(&parent, game, parent_acc, &mut acc);

        let score = -alpha_beta_threaded(
            game,
            &acc,
            depth - 1,
            -beta,
            -alpha,
            net,
            table,
            nodes,
            sel_depth,
            on_search,
            zobrist,
        );

        println!("After move: {}", move_.to_uci());
        game.board.print();

        game.unmake_move();
        game.hash = old_hash;

        println!("After unmake move: {}", move_.to_uci());
        game.board.print();

        if score > best_score {
            best_score = score;
            best_move = parent.last_move().cloned();
        }

        if score >= beta {
            table.store(hash, depth, beta, best_move, TTFlag::LowerBound);
            return beta;
        }

        if score > alpha {
            alpha = score;
        }

        println!("Ended Move applying");
    }

    if best_move.is_none() {
        let score = if parent.is_checkmate() {
            99999 + depth
        } else if parent.is_stalemate() {
            -5000
        } else if parent.is_draw() {
            -5000
        } else {
            0
        };

        // table.store(self, self.game.get_fen(), depth, score, None, TTFlag::Exact);
        return score;
    }

    table.store(hash, depth, best_score, best_move, TTFlag::Exact);

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
    // Check if we should stop
    if !on_search.load(Ordering::SeqCst) {
        // return self.score;
        return 0;
    }

    println!("Quiscence");
    // println!("Quiscence Depth: {}", sel_depth);

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

    let children = game.get_legal_captures();

    if !on_search.load(Ordering::SeqCst) {
        return alpha;
    }

    // Sort captures by MVV-LVA
    // children.sort_by(|a, b| {
    //     // Use MVV-LVA scoring
    //     let a_score = &self.mvv_lva_score(&a.game.last_move().unwrap());
    //     let b_score = &self.mvv_lva_score(&b.game.last_move().unwrap());
    //     b_score.cmp(&a_score)
    // });

    if children.len() > 0 {
        *sel_depth += 1;
    }

    let parent_game = game.clone();

    for capture in children {
        if !on_search.load(Ordering::SeqCst) {
            break;
        }

        let old_hash = game.hash;
        game._apply_move(capture, zobrist);
        let mut child_acc = net.empty_accumulator();
        net.update(&parent_game, game, parent_acc, &mut child_acc);

        let score = -quiscence_threaded(
            game, &child_acc, -beta, -alpha, net, table, nodes, sel_depth, on_search, zobrist,
        );

        game.unmake_move();
        game.hash = old_hash;

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

// fn mvv_lva_score(&self, m: &Move) -> usize {
//     // Most Valuable Victim - Least Valuable Attacker
//     let victim_value = match m.captured {
//         Some(p) => p.piece_type.get_value(),
//         None => 0,
//     } as usize;
//     let attacker_value = match self.game.board.get_piece_at_square(m.from_pos) {
//         Some(p) => p.piece_type.get_value(),
//         None => 0,
//     } as usize;
//     (victim_value * 10) - attacker_value
// }
//
// fn move_ordering_score(&self, m: &Move) -> usize {
//     let mut score = 0;
//
//     // Capture moves
//     if m.captured.is_some() {
//         // score += 5000 + self.mvv_lva_score(m);
//         score += self.mvv_lva_score(m);
//     }
//
//     // Killer moves (from history)
//     // score += self.killer_heuristic(m);
//
//     // History heuristic
//     // score += self.history_heuristic(m);
//
//     score
// }

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
