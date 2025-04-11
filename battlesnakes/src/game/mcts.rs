#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use core::f64;
use std::time::Instant;

use crate::{Board, Move, game::simulator::Simulator, game::tree::Node};

const ERROR_MARGIN: f64 = 0.00001;

/*
TODO
+ display Board inside of the Tree dot file
+ handle food better
*/

pub struct Mcts;

impl Mcts {
    fn get_best_ucb_local_index(node: &Node, num_sims: i32) -> i32 {
        let mut candidates = vec![0];

        let mut best_ucb = f64::MIN;
        for (i, child) in node.children.iter().enumerate() {
            let ucb = child.get_ucb(num_sims);
            if ucb > best_ucb {
                best_ucb = ucb;
                candidates.clear();
                candidates.push(i);
            } else if f64::abs(ucb - node.children[candidates[0] as usize].get_ucb(num_sims))
                < ERROR_MARGIN
            {
                candidates.push(i);
            }
        }

        assert!(!candidates.is_empty(), "no candidates found");

        candidates[rand::random_range(0..candidates.len())]
            .try_into()
            .unwrap()
    }

    /// # Panics
    ///
    /// Panics if the used index could not be converted to usize
    #[must_use]
    pub fn get_move(board: &Board, _turn: &i32) -> Move {
        let start = Instant::now();
        let mut global_node_id = 1;
        let mut num_sims = 0;

        let mut root = Node::new(0, None);
        while start.elapsed().as_millis() < 200 {
            let mut way_back: Vec<usize> = Vec::new(); // vec with local indexes of used nodes from top to bottom
            let mut sim = Simulator::from_board(board.clone());
            // println!("original board:");
            // board.print();

            // traverse tree to find best node using ucb formula
            let mut current_node = &mut root;
            while !current_node.children.is_empty() {
                let best_index = Mcts::get_best_ucb_local_index(current_node, num_sims);

                way_back.push(best_index.try_into().unwrap());
                current_node = &mut current_node.children
                    [<i32 as TryInto<usize>>::try_into(best_index).unwrap()];

                sim.make_move(&current_node.current_move.clone().unwrap());
                // println!(
                //     "board after move: {}",
                //     current_node.current_move.clone().unwrap()
                // );
                // sim.get_board().print();
            }

            // expand with all reasonable moves
            let possible_moves = sim.get_reasonable_moves();
            // println!("possible moves: {:?}", possible_moves);

            for m in &possible_moves {
                let new_node = Node::new(global_node_id, Some(m.clone()));
                current_node.add_child(new_node);
                global_node_id += 1;

                // println!("expanded tree with move: {}", m);
            }

            // choose random move from new nodes
            let mut new_current = &mut root;
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

                let death_cause = sim.simulate_turns(40);
                // println!("Sim result: {:?}\n", death_cause);
                if death_cause.is_some() {
                    for i in way_back {
                        new_current.children[i].simulations += 1;
                        new_current = &mut new_current.children[i];
                    }
                } else {
                    for i in way_back {
                        new_current.children[i].wins += 1;
                        new_current.children[i].simulations += 1;

                        new_current = &mut new_current.children[i];
                    }
                }
            }

            num_sims += 1;
            root.simulations = num_sims;
        }

        // root.save_graph(format!("turn_{turn}").as_str());

        // return move with highest ucb value
        let mut best_move = None;
        let mut best_ucb = f64::MIN;
        for child in &root.children {
            let ucb = child.get_ucb(num_sims);
            if ucb > best_ucb {
                best_ucb = ucb;
                best_move.clone_from(&child.current_move);
            }
        }
        if let Some(best_move) = best_move {
            println!("time after move chosen: {:?}", start.elapsed());
            println!("number of simulations: {num_sims}");
            best_move
        } else {
            // println!("no best move found, returning default move");
            Move::Up
        }
    }
}
