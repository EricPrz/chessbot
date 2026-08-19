use crate::enums::Colorr::WHITE;
use crate::game::Game;
use crate::moves::Move;
use nnue_rs::{Accumulator, Board, Network};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::time::Instant;

#[derive(Clone)]
pub struct SearchNode {
    pub children: Vec<SearchNode>,
    pub game: Game,
    pub acc: Accumulator,
    pub score: i32,
    pub depth: i32,
}

impl SearchNode {
    pub fn new_root(game: &Game, net: &Network, table: &mut TranspositionTable) -> Self {
        let fen = game.get_fen();
        let score = net.evaluate(game);

        let new_node = SearchNode {
            children: Vec::new(),
            game: game.clone(),
            acc: net.accumulator(game),
            score: score,
            depth: 0,
        };

        table.map.insert(fen, TTEntry::new(&new_node));

        new_node
    }

    pub fn evaluate(&self, net: &Network) -> i32 {
        net.evaluate(&self.game)
    }

    pub fn get_children(&mut self, net: &Network, table: &mut TranspositionTable) {
        for game_child in self.game.get_children() {
            let fen = game_child.get_fen();
            let mut child_acc = net.empty_accumulator();
            net.update(&self.game, &game_child, &self.acc, &mut child_acc);

            let score: i32 = if let Some(entry) = table.map.get(&fen) {
                entry.score
            } else {
                let acc_eval = net.evaluate_accumulator(&child_acc, game_child.side_to_move());
                // println!("Acc Eval: {}", acc_eval);
                acc_eval
            };

            let new_node = SearchNode {
                children: Vec::new(),
                game: game_child,
                acc: child_acc,
                score: score,
                depth: self.depth + 1,
            };

            if table.map.get(&fen).is_none() {
                table.map.insert(fen.clone(), TTEntry::new(&new_node));
            }

            self.children.push(new_node);
        }

        let is_maxing = self.game.turn == WHITE;
        match is_maxing {
            true => self.children.sort_by(|a, b| b.score.cmp(&a.score)),
            false => self.children.sort_by(|a, b| a.score.cmp(&b.score)),
        }
    }

    pub fn get_children_captures(&mut self, net: &Network, table: &mut TranspositionTable) {
        for game_child in self.game.get_children() {
            if let Some(l) = game_child.last_move() {
                if l.captured.is_none() {
                    continue;
                }
            }

            let fen = game_child.get_fen();

            let mut child_acc = net.empty_accumulator();
            net.update(&self.game, &game_child, &self.acc, &mut child_acc);

            let score: i32 = if let Some(entry) = table.map.get(&fen) {
                entry.score
            } else {
                let acc_eval = net.evaluate_accumulator(&child_acc, game_child.side_to_move());
                println!("Acc Eval: {}", acc_eval);
                acc_eval
            };

            let new_node = SearchNode {
                children: Vec::new(),
                game: game_child,
                acc: child_acc,
                score: score,
                depth: self.depth + 1,
            };

            if table.map.get(&fen).is_none() {
                table.map.insert(fen.clone(), TTEntry::new(&new_node));
            }

            self.children.push(new_node);
        }

        let is_maxing = self.game.turn == WHITE;
        match is_maxing {
            true => self.children.sort_by(|a, b| b.score.cmp(&a.score)),
            false => self.children.sort_by(|a, b| a.score.cmp(&b.score)),
        }
    }

