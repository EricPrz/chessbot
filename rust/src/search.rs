use crate::enums::Colorr::{BLACK, WHITE};
use crate::enums::{PieceType, print_bitboard};
use crate::game::Game;
use crate::moves::Move;
use crate::piece::Piecee;
use nnue_rs::{Accumulator, Board, Network};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rayon::prelude::*;

#[derive(Clone)]
pub struct SearchNode {
    pub parent: Option<Rc<SearchNode>>,
    pub game: Game,
    pub acc: RefCell<Accumulator>,
    pub score: i32,
    pub depth: i32,
}

impl SearchNode {
    pub fn new_root(game: &Game, net: &Network, table: &mut TranspositionTable) -> Self {
        let fen = game.get_fen();
        let score = net.evaluate(game);

        let new_node = SearchNode {
            parent: None,
            game: game.clone(),
            acc: RefCell::new(net.empty_accumulator()),
            score: score,
            depth: 0,
        };

        table.map.insert(fen, TTEntry::new(&new_node));

        new_node
    }

    pub fn get_children(&self, net: &Network, table: &mut TranspositionTable) -> Vec<SearchNode> {
        let mut children = Vec::new();

        for game_child in self.game.get_children() {
            let fen = game_child.get_fen();

            let score: i32 = if let Some(entry) = table.map.get(&fen) {
                entry.score
            } else {
                net.evaluate(&game_child)
            };

            let new_node = SearchNode {
                parent: Some(Rc::new(self.to_owned())),
                game: game_child,
                acc: RefCell::new(net.empty_accumulator()),
                score: score,
                depth: self.depth + 1,
            };

            if table.map.get(&fen).is_none() {
                table.map.insert(fen.clone(), TTEntry::new(&new_node));
            }

            children.push(new_node);
        }

        let is_maxing = self.game.turn == WHITE;
        match is_maxing {
            true => children.sort_by(|a, b| b.score.cmp(&a.score)),
            false => children.sort_by(|a, b| a.score.cmp(&b.score)),
        }

        children
    }

    pub fn get_children_captures(
        &self,
        net: &Network,
        table: &mut TranspositionTable,
    ) -> Vec<SearchNode> {
        let mut children = Vec::new();

        for game_child in self.game.get_children() {
            if let Some(l) = game_child.last_move() {
                if l.captured.is_none() {
                    continue;
                }
            }

            let fen = game_child.get_fen();

            let score: i32 = if let Some(entry) = table.map.get(&fen) {
                entry.score
            } else {
                net.evaluate(&game_child)
            };

            let new_node = SearchNode {
                parent: Some(Rc::new(self.to_owned())),
                game: game_child,
                acc: RefCell::new(net.empty_accumulator()),
                score: score,
                depth: self.depth + 1,
            };

            if table.map.get(&fen).is_none() {
                table.map.insert(fen.clone(), TTEntry::new(&new_node));
            }

            children.push(new_node);
        }

        let is_maxing = self.game.turn == WHITE;
        match is_maxing {
            true => children.sort_by(|a, b| b.score.cmp(&a.score)),
            false => children.sort_by(|a, b| a.score.cmp(&b.score)),
        }

        children
    }

    pub fn quiscence(
        &self,
        mut alpha: i32,
        beta: i32,
        net: &Network,
        table: &mut TranspositionTable,
    ) -> i32 {
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

        // Generate only CAPTURE moves (and maybe checks)
        let captures = self.get_children_captures(net, table); // You'll need this method

        // Sort captures by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        // This helps with pruning
        let mut sorted_captures = captures;
        sorted_captures.sort_by(|a, b| {
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

        for capture_child in sorted_captures {
            // Recursively search captures (with negamax)
            let score = -capture_child.quiscence(-beta, -alpha, net, table);

            if score >= beta {
                return beta; // Beta cut-off
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha // Return the best score found
    }

    pub fn alpha_beta(
        &mut self,
        depth: i32,
        mut alpha: i32,
        beta: i32,
        net: &Network,
        table: &mut TranspositionTable,
    ) -> i32 {
        if depth == 0 {
            let score = if self.game.is_last_move_capture() {
                self.quiscence(alpha, beta, net, table)
            } else {
                self.score
            };

            table.store(self, self.game.get_fen(), 0, score, None, TTFlag::Exact);
            return score;
        }

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

        let moves = self.get_children(net, table);
        let mut best_score = -i32::MAX;
        let mut best_move: Option<Move> = None;

        for mut child in moves {
            let score = -child.alpha_beta(depth - 1, -beta, -alpha, net, table);

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
            let score = if self.game.is_check() {
                -99999 + depth
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

    pub fn iterative_deepening(
        &mut self,
        max_depth: i32,
        net: &Network,
        table: &mut TranspositionTable,
    ) -> Option<Move> {
        let mut best_move = None;
        let mut best_score = -std::i32::MAX;

        for depth in 1..=max_depth {
            // Search to this depth
            let score = self.alpha_beta(depth, -std::i32::MAX, std::i32::MAX, net, table);

            // AFTER searching, get the best move from TT
            if let Some(entry) = table.map.get(&self.game.get_fen()) {
                if entry.depth >= depth {
                    best_score = score;
                    best_move = entry.best_move; // ← THE MOVE!
                    log::info!(
                        "Depth {}: score={}, move={:?}",
                        depth,
                        best_score,
                        best_move
                    );
                }
            }

            //if self.time_is_up() {
            //    break;
            //}
        }

        best_move // Return the best move found
    }

    pub fn iddfs(
        &mut self,
        max_depth: i32,
        net: &Network,
        table: &mut TranspositionTable,
    ) -> Vec<SearchNode> {
        let mut c = Vec::new();
        for depth in 1..=max_depth {
            c = self.dls(depth, net, table);
        }
        return c;
    }

    pub fn dls(
        &mut self,
        depth: i32,
        net: &Network,
        table: &mut TranspositionTable,
    ) -> Vec<SearchNode> {
        if depth == 0 {
            return vec![self.to_owned()];
        }

        let mut results = Vec::new();
        let is_white = self.game.turn == WHITE;
        for mut child in self.get_children(net, table) {
            let mut sub_results = child.dls(depth - 1, net, table);
            results.append(&mut sub_results);
        }

        results
    }
}

#[derive(PartialEq, Eq)]
enum TTFlag {
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
