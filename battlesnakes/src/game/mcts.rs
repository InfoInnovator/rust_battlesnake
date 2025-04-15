#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use core::f64;

use crate::{Board, Move, game::simulator::Simulator, game::tree::Node};

const ERROR_MARGIN: f64 = 0.00001;

/// Monte Carlo Tree Search (MCTS) implementation for the Battlesnake game.
pub struct Mcts {
    total_sims: u32,
    root: Node,
    global_node_id: u32,
    current_iteration: u32,
    max_iterations: u32,
}

impl Mcts {
    /// Creates a new instance of Mcts.
    #[must_use]
    pub fn new(max_iterations: u32) -> Self {
        Self {
            global_node_id: 1,
            total_sims: 0,
            root: Node::new(0, None),
            current_iteration: 0,
            max_iterations,
        }
    }

    /// Finds the best child node using the Upper Confidence Bound (UCB) formula.
    ///
    /// It only considers the direct children of the given node.
    /// If there is a single best child, it returns its index.
    /// If there are multiple children with the same UCB value,
    /// it randomly selects one of them.
    fn get_best_ucb_index(node: &Node, total_sims: u32) -> usize {
        let mut candidates = vec![0];

        let mut best_ucb = f64::MIN;
        for (i, child) in node.children.iter().enumerate() {
            let ucb = child.get_ucb(total_sims.into());
            if ucb > best_ucb {
                best_ucb = ucb;
                candidates.clear();
                candidates.push(i);
            } else if f64::abs(
                ucb - node.children[candidates[0] as usize].get_ucb(total_sims.into()),
            ) < ERROR_MARGIN
            {
                candidates.push(i);
            }
        }

        assert!(!candidates.is_empty(), "no candidates found");

        candidates[rand::random_range(0..candidates.len())]
    }

    /// Performs the Monte Carlo Tree Search (MCTS) algorithm to find the best move.
    ///
    /// # Panics
    ///
    /// Panics if the used index could not be converted to usize
    #[must_use]
    pub fn get_move(&mut self, board: &Board, turn: &i32) -> Move {
        while self.current_iteration < self.max_iterations {
            // vec with local indexes of used nodes from top to bottom
            let mut way_back: Vec<usize> = Vec::new();

            let mut sim = Simulator::from_board(board.clone());

            // traverse tree to find best node using ucb formula
            let mut current_node = &mut self.root;
            while !current_node.children.is_empty() {
                // determine best local child
                let best_index = Self::get_best_ucb_index(current_node, self.total_sims);

                // enter the best child
                way_back.push(best_index);
                current_node = &mut current_node.children[best_index];

                // perform the move in the simulator
                sim.make_move(&current_node.current_move.clone().unwrap());
            }

            // expand the child with all reasonable moves
            let possible_moves = sim.get_reasonable_moves();
            for m in &possible_moves {
                let new_node = Node::new(self.global_node_id, Some(m.clone()));
                current_node.add_child(new_node);
                self.global_node_id += 1;
            }

            // choose a random move from new nodes to perform a full playout
            let mut new_current = &mut self.root;
            if possible_moves.is_empty() {
                for i in way_back {
                    new_current.children[i].simulations += 1;
                    new_current = &mut new_current.children[i];
                }
            } else {
                let random_index = rand::random_range(0..possible_moves.len());
                let random_move = possible_moves[random_index].clone();
                way_back.push(random_index);
                sim.make_move(&random_move);

                let turns = sim.simulate_turns(100);
                for i in way_back {
                    new_current.children[i].reward += 10.0 + (0.25 * f64::from(turns));
                    new_current.children[i].simulations += 1;

                    new_current = &mut new_current.children[i];
                }
            }

            self.root.simulations += 1;
            self.total_sims += 1;
            self.current_iteration += 1;
        }

        self.root.save_graph(format!("turn_{turn}").as_str());

        self.get_best_move().unwrap_or_else(|| {
            log::error!("No best move found, returning default move: UP");
            Move::Up
        })
    }

    /// Returns the best move based on the simulation results.
    fn get_best_move(&self) -> Option<Move> {
        let mut best_move = None;
        let mut best_win = f64::MIN;

        for child in &self.root.children {
            let win = child.reward / f64::from(child.simulations);
            if win > best_win {
                best_win = win;
                best_move = Some(child.current_move.clone().unwrap());
            }
        }

        best_move
    }
}