    pub fn quiscence(
        &mut self,
        mut alpha: i32,
        beta: i32,
        net: &Network,
        table: &mut TranspositionTable,
        nodes: &mut usize,
        sel_depth: &mut usize,
        on_search: &mut bool,
    ) -> i32 {
        if !*on_search {
            return self.score; // Return current evaluation
        }

        *nodes += 1;

        // Stand-pat: evaluate current position
        let stand_pat = self.score;

        // If stand-pat is already above beta, return beta (fail-high)
        if stand_pat >= beta {
            return beta;
        }

        // If stand-pat is better than alpha, update alpha
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        if !*on_search {
            return self.score; // Return current evaluation
        }

        // Generate only CAPTURE moves (and maybe checks)
        self.get_children_captures(net, table); // You'll need this method

        // Sort captures by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        // This helps with pruning
        self.children.sort_by(|a, b| {
            b.game
                .last_move()
                .unwrap()
                .captured
                .unwrap()
                .piece_type
                .get_value()
                .cmp(
                    &a.game
                        .last_move()
                        .unwrap()
                        .captured
                        .unwrap()
                        .piece_type
                        .get_value(),
                )
        });

        let mut children = std::mem::take(&mut self.children);
        if children.len() > 0 {
            *sel_depth += 1;
        }
        for capture_child in &mut children {
            if !*on_search {
                return self.score; // Return current evaluation
            }

            let score =
                -capture_child.quiscence(-beta, -alpha, net, table, nodes, sel_depth, on_search);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    // pub fn alpha_beta(
    //     &mut self,
    //     depth: i32,
    //     mut alpha: i32,
    //     beta: i32,
    //     net: &Network,
    //     table: &mut TranspositionTable,
    //     nodes: &mut usize,
    //     sel_depth: &mut usize,
    //     on_search: &mut bool,
    // ) -> i32 {
    //     // Check if we should stop searching
    //     if !*on_search {
    //         return self.score; // Return current evaluation
    //     }
    //
    //     *nodes += 1;
    //
    //     if depth == 0 {
    //         let score = if self.game.is_last_move_capture() {
    //             *sel_depth += 1;
    //             self.quiscence(alpha, beta, net, table, nodes, sel_depth, on_search)
    //         } else {
    //             self.score
    //         };
    //
    //         table.store(self, self.game.get_fen(), 0, score, None, TTFlag::Exact);
    //         return score;
    //     }
    //
    //     if let Some(entry) = table.map.get(&self.game.get_fen()) {
    //         if entry.depth >= depth {
    //             if entry.flag == TTFlag::Exact {
    //                 return entry.score;
    //             }
    //             if entry.flag == TTFlag::LowerBound && entry.score >= beta {
    //                 return beta;
    //             }
    //             if entry.flag == TTFlag::UpperBound && entry.score <= alpha {
    //                 return alpha;
    //             }
    //         }
    //     }
    //
    //     if !*on_search {
    //         return self.score; // Return current evaluation
    //     }
    //
    //     self.get_children(net, table);
    //     let mut best_score = -i32::MAX;
    //     let mut best_move: Option<Move> = None;
    //
    //     for mut child in self.children.clone() {
    //         if !*on_search {
    //             break;
    //         }
    //
    //         let score = -child.alpha_beta(
    //             depth - 1,
    //             -beta,
    //             -alpha,
    //             net,
    //             table,
    //             nodes,
    //             sel_depth,
    //             on_search,
    //         );
    //
    //         if score > best_score {
    //             best_score = score;
    //             best_move = child.game.last_move();
    //         }
    //
    //         if score >= beta {
    //             table.store(
    //                 self,
    //                 self.game.get_fen(),
    //                 depth,
    //                 beta,
    //                 best_move,
    //                 TTFlag::LowerBound,
    //             );
    //             return beta;
    //         }
    //
    //         if score > alpha {
    //             alpha = score;
    //         }
    //     }
    //
    //     if best_move.is_none() {
    //         let score = if self.game.is_checkmate() {
    //             99999 + depth
    //         } else if self.game.is_stalemate() {
    //             -1000
    //         } else if self.game.is_draw() {
    //             -1000
    //         } else {
    //             0
    //         };
    //
    //         table.store(self, self.game.get_fen(), depth, score, None, TTFlag::Exact);
    //         return score;
    //     }
    //
    //     table.store(
    //         self,
    //         self.game.get_fen(),
    //         depth,
    //         best_score,
    //         best_move,
    //         TTFlag::Exact,
    //     );
    //
    //     best_score
    // }
    //
    // pub fn iterative_deepening(
    //     &mut self,
    //     max_depth: i32,
    //     net: &Network,
    //     table: &mut TranspositionTable,
    //     on_search: &mut Arc<AtomicBool>,
    // ) -> Option<Move> {
    //     let mut best_move = None;
    //     let mut best_score = -std::i32::MAX;
    //
    //     let initial_search_time = Instant::now();
    //     let mut nodes: usize = 0;
    //     let mut max_sel_depth = 0;
    //
    //     for depth in 1..=max_depth {
    //         if !*on_search {
    //             break;
    //         }
    //
    //         let mut sel_depth = 0;
    //         // Search to this depth
    //         let score = self.alpha_beta(
    //             depth,
    //             -std::i32::MAX,
    //             std::i32::MAX,
    //             net,
    //             table,
    //             &mut nodes,
    //             &mut sel_depth,
    //             on_search,
    //         );
    //
    //         if sel_depth > max_sel_depth {
    //             max_sel_depth = sel_depth;
    //         }
    //
    //         // AFTER searching, get the best move from TT
    //         if let Some(entry) = table.map.get(&self.game.get_fen()) {
    //             if entry.depth >= depth {
    //                 best_score = score;
    //                 best_move = entry.best_move; // ← THE MOVE!
    //                 log::info!(
    //                     "Depth {}: score={}, move={:?}",
    //                     depth,
    //                     best_score,
    //                     best_move
    //                 );
    //                 let elapsed = initial_search_time.elapsed().as_millis() as usize;
    //                 let elpased_fixed = if elapsed == 0 { 1 } else { elapsed };
    //                 let nps = (nodes * 1000) / elpased_fixed;
    //                 println!(
    //                     "info depth {} seldepth {} score cp {} nodes {} nps {} time {} pv {}",
    //                     depth,
    //                     max_sel_depth,
    //                     score,
    //                     nodes,
    //                     nps,
    //                     elapsed,
    //                     best_move.unwrap().to_uci()
    //                 );
    //             }
    //         }
    //
    //         //if self.time_is_up() {
    //         //    break;
    //         //}
    //     }
    //
    //     best_move // Return the best move found
    // }

    // pub fn iddfs(
    //     &mut self,
    //     max_depth: i32,
    //     net: &Network,
    //     table: &mut TranspositionTable,
    // ) -> Vec<SearchNode> {
    //     let mut c = Vec::new();
    //     for depth in 1..=max_depth {
    //         c = self.dls(depth, net, table);
    //     }
    //     return c;
    // }
    //
    // pub fn dls(
    //     &mut self,
    //     depth: i32,
    //     net: &Network,
    //     table: &mut TranspositionTable,
    // ) -> Vec<SearchNode> {
    //     if depth == 0 {
    //         return vec![self.to_owned()];
    //     }
    //
    //     let mut results = Vec::new();
    //     let is_white = self.game.turn == WHITE;
    //     for mut child in self.get_children(net, table) {
    //         let mut sub_results = child.dls(depth - 1, net, table);
    //         results.append(&mut sub_results);
    //     }
    //
    //     results
    // }
}

#[derive(PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

pub struct TTEntry {
    pub hash: String,
    pub depth: i32,
    pub score: i32,
    pub best_move: Option<Move>,
    pub flag: TTFlag,
}

impl TTEntry {
    pub fn new(search_node: &SearchNode) -> TTEntry {
        TTEntry {
            hash: search_node.game.get_fen(),
            depth: search_node.depth,
            score: search_node.score,
            best_move: search_node.game.last_move(),
            flag: TTFlag::Exact,
        }
    }
}

pub struct TranspositionTable {
    pub map: HashMap<String, TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            map: HashMap::new(),
        }
    }

    pub fn store(
        &mut self,
        search_node: &SearchNode,
        hash: String,
        depth: i32,
        score: i32,
        best_move: Option<Move>,
        flag: TTFlag,
    ) {
        let is_empty = self.map.get(&hash).is_none();
        let mut entry = if is_empty {
            &mut TTEntry::new(&search_node)
        } else {
            self.map.get_mut(&hash).unwrap()
        };

        // Replacement strategy: replace if:
        // 1. Slot is empty (hash == 0)
        // 2. New depth is deeper
        // 3. Same depth but new is better (prefer EXACT > LOWER > UPPER)
        if is_empty || depth > entry.depth {
            entry.hash = hash;
            entry.depth = depth;
            entry.score = score;
            entry.best_move = best_move;
            entry.flag = flag;
        }
    }
}

impl SearchNode {
    pub fn iterative_deepening_threaded(
        &mut self,
        max_depth: i32,
        net: &Network,
        table: &mut TranspositionTable,
        on_search: &Arc<AtomicBool>,
    ) -> Option<Move> {
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
            let score = self.alpha_beta_threaded(
                depth,
                -std::i32::MAX,
                std::i32::MAX,
                net,
                table,
                &mut nodes,
                &mut sel_depth,
                on_search,
            );

            if !on_search.load(Ordering::SeqCst) {
                break;
            }

            if sel_depth > max_sel_depth {
                max_sel_depth = sel_depth;
            }

            if let Some(entry) = table.map.get(&self.game.get_fen()) {
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
        &mut self,
        depth: i32,
        mut alpha: i32,
        beta: i32,
        net: &Network,
        table: &mut TranspositionTable,
        nodes: &mut usize,
        sel_depth: &mut usize,
        on_search: &Arc<AtomicBool>,
    ) -> i32 {
        // Check if we should stop
        if !on_search.load(Ordering::SeqCst) {
            return self.score;
        }

        *nodes += 1;

        if depth == 0 {
            let score = if self.game.is_last_move_capture() {
                *sel_depth += 1;
                self.quiscence_threaded(alpha, beta, net, table, nodes, sel_depth, on_search)
            } else {
                self.score
            };

            table.store(self, self.game.get_fen(), 0, score, None, TTFlag::Exact);
            return score;
        }

        // Check TT
        if let Some(entry) = table.map.get(&self.game.get_fen()) {
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

        // Check again before generating children
        if !on_search.load(Ordering::SeqCst) {
            return self.score;
        }

        self.get_children(net, table);
        let mut best_score = -i32::MAX;
        let mut best_move: Option<Move> = None;

        for mut child in self.children.clone() {
            // Check before each recursive call
            if !on_search.load(Ordering::SeqCst) {
                break;
            }

            let score = -child.alpha_beta_threaded(
                depth - 1,
                -beta,
                -alpha,
                net,
                table,
                nodes,
                sel_depth,
                on_search,
            );

            if score > best_score {
                best_score = score;
                best_move = child.game.last_move();
            }

            if score >= beta {
                table.store(
                    self,
                    self.game.get_fen(),
                    depth,
                    beta,
                    best_move,
                    TTFlag::LowerBound,
                );
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        if best_move.is_none() {
            let score = if self.game.is_checkmate() {
                99999 + depth
            } else if self.game.is_stalemate() {
                -1000
            } else if self.game.is_draw() {
                -1000
            } else {
                0
            };

            table.store(self, self.game.get_fen(), depth, score, None, TTFlag::Exact);
            return score;
        }

        table.store(
            self,
            self.game.get_fen(),
            depth,
            best_score,
            best_move,
            TTFlag::Exact,
        );

        best_score
    }

    pub fn quiscence_threaded(
        &mut self,
        mut alpha: i32,
        beta: i32,
        net: &Network,
        table: &mut TranspositionTable,
        nodes: &mut usize,
        sel_depth: &mut usize,
        on_search: &Arc<AtomicBool>,
    ) -> i32 {
        // Check if we should stop
        if !on_search.load(Ordering::SeqCst) {
            return self.score;
        }

        *nodes += 1;

        let stand_pat = net.evaluate_accumulator(&self.acc, self.game.side_to_move());

        if stand_pat >= beta {
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        if !on_search.load(Ordering::SeqCst) {
            return alpha;
        }

        self.get_children_captures(net, table);

        if !on_search.load(Ordering::SeqCst) {
            return alpha;
        }

        // Sort captures by MVV-LVA
        self.children.sort_by(|a, b| {
            b.game
                .last_move()
                .unwrap()
                .captured
                .unwrap()
                .piece_type
                .get_value()
                .cmp(
                    &a.game
                        .last_move()
                        .unwrap()
                        .captured
                        .unwrap()
                        .piece_type
                        .get_value(),
                )
        });

        // self.children.sort_by(|&mut a, mut &b| {
        //     // Use MVV-LVA scoring
        //     let a_score = &self.mvv_lva_score(&a.game.last_move().unwrap());
        //     let b_score = &self.mvv_lva_score(&b.game.last_move().unwrap());
        //     b_score.cmp(&a_score)
        // });

        let mut children = std::mem::take(&mut self.children);
        if children.len() > 0 {
            *sel_depth += 1;
        }

        for capture_child in &mut children {
            if !on_search.load(Ordering::SeqCst) {
                break;
            }

            let score = -capture_child
                .quiscence_threaded(-beta, -alpha, net, table, nodes, sel_depth, on_search);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }
}

impl SearchNode {
    fn mvv_lva_score(&self, m: &Move) -> usize {
        // Most Valuable Victim - Least Valuable Attacker
        let victim_value = match m.captured {
            Some(p) => p.piece_type.get_value(),
            None => 0,
        } as usize;
        let attacker_value = match self.game.board.get_piece_at_square(m.from_pos) {
            Some(p) => p.piece_type.get_value(),
            None => 0,
        } as usize;
        (victim_value * 10) - attacker_value
    }

    fn move_ordering_score(&self, m: &Move) -> usize {
        let mut score = 0;

        // Capture moves
        if m.captured.is_some() {
            score += 5000 + self.mvv_lva_score(m);
        }

        // Killer moves (from history)
        // score += self.killer_heuristic(m);

        // History heuristic
        // score += self.history_heuristic(m);

        score
    }
}
